use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_required_parameter_after_optional_jsdoc_parameter_given_checked_javascript_when_checking_types()
 {
    // Arrange
    // Pinned fixture: conformance/jsdoc/checkJsdocOptionalParamOrder.ts.
    let source = SourceFile::from_path(
        Path::new("parameters.js"),
        "// @ts-check\n/** @param {number} first\n * @param {number} [optional]\n * @param {number} required\n */\nfunction choose(first, optional, required) {}",
    )
    .expect("a JavaScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 1016),
        "expected TS1016 for a required parameter after an optional JSDoc parameter, got {:?}",
        result.diagnostics()
    );
}
