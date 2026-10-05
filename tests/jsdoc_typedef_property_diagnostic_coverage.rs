use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_jsdoc_typedef_property_mismatch_given_annotated_object_when_checking_javascript() {
    // Arrange
    // Pinned TypeScript 7.0.2 case: conformance/jsdoc/typedefTagNested.ts.
    // Equivalent checked JavaScript reports TS2322 for the mismatched property in TS-Go.
    let source = SourceFile::from_path(
        Path::new("point.js"),
        "// @ts-check\n/** @typedef {Object} Point\n * @property {number} x\n * @property {number} y\n */\n/** @type {Point} */\nconst point = { x: \"wrong\", y: 1 };",
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
        "expected TS2322 for a JSDoc typedef property mismatch, got {:?}",
        result.diagnostics()
    );
}
