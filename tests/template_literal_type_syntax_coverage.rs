use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned TypeScript 7.0.2 case: conformance/types/literal/templateLiteralTypes8.ts.
// TS-Go accepts a template literal type with a string substitution.
#[test]
fn should_parse_template_literal_type_given_string_substitution_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(Path::new("main.ts"), "type Greeting = `hello ${string}`;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
