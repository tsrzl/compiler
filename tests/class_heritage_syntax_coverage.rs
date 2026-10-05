use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned TypeScript 7.0.2 project fixture: projects/Quote'InName/m'ain.ts.
// TS-Go parses a class extending the qualified name test.ClassA without syntax diagnostics.
#[test]
fn should_parse_qualified_class_heritage_given_dotted_base_name_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(Path::new("main.ts"), "class ClassC extends test.ClassA {}")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

// Pinned TypeScript 7.0.2 case: conformance/parser/ecmascript5/Generics/parserGenericsInTypeContexts2.ts.
// TS-Go accepts generic arguments in a class heritage clause.
#[test]
fn should_parse_generic_class_heritage_given_type_argument_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(Path::new("main.ts"), "class C extends A<X> {}")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
