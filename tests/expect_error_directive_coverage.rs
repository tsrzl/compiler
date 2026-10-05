use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_unused_expect_error_given_valid_following_statement_when_compiling() {
    // TypeScript 7.0.2 case: conformance/directives/ts-expect-error.ts.
    // With target ES2015, an unused @ts-expect-error directive reports TS2578.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("expect-error.ts"),
        "// @ts-expect-error\nconst value: string = \"ok\";",
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
            .any(|diagnostic| diagnostic.code() == 2578),
        "an expect-error directive without a following error must be reported"
    );
}
