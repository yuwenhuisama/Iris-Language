use crate::{PackageId, RelativePath, Sha256Digest};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildReceipts {
    pub receipt_version: u32,
    pub artifacts: Vec<BuildReceipt>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildReceipt {
    pub package_id: PackageId,
    pub api_major: u32,
    pub source_sha256: Sha256Digest,
    pub metadata_sha256: Sha256Digest,
    pub platform: String,
    pub artifact: RelativePath,
    pub artifact_sha256: Sha256Digest,
}

pub fn platform() -> &'static str {
    env!("IRIS_PACKAGE_TARGET")
}
