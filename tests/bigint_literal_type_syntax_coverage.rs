use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_parse_bigint_literal_union_given_type_alias_when_building_syntax_tree() {
    // Pinned fixture: compiler/bigintPropertyName.ts.
    // Arrange
    let source = SourceFile::from_path(Path::new("token.ts"), "type Token = 6n | 7n | 8n;")
        .expect("the TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
