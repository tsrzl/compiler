use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;

// Pinned TypeScript project case: projects/declarations_ImportedInPrivate/useModule.ts.
#[test]
fn should_preserve_exported_namespace_given_namespace_declaration_when_emitting_declarations() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("projects/declarations_ImportedInPrivate/useModule.ts"),
        "export namespace PublicApi { export const visible: number = 1; }",
    )
    .expect("the source path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2015)
        .with_module(ModuleKind::Es2015)
        .with_declaration(true)
        .with_emit_declaration_only(true);

    // Act
    let result = Compiler::with_options(options).compile(source);
    let declaration = result
        .emitted_files()
        .iter()
        .find(|file| {
            file.path().file_name().and_then(|name| name.to_str()) == Some("useModule.d.ts")
        })
        .expect("the consumer declaration output is emitted")
        .text();

    // Assert
    assert_eq!(
        declaration,
        "export declare namespace PublicApi {\n    const visible: number;\n}\n"
    );
}
