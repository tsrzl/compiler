use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_no_matching_constructor_given_jsdoc_overloads_when_checking_javascript() {
    // Arrange
    // Pinned TypeScript 7.0.2 case: conformance/jsdoc/overloadTag2.ts.
    // TS-Go reports TS2769 for a boolean argument outside the JSDoc overloads.
    let source = SourceFile::from_path(
        Path::new("overload.js"),
        "// @ts-check\nclass Foo {\n  /** @constructor\n   * @overload\n   * @param {string} value\n   */\n  /** @constructor\n   * @overload\n   * @param {number} value\n   */\n  /** @constructor\n   * @param {string | number} value\n   */\n  constructor(value) {}\n}\nnew Foo(true);",
    )
    .expect("a JavaScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2769),
        "expected TS2769 for a boolean argument without a JSDoc overload, got {:?}",
        result.diagnostics()
    );
}
