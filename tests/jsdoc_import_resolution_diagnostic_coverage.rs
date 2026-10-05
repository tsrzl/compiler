use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_imported_jsdoc_type_mismatch_given_adjacent_declaration_when_compiling_sources() {
    // Arrange
    // Pinned TypeScript 7.0.2 case: conformance/jsdoc/importTag19.ts.
    // An equivalent checked-JavaScript input resolves @import and reports TS2322 in TS-Go.
    let type_source = SourceFile::from_path(
        Path::new("types.d.ts"),
        "export interface Item { count: number; }",
    )
    .expect("a declaration path has a supported source kind");
    let implementation = SourceFile::from_path(
        Path::new("main.js"),
        "// @ts-check\n/** @import { Item } from \"./types\" */\n/** @type {Item} */\nconst item = { count: \"wrong\" };",
    )
    .expect("a JavaScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([type_source, implementation]);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2322),
        "expected TS2322 for a mismatched property from a JSDoc import, got {:?}",
        result.diagnostics()
    );
}
