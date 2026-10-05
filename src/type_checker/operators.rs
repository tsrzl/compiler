//! Operand diagnostics for binary expression operators.

use std::collections::HashMap;

use crate::binder::ScopedSymbolTable;
use crate::syntax::{BinaryOperator, Diagnostic, Expression, TextSpan};

use super::infer_expression_types;

pub(super) fn check_binary_operator(
    operator: BinaryOperator,
    operator_span: TextSpan,
    left: &Expression,
    right: &Expression,
    environment: &HashMap<String, Vec<String>>,
    symbols: &ScopedSymbolTable<'_>,
    strict_null_checks: bool,
) -> Option<Diagnostic> {
    match operator {
        BinaryOperator::NullishCoalesce => check_nullish_coalesce(left, environment, symbols),
        BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => None,
        BinaryOperator::Subtract
        | BinaryOperator::Multiply
        | BinaryOperator::Divide
        | BinaryOperator::Remainder => {
            check_numeric_operator(operator, operator_span, left, right, environment, symbols)
        }
        BinaryOperator::Add => check_add_operator(operator_span, left, right, environment, symbols),
        BinaryOperator::LessThan
        | BinaryOperator::GreaterThan
        | BinaryOperator::LessThanOrEqual
        | BinaryOperator::GreaterThanOrEqual
        | BinaryOperator::Equal
        | BinaryOperator::StrictEqual
        | BinaryOperator::NotEqual
        | BinaryOperator::StrictNotEqual => check_comparison_operator(
            operator,
            operator_span,
            left,
            right,
            environment,
            symbols,
            strict_null_checks,
        ),
    }
}

fn check_nullish_coalesce(
    left: &Expression,
    environment: &HashMap<String, Vec<String>>,
    symbols: &ScopedSymbolTable<'_>,
) -> Option<Diagnostic> {
    let left_types = infer_expression_types(left, environment, symbols)?;
    let left_operand_is_never_nullish = !left_types.is_empty()
        && left_types.iter().all(|type_name| {
            !matches!(type_name.as_str(), "null" | "undefined" | "any" | "unknown")
        });
    left_operand_is_never_nullish.then(|| {
        Diagnostic::new(
            2869,
            "Right operand of ?? is unreachable because the left operand is never nullish.",
            left.span(),
        )
    })
}

fn check_numeric_operator(
    operator: BinaryOperator,
    operator_span: TextSpan,
    left: &Expression,
    right: &Expression,
    environment: &HashMap<String, Vec<String>>,
    symbols: &ScopedSymbolTable<'_>,
) -> Option<Diagnostic> {
    let left_types = infer_expression_types(left, environment, symbols)?;
    let right_types = infer_expression_types(right, environment, symbols)?;
    if !is_numeric_operation_operand(&left_types) {
        return Some(Diagnostic::new(
            2362,
            "The left-hand side of an arithmetic operation must be of type 'any', 'number', 'bigint' or an enum type.",
            left.span(),
        ));
    }
    if !is_numeric_operation_operand(&right_types) {
        return Some(Diagnostic::new(
            2363,
            "The right-hand side of an arithmetic operation must be of type 'any', 'number', 'bigint' or an enum type.",
            right.span(),
        ));
    }
    if !numeric_kinds_are_compatible(&left_types, &right_types) {
        return Some(mixed_numeric_kinds_diagnostic(
            operator,
            operator_span,
            &left_types,
            &right_types,
        ));
    }
    None
}

fn check_add_operator(
    operator_span: TextSpan,
    left: &Expression,
    right: &Expression,
    environment: &HashMap<String, Vec<String>>,
    symbols: &ScopedSymbolTable<'_>,
) -> Option<Diagnostic> {
    let left_types = infer_expression_types(left, environment, symbols)?;
    let right_types = infer_expression_types(right, environment, symbols)?;
    let can_concatenate = left_types.iter().any(|type_name| type_name == "string")
        || right_types.iter().any(|type_name| type_name == "string");
    let both_numeric = is_numeric_operation_operand(&left_types)
        && is_numeric_operation_operand(&right_types)
        && numeric_kinds_are_compatible(&left_types, &right_types);
    if can_concatenate || both_numeric {
        return None;
    }
    Some(Diagnostic::new(
        2365,
        format!(
            "Operator '+' cannot be applied to types '{}' and '{}'.",
            left_types.join(" | "),
            right_types.join(" | ")
        ),
        operator_span,
    ))
}

