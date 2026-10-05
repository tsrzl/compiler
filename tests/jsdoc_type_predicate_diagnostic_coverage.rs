use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_missing_boolean_member_given_jsdoc_type_predicate_when_checking_javascript() {
    // Arrange
    // Pinned TypeScript 7.0.2 case: conformance/jsdoc/returnTagTypeGuard.ts.
    // An equivalent checked-JavaScript predicate narrows to boolean and reports TS2339.
    let source = SourceFile::from_path(
        Path::new("predicate.js"),
        "// @ts-check\n/** @param {unknown} value\n * @returns {value is boolean}\n */\nfunction isBoolean(value) { return typeof value === \"boolean\"; }\n/** @param {unknown} value */\nfunction use(value) { if (isBoolean(value)) value.toFixed(); }",
    )
    .expect("a JavaScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2339),
        "expected TS2339 after the JSDoc predicate narrows to boolean, got {:?}",
        result.diagnostics()
    );
}
