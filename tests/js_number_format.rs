use tsrzl::jsnum::number_to_string;

#[test]
fn should_use_exponent_form_given_value_of_at_least_1e21_when_formatting_number() {
    // Arrange
    let value = 1.5e21;

    // Act
    let actual = number_to_string(value);

    // Assert
    assert_eq!(actual, "1.5e+21");
}

#[test]
fn should_use_positional_form_given_large_integer_below_1e21_when_formatting_number() {
    // Arrange
    let value = 123_456_789_012_345_680_000.0;

    // Act
    let actual = number_to_string(value);

    // Assert
    assert_eq!(actual, "123456789012345680000");
}

#[test]
fn should_use_leading_zeros_given_value_above_1e_minus_7_when_formatting_number() {
    // Arrange
    let value = 0.000_001_5;

    // Act
    let actual = number_to_string(value);

    // Assert
    assert_eq!(actual, "0.0000015");
}

#[test]
fn should_use_negative_exponent_given_value_below_1e_minus_6_when_formatting_number() {
    // Arrange
    let value = 1.5e-7;

    // Act
    let actual = number_to_string(value);

    // Assert
    assert_eq!(actual, "1.5e-7");
}

#[test]
fn should_print_infinity_given_overflowing_value_when_formatting_number() {
    // Arrange
    let value = f64::INFINITY;

    // Act
    let actual = number_to_string(value);

    // Assert
    assert_eq!(actual, "Infinity");
}

#[test]
fn should_round_half_to_even_given_tied_shortest_digits_when_formatting_number() {
    // Arrange
    let value = f64::from_bits(0x4302_4cf3_fde2_7eb2);

    // Act
    let actual = number_to_string(value);

    // Assert
    assert_eq!(actual, "643895050129366.2");
}
