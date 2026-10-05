use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned TypeScript 7.0.2 case: conformance/types/conditional/conditionalTypes1.ts.
// TS-Go accepts conditional type syntax with an extends check and two branches.
#[test]
fn should_parse_conditional_type_given_extends_check_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("main.ts"),
        "type Kind = string extends string ? true : false;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
