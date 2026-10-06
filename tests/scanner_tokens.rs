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
