mod numeric;
mod rounding;
mod text;

use numeric::convert_number;
use text::convert_string;

/// A source literal value produced during lexical conversion.
#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    /// A decimal rendering of an arbitrary-precision integer.
    Integer(String),
    /// An IEEE-754 binary32 literal value.
    Float32(f32),
    /// An IEEE-754 binary64 literal value.
    Float64(f64),
    /// A Unicode string literal value.
    String(String),
}

/// The literal values, lexical diagnostics, and precision warnings for source text.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LiteralConversion {
    values: Vec<Literal>,
    diagnostics: Vec<&'static str>,
    warnings: Vec<&'static str>,
}

impl LiteralConversion {
    /// Returns successfully converted literal values.
    pub fn values(&self) -> &[Literal] {
        &self.values
    }

    /// Returns stable lexical diagnostic codes.
    pub fn diagnostics(&self) -> &[&'static str] {
        &self.diagnostics
    }

    /// Returns stable precision warning codes.
    pub fn warnings(&self) -> &[&'static str] {
        &self.warnings
    }
}

/// Converts standalone source literal segments without evaluating expressions.
/// The end of a `IRIS-V1-COLLECTIONS-C023` Regex literal starting at `index`.
///
/// Answers `None` when the slash does not open one, so ordinary division is
/// left alone.
fn regex_literal_end(bytes: &[u8], index: usize) -> Option<usize> {
    let mut cursor = index + 1;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'\\' => cursor += 2,
            b'/' => {
                cursor += 1;
                while bytes.get(cursor).is_some_and(u8::is_ascii_alphabetic) {
                    cursor += 1;
                }
                return Some(cursor);
            }
            b'\n' | b'\r' => return None,
            _ => cursor += 1,
        }
    }
    None
}

pub fn convert_literals(source: &str) -> LiteralConversion {
    let mut conversion = LiteralConversion::default();
    let bytes = source.as_bytes();
    let mut index = 0;
    let mut pending_string = String::new();
    // An empty accumulator cannot distinguish "no string seen" from "an EMPTY
    // string seen", so `""` produced no value at all and could not be
    // evaluated. C041 makes a String a sequence of scalars, and the empty
    // sequence is one of them.
    let mut saw_string = false;
    while index < bytes.len() {
        if bytes[index].is_ascii_whitespace() {
            index += 1;
            continue;
        }
        if b"[],;".contains(&bytes[index]) {
            flush_string(&mut conversion, &mut pending_string, &mut saw_string);
            index += 1;
            continue;
        }
        if is_identifier_start(bytes[index]) && !starts_string(bytes, index) {
            index = identifier_end(bytes, index);
            continue;
        }
        // `IRIS-V1-COLLECTIONS-C023` makes `/pattern/flags` a Regex literal.
        // This pass walks the whole source without token context, so a class
        // such as `[0-9]` inside one was read as a numeric literal and reported
        // LEX_INVALID_RADIX_DIGIT for the entire source. A Regex is skipped
        // whole; the scanner owns its tokenization.
        if bytes[index] == b'/'
            && let Some(end) = regex_literal_end(bytes, index)
        {
            flush_string(&mut conversion, &mut pending_string, &mut saw_string);
            index = end;
            continue;
        }
        let result = match bytes[index] {
            b'\'' | b'"' | b'r' if starts_string(bytes, index) => convert_string(&source[index..]),
            byte if byte.is_ascii_digit()
                || (byte == b'.'
                    && (next_is_digit(bytes, index) || bytes.get(index + 1) == Some(&b'_'))) =>
            {
                flush_string(&mut conversion, &mut pending_string, &mut saw_string);
                convert_number(&source[index..])
            }
            _ => {
                index += 1;
                continue;
            }
        };
        index += result.width;
        match result.value {
            Some(Literal::String(value)) => {
                pending_string.push_str(&value);
                saw_string = true;
            }
            Some(value) => conversion.values.push(value),
            None => {}
        }
        if let Some(code) = result.diagnostic {
            conversion.diagnostics.push(code);
        }
        if let Some(warning) = result.warning {
            conversion.warnings.push(warning);
        }
    }
    flush_string(&mut conversion, &mut pending_string, &mut saw_string);
    conversion
}

pub(crate) fn numeric_literal_width(source: &str) -> usize {
    numeric::convert_number(source).width
}

pub(crate) struct Segment {
    width: usize,
    value: Option<Literal>,
    diagnostic: Option<&'static str>,
    warning: Option<&'static str>,
}

impl Segment {
    pub(crate) const fn invalid(width: usize, diagnostic: &'static str) -> Self {
        Self {
            width,
            value: None,
            diagnostic: Some(diagnostic),
            warning: None,
        }
    }

    pub(crate) const fn value(width: usize, value: Literal, warning: Option<&'static str>) -> Self {
        Self {
            width,
            value: Some(value),
            diagnostic: None,
            warning,
        }
    }
}

fn flush_string(conversion: &mut LiteralConversion, pending: &mut String, saw: &mut bool) {
    if *saw {
        conversion
            .values
            .push(Literal::String(std::mem::take(pending)));
        *saw = false;
    }
}

fn starts_string(bytes: &[u8], index: usize) -> bool {
    matches!(bytes[index], b'\'' | b'"')
        || bytes[index] == b'r' && matches!(bytes.get(index + 1), Some(b'\'' | b'"' | b'#'))
}

fn next_is_digit(bytes: &[u8], index: usize) -> bool {
    bytes.get(index + 1).is_some_and(u8::is_ascii_digit)
}

fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn identifier_end(bytes: &[u8], mut index: usize) -> usize {
    while bytes
        .get(index)
        .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
    {
        index += 1;
    }
    index
}
