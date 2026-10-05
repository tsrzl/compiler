use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_parse_parameter_decorator_given_class_method_parameter_when_building_syntax_tree() {
    // Upstream: conformance/decorators/class/method/parameter/decoratorOnClassMethodParameter1.ts.
    // With target ES2015 and experimentalDecorators, TS-Go accepts a method parameter decorator.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("parameter-decorator.ts"),
        "declare function dec(target: Object, propertyKey: string | symbol, parameterIndex: number): void;\nclass C { method(@dec p: number) {} }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
