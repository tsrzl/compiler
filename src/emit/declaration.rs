//! TypeScript declaration output for the syntax forms currently supported.

use std::collections::HashMap;

use crate::enum_values::{EnumValue, format_enum_number, values_for_enum};
use crate::syntax::{
    ArrowFunctionBody, BinaryOperator, ClassDeclaration, ClassMember, EnumDeclaration,
    ExportSpecifier, Expression, FunctionDeclaration, FunctionParameter, Program,
    PropertyDeclaration, Statement, TypeReference, VariableDeclaration, VariableDeclarationKind,
};

pub(crate) fn emit(program: &Program) -> String {
    let external_module = program
        .statements()
        .iter()
        .any(Statement::is_external_module_indicator);
    let mut output = String::new();
    for statement in program.statements() {
        emit_statement(
            statement.declaration(),
            statement.is_exported(),
            external_module,
            &mut output,
        );
    }
    output
}

fn emit_statement(
    statement: &Statement,
    exported: bool,
    external_module: bool,
    output: &mut String,
) {
    match statement {
        Statement::VariableDeclaration(declaration) if exported || !external_module => {
            emit_variable(declaration, exported, output);
        }
        Statement::FunctionDeclaration(function) if exported || !external_module => {
            emit_function(function, exported, output);
        }
        Statement::ClassDeclaration(declaration) if exported || !external_module => {
            emit_class(declaration, exported, output);
        }
        Statement::EnumDeclaration(declaration) if exported || !external_module => {
            emit_enum(declaration, exported, output);
        }
        Statement::InterfaceDeclaration(declaration) if exported || !external_module => {
            if exported {
                output.push_str("export ");
            }
            output.push_str("interface ");
            output.push_str(declaration.name());
            output.push_str(" {\n");
            for member in declaration.members() {
                output.push_str("    ");
                output.push_str(member.name());
                if member.is_optional() {
                    output.push('?');
                }
                output.push_str(": ");
                emit_type(member.type_annotation(), output);
                output.push_str(";\n");
            }
            output.push_str("}\n");
        }
        Statement::TypeAliasDeclaration(declaration) if exported || !external_module => {
            if exported {
                output.push_str("export ");
            }
            output.push_str("type ");
            output.push_str(declaration.name());
            output.push_str(" = ");
            emit_type(declaration.type_annotation(), output);
            output.push_str(";\n");
        }
        Statement::ExportDefault(expression) => {
            output.push_str("declare const _default");
            if let Some(literal) = literal_initializer(expression) {
                output.push_str(" = ");
                output.push_str(&literal);
            } else {
                output.push_str(": ");
                output.push_str(&infer_expression_type(expression));
            }
            output.push_str(";\nexport default _default;\n");
        }
        Statement::ExportNamed(specifiers) => emit_named_exports(specifiers, false, None, output),
        Statement::ExportAll(export) => {
            output.push_str("export * from ");
            output.push_str(export.raw_module_specifier());
            output.push_str(";\n");
        }
        Statement::ExportTypeNamed(specifiers) => {
            emit_named_exports(specifiers, true, None, output);
        }
        Statement::ExportNamedFrom(export) => {
            emit_named_exports(
                export.specifiers(),
                export.is_type_only(),
                Some(export.raw_module_specifier()),
                output,
            );
        }
        Statement::VariableDeclaration(_)
        | Statement::EnumDeclaration(_)
        | Statement::NamespaceDeclaration(_)
        | Statement::InterfaceDeclaration(_)
        | Statement::TypeAliasDeclaration(_)
        | Statement::FunctionDeclaration(_)
        | Statement::ClassDeclaration(_)
        | Statement::ExpressionStatement(_)
        | Statement::ControlFlowStatement(_)
        | Statement::Break { .. }
        | Statement::Continue { .. }
        | Statement::ImportDeclaration(_)
        | Statement::ExportedDeclaration(_) => {}
    }
}

