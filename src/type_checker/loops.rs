use std::collections::{HashMap, HashSet};

use crate::binder::ScopedSymbolTable;
use crate::syntax::{
    Diagnostic, Expression, ForInitializer, FunctionBodyStatement, FunctionDeclaration,
    TypeReference,
};

use super::callables::CallableMap;

#[derive(Clone, Copy)]
pub(super) enum FunctionBodyOwner<'owner> {
    Program,
    Function(&'owner FunctionDeclaration, Option<&'owner str>),
    Arrow(Option<&'owner TypeReference>),
}

impl<'owner> FunctionBodyOwner<'owner> {
    pub(super) const fn return_type(self) -> Option<&'owner TypeReference> {
        match self {
            Self::Program | Self::Arrow(None) => None,
            Self::Function(function, _) => function.return_type(),
            Self::Arrow(Some(return_type)) => Some(return_type),
        }
    }

    pub(super) const fn is_function(self) -> bool {
        !matches!(self, Self::Program)
    }

    pub(super) const fn constructor_class(self) -> Option<&'owner str> {
        match self {
            Self::Function(_, class_name) => class_name,
            Self::Program | Self::Arrow(_) => None,
        }
    }
}

pub(super) struct FunctionBodyContext<'context, 'symbols> {
    pub(super) owner: FunctionBodyOwner<'context>,
    pub(super) symbols: &'context ScopedSymbolTable<'symbols>,
    pub(super) functions: &'context CallableMap,
    pub(super) strict_null_checks: bool,
}

pub(super) fn check_loop_statement(
    statement: &FunctionBodyStatement,
    environment: &HashMap<String, Vec<String>>,
    constant_bindings: &HashSet<String>,
    context: &FunctionBodyContext<'_, '_>,
) -> Vec<Diagnostic> {
    match statement {
        FunctionBodyStatement::While { condition, body } => check_condition_and_body(
            condition,
            body,
            false,
            environment,
            constant_bindings,
            context,
        ),
        FunctionBodyStatement::DoWhile { body, condition } => check_condition_and_body(
            condition,
            body,
            true,
            environment,
            constant_bindings,
            context,
        ),
        FunctionBodyStatement::For {
            initializer,
            condition,
            incrementor,
            body,
        } => check_for_loop(
            ForLoopParts {
                initializer: initializer.as_ref(),
                condition: condition.as_ref(),
                incrementor: incrementor.as_ref(),
                body,
            },
            environment,
            constant_bindings,
            context,
        ),
        FunctionBodyStatement::ForOf {
            initializer,
            iterable,
            body,
        } => check_for_of_loop(
            ForOfLoopParts {
                initializer,
                iterable,
                body,
            },
            environment,
            constant_bindings,
            context,
        ),
        FunctionBodyStatement::ForIn {
            initializer,
            object,
            body,
        } => check_for_in_loop(
            ForInLoopParts {
                initializer,
                object,
                body,
            },
            environment,
            constant_bindings,
            context,
        ),
        FunctionBodyStatement::VariableDeclaration(_)
        | FunctionBodyStatement::Expression(_)
        | FunctionBodyStatement::Return(_)
        | FunctionBodyStatement::Throw(_)
        | FunctionBodyStatement::If { .. }
        | FunctionBodyStatement::Switch { .. }
        | FunctionBodyStatement::Try { .. }
        | FunctionBodyStatement::Break { .. }
        | FunctionBodyStatement::Continue { .. } => Vec::new(),
    }
}

fn check_condition_and_body(
    condition: &Expression,
    body: &[FunctionBodyStatement],
    condition_follows_body: bool,
    environment: &HashMap<String, Vec<String>>,
    constant_bindings: &HashSet<String>,
    context: &FunctionBodyContext<'_, '_>,
) -> Vec<Diagnostic> {
    let check_condition = || {
        super::check_expression_statement(
            condition,
            environment,
            context.functions,
            context.symbols,
            constant_bindings,
            context.strict_null_checks,
            context.owner.constructor_class(),
        )
    };
    let check_body = || {
        super::check_nested_function_body(
            body,
            context.owner,
            context.symbols,
            environment,
            context.functions,
            constant_bindings,
            context.strict_null_checks,
        )
    };
    if condition_follows_body {
        check_body().into_iter().chain(check_condition()).collect()
    } else {
        check_condition().into_iter().chain(check_body()).collect()
    }
}

