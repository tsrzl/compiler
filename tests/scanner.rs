use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_accept_unicode_escape_given_identifier_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(Path::new("answer.ts"), r"const \u0061 = 1;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_reject_escaped_keyword_given_unicode_escape_when_scanning_typescript() {
    // Arrange
    let source = SourceFile::from_path(Path::new("keyword.ts"), r"\u0076ar value = 1;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert!(
        syntax_tree
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 1260)
    );
}

#[test]
fn should_report_consecutive_numeric_separators_given_numeric_literal_when_scanning_typescript() {
    // Arrange
    let source = SourceFile::from_path(Path::new("number.ts"), "const value = 1__2;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert!(
        syntax_tree
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 6189)
    );
}

#[test]
fn should_report_unterminated_string_given_unescaped_line_terminator_when_scanning_typescript() {
    // Arrange
    let source = SourceFile::from_path(Path::new("string.ts"), "const value = \"first\nsecond\";")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert!(
        syntax_tree
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 1002)
    );
}

#[test]
fn should_parse_regular_expression_literal_given_variable_initializer_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(Path::new("pattern.ts"), "const pattern = /answer/i;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
