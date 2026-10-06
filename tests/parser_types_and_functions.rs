use tsrzl::ast::{Ast, NodeId, SyntaxKind};
use tsrzl::parser::{ParseOptions, ParsedSourceFile, ScriptKind, parse_source_file};

fn parse(text: &str) -> ParsedSourceFile {
    parse_source_file(&ParseOptions::new("test.ts", ScriptKind::Ts), text)
}

fn first_statement(ast: &Ast) -> NodeId {
    let source_file = ast
        .node(ast.root())
        .data()
        .as_source_file()
        .expect("the root is a source file");
    ast.list(source_file.statements)[0]
}

/// Returns the first node of `kind` in a pre-order walk.
fn find(ast: &Ast, id: NodeId, kind: SyntaxKind) -> Option<NodeId> {
    if ast.node(id).kind() == kind {
        return Some(id);
    }
    ast.children(id)
        .into_iter()
        .find_map(|child| find(ast, child, kind))
}

#[test]
fn should_parse_conditional_type_given_extends_question_colon_when_parsing_type_annotation() {
    // Arrange
    let parsed = parse("let x: A extends B ? C : D;");

    // Act
    let actual = find(
        parsed.ast(),
        parsed.ast().root(),
        SyntaxKind::ConditionalType,
    );

    // Assert
    assert!(actual.is_some());
}

#[test]
fn should_parse_arrow_function_given_typed_parenthesized_parameter_when_parsing_expression() {
    // Arrange
    let parsed = parse("const f = (a: number) => a;");

    // Act
    let actual = find(parsed.ast(), parsed.ast().root(), SyntaxKind::ArrowFunction);

    // Assert
    assert!(actual.is_some());
}

#[test]
fn should_attach_type_arguments_to_call_given_generic_invocation_when_parsing_expression() {
    // Arrange
    let parsed = parse("f<string>(x);");
    let ast = parsed.ast();
    let call = find(ast, ast.root(), SyntaxKind::CallExpression).expect("a call is parsed");

    // Act
    let actual = ast
        .node(call)
        .data()
        .as_call_expression()
        .and_then(|call| call.type_arguments)
        .map(tsrzl::ast::NodeList::len);

    // Assert
    assert_eq!(actual, Some(1));
}

#[test]
fn should_parse_for_of_statement_given_const_binding_when_parsing_statement() {
    // Arrange
    let parsed = parse("for (const item of items) {}");

    // Act
    let actual = parsed.ast().node(first_statement(parsed.ast())).kind();

    // Assert
    assert_eq!(actual, SyntaxKind::ForOfStatement);
}

#[test]
fn should_parse_as_expression_given_type_assertion_operator_when_parsing_expression() {
    // Arrange
    let parsed = parse("value as string;");

    // Act
    let actual = find(parsed.ast(), parsed.ast().root(), SyntaxKind::AsExpression);

    // Assert
    assert!(actual.is_some());
}
