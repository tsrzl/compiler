use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::{Statement, SyntaxTree};

#[test]
fn should_parse_namespace_reexport_given_identifier_alias_when_building_syntax_tree() {
    // Arrange
    // Pinned fixture: conformance/externalModules/typeOnly/exportNamespace2.ts.
    let source = SourceFile::from_path(
        Path::new("namespace-export.ts"),
        "export * as api from './module';",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
    assert!(matches!(
        syntax_tree.program().statements().first(),
        Some(Statement::ExportAll(_))
    ));
}
