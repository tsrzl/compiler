use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_preserve_labeled_break_given_labeled_while_loop_when_emitting_javascript() {
    // Arrange
    // Pinned fixture area: conformance/statements/labeledStatements.
    let source = SourceFile::from_path(
        Path::new("labeled-loop.ts"),
        "outer: while (true) { break outer; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015)).compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    let output = result.emitted_files()[0].text();
    assert!(
        output.contains("outer: while (true)") && output.contains("break outer;"),
        "the loop label and its break target should remain in emitted JavaScript: {output}"
    );
}
