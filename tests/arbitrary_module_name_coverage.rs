use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::{Statement, SyntaxTree};

#[test]
fn should_parse_string_literal_export_name_given_export_alias_when_building_tree() {
    // TypeScript 7.0.2 case: conformance/es2022/arbitraryModuleNamespaceIdentifiers/arbitraryModuleNamespaceIdentifiers_syntax.ts.
    // With target ES2022 and module ES2022, a string-literal export name is valid.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("arbitrary-export.ts"),
        "export const foo = 123; export { foo as \"valid 1\" };",
    )
    .expect("the TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
    let Some(Statement::ExportNamed(specifiers)) = syntax_tree.program().statements().get(1) else {
        panic!("the second statement should be a named export list");
    };
    assert_eq!(specifiers[0].exported_name(), "valid 1");
}
