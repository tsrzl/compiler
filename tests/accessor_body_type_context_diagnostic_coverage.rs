use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_ts1183_given_getter_body_in_object_type_when_checking_types() {
    // Pinned fixture: compiler/accessorBodyInTypeContext.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("accessor-type-context.ts"),
        "type A = { get foo() { return 0; } };",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().iter().any(|diagnostic| {
            diagnostic.code() == 1183 && diagnostic.message().contains("ambient contexts")
        }),
        "a getter implementation in an object type should report TS1183, got {:?}",
        result.diagnostics()
    );
}
