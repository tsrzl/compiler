use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_report_invalid_unicode_escape_given_non_hex_digit_when_scanning_string_literal() {
    // Oracle: conformance/scanner/ecmascript5/scannerS7.8.4_A7.1_T4.ts, target ES2015;
    // its .errors.txt baseline reports TS1125 at the `G` in `"\\u000G"`.
    // Arrange
    let source = SourceFile::from_path(Path::new("invalid-escape.ts"), r#""\u000G""#)
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert!(
        syntax_tree
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 1125),
        "expected TS1125 for an invalid Unicode escape, got {:?}",
        syntax_tree.diagnostics()
    );
}
