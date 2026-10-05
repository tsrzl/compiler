use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned TypeScript 7.0.2 case: conformance/jsx/tsxNamespacedTagName1.tsx.
// TS-Go accepts a JSX tag name containing a namespace separator.
#[test]
fn should_parse_namespaced_jsx_tag_given_namespace_and_tag_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("view.tsx"),
        "// @ts-nocheck\nconst view = <svg:path></svg:path>;",
    )
    .expect("a TSX path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
