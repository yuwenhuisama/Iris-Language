use super::Error;
use iris_native_host::PackageSource;
use iris_package::PreparedPackageTree;
use std::collections::BTreeSet;

pub(super) fn collect(tree: &PreparedPackageTree) -> Result<Vec<PackageSource>, Error> {
    let mut sources = Vec::new();
    for package in &tree.packages {
        let mut allowed_imports: BTreeSet<String> =
            package.manifest.entry_modules.iter().cloned().collect();
        for dependency in package.manifest.dependencies.values() {
            let declared = tree
                .packages
                .iter()
                .find(|candidate| candidate.manifest.package_id == dependency.package_id)
                .ok_or_else(|| {
                    iris_package::PackageError::Conflict(dependency.package_id.to_string())
                })?;
            allowed_imports.extend(declared.manifest.entry_modules.iter().cloned());
        }
        for source in &package.sources {
            let path = package
                .root
                .join(source.path.as_str())
                .display()
                .to_string();
            sources.push(PackageSource {
                package_id: package.manifest.package_id.to_string(),
                api_major: package.manifest.api_major,
                version: package.manifest.version.to_string(),
                source: String::from_utf8(source.bytes.clone()).map_err(|source| {
                    Error::Source {
                        path: path.clone(),
                        source,
                    }
                })?,
                path,
                allowed_imports: allowed_imports.clone(),
            });
        }
    }
    Ok(sources)
}

pub(super) fn digest(text: &str) -> Result<[u8; 32], Error> {
    if text.len() != 64 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(Error::Digest);
    }
    let mut bytes = [0; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte =
            u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).map_err(|_| Error::Digest)?;
    }
    Ok(bytes)
}
