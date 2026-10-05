use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_template_constraint_mismatch_given_jsdoc_typedef_when_checking_javascript() {
    // Arrange
    // Pinned TypeScript 7.0.2 case: conformance/jsdoc/checkJsdocTypeTag4.ts.
    // An equivalent checked-JavaScript input reports TS2344 for the failed constraint.
    let source = SourceFile::from_path(
        Path::new("box.js"),
        "// @ts-check\n/** @template {string} T\n * @typedef {{ value: T }} Box\n */\n/** @type {Box<number>} */\nlet box;",
    )
    .expect("a JavaScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2344),
        "expected TS2344 for a JSDoc typedef constraint mismatch, got {:?}",
        result.diagnostics()
    );
}
