//! Loads an on-disk package fixture: `iris.toml` plus its ordered source files.
//!
//! `IRIS-V1-META-C003` requires a publishable package to declare `package_id`,
//! `api_major` and ordered source entries in `iris.toml`, and
//! `IRIS-V1-META-C010` aborts the load when that manifest is missing or
//! invalid. Chapter 08 vectors name a concrete fixture directory rather than
//! carrying their program inline, so the corpus reads the tree from disk.
//!
//! Only the manifest subset those vectors actually observe is parsed. The
//! dependency, permission, native-artifact and lock fields `C003` also lists
//! are NOT invented here: a vector needing them is recorded as blocked rather
//! than served by a stub that would look like support.

use std::path::Path;

/// One loaded package fixture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Package {
    /// The manifest's `package_id`, which `D-431` makes part of a global's
    /// identity and `D-243` makes part of a named nominal Type's identity.
    pub package_id: String,
    /// The manifest's `api_major`.
    pub api_major: u32,
    /// Each ordered source entry as `(relative path, contents)`.
    ///
    /// `IRIS-V1-META-C017` initializes in manifest-declared source order, so
    /// the order these are listed in is load-bearing rather than incidental.
    pub sources: Vec<(String, String)>,
}

/// Reads the package fixture rooted at `directory`.
///
/// `IRIS-V1-META-C010` aborts the load on a missing or invalid manifest, so
/// every failure here is reported rather than defaulted.
pub fn load(directory: &Path) -> Result<Package, String> {
    let manifest_path = directory.join("iris.toml");
    let manifest = std::fs::read_to_string(&manifest_path)
        .map_err(|error| format!("{}: {error}", manifest_path.display()))?;
    let manifest = parse_manifest(&manifest)?;
    let sources = manifest
        .sources
        .iter()
        .map(|entry| {
            let path = directory.join(entry);
            std::fs::read_to_string(&path)
                .map(|contents| (entry.clone(), contents))
                .map_err(|error| format!("{}: {error}", path.display()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Package {
        package_id: manifest.package_id,
        api_major: manifest.api_major,
        sources,
    })
}

struct Manifest {
    package_id: String,
    api_major: u32,
    sources: Vec<String>,
}

/// Parses the manifest subset chapter 08 vectors observe.
///
/// This is deliberately not a general TOML reader: it accepts the flat
/// `key = value` and `key = ["a", "b"]` forms those fixtures use, and reports
/// anything else rather than guessing, so an unsupported manifest aborts the
/// load exactly as `IRIS-V1-META-C010` requires.
fn parse_manifest(text: &str) -> Result<Manifest, String> {
    let mut package_id = None;
    let mut api_major = None;
    let mut sources = Vec::new();
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() || line.starts_with('[') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(format!("manifest line is not a key/value pair: {line}"));
        };
        let (key, value) = (key.trim(), value.trim());
        match key {
            "package_id" => package_id = Some(unquote(value)?),
            "api_major" => {
                api_major = Some(
                    value
                        .parse::<u32>()
                        .map_err(|_| format!("api_major must be an integer: {value}"))?,
                );
            }
            "sources" => sources = parse_array(value)?,
            // A manifest key this loader does not model is IGNORED rather than
            // rejected, so a fixture may carry the version, dependency or
            // permission fields `C003` lists without this pretending to honour
            // them. A vector that OBSERVES such a field stays blocked.
            _ => {}
        }
    }
    Ok(Manifest {
        package_id: package_id.ok_or("manifest declares no package_id")?,
        api_major: api_major.ok_or("manifest declares no api_major")?,
        sources,
    })
}

fn parse_array(value: &str) -> Result<Vec<String>, String> {
    let inner = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or_else(|| format!("expected an array: {value}"))?;
    inner
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(unquote)
        .collect()
}

fn unquote(value: &str) -> Result<String, String> {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(str::to_owned)
        .ok_or_else(|| format!("expected a quoted string: {value}"))
}

#[cfg(test)]
mod tests;
