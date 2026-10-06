use tsrzl::ast::{Ast, NodeFlags, NodeId, SyntaxKind};
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

fn first_expression(ast: &Ast) -> NodeId {
    ast.node(first_statement(ast))
        .data()
        .as_expression_statement()
        .expect("the first statement is an expression statement")
        .expression
}

/// Renders an expression tree as nested kinds, such as `Binary(Identifier Plus Binary(..))`.
fn shape(ast: &Ast, id: NodeId) -> String {
    let children = ast.children(id);
    let kind = format!("{:?}", ast.node(id).kind());
    if children.is_empty() {
        return kind;
    }
    let inner = children
        .iter()
        .map(|&child| shape(ast, child))
        .collect::<Vec<_>>()
        .join(" ");
    format!("{kind}({inner})")
}

#[test]
fn should_bind_multiplication_tighter_given_mixed_additive_operators_when_parsing_expression() {
    // Arrange
    let parsed = parse("a + b * c;");

    // Act
    let actual = shape(parsed.ast(), first_expression(parsed.ast()));

    // Assert
    assert_eq!(
        actual,
        "BinaryExpression(Identifier PlusToken BinaryExpression(Identifier AsteriskToken Identifier))"
    );
}

#[test]
fn should_associate_to_the_right_given_chained_exponentiation_when_parsing_expression() {
    // Arrange
    let parsed = parse("a ** b ** c;");

    // Act
    let actual = shape(parsed.ast(), first_expression(parsed.ast()));

    // Assert
    assert_eq!(
        actual,
        "BinaryExpression(Identifier AsteriskAsteriskToken BinaryExpression(Identifier AsteriskAsteriskToken Identifier))"
    );
}

#[test]
fn should_flag_optional_chain_given_call_after_question_dot_when_parsing_expression() {
    // Arrange
    let parsed = parse("a?.b();");
    let ast = parsed.ast();

    // Act
    let actual = ast.node(first_expression(ast)).flags();

    // Assert
    assert!(actual.intersects(NodeFlags::OPTIONAL_CHAIN));
}

#[test]
fn should_parse_labeled_statement_given_identifier_and_colon_when_parsing_statement() {
    // Arrange
    let parsed = parse("outer: a;");
    let ast = parsed.ast();

    // Act
    let actual = ast.node(first_statement(ast)).kind();

    // Assert
    assert_eq!(actual, SyntaxKind::LabeledStatement);
}
