use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;

// Pinned TypeScript project fixture: projects/declarations_ImportedUseInFunction/useModule.ts and fncOnly_m4.ts.
#[test]
fn should_preserve_imported_function_return_type_given_exported_call_result_when_emitting_declarations()
 {
    // Arrange
    let consumer = SourceFile::from_path(
        Path::new("projects/declarations_ImportedUseInFunction/useModule.ts"),
        "import * as m4 from './fncOnly_m4'; export var useFncOnly_m4_f4 = m4.foo();",
    )
    .expect("the consumer fixture path has a supported source kind");
    let dependency = SourceFile::from_path(
        Path::new("projects/declarations_ImportedUseInFunction/fncOnly_m4.ts"),
        "export class d {} export var x: d; export function foo(): d { return new d(); }",
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
        declaration.contains("export declare var useFncOnly_m4_f4: m4.d;"),
        "the declaration should retain the imported function return type; got {declaration:?}"
    );
}
