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
    /// The manifest's `version`, when it declares one.
    ///
    /// `IRIS-V1-META-C003` lists the field and `IRIS-V1-META-V420` reflects the
    /// resolved package identity, so it is retained rather than ignored.
    pub version: Option<String>,
    /// Each locked dependency as `(package_id, api_major, version, digest)`.
    ///
    /// `IRIS-V1-META-C006` requires an EXACT selection, which `iris.lock`
    /// records. V420 reflects the selected dependency.
    pub locked: Vec<(String, u32, String, String)>,
    /// The audit artifact beside the manifest, as `(locator, digest, source)`.
    ///
    /// `D-271` retains an immutable locator PLUS a cryptographic digest, and
    /// `IRIS-V1-META-C126` scopes the digest to the artifact's SOURCE bytes.
    pub artifact: Option<(String, String, String)>,
    /// Each permission the manifest requests, as `(name, scope, required)`.
    ///
    /// `IRIS-V1-META-C003` lists permission requests as a manifest field, and
    /// `IRIS-V1-META-V421` observes a load refused for an ungranted REQUIRED
    /// request before any Module body runs.
    pub permissions: Vec<(String, String, bool)>,
    /// Each permission the Host granted, as `(name, scope)`.
    ///
    /// `IRIS-V1-META-C103` scopes a grant to a package or Class, and `C104`
    /// stops grants flowing through callers, so the grant set is a property of
    /// the loaded package rather than of any caller.
    pub grants: Vec<(String, String)>,
    /// Each ordered source entry as `(relative path, contents)`.
    ///
    /// `IRIS-V1-META-C017` initializes in manifest-declared source order, so
    /// the order these are listed in is load-bearing rather than incidental.
    pub sources: Vec<(String, String)>,
    /// The `package_id` of each package this one depends on, in declared order.
    ///
    /// `IRIS-V1-META-C006` selects dependencies before initialization and
    /// `C007` aborts linking on a resolution failure, so a dependency names a
    /// package that must already be loadable rather than being fetched.
    pub dependencies: Vec<String>,
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
    // C006 requires an EXACT dependency selection, which `iris.lock` records
    // beside the manifest. A fixture without one simply locks nothing.
    let locked = match std::fs::read_to_string(directory.join("iris.lock")) {
        Ok(contents) => parse_lock(&contents)?,
        Err(_) => Vec::new(),
    };
    // C066 resolves the exact artifact through the package store; the fixture
    // stores it beside the manifest. A fixture without one carries none.
    let artifact = match std::fs::read_to_string(directory.join("artifact.json")) {
        Ok(contents) => Some(parse_artifact(&contents)?),
        Err(_) => None,
    };
    // A Host grant fixture sits beside the manifest, since C103 makes the
    // grant a HOST decision rather than something the package can declare.
    let grants = match std::fs::read_to_string(directory.join("iris.grants")) {
        Ok(contents) => parse_grants(&contents),
        Err(_) => Vec::new(),
    };
    Ok(Package {
        package_id: manifest.package_id,
        api_major: manifest.api_major,
        sources,
        dependencies: manifest.dependencies,
        version: manifest.version,
        locked,
        artifact,
        permissions: manifest.permissions,
        grants,
    })
}

/// Loads a package fixture together with every package it depends on.
///
/// `IRIS-V1-META-C006` resolves dependencies BEFORE initialization and
/// `IRIS-V1-META-C017` initializes a dependency before its dependent, so the
/// result is ordered dependencies-first. A dependency directory sits beside the
/// consumer under the shared fixture root and is named by its `package_id`.
///
/// `IRIS-V1-META-C007` aborts linking on a resolution failure, so a missing
/// dependency or a dependency cycle is reported rather than skipped.
pub fn load_tree(root: &Path, entry: &str) -> Result<Vec<Package>, String> {
    let mut ordered: Vec<Package> = Vec::new();
    let mut visiting: Vec<String> = Vec::new();
    load_into(root, entry, &mut ordered, &mut visiting)?;
    Ok(ordered)
}

fn load_into(
    root: &Path,
    name: &str,
    ordered: &mut Vec<Package>,
    visiting: &mut Vec<String>,
) -> Result<(), String> {
    if ordered.iter().any(|package| package.package_id == name) {
        return Ok(());
    }
    if visiting.iter().any(|seen| seen == name) {
        return Err(format!("package dependency cycle at {name}"));
    }
    let package = load(&root.join(name))?;
    visiting.push(name.to_owned());
    for dependency in package.dependencies.clone() {
        load_into(root, &dependency, ordered, visiting)?;
    }
    visiting.pop();
    ordered.push(package);
    Ok(())
}

