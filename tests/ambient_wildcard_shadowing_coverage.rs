use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_missing_wildcard_export_given_standalone_ambient_declaration_when_compiling_sources()
 {
    // Pinned TypeScript 7.0.2 case: conformance/ambient/ambientDeclarationsPatterns_merging1.ts.
    // TS-Go reports TS2305 when a standalone declaration shadows the wildcard module's exports.
    // Arrange
    let wildcard_declaration = SourceFile::from_path(
        Path::new("types.d.ts"),
        "declare module \"*.foo\" { export const everywhere: string; }",
    )
    .expect("the wildcard declaration path has a supported source kind");
    let concrete_declaration = SourceFile::from_path(
        Path::new("augmentation.d.ts"),
        "declare module \"a.foo\" { export const onlyInA: number; }",
    )
    .expect("the concrete declaration path has a supported source kind");
    let consumer = SourceFile::from_path(
        Path::new("consumer.ts"),
        "import { everywhere, onlyInA } from \"a.foo\";\nconst text: string = everywhere;\nconst count: number = onlyInA;",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(
        CompilerOptions::new(ScriptTarget::Es2015).with_module(ModuleKind::CommonJs),
    );

    // Act
    let result = compiler.compile_sources([wildcard_declaration, concrete_declaration, consumer]);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2305),
        "expected the standalone module declaration to shadow the wildcard export, got {:?}",
        result.diagnostics()
    );
}
