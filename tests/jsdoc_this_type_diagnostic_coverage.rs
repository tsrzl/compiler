use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_missing_member_given_jsdoc_this_type_when_checking_javascript() {
    // Arrange
    // Pinned TypeScript 7.0.2 case: conformance/jsdoc/thisTag3.ts.
    // An equivalent checked-JavaScript function reports TS2339 for a missing member.
    let source = SourceFile::from_path(
        Path::new("this.js"),
        "// @ts-check\n/** @typedef {{ fn(): void }} T */\n/** @this {T} */\nfunction call() { this.missing(); }",
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
        "expected TS2339 for a missing member on JSDoc-typed this, got {:?}",
        result.diagnostics()
    );
}
