use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_implicit_any_yield_given_unannotated_generator_when_checking_types() {
    // Upstream: conformance/generators/generatorImplicitAny.ts, function g2.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("generator.ts"),
        "function* values() { const value = yield; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(
        result
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code())
            .collect::<Vec<_>>(),
        [7057]
    );
}

#[test]
fn should_accept_contextually_typed_yield_given_annotated_variable_when_checking_types() {
    // Upstream: conformance/generators/generatorImplicitAny.ts, function g3.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("generator.ts"),
        "function* values() { const value: string = yield; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}
