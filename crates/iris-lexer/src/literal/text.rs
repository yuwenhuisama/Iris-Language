use super::{Literal, Segment};

pub(super) fn convert_string(source: &str) -> Segment {
    let bytes = source.as_bytes();
    let (raw, fence, quote_start) = raw_header(bytes);
    if fence > 255 {
        return Segment::invalid(fence + 2, "LEX_BAD_RAW_FENCE");
    }
    let quote = bytes[quote_start];
    let triple =
        bytes.get(quote_start + 1) == Some(&quote) && bytes.get(quote_start + 2) == Some(&quote);
    let opener = if triple { 3 } else { 1 };
    let content_start = quote_start + opener;
    let Some(close) = closing(bytes, content_start, quote, triple, fence) else {
        return Segment::invalid(
            bytes.len(),
            if raw {
                "LEX_BAD_RAW_FENCE"
            } else {
                "LEX_BAD_ESCAPE"
            },
        );
    };
    let content = &source[content_start..close];
    let value = if raw {
        content.to_owned()
    } else {
        let Some(value) = unescape(content) else {
            return Segment::invalid(close + opener + fence, "LEX_BAD_ESCAPE");
        };
        value
    };
    let value = if triple {
        match strip_indent(&value) {
            Ok(value) => value,
            Err(segment) => return segment,
        }
    } else {
        value
    };
    // `IRIS-V1-COLLECTIONS-C048` makes interpolation an EVALUATION rule: each
    // `${expr}` is evaluated left to right and converted through dynamic
    // `to_string`. The lexer therefore leaves the segments intact rather than
    // computing anything here, which also stops an escaped dollar from
    // interpolating, since unescaping no longer feeds a later scan.
    Segment::value(close + opener + fence, Literal::String(value), None)
}

fn raw_header(bytes: &[u8]) -> (bool, usize, usize) {
    if bytes.first() != Some(&b'r') {
        return (false, 0, 0);
    }
    let fence = bytes[1..].iter().take_while(|byte| **byte == b'#').count();
    (true, fence, fence + 1)
}

fn closing(bytes: &[u8], mut index: usize, quote: u8, triple: bool, fence: usize) -> Option<usize> {
    let quote_count = if triple { 3 } else { 1 };
    while index + quote_count + fence <= bytes.len() {
        if bytes[index..].starts_with(&vec![quote; quote_count])
            && bytes[index + quote_count..].starts_with(&vec![b'#'; fence])
        {
            return Some(index);
        }
        if bytes[index] == b'\\' && fence == 0 {
            index += 2;
        } else {
            index += 1;
        }
    }
    None
}

fn unescape(value: &str) -> Option<String> {
    let mut output = String::new();
    let mut characters = value.chars();
    while let Some(character) = characters.next() {
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
