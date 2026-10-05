use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_jsdoc_template_return_mismatch_given_number_argument_when_checking_javascript() {
    // Pinned TypeScript 7.0.2 case: conformance/jsdoc/jsdocTemplateTag.ts.
    // TS-Go infers number for the generic JSDoc return and reports TS2322 at the string assignment.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("template.js"),
        "// @ts-check\n/** @template T\n * @param {T} value\n * @returns {T}\n */\nfunction identity(value) { return value; }\n/** @type {string} */\nconst message = identity(42);",
    )
    .expect("the JavaScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2322),
        "expected number-to-string assignment mismatch, got {:?}",
        result.diagnostics()
    );
}
