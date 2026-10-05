use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned project case: projects/NestedDeclare/consume.ts; TypeScript parses this declaration.
#[test]
fn should_parse_ambient_module_given_quoted_module_declaration_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("consume.ts"),
        "declare module \"math\" { export function baz(); }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
