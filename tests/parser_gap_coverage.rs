use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_parse_tagged_template_as_single_statement_given_tag_expression_when_building_tree() {
    // Upstream: conformance/es6/templates/taggedTemplateStringsWithTagsTypedAsAny.ts
    // Scenario: `var f: any; f `abc`` with target ES2015.
    // Arrange
    let source = SourceFile::from_path(Path::new("tagged-template.ts"), "var f: any;\nf `abc`;")
        .expect("the TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.program().statements().len(), 2);
}
