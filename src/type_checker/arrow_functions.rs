use std::collections::{HashMap, HashSet};

use crate::binder::ScopedSymbolTable;
use crate::syntax::{ArrowFunctionBody, Diagnostic, Expression, FunctionParameter, TypeReference};

use super::callables::CallableMap;
use super::loops::FunctionBodyOwner;

#[derive(Clone, Copy)]
struct ArrowCheckContext<'context, 'symbols> {
    environment: &'context HashMap<String, Vec<String>>,
    functions: &'context CallableMap,
    symbols: &'context ScopedSymbolTable<'symbols>,
    constant_bindings: &'context HashSet<String>,
    strict_null_checks: bool,
}

pub(super) fn check_arrow_function_expressions(
    expression: &Expression,
    environment: &HashMap<String, Vec<String>>,
    functions: &CallableMap,
    symbols: &ScopedSymbolTable<'_>,
    constant_bindings: &HashSet<String>,
    strict_null_checks: bool,
) -> Vec<Diagnostic> {
    let context = ArrowCheckContext {
        environment,
        functions,
        symbols,
        constant_bindings,
        strict_null_checks,
    };
    check_expression(expression, &context)
}

fn check_expression(
    expression: &Expression,
    context: &ArrowCheckContext<'_, '_>,
) -> Vec<Diagnostic> {
    match expression {
        Expression::ArrowFunction {
            parameters,
            return_type,
            body,
            ..
        } => check_arrow_function(parameters, return_type.as_ref(), body, context),
        Expression::BinaryExpression { left, right, .. }
        | Expression::AssignmentExpression { left, right, .. } => {
            check_children([left.as_ref(), right.as_ref()], context)
        }
        Expression::ConditionalExpression {
            condition,
            when_true,
            when_false,
            ..
        } => check_children(
            [condition.as_ref(), when_true.as_ref(), when_false.as_ref()],
            context,
        ),
        Expression::ParenthesizedExpression { expression, .. }
        | Expression::TypeAssertionExpression { expression, .. }
        | Expression::UnaryExpression {
            operand: expression,
            ..
        }
        | Expression::PropertyAccessExpression {
            receiver: expression,
            ..
        } => check_expression(expression, context),
        Expression::CallExpression {
            callee, arguments, ..
        } => check_children(
            std::iter::once(callee.as_ref()).chain(arguments.iter()),
            context,
        ),
        Expression::NewExpression {
            constructor,
            arguments,
            ..
        } => check_children(
            std::iter::once(constructor.as_ref()).chain(arguments.iter()),
            context,
        ),
        Expression::ElementAccessExpression {
            receiver, argument, ..
        } => check_children([receiver.as_ref(), argument.as_ref()], context),
        Expression::ObjectLiteral { properties, .. } => check_children(
            properties.iter().map(crate::syntax::ObjectProperty::value),
            context,
        ),
        Expression::ArrayLiteral { elements, .. } => check_children(elements.iter(), context),
        Expression::NumberLiteral { .. }
        | Expression::BooleanLiteral { .. }
        | Expression::NullLiteral { .. }
        | Expression::StringLiteral { .. }
        | Expression::Identifier { .. } => Vec::new(),
    }
}

fn check_arrow_function(
    parameters: &[FunctionParameter],
    return_type: Option<&TypeReference>,
    body: &ArrowFunctionBody,
    context: &ArrowCheckContext<'_, '_>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut arrow_environment = context.environment.clone();
    for parameter in parameters {
        if let Some(initializer) = parameter.initializer() {
            if let Some(annotation) = parameter.type_annotation() {
                diagnostics.extend(super::check_expression_assignability(
                    initializer,
                    annotation,
                    initializer.span(),
                    context.symbols,
                    &arrow_environment,
                    context.strict_null_checks,
                ));
            }
            diagnostics.extend(super::check_expression_statement(
                initializer,
                &arrow_environment,
                context.functions,
                context.symbols,
                context.constant_bindings,
                context.strict_null_checks,
                None,
            ));
        }
        let parameter_types = if let Some(annotation) = parameter.type_annotation() {
            diagnostics.extend(super::check_type_reference(annotation, context.symbols));
            context.symbols.resolve_annotation(annotation)
        } else if let Some(initializer) = parameter.initializer() {
            super::infer_expression_types(initializer, &arrow_environment, context.symbols)
                .unwrap_or_else(|| vec!["any".to_owned()])
        } else {
            vec!["any".to_owned()]
        };
        arrow_environment.insert(parameter.name().to_owned(), parameter_types);
    }
    if let Some(return_type) = return_type {
        diagnostics.extend(super::check_type_reference(return_type, context.symbols));
    }
    let arrow_context = ArrowCheckContext {
        environment: &arrow_environment,
        ..*context
    };
    match body {
        ArrowFunctionBody::Expression(expression) => {
            diagnostics.extend(check_arrow_expression_body(
                expression,
                return_type,
                &arrow_context,
            ));
        }
        ArrowFunctionBody::Block(statements) => {
            diagnostics.extend(super::check_nested_function_body(
                statements,
                FunctionBodyOwner::Arrow(return_type),
                context.symbols,
                &arrow_environment,
                context.functions,
                context.constant_bindings,
                context.strict_null_checks,
            ));
        }
    }
    diagnostics
}

fn check_arrow_expression_body(
    expression: &Expression,
    return_type: Option<&TypeReference>,
    context: &ArrowCheckContext<'_, '_>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    if let Some(return_type) = return_type
        && let Some(actual_types) =
            super::infer_expression_types(expression, context.environment, context.symbols)
        && let Some(diagnostic) = super::check_assignability(
            &actual_types,
            return_type,
            expression.span(),
            context.symbols,
            context.strict_null_checks,
        )
    {
        diagnostics.push(diagnostic);
    }
    diagnostics.extend(super::check_expression_statement(
        expression,
        context.environment,
        context.functions,
        context.symbols,
        context.constant_bindings,
        context.strict_null_checks,
        None,
    ));
    diagnostics
}

fn check_children<'expression>(
    expressions: impl IntoIterator<Item = &'expression Expression>,
    context: &ArrowCheckContext<'_, '_>,
) -> Vec<Diagnostic> {
    expressions
        .into_iter()
        .flat_map(|expression| check_expression(expression, context))
        .collect()
}
