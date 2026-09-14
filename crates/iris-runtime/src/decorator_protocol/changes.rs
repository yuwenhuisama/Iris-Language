use super::ArgumentError;
use crate::Value;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChangeField {
    Positional,
    Keywords,
    Rest,
    KeywordRest,
    Block,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ArgumentChanges {
    positional: Option<Vec<(String, Value)>>,
    keywords: Option<Vec<(String, Value)>>,
    rest: Option<Vec<Value>>,
    keyword_rest: Option<Vec<(String, Value)>>,
    block: Option<Value>,
}

impl ArgumentChanges {
    pub fn empty() -> Self {
        Self::default()
    }

    /// Parses evaluated constructor keywords, preserving absence independently of nil.
    pub fn parse(keywords: &[(String, Value)]) -> Result<Self, ArgumentError> {
        let mut seen = std::collections::HashSet::new();
        for (name, _) in keywords {
            if !matches!(
                name.as_str(),
                "positional" | "keywords" | "rest" | "keyword_rest" | "block"
            ) {
                return Err(ArgumentError::UnknownKeyword(name.clone()));
            }
            if !seen.insert(name) {
                return Err(ArgumentError::DuplicateKeyword(name.clone()));
            }
        }
        let mut changes = Self::empty();
        for (name, value) in keywords {
            match name.as_str() {
                "positional" => {
                    changes.positional = Some(snapshot_hash(value, ChangeField::Positional)?)
                }
                "keywords" => changes.keywords = Some(snapshot_hash(value, ChangeField::Keywords)?),
                "rest" => changes.rest = Some(snapshot_array(value)?),
                "keyword_rest" => {
                    changes.keyword_rest = Some(snapshot_hash(value, ChangeField::KeywordRest)?)
                }
                "block" => changes.block = Some(value.clone()),
                _ => return Err(ArgumentError::UnknownKeyword(name.clone())),
            }
        }
        Ok(changes)
    }

    pub fn positional(&self) -> Option<&[(String, Value)]> {
        self.positional.as_deref()
    }
    pub fn keywords(&self) -> Option<&[(String, Value)]> {
        self.keywords.as_deref()
    }
    pub fn rest(&self) -> Option<&[Value]> {
        self.rest.as_deref()
    }
    pub fn keyword_rest(&self) -> Option<&[(String, Value)]> {
        self.keyword_rest.as_deref()
    }
    pub const fn block(&self) -> Option<&Value> {
        self.block.as_ref()
    }

    pub fn visit_values<'a>(&'a self, visitor: &mut impl FnMut(&'a Value)) {
        for entries in [&self.positional, &self.keywords, &self.keyword_rest]
            .into_iter()
            .flatten()
        {
            for (_, value) in entries {
                visitor(value);
            }
        }
        for value in self.rest.iter().flatten().chain(self.block.iter()) {
            visitor(value);
        }
    }
}

pub(super) fn snapshot_hash(
    value: &Value,
    field: ChangeField,
) -> Result<Vec<(String, Value)>, ArgumentError> {
    let entries = match value {
        Value::Hash(hash) => hash.entries(),
        Value::ImmutableHash(hash) => hash.entries().to_vec(),
        _ => return Err(ArgumentError::WrongShape(field)),
    };
    let mut names = std::collections::HashSet::new();
    entries
        .into_iter()
        .map(|(key, value)| {
            let Value::Symbol(name) = key else {
                return Err(ArgumentError::NonSymbolKey(field));
            };
            if !names.insert(name.clone()) {
                return Err(ArgumentError::DuplicateName(name));
            }
            Ok((name, value))
        })
        .collect()
}

pub(super) fn snapshot_array(value: &Value) -> Result<Vec<Value>, ArgumentError> {
    match value {
        Value::Array(array) => Ok(array.elements()),
        Value::ImmutableArray(array) => Ok(array.elements().to_vec()),
        _ => Err(ArgumentError::WrongShape(ChangeField::Rest)),
    }
}
