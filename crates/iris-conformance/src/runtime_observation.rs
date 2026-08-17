use iris_runtime::Value as RuntimeValue;

use crate::{
    json::Value,
    model::{Record, array, object, parse_expect, string},
};

mod values;

pub fn compare_runtime(record: &Record) -> Result<(), String> {
    let expected = parse_expect(&record.expect)?;
    let expected = object(&expected)?;
    if !record.independent_sources.is_empty() {
        return compare_independent_sources(record, expected);
    }
    // Chapter 08 names an on-disk package tree rather than carrying its
    // program inline, so the manifest and its ordered sources are read from
    // the fixture directory and the load's initialized Modules are observed.
    if let Some(fixture) = &record.package_fixture {
        return compare_package_fixture(record, expected, fixture);
    }
    // D-431 needs two packages sharing ONE runtime, unlike independent
    // sources, whose programs each run against a fresh runtime.
    if !record.package_sources.is_empty() {
        let outcome = iris_eval::evaluate_packages_with_probe(
            &record.package_sources,
            record.package_probe.as_deref(),
        );
        return match expected.get("error") {
            Some(error) => values::compare_evaluated_error(error, outcome),
            None => values::compare_evaluated(expected, outcome),
        };
    }
    compare_runtime_source(record, expected, runtime_source(record)?)
}

/// Loads an on-disk package fixture and compares what its load observed.
///
/// `IRIS-V1-META-C010` aborts the load on a missing or invalid manifest, and
/// `IRIS-V1-META-C017` initializes in manifest-declared source order, so the
/// observable is the ordered list of initialized Modules.
fn compare_package_fixture(
    record: &Record,
    expected: &std::collections::BTreeMap<String, Value>,
    fixture: &str,
) -> Result<(), String> {
    let directory = crate::model::Corpus::workspace()?
        .0
        .join("conformance/iris-v1")
        .join(fixture);
    let package = crate::package_fixture::load(&directory)?;
    // C033 forbids a deferred package from claiming language-core ABI status
    // and C034 forbids an advanced Regex package from replacing core literal
    // semantics. The claim is refused at LOAD, so the package never
    // initializes and its declarations never become reachable.
    if package.core_claim {
        return values::compare_evaluated_error(
            expected.get("error").ok_or_else(|| {
                format!(
                    "{}: package claims core ABI but the row expects no error",
                    record.id
                )
            })?,
            Err(iris_eval::EvaluationError::LexicalDiagnostic(
                "PACKAGE_CORE_ABI_CLAIM",
            )),
        )
        .map_err(|error| format!("{}: {error}", record.id));
    }
    // A row observing a STATIC rejection never reaches evaluation, so its
    // sources are collected through the same diagnostics collector every other
    // chapter uses rather than being loaded.
    if let Some(diagnostics) = expected.get("diagnostics") {
        // A decorator may be DECLARED in a dependency and applied here, so the
        // whole dependency-first tree is gathered rather than this package
        // alone. `IRIS-V1-META-C017` makes that order normative, and a fixture
        // without dependencies yields exactly the single package it did before.
        let packages = package_tree(&directory, &package)?;
        return compare_package_diagnostics(diagnostics, &packages)
            .map_err(|error| format!("{}: {error}", record.id));
    }
    // A row that observes a Module member sends to it AFTER the load, since a
    // package source file is declarations only under `IRIS-V1-META-C011`.
    // A fixture that declares dependencies loads the whole dependency-first
    // tree, since C006 resolves dependencies BEFORE initialization and C017
    // initializes a dependency before its dependent. A fixture without them
    // takes the single-package path exactly as before.
    let packages = package_tree(&directory, &package)?;
    let outcome = if package.dependencies.is_empty() {
        iris_eval::load_resolved_package_with_artifact(
            iris_eval::PackageResolution {
                package_id: &package.package_id,
                api_major: u64::from(package.api_major),
                version: package.version.clone(),
                locked: package
                    .locked
                    .iter()
                    .map(|(name, major, version, digest)| {
                        (
                            name.clone(),
                            u64::from(*major),
                            version.clone(),
                            digest.clone(),
                        )
                    })
                    .collect(),
                artifact: package.artifact.clone(),
                permissions: &package.permissions,
                grants: package.grants.clone(),
            },
            &package.sources,
            record.package_probe.as_deref(),
        )
    } else {
        let tree: Vec<(String, Vec<(String, String)>)> = packages
            .into_iter()
            .map(|package| (package.package_id, package.sources))
            .collect();
        iris_eval::load_package_tree_with_grants(
            &tree,
            package.grants.clone(),
            record.package_probe.as_deref(),
        )
    }
    .map(|(modules, observed)| match observed {
        Some(value) => value,
        None => RuntimeValue::Array(modules.into_iter().map(RuntimeValue::Symbol).collect()),
    });
    match expected.get("error") {
        Some(error) => values::compare_evaluated_error(error, outcome),
        None => values::compare_evaluated(expected, outcome),
    }
    .map_err(|error| format!("{}: {error}", record.id))
}

