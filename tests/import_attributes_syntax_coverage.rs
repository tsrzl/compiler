use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_parse_side_effect_import_given_json_attribute_when_building_syntax_tree() {
    // Pinned fixture: conformance/importAttributes/importAttributes1.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("import-attributes.ts"),
        "import \"./0\" with { type: \"json\" }",
    )
    .expect("the TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

#[test]
fn should_parse_reexport_given_json_import_attribute_when_building_syntax_tree() {
    // Pinned fixture: conformance/importAttributes/importAttributes2.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("reexport-attributes.ts"),
        "export {} from \"./0\" with { type: \"json\" };",
    )
    .expect("the TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
