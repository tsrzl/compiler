//! Type checking for the syntax constructs currently represented in the AST.

mod arrow_functions;
mod callables;
mod enums;
mod expressions;
mod loops;
mod operators;
mod statements;

use crate::binder::ScopedSymbolTable;
use crate::syntax::{
    AssignmentOperator, BinaryOperator, Diagnostic, Expression, FunctionBodyStatement,
    FunctionDeclaration, ObjectProperty, Program, ReturnStatement, Statement, TypeReference,
    UnaryOperator, VariableDeclaration, VariableDeclarationKind,
};
use std::collections::{HashMap, HashSet};

use callables::{
    CallableMap, CallableSignature, callable_return_types, constructor_callable_name,
    declared_callables,
};
use expressions::{
    ExpressionCheckOptions, check_expression_calls, check_expression_names,
    check_property_accesses, is_constructor_readonly_property_initialization,
    readonly_property_assignment,
};
use loops::FunctionBodyOwner;
use operators::check_binary_operator;
use statements::{
    check_class_declaration, check_control_flow_statement, check_exported_types,
    check_exported_values, check_function_declaration, check_interface_declaration,
    check_top_level_control_flow,
};

pub(crate) fn check(
    program: &Program,
    symbols: &ScopedSymbolTable<'_>,
    strict_null_checks: bool,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let functions = declared_callables(program);
    let mut declared_names = declared_names(program, &functions);
    let mut environment = initial_environment(&functions, symbols, &mut declared_names);
    bind_class_values(program, &mut declared_names, &mut environment);
    let mut constant_bindings = constant_bindings(program);

    for statement in program.statements() {
        match statement.declaration() {
            Statement::VariableDeclaration(declaration) => {
                if let Some(annotation) = declaration.type_annotation() {
                    diagnostics.extend(check_type_reference(annotation, symbols));
                }
                let (mut declaration_diagnostics, types) = check_variable(
                    declaration,
                    symbols,
                    &environment,
                    &functions,
                    &mut constant_bindings,
                    strict_null_checks,
                    None,
                );
                diagnostics.append(&mut declaration_diagnostics);
                if let Some(types) = types {
                    environment.insert(declaration.name().to_owned(), types);
                }
            }
            Statement::FunctionDeclaration(function) => diagnostics.extend(
                check_function_declaration(
                    function,
                    symbols,
                    &environment,
                    &functions,
                    &constant_bindings,
                    strict_null_checks,
                    None,
                ),
            ),
            Statement::ExpressionStatement(expression) | Statement::ExportDefault(expression) => {
                diagnostics.extend(check_expression_statement(
                    expression,
                    &environment,
                    &functions,
                    symbols,
                    &constant_bindings,
                    strict_null_checks,
                    None,
                ));
            }
            Statement::ControlFlowStatement(control_flow) => diagnostics.extend(
                check_top_level_control_flow(
                    control_flow,
                    symbols,
                    &environment,
                    &functions,
                    &constant_bindings,
                    strict_null_checks,
                ),
            ),
            Statement::Break { span } => diagnostics.push(Diagnostic::new(
                1105,
                "A 'break' statement can only be used within an enclosing iteration or switch statement.",
                *span,
            )),
            Statement::Continue { span } => diagnostics.push(Diagnostic::new(
                1104,
                "A 'continue' statement can only be used within an enclosing iteration statement.",
                *span,
            )),
            Statement::ExportNamed(specifiers) => {
                diagnostics.extend(check_exported_values(specifiers, &declared_names));
            }
            Statement::ExportTypeNamed(specifiers) => {
                diagnostics.extend(check_exported_types(specifiers, symbols));
            }
            Statement::InterfaceDeclaration(declaration) => diagnostics.extend(
                check_interface_declaration(declaration, symbols),
            ),
            Statement::ClassDeclaration(declaration) => diagnostics.extend(
                check_class_declaration(
                    declaration,
                    symbols,
                    &environment,
                    &functions,
                    &constant_bindings,
                    strict_null_checks,
                ),
            ),
            Statement::EnumDeclaration(declaration) => {
                diagnostics.extend(enums::check_enum_declaration(declaration));
            }
            Statement::TypeAliasDeclaration(declaration) => {
                diagnostics.extend(check_type_reference(declaration.type_annotation(), symbols));
            }
            Statement::ImportDeclaration(_)
            | Statement::ExportNamedFrom(_)
            | Statement::ExportAll(_)
            | Statement::ExportedDeclaration(_) => {}
        }
    }

    diagnostics
}

fn bind_class_values(
    program: &Program,
    declared_names: &mut HashSet<String>,
    environment: &mut HashMap<String, Vec<String>>,
) {
    for statement in program.statements() {
        if let Some(class) = statement.as_class_declaration() {
            declared_names.insert(class.name().to_owned());
            environment.insert(
                class.name().to_owned(),
                vec![format!("typeof {}", class.name())],
            );
        }
        if let Some(enum_declaration) = statement.as_enum_declaration() {
            declared_names.insert(enum_declaration.name().to_owned());
            environment.insert(
                enum_declaration.name().to_owned(),
                vec![format!("typeof {}", enum_declaration.name())],
            );
        }
    }
}

fn constant_bindings(program: &Program) -> HashSet<String> {
    program
        .statements()
        .iter()
        .filter_map(Statement::as_variable_declaration)
        .filter(|declaration| declaration.declaration_kind() == VariableDeclarationKind::Const)
        .map(|declaration| declaration.name().to_owned())
        .collect()
}

fn declared_names(program: &Program, functions: &CallableMap) -> HashSet<String> {
    let mut names = functions.keys().cloned().collect::<HashSet<_>>();
    for statement in program.statements() {
        if let Some(declaration) = statement.as_variable_declaration() {
            names.insert(declaration.name().to_owned());
        }
        if let Some(declaration) = statement.as_interface_declaration() {
            names.insert(declaration.name().to_owned());
        }
        if let Some(declaration) = statement.as_type_alias_declaration() {
            names.insert(declaration.name().to_owned());
        }
        if let Some(declaration) = statement.as_enum_declaration() {
            names.insert(declaration.name().to_owned());
        }
    }
    names
}

