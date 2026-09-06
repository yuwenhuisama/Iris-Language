use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum PackageError {
    #[error("package I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid TOML: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("cannot serialize package record: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[error("invalid {field}: {value}")]
    Invalid { field: &'static str, value: String },
    #[error("local Git transport requires explicit allow_local_git")]
    LocalGitDenied,
    #[error("native builds require explicit allow_native_build; build scripts are not sandboxed")]
    NativeBuildDenied,
    #[error("permission {permission} denied for {package}")]
    PermissionDenied { package: String, permission: String },
    #[error("package integrity mismatch: {path}")]
    Integrity { path: PathBuf },
    #[error("unsafe or non-regular package path: {0}")]
    UnsafePath(PathBuf),
    #[error("package identity, API, version or revision conflict: {0}")]
    Conflict(String),
    #[error("package dependency cycle: {0}")]
    Cycle(String),
    #[error("missing or stale build receipt for {0}")]
    MissingBuild(String),
    #[error("{program} failed ({status:?}): {stderr}")]
    Command {
        program: &'static str,
        status: Option<i32>,
        stderr: String,
    },
}

pub(crate) fn invalid(field: &'static str, value: impl Into<String>) -> PackageError {
    PackageError::Invalid {
        field,
        value: value.into(),
    }
}
