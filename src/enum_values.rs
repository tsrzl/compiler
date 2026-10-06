use std::collections::HashMap;

use crate::syntax::{BinaryOperator, EnumDeclaration, Expression, UnaryOperator};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum EnumValue {
    Number(f64),
    String(String),
    Computed,
}

pub(crate) fn values_for_enum(declaration: &EnumDeclaration) -> Vec<EnumValue> {
    let mut values = Vec::with_capacity(declaration.members().len());
    let mut numeric_members = HashMap::new();
    let mut next_numeric_value = Some(0.0);

    for member in declaration.members() {
        let value = match member.initializer() {
            Some(Expression::StringLiteral { raw, .. }) => EnumValue::String(raw.clone()),
            Some(initializer) => {
                numeric_constant(initializer, declaration.name(), &numeric_members)
                    .map_or(EnumValue::Computed, EnumValue::Number)
            }
            None => next_numeric_value.map_or(EnumValue::Computed, EnumValue::Number),
        };
        next_numeric_value = match &value {
            EnumValue::Number(value) => {
                numeric_members.insert(member.name().to_owned(), *value);
                Some(*value + 1.0)
            }
            EnumValue::String(_) | EnumValue::Computed => None,
        };
        values.push(value);
    }

    values
}

pub(crate) fn format_enum_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}

fn numeric_constant(
    expression: &Expression,
    enum_name: &str,
    numeric_members: &HashMap<String, f64>,
) -> Option<f64> {
    match expression {
        Expression::NumberLiteral { value, .. } => parse_numeric_literal(value),
        Expression::Identifier { name, .. } => numeric_members.get(name).copied(),
        Expression::PropertyAccessExpression { receiver, name, .. }
            if matches!(
                receiver.as_ref(),
                Expression::Identifier { name: receiver_name, .. } if receiver_name == enum_name
            ) =>
        {
            numeric_members.get(name).copied()
        }
        Expression::ParenthesizedExpression { expression, .. }
        | Expression::TypeAssertionExpression { expression, .. } => {
            numeric_constant(expression, enum_name, numeric_members)
        }
        Expression::UnaryExpression {
            operator, operand, ..
        } => {
            let value = numeric_constant(operand, enum_name, numeric_members)?;
            match operator {
                UnaryOperator::Plus => Some(value),
                UnaryOperator::Negate => Some(-value),
                UnaryOperator::LogicalNot | UnaryOperator::TypeOf => None,
            }
        }
        Expression::BinaryExpression {
            left,
            operator,
            right,
            ..
        } => {
            let left = numeric_constant(left, enum_name, numeric_members)?;
            let right = numeric_constant(right, enum_name, numeric_members)?;
            match operator {
                BinaryOperator::Add => Some(left + right),
                BinaryOperator::Subtract => Some(left - right),
                BinaryOperator::Multiply => Some(left * right),
                BinaryOperator::Divide if right != 0.0 => Some(left / right),
                BinaryOperator::Remainder if right != 0.0 => Some(left % right),
                BinaryOperator::Divide
                | BinaryOperator::Remainder
                | BinaryOperator::LessThan
                | BinaryOperator::GreaterThan
                | BinaryOperator::LessThanOrEqual
                | BinaryOperator::GreaterThanOrEqual
                | BinaryOperator::Equal
                | BinaryOperator::StrictEqual
                | BinaryOperator::NotEqual
                | BinaryOperator::StrictNotEqual
                | BinaryOperator::LogicalAnd
                | BinaryOperator::LogicalOr
                | BinaryOperator::NullishCoalesce => None,
            }
        }
        Expression::BooleanLiteral { .. }
        | Expression::NullLiteral { .. }
        | Expression::StringLiteral { .. }
        | Expression::PropertyAccessExpression { .. }
        | Expression::ConditionalExpression { .. }
        | Expression::AssignmentExpression { .. }
        | Expression::CallExpression { .. }
        | Expression::NewExpression { .. }
        | Expression::ArrowFunction { .. }
        | Expression::ObjectLiteral { .. }
        | Expression::ArrayLiteral { .. }
        | Expression::ElementAccessExpression { .. } => None,
    }
}

fn parse_numeric_literal(value: &str) -> Option<f64> {
    let normalized = value.replace('_', "");
    if let Some(value) = normalized
        .strip_prefix("0x")
        .or_else(|| normalized.strip_prefix("0X"))
    {
        parse_radix_number(value, 16)
    } else if let Some(value) = normalized
        .strip_prefix("0b")
        .or_else(|| normalized.strip_prefix("0B"))
    {
        parse_radix_number(value, 2)
    } else if let Some(value) = normalized
        .strip_prefix("0o")
        .or_else(|| normalized.strip_prefix("0O"))
    {
        parse_radix_number(value, 8)
    } else {
        normalized.parse().ok()
    }
}

fn parse_radix_number(value: &str, radix: u32) -> Option<f64> {
    u64::from_str_radix(value, radix)
        .ok()?
        .to_string()
        .parse()
        .ok()
}
