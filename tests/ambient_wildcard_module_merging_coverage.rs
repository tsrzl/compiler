use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_merge_concrete_module_with_wildcard_ambient_declaration_when_compiling_sources() {
    // Pinned TypeScript 7.0.2 case: conformance/ambient/ambientDeclarationsPatterns_merging1.ts.
    // TS-Go combines the wildcard export with the concrete module's additional export.
    // Arrange
    let wildcard_declaration = SourceFile::from_path(
        Path::new("types.d.ts"),
        "declare module \"*.foo\" { export const everywhere: string; }",
    )
    .expect("the declaration path has a supported source kind");
    let consumer = SourceFile::from_path(
        Path::new("test.ts"),
        "declare module \"a.foo\" { export const onlyInA: number; }\nimport { everywhere, onlyInA } from \"a.foo\";\nconst text: string = everywhere;\nconst count: number = onlyInA;",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(
        CompilerOptions::new(ScriptTarget::Es2015).with_module(ModuleKind::CommonJs),
    );

    // Act
    let result = compiler.compile_sources([wildcard_declaration, consumer]);

    // Assert
    assert_eq!(result.diagnostics(), []);
}
