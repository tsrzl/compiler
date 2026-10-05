use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_suppress_type_errors_given_nocheck_directive_when_compiling_file() {
    // TypeScript 7.0.2 case: conformance/directives/ts-expect-error-nocheck.ts.
    // With target ES2015, @ts-nocheck suppresses semantic diagnostics in the file.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("unchecked.ts"),
        "// @ts-nocheck\nconst value: number = \"wrong\";",
    )
    .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.code() != 2322),
        "@ts-nocheck suppresses type diagnostics for the file"
    );
}