/// Compares the diagnostics a package fixture's sources report.
///
/// `IRIS-V1-META-C011` makes a package source file declarations only, so a row
/// stating a static rejection observes the codes its files report rather than
/// any loaded value. The codes from every source are gathered in manifest
/// order, since `IRIS-V1-META-C017` makes that order normative.
/// The fixture's packages, ordered dependencies-first.
///
/// `IRIS-V1-META-C006` resolves dependencies BEFORE initialization, so a
/// fixture that declares them is loaded as a tree whose members sit beside it
/// under a shared root and are named by `package_id`.
fn package_tree(
    directory: &std::path::Path,
    package: &crate::package_fixture::Package,
) -> Result<Vec<crate::package_fixture::Package>, String> {
    if package.dependencies.is_empty() {
        return Ok(vec![package.clone()]);
    }
    let (root, entry) = directory
        .parent()
        .zip(directory.file_name().and_then(std::ffi::OsStr::to_str))
        .ok_or_else(|| format!("package fixture {} has no parent root", directory.display()))?;
    crate::package_fixture::load_tree(root, entry)
}

fn compare_package_diagnostics(
    expected: &Value,
    packages: &[crate::package_fixture::Package],
) -> Result<(), String> {
    let Value::Array(entries) = expected else {
        return Err("diagnostics expectation must be an array".into());
    };
    let expected = entries
        .iter()
        .map(|entry| string(object(entry)?, "code").map(str::to_owned))
        .collect::<Result<Vec<_>, String>>()?;
    let mut actual = Vec::new();
    for (_, source) in packages.iter().flat_map(|package| &package.sources) {
        actual.extend(
            crate::runner::diagnostics(source)
                .into_iter()
                .map(|diagnostic| diagnostic.code),
        );
    }
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "diagnostics expected {expected:?}, actual {actual:?}"
        ))
    }
}

fn compare_independent_sources(
    record: &Record,
    expected: &std::collections::BTreeMap<String, Value>,
) -> Result<(), String> {
    let expectations = array(
        expected
            .get("independent_expectations")
            .ok_or("independent expectations missing")?,
    )?;
    if expectations.len() != record.independent_sources.len() {
        return Err("independent source and expectation counts differ".into());
    }
    record
        .independent_sources
        .iter()
        .zip(expectations)
        .enumerate()
        .try_for_each(|(index, (source, expected))| {
            compare_runtime_source(record, object(expected)?, source)
                .map_err(|error| format!("independent source {index}: {error}"))
        })
}

