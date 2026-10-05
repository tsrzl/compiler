use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_resolve_wildcard_ambient_module_given_suffix_import_when_compiling_sources() {
    // Pinned TypeScript 7.0.2 case: conformance/ambient/ambientDeclarationsPatterns.ts.
    // TS-Go resolves a `*!text` ambient module for an import ending in `!text`.
    // Arrange
    let declarations = SourceFile::from_path(
        Path::new("declarations.d.ts"),
        "declare module \"*!text\" { const text: string; export default text; }",
    )
    .expect("a declaration path has a supported source kind");
    let consumer = SourceFile::from_path(
        Path::new("user.ts"),
        "import fileText from \"./readme!text\"; const content: string = fileText;",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(
        CompilerOptions::new(ScriptTarget::Es2015).with_module(ModuleKind::CommonJs),
    );

    // Act
    let result = compiler.compile_sources([declarations, consumer]);

    // Assert
    assert_eq!(result.diagnostics(), []);
}
