use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::{Statement, SyntaxTree};

#[test]
fn should_parse_string_literal_namespace_export_given_star_reexport_when_building_tree() {
    // TypeScript 7.0.2 case: conformance/es2022/arbitraryModuleNamespaceIdentifiers/arbitraryModuleNamespaceIdentifiers_syntax.ts.
    // With target ES2022 and module ES2022, a string-literal namespace export name is valid.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("namespace-export.ts"),
        "export * as \"valid 4\" from \"./values-valid\";",
    )
    .expect("the TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
    assert!(
        matches!(
            syntax_tree.program().statements().first(),
            Some(Statement::ExportAll(_))
        ),
        "the string-named namespace re-export should form an export-all declaration"
    );
}
