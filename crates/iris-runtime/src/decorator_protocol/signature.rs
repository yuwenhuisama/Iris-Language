use super::{ArgumentError, ParameterCategory};
use crate::Value;

#[derive(Clone, Debug, PartialEq)]
pub struct InvocationParameter {
    name: String,
    category: ParameterCategory,
    parameter_type: Value,
    optional: bool,
}

impl InvocationParameter {
    pub fn new(
        name: impl Into<String>,
        category: ParameterCategory,
        parameter_type: Value,
    ) -> Self {
        Self {
            name: name.into(),
            category,
            parameter_type,
            optional: false,
        }
    }
    pub fn with_optional(mut self, optional: bool) -> Self {
        self.optional = optional;
        self
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub const fn category(&self) -> ParameterCategory {
        self.category
    }
    pub const fn parameter_type(&self) -> &Value {
        &self.parameter_type
    }
    pub const fn optional(&self) -> bool {
        self.optional
    }

    fn rank(&self) -> u8 {
        match (self.category, self.optional) {
            (ParameterCategory::Positional, false) => 0,
            (ParameterCategory::Positional, true) => 1,
            (ParameterCategory::Rest, _) => 2,
            (ParameterCategory::Keyword, false) => 3,
            (ParameterCategory::Keyword, true) => 4,
            (ParameterCategory::KeywordRest, _) => 5,
            (ParameterCategory::Block, _) => 6,
        }
    }
}

/// Exact, already-substituted backend Type values, without executable defaults.
#[derive(Clone, Debug, PartialEq)]
pub struct InvocationSignature {
    parameters: Vec<InvocationParameter>,
    result: Value,
    is_async: bool,
}

impl InvocationSignature {
    pub fn new(
        parameters: Vec<InvocationParameter>,
        result: Value,
        is_async: bool,
    ) -> Result<Self, ArgumentError> {
        let mut names = std::collections::HashSet::new();
        let mut previous = 0;
        let mut channels = Vec::new();
        for parameter in &parameters {
            if !names.insert(parameter.name()) {
                return Err(ArgumentError::DuplicateName(parameter.name.clone()));
            }
            if parameter.rank() < previous {
                return Err(ArgumentError::InvalidParameterOrder(parameter.name.clone()));
            }
            previous = parameter.rank();
            match parameter.category {
                ParameterCategory::Rest
                | ParameterCategory::KeywordRest
                | ParameterCategory::Block => {
                    if channels.contains(&parameter.category) {
                        return Err(ArgumentError::InvalidParameterOrder(parameter.name.clone()));
                    }
                    channels.push(parameter.category);
                }
                ParameterCategory::Positional | ParameterCategory::Keyword => {}
            }
            if parameter.optional
                && matches!(
                    parameter.category,
                    ParameterCategory::Rest | ParameterCategory::KeywordRest
                )
            {
                return Err(ArgumentError::InvalidOptionality(parameter.name.clone()));
            }
        }
        Ok(Self {
            parameters,
            result,
            is_async,
        })
    }
    pub fn parameters(&self) -> &[InvocationParameter] {
        &self.parameters
    }
    pub const fn result(&self) -> &Value {
        &self.result
    }
    pub const fn is_async(&self) -> bool {
        self.is_async
    }
    pub fn parameter(&self, name: &str) -> Option<&InvocationParameter> {
        self.parameters
            .iter()
            .find(|parameter| parameter.name() == name)
    }
    pub fn channel(&self, category: ParameterCategory) -> Option<&InvocationParameter> {
        self.parameters
            .iter()
            .find(|parameter| parameter.category() == category)
    }

    pub fn visit_values<'a>(&'a self, visitor: &mut impl FnMut(&'a Value)) {
        visitor(&self.result);
        for parameter in &self.parameters {
            visitor(parameter.parameter_type());
        }
    }
}
