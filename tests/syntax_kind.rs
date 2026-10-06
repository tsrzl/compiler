use tsrzl::ast::{SyntaxKind, TokenFlags};

#[test]
fn should_classify_break_as_reserved_word_given_break_keyword_when_checking_kind() {
    // Arrange
    let kind = SyntaxKind::BreakKeyword;

    // Act
    let actual = kind.is_reserved_word();

    // Assert
    assert!(actual);
}

#[test]
fn should_expose_typescript_go_bit_values_given_combined_token_flags_when_reading_bits() {
    // Arrange
    let flags = TokenFlags::SCIENTIFIC | TokenFlags::OCTAL;

    // Act
    let actual = flags.bits();

    // Assert
    assert_eq!(actual, 48);
}
