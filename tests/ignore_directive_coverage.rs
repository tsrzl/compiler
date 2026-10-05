use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_suppress_type_error_given_ignore_directive_on_preceding_line_when_compiling() {
    // TypeScript 7.0.2 case: conformance/directives/ts-ignore.ts.
    // With target ES2015, @ts-ignore suppresses the following assignment error.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("ignore.ts"),
        "// @ts-ignore\nconst value: number = \"nope\";",
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
        "@ts-ignore suppresses the following line's type error"
    );
}
