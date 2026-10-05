use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_ignore_first_line_hashbang_given_typescript_source_when_parsing() {
    // Upstream: compiler/shebang.ts. Oracle config: --target ES2015 --noEmit.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("shebang.ts"),
        "#!/usr/bin/env node\nvar foo = 'I wish the generated JS to be executed in node';",
    )
    .expect("the TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
    assert_eq!(
        syntax_tree.program().statements()[0]
            .as_variable_declaration()
            .map(|declaration| declaration.name()),
        Some("foo")
    );
}
