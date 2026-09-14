use super::{EvaluationError, SourceEvaluator};
use iris_runtime::{
    CallableKind, CallableSignature, ComposedType, NominalType, SignatureType, TypeAtom, Value,
};
use iris_syntax::TypeExpression;

impl SourceEvaluator {
    pub(super) fn composed_admits(
        &mut self,
        value: &Value,
        composed: &ComposedType,
    ) -> Result<bool, EvaluationError> {
        let (members, union) = match composed {
            ComposedType::Never => return Ok(false),
            ComposedType::Union(members) => (members, true),
            ComposedType::Intersection(members) => (members, false),
        };
        for member in members {
            let admitted = match member {
                TypeAtom::Nominal(class, arguments) => {
                    self.type_test(value, &Value::Type(*class, arguments.clone()))?
                        == Value::Bool(true)
                }
                TypeAtom::NonNil => !matches!(value, Value::Nil),
                TypeAtom::Contract(contract, arguments) => {
                    self.closed_contract_admits(value, (*contract, arguments))?
                }
                TypeAtom::Iteration(_) => false,
                TypeAtom::Union(members) => {
                    self.composed_admits(value, &ComposedType::Union(members.clone()))?
                }
                TypeAtom::Intersection(members) => {
                    self.composed_admits(value, &ComposedType::Intersection(members.clone()))?
                }
            };
            if admitted == union {
                return Ok(union);
            }
        }
        Ok(!union)
    }

    pub(super) fn wrapper_type_value(
        &mut self,
        annotation: &TypeExpression,
    ) -> Result<Value, EvaluationError> {
        self.reify_type(annotation)
    }

    pub(super) fn intern_source_callable(
        &mut self,
        name: &str,
        arguments: &[TypeExpression],
    ) -> Result<Value, EvaluationError> {
        let [TypeExpression::Function { parameters, result }] = arguments else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        let parameters = parameters
            .iter()
            .map(|parameter| self.signature_type(parameter))
            .collect::<Result<Vec<_>, _>>()?;
        let result = self.signature_type(result)?;
        let signature = CallableSignature::new(parameters, result);
        let registry = self.runtime.registry_mut();
        match name {
            "Block" => registry.intern_block_alias(signature),
            "Closure" => registry.intern_callable_type(CallableKind::Closure, signature),
            "BoundMethod" => registry.intern_callable_type(CallableKind::BoundMethod, signature),
            _ => return Err(EvaluationError::UnsupportedConstruct),
        }
        .map_err(|_| EvaluationError::UnsupportedConstruct)
    }

    pub(super) fn signature_type(
        &mut self,
        annotation: &TypeExpression,
    ) -> Result<SignatureType, EvaluationError> {
        match self.reify_type(annotation)? {
            Value::Type(class, arguments) => {
                Ok(SignatureType::Nominal(NominalType::new(class, arguments)))
            }
            Value::ComposedType(composed) => Ok(SignatureType::Composed(composed)),
            _ => Err(EvaluationError::UnsupportedConstruct),
        }
    }

    pub(super) fn closure_type(
        &mut self,
        identity: iris_runtime::ObjectId,
    ) -> Result<Value, EvaluationError> {
        if let Some(next) = self.wrapper_next.get(&identity) {
            return self.reify_type(&super::wrapper_chain::next_annotation(next.is_async()));
        }
        let record = self
            .closures
            .get(&identity)
            .ok_or(EvaluationError::UnsupportedConstruct)?;
        if super::body_yields(&record.body) {
            return Err(EvaluationError::UnsupportedConstruct);
        }
        let context = record.lexical_context.clone();
        let parameters = record
            .full_parameters
            .iter()
            .map(|parameter| {
                parameter
                    .annotation
                    .clone()
                    .unwrap_or(TypeExpression::Name("Object".into()))
            })
            .collect();
        let result = record
            .return_type
            .clone()
            .unwrap_or(TypeExpression::Name("Object".into()));
        let result = if record.is_async {
            TypeExpression::Generic {
                name: "Task".into(),
                arguments: vec![result],
            }
        } else {
            result
        };
        let previous = self.wrapper_context();
        self.restore_wrapper_context(context);
        let reified = self.intern_source_callable(
            "Closure",
            &[TypeExpression::Function {
                parameters,
                result: Box::new(result),
            }],
        );
        self.restore_wrapper_context(previous);
        reified
    }

    pub(super) fn wrapper_annotation_admits(
        &mut self,
        value: &Value,
        annotation: &TypeExpression,
    ) -> Result<bool, EvaluationError> {
        match annotation {
            TypeExpression::Generic { name, .. }
                if matches!(name.as_str(), "Closure" | "BoundMethod" | "Block") =>
            {
                let expected = self.reify_type(annotation)?;
                self.type_test(value, &expected)
                    .map(|value| value == Value::Bool(true))
            }
            TypeExpression::Union(members) => {
                for member in members {
                    if self.wrapper_annotation_admits(value, member)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            TypeExpression::Intersection(members) => {
                for member in members {
                    if !self.wrapper_annotation_admits(value, member)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            _ => self.annotation_admits(value, annotation),
        }
    }

    pub(super) fn wrapper_accepts(&mut self, annotation: &Value, value: &Value) -> bool {
        self.type_test(value, annotation) == Ok(Value::Bool(true))
    }
}
