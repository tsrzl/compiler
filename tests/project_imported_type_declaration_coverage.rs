use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;

// Pinned TypeScript project fixture: projects/declarations_SimpleImport/useModule.ts and m4.ts.
// TS-Go 7.0.2 emits `export declare const value: m4.d;` for this imported value.
#[test]
fn should_preserve_imported_class_type_given_exported_inferred_value_when_emitting_declarations() {
    // Arrange
    let consumer = SourceFile::from_path(
        Path::new("projects/declarations_SimpleImport/useModule.ts"),
        "import * as m4 from './m4'; export const value = m4.x;",
    )
    .expect("the consumer fixture path has a supported source kind");
    let dependency = SourceFile::from_path(
        Path::new("projects/declarations_SimpleImport/m4.ts"),
        "export class d {} export var x: d; export function foo() { return new d(); }",
    )
    .expect("the dependency fixture path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2015)
        .with_module(ModuleKind::Es2015)
        .with_declaration(true)
        .with_emit_declaration_only(true);

    // Act
    let result = Compiler::with_options(options).compile_sources([consumer, dependency]);

    // Assert
    let declaration = result
        .emitted_files()
        .iter()
        .find(|file| {
            file.path().file_name().and_then(|name| name.to_str()) == Some("useModule.d.ts")
        })
        .expect("the consumer declaration output is emitted")
        .text();
    assert!(
        declaration.contains("export declare const value: m4.d;"),
        "the declaration should retain the imported class type; got {declaration:?}"
    );
}
