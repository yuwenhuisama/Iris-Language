use crate::NativeError;
use std::collections::BTreeSet;

#[derive(Clone, Debug)]
pub struct PackageSource {
    pub package_id: String,
    pub api_major: u32,
    pub version: String,
    pub path: String,
    pub source: String,
    pub allowed_imports: BTreeSet<String>,
}
impl PackageSource {
    pub fn validate_metadata(sources: &[Self]) -> Result<(), NativeError> {
        let mut packages = std::collections::BTreeMap::new();
        for source in sources {
            if source.package_id.is_empty() || source.api_major == 0 || source.version.is_empty() {
                return Err(NativeError::Metadata);
            }
            let metadata = (source.api_major, &source.version, &source.allowed_imports);
            if let Some(previous) = packages.insert(&source.package_id, metadata)
                && previous != metadata
            {
                return Err(NativeError::Conflict);
            }
        }
        Ok(())
    }

    pub fn validate_imports(&self) -> Result<(), NativeError> {
        let parsed = iris_parser::parse(&self.source);
        if !parsed.program_accepted {
            return Err(NativeError::Metadata);
        }
        for declaration in parsed.program.declarations {
            if let iris_syntax::Declaration::Import(import) = declaration
                && !self.allowed_imports.contains(&import.target)
            {
                return Err(NativeError::Denied);
            }
        }
        Ok(())
    }
}
