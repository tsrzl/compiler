use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

// TypeScript 7.0.2 oracle: conformance/salsa/checkSpecialPropertyAssignments.ts, target=es2015,
// allowJs=true, checkJs=true, noEmit=true; its pinned .errors.txt reports TS2322 for string[] to number[].
#[test]
fn should_report_jsdoc_array_type_mismatch_given_javascript_assignment_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("bug24252.js"),
        "/** @type {string[]} */ var source = []; /** @type {number[]} */ var target; target = source;",
    )
    .expect("the JavaScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(
        result
            .diagnostics()
            .iter()
            .filter(|diagnostic| diagnostic.code() == 2322)
            .count(),
        1
    );
}
