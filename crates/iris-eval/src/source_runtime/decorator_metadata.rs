use super::{EvaluationError, SourceEvaluator};
use iris_runtime::{ClassId, Value};
use iris_syntax::MethodDeclaration;

impl SourceEvaluator {
    pub(super) fn class_decorator_metadata(
        &mut self,
        class: ClassId,
    ) -> Result<Value, EvaluationError> {
        let name = self
            .origin_names
            .get(&class)
            .cloned()
            .or_else(|| {
                self.names
                    .iter()
                    .filter(|(_, binding)| binding.value() == Value::Class(class))
                    .map(|(name, _)| name.clone())
                    .min()
            })
            .ok_or(EvaluationError::NameError)?;
        let mut selectors = self
            .selectors
            .iter()
            .map(|(name, selector)| (name.clone(), *selector))
            .collect::<Vec<_>>();
        selectors.sort_by(|left, right| left.0.cmp(&right.0));
        let methods = selectors
            .into_iter()
            .filter_map(|(_, selector)| self.candidate_method_declaration(class, selector))
            .filter(|method| method.visibility == iris_syntax::Visibility::Public)
            .cloned()
            .collect::<Vec<_>>();
        let methods = methods
            .iter()
            .map(|method| self.method_decorator_metadata(class, method))
            .collect::<Result<Vec<_>, _>>()?;
        let (superclass, spine, revision_number, capabilities, modules, mro) = if let Ok(origin) =
            self.runtime.registry().staged_origin(class)
        {
            let capabilities = [
                iris_runtime::Capability::MethodSet,
                iris_runtime::Capability::MethodBody,
                iris_runtime::Capability::PropertySet,
                iris_runtime::Capability::PropertyBody,
                iris_runtime::Capability::Modules,
                iris_runtime::Capability::Superclass,
                iris_runtime::Capability::Subclass,
                iris_runtime::Capability::Shape,
                iris_runtime::Capability::ClassStateSet,
                iris_runtime::Capability::ClassStateWrite,
                iris_runtime::Capability::InstanceState,
                iris_runtime::Capability::Native,
            ]
            .into_iter()
            .filter(|capability| {
                self.runtime
                    .registry()
                    .require_candidate_meta_capability(class, *capability)
                    .is_err()
            })
            .map(|capability| Value::Symbol(super::capability_name(capability).into()))
            .collect();
            (
                origin
                    .runtime_superclass()
                    .map_or(Value::Nil, |owner| Value::Type(owner, Vec::new())),
                Value::Integer(origin.static_spine().identity().into()),
                Value::Nil,
                Value::ReadonlyArray(capabilities),
                Value::ReadonlyArray(
                    origin
                        .composition_edges()
                        .iter()
                        .map(|edge| self.module_symbol(edge.module()))
                        .collect(),
                ),
                Value::ReadonlyArray(
                    origin
                        .mro()
                        .iter()
                        .map(|entry| match entry {
                            iris_runtime::MroEntry::Class(owner) => Value::Type(*owner, Vec::new()),
                            iris_runtime::MroEntry::Module(module) => self.module_symbol(*module),
                        })
                        .collect(),
                ),
            )
        } else {
            let revision = self
                .runtime
                .registry()
                .active(class)
                .map_err(EvaluationError::Class)?;
            let superclass = revision
                .runtime_superclass()
                .map_or(Value::Nil, |class| Value::Type(class, Vec::new()));
            let spine = Value::Integer(revision.static_spine().identity().into());
            let revision_number = Value::Integer(revision.number().into());
            let capabilities = Value::ReadonlyArray(
                revision
                    .meta_capabilities()
                    .denied()
                    .into_iter()
                    .map(|capability| Value::Symbol(super::capability_name(capability).into()))
                    .collect(),
            );
            let modules = Value::ReadonlyArray(
                revision
                    .modules()
                    .iter()
                    .map(|module| self.module_symbol(*module))
                    .collect(),
            );
            (
                superclass,
                spine,
                revision_number,
                capabilities,
                modules,
                self.ancestors(class)?,
            )
        };
        let contracts = Value::ReadonlyArray(
            self.class_contracts
                .get(&class)
                .into_iter()
                .flatten()
                .map(|contract| Value::Contract(*contract, Vec::new()))
                .collect(),
        );
        let mro = match mro {
            Value::Array(array) => Value::ReadonlyArray(
                array
                    .elements()
                    .into_iter()
                    .map(|value| match value {
                        Value::Class(class) => Value::Type(class, Vec::new()),
                        other => other,
                    })
                    .collect(),
            ),
            other => other,
        };
        let properties = self.class_properties(class)?;
        let properties = match properties {
            Value::Array(array) => Value::ReadonlyArray(array.elements()),
            other => other,
        };
        self.metadata_record(vec![
            ("name", Value::Symbol(name)),
            ("package", Value::Text(self.package.clone())),
            ("type", Value::Type(class, Vec::new())),
            ("methods", Value::ReadonlyArray(methods)),
            ("static_spine", spine),
            ("active_revision", revision_number),
            ("runtime_superclass", superclass),
            ("mro", mro),
            ("contracts", contracts),
            ("modules", modules),
            ("properties", properties),
            ("meta_capabilities", capabilities),
        ])
    }

