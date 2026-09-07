use super::rounding::{HexFloat, round_hexadecimal};
use super::{Literal, Segment};

pub(crate) fn convert_number(source: &str) -> Segment {
    let width = candidate_width(source.as_bytes());
    let candidate = &source[..width];
    if has_radix_prefix(candidate) && candidate.as_bytes().get(2) == Some(&b'_') {
        return Segment::invalid(width, "LEX_EMPTY_RADIX_PREFIX");
    }
    if let Some(code) = separator_error(candidate) {
        return Segment::invalid(width, code);
    }
    let compact: String = candidate
        .chars()
        .filter(|character| *character != '_')
        .collect();
    if has_radix_prefix(&compact) && compact.len() == 2 {
        return Segment::invalid(width, "LEX_EMPTY_RADIX_PREFIX");
    }
    if let Some((body, suffix)) = float_suffix(&compact) {
        return convert_float(body, suffix, width);
    }
    if is_hex_float(&compact) {
        return convert_float(&compact, None, width);
    }
    if let Some(code) = radix_error(&compact) {
        return Segment::invalid(width, code);
    }
    if is_float(&compact) {
        return convert_float(&compact, None, width);
    }
    match BigInteger::from_radix(&compact, radix(&compact)) {
        Some(value) => Segment::value(width, Literal::Integer(value.decimal()), None),
        None => Segment::invalid(width, "LEX_INVALID_RADIX_DIGIT"),
    }
}

fn candidate_width(bytes: &[u8]) -> usize {
    let exponent_markers: &[u8] = match bytes {
        [b'0', b'x' | b'X', ..] => b"pP",
        [b'0', b'b' | b'B' | b'o' | b'O', ..] => b"",
        _ => b"eE",
    };
    let mut width = 0;
    while let Some(byte) = bytes.get(width) {
        if *byte == b'.' && bytes.get(width + 1) == Some(&b'.') {
            break;
        }
        let exponent_sign = matches!(byte, b'+' | b'-')
            && width > 0
            && exponent_markers.contains(&bytes[width - 1]);
        if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.') || exponent_sign {
            width += 1;
        } else {
            break;
        }
    }
    width
}

fn has_radix_prefix(value: &str) -> bool {
    matches!(
        value.as_bytes(),
        [b'0', b'b' | b'B' | b'o' | b'O' | b'x' | b'X', ..]
    )
}

fn radix(value: &str) -> u32 {
    match value.as_bytes() {
        [b'0', b'b' | b'B', ..] => 2,
        [b'0', b'o' | b'O', ..] => 8,
        [b'0', b'x' | b'X', ..] => 16,
        _ => 10,
    }
}

fn radix_error(value: &str) -> Option<&'static str> {
    if !has_radix_prefix(value) {
        return None;
    }
    let digits = &value[2..];
    if digits.is_empty() {
        return Some("LEX_EMPTY_RADIX_PREFIX");
    }
    let base = radix(value);
    if digits
        .bytes()
        .any(|byte| digit(byte).is_none_or(|digit| digit >= base))
    {
        return Some("LEX_INVALID_RADIX_DIGIT");
    }
    None
}

fn separator_error(value: &str) -> Option<&'static str> {
    let bytes = value.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if *byte != b'_' {
            continue;
        }
        let before = index
            .checked_sub(1)
            .and_then(|position| bytes.get(position));
        let after = bytes.get(index + 1);
        if !before.is_some_and(u8::is_ascii_alphanumeric)
            || !after.is_some_and(u8::is_ascii_alphanumeric)
            || matches!(
                before,
                Some(b'e' | b'E' | b'p' | b'P' | b'x' | b'X' | b'b' | b'B' | b'o' | b'O')
            )
        {
            return Some("LEX_BAD_NUMERIC_SEPARATOR");
        }
    }
    None
}

fn float_suffix(value: &str) -> Option<(&str, Option<u8>)> {
    if let Some(body) = value.strip_suffix("f32") {
        Some((body, Some(32)))
    } else if let Some(body) = value.strip_suffix("f64") {
        Some((body, Some(64)))
    } else if value.ends_with("F32") || value.ends_with("F64") {
        Some((value, Some(0)))
    } else {
        None
    }
}

