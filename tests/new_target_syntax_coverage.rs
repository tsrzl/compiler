use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_parse_new_target_meta_property_given_function_body_when_building_syntax_tree() {
    // Upstream: conformance/es6/newTarget/newTarget.es6.ts, function f1 body.
    // Oracle: --target ES2015 --noEmit accepts `const g = new.target;`.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("new-target.ts"),
        "function f1() { const g = new.target; }",
    )
    .expect("the TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
