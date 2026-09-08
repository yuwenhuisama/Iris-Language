/// An unsupported construct or a statically diagnosed program violation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompileError {
    /// Existing construct description, or the static diagnostic code.
    pub construct: String,
    /// Distinguishes backend coverage from a rejected program.
    pub kind: CompileErrorKind,
}

/// The reason compilation cannot produce an executable program.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompileErrorKind {
    UnsupportedConstruct,
    StaticDiagnostic { code: &'static str },
}

impl CompileError {
    pub(crate) fn new(construct: impl Into<String>) -> Self {
        Self {
            construct: construct.into(),
            kind: CompileErrorKind::UnsupportedConstruct,
        }
    }

    pub(super) fn diagnostic(code: &'static str) -> Self {
        Self {
            construct: code.to_owned(),
            kind: CompileErrorKind::StaticDiagnostic { code },
        }
    }
}
