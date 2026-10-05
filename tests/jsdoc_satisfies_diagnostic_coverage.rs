use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_extra_property_given_jsdoc_satisfies_tag_when_checking_javascript() {
    // Arrange
    // Pinned TypeScript 7.0.2 case: conformance/jsdoc/checkJsdocSatisfiesTag9.ts.
    // An equivalent checked-JavaScript source reports TS2353 in TS-Go.
    let source = SourceFile::from_path(
        Path::new("palette.js"),
        "// @ts-check\n/** @typedef {Object} Color\n * @property {number} r\n * @property {number} g\n * @property {number} b\n */\n/** @satisfies {Record<string, Color>} */\nconst palette = { black: { r: 0, g: 0, d: 0 } };",
    )
    .expect("a JavaScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2353),
        "expected TS2353 for an extra JSDoc-satisfies property, got {:?}",
        result.diagnostics()
    );
}
