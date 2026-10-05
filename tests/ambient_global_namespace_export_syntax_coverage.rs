use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned project case: projects/declarations_ExportNamespace/decl.d.ts; TS-Go accepts `export as namespace`.
#[test]
fn should_parse_global_namespace_export_given_ambient_declaration_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(Path::new("ambient.d.ts"), "export as namespace moduleA;")
        .expect("a TypeScript declaration path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