#[derive(Clone, Copy)]
struct ForOfLoopParts<'a> {
    initializer: &'a ForInitializer,
    iterable: &'a Expression,
    body: &'a [FunctionBodyStatement],
}

#[derive(Clone, Copy)]
struct ForInLoopParts<'a> {
    initializer: &'a ForInitializer,
    object: &'a Expression,
    body: &'a [FunctionBodyStatement],
}

fn check_for_of_loop(
    parts: ForOfLoopParts<'_>,
    environment: &HashMap<String, Vec<String>>,
    constant_bindings: &HashSet<String>,
    context: &FunctionBodyContext<'_, '_>,
) -> Vec<Diagnostic> {
    let mut loop_environment = environment.clone();
    let mut loop_constants = constant_bindings.clone();
    let mut diagnostics = super::check_expression_statement(
        parts.iterable,
        environment,
        context.functions,
        context.symbols,
        constant_bindings,
        context.strict_null_checks,
        context.owner.constructor_class(),
    );
    let iterable_types =
        super::infer_expression_types(parts.iterable, environment, context.symbols);
    let element_types = iterable_types
        .as_deref()
        .map_or_else(Vec::new, iterable_element_types);
    if let Some(iterable_types) = &iterable_types
        && iterable_types
            .iter()
            .any(|iterable_type| !is_iterable_type(iterable_type))
    {
        diagnostics.push(Diagnostic::new(
            2488,
            format!(
                "Type '{}' must have a '[Symbol.iterator]()' method that returns an iterator.",
                iterable_types.join(" | ")
            ),
            parts.iterable.span(),
        ));
    }
    diagnostics.extend(check_iteration_initializer(
        parts.initializer,
        &element_types,
        &mut loop_environment,
        &mut loop_constants,
        context,
    ));
    diagnostics.extend(super::check_nested_function_body(
        parts.body,
        context.owner,
        context.symbols,
        &loop_environment,
        context.functions,
        &loop_constants,
        context.strict_null_checks,
    ));
    diagnostics
}

fn check_for_in_loop(
    parts: ForInLoopParts<'_>,
    environment: &HashMap<String, Vec<String>>,
    constant_bindings: &HashSet<String>,
    context: &FunctionBodyContext<'_, '_>,
) -> Vec<Diagnostic> {
    let mut loop_environment = environment.clone();
    let mut loop_constants = constant_bindings.clone();
    let mut diagnostics = super::check_expression_statement(
        parts.object,
        environment,
        context.functions,
        context.symbols,
        constant_bindings,
        context.strict_null_checks,
        context.owner.constructor_class(),
    );
    if let Some(object_types) =
        super::infer_expression_types(parts.object, environment, context.symbols)
        && object_types
            .iter()
            .any(|object_type| !is_for_in_object_type(object_type, context.symbols))
    {
        diagnostics.push(Diagnostic::new(
            2407,
            format!(
                "The right-hand side of a 'for...in' statement must be of type 'any', an object type or a type parameter, but here has type '{}'.",
                object_types.join(" | ")
            ),
            parts.object.span(),
        ));
    }
    diagnostics.extend(check_iteration_initializer(
        parts.initializer,
        &["string".to_owned()],
        &mut loop_environment,
        &mut loop_constants,
        context,
    ));
    diagnostics.extend(super::check_nested_function_body(
        parts.body,
        context.owner,
        context.symbols,
        &loop_environment,
        context.functions,
        &loop_constants,
        context.strict_null_checks,
    ));
    diagnostics
}

fn is_for_in_object_type(type_name: &str, symbols: &ScopedSymbolTable<'_>) -> bool {
    type_name == "any"
        || type_name == "object"
        || type_name.ends_with("[]")
        || symbols.interface_properties(type_name).is_some()
}

