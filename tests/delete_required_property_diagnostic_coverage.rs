use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_ts2790_given_delete_on_required_property_when_checking_types() {
    // Pinned fixture: compiler/deleteExpressionMustBeOptional_exactOptionalPropertyTypes.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("delete-required-property.ts"),
        "interface Box { value: number; }\nconst box: Box = { value: 1 };\ndelete box.value;",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(
        CompilerOptions::new(ScriptTarget::Es2015).with_strict_null_checks(true),
    );

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2790),
        "deleting a required property should report TS2790, got {:?}",
        result.diagnostics()
    );
}
