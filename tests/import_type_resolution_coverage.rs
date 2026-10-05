use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_resolve_imported_interface_given_import_type_reference_when_compiling_sources() {
    // Pinned fixture: conformance/types/import/importTypeLocal.ts.
    // Arrange
    let point = SourceFile::from_path(
        Path::new("point.ts"),
        "export interface Point { x: number; }",
    )
    .expect("the TypeScript path has a supported source kind");
    let main = SourceFile::from_path(
        Path::new("main.ts"),
        "type ImportedPoint = import(\"./point\").Point; const point: ImportedPoint = { x: 1 };",
    )
    .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(
        CompilerOptions::new(ScriptTarget::Es2015).with_module(ModuleKind::CommonJs),
    );

    // Act
    let result = compiler.compile_sources([point, main]);

    // Assert
    assert_eq!(
        result.diagnostics(),
        [],
        "an import type should resolve its exported interface from a sibling source"
    );
}