fn emit_enum(declaration: &EnumDeclaration, exported: bool, output: &mut String) {
    if exported {
        output.push_str("export ");
    }
    output.push_str("declare enum ");
    output.push_str(declaration.name());
    output.push_str(" {\n");
    for (index, (member, value)) in declaration
        .members()
        .iter()
        .zip(values_for_enum(declaration))
        .enumerate()
    {
        emit_enum_member(member.name(), value, output);
        if index + 1 < declaration.members().len() {
            output.push_str(",\n");
        } else {
            output.push('\n');
        }
    }
    output.push_str("}\n");
}

fn emit_enum_member(name: &str, value: EnumValue, output: &mut String) {
    output.push_str("    ");
    output.push_str(name);
    match value {
        EnumValue::Number(value) => {
            output.push_str(" = ");
            output.push_str(&format_enum_number(value));
        }
        EnumValue::String(value) => {
            output.push_str(" = ");
            output.push_str(&value);
        }
        EnumValue::Computed => {}
    }
}

fn emit_named_exports(
    specifiers: &[ExportSpecifier],
    type_only: bool,
    module_specifier: Option<&str>,
    output: &mut String,
) {
    output.push_str(if type_only {
        "export type { "
    } else {
        "export { "
    });
    for (index, specifier) in specifiers.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push_str(specifier.local_name());
        if specifier.local_name() != specifier.exported_name() {
            output.push_str(" as ");
            output.push_str(specifier.exported_name());
        }
    }
    output.push_str(" }");
    if let Some(module_specifier) = module_specifier {
        output.push_str(" from ");
        output.push_str(module_specifier);
    }
    output.push_str(";\n");
}

fn emit_variable(declaration: &VariableDeclaration, exported: bool, output: &mut String) {
    if exported {
        output.push_str("export ");
    }
    output.push_str("declare ");
    output.push_str(match declaration.declaration_kind() {
        VariableDeclarationKind::Const => "const ",
        VariableDeclarationKind::Let => "let ",
        VariableDeclarationKind::Var => "var ",
    });
    output.push_str(declaration.name());
    if let Some(annotation) = declaration.type_annotation() {
        output.push_str(": ");
        emit_type(annotation, output);
    } else if declaration.declaration_kind() == VariableDeclarationKind::Const
        && let Some(initializer) = declaration.initializer()
        && let Some(literal) = literal_initializer(initializer)
    {
        output.push_str(" = ");
        output.push_str(&literal);
    } else if let Some(initializer) = declaration.initializer() {
        output.push_str(": ");
        output.push_str(&infer_expression_type(initializer));
    } else {
        output.push_str(": ");
        output.push_str("any");
    }
    output.push_str(";\n");
}

fn emit_function(function: &FunctionDeclaration, exported: bool, output: &mut String) {
    let parameter_types = infer_function_parameter_types(function.parameters());
    if exported {
        output.push_str("export ");
    }
    output.push_str("declare function ");
    output.push_str(function.name());
    emit_function_parameters(function.parameters(), &parameter_types, output);
    output.push_str(": ");
    output.push_str(&infer_function_return_type(function, &parameter_types));
    output.push_str(";\n");
}

fn emit_class(declaration: &ClassDeclaration, exported: bool, output: &mut String) {
    if exported {
        output.push_str("export ");
    }
    output.push_str("declare class ");
    output.push_str(declaration.name());
    if let Some(base_class) = declaration.base_class() {
        output.push_str(" extends ");
        output.push_str(base_class);
    }
    output.push_str(" {\n");
    for member in declaration.members() {
        match member {
            ClassMember::Method(method) => {
                let is_private_method = method.is_private() && method.name() != "constructor";
                if method.name() == "constructor" {
                    for parameter in method
                        .parameters()
                        .iter()
                        .filter(|parameter| parameter.is_parameter_property())
                    {
                        emit_parameter_property(parameter, output);
                    }
                }
                output.push_str("    ");
                if is_private_method {
                    output.push_str("private ");
                } else if method.is_protected() {
                    output.push_str("protected ");
                }
                if method.is_static() {
                    output.push_str("static ");
                }
                output.push_str(method.name());
                if is_private_method {
                    output.push_str(";\n");
                    continue;
                }
                let parameter_types = infer_function_parameter_types(method.parameters());
                emit_function_parameters(method.parameters(), &parameter_types, output);
                if method.name() != "constructor" {
                    output.push_str(": ");
                    output.push_str(&infer_function_return_type(method, &parameter_types));
                }
                output.push_str(";\n");
            }
            ClassMember::Property(property) => emit_class_property(property, output),
        }
    }
    output.push_str("}\n");
}

