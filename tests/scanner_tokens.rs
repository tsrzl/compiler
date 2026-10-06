use tsrzl::ast::SyntaxKind;
use tsrzl::scanner::Scanner;

fn scan_kinds(text: &str) -> Vec<SyntaxKind> {
    let mut scanner = Scanner::new(text);
    let mut kinds = Vec::new();
    loop {
        let kind = scanner.scan();
        kinds.push(kind);
        if kind == SyntaxKind::EndOfFile {
            return kinds;
        }
    }
}

#[test]
fn should_scan_longest_punctuators_given_operator_sequence_when_scanning_tokens() {
    // Arrange
    let text = "{ ( ?? ?. ??= >>= ... !== => **= }";

    // Act
    let actual = scan_kinds(text);

    // Assert
    assert_eq!(
        actual,
        [
            SyntaxKind::OpenBraceToken,
            SyntaxKind::OpenParenToken,
            SyntaxKind::QuestionQuestionToken,
            SyntaxKind::QuestionDotToken,
            SyntaxKind::QuestionQuestionEqualsToken,
            SyntaxKind::GreaterThanToken,
            SyntaxKind::GreaterThanToken,
            SyntaxKind::EqualsToken,
            SyntaxKind::DotDotDotToken,
            SyntaxKind::ExclamationEqualsEqualsToken,
            SyntaxKind::EqualsGreaterThanToken,
            SyntaxKind::AsteriskAsteriskEqualsToken,
            SyntaxKind::CloseBraceToken,
            SyntaxKind::EndOfFile,
        ]
    );
}

#[test]
fn should_scan_keyword_given_reserved_word_when_scanning_tokens() {
    // Arrange
    let text = "const";

    // Act
    let actual = scan_kinds(text);

    // Assert
    assert_eq!(actual, [SyntaxKind::ConstKeyword, SyntaxKind::EndOfFile]);
}

#[test]
fn should_scan_identifier_given_non_ascii_identifier_when_scanning_tokens() {
    // Arrange
    let text = "café_1";

    // Act
    let actual = scan_kinds(text);

    // Assert
    assert_eq!(actual, [SyntaxKind::Identifier, SyntaxKind::EndOfFile]);
}

#[test]
fn should_return_identifier_text_given_unicode_identifier_when_reading_token_value() {
    // Arrange
    let mut scanner = Scanner::new("  café_1;");

    // Act
    scanner.scan();

    // Assert
    assert_eq!(scanner.token_value(), "café_1");
}

#[test]
fn should_cook_escape_sequences_given_string_literal_when_reading_token_value() {
    // Arrange
    let mut scanner = Scanner::new(
        r#""a\tb\x41B\u{1F600}😀\
c""#,
    );

    // Act
    scanner.scan();

    // Assert
    assert_eq!(scanner.token_value(), "a\tbAB😀😀c");
}

#[test]
fn should_report_unterminated_string_given_line_break_in_literal_when_scanning_tokens() {
    // Arrange
    let mut scanner = Scanner::new("'abc\n");

    // Act
    scanner.scan();

    // Assert
    let codes = scanner
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.message().code())
        .collect::<Vec<_>>();
    assert_eq!(codes, [1002]);
}
