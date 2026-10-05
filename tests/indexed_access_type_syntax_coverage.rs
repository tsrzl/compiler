use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned TypeScript 7.0.2 case: conformance/types/keyof/keyofAndIndexedAccess.ts.
// TS-Go accepts a property lookup in a type expression.
#[test]
fn should_parse_indexed_access_type_given_property_key_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("main.ts"),
        "interface Person { name: string; } type Name = Person[\"name\"];",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
