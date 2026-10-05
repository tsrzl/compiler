use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_missing_string_member_given_jsdoc_callback_parameter_when_checking_javascript() {
    // Pinned TypeScript 7.0.2 case: conformance/jsdoc/callbackTag1.ts.
    // TS-Go contextually types the callback parameter as string and reports TS2551 for toFixed.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("callback.js"),
        "// @ts-check\n/** @callback StringHandler\n * @param {string} value\n * @returns {string}\n */\n/** @type {StringHandler} */\nconst format = value => value.toFixed();",
    )
    .expect("the JavaScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2551),
        "expected a string member diagnostic for toFixed, got {:?}",
        result.diagnostics()
    );
}
