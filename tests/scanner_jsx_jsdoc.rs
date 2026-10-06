use tsrzl::ast::SyntaxKind;
use tsrzl::scanner::{LanguageVariant, Scanner};

fn jsx_scanner(text: &str) -> Scanner<'_> {
    Scanner::new(text).with_language_variant(LanguageVariant::Jsx)
}

#[test]
fn should_scan_text_until_tag_given_jsx_children_when_scanning_jsx_token() {
    // Arrange
    let mut scanner = jsx_scanner("hello world</div>");

    // Act
    let actual = scanner.scan_jsx_token();

    // Assert
    assert_eq!(
        (actual, scanner.token_value()),
        (SyntaxKind::JsxText, "hello world")
    );
}

#[test]
fn should_classify_all_white_space_given_text_with_line_break_when_scanning_jsx_token() {
    // Arrange
    let mut scanner = jsx_scanner("  \n  <span>");

    // Act
    let actual = scanner.scan_jsx_token();

    // Assert
    assert_eq!(actual, SyntaxKind::JsxTextAllWhiteSpaces);
}

#[test]
fn should_report_raw_greater_than_given_jsx_text_when_scanning_jsx_token() {
    // Arrange
    let mut scanner = jsx_scanner("a > b<");

    // Act
    scanner.scan_jsx_token();

    // Assert
    let codes = scanner
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.message().code())
        .collect::<Vec<_>>();
    assert_eq!(codes, [1382]);
}

#[test]
fn should_append_dashed_parts_given_identifier_when_scanning_jsx_identifier() {
    // Arrange
    let mut scanner = jsx_scanner("data-test-id=");
    scanner.scan();

    // Act
    let actual = scanner.scan_jsx_identifier();

    // Assert
    assert_eq!(
        (actual, scanner.token_value()),
        (SyntaxKind::Identifier, "data-test-id")
    );
}

#[test]
fn should_keep_backslashes_given_quoted_attribute_when_scanning_jsx_attribute_value() {
    // Arrange
    let mut scanner = jsx_scanner(r#" "a\nb""#);

    // Act
    let actual = scanner.scan_jsx_attribute_value();

    // Assert
    assert_eq!(
        (actual, scanner.token_value()),
        (SyntaxKind::StringLiteral, r"a\nb")
    );
}

#[test]
fn should_stop_comment_text_before_tag_given_jsdoc_text_when_scanning_jsdoc_comment_text() {
    // Arrange
    let mut scanner = Scanner::new("some text @param x");

    // Act
    let actual = scanner.scan_jsdoc_comment_text_token(false);

    // Assert
    assert_eq!(
        (actual, scanner.token_value()),
        (SyntaxKind::JSDocCommentTextToken, "some text ")
    );
}

#[test]
fn should_include_dashes_given_tag_name_when_scanning_jsdoc_token() {
    // Arrange
    let mut scanner = Scanner::new("see-also rest");

    // Act
    let actual = scanner.scan_jsdoc_token();

    // Assert
    assert_eq!(
        (actual, scanner.token_value()),
        (SyntaxKind::Identifier, "see-also")
    );
}
