use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_parse_string_literal_property_type_given_interface_when_building_syntax_tree() {
    // Pinned fixture: conformance/controlFlow/exhaustiveSwitchStatements1.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("shape.ts"),
        "interface Square { kind: \"square\"; size: number; }",
    )
    .expect("the TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
