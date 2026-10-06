use tsrzl::ast::SyntaxKind;
use tsrzl::scanner::Scanner;

#[test]
fn should_combine_unsigned_shift_assignment_given_greater_than_when_rescanning() {
    // Arrange
    let mut scanner = Scanner::new("a >>>= b");
    scanner.scan();
    scanner.scan();

    // Act
    let actual = scanner.rescan_greater_than_token();

    // Assert
    assert_eq!(
        (actual, scanner.token_end()),
        (SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken, 6)
    );
}

#[test]
fn should_split_shift_given_less_than_less_than_when_rescanning() {
    // Arrange
    let mut scanner = Scanner::new("<<T>");
    scanner.scan();

    // Act
    let actual = scanner.rescan_less_than_token();

    // Assert
    assert_eq!(
        (actual, scanner.token_end()),
        (SyntaxKind::LessThanToken, 1)
    );
}

#[test]
fn should_scan_regular_expression_given_slash_when_rescanning() {
    // Arrange
    let mut scanner = Scanner::new("/a[/]b/gi;");
    scanner.scan();

    // Act
    let actual = scanner.rescan_slash_token();

    // Assert
    assert_eq!(
        (actual, scanner.token_value()),
        (SyntaxKind::RegularExpressionLiteral, "/a[/]b/gi")
    );
}

#[test]
fn should_report_unterminated_regular_expression_given_line_break_when_rescanning() {
    // Arrange
    let mut scanner = Scanner::new("/abc;\nx");
    scanner.scan();

    // Act
    scanner.rescan_slash_token();

    // Assert
    let reported = scanner
        .diagnostics()
        .iter()
        .map(|diagnostic| (diagnostic.message().code(), diagnostic.length()))
        .collect::<Vec<_>>();
    assert_eq!(reported, [(1161, 4)]);
}

#[test]
fn should_scan_template_middle_given_closing_brace_when_rescanning_template() {
    // Arrange
    let mut scanner = Scanner::new("} b ${");
    scanner.scan();

    // Act
    let actual = scanner.rescan_template_token(false);

    // Assert
    assert_eq!(
        (actual, scanner.token_value()),
        (SyntaxKind::TemplateMiddle, " b ")
    );
}

#[test]
fn should_split_hash_given_private_identifier_when_rescanning() {
    // Arrange
    let mut scanner = Scanner::new("#x");
    scanner.scan();

    // Act
    let actual = scanner.rescan_hash_token();

    // Assert
    assert_eq!((actual, scanner.token_end()), (SyntaxKind::HashToken, 1));
}

#[test]
fn should_split_question_given_question_question_when_rescanning() {
    // Arrange
    let mut scanner = Scanner::new("??");
    scanner.scan();

    // Act
    let actual = scanner.rescan_question_token();

    // Assert
    assert_eq!(
        (actual, scanner.token_end()),
        (SyntaxKind::QuestionToken, 1)
    );
}

#[test]
fn should_split_asterisk_equals_given_generator_default_when_rescanning() {
    // Arrange
    let mut scanner = Scanner::new("*=");
    scanner.scan();

    // Act
    let actual = scanner.rescan_asterisk_equals_token();

    // Assert
    assert_eq!((actual, scanner.token_end()), (SyntaxKind::EqualsToken, 1));
}

#[test]
fn should_scan_token_at_offset_given_reset_position_when_scanning() {
    // Arrange
    let mut scanner = Scanner::new("let value = 1;");
    scanner.reset_pos(4);

    // Act
    let token = scanner.scan();

    // Assert
    assert_eq!(
        (token, scanner.token_start(), scanner.token_end()),
        (SyntaxKind::Identifier, 4, 9)
    );
}
