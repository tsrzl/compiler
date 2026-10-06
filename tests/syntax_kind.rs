use tsrzl::ast::SyntaxKind;

#[test]
fn should_classify_break_as_reserved_word_given_break_keyword_when_checking_kind() {
    // Arrange
    let kind = SyntaxKind::BreakKeyword;

    // Act
    let actual = kind.is_reserved_word();

    // Assert
    assert!(actual);
}
