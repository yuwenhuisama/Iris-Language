use crate::Value;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeBridge {
    pub package: String,
    pub code: String,
    pub message: String,
    pub os_code: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExceptionOrigin {
    pub location: Value,
    pub original_stack: Vec<Value>,
    pub native_bridge: Option<NativeBridge>,
}

impl From<Value> for ExceptionOrigin {
    fn from(location: Value) -> Self {
        Self {
            location,
            original_stack: Vec::new(),
            native_bridge: None,
        }
    }
}
