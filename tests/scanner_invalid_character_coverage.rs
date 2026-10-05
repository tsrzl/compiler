use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_report_invalid_character_given_nul_characters_when_scanning_typescript() {
    // Oracle: TS-Go 7.0.2, conformance/scanner/ecmascript5/scannerUnexpectedNullCharacter1.ts
    // with target ES2015 reports TS1127 at both NUL bytes (artifact: scannerUnexpectedNullCharacter1.errors.txt).
    // Arrange
    let source = SourceFile::from_path(
        Path::new("scannerUnexpectedNullCharacter1.ts"),
        "// @target: es2015\nfoo\0+\0bar;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(
        syntax_tree
            .diagnostics()
            .iter()
            .filter(|diagnostic| diagnostic.code() == 1127)
            .count(),
        2
    );
}
