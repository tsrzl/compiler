use tsrzl::ast::{SyntaxKind, TokenFlags};
use tsrzl::scanner::{CommentDirectiveKind, LanguageVariant, Scanner};

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

fn first_token_value(text: &str) -> (SyntaxKind, String) {
    let mut scanner = Scanner::new(text);
    let kind = scanner.scan();
    (kind, scanner.token_value().to_owned())
}

fn diagnostic_codes(text: &str) -> Vec<u32> {
    let mut scanner = Scanner::new(text);
    while scanner.scan() != SyntaxKind::EndOfFile {}
    scanner
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.message().code())
        .collect()
}

#[test]
fn should_normalize_value_given_separated_decimal_with_exponent_when_scanning_numeric_literal() {
    // Arrange
    let text = "1_000.50e1";

    // Act
    let actual = first_token_value(text);

    // Assert
    assert_eq!(actual, (SyntaxKind::NumericLiteral, "10005".to_owned()));
}

#[test]
fn should_convert_to_decimal_value_given_hexadecimal_literal_when_scanning_numeric_literal() {
    // Arrange
    let text = "0x1F";

    // Act
    let actual = first_token_value(text);

    // Assert
    assert_eq!(actual, (SyntaxKind::NumericLiteral, "31".to_owned()));
}

#[test]
fn should_convert_to_decimal_bigint_given_binary_bigint_literal_when_scanning_numeric_literal() {
    // Arrange
    let text = "0b101n";

    // Act
    let actual = first_token_value(text);

    // Assert
    assert_eq!(actual, (SyntaxKind::BigIntLiteral, "5n".to_owned()));
}

#[test]
fn should_scan_fraction_given_leading_dot_when_scanning_numeric_literal() {
    // Arrange
    let text = ".25";

    // Act
    let actual = first_token_value(text);

    // Assert
    assert_eq!(actual, (SyntaxKind::NumericLiteral, "0.25".to_owned()));
}

#[test]
fn should_report_leading_zero_given_decimal_starting_with_zero_when_scanning_numeric_literal() {
    // Arrange
    let text = "08";

    // Act
    let actual = diagnostic_codes(text);

    // Assert
    assert_eq!(actual, [1489]);
}

#[test]
fn should_report_consecutive_separators_given_doubled_underscore_when_scanning_numeric_literal() {
    // Arrange
    let text = "1__0";

    // Act
    let actual = diagnostic_codes(text);

    // Assert
    assert_eq!(actual, [6189]);
}

#[test]
fn should_report_identifier_after_number_given_adjacent_letters_when_scanning_numeric_literal() {
    // Arrange
    let text = "3in";

    // Act
    let actual = diagnostic_codes(text);

    // Assert
    assert_eq!(actual, [1351]);
}

#[test]
fn should_cook_unicode_escapes_given_escaped_identifier_when_scanning_tokens() {
    // Arrange
    let text = r"ab\u{63}";

    // Act
    let actual = first_token_value(text);

    // Assert
    assert_eq!(actual, (SyntaxKind::Identifier, "abc".to_owned()));
}

#[test]
fn should_scan_template_head_given_substitution_start_when_scanning_tokens() {
    // Arrange
    let text = "`a$\u{7b}";

    // Act
    let actual = first_token_value(text);

    // Assert
    assert_eq!(actual, (SyntaxKind::TemplateHead, "a".to_owned()));
}

#[test]
fn should_normalize_carriage_return_given_template_without_substitution_when_scanning_tokens() {
    // Arrange
    let text = "`a\r\nb`";

    // Act
    let actual = first_token_value(text);

    // Assert
    assert_eq!(
        actual,
        (SyntaxKind::NoSubstitutionTemplateLiteral, "a\nb".to_owned())
    );
}

#[test]
fn should_scan_private_identifier_given_hash_name_when_scanning_tokens() {
    // Arrange
    let text = "#secret";

    // Act
    let actual = first_token_value(text);

    // Assert
    assert_eq!(
        actual,
        (SyntaxKind::PrivateIdentifier, "#secret".to_owned())
    );
}

#[test]
fn should_skip_shebang_given_hash_bang_at_file_start_when_scanning_tokens() {
    // Arrange
    let text = "#!/usr/bin/env node\nx";

    // Act
    let actual = scan_kinds(text);

    // Assert
    assert_eq!(actual, [SyntaxKind::Identifier, SyntaxKind::EndOfFile]);
}

#[test]
fn should_report_merge_conflict_given_conflict_marker_line_when_scanning_tokens() {
    // Arrange
    let text = "<<<<<<< HEAD\nx";

    // Act
    let actual = diagnostic_codes(text);

    // Assert
    assert_eq!(actual, [1185]);
}

#[test]
fn should_record_ignore_directive_given_ts_ignore_comment_when_scanning_tokens() {
    // Arrange
    let mut scanner = Scanner::new("// @ts-ignore\nx");

    // Act
    while scanner.scan() != SyntaxKind::EndOfFile {}

    // Assert
    let directives = scanner
        .comment_directives()
        .iter()
        .map(|directive| (directive.kind(), directive.start(), directive.end()))
        .collect::<Vec<_>>();
    assert_eq!(directives, [(CommentDirectiveKind::Ignore, 0, 13)]);
}

#[test]
fn should_flag_deprecated_tag_given_preceding_jsdoc_when_scanning_tokens() {
    // Arrange
    let mut scanner = Scanner::new("/** @deprecated */ x");

    // Act
    scanner.scan();

    // Assert
    assert!(
        scanner
            .token_flags()
            .intersects(TokenFlags::PRECEDING_JSDOC_WITH_DEPRECATED)
    );
}

#[test]
fn should_scan_closing_tag_start_given_jsx_variant_when_scanning_tokens() {
    // Arrange
    let mut scanner = Scanner::new("</").with_language_variant(LanguageVariant::Jsx);

    // Act
    let actual = scanner.scan();

    // Assert
    assert_eq!(actual, SyntaxKind::LessThanSlashToken);
}
