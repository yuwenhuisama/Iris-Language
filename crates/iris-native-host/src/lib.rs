//! Trusted, thread-affine native module execution shared by both Iris engines.
mod callbacks;
mod declarations;
mod error;
mod frame;
mod metadata;
mod package;
mod registry;
mod resource;
pub use error::NativeError;
pub use metadata::*;
pub use package::PackageSource;
pub use registry::{NativePolicy, NativeRegistry, TrustedModule, sha256};
#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests assert fixture setup")]
mod tests;
