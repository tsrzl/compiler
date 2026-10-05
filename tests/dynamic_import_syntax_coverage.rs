use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_parse_dynamic_import_call_given_string_specifier_when_building_syntax_tree() {
    // Pinned fixture: conformance/dynamicImport/importCallExpression1ES2020.ts.
    // Arrange
    let source = SourceFile::from_path(Path::new("main.ts"), "import(\"./0\");")
        .expect("the TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
