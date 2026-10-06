//! ECMAScript `Number::toString` formatting.

/// Formats `value` as ECMAScript `Number.prototype.toString()` does for radix 10.
#[must_use]
pub fn number_to_string(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_owned();
    }
    if value == 0.0 {
        return "0".to_owned();
    }
    if value.is_infinite() {
        return if value > 0.0 { "Infinity" } else { "-Infinity" }.to_owned();
    }
    if value < 0.0 {
        return format!("-{}", number_to_string(-value));
    }

    let scientific = shortest_scientific(value);
    let (mantissa, exponent) = scientific
        .split_once('e')
        .expect("scientific formatting contains an exponent");
    let digits = mantissa.replace('.', "");
    let digit_count = i32::try_from(digits.len()).expect("f64 has at most 17 significant digits");
    let point = exponent.parse::<i32>().expect("exponent is an integer") + 1;

    if digit_count <= point && point <= 21 {
        let zeros = usize::try_from(point - digit_count).expect("point is past the digits");
        return format!("{digits}{}", "0".repeat(zeros));
    }
    if 0 < point && point <= 21 {
        let split = usize::try_from(point).expect("point is positive");
        return format!("{}.{}", &digits[..split], &digits[split..]);
    }
    if -6 < point && point <= 0 {
        let zeros = usize::try_from(-point).expect("point is not positive");
        return format!("0.{}{digits}", "0".repeat(zeros));
    }
    let sign = if point - 1 < 0 { '-' } else { '+' };
    let exponent = (point - 1).abs();
    match digits.split_at(1) {
        (first, "") => format!("{first}e{sign}{exponent}"),
        (first, rest) => format!("{first}.{rest}e{sign}{exponent}"),
    }
}

/// Returns the shortest round-tripping scientific form, choosing the even digit on exact ties.
fn shortest_scientific(value: f64) -> String {
    // `{:e}` yields the shortest round-tripping digit count; exact-precision formatting then
    // rounds half to even at that length, as ECMAScript and TypeScript-Go do.
    let shortest = format!("{value:e}");
    let fraction_digits = shortest
        .split_once('e')
        .map_or(0, |(mantissa, _)| mantissa.len().saturating_sub(2));
    let rounded = format!("{value:.fraction_digits$e}");
    if rounded.parse::<f64>() == Ok(value) {
        rounded
    } else {
        shortest
    }
}