fn numeric_kinds_are_compatible(left_types: &[String], right_types: &[String]) -> bool {
    if left_types.iter().any(|type_name| type_name == "any")
        || right_types.iter().any(|type_name| type_name == "any")
    {
        return true;
    }

    let left_has_bigint = left_types.iter().any(|type_name| type_name == "bigint");
    let left_has_number = left_types.iter().any(|type_name| type_name == "number");
    let right_has_bigint = right_types.iter().any(|type_name| type_name == "bigint");
    let right_has_number = right_types.iter().any(|type_name| type_name == "number");
    !(left_has_bigint && right_has_number || left_has_number && right_has_bigint)
}

fn mixed_numeric_kinds_diagnostic(
    operator: BinaryOperator,
    operator_span: TextSpan,
    left_types: &[String],
    right_types: &[String],
) -> Diagnostic {
    Diagnostic::new(
        2365,
        format!(
            "Operator '{}' cannot be applied to types '{}' and '{}'.",
            operator.as_str(),
            left_types.join(" | "),
            right_types.join(" | ")
        ),
        operator_span,
    )
}

fn check_comparison_operator(
    operator: BinaryOperator,
    operator_span: TextSpan,
    left: &Expression,
    right: &Expression,
    environment: &HashMap<String, Vec<String>>,
    symbols: &ScopedSymbolTable<'_>,
    strict_null_checks: bool,
) -> Option<Diagnostic> {
    let left_types = infer_expression_types(left, environment, symbols)?;
    let right_types = infer_expression_types(right, environment, symbols)?;
    let is_equality_operator = matches!(
        operator,
        BinaryOperator::Equal
            | BinaryOperator::StrictEqual
            | BinaryOperator::NotEqual
            | BinaryOperator::StrictNotEqual
    );
    let operands_are_comparable = if is_equality_operator {
        equality_operands_are_comparable(&left_types, &right_types, strict_null_checks)
    } else {
        relational_operands_are_comparable(&left_types, &right_types)
    };
    if operands_are_comparable {
        return None;
    }

    if is_equality_operator {
        Some(Diagnostic::new(
            2367,
            format!(
                "This comparison appears to be unintentional because the types '{}' and '{}' have no overlap.",
                left_types.join(" | "),
                right_types.join(" | ")
            ),
            left.span(),
        ))
    } else {
        Some(Diagnostic::new(
            2365,
            format!(
                "Operator '{}' cannot be applied to types '{}' and '{}'.",
                operator.as_str(),
                left_types.join(" | "),
                right_types.join(" | ")
            ),
            operator_span,
        ))
    }
}

fn equality_operands_are_comparable(
    left_types: &[String],
    right_types: &[String],
    strict_null_checks: bool,
) -> bool {
    left_types.iter().any(|left_type| {
        right_types.iter().any(|right_type| {
            left_type == right_type
                || matches!(left_type.as_str(), "any" | "unknown")
                || matches!(right_type.as_str(), "any" | "unknown")
                || (!strict_null_checks
                    && (matches!(left_type.as_str(), "null" | "undefined")
                        || matches!(right_type.as_str(), "null" | "undefined")))
        })
    })
}

fn relational_operands_are_comparable(left_types: &[String], right_types: &[String]) -> bool {
    left_types.iter().all(|left_type| {
        right_types.iter().all(|right_type| {
            (left_type == "any" || right_type == "any")
                || (left_type == right_type && matches!(left_type.as_str(), "number" | "string"))
        })
    })
}

fn is_numeric_operation_operand(type_names: &[String]) -> bool {
    !type_names.is_empty()
        && type_names
            .iter()
            .all(|type_name| matches!(type_name.as_str(), "any" | "number" | "bigint"))
}
