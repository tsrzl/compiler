use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned TypeScript input: conformance/additionalChecks/noPropertyAccessFromIndexSignature1.ts.
#[test]
fn should_parse_string_index_signature_given_interface_member_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("input.ts"),
        "interface Values { [key: string]: string; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