fn emit_parameter_property(parameter: &FunctionParameter, output: &mut String) {
    output.push_str("    ");
    if parameter.is_private_parameter_property() {
        output.push_str("private ");
    } else if parameter.is_protected_parameter_property() {
        output.push_str("protected ");
    }
    if parameter.is_readonly_parameter_property() {
        output.push_str("readonly ");
    }
    output.push_str(parameter.name());
    if parameter.is_private_parameter_property() {
        output.push_str(";\n");
        return;
    }
    if parameter.is_optional() && parameter.initializer().is_none() {
        output.push('?');
    }
    output.push_str(": ");
    if let Some(annotation) = parameter.type_annotation() {
        emit_type(annotation, output);
    } else if let Some(initializer) = parameter.initializer() {
        output.push_str(&infer_expression_type(initializer));
    } else {
        output.push_str("any");
    }
    output.push_str(";\n");
}

fn emit_class_property(property: &PropertyDeclaration, output: &mut String) {
    output.push_str("    ");
    if property.is_private() {
        output.push_str("private ");
    } else if property.is_protected() {
        output.push_str("protected ");
    }
    if property.is_static() {
        output.push_str("static ");
    }
    if property.is_readonly() {
        output.push_str("readonly ");
    }
    output.push_str(property.name());
    if property.is_private() {
        output.push_str(";\n");
        return;
    }
    output.push_str(": ");
    if let Some(annotation) = property.type_annotation() {
        emit_type(annotation, output);
    } else if let Some(initializer) = property.initializer() {
        output.push_str(&infer_expression_type(initializer));
    } else {
        output.push_str("any");
    }
    output.push_str(";\n");
}

fn emit_function_parameters(
    parameters: &[FunctionParameter],
    parameter_types: &HashMap<&str, String>,
    output: &mut String,
) {
    output.push('(');
    for (index, parameter) in parameters.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push_str(parameter.name());
        if parameter.is_optional() {
            output.push('?');
        }
        output.push_str(": ");
        output.push_str(&infer_parameter_type(parameter, parameter_types));
    }
    output.push(')');
}

fn infer_function_return_type(
    function: &FunctionDeclaration,
    parameter_types: &HashMap<&str, String>,
) -> String {
    if let Some(return_type) = function.return_type() {
        return_type.type_spellings().collect::<Vec<_>>().join(" | ")
    } else if let Some(expression) = function
        .body()
        .iter()
        .find_map(|statement| statement.expression())
    {
        infer_expression_type_with_parameters(expression, parameter_types)
    } else {
        "any".to_owned()
    }
}

fn infer_function_parameter_types(parameters: &[FunctionParameter]) -> HashMap<&str, String> {
    let mut parameter_types = HashMap::new();
    for parameter in parameters {
        let parameter_type = infer_parameter_type(parameter, &parameter_types);
        parameter_types.insert(parameter.name(), parameter_type);
    }
    parameter_types
}

fn infer_parameter_type(
    parameter: &FunctionParameter,
    parameter_types: &HashMap<&str, String>,
) -> String {
    parameter.type_annotation().map_or_else(
        || {
            parameter.initializer().map_or_else(
                || "any".to_owned(),
                |initializer| infer_expression_type_with_parameters(initializer, parameter_types),
            )
        },
        |annotation| annotation.type_spellings().collect::<Vec<_>>().join(" | "),
    )
}

fn emit_type(type_reference: &TypeReference, output: &mut String) {
    for (index, type_spelling) in type_reference.type_spellings().enumerate() {
        if index > 0 {
            output.push_str(" | ");
        }
        output.push_str(&type_spelling);
    }
}

