use std::collections::{HashMap, HashSet};

use crate::binder::{InterfacePropertyType, PropertyAccessibility};
use crate::syntax::{Diagnostic, Expression};

use super::{
    CallableMap, ScopedSymbolTable, check_assignment_expression, check_binary_expression,
    check_call_expression, check_constructor_arguments, check_type_reference,
    infer_expression_types,
};

pub(super) fn check_expression_names(
    expression: &Expression,
    environment: &HashMap<String, Vec<String>>,
    symbols: &ScopedSymbolTable<'_>,
) -> Vec<Diagnostic> {
    match expression {
        Expression::Identifier { name, span } if !environment.contains_key(name) => {
            vec![Diagnostic::new(
                2304,
                format!("Cannot find name '{name}'."),
                *span,
            )]
        }
        Expression::BinaryExpression { left, right, .. }
        | Expression::AssignmentExpression { left, right, .. } => {
            check_expression_names(left, environment, symbols)
                .into_iter()
                .chain(check_expression_names(right, environment, symbols))
                .collect()
        }
        Expression::ConditionalExpression {
            condition,
            when_true,
            when_false,
            ..
        } => check_expression_names(condition, environment, symbols)
            .into_iter()
            .chain(check_expression_names(when_true, environment, symbols))
            .chain(check_expression_names(when_false, environment, symbols))
            .collect(),
        Expression::ParenthesizedExpression { expression, .. } => {
            check_expression_names(expression, environment, symbols)
        }
        Expression::TypeAssertionExpression {
            expression,
            type_annotation,
            ..
        } => check_expression_names(expression, environment, symbols)
            .into_iter()
            .chain(check_type_reference(type_annotation, symbols))
            .collect(),
        Expression::UnaryExpression { operand, .. } => {
            check_expression_names(operand, environment, symbols)
        }
        Expression::PropertyAccessExpression { receiver, .. } => {
            check_expression_names(receiver, environment, symbols)
        }
        Expression::ElementAccessExpression {
            receiver, argument, ..
        } => check_expression_names(receiver, environment, symbols)
            .into_iter()
            .chain(check_expression_names(argument, environment, symbols))
            .collect(),
        Expression::CallExpression {
            callee, arguments, ..
        } => check_expression_names(callee, environment, symbols)
            .into_iter()
            .chain(
                arguments
                    .iter()
                    .flat_map(|argument| check_expression_names(argument, environment, symbols)),
            )
            .collect(),
        Expression::NewExpression {
            constructor,
            arguments,
            ..
        } => check_expression_names(constructor, environment, symbols)
            .into_iter()
            .chain(
                arguments
                    .iter()
                    .flat_map(|argument| check_expression_names(argument, environment, symbols)),
            )
            .collect(),
        Expression::ObjectLiteral { properties, .. } => properties
            .iter()
            .flat_map(|property| check_expression_names(property.value(), environment, symbols))
            .collect(),
        Expression::ArrayLiteral { elements, .. } => elements
            .iter()
            .flat_map(|element| check_expression_names(element, environment, symbols))
            .collect(),
        Expression::NumberLiteral { .. }
        | Expression::BooleanLiteral { .. }
        | Expression::NullLiteral { .. }
        | Expression::StringLiteral { .. }
        | Expression::ArrowFunction { .. }
        | Expression::Identifier { .. } => Vec::new(),
    }
}

