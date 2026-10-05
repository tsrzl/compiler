use std::path::Path;

use tsrzl::compiler::{CompilationResult, Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;

fn compile_sources(sources: &[(&str, &str)]) -> CompilationResult {
    let source_files = sources
        .iter()
        .map(|(path, text)| {
            SourceFile::from_path(Path::new(*path), *text)
                .expect("an ambient module path has a supported source kind")
        })
        .collect::<Vec<_>>();
    let compiler = Compiler::with_options(
        CompilerOptions::new(ScriptTarget::Es2015).with_module(ModuleKind::CommonJs),
    );

    compiler.compile_sources(source_files)
}

#[test]
fn should_allow_arbitrary_member_given_shorthand_module_import_when_compiling_sources() {
    // Pinned TypeScript 7.0.2 case: conformance/ambient/ambientShorthand.ts.
    // TS-Go types imports from a shorthand ambient module as any.
    // Arrange
    let sources = [
        ("declarations.d.ts", "declare module \"lib\";"),
        (
            "consumer.ts",
            "import value from \"lib\";\nconst member = value.anything;",
        ),
    ];

    // Act
    let result = compile_sources(&sources);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_merge_duplicate_shorthand_modules_given_import_when_compiling_sources() {
    // Pinned TypeScript 7.0.2 case: conformance/ambient/ambientShorthand_duplicate.ts.
    // TS-Go merges repeated shorthand declarations and resolves their import.
    // Arrange
    let sources = [
        ("first.d.ts", "declare module \"duplicate\";"),
        ("second.d.ts", "declare module \"duplicate\";"),
        (
            "consumer.ts",
            "import value from \"duplicate\";\nconst member = value.anything;",
        ),
    ];

    // Act
    let result = compile_sources(&sources);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_merge_members_given_repeated_ambient_module_declarations_when_compiling_sources() {
    // Pinned TypeScript 7.0.2 case: conformance/ambient/ambientExternalModuleMerging.ts.
    // TS-Go exposes members from both declarations through the imported module.
    // Arrange
    let sources = [
        (
            "first.d.ts",
            "declare module \"M\" { export var x: string; }",
        ),
        (
            "second.d.ts",
            "declare module \"M\" { export var y: string; }",
        ),
        (
            "consumer.ts",
            "import M = require(\"M\");\nconst x: string = M.x;\nconst y: string = M.y;",
        ),
    ];

    // Act
    let result = compile_sources(&sources);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_merge_shorthand_module_with_named_module_declaration_when_compiling_sources() {
    // Pinned TypeScript 7.0.2 case: conformance/ambient/ambientShorthand_merging.ts.
    // TS-Go combines a shorthand import with the exports added by a named declaration.
    // Arrange
    let sources = [
        ("shorthand.d.ts", "declare module \"combo\";"),
        (
            "named.d.ts",
            "declare module \"combo\" { export const bar: number; }",
        ),
        (
            "consumer.ts",
            "import value, { bar } from \"combo\";\nconst result: number = bar;\nconst member = value.anything;",
        ),
    ];

    // Act
    let result = compile_sources(&sources);

    // Assert
    assert_eq!(result.diagnostics(), []);
}
