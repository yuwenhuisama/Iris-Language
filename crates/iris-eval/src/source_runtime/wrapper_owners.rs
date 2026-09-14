use super::WrapperChain;
use crate::source_runtime::{EvaluationError, SourceEvaluator};
use iris_runtime::{MethodOwner, NominalType, Value};
use iris_syntax::TypeExpression;

impl SourceEvaluator {
    pub(in crate::source_runtime) fn invoke_closed_initializer(
        &mut self,
        method: iris_runtime::Method,
        receiver: iris_runtime::ObjectId,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        let class = self
            .runtime
            .class_of(receiver)
            .map_err(EvaluationError::Construction)?;
        let owner_arguments = self
            .runtime
            .type_arguments_of(receiver)
            .map_err(EvaluationError::Construction)?;
        let bindings = self
            .wrapper_owner_parameters
            .get(&class)
            .into_iter()
            .flatten()
            .zip(owner_arguments)
            .map(|(name, argument)| Ok((name.clone(), self.wrapper_nominal_annotation(argument)?)))
            .collect::<Result<Vec<_>, EvaluationError>>()?;
        let previous = self.method_type_bindings.clone();
        self.method_type_bindings.extend(bindings);
        let result = self.invoke_method(method, Value::Object(receiver), arguments);
        self.method_type_bindings = previous;
        result
    }

    pub(in crate::source_runtime) fn open_owner_method_metadata(
        &mut self,
        class: iris_runtime::ClassId,
        method: &iris_syntax::MethodDeclaration,
    ) -> Result<Value, EvaluationError> {
        let mut metadata = method.clone();
        let parameters = self
            .wrapper_owner_parameters
            .get(&class)
            .ok_or(EvaluationError::UnsupportedConstruct)?;
        metadata.type_parameters.extend(parameters.iter().cloned());
        self.owned_method_metadata(Value::Class(class), &metadata)
    }

    pub(in crate::source_runtime) fn wrapper_owner_arguments(
        &self,
        chain: &WrapperChain,
        receiver: &Value,
    ) -> Result<Vec<NominalType>, EvaluationError> {
        let MethodOwner::Class(class) = chain.original.owner() else {
            return Ok(Vec::new());
        };
        let Some(parameters) = self.wrapper_owner_parameters.get(&class) else {
            return Ok(Vec::new());
        };
        if parameters.is_empty() {
            return Ok(Vec::new());
        }
        let Value::Object(object) = receiver else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        if self
            .runtime
            .class_of(*object)
            .map_err(EvaluationError::Construction)?
            != class
        {
            return Err(EvaluationError::UnsupportedConstruct);
        }
        let arguments = self
            .runtime
            .type_arguments_of(*object)
            .map_err(EvaluationError::Construction)?;
        if arguments.len() != parameters.len() {
            return Err(EvaluationError::TypeContractError);
        }
        Ok(arguments.to_vec())
    }

    pub(in crate::source_runtime) fn bind_wrapper_owner(
        &mut self,
        chain: &WrapperChain,
        arguments: &[NominalType],
    ) -> Result<(), EvaluationError> {
        let MethodOwner::Class(class) = chain.original.owner() else {
            return Ok(());
        };
        let parameters = self
            .wrapper_owner_parameters
            .get(&class)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let bindings = parameters
            .iter()
            .zip(arguments)
            .filter(|(name, _)| !chain.declaration.type_parameters.contains(name))
            .map(|(name, argument)| Ok((name.clone(), self.wrapper_nominal_annotation(argument)?)))
            .collect::<Result<Vec<_>, EvaluationError>>()?;
        self.method_type_bindings.extend(bindings);
        Ok(())
    }

    pub(in crate::source_runtime) fn wrapper_nominal_annotation(
        &self,
        nominal: &NominalType,
    ) -> Result<TypeExpression, EvaluationError> {
        let name = [
            "Object", "Nil", "Bool", "Integer", "Float32", "Float64", "String",
        ]
        .into_iter()
        .find(|name| self.class_name(name) == Ok(Some(nominal.class())))
        .map_or_else(|| self.class_display_name(nominal.class()), str::to_owned);
        if self.class_name(&name)? != Some(nominal.class()) {
            return Err(EvaluationError::UnsupportedConstruct);
        }
        if nominal.arguments().is_empty() {
            Ok(TypeExpression::Name(name))
        } else {
            Ok(TypeExpression::Generic {
                name,
                arguments: nominal
                    .arguments()
                    .iter()
                    .map(|argument| self.wrapper_nominal_annotation(argument))
                    .collect::<Result<Vec<_>, _>>()?,
            })
        }
    }
}
