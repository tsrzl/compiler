use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_reject_augmentation_member_given_distinct_wildcard_module_specifier_when_checking_types()
{
    // Pinned TypeScript 7.0.2 case: conformance/ambient/ambientDeclarationsPatterns_merging3.ts.
    // TS-Go keeps the `a.foo` augmentation out of the matching `b.foo` module's exported type.
    // Arrange
    let declarations = SourceFile::from_path(
        Path::new("types.d.ts"),
        "declare module \"*.foo\" { export interface Shape { star: string; } }",
    )
    .expect("the declaration path has a supported source kind");
    let consumer = SourceFile::from_path(
        Path::new("test.ts"),
        "declare module \"a.foo\" { export interface Shape { onlyA: string; } }\nimport { Shape } from \"b.foo\";\ndeclare let value: Shape;\nvalue.onlyA;",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(
        CompilerOptions::new(ScriptTarget::Es2015).with_module(ModuleKind::CommonJs),
    );

    // Act
    let result = compiler.compile_sources([declarations, consumer]);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2339),
        "expected the a.foo augmentation to be absent from b.foo, got {:?}",
        result.diagnostics()
    );
}
