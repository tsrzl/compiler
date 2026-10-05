use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_parse_typeof_import_given_module_specifier_when_building_syntax_tree() {
    // Pinned fixture: conformance/types/import/importTypeAmbient.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("import-type.ts"),
        "type Module = typeof import(\"foo\");",
    )
    .expect("the TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
