use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_report_unterminated_regex_given_regex_at_end_of_file_when_scanning() {
    // Upstream: compiler/unterminatedRegexAtEndOfSource1.ts; target ES2015.
    // The pinned TS-Go diagnostic is TS1161.
    // Arrange
    let source = SourceFile::from_path(Path::new("broken.ts"), "var a = /")
        .expect("the TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert!(
        syntax_tree
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 1161),
        "expected TS1161 for the unterminated regex, got {:?}",
        syntax_tree.diagnostics()
    );
}
