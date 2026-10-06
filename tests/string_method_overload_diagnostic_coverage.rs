use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_ts2389_given_mismatched_string_method_overload_when_checking_types() {
    // Pinned fixture: compiler/ClassDeclaration22.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("string-method-overload.ts"),
        "class C {\n  \"foo\"();\n  \"bar\"() {}\n}",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2389),
        "the mismatched string-named overload should report TS2389, got {:?}",
        result.diagnostics()
    );
}
