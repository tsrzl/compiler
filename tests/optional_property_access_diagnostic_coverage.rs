use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_possibly_undefined_given_optional_parameter_read_when_checking_types() {
    // Arrange
    // Pinned TS-Go 7.0.2 compiler case: compiler/optionalParamArgsTest.ts.
    let source = SourceFile::from_path(
        Path::new("optionalParamArgsTest.ts"),
        "function F4(F4A1: number, F4A2?: number) { return F4A1 + F4A2; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 18048),
        "expected TS18048 for reading an optional parameter without narrowing, got {:?}",
        result.diagnostics()
    );
}