fn initial_environment(
    functions: &CallableMap,
    symbols: &ScopedSymbolTable<'_>,
    declared_names: &mut HashSet<String>,
) -> HashMap<String, Vec<String>> {
    let mut environment = functions
        .iter()
        .map(|(name, signature)| (name.clone(), callable_return_types(signature, symbols)))
        .collect::<HashMap<_, _>>();
    if let Some(imports) = symbols.imported_values() {
        environment.extend(
            imports
                .iter()
                .map(|(name, types)| (name.clone(), types.clone())),
        );
        declared_names.extend(imports.keys().cloned());
    }
    environment
        .entry("undefined".to_owned())
        .or_insert_with(|| vec!["undefined".to_owned()]);
    environment
}

fn check_variable(
    declaration: &VariableDeclaration,
    symbols: &ScopedSymbolTable<'_>,
    environment: &HashMap<String, Vec<String>>,
    functions: &CallableMap,
    constant_bindings: &mut HashSet<String>,
    strict_null_checks: bool,
    constructor_class: Option<&str>,
) -> (Vec<Diagnostic>, Option<Vec<String>>) {
    let mut diagnostics = Vec::new();
    let binding_types = match (declaration.type_annotation(), declaration.initializer()) {
        (Some(annotation), Some(Expression::ObjectLiteral { properties, span })) => {
            if let Some(diagnostic) = check_object_literal(
                properties,
                annotation,
                *span,
                symbols,
                environment,
                strict_null_checks,
            ) {
                diagnostics.push(diagnostic);
            }
            Some(symbols.resolve_annotation(annotation))
        }
        (Some(annotation), Some(Expression::ArrayLiteral { elements, .. })) => {
            let expected_types = symbols.resolve_annotation(annotation);
            let expected_element_types = expected_types
                .iter()
                .filter_map(|type_name| type_name.strip_suffix("[]"))
                .map(str::to_owned)
                .collect::<Vec<_>>();
            if expected_element_types.is_empty() {
                if let Some(actual_types) =
                    infer_array_expression_types(elements, environment, symbols)
                    && let Some(diagnostic) = check_assignability(
                        &actual_types,
                        annotation,
                        declaration.name_span(),
                        symbols,
                        strict_null_checks,
                    )
                {
                    diagnostics.push(diagnostic);
                }
            } else {
                diagnostics.extend(check_array_elements(
                    elements,
                    &expected_element_types,
                    symbols,
                    environment,
                    strict_null_checks,
                ));
            }
            Some(expected_types)
        }
        (Some(annotation), Some(initializer)) => {
            diagnostics.extend(check_expression_assignability(
                initializer,
                annotation,
                declaration.name_span(),
                symbols,
                environment,
                strict_null_checks,
            ));
            Some(symbols.resolve_annotation(annotation))
        }
        (Some(annotation), None) => Some(symbols.resolve_annotation(annotation)),
        (None, Some(initializer)) => infer_expression_types(initializer, environment, symbols),
        (None, None) => None,
    };

    if let Some(initializer) = declaration.initializer() {
        diagnostics.extend(check_expression_names(initializer, environment, symbols));
        diagnostics.extend(check_expression_calls(
            initializer,
            environment,
            functions,
            symbols,
            ExpressionCheckOptions {
                strict_null_checks,
                constant_bindings,
                constructor_class,
            },
        ));
        diagnostics.extend(check_property_accesses(initializer, environment, symbols));
        diagnostics.extend(arrow_functions::check_arrow_function_expressions(
            initializer,
            environment,
            functions,
            symbols,
            constant_bindings,
            strict_null_checks,
        ));
    }
    match declaration.declaration_kind() {
        VariableDeclarationKind::Const => {
            constant_bindings.insert(declaration.name().to_owned());
        }
        VariableDeclarationKind::Let | VariableDeclarationKind::Var => {
            constant_bindings.remove(declaration.name());
        }
    }
    (diagnostics, binding_types)
}

fn check_array_elements(
    elements: &[Expression],
    expected_element_types: &[String],
    symbols: &ScopedSymbolTable<'_>,
    environment: &HashMap<String, Vec<String>>,
    strict_null_checks: bool,
) -> Vec<Diagnostic> {
    elements
        .iter()
        .filter_map(|element| {
            let actual_types = infer_expression_types(element, environment, symbols)?;
            if types_are_assignable(
                &actual_types,
                expected_element_types,
                symbols,
                strict_null_checks,
            ) {
                return None;
            }
            Some(Diagnostic::new(
                2322,
                format!(
                    "Type '{}' is not assignable to type '{}'.",
                    actual_types.join(" | "),
                    expected_element_types.join(" | ")
                ),
                element.span(),
            ))
        })
        .collect()
}

fn check_object_literal(
    properties: &[ObjectProperty],
    annotation: &TypeReference,
    span: crate::syntax::TextSpan,
    symbols: &ScopedSymbolTable<'_>,
    environment: &HashMap<String, Vec<String>>,
    strict_null_checks: bool,
) -> Option<Diagnostic> {
    let expected_names = symbols.resolve_annotation(annotation);
    if expected_names
        .iter()
        .any(|name| name == "any" || name == "unknown" || !symbols.is_type_name_defined(name))
    {
        return None;
    }

    if expected_names.len() == 1
        && let Some(interface_properties) = symbols.interface_properties(&expected_names[0])
    {
        for property in properties {
            if !interface_properties
                .iter()
                .any(|expected_property| expected_property.name == property.name())
            {
                return Some(Diagnostic::new(
                    2353,
                    format!(
                        "Object literal may only specify known properties, and '{}' does not exist in type '{}'.",
                        property.name(),
                        expected_names[0]
                    ),
                    property.name_span(),
                ));
            }
        }
    }

    let mut found_interface = false;
    let mut first_diagnostic = None;
    for expected_name in &expected_names {
        let Some(interface_properties) = symbols.interface_properties(expected_name) else {
            continue;
        };
        found_interface = true;
        let mut candidate_diagnostic = None;
        for expected_property in interface_properties {
            let Some(actual_property) = properties
                .iter()
                .find(|property| property.name() == expected_property.name)
            else {
                if !expected_property.optional {
                    candidate_diagnostic = Some(Diagnostic::new(
                        2741,
                        format!(
                            "Property '{}' is missing in the object literal.",
                            expected_property.name
                        ),
                        span,
                    ));
                    break;
                }
                continue;
            };

            let Some(actual_types) =
                infer_expression_types(actual_property.value(), environment, symbols)
            else {
                continue;
            };
            let expected_types = symbols.resolve_names(&expected_property.type_names);
            if expected_types
                .iter()
                .any(|name| !symbols.is_type_name_defined(name))
            {
                continue;
            }
            if !types_are_assignable(&actual_types, &expected_types, symbols, strict_null_checks) {
                candidate_diagnostic = Some(Diagnostic::new(
                    2322,
                    format!(
                        "Type '{}' is not assignable to type '{}' for property '{}'.",
                        actual_types.join(" | "),
                        expected_types.join(" | "),
                        expected_property.name
                    ),
                    actual_property.value().span(),
                ));
                break;
            }
        }

        candidate_diagnostic.as_ref()?;
        if first_diagnostic.is_none() {
            first_diagnostic = candidate_diagnostic;
        }
    }

    first_diagnostic.or_else(|| {
        (!found_interface).then(|| {
            Diagnostic::new(
                2322,
                format!(
                    "Type '{{}}' is not assignable to type '{}'.",
                    expected_names.join(" | ")
                ),
                span,
            )
        })
    })
}

