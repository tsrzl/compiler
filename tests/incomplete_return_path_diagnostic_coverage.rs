use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_missing_return_path_given_non_void_function_with_partial_return_when_checking_types()
 {
    // Arrange
    // Pinned TS-Go 7.0.2 reports TS2366; compiler/exhaustiveSwitchImplicitReturn.errors.txt
    // covers the same control-flow rule. Strict mode enables strictNullChecks.
    let source = SourceFile::from_path(
        Path::new("exhaustiveSwitchImplicitReturn.ts"),
        "function choose(flag: boolean): number { if (flag) { return 1; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2366),
        "expected TS2366 for a non-void function with a path that does not return, got {:?}",
        result.diagnostics()
    );
}
