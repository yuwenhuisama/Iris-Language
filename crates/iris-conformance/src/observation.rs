use iris_eval::Value as IrisValue;
use iris_lexer::{convert_literals, lex};
use iris_syntax::render_parse_shapes;

use crate::{
    json::Value,
    model::{Record, object, parse_expect, render, string, value_string},
    runner::diagnostics,
};

pub fn compare(record: &Record, parsed: &iris_parser::ParseResult) -> Result<(), String> {
    let expected = parse_expect(&record.expect)?;
    let expected = object(&expected)?;
    if let Some(Value::Array(entries)) = expected.get("diagnostics") {
        let expected = entries
            .iter()
            .map(|entry| string(object(entry)?, "code").map(str::to_owned))
            .collect::<Result<Vec<_>, String>>()?;
        // Lexical diagnostics do not accumulate across a whole source: lexing
        // reports the first and stops. A row naming several INDEPENDENT
        // malformed inputs therefore states them as independent sources, and
        // the codes are gathered across all of them in order.
        let sources: Vec<&str> = if record.independent_sources.is_empty() {
            vec![record.source.as_str()]
        } else {
            record
                .independent_sources
                .iter()
                .map(String::as_str)
                .collect()
        };
        let actual = sources
            .into_iter()
            .flat_map(|source| {
                diagnostics(source)
                    .into_iter()
                    .map(|value| value.code)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        if !expected.iter().all(|code| actual.contains(code)) {
            return Err(format!(
                "diagnostics expected {expected:?}, actual {actual:?}"
            ));
        }
    }
    if let Some(value) = expected.get("value") {
        compare_value(value, &record.source)?;
    }
    if let Some(Value::Object(artifact)) = expected.get("artifact") {
        compare_artifact(artifact, parsed, &record.source)?;
    }
    Ok(())
}

fn compare_value(expected: &Value, source: &str) -> Result<(), String> {
    let conversion = convert_literals(source);
    let values = conversion
        .values()
        .iter()
        .cloned()
        .map(IrisValue::from)
        .collect::<Vec<_>>();
    let actual = match values.as_slice() {
        [] => return Err("value unsupported: no converted literal".into()),
        [value] => value.clone(),
        _ => IrisValue::Array(values),
    };
    if render(expected) == value_render(&actual) {
        Ok(())
    } else {
        Err(format!(
            "value expected {}, actual {}",
            render(expected),
            value_render(&actual)
        ))
    }
}

fn value_render(value: &IrisValue) -> String {
    match value {
        IrisValue::Integer(value) => format!("{{\"integer\":\"{value}\"}}"),
        IrisValue::Float32Bits(value) => format!("{{\"float32_bits\":\"0x{value:08x}\"}}"),
        IrisValue::Float64Bits(value) => format!("{{\"float64_bits\":\"0x{value:016x}\"}}"),
        IrisValue::String(value) => render(&Value::Object(
            [(String::from("string"), Value::String(value.clone()))].into(),
        )),
        IrisValue::Array(values) => format!(
            "{{\"array\":[{}]}}",
            values
                .iter()
                .map(value_render)
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}

fn compare_artifact(
    expected: &std::collections::BTreeMap<String, Value>,
    parsed: &iris_parser::ParseResult,
    source: &str,
) -> Result<(), String> {
    if let Some(Value::Array(shapes)) = expected.get("parse_shapes") {
        let expected = shapes
            .iter()
            .map(value_string)
            .collect::<Result<Vec<_>, _>>()?;
        let actual = parse_shapes(parsed);
        if expected != actual {
            return Err(format!(
                "parse_shapes expected {expected:?}, actual {actual:?}"
            ));
        }
    }
    if let Some(Value::String(shape)) = expected.get("parse_shape") {
        let actual = parse_shapes(parsed).into_iter().next().unwrap_or_default();
        if shape != &actual {
            return Err(format!("parse_shape expected {shape:?}, actual {actual:?}"));
        }
    }
    if let Some(Value::Array(tokens)) = expected.get("tokens") {
        let expected = tokens
            .iter()
            .map(value_string)
            .collect::<Result<Vec<_>, _>>()?;
        let actual = lex(source.as_bytes())
            .tokens()
            .iter()
            .map(|token| format!("{:?}", token.kind).to_uppercase())
            .collect::<Vec<_>>();
        if !expected.iter().all(|token| actual.contains(token)) {
            return Err(format!(
                "tokens expected containment {expected:?}, actual {actual:?}"
            ));
        }
    }
    if expected.contains_key("float_results") {
        let conversion = convert_literals(source);
        if conversion.warnings().is_empty() {
            return Err("float_results expected precision observation, actual no warning".into());
        }
    }
    Ok(())
}

fn parse_shapes(parsed: &iris_parser::ParseResult) -> Vec<String> {
    render_parse_shapes(&parsed.program)
}

#[cfg(test)]
mod tests {
    use super::parse_shapes;
    use iris_parser::parse;
    use iris_syntax::render_parse_shapes;

    #[test]
    fn declaration_parse_shapes_match_syntax_renderer() {
        // Given
        let parsed = parse("class Pair<T, U> where T: A & B, U: C {}");

        // When
        let observed = parse_shapes(&parsed);

        // Then
        assert_eq!(observed, render_parse_shapes(&parsed.program));
    }
}