fn check_function(
    function: &FunctionDeclaration,
    symbols: &ScopedSymbolTable<'_>,
    outer_environment: &HashMap<String, Vec<String>>,
    functions: &CallableMap,
    outer_constant_bindings: &HashSet<String>,
    strict_null_checks: bool,
    constructor_class: Option<&str>,
) -> Vec<Diagnostic> {
    let mut environment = outer_environment.clone();
    let mut constant_bindings = outer_constant_bindings.clone();
    let mut callable_signatures = functions.clone();
    for parameter in function.parameters() {
        constant_bindings.remove(parameter.name());
        if let Some(annotation) = parameter.type_annotation() {
            environment.insert(
                parameter.name().to_owned(),
                symbols.resolve_annotation(annotation),
            );
        } else if let Some(initializer) = parameter.initializer() {
            let types = infer_expression_types(initializer, &environment, symbols)
                .unwrap_or_else(|| vec!["any".to_owned()]);
            environment.insert(parameter.name().to_owned(), types);
        }
    }
    check_function_body(
        function.body(),
        FunctionBodyOwner::Function(function, constructor_class),
        symbols,
        &mut environment,
        &mut callable_signatures,
        &mut constant_bindings,
        strict_null_checks,
    )
}

fn check_function_body(
    statements: &[FunctionBodyStatement],
    owner: FunctionBodyOwner<'_>,
    symbols: &ScopedSymbolTable<'_>,
    environment: &mut HashMap<String, Vec<String>>,
    functions: &mut CallableMap,
    constant_bindings: &mut HashSet<String>,
    strict_null_checks: bool,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for statement in statements {
        match statement {
            FunctionBodyStatement::VariableDeclaration(declaration) => {
                if let Some(signature) = CallableSignature::from_arrow(declaration) {
                    let return_types = callable_return_types(&signature, symbols);
                    functions.insert(declaration.name().to_owned(), signature);
                    environment.insert(declaration.name().to_owned(), return_types);
                }
                if let Some(annotation) = declaration.type_annotation() {
                    diagnostics.extend(check_type_reference(annotation, symbols));
                }
                let (mut declaration_diagnostics, binding_types) = check_variable(
                    declaration,
                    symbols,
                    environment,
                    functions,
                    constant_bindings,
                    strict_null_checks,
                    owner.constructor_class(),
                );
                diagnostics.append(&mut declaration_diagnostics);
                if let Some(binding_types) = binding_types {
                    environment.insert(declaration.name().to_owned(), binding_types);
                }
            }
            FunctionBodyStatement::Expression(expression)
            | FunctionBodyStatement::Throw(expression) => {
                diagnostics.extend(check_expression_statement(
                    expression,
                    environment,
                    functions,
                    symbols,
                    constant_bindings,
                    strict_null_checks,
                    owner.constructor_class(),
                ));
            }
            FunctionBodyStatement::Return(statement) => diagnostics.extend(check_return_statement(
                statement,
                owner,
                symbols,
                environment,
                functions,
                constant_bindings,
                strict_null_checks,
            )),
            control_flow @ (FunctionBodyStatement::If { .. }
            | FunctionBodyStatement::While { .. }
            | FunctionBodyStatement::DoWhile { .. }
            | FunctionBodyStatement::For { .. }
            | FunctionBodyStatement::ForOf { .. }
            | FunctionBodyStatement::ForIn { .. }
            | FunctionBodyStatement::Switch { .. }
            | FunctionBodyStatement::Try { .. }) => {
                diagnostics.extend(check_control_flow_statement(
                    control_flow,
                    owner,
                    symbols,
                    environment,
                    functions,
                    constant_bindings,
                    strict_null_checks,
                ));
            }
            FunctionBodyStatement::Break { .. } | FunctionBodyStatement::Continue { .. } => {}
        }
    }
    diagnostics
}

fn check_return_statement(
    statement: &ReturnStatement,
    owner: FunctionBodyOwner<'_>,
    symbols: &ScopedSymbolTable<'_>,
    environment: &HashMap<String, Vec<String>>,
    functions: &CallableMap,
    constant_bindings: &HashSet<String>,
    strict_null_checks: bool,
) -> Vec<Diagnostic> {
    if owner.is_function() {
        return check_function_return_statement(
            statement,
            owner,
            symbols,
            environment,
            functions,
            constant_bindings,
            strict_null_checks,
        );
    }
    vec![Diagnostic::new(
        1108,
        "A 'return' statement can only be used within a function body.",
        statement.span(),
    )]
}

fn check_nested_function_body(
    statements: &[FunctionBodyStatement],
    owner: FunctionBodyOwner<'_>,
    symbols: &ScopedSymbolTable<'_>,
    environment: &HashMap<String, Vec<String>>,
    functions: &CallableMap,
    constant_bindings: &HashSet<String>,
    strict_null_checks: bool,
) -> Vec<Diagnostic> {
    let mut nested_environment = environment.clone();
    let mut nested_constant_bindings = constant_bindings.clone();
    let mut nested_functions = functions.clone();
    check_function_body(
        statements,
        owner,
        symbols,
        &mut nested_environment,
        &mut nested_functions,
        &mut nested_constant_bindings,
        strict_null_checks,
    )
}

