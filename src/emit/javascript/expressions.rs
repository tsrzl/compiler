use crate::compiler::ScriptTarget;
use crate::syntax::Expression;

use super::EmitContext;
use super::{
    emit_array_literal, emit_arrow_function, emit_call_expression, emit_conditional_expression,
    emit_element_access_expression, emit_infix_expression, emit_object_literal,
    emit_property_access_expression, emit_unary_expression,
};

pub(super) fn emit_expression(
    expression: &Expression,
    context: &EmitContext,
    target: ScriptTarget,
    indentation: usize,
) -> String {
    match expression {
        Expression::NumberLiteral { value, .. } | Expression::StringLiteral { raw: value, .. } => {
            value.clone()
        }
        Expression::BooleanLiteral { value, .. } => value.to_string(),
        Expression::NullLiteral { .. } => "null".to_owned(),
        Expression::Identifier { name, .. } => emit_identifier_expression(name, context),
        Expression::BinaryExpression {
            left,
            operator,
            right,
            ..
        } => emit_infix_expression(left, operator.as_str(), right, context, target, indentation),
        Expression::ConditionalExpression {
            condition,
            when_true,
            when_false,
            ..
        } => emit_conditional_expression(
            condition,
            when_true,
            when_false,
            context,
            target,
            indentation,
        ),
        Expression::ParenthesizedExpression { expression, .. } => {
            format!(
                "({})",
                emit_expression(expression, context, target, indentation)
            )
        }
        Expression::TypeAssertionExpression { expression, .. } => {
            emit_expression(expression, context, target, indentation)
        }
        Expression::AssignmentExpression {
            left,
            right,
            operator,
            ..
        } => emit_infix_expression(left, operator.as_str(), right, context, target, indentation),
        Expression::CallExpression {
            callee, arguments, ..
        } => emit_call_expression(callee, arguments, context, target, indentation),
        Expression::NewExpression {
            constructor,
            arguments,
            ..
        } => emit_new_expression(constructor, arguments, context, target, indentation),
        Expression::ArrowFunction {
            parameters,
            parameters_parenthesized,
            body,
            ..
        } => emit_arrow_function(
            parameters,
            *parameters_parenthesized,
            body,
            context,
            target,
            indentation,
        ),
        Expression::PropertyAccessExpression { .. }
        | Expression::ElementAccessExpression { .. } => {
            emit_access_expression(expression, context, target, indentation)
        }
        Expression::UnaryExpression {
            operator, operand, ..
        } => emit_unary_expression(*operator, operand, context, target, indentation),
        Expression::ObjectLiteral { properties, .. } => {
            emit_object_literal(properties, context, target, indentation)
        }
        Expression::ArrayLiteral { elements, .. } => {
            emit_array_literal(elements, context, target, indentation)
        }
    }
}

fn emit_access_expression(
    expression: &Expression,
    context: &EmitContext,
    target: ScriptTarget,
    indentation: usize,
) -> String {
    match expression {
        Expression::PropertyAccessExpression { receiver, name, .. } => {
            emit_property_access_expression(receiver, name, context, target, indentation)
        }
        Expression::ElementAccessExpression {
            receiver, argument, ..
        } => emit_element_access_expression(receiver, argument, context, target, indentation),
        _ => unreachable!("only access expressions are passed to this emitter"),
    }
}

fn emit_new_expression(
    constructor: &Expression,
    arguments: &[Expression],
    context: &EmitContext,
    target: ScriptTarget,
    indentation: usize,
) -> String {
    format!(
        "new {}",
        emit_call_expression(constructor, arguments, context, target, indentation)
    )
}

fn emit_identifier_expression(name: &str, context: &EmitContext) -> String {
    context
        .import_references
        .get(name)
        .cloned()
        .unwrap_or_else(|| name.to_owned())
}
