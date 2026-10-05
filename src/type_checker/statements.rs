use std::collections::{HashMap, HashSet};

use crate::binder::ScopedSymbolTable;
use crate::syntax::{
    ClassDeclaration, ClassMember, Diagnostic, ExportSpecifier, FunctionBodyStatement,
    FunctionDeclaration, InterfaceDeclaration, PropertyDeclaration,
};

use super::callables::CallableMap;
use super::loops::{FunctionBodyContext, FunctionBodyOwner, check_loop_statement};

pub(super) fn check_function_declaration(
    function: &FunctionDeclaration,
    symbols: &ScopedSymbolTable<'_>,
    environment: &HashMap<String, Vec<String>>,
    functions: &CallableMap,
    constant_bindings: &HashSet<String>,
    strict_null_checks: bool,
    constructor_class: Option<&str>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut parameter_environment = environment.clone();
    for parameter in function.parameters() {
        if let Some(annotation) = parameter.type_annotation() {
            diagnostics.extend(super::check_type_reference(annotation, symbols));
            if let Some(initializer) = parameter.initializer() {
                diagnostics.extend(super::check_expression_statement(
                    initializer,
                    &parameter_environment,
                    functions,
                    symbols,
                    constant_bindings,
                    strict_null_checks,
                    None,
                ));
                diagnostics.extend(super::check_expression_assignability(
                    initializer,
                    annotation,
                    initializer.span(),
                    symbols,
                    &parameter_environment,
                    strict_null_checks,
                ));
            }
            parameter_environment.insert(
                parameter.name().to_owned(),
                symbols.resolve_annotation(annotation),
            );
        } else if let Some(initializer) = parameter.initializer() {
            diagnostics.extend(super::check_expression_statement(
                initializer,
                &parameter_environment,
                functions,
                symbols,
                constant_bindings,
                strict_null_checks,
                None,
            ));
            let parameter_types =
                super::infer_expression_types(initializer, &parameter_environment, symbols)
                    .unwrap_or_else(|| vec!["any".to_owned()]);
            parameter_environment.insert(parameter.name().to_owned(), parameter_types);
        }
    }
    if let Some(annotation) = function.return_type() {
        diagnostics.extend(super::check_type_reference(annotation, symbols));
    }
    diagnostics.extend(super::check_function(
        function,
        symbols,
        environment,
        functions,
        constant_bindings,
        strict_null_checks,
        constructor_class,
    ));
    diagnostics
}

pub(super) fn check_top_level_control_flow(
    control_flow: &FunctionBodyStatement,
    symbols: &ScopedSymbolTable<'_>,
    environment: &HashMap<String, Vec<String>>,
    functions: &CallableMap,
    constant_bindings: &HashSet<String>,
    strict_null_checks: bool,
) -> Vec<Diagnostic> {
    super::check_nested_function_body(
        std::slice::from_ref(control_flow),
        FunctionBodyOwner::Program,
        symbols,
        environment,
        functions,
        constant_bindings,
        strict_null_checks,
    )
}

pub(super) fn check_control_flow_statement(
    statement: &FunctionBodyStatement,
    owner: FunctionBodyOwner<'_>,
    symbols: &ScopedSymbolTable<'_>,
    environment: &HashMap<String, Vec<String>>,
    functions: &CallableMap,
    constant_bindings: &HashSet<String>,
    strict_null_checks: bool,
) -> Vec<Diagnostic> {
    let context = FunctionBodyContext {
        owner,
        symbols,
        functions,
        strict_null_checks,
    };
    match statement {
        FunctionBodyStatement::If { .. } => {
            check_if_statement(statement, environment, constant_bindings, &context)
        }
        FunctionBodyStatement::Switch { .. } => {
            check_switch_statement(statement, environment, constant_bindings, &context)
        }
        FunctionBodyStatement::Try { .. } => {
            check_try_statement(statement, environment, constant_bindings, &context)
        }
        FunctionBodyStatement::While { .. }
        | FunctionBodyStatement::DoWhile { .. }
        | FunctionBodyStatement::For { .. }
        | FunctionBodyStatement::ForOf { .. }
        | FunctionBodyStatement::ForIn { .. } => {
            check_loop_statement(statement, environment, constant_bindings, &context)
        }
        FunctionBodyStatement::VariableDeclaration(_)
        | FunctionBodyStatement::Expression(_)
        | FunctionBodyStatement::Return(_)
        | FunctionBodyStatement::Throw(_)
        | FunctionBodyStatement::Break { .. }
        | FunctionBodyStatement::Continue { .. } => Vec::new(),
    }
}

pub(super) fn check_interface_declaration(
    declaration: &InterfaceDeclaration,
    symbols: &ScopedSymbolTable<'_>,
) -> Vec<Diagnostic> {
    declaration
        .members()
        .iter()
        .flat_map(|member| super::check_type_reference(member.type_annotation(), symbols))
        .collect()
}

