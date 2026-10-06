use tsrzl::jsnum::{from_string, parse_pseudo_big_int};

#[test]
fn should_parse_hexadecimal_value_given_prefixed_text_when_converting_string_to_number() {
    // Arrange
    let text = "0x1F";

    // Act
    let actual = from_string(text);

    // Assert
    assert_eq!(actual, 31.0);
}

#[test]
fn should_ignore_surrounding_white_space_given_padded_decimal_when_converting_string_to_number() {
    // Arrange
    let text = " \u{a0}12.5e1\n";

    // Act
    let actual = from_string(text);

    // Assert
    assert_eq!(actual, 125.0);
}

#[test]
fn should_return_nan_given_numeric_separator_when_converting_string_to_number() {
    // Arrange
    let text = "1_000";

    // Act
    let actual = from_string(text);

    // Assert
    assert!(actual.is_nan());
}

#[test]
fn should_round_to_nearest_even_given_hexadecimal_wider_than_mantissa_when_converting_string_to_number()
 {
    // Arrange
    let text = "0x20000000000001";

    // Act
    let actual = from_string(text);

    // Assert
    assert_eq!(actual, 9_007_199_254_740_992.0);
}

#[test]
fn should_return_decimal_digits_given_binary_big_int_when_parsing_pseudo_big_int() {
    // Arrange
    let text = "0b1010n";

    // Act
    let actual = parse_pseudo_big_int(text);

    // Assert
    assert_eq!(actual, "10");
}

#[test]
fn should_return_decimal_digits_given_hexadecimal_wider_than_u128_when_parsing_pseudo_big_int() {
    // Arrange
    let text = "0x1000000000000000000000000000000000";

    // Act
    let actual = parse_pseudo_big_int(text);

    // Assert
    assert_eq!(actual, "5444517870735015415413993718908291383296");
}
