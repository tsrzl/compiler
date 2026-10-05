use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_return_type_mismatch_given_jsdoc_returns_tag_when_checking_types() {
    // Arrange
    // Pinned fixture: conformance/jsdoc/checkJsdocReturnTag2.ts.
    // Its outFile case is unsupported by the runner; the reduced source reports TS2322 in TS-Go.
    let source = SourceFile::from_path(
        Path::new("returns.js"),
        "// @ts-check\n/** @returns {string} */\nfunction getName() { return 5; }",
    )
    .expect("a JavaScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2322),
        "expected TS2322 for a numeric return with a JSDoc string return type, got {:?}",
        result.diagnostics()
    );
}
