use iris_runtime::{Kernel, Value, Visibility};
use iris_syntax::MethodDeclaration;

use crate::{EvaluationError, Value as LiteralValue, evaluate_literals};

pub(super) fn visibility(method: &MethodDeclaration) -> Visibility {
    match method.visibility {
        iris_syntax::Visibility::Public => Visibility::Public,
        iris_syntax::Visibility::Private => Visibility::Private,
        iris_syntax::Visibility::Protected => Visibility::Protected,
    }
}

pub(super) fn literal(source: &str) -> Result<Value, EvaluationError> {
    match source {
        "nil" => return Ok(Value::Nil),
        "true" => return Ok(Value::Bool(true)),
        "false" => return Ok(Value::Bool(false)),
        _ => {}
    }
    // C067 gives Bytes and ByteArray their own literal prefixes. The body is
    // decoded here rather than in the literal-only evaluator, whose observable
    // Value set is fixed by the grammar vectors.
    if let Some(bytes) = byte_literal(source)? {
        return Ok(bytes);
    }
    match evaluate_literals(source)? {
        LiteralValue::Integer(value) => value
            .parse()
            .map(Value::Integer)
            .map_err(|_| EvaluationError::UnsupportedConstruct),
        LiteralValue::Float32Bits(bits) => Ok(Value::Float32(f32::from_bits(bits))),
        LiteralValue::Float64Bits(bits) => Ok(Value::Float64(f64::from_bits(bits))),
        // IRIS-V1-COLLECTIONS-C041 makes a String an immutable sequence of
        // Unicode scalar values, which the literal converter has already
        // validated and unescaped.
        LiteralValue::String(value) => Ok(Value::Text(value)),
        LiteralValue::Array(_) => Err(EvaluationError::UnsupportedConstruct),
    }
}

/// Decodes a `IRIS-V1-COLLECTIONS-C067` byte literal.
///
/// Returns `None` when `source` is not one, so ordinary literals continue
/// unchanged. Non-ASCII text contributes its UTF-8 bytes and `\xNN` injects one
/// raw byte. `b`/`br` produce immutable Bytes; `mb`/`mbr` produce a ByteArray.
fn byte_literal(source: &str) -> Result<Option<Value>, EvaluationError> {
    let Some(prefix_end) = source.find(['"', '\'']) else {
        return Ok(None);
    };
    let (prefix, body) = source.split_at(prefix_end);
    let mutable = match prefix {
        "b" | "br" => false,
        "mb" | "mbr" => true,
        _ => return Ok(None),
    };
    let raw = prefix.ends_with('r');
    let quote = body.chars().next().unwrap_or('"');
    let body = body
        .strip_prefix(quote)
        .and_then(|body| body.strip_suffix(quote))
        .ok_or(EvaluationError::UnsupportedConstruct)?;
    let mut bytes = Vec::new();
    let mut characters = body.chars();
    while let Some(character) = characters.next() {
        if raw || character != '\\' {
            let mut buffer = [0_u8; 4];
            bytes.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
            continue;
        }
        let escape = characters
            .next()
            .ok_or(EvaluationError::UnsupportedConstruct)?;
        match escape {
            // `\xNN` injects ONE raw byte, which is why a byte literal cannot
            // simply reuse the String unescaper: the result need not be UTF-8.
            'x' => {
                let high = characters
                    .next()
                    .ok_or(EvaluationError::UnsupportedConstruct)?;
                let low = characters
                    .next()
                    .ok_or(EvaluationError::UnsupportedConstruct)?;
                let value = u8::from_str_radix(&format!("{high}{low}"), 16)
                    .map_err(|_| EvaluationError::UnsupportedConstruct)?;
                bytes.push(value);
            }
            'n' => bytes.push(b'\n'),
            'r' => bytes.push(b'\r'),
            't' => bytes.push(b'\t'),
            '0' => bytes.push(0),
            '\\' => bytes.push(b'\\'),
            '"' => bytes.push(b'"'),
            '\'' => bytes.push(b'\''),
            _ => return Err(EvaluationError::UnsupportedConstruct),
        }
    }
    Ok(Some(if mutable {
        Value::ByteArray(iris_runtime::ByteArrayRef::new(bytes))
    } else {
        Value::Bytes(bytes)
    }))
}

pub(super) fn builtin(name: &str, kernel: &Kernel) -> Option<Value> {
    match name {
        "nil" => Some(Value::Nil),
        "true" => Some(Value::Bool(true)),
        "false" => Some(Value::Bool(false)),
        "Object" => kernel
            .class(iris_runtime::BuiltinClass::Object)
            .ok()
            .map(Value::Class),
        "Integer" => kernel
            .class(iris_runtime::BuiltinClass::Integer)
            .ok()
            .map(Value::Class),
        "String" => kernel
            .class(iris_runtime::BuiltinClass::String)
            .ok()
            .map(Value::Class),
        "Nil" => kernel
            .class(iris_runtime::BuiltinClass::Nil)
            .ok()
            .map(Value::Class),
        "Bool" => kernel
            .class(iris_runtime::BuiltinClass::Bool)
            .ok()
            .map(Value::Class),
        "Float32" => kernel
            .class(iris_runtime::BuiltinClass::Float32)
            .ok()
            .map(Value::Class),
        "Float64" => kernel
            .class(iris_runtime::BuiltinClass::Float64)
            .ok()
            .map(Value::Class),
        // C125 fixes `Transformation.empty` as the transformation of a
        // decorator that changes nothing. The name itself denotes that empty
        // transformation, and `empty` on it answers itself, so the minimal
        // surface needs no separate Transformation class object.
        "Transformation" => Some(Value::Transformation {
            kind: "class",
            staged: Vec::new(),
        }),
        _ => None,
    }
}
