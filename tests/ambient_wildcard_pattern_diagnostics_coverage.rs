use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_multiple_asterisks_given_wildcard_ambient_module_when_compiling_sources() {
    // Pinned TypeScript 7.0.2 case: conformance/ambient/ambientDeclarationsPatterns_tooManyAsterisks.ts.
    // TS-Go reports TS5061 because an ambient module pattern allows at most one asterisk.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("ambient.ts"),
        "declare module \"too*many*asterisks\" {}",
    )
    .expect("the ambient module path has a supported source kind");
    let compiler = Compiler::with_options(
        CompilerOptions::new(ScriptTarget::Es2015).with_module(ModuleKind::CommonJs),
    );

    // Act
    let result = compiler.compile_sources([source]);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 5061),
        "expected TS5061 for a wildcard module pattern with multiple asterisks, got {:?}",
        result.diagnostics()
    );
}
