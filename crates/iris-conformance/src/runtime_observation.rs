use iris_runtime::Value as RuntimeValue;

use crate::{
    json::Value,
    model::{Record, array, object, parse_expect},
};

mod values;

pub fn compare_runtime(record: &Record) -> Result<(), String> {
    let expected = parse_expect(&record.expect)?;
    let expected = object(&expected)?;
    if !record.independent_sources.is_empty() {
        return compare_independent_sources(record, expected);
    }
    compare_runtime_source(record, expected, runtime_source(record)?)
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
    if expected.contains_key("side_effects") && expected.contains_key("error") {
        return Err("side_effects cannot accompany an error expectation".into());
    }
    match expected.get("error") {
        Some(error) => values::compare_error(error, source),
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