pub(super) fn check_class_declaration(
    declaration: &ClassDeclaration,
    symbols: &ScopedSymbolTable<'_>,
    environment: &HashMap<String, Vec<String>>,
    functions: &CallableMap,
    constant_bindings: &HashSet<String>,
    strict_null_checks: bool,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut class_environment = environment.clone();
    if let Some(base_class) = declaration.base_class() {
        class_environment.insert("super".to_owned(), vec![base_class.to_owned()]);
    }
    diagnostics.extend(check_class_property_overrides(
        declaration,
        symbols,
        strict_null_checks,
    ));
    if let (Some(base_class), Some(span)) =
        (declaration.base_class(), declaration.base_class_span())
        && !environment.contains_key(base_class)
    {
        diagnostics.push(Diagnostic::new(
            2304,
            format!("Cannot find name '{base_class}'."),
            span,
        ));
    }
    for member in declaration.members() {
        let mut member_environment = class_environment.clone();
        member_environment.insert(
            "this".to_owned(),
            vec![if member.is_static() {
                format!("typeof {}", declaration.name())
            } else {
                declaration.name().to_owned()
            }],
        );
        if member.is_static()
            && let Some(base_class) = declaration.base_class()
        {
            member_environment.insert("super".to_owned(), vec![format!("typeof {base_class}")]);
        }
        match member {
            ClassMember::Method(method) => diagnostics.extend(check_function_declaration(
                method,
                symbols,
                &member_environment,
                functions,
                constant_bindings,
                strict_null_checks,
                (method.name() == "constructor" && !method.is_static())
                    .then_some(declaration.name()),
            )),
            ClassMember::Property(property) => diagnostics.extend(check_class_property(
                property,
                symbols,
                &member_environment,
                functions,
                constant_bindings,
                strict_null_checks,
            )),
        }
    }
    diagnostics
}

fn check_class_property_overrides(
    declaration: &ClassDeclaration,
    symbols: &ScopedSymbolTable<'_>,
    strict_null_checks: bool,
) -> Vec<Diagnostic> {
    let Some(base_class) = declaration.base_class() else {
        return Vec::new();
    };
    let Some(base_properties) = symbols.interface_properties(base_class) else {
        return Vec::new();
    };
    let Some(class_properties) = symbols.interface_properties(declaration.name()) else {
        return Vec::new();
    };

    declaration
        .members()
        .iter()
        .filter_map(ClassMember::as_property)
        .filter_map(|property| {
            let base_property = base_properties
                .iter()
                .find(|base_property| base_property.name == property.name())?;
            if base_property.method_parameters.is_some() {
                return None;
            }
            let class_property = class_properties
                .iter()
                .find(|class_property| class_property.name == property.name())?;
            let expected_types = symbols.resolve_names(&base_property.type_names);
            let actual_types = symbols.resolve_names(&class_property.type_names);
            if super::types_are_assignable(
                &actual_types,
                &expected_types,
                symbols,
                strict_null_checks,
            ) {
                return None;
            }
            Some(Diagnostic::new(
                2416,
                format!(
                    "Property '{}' in type '{}' is not assignable to the same property in base type '{}'.\n  Type '{}' is not assignable to type '{}'.",
                    property.name(),
                    declaration.name(),
                    base_class,
                    actual_types.join(" | "),
                    expected_types.join(" | ")
                ),
                property.name_span(),
            ))
        })
        .collect()
}

fn check_class_property(
    property: &PropertyDeclaration,
    symbols: &ScopedSymbolTable<'_>,
    environment: &HashMap<String, Vec<String>>,
    functions: &CallableMap,
    constant_bindings: &HashSet<String>,
    strict_null_checks: bool,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    if let Some(annotation) = property.type_annotation() {
        diagnostics.extend(super::check_type_reference(annotation, symbols));
    }
    if let Some(initializer) = property.initializer() {
        diagnostics.extend(super::check_expression_statement(
            initializer,
            environment,
            functions,
            symbols,
            constant_bindings,
            strict_null_checks,
            None,
        ));
        if let Some(annotation) = property.type_annotation() {
            diagnostics.extend(super::check_expression_assignability(
                initializer,
                annotation,
                initializer.span(),
                symbols,
                environment,
                strict_null_checks,
            ));
        }
    }
    diagnostics
}

pub(super) fn check_exported_values(
    specifiers: &[ExportSpecifier],
    declared_names: &HashSet<String>,
) -> Vec<Diagnostic> {
    specifiers
        .iter()
        .filter(|specifier| !declared_names.contains(specifier.local_name()))
        .map(|specifier| {
            Diagnostic::new(
                2304,
                format!("Cannot find name '{}'.", specifier.local_name()),
                specifier.span(),
            )
        })
        .collect()
}

