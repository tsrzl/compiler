use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

// TypeScript 7.0.2 oracle: compiler/capturedParametersInInitializers1.ts, @strict: false, @target: es6.
// Its pinned .errors.txt reports TS2373 when a default initializer references a later parameter.
#[test]
fn should_report_forward_parameter_reference_given_default_initializer_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("capturedParametersInInitializers1.ts"),
        "function choose(value = fallback, fallback = 1) {}",
    )
    .expect("the TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2373);
}
