use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;
use tsrzl::syntax::{Expression, SyntaxTree};

#[test]
fn should_parse_async_arrow_function_given_parenthesized_parameters_when_building_syntax_tree() {
    // Upstream: conformance/async/es2017/asyncArrowFunction/asyncArrowFunction1_es2017.ts.
    // Arrange
    let source = SourceFile::from_path(Path::new("answer.ts"), "const answer = async () => 42;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

#[test]
fn should_parse_async_arrow_function_given_single_parameter_when_building_syntax_tree() {
    // Upstream: conformance/async/es2017/asyncArrowFunction/asyncUnParenthesizedArrowFunction_es2017.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("identity.ts"),
        "const identity = async value => value;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
    let declaration = syntax_tree.program().statements()[0]
        .as_variable_declaration()
        .expect("the source declares an arrow function");
    assert!(matches!(
        declaration.initializer(),
        Some(Expression::ArrowFunction { parameters, .. })
            if parameters.len() == 1 && parameters[0].name() == "value"
    ));
}

#[test]
fn should_parse_await_expression_given_async_arrow_body_when_building_syntax_tree() {
    // Upstream: conformance/async/es2017/asyncArrowFunction/asyncArrowFunctionCapturesThis_es2017.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("example.ts"),
        "class Example { load() { const read = async () => await this; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

#[test]
fn should_emit_async_arrow_function_given_es2017_target_when_compiling() {
    // Upstream: conformance/async/es2017/asyncArrowFunction/asyncArrowFunction1_es2017.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "var answer = async (): Promise<void> => {\n};",
    )
    .expect("a TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2017).with_module(ModuleKind::Es2015);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    assert!(
        result.emitted_files()[0]
            .text()
            .contains("var answer = async () => {\n};")
    );
}
