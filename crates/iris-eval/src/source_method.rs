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
    // C052 makes an `m` literal follow the corresponding String literal family
    // for content, interpolation and indentation BEFORE the MutableString value
    // is created, so the body is delegated rather than re-parsed.
    if let Some(regex) = regex_literal(source)? {
        return Ok(regex);
    }
    if let Some(text) = mutable_string_literal(source) {
        let Value::Text(text) = literal(&text)? else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        return Ok(Value::MutableString(iris_runtime::MutableStringRef::new(
            text,
        )));
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

/// Decodes a `IRIS-V1-COLLECTIONS-C076` Regex literal.
///
/// Returns `None` when `source` is not one. `C080` fixes the flag set as
/// exactly `i`, `m`, `s` and `x` and requires a duplicate or unsupported flag to
/// be diagnosed, and `C081` stores the accepted flags in canonical `imsx` order
/// so `/a/im` and `/a/mi` are equal.
fn regex_literal(source: &str) -> Result<Option<Value>, EvaluationError> {
    let body = match source.strip_prefix("r/") {
        Some(body) => body,
        None => match source.strip_prefix('/') {
            Some(body) => body,
            None => return Ok(None),
        },
    };
    let Some(close) = body.rfind('/') else {
        return Ok(None);
    };
    let (pattern, flags) = body.split_at(close);
    let flags = &flags[1..];
    let mut seen = Vec::new();
    for flag in flags.chars() {
        // C080 rejects an unsupported flag, and C081 makes a DUPLICATE invalid
        // before canonicalization rather than something to deduplicate.
        if !matches!(flag, 'i' | 'm' | 's' | 'x') || seen.contains(&flag) {
            return Err(EvaluationError::LexicalDiagnostic("LEX_BAD_REGEX_FLAGS"));
        }
        seen.push(flag);
    }
    let canonical: String = "imsx".chars().filter(|flag| seen.contains(flag)).collect();
    // C078 fixes the supported subset and C079 forbids backreferences,
    // lookbehind and other constructs needing unbounded backtracking. The
    // engine enforces exactly that subset, so an unsupported construct is
    // rejected here rather than accepted and mis-executed.
    compiled_regex(pattern, &canonical)?;
    Ok(Some(Value::Regex(Box::new(iris_runtime::RegexValue {
        pattern: pattern.to_owned(),
        flags: canonical,
    }))))
}

/// Compiles a canonical pattern and flags into an engine Regex.
///
/// `IRIS-V1-COLLECTIONS-C080` maps each flag onto its engine equivalent, and
/// Unicode mode is always on and fixed by the language major.
pub(crate) fn compiled_regex(
    pattern: &str,
    canonical_flags: &str,
) -> Result<regex::Regex, EvaluationError> {
    regex::RegexBuilder::new(pattern)
        .case_insensitive(canonical_flags.contains('i'))
        .multi_line(canonical_flags.contains('m'))
        .dot_matches_new_line(canonical_flags.contains('s'))
        .ignore_whitespace(canonical_flags.contains('x'))
        .unicode(true)
        .build()
        // C079 names the unsupported constructs individually, and V353 requires
        // a DISTINCT diagnostic per construct rather than one generic Regex
        // failure. The engine already reports which one it refused, so the code
        // is derived from that rather than guessed from the pattern text.
        .map_err(|error| {
            let reported = error.to_string();
            if reported.contains("backreference") {
                EvaluationError::LexicalDiagnostic("REGEX_UNSUPPORTED_BACKREFERENCE")
            } else if reported.contains("look-around")
                || reported.contains("look-behind")
                || reported.contains("look-ahead")
            {
                EvaluationError::LexicalDiagnostic("REGEX_UNSUPPORTED_LOOKBEHIND")
            } else {
                EvaluationError::RegexSyntaxError
            }
        })
}

/// Strips a `IRIS-V1-COLLECTIONS-C052` MutableString prefix.
///
/// Returns the equivalent String literal source, so the content rules are
/// applied by the String path rather than duplicated here. Returns `None` when
/// `source` is not an `m` literal.
fn mutable_string_literal(source: &str) -> Option<String> {
    let prefix_end = source.find(['"', '\''])?;
    let (prefix, body) = source.split_at(prefix_end);
    match prefix {
        "m" => Some(body.to_owned()),
        "mr" => Some(format!("r{body}")),
        _ => None,
    }
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
