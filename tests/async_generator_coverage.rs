use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_parse_async_generator_declaration_given_es2018_function_syntax_when_building_tree() {
    // Upstream: conformance/asyncGenerators/asyncGeneratorParameterEvaluation.ts, target ES2018.
    // The asyncGeneratorParameterEvaluation(target=es2018).js baseline preserves `async function* f1`.
    // Arrange
    let source = SourceFile::from_path(Path::new("async-generator.ts"), "async function* f1() {}")
        .expect("the TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
    assert_eq!(
        syntax_tree.program().statements()[0]
            .as_function_declaration()
            .map(tsrzl::syntax::FunctionDeclaration::name),
        Some("f1")
    );
}
