use super::{EvaluationError, SourceEvaluator};
use iris_runtime::{CallableKind, ClassId, SignatureType, Value};

impl SourceEvaluator {
    pub(super) fn callable_type_send(
        &mut self,
        identity: ClassId,
        selector: &str,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        let descriptor = self
            .runtime
            .registry()
            .callable_type(identity)
            .ok_or(EvaluationError::UnsupportedConstruct)?;
        let name = match descriptor.kind() {
            CallableKind::Closure => "Closure",
            CallableKind::BoundMethod => "BoundMethod",
        };
        match (selector, arguments) {
            ("==", [other]) => Ok(Value::Bool(*other == Value::Type(identity, Vec::new()))),
            ("!=", [other]) => Ok(Value::Bool(*other != Value::Type(identity, Vec::new()))),
            ("type", []) => Ok(Value::Type(identity, Vec::new())),
            ("name" | "to_string", []) => Ok(Value::Text(name.into())),
            ("kind", []) => Ok(Value::Symbol(name.to_ascii_lowercase())),
            ("package", []) => Ok(Value::Symbol("Kernel".into())),
            ("parameters", []) => Ok(Value::ReadonlyArray(
                descriptor
                    .signature()
                    .parameters()
                    .iter()
                    .map(signature_value)
                    .collect(),
            )),
            ("result", []) => Ok(signature_value(descriptor.signature().result())),
            ("subtype?", [Value::Type(target, _)]) => self.subtype(identity, *target),
            ("assignable?", [Value::Type(source, _)]) => self.subtype(*source, identity),
            _ => Err(EvaluationError::UnsupportedConstruct),
        }
    }
}

fn signature_value(signature: &SignatureType) -> Value {
    match signature {
        SignatureType::Nominal(nominal) => {
            Value::Type(nominal.class(), nominal.arguments().to_vec())
        }
        SignatureType::Composed(composed) => Value::ComposedType(composed.clone()),
    }
}
