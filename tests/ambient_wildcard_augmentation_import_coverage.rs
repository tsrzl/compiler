use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_missing_export_given_augmentation_for_another_wildcard_module_when_compiling_sources()
 {
    // Pinned TypeScript 7.0.2 case: conformance/ambient/ambientDeclarationsPatterns_merging2.ts.
    // TS-Go reports TS2305 when an augmentation of a.foo does not add an export to b.foo.
    // Arrange
    let wildcard_declaration = SourceFile::from_path(
        Path::new("types.d.ts"),
        "declare module \"*.foo\" { export const everywhere: string; }",
    )
    .expect("the wildcard declaration path has a supported source kind");
    let augmentation = SourceFile::from_path(
        Path::new("a.ts"),
        "import { everywhere, onlyInA } from \"a.foo\";\ndeclare module \"a.foo\" { export const onlyInA: number; }",
    )
    .expect("the augmentation path has a supported source kind");
    let other_module =
        SourceFile::from_path(Path::new("b.ts"), "import { onlyInA } from \"b.foo\";")
            .expect("the consumer path has a supported source kind");
    let compiler = Compiler::with_options(
        CompilerOptions::new(ScriptTarget::Es2015).with_module(ModuleKind::CommonJs),
    );

    // Act
    let result = compiler.compile_sources([wildcard_declaration, augmentation, other_module]);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2305),
        "expected the a.foo augmentation to stay out of b.foo, got {:?}",
        result.diagnostics()
    );
}
