use iris_runtime::Value;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreErrorKind {
    ArgumentError,
    TypeError,
}

impl CoreErrorKind {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::ArgumentError => "ArgumentError",
            Self::TypeError => "TypeError",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HostException {
    pub kind: CoreErrorKind,
    pub value: Value,
    pub context: Option<Value>,
}

impl crate::EvaluationError {
    pub(crate) fn into_language_error(self) -> Self {
        match self {
            Self::HostException(exception) => Self::Raised(exception.value),
            error => error,
        }
    }
}