fn literal_initializer(expression: &Expression) -> Option<String> {
    match expression {
        Expression::NumberLiteral { value, .. } | Expression::StringLiteral { raw: value, .. } => {
            Some(value.clone())
        }
        Expression::BooleanLiteral { value, .. } => Some(value.to_string()),
        Expression::NullLiteral { .. } => Some("null".to_owned()),
        Expression::ParenthesizedExpression { expression, .. } => literal_initializer(expression),
        Expression::TypeAssertionExpression { .. }
        | Expression::Identifier { .. }
        | Expression::BinaryExpression { .. }
        | Expression::ConditionalExpression { .. }
        | Expression::AssignmentExpression { .. }
        | Expression::CallExpression { .. }
        | Expression::NewExpression { .. }
        | Expression::ArrowFunction { .. }
        | Expression::PropertyAccessExpression { .. }
        | Expression::ElementAccessExpression { .. }
        | Expression::UnaryExpression { .. }
        | Expression::ObjectLiteral { .. }
        | Expression::ArrayLiteral { .. } => None,
    }
}

fn infer_expression_type(expression: &Expression) -> String {
    infer_expression_type_with_parameters(expression, &HashMap::new())
}

fn infer_expression_type_with_parameters(
    expression: &Expression,
    parameter_types: &HashMap<&str, String>,
) -> String {
    match expression {
        Expression::UnaryExpression {
            operator: crate::syntax::UnaryOperator::LogicalNot,
            ..
        }
        | Expression::BooleanLiteral { .. } => "boolean".to_owned(),
        Expression::NullLiteral { .. } => "null".to_owned(),
        Expression::StringLiteral { .. } => "string".to_owned(),
        Expression::TypeAssertionExpression {
            type_annotation, ..
        } => type_annotation
            .type_spellings()
            .collect::<Vec<_>>()
            .join(" | "),
        Expression::BinaryExpression {
            left,
            operator,
            right,
            ..
        } => infer_binary_expression_type(left, *operator, right, parameter_types),
        Expression::ConditionalExpression {
            when_true,
            when_false,
            ..
        } => {
            let true_type = infer_expression_type_with_parameters(when_true, parameter_types);
            let false_type = infer_expression_type_with_parameters(when_false, parameter_types);
            if true_type == "any" || false_type == "any" {
                "any".to_owned()
            } else if true_type == false_type {
                true_type
            } else {
                format!("{true_type} | {false_type}")
            }
        }
        Expression::NumberLiteral { value, .. } => {
            crate::type_system::numeric_literal_type(value).to_owned()
        }
        Expression::UnaryExpression {
            operator: crate::syntax::UnaryOperator::Plus | crate::syntax::UnaryOperator::Negate,
            ..
        } => "number".to_owned(),
        Expression::ParenthesizedExpression { expression, .. } => {
            infer_expression_type_with_parameters(expression, parameter_types)
        }
        Expression::ArrayLiteral { elements, .. } => crate::type_system::array_type(
            elements
                .iter()
                .map(|element| infer_expression_type_with_parameters(element, parameter_types)),
        ),
        Expression::AssignmentExpression {
            right,
            operator: crate::syntax::AssignmentOperator::Assign,
            ..
        } => infer_expression_type_with_parameters(right, parameter_types),
        Expression::AssignmentExpression {
            left,
            right,
            operator: crate::syntax::AssignmentOperator::AddAssign,
            ..
        } => {
            let left_type = infer_expression_type_with_parameters(left, parameter_types);
            let right_type = infer_expression_type_with_parameters(right, parameter_types);
            if left_type == "string" || right_type == "string" {
                "string".to_owned()
            } else if left_type == "any" || right_type == "any" {
                "any".to_owned()
            } else {
                "number".to_owned()
            }
        }
        Expression::Identifier { name, .. } => parameter_types
            .get(name.as_str())
            .cloned()
            .unwrap_or_else(|| "any".to_owned()),
        Expression::NewExpression { constructor, .. } => match constructor.as_ref() {
            Expression::Identifier { name, .. } => name.clone(),
            _ => "any".to_owned(),
        },
        Expression::ArrowFunction {
            parameters,
            return_type,
            body,
            ..
        } => infer_arrow_function_type(parameters, return_type.as_ref(), body),
        Expression::CallExpression { .. }
        | Expression::PropertyAccessExpression { .. }
        | Expression::ElementAccessExpression { .. }
        | Expression::ObjectLiteral { .. } => "any".to_owned(),
    }
}

