use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_parse_type_only_star_reexport_given_export_declaration_when_building_syntax_tree() {
    // Upstream: conformance/externalModules/typeOnly/exportNamespace4.ts.
    // With target=ES2015, module=CommonJS, and declaration output, TS-Go accepts the export.
    // Arrange
    let source = SourceFile::from_path(Path::new("type-only-star.ts"), "export type * from './a';")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