fn check_function_return_statement(
    statement: &ReturnStatement,
    owner: FunctionBodyOwner<'_>,
    symbols: &ScopedSymbolTable<'_>,
    environment: &HashMap<String, Vec<String>>,
    functions: &CallableMap,
    constant_bindings: &HashSet<String>,
    strict_null_checks: bool,
) -> Vec<Diagnostic> {
    let Some(expression) = statement.expression() else {
        return Vec::new();
    };
    let mut diagnostics = check_expression_names(expression, environment, symbols);
    diagnostics.extend(check_expression_calls(
        expression,
        environment,
        functions,
        symbols,
        ExpressionCheckOptions {
            strict_null_checks,
            constant_bindings,
            constructor_class: owner.constructor_class(),
        },
    ));
    diagnostics.extend(check_property_accesses(expression, environment, symbols));
    if let Some(return_type) = owner.return_type() {
        diagnostics.extend(check_expression_assignability(
            expression,
            return_type,
            expression.span(),
            symbols,
            environment,
            strict_null_checks,
        ));
    }
    diagnostics
}

fn check_expression_assignability(
    expression: &Expression,
    expected_annotation: &TypeReference,
    span: crate::syntax::TextSpan,
    symbols: &ScopedSymbolTable<'_>,
    environment: &HashMap<String, Vec<String>>,
    strict_null_checks: bool,
) -> Vec<Diagnostic> {
    match expression {
        Expression::ConditionalExpression {
            when_true,
            when_false,
            ..
        } => check_expression_assignability(
            when_true,
            expected_annotation,
            when_true.span(),
            symbols,
            environment,
            strict_null_checks,
        )
        .into_iter()
        .chain(check_expression_assignability(
            when_false,
            expected_annotation,
            when_false.span(),
            symbols,
            environment,
            strict_null_checks,
        ))
        .collect(),
        Expression::ParenthesizedExpression {
            expression: inner, ..
        } => check_expression_assignability(
            inner,
            expected_annotation,
            inner.span(),
            symbols,
            environment,
            strict_null_checks,
        ),
        _ => infer_expression_types(expression, environment, symbols)
            .and_then(|actual_types| {
                check_assignability(
                    &actual_types,
                    expected_annotation,
                    span,
                    symbols,
                    strict_null_checks,
                )
            })
            .into_iter()
            .collect(),
    }
}

fn check_expression_statement(
    expression: &Expression,
    environment: &HashMap<String, Vec<String>>,
    functions: &CallableMap,
    symbols: &ScopedSymbolTable<'_>,
    constant_bindings: &HashSet<String>,
    strict_null_checks: bool,
    constructor_class: Option<&str>,
) -> Vec<Diagnostic> {
    check_expression_names(expression, environment, symbols)
        .into_iter()
        .chain(check_expression_calls(
            expression,
            environment,
            functions,
            symbols,
            ExpressionCheckOptions {
                strict_null_checks,
                constant_bindings,
                constructor_class,
            },
        ))
        .chain(check_property_accesses(expression, environment, symbols))
        .chain(arrow_functions::check_arrow_function_expressions(
            expression,
            environment,
            functions,
            symbols,
            constant_bindings,
            strict_null_checks,
        ))
        .collect()
}

pub(super) fn infer_expression_types(
    expression: &Expression,
    environment: &HashMap<String, Vec<String>>,
    symbols: &ScopedSymbolTable<'_>,
) -> Option<Vec<String>> {
    if let Some(path) = expression_path(expression)
        && let Some(types) = environment.get(&path)
    {
        return Some(types.clone());
    }

    match expression {
        Expression::NumberLiteral { value, .. } => Some(vec![
            crate::type_system::numeric_literal_type(value).to_owned(),
        ]),
        Expression::BooleanLiteral { .. }
        | Expression::UnaryExpression {
            operator: UnaryOperator::LogicalNot,
            ..
        } => Some(vec!["boolean".to_owned()]),
        Expression::NullLiteral { .. } => Some(vec!["null".to_owned()]),
        Expression::StringLiteral { .. } => Some(vec!["string".to_owned()]),
        Expression::TypeAssertionExpression {
            type_annotation, ..
        } => Some(symbols.resolve_annotation(type_annotation)),
        Expression::Identifier { name, .. } => environment.get(name).cloned(),
        Expression::ParenthesizedExpression { expression, .. } => {
            infer_expression_types(expression, environment, symbols)
        }
        Expression::AssignmentExpression {
            left,
            right,
            operator,
            ..
        } => infer_assignment_expression_types(*operator, left, right, environment, symbols),
        Expression::CallExpression { callee, .. } => {
            infer_call_expression_types(callee, environment, symbols)
        }
        Expression::NewExpression { constructor, .. } => match constructor.as_ref() {
            Expression::Identifier { name, .. } => Some(vec![name.clone()]),
            _ => None,
        },
        Expression::ElementAccessExpression { receiver, .. } => {
            infer_element_access_types(receiver, environment, symbols)
        }
        Expression::PropertyAccessExpression { receiver, name, .. } => {
            let receiver_types = infer_expression_types(receiver, environment, symbols)?;
            let mut property_types = Vec::new();
            for receiver_type in receiver_types {
                if name == "length" && (receiver_type.ends_with("[]") || receiver_type == "string")
                {
                    if !property_types
                        .iter()
                        .any(|property_type| property_type == "number")
                    {
                        property_types.push("number".to_owned());
                    }
                    continue;
                }
                let Some(properties) = symbols.properties_for_type(&receiver_type) else {
                    continue;
                };
                if let Some(property) = properties.iter().find(|property| property.name == *name) {
                    for property_type in symbols.resolve_names(&property.type_names) {
                        if !property_types.contains(&property_type) {
                            property_types.push(property_type);
                        }
                    }
                }
            }
            (!property_types.is_empty()).then_some(property_types)
        }
        Expression::UnaryExpression { operand, .. } => {
            let operand_types = infer_expression_types(operand, environment, symbols)?;
            (!operand_types.is_empty() && operand_types.iter().all(|name| name == "number"))
                .then(|| vec!["number".to_owned()])
        }
        Expression::ArrowFunction { .. } | Expression::ObjectLiteral { .. } => None,
        Expression::ArrayLiteral { elements, .. } => {
            infer_array_expression_types(elements, environment, symbols)
        }
        Expression::ConditionalExpression {
            when_true,
            when_false,
            ..
        } => {
            let true_types = infer_expression_types(when_true, environment, symbols)?;
            let false_types = infer_expression_types(when_false, environment, symbols)?;
            let mut result_types = Vec::new();
            for type_name in true_types.into_iter().chain(false_types) {
                if type_name == "any" {
                    return Some(vec!["any".to_owned()]);
                }
                if !result_types.contains(&type_name) {
                    result_types.push(type_name);
                }
            }
            (!result_types.is_empty()).then_some(result_types)
        }
        Expression::BinaryExpression {
            left,
            operator,
            right,
            ..
        } => infer_binary_expression_types(*operator, left, right, environment, symbols),
    }
}

