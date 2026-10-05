use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned TypeScript 7.0.2 case: conformance/types/intersection/operatorsAndIntersectionTypes.ts.
// TS-Go accepts an intersection between two named types.
#[test]
fn should_parse_intersection_type_given_two_named_types_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("main.ts"),
        "interface Left {}\ninterface Right {}\ntype Combined = Left & Right;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
