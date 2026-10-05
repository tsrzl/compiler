use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_accept_code_point_escape_given_identifier_when_scanning_typescript() {
    // Upstream: conformance/scanner/ecmascript5/scannerUnicodeEscapeInKeyword2.ts, file2.ts.
    // Oracle: TS-Go 7.0.2 accepts this spelling with --target ESNext --noEmit.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("unicode-identifier.ts"),
        r"var \u{0061}wait = 12;",
    )
    .expect("the TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
    assert_eq!(
        syntax_tree.program().statements()[0]
            .as_variable_declaration()
            .map(|declaration| declaration.name()),
        Some(r"\u{0061}wait")
    );
}
