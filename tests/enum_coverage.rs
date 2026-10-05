use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned TypeScript input: conformance/constEnums/constEnum1.ts.
#[test]
fn should_parse_const_enum_declaration_given_const_modifier_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "const enum Answer { FortyTwo = 42 }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
    let declaration = syntax_tree.program().statements()[0]
        .as_enum_declaration()
        .expect("the statement should be an enum declaration");
    assert!(declaration.is_const());
}
