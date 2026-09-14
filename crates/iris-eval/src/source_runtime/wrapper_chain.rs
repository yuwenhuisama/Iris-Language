use super::{EvaluationError, SourceEvaluator};
use iris_runtime::{Capability, ClassId, Method, ObjectId, Value};
use iris_syntax::{MethodDeclaration, MethodKind, ParameterCategory, TypeExpression};
use std::rc::Rc;

#[path = "wrapper_materialization.rs"]
mod materialization;
use materialization::CanonicalMethod;

#[path = "wrapper_owners.rs"]
mod owners;

pub(super) struct WrapperChain {
    pub original: Method,
    pub declaration: MethodDeclaration,
    pub wrappers: Vec<ObjectId>,
    pub qualifier: Option<iris_runtime::ContractId>,
    pub canonical: Option<Rc<CanonicalMethod>>,
}

impl super::gc_roots::TraceRoots for WrapperChain {
    fn trace_roots(&self, roots: &mut Vec<Value>) {
        roots.push(Value::Method(self.original));
        roots.extend(self.wrappers.iter().copied().map(Value::Closure));
        if let Some(canonical) = &self.canonical {
            canonical.trace_roots(roots);
        }
    }
}

pub(super) fn next_annotation(is_async: bool) -> TypeExpression {
    TypeExpression::Generic {
        name: "Closure".into(),
        arguments: vec![TypeExpression::Function {
            parameters: vec![TypeExpression::Name("ArgumentChanges".into())],
            result: Box::new(if is_async {
                TypeExpression::Generic {
                    name: "Task".into(),
                    arguments: vec![TypeExpression::Name("Object".into())],
                }
            } else {
                TypeExpression::Name("Object".into())
            }),
        }],
    }
}

impl SourceEvaluator {
    pub(super) fn wrapper_type_error(&mut self) -> EvaluationError {
        self.core_boundary_error(EvaluationError::Runtime(iris_runtime::KernelError::Type))
    }

    pub(super) fn admit_wrapper(
        &mut self,
        value: &Value,
        is_async: bool,
    ) -> Result<ObjectId, EvaluationError> {
        let Value::Closure(identity) = value else {
            return Err(self.wrapper_type_error());
        };
        let valid = self.closures.get(identity).is_some_and(|record| {
            let [invocation, next] = record.full_parameters.as_slice() else {
                return false;
            };
            record.is_async == is_async
                && !super::body_yields(&record.body)
                && record.return_type == Some(TypeExpression::Name("Object".into()))
                && invocation.annotation == Some(TypeExpression::Name("Invocation".into()))
                && next.annotation == Some(next_annotation(is_async))
                && [invocation, next].iter().all(|parameter| {
                    parameter.category == ParameterCategory::Positional
                        && parameter.default.is_none()
                })
        });
        if valid {
            Ok(*identity)
        } else {
            Err(self.wrapper_type_error())
        }
    }

    pub(super) fn install_wrapper_chain(
        &mut self,
        class: ClassId,
        declaration: &MethodDeclaration,
        wrappers: Vec<ObjectId>,
    ) -> Result<(), EvaluationError> {
        if (!declaration.type_parameters.is_empty()
            && !matches!(declaration.kind, MethodKind::Class | MethodKind::Instance))
            || !matches!(
                declaration.kind,
                MethodKind::Instance | MethodKind::Property | MethodKind::Class
            )
            || (self.generic_definitions.contains(&class)
                && declaration.kind != MethodKind::Instance)
            || declaration
                .body
                .as_ref()
                .is_some_and(|body| super::body_yields(body))
        {
            return Err(EvaluationError::UnsupportedConstruct);
        }
        let qualified = self.qualified_wrapper_target(class, declaration)?;
        let selector = self.selector(&declaration.selector);
        let original = match &qualified {
            Some((_, original, _)) => *original,
            None if declaration.kind == MethodKind::Class => *self
                .singleton_identities
                .get(&(class, selector))
                .ok_or(EvaluationError::UnsupportedConstruct)?,
            None => self
                .runtime
                .registry()
                .staged_method(class, selector)
                .and_then(|identity| self.runtime.registry().method_by_id(identity))
                .ok_or(EvaluationError::UnsupportedConstruct)?,
        };
        let published = match &qualified {
            Some((contract, _, _)) => self.publish_qualified_wrapper(*contract, original)?,
            None => match declaration.kind {
                MethodKind::Property => {
                    self.runtime
                        .registry()
                        .require_candidate_meta_capability(class, Capability::PropertyBody)
                        .map_err(EvaluationError::Class)?;
                    let published = self
                        .runtime
                        .registry_mut()
                        .publish_origin_method(
                            class,
                            selector,
                            original.body(),
                            original.visibility(),
                        )
                        .map_err(EvaluationError::Class)?;
                    self.property_methods.insert(published.id(), true);
                    published
                }
                MethodKind::Instance => self
                    .runtime
                    .registry_mut()
                    .publish_candidate_decorated_method(
                        class,
                        selector,
                        original.body(),
                        original.visibility(),
                        Vec::new(),
                    )
                    .map_err(EvaluationError::Class)?,
                MethodKind::Class => {
                    self.runtime
                        .registry()
                        .require_candidate_meta_capability(class, Capability::MethodBody)
                        .map_err(EvaluationError::Class)?;
                    let published = self
                        .runtime
                        .registry_mut()
                        .publish_singleton_method(
                            class,
                            selector,
                            original.body(),
                            original.visibility(),
                        )
                        .map_err(EvaluationError::Class)?;
                    self.singleton_identities
                        .insert((class, selector), published);
                    published
                }
                MethodKind::Module => {
                    return Err(EvaluationError::UnsupportedConstruct);
                }
            },
        };
        let (original, wrappers) = match self.wrapper_chains.get(&original.id()) {
            Some(chain) => (
                chain.original,
                chain.wrappers.iter().copied().chain(wrappers).collect(),
            ),
            None => (original, wrappers),
        };
        let canonical = (!declaration.type_parameters.is_empty()
            || self.generic_definitions.contains(&class))
        .then(|| Rc::new(CanonicalMethod::new(published, self.wrapper_context())));
        self.wrapper_chains.insert(
            published.id(),
            Rc::new(WrapperChain {
                original,
                declaration: qualified
                    .as_ref()
                    .map_or_else(|| declaration.clone(), |(_, _, selected)| selected.clone()),
                wrappers,
                qualifier: qualified.map(|(contract, _, _)| contract),
                canonical,
            }),
        );
        Ok(())
    }
}
