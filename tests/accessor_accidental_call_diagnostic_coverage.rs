use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_ts6234_given_call_to_get_accessor_when_checking_types() {
    // Pinned fixture: compiler/accessorAccidentalCallDiagnostic.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("accessor-call.ts"),
        "class Test { get property(): number { return 1; } }\nfunction read(test: Test) { return test.property(); }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().iter().any(|diagnostic| {
            diagnostic.code() == 6234 && diagnostic.message().contains("get accessor")
        }),
        "calling a getter should report TS6234 with the accessor-specific message, got {:?}",
        result.diagnostics()
    );
}
