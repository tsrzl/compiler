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