fn infer_call_expression_types(
    callee: &Expression,
    environment: &HashMap<String, Vec<String>>,
    symbols: &ScopedSymbolTable<'_>,
) -> Option<Vec<String>> {
    match callee {
        Expression::Identifier { name, .. } => environment.get(name).cloned(),
        Expression::ParenthesizedExpression { expression, .. } => match expression.as_ref() {
            Expression::Identifier { name, .. } => environment.get(name).cloned(),
            _ => None,
        },
        Expression::PropertyAccessExpression { receiver, name, .. } => {
            infer_class_method_return_types(receiver, name, environment, symbols)
        }
        _ => None,
    }
}

fn infer_class_method_return_types(
    receiver: &Expression,
    method_name: &str,
    environment: &HashMap<String, Vec<String>>,
    symbols: &ScopedSymbolTable<'_>,
) -> Option<Vec<String>> {
    let receiver_types = infer_expression_types(receiver, environment, symbols)?;
    let mut return_types = Vec::new();
    for receiver_type in receiver_types {
        let Some(properties) = symbols.properties_for_type(&receiver_type) else {
            continue;
        };
        let Some(method_return_types) = properties
            .iter()
            .find(|property| property.name == method_name)
            .and_then(|property| property.method_return_types.as_deref())
        else {
            continue;
        };
        for return_type in symbols.resolve_names(method_return_types) {
            if !return_types.contains(&return_type) {
                return_types.push(return_type);
            }
        }
    }
    (!return_types.is_empty()).then_some(return_types)
}

fn infer_assignment_expression_types(
    operator: AssignmentOperator,
    left: &Expression,
    right: &Expression,
    environment: &HashMap<String, Vec<String>>,
    symbols: &ScopedSymbolTable<'_>,
) -> Option<Vec<String>> {
    match operator {
        AssignmentOperator::Assign => infer_expression_types(right, environment, symbols),
        AssignmentOperator::AddAssign => {
            infer_add_assignment_types(left, right, environment, symbols)
        }
    }
}

fn infer_binary_expression_types(
    operator: BinaryOperator,
    left: &Expression,
    right: &Expression,
    environment: &HashMap<String, Vec<String>>,
    symbols: &ScopedSymbolTable<'_>,
) -> Option<Vec<String>> {
    let left_types = infer_expression_types(left, environment, symbols)?;
    let right_types = infer_expression_types(right, environment, symbols)?;
    match operator {
        BinaryOperator::NullishCoalesce => {
            let operand_types = left_types
                .into_iter()
                .filter(|type_name| !matches!(type_name.as_str(), "null" | "undefined"))
                .chain(right_types);
            let mut result_types = Vec::new();
            for type_name in operand_types {
                if type_name == "any" {
                    return Some(vec!["any".to_owned()]);
                }
                if !result_types.contains(&type_name) {
                    result_types.push(type_name);
                }
            }
            (!result_types.is_empty()).then_some(result_types)
        }
        BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => {
            let operand_types = left_types.into_iter().chain(right_types);
            let mut result_types = Vec::new();
            for type_name in operand_types {
                if type_name == "any" {
                    return Some(vec!["any".to_owned()]);
                }
                if !result_types.contains(&type_name) {
                    result_types.push(type_name);
                }
            }
            (!result_types.is_empty()).then_some(result_types)
        }
        BinaryOperator::Add
            if left_types.iter().any(|type_name| type_name == "string")
                || right_types.iter().any(|type_name| type_name == "string") =>
        {
            Some(vec!["string".to_owned()])
        }
        BinaryOperator::Add
        | BinaryOperator::Subtract
        | BinaryOperator::Multiply
        | BinaryOperator::Divide
        | BinaryOperator::Remainder
            if !left_types.is_empty()
                && !right_types.is_empty()
                && left_types.iter().all(|type_name| type_name == "bigint")
                && right_types.iter().all(|type_name| type_name == "bigint") =>
        {
            Some(vec!["bigint".to_owned()])
        }
        BinaryOperator::Add
        | BinaryOperator::Subtract
        | BinaryOperator::Multiply
        | BinaryOperator::Divide
        | BinaryOperator::Remainder
            if !left_types.is_empty()
                && !right_types.is_empty()
                && left_types.iter().all(|type_name| type_name == "number")
                && right_types.iter().all(|type_name| type_name == "number") =>
        {
            Some(vec!["number".to_owned()])
        }
        BinaryOperator::LessThan
        | BinaryOperator::GreaterThan
        | BinaryOperator::LessThanOrEqual
        | BinaryOperator::GreaterThanOrEqual
        | BinaryOperator::Equal
        | BinaryOperator::StrictEqual
        | BinaryOperator::NotEqual
        | BinaryOperator::StrictNotEqual
            if !left_types.is_empty() && !right_types.is_empty() =>
        {
            Some(vec!["boolean".to_owned()])
        }
        _ => None,
    }
}

fn infer_element_access_types(
    receiver: &Expression,
    environment: &HashMap<String, Vec<String>>,
    symbols: &ScopedSymbolTable<'_>,
) -> Option<Vec<String>> {
    let receiver_types = infer_expression_types(receiver, environment, symbols)?;
    let mut element_types = Vec::new();
    for receiver_type in receiver_types {
        let element_type = receiver_type
            .strip_suffix("[]")
            .or_else(|| (receiver_type == "string").then_some("string"))
            .or_else(|| (receiver_type == "any").then_some("any"));
        if let Some(element_type) = element_type
            && !element_types
                .iter()
                .any(|existing| existing == element_type)
        {
            element_types.push(element_type.to_owned());
        }
    }
    (!element_types.is_empty()).then_some(element_types)
}

