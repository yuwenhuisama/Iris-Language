use super::{ArgumentChanges, ArgumentError, InvocationPayload, InvocationSignature, SourceCall};
use crate::Value;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlotKind {
    Method,
    Getter,
    Setter,
}

impl SlotKind {
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Method => "method",
            Self::Getter => "getter",
            Self::Setter => "setter",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InvocationSlot {
    declaring: Value,
    selector: String,
    qualifier: Option<Value>,
    kind: SlotKind,
}

impl InvocationSlot {
    pub fn new(declaring: Value, selector: impl Into<String>, kind: SlotKind) -> Self {
        Self {
            declaring,
            selector: selector.into(),
            qualifier: None,
            kind,
        }
    }
    pub fn qualified(mut self, qualifier: Value) -> Self {
        self.qualifier = Some(qualifier);
        self
    }
    pub const fn declaring(&self) -> &Value {
        &self.declaring
    }
    pub fn selector(&self) -> &str {
        &self.selector
    }
    pub const fn qualifier(&self) -> Option<&Value> {
        self.qualifier.as_ref()
    }
    pub const fn kind(&self) -> SlotKind {
        self.kind
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectedCall {
    receiver: Value,
    slot: InvocationSlot,
    signature: InvocationSignature,
    owner_type_arguments: Vec<Value>,
    method_type_arguments: Vec<Value>,
}

impl SelectedCall {
    pub const fn new(
        receiver: Value,
        slot: InvocationSlot,
        signature: InvocationSignature,
    ) -> Self {
        Self {
            receiver,
            slot,
            signature,
            owner_type_arguments: Vec::new(),
            method_type_arguments: Vec::new(),
        }
    }
    pub fn with_type_arguments(mut self, owner: Vec<Value>, method: Vec<Value>) -> Self {
        self.owner_type_arguments = owner;
        self.method_type_arguments = method;
        self
    }
    pub const fn receiver(&self) -> &Value {
        &self.receiver
    }
    pub const fn slot(&self) -> &InvocationSlot {
        &self.slot
    }
    pub const fn signature(&self) -> &InvocationSignature {
        &self.signature
    }
    pub fn owner_type_arguments(&self) -> &[Value] {
        &self.owner_type_arguments
    }
    pub fn method_type_arguments(&self) -> &[Value] {
        &self.method_type_arguments
    }

    pub fn visit_values<'a>(&'a self, visitor: &mut impl FnMut(&'a Value)) {
        visitor(&self.receiver);
        visitor(self.slot.declaring());
        if let Some(qualifier) = self.slot.qualifier() {
            visitor(qualifier);
        }
        self.signature.visit_values(visitor);
        for value in self
            .owner_type_arguments
            .iter()
            .chain(&self.method_type_arguments)
        {
            visitor(value);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Invocation {
    selected: SelectedCall,
    source: SourceCall,
    payload: InvocationPayload,
}

impl Invocation {
    /// Called by the backend only after original preparation and validation succeed.
    pub const fn new(
        selected: SelectedCall,
        source: SourceCall,
        payload: InvocationPayload,
    ) -> Self {
        Self {
            selected,
            source,
            payload,
        }
    }
    pub const fn selected(&self) -> &SelectedCall {
        &self.selected
    }
    pub const fn source(&self) -> &SourceCall {
        &self.source
    }
    pub const fn payload(&self) -> &InvocationPayload {
        &self.payload
    }

    pub fn patch(
        &self,
        changes: &ArgumentChanges,
        accepts: impl FnMut(&Value, &Value) -> bool,
    ) -> Result<Self, ArgumentError> {
        let payload = self
            .payload
            .patch(self.selected.signature(), changes, accepts)?;
        Ok(Self {
            selected: self.selected.clone(),
            source: self.source.clone(),
            payload,
        })
    }

    pub fn visit_values<'a>(&'a self, visitor: &mut impl FnMut(&'a Value)) {
        self.selected.visit_values(visitor);
        self.source.visit_values(visitor);
        self.payload.visit_values(visitor);
    }
}
