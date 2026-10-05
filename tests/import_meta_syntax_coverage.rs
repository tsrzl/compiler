use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_parse_import_meta_expression_given_module_source_when_building_syntax_tree() {
    // Upstream: conformance/es2019/importMeta/importMeta.ts, moduleLookingFile01.ts.
    // Oracle: --target ESNext --module ES2020 --noEmit; the source parses without diagnostics.
    // Arrange
    let source = SourceFile::from_path(Path::new("module.ts"), "export let value = import.meta;")
        .expect("the TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
