use std::collections::HashMap;

use crate::binder::ScopedSymbolTable;
use crate::syntax::{
    ClassMember, Expression, FunctionDeclaration, FunctionParameter, Program, TypeReference,
    VariableDeclaration,
};

#[derive(Clone)]
pub(super) struct CallableSignature {
    parameters: Vec<FunctionParameter>,
    return_type: Option<TypeReference>,
    arrow_return_expressions: Option<Vec<Option<Expression>>>,
}

impl CallableSignature {
    fn new(
        parameters: &[FunctionParameter],
        return_type: Option<&TypeReference>,
        arrow_return_expressions: Option<Vec<Option<Expression>>>,
    ) -> Self {
        Self {
            parameters: parameters.to_vec(),
            return_type: return_type.cloned(),
            arrow_return_expressions,
        }
    }

    pub(super) fn parameters(&self) -> &[FunctionParameter] {
        &self.parameters
    }

    pub(super) fn return_type(&self) -> Option<&TypeReference> {
        self.return_type.as_ref()
    }

    pub(super) fn arrow_return_expressions(&self) -> Option<&[Option<Expression>]> {
        self.arrow_return_expressions.as_deref()
    }

    pub(super) fn parameter_types(
        &self,
        environment: &HashMap<String, Vec<String>>,
        symbols: &ScopedSymbolTable<'_>,
    ) -> Vec<Vec<String>> {
        infer_parameter_types(&self.parameters, environment, symbols)
    }

    fn from_function(function: &FunctionDeclaration) -> Self {
        Self::new(function.parameters(), function.return_type(), None)
    }

    pub(super) fn from_class_method(parameters: &[FunctionParameter]) -> Self {
        Self::new(parameters, None, None)
    }

    pub(super) fn from_arrow(variable: &VariableDeclaration) -> Option<Self> {
        let Some(Expression::ArrowFunction {
            parameters,
            return_type,
            body,
            ..
        }) = variable.initializer()
        else {
            return None;
        };
        let arrow_return_expressions = Some(
            body.return_expressions()
                .into_iter()
                .map(Option::<&Expression>::cloned)
                .collect(),
        );
        Some(Self::new(
            parameters,
            return_type.as_ref(),
            arrow_return_expressions,
        ))
    }
}

pub(super) type CallableMap = HashMap<String, CallableSignature>;

pub(super) fn declared_callables(program: &Program) -> CallableMap {
    let mut callables = HashMap::new();
    for statement in program.statements() {
        if let Some(function) = statement.as_function_declaration() {
            callables.insert(
                function.name().to_owned(),
                CallableSignature::from_function(function),
            );
        } else if let Some(variable) = statement.as_variable_declaration()
            && let Some(signature) = CallableSignature::from_arrow(variable)
        {
            callables.insert(variable.name().to_owned(), signature);
        }
        if let Some(class) = statement.as_class_declaration() {
            let constructor = class.members().iter().find_map(|member| match member {
                ClassMember::Method(method) if method.name() == "constructor" => Some(method),
                ClassMember::Method(_) | ClassMember::Property(_) => None,
            });
            let signature = constructor.map_or_else(
                || CallableSignature::new(&[], None, None),
                CallableSignature::from_function,
            );
            callables.insert(constructor_callable_name(class.name()), signature);
        }
    }
    callables
}

pub(super) fn constructor_callable_name(class_name: &str) -> String {
    format!("<constructor:{class_name}>")
}

pub(super) fn callable_return_types(
    signature: &CallableSignature,
    symbols: &ScopedSymbolTable<'_>,
) -> Vec<String> {
    if let Some(return_type) = signature.return_type() {
        return symbols.resolve_annotation(return_type);
    }
    let Some(return_expressions) = signature.arrow_return_expressions() else {
        return vec!["any".to_owned()];
    };
    if return_expressions.is_empty() {
        return vec!["void".to_owned()];
    }
    let parameter_environment = signature
        .parameters()
        .iter()
        .zip(signature.parameter_types(&HashMap::new(), symbols))
        .map(|(parameter, parameter_types)| (parameter.name().to_owned(), parameter_types))
        .collect::<HashMap<_, _>>();
    let mut return_types = Vec::new();
    for expression in return_expressions {
        let expression_types = expression.as_ref().map_or_else(
            || vec!["undefined".to_owned()],
            |expression| {
                super::infer_expression_types(expression, &parameter_environment, symbols)
                    .unwrap_or_else(|| vec!["any".to_owned()])
            },
        );
        if expression_types.iter().any(|type_name| type_name == "any") {
            return vec!["any".to_owned()];
        }
        for type_name in expression_types {
            if !return_types.contains(&type_name) {
                return_types.push(type_name);
            }
        }
    }
    if return_types.is_empty() {
        vec!["void".to_owned()]
    } else {
        return_types
    }
}

fn infer_parameter_types(
    parameters: &[FunctionParameter],
    environment: &HashMap<String, Vec<String>>,
    symbols: &ScopedSymbolTable<'_>,
) -> Vec<Vec<String>> {
    let mut parameter_environment = environment.clone();
    let mut parameter_types = Vec::with_capacity(parameters.len());
    for parameter in parameters {
        let types = if let Some(annotation) = parameter.type_annotation() {
            symbols.resolve_annotation(annotation)
        } else if let Some(initializer) = parameter.initializer() {
            super::infer_expression_types(initializer, &parameter_environment, symbols)
                .unwrap_or_else(|| vec!["any".to_owned()])
        } else {
            vec!["any".to_owned()]
        };
        parameter_environment.insert(parameter.name().to_owned(), types.clone());
        parameter_types.push(types);
    }
    parameter_types
}
