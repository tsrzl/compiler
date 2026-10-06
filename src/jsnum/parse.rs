//! ECMAScript `StringToNumber`, following TypeScript-Go's `jsnum.FromString`.

use super::bigint::prefix_radix;

/// Converts `text` to a number as ECMAScript `StringToNumber` does.
#[must_use]
pub fn from_string(text: &str) -> f64 {
    let text = text.trim_matches(is_str_white_space);
    match text {
        "" => return 0.0,
        "Infinity" | "+Infinity" => return f64::INFINITY,
        "-Infinity" => return f64::NEG_INFINITY,
        _ => {}
    }
    if !text.chars().all(is_number_char) {
        return f64::NAN;
    }
    if let Some(value) = try_parse_integer(text) {
        return value;
    }
    let (unsigned, negative) = match text.strip_prefix('-') {
        Some(rest) => (rest, true),
        None => (text.strip_prefix('+').unwrap_or(text), false),
    };
    if !unsigned
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_digit() || first == '.')
    {
        return f64::NAN;
    }
    let value = parse_float_text(unsigned);
    if negative { -value } else { value }
}

fn is_str_white_space(character: char) -> bool {
    matches!(
        character,
        '\n' | '\r'
            | '\u{2028}'
            | '\u{2029}'
            | '\t'
            | '\u{b}'
            | '\u{c}'
            | '\u{feff}'
            | ' '
            | '\u{a0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200a}' | '\u{202f}' | '\u{205f}' | '\u{3000}'
    )
}

fn is_number_char(character: char) -> bool {
    character.is_ascii_hexdigit() || matches!(character, '.' | '-' | '+' | 'x' | 'X' | 'o' | 'O')
}

/// Parses prefixed or plain decimal integers, returning `None` for other numeric forms.
fn try_parse_integer(text: &str) -> Option<f64> {
    if text.len() > 2
        && let Some(radix) = prefix_radix(text)
    {
        let digits = &text[2..];
        if !digits.chars().all(|digit| digit.is_digit(radix)) {
            return Some(f64::NAN);
        }
        return Some(power_of_two_radix_to_f64(digits, radix));
    }
    let digits = trim_leading_zeros(text);
    if !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    Some(digits.parse::<f64>().unwrap_or(f64::NAN))
}

/// Converts digits in radix 2, 8, or 16 to the nearest `f64`, rounding ties to even.
fn power_of_two_radix_to_f64(digits: &str, radix: u32) -> f64 {
    let bits_per_digit = radix.trailing_zeros();
    let mut mantissa = 0_u64;
    let mut dropped_bits = 0_i32;
    let mut sticky = false;
    for digit in digits.chars().filter_map(|digit| digit.to_digit(radix)) {
        for shift in (0..bits_per_digit).rev() {
            let bit = u64::from((digit >> shift) & 1);
            if mantissa.leading_zeros() > 0 {
                mantissa = (mantissa << 1) | bit;
            } else {
                dropped_bits += 1;
                sticky |= bit == 1;
            }
        }
    }
    // Folding the sticky bit into the lowest of 64 bits preserves round-half-to-even when the
    // 64-bit mantissa is rounded to 53 bits by the conversion below.
    let mantissa = if sticky { mantissa | 1 } else { mantissa };
    #[allow(clippy::cast_precision_loss)]
    let value = mantissa as f64;
    value * 2_f64.powi(dropped_bits)
}

fn parse_float_text(text: &str) -> f64 {
    let (integer, rest, has_dot) = match text.split_once('.') {
        Some((integer, rest)) => (integer, rest, true),
        None => (text, "", false),
    };
    let (integer, fraction, exponent) = if has_dot {
        let (fraction, exponent) = split_exponent(rest);
        (integer, fraction, exponent)
    } else {
        let (integer, exponent) = split_exponent(integer);
        (integer, "", exponent)
    };

    let mut normalized = String::with_capacity(text.len() + 3);
    if integer.is_empty() {
        if has_dot && fraction.is_empty() || exponent == Some("") {
            return f64::NAN;
        }
        normalized.push('0');
    } else {
        let integer = trim_leading_zeros(integer);
        if !is_all_digits(integer) {
            return f64::NAN;
        }
        normalized.push_str(integer);
    }
    if has_dot {
        normalized.push('.');
        if fraction.is_empty() {
            normalized.push('0');
        } else {
            let fraction = trim_trailing_zeros(fraction);
            if !is_all_digits(fraction) {
                return f64::NAN;
            }
            normalized.push_str(fraction);
        }
    }
    if let Some(exponent) = exponent {
        normalized.push('e');
        let exponent = match exponent.strip_prefix('-') {
            Some(rest) => {
                normalized.push('-');
                rest
            }
            None => exponent.strip_prefix('+').unwrap_or(exponent),
        };
        let exponent = trim_leading_zeros(exponent);
        if !is_all_digits(exponent) {
            return f64::NAN;
        }
        normalized.push_str(exponent);
    }
    normalized.parse::<f64>().unwrap_or(f64::NAN)
}

fn split_exponent(text: &str) -> (&str, Option<&str>) {
    match text.find(['e', 'E']) {
        Some(index) => (&text[..index], Some(&text[index + 1..])),
        None => (text, None),
    }
}

fn trim_leading_zeros(text: &str) -> &str {
    match text.trim_start_matches('0') {
        "" if !text.is_empty() => "0",
        trimmed => trimmed,
    }
}

fn trim_trailing_zeros(text: &str) -> &str {
    match text.trim_end_matches('0') {
        "" if !text.is_empty() => "0",
        trimmed => trimmed,
    }
}

fn is_all_digits(text: &str) -> bool {
    text.bytes().all(|byte| byte.is_ascii_digit())
}
