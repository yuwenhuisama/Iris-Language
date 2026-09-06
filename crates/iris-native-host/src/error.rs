use iris_runtime::{ExceptionOrigin, ObjectId, Value};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, thiserror::Error)]
pub enum NativeError {
    #[error("native code is not authorized")]
    Denied,
    #[error("native metadata is invalid or incompatible")]
    Metadata,
    #[error("native artifact or metadata digest does not match")]
    Digest,
    #[error("native module export conflicts with an existing declaration")]
    Conflict,
    #[error("native ABI protocol failure (status {0})")]
    Protocol(i32),
    #[error("native argument or return type mismatch")]
    Type,
    #[error("native resource is closed or belongs to another registry")]
    Closed,
    #[error("native resource quota exceeded")]
    Quota,
    #[error("{code}: {message}")]
    Raised {
        code: String,
        message: String,
        os_code: i64,
    },
    #[error("{error}")]
    Context {
        error: Box<NativeError>,
        context: Box<iris_runtime::Value>,
    },
    #[error("native artifact I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("native library loading: {0}")]
    Library(#[from] libloading::Error),
    #[error("native metadata JSON: {0}")]
    Json(#[from] serde_json::Error),
}
impl NativeError {
    pub fn raised_value(&self) -> iris_runtime::Value {
        let code = match self {
            Self::Context { error, .. } => return error.raised_value(),
            Self::Raised { code, .. } => code.as_str(),
            Self::Type => "TypeContractError",
            Self::Closed => "ClosedResourceError",
            Self::Quota => "NativeResourceQuotaError",
            Self::Denied => "NativePermissionError",
            Self::Metadata
            | Self::Digest
            | Self::Conflict
            | Self::Protocol(_)
            | Self::Io(_)
            | Self::Library(_)
            | Self::Json(_) => "NativeBoundaryError",
        };
        iris_runtime::Value::Symbol(code.to_owned())
    }

    pub fn into_propagation(self) -> (iris_runtime::Value, iris_runtime::Value) {
        let value = self.raised_value();
        match self {
            Self::Context { context, .. } => (value, *context),
            _ => {
                let context = context(value.clone(), Value::Nil.into());
                (value, context)
            }
        }
    }
}
static NEXT_CONTEXT: AtomicU64 = AtomicU64::new(u64::MAX);

pub(crate) fn context(value: Value, origin: ExceptionOrigin) -> Value {
    Value::ExceptionContext(
        ObjectId::new(NEXT_CONTEXT.fetch_sub(1, Ordering::Relaxed)),
        Box::new(value),
        Box::new(Value::Nil),
        Vec::new(),
        Vec::new(),
        Box::new(origin),
    )
}