pub(super) fn check_exported_types(
    specifiers: &[ExportSpecifier],
    symbols: &ScopedSymbolTable<'_>,
) -> Vec<Diagnostic> {
    specifiers
        .iter()
        .filter(|specifier| !symbols.is_type_name_defined(specifier.local_name()))
        .map(|specifier| {
            Diagnostic::new(
                2304,
                format!("Cannot find name '{}'.", specifier.local_name()),
                specifier.span(),
            )
        })
        .collect()
}

pub(super) fn check_if_statement(
    statement: &FunctionBodyStatement,
    environment: &HashMap<String, Vec<String>>,
    constant_bindings: &HashSet<String>,
    context: &FunctionBodyContext<'_, '_>,
) -> Vec<Diagnostic> {
    let FunctionBodyStatement::If {
        condition,
        then_body,
        else_body,
    } = statement
    else {
        return Vec::new();
    };
    let mut diagnostics = super::check_expression_statement(
        condition,
        environment,
        context.functions,
        context.symbols,
        constant_bindings,
        context.strict_null_checks,
        context.owner.constructor_class(),
    );
    diagnostics.extend(super::check_nested_function_body(
        then_body,
        context.owner,
        context.symbols,
        environment,
        context.functions,
        constant_bindings,
        context.strict_null_checks,
    ));
    if let Some(else_body) = else_body {
        diagnostics.extend(super::check_nested_function_body(
            else_body,
            context.owner,
            context.symbols,
            environment,
            context.functions,
            constant_bindings,
            context.strict_null_checks,
        ));
    }
    diagnostics
}

pub(super) fn check_switch_statement(
    statement: &FunctionBodyStatement,
    environment: &HashMap<String, Vec<String>>,
    constant_bindings: &HashSet<String>,
    context: &FunctionBodyContext<'_, '_>,
) -> Vec<Diagnostic> {
    let FunctionBodyStatement::Switch {
        expression,
        clauses,
    } = statement
    else {
        return Vec::new();
    };
    let mut diagnostics = super::check_expression_statement(
        expression,
        environment,
        context.functions,
        context.symbols,
        constant_bindings,
        context.strict_null_checks,
        context.owner.constructor_class(),
    );
    for clause in clauses {
        if let Some(case_expression) = clause.expression() {
            diagnostics.extend(super::check_expression_statement(
                case_expression,
                environment,
                context.functions,
                context.symbols,
                constant_bindings,
                context.strict_null_checks,
                context.owner.constructor_class(),
            ));
        }
        diagnostics.extend(super::check_nested_function_body(
            clause.statements(),
            context.owner,
            context.symbols,
            environment,
            context.functions,
            constant_bindings,
            context.strict_null_checks,
        ));
    }
    diagnostics
}

pub(super) fn check_try_statement(
    statement: &FunctionBodyStatement,
    environment: &HashMap<String, Vec<String>>,
    constant_bindings: &HashSet<String>,
    context: &FunctionBodyContext<'_, '_>,
) -> Vec<Diagnostic> {
    let FunctionBodyStatement::Try {
        try_body,
        catch_clause,
        finally_body,
    } = statement
    else {
        return Vec::new();
    };
    let mut diagnostics = super::check_nested_function_body(
        try_body,
        context.owner,
        context.symbols,
        environment,
        context.functions,
        constant_bindings,
        context.strict_null_checks,
    );
    if let Some(catch_clause) = catch_clause {
        let mut catch_environment = environment.clone();
        let mut catch_constants = constant_bindings.clone();
        if let Some(variable) = catch_clause.variable() {
            if let Some(annotation) = variable.type_annotation() {
                let names = annotation.names();
                if names.len() != 1
                    || !matches!(names.first().map(String::as_str), Some("any" | "unknown"))
                {
                    diagnostics.push(Diagnostic::new(
                        1196,
                        "Catch clause variable type annotation must be 'any' or 'unknown' if specified.",
                        annotation.span(),
                    ));
                }
                diagnostics.extend(super::check_type_reference(annotation, context.symbols));
            }
            let (mut variable_diagnostics, binding_types) = super::check_variable(
                variable,
                context.symbols,
                &catch_environment,
                context.functions,
                &mut catch_constants,
                context.strict_null_checks,
                context.owner.constructor_class(),
            );
            diagnostics.append(&mut variable_diagnostics);
            catch_environment.insert(
                variable.name().to_owned(),
                binding_types.unwrap_or_else(|| vec!["any".to_owned()]),
            );
        }
        diagnostics.extend(super::check_nested_function_body(
            catch_clause.body(),
            context.owner,
            context.symbols,
            &catch_environment,
            context.functions,
            &catch_constants,
            context.strict_null_checks,
        ));
    }
    if let Some(finally_body) = finally_body {
        diagnostics.extend(super::check_nested_function_body(
            finally_body,
            context.owner,
            context.symbols,
            environment,
            context.functions,
            constant_bindings,
            context.strict_null_checks,
        ));
    }
    diagnostics
}
