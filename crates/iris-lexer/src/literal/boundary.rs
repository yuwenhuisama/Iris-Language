use crate::TokenKind;

#[derive(Clone, Copy)]
struct RecursionBudget(u8);

impl RecursionBudget {
    const ROOT: Self = Self(64);

    const fn enter(self) -> Result<Self, &'static str> {
        match self.0.checked_sub(1) {
            Some(remaining) => Ok(Self(remaining)),
            None => Err("LEX_RESOURCE_LIMIT"),
        }
    }
}

pub(crate) struct StringHeader {
    pub kind: TokenKind,
    pub raw: bool,
    pub fence: usize,
    pub quote_start: usize,
    pub quote: u8,
    pub quote_count: usize,
}

pub(crate) fn string_header(bytes: &[u8]) -> Result<Option<StringHeader>, &'static str> {
    let prefix_end = bytes
        .iter()
        .take_while(|byte| byte.is_ascii_alphabetic())
        .count();
    let prefix = &bytes[..prefix_end];
    let fence = bytes[prefix_end..]
        .iter()
        .take_while(|byte| **byte == b'#')
        .count();
    let quote_start = prefix_end + fence;
    let Some(&quote @ (b'\'' | b'"')) = bytes.get(quote_start) else {
        return Ok(None);
    };
    let (kind, raw) = match prefix {
        b"" => (TokenKind::StringLiteral, false),
        b"r" => (TokenKind::StringLiteral, true),
        b"m" => (TokenKind::MutableStringLiteral, false),
        b"mr" => (TokenKind::MutableStringLiteral, true),
        b"b" => (TokenKind::BytesLiteral, false),
        b"br" => (TokenKind::BytesLiteral, true),
        b"mb" => (TokenKind::ByteArrayLiteral, false),
        b"mbr" => (TokenKind::ByteArrayLiteral, true),
        b"rm" | b"bm" | b"rb" | b"rbm" | b"brm" => return Err("LEX_BAD_LITERAL_PREFIX"),
        _ => return Ok(None),
    };
    if fence > 255 || (fence > 0 && !raw) {
        return Err("LEX_BAD_RAW_FENCE");
    }
    let quote_count = if bytes.get(quote_start + 1) == Some(&quote)
        && bytes.get(quote_start + 2) == Some(&quote)
    {
        3
    } else {
        1
    };
    Ok(Some(StringHeader {
        kind,
        raw,
        fence,
        quote_start,
        quote,
        quote_count,
    }))
}

pub(crate) fn string_close(bytes: &[u8], header: &StringHeader) -> Result<usize, &'static str> {
    string_close_with_budget(bytes, header, RecursionBudget::ROOT)
}

fn string_close_with_budget(
    bytes: &[u8],
    header: &StringHeader,
    budget: RecursionBudget,
) -> Result<usize, &'static str> {
    let budget = budget.enter()?;
    let mut index = header.quote_start + header.quote_count;
    while index < bytes.len() {
        if bytes[index..].starts_with(&[header.quote; 3][..header.quote_count])
            && bytes
                .get(index + header.quote_count..index + header.quote_count + header.fence)
                .is_some_and(|fence| fence.iter().all(|byte| *byte == b'#'))
        {
            return Ok(index);
        }
        match bytes[index] {
            b'\\' if !header.raw => index += 2,
            b'$' if !header.raw && header.quote == b'"' && bytes.get(index + 1) == Some(&b'{') => {
                index = interpolation_end_with_budget(bytes, index + 2, budget)?;
            }
            b'\n' | b'\r' if header.quote_count == 1 => break,
            _ => index += 1,
        }
    }
    Err(if header.raw {
        "LEX_BAD_RAW_FENCE"
    } else {
        "LEX_BAD_ESCAPE"
    })
}

pub(crate) fn regex_end(bytes: &[u8], raw: bool) -> Result<usize, &'static str> {
    regex_end_with_budget(bytes, raw, RecursionBudget::ROOT)
}

