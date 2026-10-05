use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_incompatible_overload_implementation_given_function_signature_when_checking_types()
{
    // TypeScript 7.0.2 case: conformance/functions/functionOverloadCompatibilityWithVoid01.ts
    // Oracle expectation: the number-returning overload is incompatible with its void implementation (TS2394).
    // Arrange
    let source = SourceFile::from_path(
        Path::new("functionOverloadCompatibilityWithVoid01.ts"),
        "function f(x: string): number; function f(x: string): void { return; }",
    )
    .expect("the TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2394),
        "an overload signature must be compatible with its implementation"
    );
}