fn infer_arrow_function_type(
    parameters: &[FunctionParameter],
    return_type: Option<&TypeReference>,
    body: &ArrowFunctionBody,
) -> String {
    let mut parameter_types = HashMap::new();
    let signature_parameters = parameters
        .iter()
        .map(|parameter| {
            let parameter_type = infer_parameter_type(parameter, &parameter_types);
            parameter_types.insert(parameter.name(), parameter_type.clone());
            format!(
                "{}{}: {parameter_type}",
                parameter.name(),
                if parameter.is_optional() { "?" } else { "" }
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let return_type = return_type.map_or_else(
        || infer_arrow_body_return_type(body, &parameter_types),
        |annotation| annotation.type_spellings().collect::<Vec<_>>().join(" | "),
    );
    format!("({signature_parameters}) => {return_type}")
}

fn infer_arrow_body_return_type(
    body: &ArrowFunctionBody,
    parameter_types: &HashMap<&str, String>,
) -> String {
    let mut return_types = Vec::new();
    for expression in body.return_expressions() {
        let inferred_type = expression.map_or_else(
            || "undefined".to_owned(),
            |expression| infer_expression_type_with_parameters(expression, parameter_types),
        );
        if inferred_type == "any" {
            return "any".to_owned();
        }
        for type_name in inferred_type.split(" | ") {
            if !return_types.iter().any(|existing| existing == type_name) {
                return_types.push(type_name.to_owned());
            }
        }
    }
    if return_types.is_empty() {
        "void".to_owned()
    } else {
        return_types.join(" | ")
    }
}

fn infer_binary_expression_type(
    left: &Expression,
    operator: BinaryOperator,
    right: &Expression,
    parameter_types: &HashMap<&str, String>,
) -> String {
    let left_type = infer_expression_type_with_parameters(left, parameter_types);
    let right_type = infer_expression_type_with_parameters(right, parameter_types);
    match operator {
        BinaryOperator::Add if left_type == "string" || right_type == "string" => {
            "string".to_owned()
        }
        BinaryOperator::NullishCoalesce => infer_nullish_coalescing_type(&left_type, &right_type),
        BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => {
            if left_type == "any" || right_type == "any" {
                "any".to_owned()
            } else if left_type == right_type {
                left_type
            } else {
                format!("{left_type} | {right_type}")
            }
        }
        BinaryOperator::LessThan
        | BinaryOperator::GreaterThan
        | BinaryOperator::LessThanOrEqual
        | BinaryOperator::GreaterThanOrEqual
        | BinaryOperator::Equal
        | BinaryOperator::StrictEqual
        | BinaryOperator::NotEqual
        | BinaryOperator::StrictNotEqual => "boolean".to_owned(),
        BinaryOperator::Add
        | BinaryOperator::Subtract
        | BinaryOperator::Multiply
        | BinaryOperator::Divide
        | BinaryOperator::Remainder
            if left_type == "bigint" && right_type == "bigint" =>
        {
            "bigint".to_owned()
        }
        BinaryOperator::Add
        | BinaryOperator::Subtract
        | BinaryOperator::Multiply
        | BinaryOperator::Divide
        | BinaryOperator::Remainder => "number".to_owned(),
    }
}

fn infer_nullish_coalescing_type(left_type: &str, right_type: &str) -> String {
    if left_type == "any" || right_type == "any" {
        return "any".to_owned();
    }

    let mut result_types = Vec::new();
    for type_name in left_type.split(" | ").chain(right_type.split(" | ")) {
        if !matches!(type_name, "null" | "undefined") && !result_types.contains(&type_name) {
            result_types.push(type_name);
        }
    }
    result_types.join(" | ")
}
