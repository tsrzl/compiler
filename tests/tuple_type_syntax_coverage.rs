use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned TypeScript 7.0.2 case: conformance/types/tuple/tupleElementTypes1.ts.
// TS-Go accepts a fixed-length tuple type annotation.
#[test]
fn should_parse_tuple_type_given_two_element_types_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(Path::new("main.ts"), "type Pair = [string, number];")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