fn check_iteration_initializer(
    initializer: &ForInitializer,
    iteration_types: &[String],
    environment: &mut HashMap<String, Vec<String>>,
    constant_bindings: &mut HashSet<String>,
    context: &FunctionBodyContext<'_, '_>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    match initializer {
        ForInitializer::VariableDeclarations(declarations) => {
            if let Some(declaration) = declarations.first() {
                if let Some(annotation) = declaration.type_annotation() {
                    diagnostics.extend(super::check_type_reference(annotation, context.symbols));
                }
                let (mut declaration_diagnostics, declared_types) = super::check_variable(
                    declaration,
                    context.symbols,
                    environment,
                    context.functions,
                    constant_bindings,
                    context.strict_null_checks,
                    context.owner.constructor_class(),
                );
                diagnostics.append(&mut declaration_diagnostics);
                if !iteration_types.is_empty()
                    && let Some(annotation) = declaration.type_annotation()
                {
                    if let Some(diagnostic) = super::check_assignability(
                        iteration_types,
                        annotation,
                        declaration.name_span(),
                        context.symbols,
                        context.strict_null_checks,
                    ) {
                        diagnostics.push(diagnostic);
                    }
                }
                if let Some(binding_types) = declared_types
                    .or_else(|| (!iteration_types.is_empty()).then_some(iteration_types.to_vec()))
                {
                    environment.insert(declaration.name().to_owned(), binding_types);
                }
            }
        }
        ForInitializer::Expression(expression) => {
            diagnostics.extend(super::check_expression_statement(
                expression,
                environment,
                context.functions,
                context.symbols,
                constant_bindings,
                context.strict_null_checks,
                context.owner.constructor_class(),
            ));
        }
    }
    diagnostics
}

fn iterable_element_types(iterable_types: &[String]) -> Vec<String> {
    let mut element_types = Vec::new();
    for iterable_type in iterable_types {
        let element_type = iterable_type
            .strip_suffix("[]")
            .or_else(|| (iterable_type == "string").then_some("string"))
            .or_else(|| matches!(iterable_type.as_str(), "any" | "unknown").then_some("any"));
        if let Some(element_type) = element_type
            && !element_types
                .iter()
                .any(|existing| existing == element_type)
        {
            element_types.push(element_type.to_owned());
        }
    }
    element_types
}

fn is_iterable_type(iterable_type: &str) -> bool {
    iterable_type.ends_with("[]") || matches!(iterable_type, "string" | "any" | "unknown")
}

#[derive(Clone, Copy)]
struct ForLoopParts<'a> {
    initializer: Option<&'a ForInitializer>,
    condition: Option<&'a Expression>,
    incrementor: Option<&'a Expression>,
    body: &'a [FunctionBodyStatement],
}

fn check_for_loop(
    parts: ForLoopParts<'_>,
    environment: &HashMap<String, Vec<String>>,
    constant_bindings: &HashSet<String>,
    context: &FunctionBodyContext<'_, '_>,
) -> Vec<Diagnostic> {
    let mut loop_environment = environment.clone();
    let mut loop_constants = constant_bindings.clone();
    let mut diagnostics = Vec::new();
    if let Some(initializer) = parts.initializer {
        match initializer {
            ForInitializer::VariableDeclarations(declarations) => {
                for declaration in declarations {
                    if let Some(annotation) = declaration.type_annotation() {
                        diagnostics
                            .extend(super::check_type_reference(annotation, context.symbols));
                    }
                    let (mut binding_diagnostics, binding_types) = super::check_variable(
                        declaration,
                        context.symbols,
                        &loop_environment,
                        context.functions,
                        &mut loop_constants,
                        context.strict_null_checks,
                        context.owner.constructor_class(),
                    );
                    diagnostics.append(&mut binding_diagnostics);
                    if let Some(binding_types) = binding_types {
                        loop_environment.insert(declaration.name().to_owned(), binding_types);
                    }
                }
            }
            ForInitializer::Expression(expression) => {
                diagnostics.extend(super::check_expression_statement(
                    expression,
                    &loop_environment,
                    context.functions,
                    context.symbols,
                    &loop_constants,
                    context.strict_null_checks,
                    context.owner.constructor_class(),
                ));
            }
        }
    }
    for expression in [parts.condition, parts.incrementor].into_iter().flatten() {
        diagnostics.extend(super::check_expression_statement(
            expression,
            &loop_environment,
            context.functions,
            context.symbols,
            &loop_constants,
            context.strict_null_checks,
            context.owner.constructor_class(),
        ));
    }
    diagnostics.extend(super::check_nested_function_body(
        parts.body,
        context.owner,
        context.symbols,
        &loop_environment,
        context.functions,
        &loop_constants,
        context.strict_null_checks,
    ));
    diagnostics
}
