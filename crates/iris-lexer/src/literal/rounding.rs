/// The exact rounded result of a hexadecimal source float.
pub(super) enum HexFloat {
    /// A directly rounded binary32 value.
    Float32(f32),
    /// A directly rounded binary64 value.
    Float64(f64),
}

/// Parses a hexadecimal significand and rounds it exactly once to an IEEE target width.
pub(super) fn round_hexadecimal(source: &str, width: u8) -> Option<HexFloat> {
    let format = Format::for_width(width)?;
    let (significand, exponent) = source[2..].split_once(['p', 'P'])?;
    let exponent = exponent.parse::<i64>().ok()?;
    let mut fraction_digits = 0_i64;
    let mut fraction = false;
    let mut value = BigUnsigned::zero();
    for byte in significand.bytes() {
        if byte == b'.' {
            if fraction {
                return None;
            }
            fraction = true;
            continue;
        }
        value.multiply_add(16, hex_digit(byte)?);
        if fraction {
            fraction_digits += 1;
        }
    }
    let scale = exponent.checked_sub(fraction_digits.checked_mul(4)?)?;
    let bits = round_to_bits(&value, scale, format);
    Some(match width {
        32 => HexFloat::Float32(f32::from_bits(bits as u32)),
        64 => HexFloat::Float64(f64::from_bits(bits)),
        _ => return None,
    })
}

#[derive(Clone, Copy)]
struct Format {
    precision: i64,
    minimum_exponent: i64,
    maximum_exponent: i64,
    exponent_bias: u64,
    fraction_bits: u64,
}

impl Format {
    const fn for_width(width: u8) -> Option<Self> {
        match width {
            32 => Some(Self {
                precision: 24,
                minimum_exponent: -126,
                maximum_exponent: 127,
                exponent_bias: 127,
                fraction_bits: 23,
            }),
            64 => Some(Self {
                precision: 53,
                minimum_exponent: -1022,
                maximum_exponent: 1023,
                exponent_bias: 1023,
                fraction_bits: 52,
            }),
            _ => None,
        }
    }
}

fn round_to_bits(value: &BigUnsigned, scale: i64, format: Format) -> u64 {
    if value.is_zero() {
        return 0;
    }
    let highest = value.bit_length() as i64 - 1;
    let mut exponent = highest.saturating_add(scale);
    if exponent > format.maximum_exponent {
        return infinity_bits(format);
    }
    if exponent >= format.minimum_exponent {
        let shift = highest - (format.precision - 1);
        let mut significand = rounded_shift(value, shift);
        if significand == (1_u64 << format.precision) {
            significand >>= 1;
            exponent += 1;
        }
        if exponent > format.maximum_exponent {
            return infinity_bits(format);
        }
        return normal_bits(significand, exponent, format);
    }
    let unit_exponent = format.minimum_exponent - (format.precision - 1);
    let significand = rounded_shift(value, unit_exponent.saturating_sub(scale));
    if significand == 0 {
        return 0;
    }
    if significand >= (1_u64 << (format.precision - 1)) {
        return 1_u64 << format.fraction_bits;
    }
    significand
}

fn normal_bits(significand: u64, exponent: i64, format: Format) -> u64 {
    let exponent_bits = (exponent + format.exponent_bias as i64) as u64;
    let fraction = significand & ((1_u64 << format.fraction_bits) - 1);
    (exponent_bits << format.fraction_bits) | fraction
}

fn infinity_bits(format: Format) -> u64 {
    (format.exponent_bias * 2 + 1) << format.fraction_bits
}

fn rounded_shift(value: &BigUnsigned, shift: i64) -> u64 {
    if shift <= 0 {
        return value.low_u64() << (-shift as u32);
    }
    let truncated = value.shifted_u64(shift as usize);
    let halfway = value.bit(shift as usize - 1);
    let sticky = value.any_lower_bits(shift as usize - 1);
    if halfway && (sticky || truncated & 1 == 1) {
        truncated + 1
    } else {
        truncated
    }
}

#[derive(Clone)]
struct BigUnsigned {
    limbs: Vec<u32>,
}

impl BigUnsigned {
    fn zero() -> Self {
        Self { limbs: vec![0] }
    }

    fn is_zero(&self) -> bool {
        self.limbs.iter().all(|limb| *limb == 0)
    }

    fn multiply_add(&mut self, multiplier: u32, addend: u32) {
        let mut carry = u64::from(addend);
        for limb in &mut self.limbs {
            let product = u64::from(*limb) * u64::from(multiplier) + carry;
            *limb = product as u32;
            carry = product >> 32;
        }
        if carry != 0 {
            self.limbs.push(carry as u32);
        }
    }

    fn bit_length(&self) -> usize {
        let Some((index, limb)) = self
            .limbs
            .iter()
            .enumerate()
            .rev()
            .find(|(_, limb)| **limb != 0)
        else {
            return 0;
        };
        index * 32 + (32 - limb.leading_zeros() as usize)
    }

    fn low_u64(&self) -> u64 {
        u64::from(self.limbs[0]) | self.limbs.get(1).map_or(0, |limb| u64::from(*limb) << 32)
    }

    fn shifted_u64(&self, shift: usize) -> u64 {
        let limb = shift / 32;
        let bits = shift % 32;
        let first = u64::from(self.limbs.get(limb).copied().unwrap_or(0));
        let second = u64::from(self.limbs.get(limb + 1).copied().unwrap_or(0));
        let third = u64::from(self.limbs.get(limb + 2).copied().unwrap_or(0));
        if bits == 0 {
            first | (second << 32)
        } else {
            (first >> bits) | (second << (32 - bits)) | (third << (64 - bits))
        }
    }

    fn bit(&self, index: usize) -> bool {
        self.limbs
            .get(index / 32)
            .is_some_and(|limb| limb & (1 << (index % 32)) != 0)
    }

    fn any_lower_bits(&self, end: usize) -> bool {
        let complete_limbs = end / 32;
        self.limbs[..complete_limbs].iter().any(|limb| *limb != 0)
            || (!end.is_multiple_of(32)
                && self
                    .limbs
                    .get(complete_limbs)
                    .is_some_and(|limb| limb & ((1 << (end % 32)) - 1) != 0))
    }
}

fn hex_digit(byte: u8) -> Option<u32> {
    match byte {
        b'0'..=b'9' => Some(u32::from(byte - b'0')),
        b'a'..=b'f' => Some(u32::from(byte - b'a') + 10),
        b'A'..=b'F' => Some(u32::from(byte - b'A') + 10),
        _ => None,
    }
}
