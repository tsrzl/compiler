use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned TypeScript project case: projects/NestedLocalModule-SimpleCase/test1.ts.
// TS-Go parses this nested import-equals form, then reports TS1147 semantically.
#[test]
fn should_parse_import_equals_given_namespace_declaration_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("test1.ts"),
        "namespace myModule { import foo = require(\"./test2\"); }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
