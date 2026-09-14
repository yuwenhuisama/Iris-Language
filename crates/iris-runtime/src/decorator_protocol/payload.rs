use super::changes::{snapshot_array, snapshot_hash};
use super::{ArgumentChanges, ArgumentError, ChangeField, InvocationSignature, ParameterCategory};
use crate::{ArrayRef, HashRef, Value};

#[derive(Clone, Debug, PartialEq)]
pub struct InvocationPayload {
    positional: Vec<(String, Value)>,
    keywords: Vec<(String, Value)>,
    rest: Vec<Value>,
    keyword_rest: Vec<(String, Value)>,
    block: Value,
    replaced: Vec<String>,
}

impl InvocationPayload {
    /// Snapshots already-bound parameter cells; this never executes a default.
    pub fn from_bindings(
        signature: &InvocationSignature,
        bindings: &[Value],
        accepts: impl FnMut(&Value, &Value) -> bool,
    ) -> Result<Self, ArgumentError> {
        if bindings.len() != signature.parameters().len() {
            return Err(ArgumentError::BindingCount {
                expected: signature.parameters().len(),
                actual: bindings.len(),
            });
        }
        let mut payload = Self {
            positional: Vec::new(),
            keywords: Vec::new(),
            rest: Vec::new(),
            keyword_rest: Vec::new(),
            block: Value::Nil,
            replaced: Vec::new(),
        };
        for (parameter, value) in signature.parameters().iter().zip(bindings) {
            match parameter.category() {
                ParameterCategory::Positional => payload
                    .positional
                    .push((parameter.name().into(), value.clone())),
                ParameterCategory::Keyword => payload
                    .keywords
                    .push((parameter.name().into(), value.clone())),
                ParameterCategory::Rest => payload.rest = snapshot_array(value)?,
                ParameterCategory::KeywordRest => {
                    payload.keyword_rest = snapshot_hash(value, ChangeField::KeywordRest)?
                }
                ParameterCategory::Block => payload.block = value.clone(),
            }
        }
        payload.validate(signature, accepts)?;
        Ok(payload)
    }

    pub fn positional(&self) -> &[(String, Value)] {
        &self.positional
    }
    pub fn keywords(&self) -> &[(String, Value)] {
        &self.keywords
    }
    pub fn rest(&self) -> &[Value] {
        &self.rest
    }
    pub fn keyword_rest(&self) -> &[(String, Value)] {
        &self.keyword_rest
    }
    pub const fn block(&self) -> &Value {
        &self.block
    }
    pub fn replaced(&self) -> &[String] {
        &self.replaced
    }

    pub fn patch(
        &self,
        signature: &InvocationSignature,
        changes: &ArgumentChanges,
        accepts: impl FnMut(&Value, &Value) -> bool,
    ) -> Result<Self, ArgumentError> {
        let mut next = self.clone();
        for (entries, category, target) in [
            (
                changes.positional(),
                ParameterCategory::Positional,
                &mut next.positional,
            ),
            (
                changes.keywords(),
                ParameterCategory::Keyword,
                &mut next.keywords,
            ),
        ] {
            for (name, value) in entries.into_iter().flatten() {
                let parameter = signature
                    .parameter(name)
                    .ok_or_else(|| ArgumentError::UnknownName(name.clone()))?;
                if parameter.category() != category {
                    return Err(ArgumentError::WrongChannel {
                        name: name.clone(),
                        channel: category,
                    });
                }
                let entry = target
                    .iter_mut()
                    .find(|(held, _)| held == name)
                    .ok_or_else(|| ArgumentError::MissingBinding(name.clone()))?;
                entry.1 = value.clone();
                next.replaced.push(name.clone());
            }
        }
        for (supplied, category) in [
            (changes.rest().is_some(), ParameterCategory::Rest),
            (
                changes.keyword_rest().is_some(),
                ParameterCategory::KeywordRest,
            ),
            (changes.block().is_some(), ParameterCategory::Block),
        ] {
            if supplied {
                let parameter = signature
                    .channel(category)
                    .ok_or(ArgumentError::MissingChannel(category))?;
                next.replaced.push(parameter.name().into());
            }
        }
        if let Some(rest) = changes.rest() {
            next.rest = rest.to_vec();
        }
        if let Some(rest) = changes.keyword_rest() {
            next.keyword_rest = rest.to_vec();
        }
        if let Some(block) = changes.block() {
            next.block = block.clone();
        }
        next.validate(signature, accepts)?;
        next.replaced = signature
            .parameters()
            .iter()
            .filter(|parameter| next.replaced.iter().any(|name| name == parameter.name()))
            .map(|parameter| parameter.name().into())
            .collect();
        Ok(next)
    }

    /// Allocates fresh rest containers for each inner execution's parameter cells.
    pub fn bindings(&self, signature: &InvocationSignature) -> Result<Vec<Value>, ArgumentError> {
        signature
            .parameters()
            .iter()
            .map(|parameter| {
                let value = match parameter.category() {
                    ParameterCategory::Positional => self
                        .positional
                        .iter()
                        .find(|(name, _)| name == parameter.name())
                        .map(|(_, value)| value.clone()),
                    ParameterCategory::Keyword => self
                        .keywords
                        .iter()
                        .find(|(name, _)| name == parameter.name())
                        .map(|(_, value)| value.clone()),
                    ParameterCategory::Rest => Some(Value::Array(ArrayRef::new(self.rest.clone()))),
                    ParameterCategory::KeywordRest => Some(Value::Hash(HashRef::new(
                        self.keyword_rest
                            .iter()
                            .map(|(name, value)| (Value::Symbol(name.clone()), value.clone()))
                            .collect(),
                    ))),
                    ParameterCategory::Block => Some(self.block.clone()),
                };
                value.ok_or_else(|| ArgumentError::MissingBinding(parameter.name().into()))
            })
            .collect()
    }

    pub fn visit_values<'a>(&'a self, visitor: &mut impl FnMut(&'a Value)) {
        for (_, value) in self
            .positional
            .iter()
            .chain(&self.keywords)
            .chain(&self.keyword_rest)
        {
            visitor(value);
        }
        for value in &self.rest {
            visitor(value);
        }
        visitor(&self.block);
    }
}
