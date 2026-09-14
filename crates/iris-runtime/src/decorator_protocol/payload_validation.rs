use super::{ArgumentError, InvocationPayload, InvocationSignature, ParameterCategory};
use crate::Value;

impl InvocationPayload {
    pub fn validate(
        &self,
        signature: &InvocationSignature,
        mut accepts: impl FnMut(&Value, &Value) -> bool,
    ) -> Result<(), ArgumentError> {
        for (entries, category) in [
            (self.positional(), ParameterCategory::Positional),
            (self.keywords(), ParameterCategory::Keyword),
        ] {
            for (name, _) in entries {
                let parameter = signature
                    .parameter(name)
                    .ok_or_else(|| ArgumentError::UnknownName(name.clone()))?;
                if parameter.category() != category {
                    return Err(ArgumentError::WrongChannel {
                        name: name.clone(),
                        channel: category,
                    });
                }
            }
        }
        for (present, category) in [
            (!self.rest().is_empty(), ParameterCategory::Rest),
            (
                !self.keyword_rest().is_empty(),
                ParameterCategory::KeywordRest,
            ),
            (
                !matches!(self.block(), Value::Nil),
                ParameterCategory::Block,
            ),
        ] {
            if present && signature.channel(category).is_none() {
                return Err(ArgumentError::MissingChannel(category));
            }
        }
        for (name, _) in self.keyword_rest() {
            if let Some(parameter) = signature.parameter(name) {
                match parameter.category() {
                    ParameterCategory::Positional | ParameterCategory::Keyword => {
                        return Err(ArgumentError::WrongChannel {
                            name: name.clone(),
                            channel: ParameterCategory::KeywordRest,
                        });
                    }
                    ParameterCategory::Rest
                    | ParameterCategory::KeywordRest
                    | ParameterCategory::Block => {}
                }
            }
        }
        for parameter in signature.parameters() {
            let check = |value: &Value, accepts: &mut dyn FnMut(&Value, &Value) -> bool| {
                if accepts(parameter.parameter_type(), value) {
                    Ok(())
                } else {
                    Err(ArgumentError::TypeMismatch(parameter.name().into()))
                }
            };
            match parameter.category() {
                ParameterCategory::Positional | ParameterCategory::Keyword => {
                    let value = self
                        .positional()
                        .iter()
                        .chain(self.keywords())
                        .find(|(name, _)| name == parameter.name())
                        .map(|(_, value)| value)
                        .ok_or_else(|| ArgumentError::MissingBinding(parameter.name().into()))?;
                    check(value, &mut accepts)?;
                }
                ParameterCategory::Rest => {
                    for value in self.rest() {
                        check(value, &mut accepts)?;
                    }
                }
                ParameterCategory::KeywordRest => {
                    for (_, value) in self.keyword_rest() {
                        check(value, &mut accepts)?;
                    }
                }
                ParameterCategory::Block => match self.block() {
                    Value::Nil if parameter.optional() => {}
                    Value::Nil => return Err(ArgumentError::TypeMismatch(parameter.name().into())),
                    value => check(value, &mut accepts)?,
                },
            }
        }
        Ok(())
    }
}
