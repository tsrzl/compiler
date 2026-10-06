//! Narrowing predicates modeled on TypeScript-Go's binder: they decide which conditions and
//! references become flow nodes.

use crate::ast::{Ast, NodeData, NodeFlags, NodeId, SyntaxKind};

/// Returns whether a condition can narrow the type of some reference.
pub(super) fn is_narrowing_expression(ast: &Ast, expression: NodeId) -> bool {
    match ast.node(expression).data() {
        NodeData::Identifier(_) => true,
        _ if ast.node(expression).kind() == SyntaxKind::ThisKeyword => true,
        NodeData::PropertyAccessExpression(_) | NodeData::ElementAccessExpression(_) => {
            contains_narrowable_reference(ast, expression)
        }
        NodeData::CallExpression(_) => has_narrowable_argument(ast, expression),
        NodeData::ParenthesizedExpression(inner) => is_narrowing_expression(ast, inner.expression),
        NodeData::NonNullExpression(inner) => is_narrowing_expression(ast, inner.expression),
        NodeData::TypeOfExpression(inner) => is_narrowing_expression(ast, inner.expression),
        NodeData::BinaryExpression(_) => is_narrowing_binary_expression(ast, expression),
        NodeData::PrefixUnaryExpression(unary) => {
            unary.operator == SyntaxKind::ExclamationToken
                && is_narrowing_expression(ast, unary.operand)
        }
        _ => false,
    }
}

/// Returns whether the expression is, or optionally chains to, a narrowable reference.
pub(super) fn contains_narrowable_reference(ast: &Ast, expression: NodeId) -> bool {
    if is_narrowable_reference(ast, expression) {
        return true;
    }
    let node = ast.node(expression);
    node.flags().intersects(NodeFlags::OPTIONAL_CHAIN)
        && matches!(
            node.kind(),
            SyntaxKind::PropertyAccessExpression
                | SyntaxKind::ElementAccessExpression
                | SyntaxKind::CallExpression
                | SyntaxKind::NonNullExpression
        )
        && node
            .data()
            .expression()
            .is_some_and(|inner| contains_narrowable_reference(ast, inner))
}

/// Returns whether the expression is a reference whose type control flow analysis can narrow.
pub(super) fn is_narrowable_reference(ast: &Ast, node: NodeId) -> bool {
    match ast.node(node).data() {
        NodeData::Identifier(_) | NodeData::MetaProperty(_) => true,
        _ if matches!(
            ast.node(node).kind(),
            SyntaxKind::ThisKeyword | SyntaxKind::SuperKeyword
        ) =>
        {
            true
        }
        NodeData::PropertyAccessExpression(access) => {
            is_narrowable_reference(ast, access.expression)
        }
        NodeData::ParenthesizedExpression(inner) => is_narrowable_reference(ast, inner.expression),
        NodeData::NonNullExpression(inner) => is_narrowable_reference(ast, inner.expression),
        NodeData::ElementAccessExpression(access) => {
            ast.is_string_or_numeric_literal_like(access.argument_expression)
                || ast.is_entity_name_expression(access.argument_expression)
                    && is_narrowable_reference(ast, access.expression)
        }
        NodeData::BinaryExpression(binary) => {
            let operator = ast.node(binary.operator_token).kind();
            operator == SyntaxKind::CommaToken && is_narrowable_reference(ast, binary.right)
                || operator.is_assignment_operator()
                    && ast
                        .node(binary.left)
                        .kind()
                        .is_left_hand_side_expression_kind()
        }
        _ => false,
    }
}

fn has_narrowable_argument(ast: &Ast, expression: NodeId) -> bool {
    let call = ast
        .node(expression)
        .data()
        .as_call_expression()
        .expect("narrowable arguments are checked on call expressions");
    ast.list(call.arguments)
        .iter()
        .any(|&argument| contains_narrowable_reference(ast, argument))
        || ast
            .node(call.expression)
            .data()
            .as_property_access_expression()
            .is_some_and(|access| contains_narrowable_reference(ast, access.expression))
}

fn is_narrowing_binary_expression(ast: &Ast, expression: NodeId) -> bool {
    let binary = ast
        .node(expression)
        .data()
        .as_binary_expression()
        .expect("checked by the caller");
    match ast.node(binary.operator_token).kind() {
        SyntaxKind::EqualsToken
        | SyntaxKind::BarBarEqualsToken
        | SyntaxKind::AmpersandAmpersandEqualsToken
        | SyntaxKind::QuestionQuestionEqualsToken => {
            contains_narrowable_reference(ast, binary.left)
        }
        SyntaxKind::EqualsEqualsToken
        | SyntaxKind::ExclamationEqualsToken
        | SyntaxKind::EqualsEqualsEqualsToken
        | SyntaxKind::ExclamationEqualsEqualsToken => {
            let left = ast.skip_parentheses(binary.left);
            let right = ast.skip_parentheses(binary.right);
            is_narrowable_operand(ast, left)
                || is_narrowable_operand(ast, right)
                || is_narrowing_type_of_operands(ast, right, left)
                || is_narrowing_type_of_operands(ast, left, right)
                || is_boolean_literal(ast, right) && is_narrowing_expression(ast, left)
                || is_boolean_literal(ast, left) && is_narrowing_expression(ast, right)
        }
        SyntaxKind::InstanceOfKeyword => is_narrowable_operand(ast, binary.left),
        SyntaxKind::InKeyword | SyntaxKind::CommaToken => {
            is_narrowing_expression(ast, binary.right)
        }
        _ => false,
    }
}

/// Returns whether an operand narrows when compared, looking through parentheses, the target
/// of an assignment, and the right side of a comma.
pub(super) fn is_narrowable_operand(ast: &Ast, expression: NodeId) -> bool {
    match ast.node(expression).data() {
        NodeData::ParenthesizedExpression(inner) => is_narrowable_operand(ast, inner.expression),
        NodeData::BinaryExpression(binary) => match ast.node(binary.operator_token).kind() {
            SyntaxKind::EqualsToken => is_narrowable_operand(ast, binary.left),
            SyntaxKind::CommaToken => is_narrowable_operand(ast, binary.right),
            _ => contains_narrowable_reference(ast, expression),
        },
        _ => contains_narrowable_reference(ast, expression),
    }
}

fn is_narrowing_type_of_operands(ast: &Ast, type_of: NodeId, literal: NodeId) -> bool {
    ast.node(type_of)
        .data()
        .as_type_of_expression()
        .is_some_and(|inner| is_narrowable_operand(ast, inner.expression))
        && matches!(
            ast.node(literal).kind(),
            SyntaxKind::StringLiteral | SyntaxKind::NoSubstitutionTemplateLiteral
        )
}

fn is_boolean_literal(ast: &Ast, node: NodeId) -> bool {
    matches!(
        ast.node(node).kind(),
        SyntaxKind::TrueKeyword | SyntaxKind::FalseKeyword
    )
}
