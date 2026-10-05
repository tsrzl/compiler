use std::path::Path;

use tsrzl::compiler::{CompilationResult, Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;

fn compile_sources(sources: &[(&str, &str)]) -> CompilationResult {
    let source_files = sources
        .iter()
        .map(|(path, text)| {
            SourceFile::from_path(Path::new(*path), *text)
                .expect("a TypeScript path has a supported source kind")
        })
        .collect::<Vec<_>>();
    let compiler = Compiler::with_options(
        CompilerOptions::new(ScriptTarget::Es2015).with_module(ModuleKind::CommonJs),
    );

    compiler.compile_sources(source_files)
}

#[test]
fn should_resolve_named_reexport_given_shorthand_module_when_compiling_sources() {
    // Pinned TypeScript 7.0.2 case: conformance/ambient/ambientShorthand_reExport.ts.
    // TS-Go resolves an explicitly named re-export from a shorthand ambient module.
    // Arrange
    let sources = [
        ("declarations.d.ts", "declare module \"jquery\";"),
        ("reExport.ts", "export { x } from \"jquery\";"),
        (
            "consumer.ts",
            "import { x } from \"./reExport\";\nconst value = x;",
        ),
    ];

    // Act
    let result = compile_sources(&sources);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_resolve_star_reexport_given_shorthand_module_when_compiling_sources() {
    // Pinned TypeScript 7.0.2 case: conformance/ambient/ambientShorthand_reExport.ts.
    // TS-Go resolves a namespace import through a star re-export from a shorthand module.
    // Arrange
    let sources = [
        ("declarations.d.ts", "declare module \"jquery\";"),
        ("reExport.ts", "export * from \"jquery\";"),
        (
            "consumer.ts",
            "import * as moduleNamespace from \"./reExport\";\nconst value = moduleNamespace.x;",
        ),
    ];

    // Act
    let result = compile_sources(&sources);

    // Assert
    assert_eq!(result.diagnostics(), []);
}
