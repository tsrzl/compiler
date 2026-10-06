//! Arbitrary-precision helpers for integer literals wider than native integers.

use std::fmt::Write;

/// Converts a bigint literal, with an optional `0b`, `0o`, or `0x` prefix and `n` suffix, to
/// decimal digits without leading zeros.
#[must_use]
pub fn parse_pseudo_big_int(text: &str) -> String {
    let text = text.strip_suffix('n').unwrap_or(text);
    let Some(radix) = prefix_radix(text) else {
        let trimmed = text.trim_start_matches('0');
        return if trimmed.is_empty() { "0" } else { trimmed }.to_owned();
    };
    let mut value = BigUint::default();
    for digit in text[2..].chars().filter_map(|digit| digit.to_digit(radix)) {
        value.multiply_add(radix, digit);
    }
    value.to_decimal()
}

/// Returns the radix selected by a `0b`, `0o`, or `0x` prefix.
pub(super) fn prefix_radix(text: &str) -> Option<u32> {
    match text.as_bytes().get(..2)? {
        [b'0', b'b' | b'B'] => Some(2),
        [b'0', b'o' | b'O'] => Some(8),
        [b'0', b'x' | b'X'] => Some(16),
        _ => None,
    }
}

/// An unsigned integer stored as little-endian base-10^9 limbs.
#[derive(Debug, Default)]
struct BigUint {
    limbs: Vec<u32>,
}

const LIMB_BASE: u64 = 1_000_000_000;

impl BigUint {
    fn multiply_add(&mut self, multiplier: u32, addend: u32) {
        let mut carry = u64::from(addend);
        for limb in &mut self.limbs {
            let product = u64::from(*limb) * u64::from(multiplier) + carry;
            *limb = u32::try_from(product % LIMB_BASE).expect("limb is below its base");
            carry = product / LIMB_BASE;
        }
        while carry > 0 {
            self.limbs
                .push(u32::try_from(carry % LIMB_BASE).expect("limb is below its base"));
            carry /= LIMB_BASE;
        }
    }

    fn to_decimal(&self) -> String {
        let Some((most_significant, rest)) = self.limbs.split_last() else {
            return "0".to_owned();
        };
        let mut decimal = most_significant.to_string();
        for limb in rest.iter().rev() {
            write!(decimal, "{limb:09}").expect("writing to a string cannot fail");
        }
        decimal
    }
}