fn infer_add_assignment_types(
    left: &Expression,
    right: &Expression,
    environment: &HashMap<String, Vec<String>>,
    symbols: &ScopedSymbolTable<'_>,
) -> Option<Vec<String>> {
    let left_types = infer_expression_types(left, environment, symbols)?;
    let right_types = infer_expression_types(right, environment, symbols)?;
    if left_types.iter().any(|name| name == "string")
        || right_types.iter().any(|name| name == "string")
    {
        Some(vec!["string".to_owned()])
    } else if left_types.iter().any(|name| name == "any")
        || right_types.iter().any(|name| name == "any")
    {
        Some(vec!["any".to_owned()])
    } else if left_types.iter().all(|name| name == "number")
        && right_types.iter().all(|name| name == "number")
    {
        Some(vec!["number".to_owned()])
    } else {
        None
    }
}

fn infer_array_expression_types(
    elements: &[Expression],
    environment: &HashMap<String, Vec<String>>,
    symbols: &ScopedSymbolTable<'_>,
) -> Option<Vec<String>> {
    let mut element_types = Vec::new();
    for element in elements {
        element_types.extend(infer_expression_types(element, environment, symbols)?);
    }
    Some(vec![crate::type_system::array_type(element_types)])
}

fn expression_path(expression: &Expression) -> Option<String> {
    match expression {
        Expression::Identifier { name, .. } => Some(name.clone()),
        Expression::PropertyAccessExpression { receiver, name, .. } => {
            Some(format!("{}.{name}", expression_path(receiver)?))
        }
        _ => None,
    }
}

fn check_assignment_expression(
    left: &Expression,
    right: &Expression,
    operator: AssignmentOperator,
    environment: &HashMap<String, Vec<String>>,
    functions: &CallableMap,
    symbols: &ScopedSymbolTable<'_>,
    options: ExpressionCheckOptions<'_>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    if !is_assignment_target(left) {
        diagnostics.push(Diagnostic::new(
            2364,
            "The left-hand side of an assignment expression must be a variable or a property access.",
            left.span(),
        ));
    } else if let Some(name) = assignment_target_identifier(left)
        && options.constant_bindings.contains(name)
    {
        diagnostics.push(Diagnostic::new(
            2588,
            format!("Cannot assign to '{name}' because it is a constant."),
            left.span(),
        ));
    } else if let Some(name) = readonly_property_assignment(left, environment, symbols)
        && !is_constructor_readonly_property_initialization(left, options, symbols)
    {
        diagnostics.push(Diagnostic::new(
            2540,
            format!("Cannot assign to '{name}' because it is a read-only property."),
            left.span(),
        ));
    } else if let (Some(expected_types), Some(actual_types)) = (
        infer_expression_types(left, environment, symbols),
        match operator {
            AssignmentOperator::Assign => infer_expression_types(right, environment, symbols),
            AssignmentOperator::AddAssign => {
                infer_add_assignment_types(left, right, environment, symbols)
            }
        },
    ) && !types_are_assignable(
        &actual_types,
        &expected_types,
        symbols,
        options.strict_null_checks,
    ) {
        diagnostics.push(Diagnostic::new(
            2322,
            format!(
                "Type '{}' is not assignable to type '{}'.",
                actual_types.join(" | "),
                expected_types.join(" | ")
            ),
            left.span(),
        ));
    }
    diagnostics.extend(check_expression_calls(
        left,
        environment,
        functions,
        symbols,
        options,
    ));
    diagnostics.extend(check_expression_calls(
        right,
        environment,
        functions,
        symbols,
        options,
    ));
    diagnostics
}

fn is_assignment_target(expression: &Expression) -> bool {
    match expression {
        Expression::Identifier { .. }
        | Expression::PropertyAccessExpression { .. }
        | Expression::ElementAccessExpression { .. } => true,
        Expression::ParenthesizedExpression { expression, .. } => is_assignment_target(expression),
        Expression::NumberLiteral { .. }
        | Expression::BooleanLiteral { .. }
        | Expression::NullLiteral { .. }
        | Expression::StringLiteral { .. }
        | Expression::BinaryExpression { .. }
        | Expression::AssignmentExpression { .. }
        | Expression::ConditionalExpression { .. }
        | Expression::CallExpression { .. }
        | Expression::NewExpression { .. }
        | Expression::ArrowFunction { .. }
        | Expression::TypeAssertionExpression { .. }
        | Expression::UnaryExpression { .. }
        | Expression::ObjectLiteral { .. }
        | Expression::ArrayLiteral { .. } => false,
    }
}

fn assignment_target_identifier(expression: &Expression) -> Option<&str> {
    match expression {
        Expression::Identifier { name, .. } => Some(name),
        Expression::ParenthesizedExpression { expression, .. } => {
            assignment_target_identifier(expression)
        }
        Expression::NumberLiteral { .. }
        | Expression::BooleanLiteral { .. }
        | Expression::NullLiteral { .. }
        | Expression::StringLiteral { .. }
        | Expression::BinaryExpression { .. }
        | Expression::AssignmentExpression { .. }
        | Expression::ConditionalExpression { .. }
        | Expression::CallExpression { .. }
        | Expression::NewExpression { .. }
        | Expression::ArrowFunction { .. }
        | Expression::TypeAssertionExpression { .. }
        | Expression::PropertyAccessExpression { .. }
        | Expression::ElementAccessExpression { .. }
        | Expression::UnaryExpression { .. }
        | Expression::ObjectLiteral { .. }
        | Expression::ArrayLiteral { .. } => None,
    }
}