pub(super) fn check_property_accesses(
    expression: &Expression,
    environment: &HashMap<String, Vec<String>>,
    symbols: &ScopedSymbolTable<'_>,
) -> Vec<Diagnostic> {
    match expression {
        Expression::PropertyAccessExpression {
            receiver,
            name,
            name_span,
            ..
        } => {
            let mut diagnostics = Vec::new();
            if let Some(receiver_types) = infer_expression_types(receiver, environment, symbols) {
                let interface_types = receiver_types
                    .iter()
                    .filter(|type_name| symbols.properties_for_type(type_name).is_some())
                    .collect::<Vec<_>>();
                let property_is_missing = interface_types.iter().any(|type_name| {
                    symbols
                        .properties_for_type(type_name)
                        .is_some_and(|properties| {
                            !properties.iter().any(|property| property.name == *name)
                        })
                });
                if property_is_missing {
                    diagnostics.push(Diagnostic::new(
                        2339,
                        format!(
                            "Property '{}' does not exist on type '{}'.",
                            name,
                            receiver_types.join(" | ")
                        ),
                        *name_span,
                    ));
                } else if let Some(property) = interface_types.iter().find_map(|type_name| {
                    symbols
                        .properties_for_type(type_name)
                        .and_then(|properties| {
                            properties.into_iter().find(|property| {
                                property.name == *name && property.modifiers.accessibility.is_some()
                            })
                        })
                }) && let Some(diagnostic) =
                    check_member_accessibility(&property, name, *name_span, environment, symbols)
                {
                    diagnostics.push(diagnostic);
                }
            }
            diagnostics.extend(check_property_accesses(receiver, environment, symbols));
            diagnostics
        }
        Expression::ElementAccessExpression {
            receiver, argument, ..
        } => check_property_accesses(receiver, environment, symbols)
            .into_iter()
            .chain(check_property_accesses(argument, environment, symbols))
            .collect(),
        Expression::CallExpression {
            callee, arguments, ..
        } => check_property_accesses_in_call(callee, arguments, environment, symbols),
        Expression::NewExpression {
            constructor,
            arguments,
            ..
        } => check_property_accesses_in_call(constructor, arguments, environment, symbols),
        Expression::BinaryExpression { left, right, .. }
        | Expression::AssignmentExpression { left, right, .. } => {
            check_property_accesses(left, environment, symbols)
                .into_iter()
                .chain(check_property_accesses(right, environment, symbols))
                .collect()
        }
        Expression::ConditionalExpression {
            condition,
            when_true,
            when_false,
            ..
        } => check_property_accesses(condition, environment, symbols)
            .into_iter()
            .chain(check_property_accesses(when_true, environment, symbols))
            .chain(check_property_accesses(when_false, environment, symbols))
            .collect(),
        Expression::ParenthesizedExpression { expression, .. }
        | Expression::TypeAssertionExpression { expression, .. } => {
            check_property_accesses(expression, environment, symbols)
        }
        Expression::UnaryExpression { operand, .. } => {
            check_property_accesses(operand, environment, symbols)
        }
        Expression::ObjectLiteral { properties, .. } => properties
            .iter()
            .flat_map(|property| check_property_accesses(property.value(), environment, symbols))
            .collect(),
        Expression::ArrayLiteral { elements, .. } => elements
            .iter()
            .flat_map(|element| check_property_accesses(element, environment, symbols))
            .collect(),
        Expression::NumberLiteral { .. }
        | Expression::BooleanLiteral { .. }
        | Expression::NullLiteral { .. }
        | Expression::StringLiteral { .. }
        | Expression::ArrowFunction { .. }
        | Expression::Identifier { .. } => Vec::new(),
    }
}

fn check_member_accessibility(
    property: &InterfacePropertyType,
    property_name: &str,
    span: crate::syntax::TextSpan,
    environment: &HashMap<String, Vec<String>>,
    symbols: &ScopedSymbolTable<'_>,
) -> Option<Diagnostic> {
    let declaring_class = property.declaring_class.as_deref()?;
    let current_class = environment
        .get("this")
        .and_then(|types| types.first())
        .map(|class_name| class_name.strip_prefix("typeof ").unwrap_or(class_name));
    match property.modifiers.accessibility? {
        PropertyAccessibility::Private if current_class != Some(declaring_class) => {
            Some(Diagnostic::new(
                2341,
                format!(
                    "Property '{property_name}' is private and only accessible within class '{declaring_class}'."
                ),
                span,
            ))
        }
        PropertyAccessibility::Protected
            if !current_class.is_some_and(|class_name| {
                symbols.is_same_or_derived_from(class_name, declaring_class)
            }) =>
        {
            Some(Diagnostic::new(
                2445,
                format!(
                    "Property '{property_name}' is protected and only accessible within class '{declaring_class}' and its subclasses."
                ),
                span,
            ))
        }
        PropertyAccessibility::Private | PropertyAccessibility::Protected => None,
    }
}

fn check_property_accesses_in_call(
    callee: &Expression,
    arguments: &[Expression],
    environment: &HashMap<String, Vec<String>>,
    symbols: &ScopedSymbolTable<'_>,
) -> Vec<Diagnostic> {
    check_property_accesses(callee, environment, symbols)
        .into_iter()
        .chain(
            arguments
                .iter()
                .flat_map(|argument| check_property_accesses(argument, environment, symbols)),
        )
        .collect()
}

#[derive(Clone, Copy)]
pub(super) struct ExpressionCheckOptions<'a> {
    pub(super) strict_null_checks: bool,
    pub(super) constant_bindings: &'a HashSet<String>,
    pub(super) constructor_class: Option<&'a str>,
}

pub(super) fn readonly_property_assignment<'expression>(
    left: &'expression Expression,
    environment: &HashMap<String, Vec<String>>,
    symbols: &ScopedSymbolTable<'_>,
) -> Option<&'expression str> {
    let Expression::PropertyAccessExpression { receiver, name, .. } = left else {
        return None;
    };
    let receiver_types = infer_expression_types(receiver, environment, symbols)?;
    receiver_types
        .iter()
        .filter_map(|receiver_type| symbols.properties_for_type(receiver_type))
        .any(|properties| {
            properties
                .iter()
                .any(|property| property.name == *name && property.modifiers.readonly)
        })
        .then_some(name)
}