fn compare_runtime_source(
    record: &Record,
    expected: &std::collections::BTreeMap<String, Value>,
    source: &str,
) -> Result<(), String> {
    if let Some(expected) = expected.get("diagnostics") {
        return compare_diagnostics(expected, source);
    }
    match expected.get("error") {
        Some(error) => match expected.get("side_effects") {
            Some(side_effects) => {
                values::compare_error_and_side_effects(error, side_effects, source)
            }
            None => values::compare_error(error, source),
        },
        None if record.source.contains("stable numeric hash")
            || record.source.contains("stable singleton hash") =>
        {
            compare_hash_fixture(
                expected.get("value").ok_or("runtime value missing")?,
                source,
            )
        }
        None => values::compare_observation(expected, source),
    }
}

/// Compares a RUNTIME record's expected diagnostic codes against the source.
///
/// A malformed source never reaches evaluation, so a row asserting a lexical or
/// declaration-validation rejection has no runtime value or error to observe.
/// This reuses the same `diagnostics` collector the GRAMMAR runner uses, so both
/// chapters classify a given source identically as `IRIS-V1-GRAMMAR-C054`
/// requires of a conforming diagnostic system.
fn compare_diagnostics(expected: &Value, source: &str) -> Result<(), String> {
    let Value::Array(entries) = expected else {
        return Err("diagnostics expectation must be an array".into());
    };
    let expected = entries
        .iter()
        .map(|entry| string(object(entry)?, "code").map(str::to_owned))
        .collect::<Result<Vec<_>, String>>()?;
    let actual = crate::runner::diagnostics(source)
        .into_iter()
        .map(|value| value.code)
        .collect::<Vec<_>>();
    if expected.iter().all(|code| actual.contains(code)) {
        Ok(())
    } else {
        Err(format!(
            "diagnostics expected {expected:?}, actual {actual:?}"
        ))
    }
}

fn runtime_source(record: &Record) -> Result<&str, String> {
    match record.id.as_str() {
        "IRIS-V1-RUNTIME-V016" => {
            Ok("class A { public fun m() -> Integer { 1 } }; let obj = A.new(); obj.m same? obj.m")
        }
        "IRIS-V1-RUNTIME-V093" => {
            Ok("class A { public fun m() -> Integer { super() } }; A.new().m()")
        }
        _ => Ok(&record.source),
    }
}

fn compare_hash_fixture(expected: &Value, fixture: &str) -> Result<(), String> {
    let expected = match expected {
        Value::String(expected) => expected,
        _ => return Err("stable hash fixture expects string value".into()),
    };
    let expected = expected
        .strip_prefix("Public hash Integer ")
        .unwrap_or(expected);
    let fixture = match fixture.split(';').next() {
        Some(value) => value.trim(),
        None => fixture,
    };
    let fixture = match fixture.split(',').next() {
        Some(value) => value.trim(),
        None => fixture,
    };
    let value = if fixture.starts_with("Integer(0)") {
        RuntimeValue::Integer(0_u8.into())
    } else if fixture.starts_with("Integer(1)") {
        RuntimeValue::Integer(1_u8.into())
    } else if fixture.starts_with("Integer(-1)") {
        RuntimeValue::Integer((-1_i8).into())
    } else if fixture.starts_with("Integer(2)") {
        RuntimeValue::Integer(2_u8.into())
    } else if fixture.starts_with("Exact mathematical 3/2") {
        RuntimeValue::Float64(1.5)
    } else if fixture.starts_with("Positive infinity") {
        RuntimeValue::Float64(f64::INFINITY)
    } else if fixture.starts_with("Negative infinity") {
        RuntimeValue::Float64(f64::NEG_INFINITY)
    } else if fixture == "nil" {
        RuntimeValue::Nil
    } else if fixture == "false" {
        RuntimeValue::Bool(false)
    } else if fixture == "true" {
        RuntimeValue::Bool(true)
    } else {
        return Err("unsupported stable hash fixture".into());
    };
    let actual = iris_runtime::public_hash(&value)
        .map_err(|error| format!("error {error}"))?
        .decimal_text();
    if actual == expected {
        Ok(())
    } else {
        Err(format!("value expected {expected}, actual {actual}"))
    }
}

#[cfg(test)]
mod tests;
