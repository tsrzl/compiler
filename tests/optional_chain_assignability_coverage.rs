use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_undefined_from_optional_chain_given_non_nullable_return_when_checking_types() {
    // TypeScript 7.0.2 case: conformance/controlFlow/controlFlowOptionalChain.ts (strict=true).
    // Its .types baseline records `o?.foo` as `string | number | undefined`.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("controlFlowOptionalChain.ts"),
        "interface Thing { foo: string; } function getFoo(value: Thing | undefined): string { return value?.foo; }",
    )
    .expect("the TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2322),
        "an optional chain can produce undefined under strict null checking"
    );
}
