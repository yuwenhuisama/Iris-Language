use crate::Value;

#[derive(Clone, Debug, PartialEq)]
pub struct SourceCall {
    positional: Vec<Value>,
    keywords: Vec<(String, Value)>,
    block: Value,
    block_omitted: bool,
    omitted: Vec<String>,
    defaulted: Vec<String>,
}

impl SourceCall {
    pub fn new(
        positional: Vec<Value>,
        keywords: Vec<(String, Value)>,
        block: Option<Value>,
    ) -> Self {
        let block_omitted = block.is_none();
        Self {
            positional,
            keywords,
            block: block.unwrap_or(Value::Nil),
            block_omitted,
            omitted: Vec::new(),
            defaulted: Vec::new(),
        }
    }
    pub fn with_provenance(mut self, omitted: Vec<String>, defaulted: Vec<String>) -> Self {
        self.omitted = omitted;
        self.defaulted = defaulted;
        self
    }
    pub fn positional(&self) -> &[Value] {
        &self.positional
    }
    pub fn keywords(&self) -> &[(String, Value)] {
        &self.keywords
    }
    pub const fn block(&self) -> &Value {
        &self.block
    }
    pub const fn block_omitted(&self) -> bool {
        self.block_omitted
    }
    pub fn omitted(&self) -> &[String] {
        &self.omitted
    }
    pub fn defaulted(&self) -> &[String] {
        &self.defaulted
    }

    pub fn visit_values<'a>(&'a self, visitor: &mut impl FnMut(&'a Value)) {
        for value in &self.positional {
            visitor(value);
        }
        for (_, value) in &self.keywords {
            visitor(value);
        }
        visitor(&self.block);
    }
}
