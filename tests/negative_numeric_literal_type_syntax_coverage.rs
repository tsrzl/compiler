use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_parse_negative_numeric_literal_given_type_alias_when_building_syntax_tree() {
    // Pinned fixture: conformance/types/literal/numericLiteralTypes1.ts.
    // Arrange
    let source = SourceFile::from_path(Path::new("signed.ts"), "type Signed = -1 | 0;")
        .expect("the TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
