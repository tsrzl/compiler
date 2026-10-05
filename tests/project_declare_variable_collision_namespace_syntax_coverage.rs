use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned project fixture: projects/declareVariableCollision/decl.d.ts.
// TS-Go parses this ambient namespace and reports TS1540 only for the nested `module` in the
// full fixture.
#[test]
fn should_parse_ambient_namespace_given_namespace_declaration_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("decl.d.ts"),
        "declare namespace A { class MyRoot {} }",
    )
    .expect("the declaration source path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