fn is_float(value: &str) -> bool {
    if value.starts_with("0x") || value.starts_with("0X") {
        return value.contains('p') || value.contains('P');
    }
    value.contains('.') || value.contains('e') || value.contains('E')
}

fn is_hex_float(value: &str) -> bool {
    (value.starts_with("0x") || value.starts_with("0X"))
        && (value.contains('p') || value.contains('P'))
}

fn convert_float(source: &str, suffix: Option<u8>, width: usize) -> Segment {
    if suffix == Some(0) {
        return Segment::invalid(width, "LEX_BAD_FLOAT_SUFFIX");
    }
    match suffix {
        Some(32) => {
            let value = if is_hex_float(source) {
                let Some(HexFloat::Float32(value)) = round_hexadecimal(source, 32) else {
                    return Segment::invalid(width, "LEX_BAD_FLOAT_SUFFIX");
                };
                value
            } else {
                let Ok(value) = source.parse::<f32>() else {
                    return Segment::invalid(width, "LEX_BAD_FLOAT_SUFFIX");
                };
                value
            };
            let warning = if source_is_nonzero(source) && (value.is_infinite() || value == 0.0) {
                Some("LEX_PRECISION_LOSS")
            } else {
                None
            };
            Segment::value(width, Literal::Float32(value), warning)
        }
        None | Some(64) => {
            let number = if is_hex_float(source) {
                let Some(HexFloat::Float64(value)) = round_hexadecimal(source, 64) else {
                    return Segment::invalid(width, "LEX_BAD_FLOAT_SUFFIX");
                };
                value
            } else {
                let Ok(value) = source.parse::<f64>() else {
                    return Segment::invalid(width, "LEX_BAD_FLOAT_SUFFIX");
                };
                value
            };
            let warning = if source_is_nonzero(source) && (number.is_infinite() || number == 0.0) {
                Some("LEX_PRECISION_LOSS")
            } else {
                None
            };
            Segment::value(width, Literal::Float64(number), warning)
        }
        Some(_) => Segment::invalid(width, "LEX_BAD_FLOAT_SUFFIX"),
    }
}

fn source_is_nonzero(value: &str) -> bool {
    value
        .bytes()
        .any(|byte| matches!(byte, b'1'..=b'9' | b'a'..=b'f' | b'A'..=b'F'))
}

fn digit(byte: u8) -> Option<u32> {
    match byte {
        b'0'..=b'9' => Some(u32::from(byte - b'0')),
        b'a'..=b'f' => Some(u32::from(byte - b'a') + 10),
        b'A'..=b'F' => Some(u32::from(byte - b'A') + 10),
        _ => None,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BigInteger {
    limbs: Vec<u32>,
}

impl BigInteger {
    fn from_radix(value: &str, radix: u32) -> Option<Self> {
        let digits = if radix == 10 { value } else { &value[2..] };
        let mut result = Self { limbs: vec![0] };
        for byte in digits.bytes() {
            result.multiply_add(radix, digit(byte)?);
        }
        Some(result)
    }

    fn multiply_add(&mut self, multiplier: u32, addend: u32) {
        let mut carry = u64::from(addend);
        for limb in &mut self.limbs {
            let value = u64::from(*limb) * u64::from(multiplier) + carry;
            *limb = value as u32;
            carry = value >> 32;
        }
        if carry != 0 {
            self.limbs.push(carry as u32);
        }
    }

    fn decimal(&self) -> String {
        let mut value = self.clone();
        let mut digits = Vec::new();
        while value.limbs.iter().any(|limb| *limb != 0) {
            digits.push(char::from(b'0' + value.divide_ten()));
        }
        if digits.is_empty() {
            return "0".into();
        }
        digits.iter().rev().collect()
    }

    fn divide_ten(&mut self) -> u8 {
        let mut remainder = 0_u64;
        for limb in self.limbs.iter_mut().rev() {
            let value = (remainder << 32) | u64::from(*limb);
            *limb = (value / 10) as u32;
            remainder = value % 10;
        }
        remainder as u8
    }
}
