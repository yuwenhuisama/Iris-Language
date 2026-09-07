use super::boundary::{interpolation_end, string_close, string_header};
use super::{Literal, Segment};

pub(crate) fn convert_string(source: &str) -> Segment {
    let bytes = source.as_bytes();
    let header = match string_header(bytes) {
        Ok(Some(header)) => header,
        Ok(None) => return Segment::invalid(bytes.len(), "LEX_BAD_LITERAL_PREFIX"),
        Err(code) => return Segment::invalid(bytes.len(), code),
    };
    let content_start = header.quote_start + header.quote_count;
    let close = match string_close(bytes, &header) {
        Ok(close) => close,
        Err(code) => return Segment::invalid(bytes.len(), code),
    };
    let width = close + header.quote_count + header.fence;
    let content = &source[content_start..close];
    let value = if header.raw {
        content.to_owned()
    } else {
        let Some(value) = unescape(content, header.quote == b'"') else {
            return Segment::invalid(width, "LEX_BAD_ESCAPE");
        };
        value
    };
    let value = if header.quote_count == 3 {
        match strip_indent(&value) {
            Ok(value) => value,
            Err(_) => return Segment::invalid(width, "LEX_BAD_MULTILINE_INDENT"),
        }
    } else {
        value
    };
    // `IRIS-V1-COLLECTIONS-C048` makes interpolation an EVALUATION rule: each
    // `${expr}` is evaluated left to right and converted through dynamic
    // `to_string`. The lexer therefore leaves the segments intact rather than
    // computing anything here, which also stops an escaped dollar from
    // interpolating, since unescaping no longer feeds a later scan.
    Segment::value(width, Literal::String(value), None)
}

fn unescape(value: &str, interpolated: bool) -> Option<String> {
    let mut output = String::new();
    let mut characters = value.chars();
    while let Some(character) = characters.next() {
        if interpolated && character == '$' && characters.as_str().starts_with('{') {
            let remainder = characters.as_str();
            let end = interpolation_end(remainder.as_bytes(), 1).ok()?;
            output.push('$');
            output.push_str(&remainder[..end]);
            characters = remainder[end..].chars();
            continue;
        }
        if character != '\\' {
            output.push(character);
            continue;
        }
        match characters.next()? {
            '\\' => output.push('\\'),
            '"' => output.push('"'),
            '\'' => output.push('\''),
            'n' => output.push('\n'),
            'r' => output.push('\r'),
            't' => output.push('\t'),
            '0' => output.push('\0'),
            'b' => output.push('\u{0008}'),
            'f' => output.push('\u{000C}'),
            'v' => output.push('\u{000B}'),
            'x' => output.push(char::from_u32(u32::from(hex_pair(&mut characters)?))?),
            'u' => output.push(unicode_escape(&mut characters)?),
            _ => return None,
        }
    }
    Some(output)
}

fn hex_pair(characters: &mut std::str::Chars<'_>) -> Option<u8> {
    let high = hex(characters.next()?)?;
    let low = hex(characters.next()?)?;
    Some(high * 16 + low)
}

fn unicode_escape(characters: &mut std::str::Chars<'_>) -> Option<char> {
    if characters.next()? != '{' {
        return None;
    }
    let mut digits = String::new();
    for character in characters.by_ref() {
        if character == '}' {
            break;
        }
        digits.push(character);
        if digits.len() > 6 {
            return None;
        }
    }
    if digits.is_empty() {
        return None;
    }
    char::from_u32(u32::from_str_radix(&digits, 16).ok()?)
}

fn hex(character: char) -> Option<u8> {
    character.to_digit(16).map(|value| value as u8)
}

fn strip_indent(value: &str) -> Result<String, Segment> {
    if !value.starts_with('\n') {
        return Ok(value.to_owned());
    }
    let Some((body, closing_indent)) = value.rsplit_once('\n') else {
        return Ok(value.to_owned());
    };
    let mut lines = Vec::new();
    for line in body[1..].split('\n') {
        if !line.is_empty() && !line.starts_with(closing_indent) {
            return Err(Segment::invalid(value.len(), "LEX_BAD_MULTILINE_INDENT"));
        }
        lines.push(line.strip_prefix(closing_indent).unwrap_or(line));
    }
    Ok(lines.join("\n"))
}