    pub(super) fn owned_method_metadata(
        &mut self,
        owner: Value,
        method: &MethodDeclaration,
    ) -> Result<Value, EvaluationError> {
        let parameters = method
            .parameters
            .iter()
            .map(|parameter| {
                let category = match parameter.category {
                    iris_syntax::ParameterCategory::Positional => "positional",
                    iris_syntax::ParameterCategory::Keyword => "keyword",
                    iris_syntax::ParameterCategory::Rest => "rest",
                    iris_syntax::ParameterCategory::KeywordRest => "keyword_rest",
                    iris_syntax::ParameterCategory::Block => "block",
                };
                let annotation = match &parameter.annotation {
                    Some(annotation) if !method.type_parameters.is_empty() => {
                        self.open_annotation_metadata(annotation, &method.type_parameters)?
                    }
                    Some(annotation) => self.wrapper_type_value(annotation)?,
                    None => self
                        .snapshot_types
                        .as_ref()
                        .ok_or(EvaluationError::UnsupportedConstruct)?
                        .object
                        .clone(),
                };
                self.metadata_record(vec![
                    ("name", Value::Symbol(parameter.name.clone())),
                    ("category", Value::Symbol(category.into())),
                    ("type", annotation),
                    ("optional", Value::Bool(parameter.default.is_some())),
                ])
            })
            .collect::<Result<Vec<_>, EvaluationError>>()?;
        let signature = self.metadata_record(vec![
            ("parameters", Value::ReadonlyArray(parameters)),
            ("is_async", Value::Bool(method.is_async)),
        ])?;
        let visibility = match method.visibility {
            iris_syntax::Visibility::Public => "public",
            iris_syntax::Visibility::Protected => "protected",
            iris_syntax::Visibility::Private => "private",
        };
        self.metadata_record(vec![
            ("selector", Value::Symbol(method.selector.clone())),
            ("owner", owner),
            ("visibility", Value::Symbol(visibility.into())),
            ("signature", signature),
            ("source", Value::Text(self.source_path.clone())),
            ("package", Value::Text(self.package.clone())),
        ])
    }

    pub(super) fn decorator_metadata_send(
        &self,
        receiver: &Value,
        selector: &str,
        arguments: &[Value],
    ) -> Result<Option<Value>, EvaluationError> {
        let Value::ImmutableHash(record) = receiver else {
            return Ok(None);
        };
        if !self
            .decorator_metadata
            .iter()
            .any(|known| known.same(record))
        {
            return Ok(None);
        }
        if selector.ends_with('=') && selector != "==" {
            return Err(EvaluationError::ReadonlyMutation);
        }
        match record.get(&Value::Symbol(selector.into())) {
            Some(value) if arguments.is_empty() => Ok(Some(value.clone())),
            Some(_) => Err(EvaluationError::ArgumentError),
            None => Ok(None),
        }
    }
}