fn regex_end_with_budget(
    bytes: &[u8],
    raw: bool,
    budget: RecursionBudget,
) -> Result<usize, &'static str> {
    let budget = budget.enter()?;
    let mut index = usize::from(raw);
    let fence = bytes[index..]
        .iter()
        .take_while(|byte| **byte == b'#')
        .count();
    if fence > 255 || (fence > 0 && !raw) {
        return Err("LEX_UNTERMINATED_LITERAL");
    }
    index += fence;
    if bytes.get(index) != Some(&b'/') {
        return Err("LEX_UNTERMINATED_LITERAL");
    }
    index += 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 2,
            b'$' if !raw && bytes.get(index + 1) == Some(&b'{') => {
                index = interpolation_end_with_budget(bytes, index + 2, budget)?;
            }
            b'/' if bytes
                .get(index + 1..index + 1 + fence)
                .is_some_and(|suffix| suffix.iter().all(|byte| *byte == b'#')) =>
            {
                index += 1 + fence;
                while bytes.get(index).is_some_and(u8::is_ascii_alphabetic) {
                    index += 1;
                }
                return Ok(index);
            }
            b'\n' | b'\r' => return Err("LEX_UNTERMINATED_LITERAL"),
            _ => index += 1,
        }
    }
    Err("LEX_UNTERMINATED_LITERAL")
}

pub(crate) fn starts_regex(bytes: &[u8]) -> bool {
    match bytes {
        [b'/', ..] => true,
        [b'r', remaining @ ..] => remaining.iter().find(|byte| **byte != b'#') == Some(&b'/'),
        _ => false,
    }
}

pub(crate) fn interpolation_end(bytes: &[u8], index: usize) -> Result<usize, &'static str> {
    interpolation_end_with_budget(bytes, index, RecursionBudget::ROOT)
}

fn interpolation_end_with_budget(
    bytes: &[u8],
    mut index: usize,
    budget: RecursionBudget,
) -> Result<usize, &'static str> {
    let budget = budget.enter()?;
    let mut depth = 1;
    let mut expression_start = true;
    while index < bytes.len() {
        if let Some(end) = comment_end(bytes, index) {
            index = end;
            continue;
        }
        if let Some(header) = string_header(&bytes[index..]).map_err(interpolation_error)? {
            index += string_close_with_budget(&bytes[index..], &header, budget)
                .map_err(interpolation_error)?
                + header.quote_count
                + header.fence;
            expression_start = false;
            continue;
        }
        if expression_start && starts_regex(&bytes[index..]) {
            index += regex_end_with_budget(&bytes[index..], bytes[index] == b'r', budget)?;
            expression_start = false;
            continue;
        }
        match bytes[index] {
            b'{' => {
                depth += 1;
                expression_start = true;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(index + 1);
                }
                expression_start = false;
            }
            b' ' | b'\t' | b'\r' | b'\n' => {}
            byte => expression_start = b"(,[=:+-*%!~&|^<>;".contains(&byte),
        }
        index += 1;
    }
    Err("LEX_UNTERMINATED_LITERAL")
}

fn interpolation_error(code: &'static str) -> &'static str {
    match code {
        "LEX_RESOURCE_LIMIT" => code,
        _ => "LEX_UNTERMINATED_LITERAL",
    }
}

pub(crate) fn comment_end(bytes: &[u8], start: usize) -> Option<usize> {
    let remaining = &bytes[start..];
    if remaining.starts_with(b"//") || remaining.starts_with(b"#!") {
        return Some(
            start
                + remaining
                    .iter()
                    .position(|byte| matches!(byte, b'\n' | b'\r'))
                    .unwrap_or(remaining.len()),
        );
    }
    if !remaining.starts_with(b"/*") {
        return None;
    }
    Some(crate::scanner::comment_end(bytes, start).unwrap_or(bytes.len()))
}
