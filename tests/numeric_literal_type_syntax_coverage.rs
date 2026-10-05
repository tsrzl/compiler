use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_parse_numeric_literal_union_given_type_alias_when_building_syntax_tree() {
    // Pinned fixture: conformance/types/literal/numericLiteralTypes1.ts.
    // Arrange
    let source = SourceFile::from_path(Path::new("status.ts"), "type Status = 0 | 1;")
        .expect("the TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