fn check_call_expression(
    callee: &Expression,
    arguments: &[Expression],
    span: crate::syntax::TextSpan,
    environment: &HashMap<String, Vec<String>>,
    functions: &CallableMap,
    symbols: &ScopedSymbolTable<'_>,
    options: ExpressionCheckOptions<'_>,
) -> Vec<Diagnostic> {
    let mut diagnostics = check_call_arguments(
        callee,
        arguments,
        span,
        environment,
        functions,
        symbols,
        options.strict_null_checks,
    );
    diagnostics.extend(check_expression_calls(
        callee,
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

fn check_binary_expression(
    expression: &Expression,
    environment: &HashMap<String, Vec<String>>,
    functions: &CallableMap,
    symbols: &ScopedSymbolTable<'_>,
    options: ExpressionCheckOptions<'_>,
) -> Vec<Diagnostic> {
    let Expression::BinaryExpression {
        left,
        operator,
        operator_span,
        right,
        ..
    } = expression
    else {
        return Vec::new();
    };
    let mut diagnostics = check_binary_operator(
        *operator,
        *operator_span,
        left,
        right,
        environment,
        symbols,
        options.strict_null_checks,
    )
    .into_iter()
    .collect::<Vec<_>>();
    diagnostics.extend(check_expression_calls(
        left,
        environment,
        functions,
        symbols,
        options,
    ));
    diagnostics.extend(check_expression_calls(
        right,
        environment,
        functions,
        symbols,
        options,
    ));
    diagnostics
}

fn check_call_arguments(
    callee: &Expression,
    arguments: &[Expression],
    call_span: crate::syntax::TextSpan,
    environment: &HashMap<String, Vec<String>>,
    functions: &CallableMap,
    symbols: &ScopedSymbolTable<'_>,
    strict_null_checks: bool,
) -> Vec<Diagnostic> {
    if matches!(callee, Expression::Identifier { name, .. } if name == "super") {
        return check_super_constructor_arguments(
            arguments,
            call_span,
            environment,
            symbols,
            strict_null_checks,
        );
    }
    if let Expression::PropertyAccessExpression { receiver, name, .. } = callee {
        return check_class_method_arguments(
            receiver,
            name,
            arguments,
            call_span,
            environment,
            symbols,
            strict_null_checks,
        );
    }
    let Some(name) = called_function_name(callee) else {
        return Vec::new();
    };
    let Some(function) = functions.get(name) else {
        return Vec::new();
    };
    check_callable_arguments(
        function,
        arguments,
        call_span,
        environment,
        symbols,
        strict_null_checks,
    )
}

fn check_super_constructor_arguments(
    arguments: &[Expression],
    call_span: crate::syntax::TextSpan,
    environment: &HashMap<String, Vec<String>>,
    symbols: &ScopedSymbolTable<'_>,
    strict_null_checks: bool,
) -> Vec<Diagnostic> {
    let Some(base_class) = environment.get("super").and_then(|types| types.first()) else {
        return Vec::new();
    };
    let Some(parameters) = symbols.constructor_parameters(base_class) else {
        return Vec::new();
    };
    let constructor = CallableSignature::from_class_method(&parameters);
    check_callable_arguments(
        &constructor,
        arguments,
        call_span,
        environment,
        symbols,
        strict_null_checks,
    )
}

fn check_class_method_arguments(
    receiver: &Expression,
    method_name: &str,
    arguments: &[Expression],
    call_span: crate::syntax::TextSpan,
    environment: &HashMap<String, Vec<String>>,
    symbols: &ScopedSymbolTable<'_>,
    strict_null_checks: bool,
) -> Vec<Diagnostic> {
    let Some(receiver_types) = infer_expression_types(receiver, environment, symbols) else {
        return Vec::new();
    };
    for receiver_type in receiver_types {
        let Some(method) = symbols
            .properties_for_type(&receiver_type)
            .and_then(|properties| {
                properties
                    .into_iter()
                    .find(|property| property.name == method_name)
            })
        else {
            continue;
        };
        let Some(parameters) = method.method_parameters.as_deref() else {
            continue;
        };
        let method = CallableSignature::from_class_method(parameters);
        return check_callable_arguments(
            &method,
            arguments,
            call_span,
            environment,
            symbols,
            strict_null_checks,
        );
    }
    Vec::new()
}

fn check_constructor_arguments(
    constructor: &Expression,
    arguments: &[Expression],
    call_span: crate::syntax::TextSpan,
    environment: &HashMap<String, Vec<String>>,
    functions: &CallableMap,
    symbols: &ScopedSymbolTable<'_>,
    strict_null_checks: bool,
) -> Vec<Diagnostic> {
    let Expression::Identifier { name, .. } = constructor else {
        return Vec::new();
    };
    let Some(function) = functions.get(&constructor_callable_name(name)) else {
        return Vec::new();
    };
    check_callable_arguments(
        function,
        arguments,
        call_span,
        environment,
        symbols,
        strict_null_checks,
    )
}

fn check_callable_arguments(
    function: &CallableSignature,
    arguments: &[Expression],
    call_span: crate::syntax::TextSpan,
    environment: &HashMap<String, Vec<String>>,
    symbols: &ScopedSymbolTable<'_>,
    strict_null_checks: bool,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    if let Some(diagnostic) = check_function_arity(function, arguments, call_span) {
        diagnostics.push(diagnostic);
    }
    let parameter_types = function.parameter_types(environment, symbols);
    for ((argument, parameter), expected_types) in arguments
        .iter()
        .zip(function.parameters())
        .zip(parameter_types)
    {
        if parameter.type_annotation().is_none() && parameter.initializer().is_none() {
            continue;
        }
        let Some(actual_types) = infer_expression_types(argument, environment, symbols) else {
            continue;
        };
        if expected_types
            .iter()
            .any(|name| !symbols.is_type_name_defined(name))
        {
            continue;
        }
        if !types_are_assignable(&actual_types, &expected_types, symbols, strict_null_checks) {
            diagnostics.push(Diagnostic::new(
                2345,
                format!(
                    "Argument of type '{}' is not assignable to parameter of type '{}'.",
                    actual_types.join(" | "),
                    expected_types.join(" | ")
                ),
                argument.span(),
            ));
        }
    }
    diagnostics
}

fn check_function_arity(
    function: &CallableSignature,
    arguments: &[Expression],
    call_span: crate::syntax::TextSpan,
) -> Option<Diagnostic> {
    let minimum_argument_count = function
        .parameters()
        .iter()
        .filter(|parameter| !parameter.is_optional())
        .count();
    let maximum_argument_count = function.parameters().len();
    let actual_argument_count = arguments.len();
    if (minimum_argument_count..=maximum_argument_count).contains(&actual_argument_count) {
        return None;
    }

    let expected_argument_count = if minimum_argument_count == maximum_argument_count {
        minimum_argument_count.to_string()
    } else {
        format!("{minimum_argument_count}-{maximum_argument_count}")
    };
    let diagnostic_span = arguments
        .get(maximum_argument_count)
        .map_or(call_span, Expression::span);
    Some(Diagnostic::new(
        2554,
        format!("Expected {expected_argument_count} arguments, but got {actual_argument_count}."),
        diagnostic_span,
    ))
}

fn called_function_name(expression: &Expression) -> Option<&str> {
    match expression {
        Expression::Identifier { name, .. } => Some(name),
        Expression::ParenthesizedExpression { expression, .. }
        | Expression::TypeAssertionExpression { expression, .. } => {
            called_function_name(expression)
        }
        Expression::BinaryExpression { .. }
        | Expression::ConditionalExpression { .. }
        | Expression::AssignmentExpression { .. }
        | Expression::CallExpression { .. }
        | Expression::NewExpression { .. }
        | Expression::PropertyAccessExpression { .. }
        | Expression::ElementAccessExpression { .. }
        | Expression::UnaryExpression { .. }
        | Expression::NumberLiteral { .. }
        | Expression::BooleanLiteral { .. }
        | Expression::NullLiteral { .. }
        | Expression::StringLiteral { .. }
        | Expression::ArrowFunction { .. }
        | Expression::ObjectLiteral { .. }
        | Expression::ArrayLiteral { .. } => None,
    }
}

fn check_assignability(
    actual_types: &[String],
    expected_annotation: &TypeReference,
    span: crate::syntax::TextSpan,
    symbols: &ScopedSymbolTable<'_>,
    strict_null_checks: bool,
) -> Option<Diagnostic> {
    let expected_types = symbols.resolve_annotation(expected_annotation);
    if expected_types
        .iter()
        .any(|name| !symbols.is_type_name_defined(name))
    {
        return None;
    }
    if types_are_assignable(actual_types, &expected_types, symbols, strict_null_checks) {
        return None;
    }
    Some(Diagnostic::new(
        2322,
        format!(
            "Type '{}' is not assignable to type '{}'.",
            actual_types.join(" | "),
            expected_types.join(" | ")
        ),
        span,
    ))
}

pub(super) fn check_type_reference(
    type_reference: &TypeReference,
    symbols: &ScopedSymbolTable<'_>,
) -> Vec<Diagnostic> {
    let mut reported_names = HashSet::new();
    type_reference
        .names()
        .iter()
        .filter(|name| !symbols.is_type_name_defined(name) && reported_names.insert(name.as_str()))
        .map(|name| {
            Diagnostic::new(
                2304,
                format!("Cannot find name '{name}'."),
                type_reference.span(),
            )
        })
        .collect()
}

fn types_are_assignable(
    actual_types: &[String],
    expected_types: &[String],
    symbols: &ScopedSymbolTable<'_>,
    strict_null_checks: bool,
) -> bool {
    actual_types.iter().all(|actual_type| {
        expected_types.iter().any(|expected_type| {
            type_is_assignable(actual_type, expected_type, symbols, strict_null_checks)
        })
    })
}

fn type_is_assignable(
    actual_type: &str,
    expected_type: &str,
    symbols: &ScopedSymbolTable<'_>,
    strict_null_checks: bool,
) -> bool {
    type_is_assignable_with_visited(
        actual_type,
        expected_type,
        symbols,
        strict_null_checks,
        &mut HashSet::new(),
    )
}

fn type_is_assignable_with_visited(
    actual_type: &str,
    expected_type: &str,
    symbols: &ScopedSymbolTable<'_>,
    strict_null_checks: bool,
    visited: &mut HashSet<(String, String)>,
) -> bool {
    if let Some((enum_name, member_name)) = actual_type.split_once('.')
        && symbols.is_enum_type(enum_name)
        && (symbols.are_same_enum_type(enum_name, expected_type)
            || symbols.enum_member_underlying_type(enum_name, member_name) == Some(expected_type))
    {
        return true;
    }
    if symbols.is_enum_type(actual_type)
        && symbols.enum_underlying_type(actual_type) == Some(expected_type)
    {
        return true;
    }
    if actual_type == "any"
        || actual_type == "never"
        || expected_type == "any"
        || expected_type == "unknown"
        || (!strict_null_checks && matches!(actual_type, "null" | "undefined"))
        || actual_type == expected_type
    {
        return true;
    }

    match (
        actual_type.strip_suffix("[]"),
        expected_type.strip_suffix("[]"),
    ) {
        (Some(actual_element_type), Some(expected_element_type)) => {
            type_is_assignable_with_visited(
                actual_element_type,
                expected_element_type,
                symbols,
                strict_null_checks,
                visited,
            )
        }
        _ => structurally_assignable(
            actual_type,
            expected_type,
            symbols,
            strict_null_checks,
            visited,
        ),
    }
}

fn structurally_assignable(
    actual_type: &str,
    expected_type: &str,
    symbols: &ScopedSymbolTable<'_>,
    strict_null_checks: bool,
    visited: &mut HashSet<(String, String)>,
) -> bool {
    let pair = (actual_type.to_owned(), expected_type.to_owned());
    if !visited.insert(pair) {
        return true;
    }
    let (Some(actual_properties), Some(expected_properties)) = (
        symbols.interface_properties(actual_type),
        symbols.interface_properties(expected_type),
    ) else {
        return false;
    };

    expected_properties.iter().all(|expected_property| {
        let Some(actual_property) = actual_properties
            .iter()
            .find(|property| property.name == expected_property.name)
        else {
            return expected_property.optional;
        };
        if actual_property.optional && !expected_property.optional {
            return false;
        }
        let actual_types = symbols.resolve_names(&actual_property.type_names);
        let expected_types = symbols.resolve_names(&expected_property.type_names);
        !actual_types.is_empty()
            && !expected_types.is_empty()
            && actual_types.iter().all(|actual_type| {
                expected_types.iter().any(|expected_type| {
                    type_is_assignable_with_visited(
                        actual_type,
                        expected_type,
                        symbols,
                        strict_null_checks,
                        visited,
                    )
                })
            })
    })
}