pub(super) fn is_constructor_readonly_property_initialization(
    left: &Expression,
    options: ExpressionCheckOptions<'_>,
    symbols: &ScopedSymbolTable<'_>,
) -> bool {
    let Some(class_name) = options.constructor_class else {
        return false;
    };
    let Expression::PropertyAccessExpression { receiver, name, .. } = left else {
        return false;
    };
    if !matches!(
        receiver.as_ref(),
        Expression::Identifier { name: receiver_name, .. } if receiver_name == "this"
    ) {
        return false;
    }
    symbols
        .interface_properties(class_name)
        .is_some_and(|properties| {
            properties.iter().any(|property| {
                property.name == *name
                    && property.modifiers.readonly
                    && property.declaring_class.as_deref() == Some(class_name)
            })
        })
}

pub(super) fn check_expression_calls(
    expression: &Expression,
    environment: &HashMap<String, Vec<String>>,
    functions: &CallableMap,
    symbols: &ScopedSymbolTable<'_>,
    options: ExpressionCheckOptions<'_>,
) -> Vec<Diagnostic> {
    match expression {
        Expression::CallExpression {
            callee,
            arguments,
            span,
        } => check_call_expression(
            callee,
            arguments,
            *span,
            environment,
            functions,
            symbols,
            options,
        ),
        Expression::NewExpression {
            constructor,
            arguments,
            span,
        } => check_new_expression_calls(
            constructor,
            arguments,
            *span,
            environment,
            functions,
            symbols,
            options,
        ),
        binary @ Expression::BinaryExpression { .. } => {
            check_binary_expression(binary, environment, functions, symbols, options)
        }
        Expression::ConditionalExpression {
            condition,
            when_true,
            when_false,
            ..
        } => check_expression_call_children(
            [condition.as_ref(), when_true.as_ref(), when_false.as_ref()],
            environment,
            functions,
            symbols,
            options,
        ),
        Expression::AssignmentExpression {
            left,
            right,
            operator,
            ..
        } => check_assignment_expression(
            left,
            right,
            *operator,
            environment,
            functions,
            symbols,
            options,
        ),
        Expression::ParenthesizedExpression { expression, .. }
        | Expression::TypeAssertionExpression { expression, .. } => {
            check_expression_calls(expression, environment, functions, symbols, options)
        }
        Expression::UnaryExpression { operand, .. } => {
            check_expression_calls(operand, environment, functions, symbols, options)
        }
        Expression::PropertyAccessExpression { receiver, .. } => {
            check_expression_calls(receiver, environment, functions, symbols, options)
        }
        Expression::ElementAccessExpression {
            receiver, argument, ..
        } => check_expression_call_children(
            [receiver.as_ref(), argument.as_ref()],
            environment,
            functions,
            symbols,
            options,
        ),
        Expression::ObjectLiteral { properties, .. } => properties
            .iter()
            .flat_map(|property| {
                check_expression_calls(property.value(), environment, functions, symbols, options)
            })
            .collect(),
        Expression::ArrayLiteral { elements, .. } => elements
            .iter()
            .flat_map(|element| {
                check_expression_calls(element, environment, functions, symbols, options)
            })
            .collect(),
        Expression::NumberLiteral { .. }
        | Expression::BooleanLiteral { .. }
        | Expression::NullLiteral { .. }
        | Expression::StringLiteral { .. }
        | Expression::ArrowFunction { .. }
        | Expression::Identifier { .. } => Vec::new(),
    }
}

fn check_new_expression_calls(
    constructor: &Expression,
    arguments: &[Expression],
    span: crate::syntax::TextSpan,
    environment: &HashMap<String, Vec<String>>,
    functions: &CallableMap,
    symbols: &ScopedSymbolTable<'_>,
    options: ExpressionCheckOptions<'_>,
) -> Vec<Diagnostic> {
    let mut diagnostics = check_constructor_arguments(
        constructor,
        arguments,
        span,
        environment,
        functions,
        symbols,
        options.strict_null_checks,
    );
    diagnostics.extend(check_expression_calls(
        constructor,
        environment,
        functions,
        symbols,
        options,
    ));
    for argument in arguments {
        diagnostics.extend(check_expression_calls(
            argument,
            environment,
            functions,
            symbols,
            options,
        ));
    }
    diagnostics
}

fn check_expression_call_children<'expression>(
    expressions: impl IntoIterator<Item = &'expression Expression>,
    environment: &HashMap<String, Vec<String>>,
    functions: &CallableMap,
    symbols: &ScopedSymbolTable<'_>,
    options: ExpressionCheckOptions<'_>,
) -> Vec<Diagnostic> {
    expressions
        .into_iter()
        .flat_map(|expression| {
            check_expression_calls(expression, environment, functions, symbols, options)
        })
        .collect()
}
