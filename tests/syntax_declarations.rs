use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_parse_interface_property_given_typescript_interface_when_building_syntax_tree() {
    // Arrange
    let source =
        SourceFile::from_path(Path::new("person.ts"), "interface Person { name: string; }")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics().len(), 0);
    let interface = syntax_tree.program().statements()[0]
        .as_interface_declaration()
        .expect("the source contains an interface declaration");
    assert_eq!(interface.name(), "Person");
    assert_eq!(interface.members()[0].name(), "name");
}
