use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_report_missing_hex_digit_given_hex_prefix_without_digits_when_scanning_typescript() {
    // Oracle: conformance/scanner/ecmascript5/scannerS7.8.3_A6.1_T1.ts, target ES2015;
    // its .errors.txt baseline reports TS1125, "Hexadecimal digit expected," for `0x`.
    // Arrange
    let source = SourceFile::from_path(Path::new("invalid-hex.ts"), "0x")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert!(
        syntax_tree
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 1125),
        "expected TS1125 for a hex prefix without digits, got {:?}",
        syntax_tree.diagnostics()
    );
}
