use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_parse_union_of_object_types_given_type_alias_when_building_syntax_tree() {
    // Pinned fixture: conformance/controlFlow/exhaustiveSwitchStatements1.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("shape.ts"),
        "type Shape = { kind: \"square\"; size: number } | { kind: \"circle\"; radius: number };",
    )
    .expect("the TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
