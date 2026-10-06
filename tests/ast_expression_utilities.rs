use tsrzl::ast::{Ast, NodeId, SyntaxKind};
use tsrzl::parser::{ParseOptions, ParsedSourceFile, ScriptKind, parse_source_file};

fn parse(text: &str) -> ParsedSourceFile {
    parse_source_file(&ParseOptions::new("test.ts", ScriptKind::Ts), text)
}

fn all(ast: &Ast, id: NodeId, kind: SyntaxKind, found: &mut Vec<NodeId>) {
    if ast.node(id).kind() == kind {
        found.push(id);
    }
    for child in ast.children(id) {
        all(ast, child, kind, found);
    }
}

fn nodes(parsed: &ParsedSourceFile, kind: SyntaxKind) -> Vec<NodeId> {
    let mut found = Vec::new();
    all(parsed.ast(), parsed.ast().root(), kind, &mut found);
    found
}

fn first(parsed: &ParsedSourceFile, kind: SyntaxKind) -> NodeId {
    nodes(parsed, kind)[0]
}

#[test]
fn should_report_optional_chain_root_given_question_dot_access_when_checking_expression() {
    // Arrange
    let parsed = parse("a?.b;");
    let access = first(&parsed, SyntaxKind::PropertyAccessExpression);

    // Act
    let root = parsed.ast().is_optional_chain_root(access);

    // Assert
    assert!(root);
}

#[test]
fn should_report_only_outer_access_as_outermost_given_chain_continuation_when_checking_expression()
{
    // Arrange
    let parsed = parse("a?.b.c;");
    let ast = parsed.ast();
    let accesses = nodes(&parsed, SyntaxKind::PropertyAccessExpression);

    // Act
    let outermost: Vec<bool> = accesses
        .iter()
        .map(|&access| ast.is_outermost_optional_chain(access))
        .collect();

    // Assert
    assert_eq!(outermost, [true, false]);
}

#[test]
fn should_report_assignment_target_given_destructured_array_element_when_checking_expression() {
    // Arrange
    let parsed = parse("[value] = items;");
    let ast = parsed.ast();
    let value = nodes(&parsed, SyntaxKind::Identifier)[0];

    // Act
    let target = ast.is_assignment_target(value);

    // Assert
    assert!(target);
}

#[test]
fn should_not_report_assignment_target_given_right_operand_when_checking_expression() {
    // Arrange
    let parsed = parse("value = other;");
    let ast = parsed.ast();
    let other = nodes(&parsed, SyntaxKind::Identifier)[1];

    // Act
    let target = ast.is_assignment_target(other);

    // Assert
    assert!(!target);
}

#[test]
fn should_report_dotted_name_given_this_property_chain_when_checking_expression() {
    // Arrange
    let parsed = parse("this.a.b;");
    let access = first(&parsed, SyntaxKind::PropertyAccessExpression);

    // Act
    let dotted = parsed.ast().is_dotted_name(access);

    // Assert
    assert!(dotted);
}

#[test]
fn should_report_logical_expression_given_negated_parenthesized_or_when_checking_expression() {
    // Arrange
    let parsed = parse("!(a || b);");
    let negation = first(&parsed, SyntaxKind::PrefixUnaryExpression);

    // Act
    let logical = parsed.ast().is_logical_expression(negation);

    // Assert
    assert!(logical);
}

#[test]
fn should_return_call_given_parenthesized_immediately_invoked_arrow_when_reading_iife() {
    // Arrange
    let parsed = parse("(() => 1)();");
    let arrow = first(&parsed, SyntaxKind::ArrowFunction);

    // Act
    let call = parsed.ast().immediately_invoked_function_expression(arrow);

    // Assert
    assert_eq!(call, Some(first(&parsed, SyntaxKind::CallExpression)));
}

#[test]
fn should_report_destructuring_assignment_given_object_pattern_assignment_when_checking_expression()
{
    // Arrange
    let parsed = parse("({ a } = source);");
    let assignment = first(&parsed, SyntaxKind::BinaryExpression);

    // Act
    let destructuring = parsed.ast().is_destructuring_assignment(assignment);

    // Assert
    assert!(destructuring);
}

#[test]
fn should_not_report_executable_given_uninitialized_var_statement_when_checking_statement() {
    // Arrange
    let parsed = parse("var value;");
    let statement = first(&parsed, SyntaxKind::VariableStatement);

    // Act
    let executable = parsed.ast().is_potentially_executable_node(statement);

    // Assert
    assert!(!executable);
}

#[test]
fn should_report_executable_given_let_statement_when_checking_statement() {
    // Arrange
    let parsed = parse("let value;");
    let statement = first(&parsed, SyntaxKind::VariableStatement);

    // Act
    let executable = parsed.ast().is_potentially_executable_node(statement);

    // Assert
    assert!(executable);
}