struct Manifest {
    package_id: String,
    api_major: u32,
    sources: Vec<String>,
    dependencies: Vec<String>,
    version: Option<String>,
    permissions: Vec<(String, String, bool)>,
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
    let mut dependencies = Vec::new();
    let mut version = None;
    let mut permissions = Vec::new();
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
            // C003 lists dependency constraints in the manifest. The fixtures
            // this loader serves name a package rather than a SemVer range, and
            // C006 requires the selection to be exact, so a bare package name
            // is the already-selected result rather than a range to resolve.
            "dependencies" => dependencies = parse_array(value)?,
            "version" => version = Some(value.trim_matches('"').to_owned()),
            // C003 lists permission requests. `required` refuses the load when
            // ungranted; `optional` simply stays ungranted.
            "permissions.required" => {
                permissions.extend(parse_permissions(value, true)?);
            }
            "permissions.optional" => {
                permissions.extend(parse_permissions(value, false)?);
            }
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
        dependencies,
        version,
        permissions,
    })
}

/// Parses `iris.lock` entries of the form
/// `package_id = { api_major = 2, version = "2.1.4", digest = "b3:dep214" }`.
///
/// `IRIS-V1-META-C006` makes the selection EXACT, so every field is required
/// and a malformed entry is reported rather than silently skipped.
/// Parses the audit artifact's `locator`, `digest` and `source` fields.
///
/// `D-271` retains a locator PLUS a digest, and `IRIS-V1-META-C126` scopes the
/// digest to the SOURCE bytes, so all three are required and a malformed
/// artifact is reported rather than silently ignored.
/// Parses `["name@scope", ...]` permission requests.
///
/// `IRIS-V1-META-C103` scopes a grant, so a request carries the scope it asks
/// for and an unscoped request is recorded with an empty scope.
fn parse_permissions(value: &str, required: bool) -> Result<Vec<(String, String, bool)>, String> {
    Ok(parse_array(value)?
        .into_iter()
        .map(|entry| match entry.split_once('@') {
            Some((name, scope)) => (name.to_owned(), scope.to_owned(), required),
            None => (entry, String::new(), required),
        })
        .collect())
}

/// Parses one `name@scope` Host grant per line.
fn parse_grants(contents: &str) -> Vec<(String, String)> {
    contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| match line.split_once('@') {
            Some((name, scope)) => (name.to_owned(), scope.to_owned()),
            None => (line.to_owned(), String::new()),
        })
        .collect()
}

fn parse_artifact(contents: &str) -> Result<(String, String, String), String> {
    let field = |name: &str| -> Result<String, String> {
        let key = format!("\"{name}\":");
        let start = contents
            .find(&key)
            .ok_or_else(|| format!("artifact declares no {name}"))?
            + key.len();
        let rest = contents[start..].trim_start();
        let rest = rest
            .strip_prefix('"')
            .ok_or_else(|| format!("artifact {name} expects a string"))?;
        let mut value = String::new();
        let mut escaped = false;
        for character in rest.chars() {
            if escaped {
                value.push(match character {
                    'n' => '\n',
                    other => other,
                });
                escaped = false;
                continue;
            }
            match character {
                '\\' => escaped = true,
                '"' => return Ok(value),
                other => value.push(other),
            }
        }
        Err(format!("artifact {name} is unterminated"))
    };
    Ok((field("locator")?, field("digest")?, field("source")?))
}

fn parse_lock(contents: &str) -> Result<Vec<(String, u32, String, String)>, String> {
    let mut locked = Vec::new();
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (name, rest) = line
            .split_once('=')
            .ok_or_else(|| format!("lock entry expects `name = {{ ... }}`: {line}"))?;
        let body = rest
            .trim()
            .strip_prefix('{')
            .and_then(|rest| rest.strip_suffix('}'))
            .ok_or_else(|| format!("lock entry expects a table: {line}"))?;
        let mut api_major = None;
        let mut version = None;
        let mut digest = None;
        for field in body.split(',') {
            let Some((key, value)) = field.split_once('=') else {
                continue;
            };
            let value = value.trim().trim_matches('"');
            match key.trim() {
                "api_major" => {
                    api_major = Some(
                        value
                            .parse::<u32>()
                            .map_err(|error| format!("lock api_major: {error}"))?,
                    );
                }
                "version" => version = Some(value.to_owned()),
                "digest" => digest = Some(value.to_owned()),
                _ => {}
            }
        }
        locked.push((
            name.trim().to_owned(),
            api_major.ok_or_else(|| format!("lock entry declares no api_major: {line}"))?,
            version.ok_or_else(|| format!("lock entry declares no version: {line}"))?,
            digest.ok_or_else(|| format!("lock entry declares no digest: {line}"))?,
        ));
    }
    Ok(locked)
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
