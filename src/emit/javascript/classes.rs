use crate::compiler::{ModuleKind, ScriptTarget};
use crate::syntax::{ClassDeclaration, ClassMember};

use super::{EmitContext, emit_function_body_statement, parameters};

pub(super) fn emit_class_declaration(
    emitter: &super::JavaScriptEmitter,
    declaration: &ClassDeclaration,
    exported: bool,
    output: &mut String,
) {
    if exported && emitter.module.is_ecma_script() {
        output.push_str("export ");
    }
    emit_class(declaration, emitter.target, &emitter.emit_context, output);
    if exported && emitter.module == ModuleKind::CommonJs {
        output.push_str("exports.");
        output.push_str(declaration.name());
        output.push_str(" = ");
        output.push_str(declaration.name());
        output.push_str(";\n");
    }
}

pub(super) fn emit_class(
    declaration: &ClassDeclaration,
    target: ScriptTarget,
    context: &EmitContext,
    output: &mut String,
) {
    output.push_str("class ");
    output.push_str(declaration.name());
    if let Some(base_class) = declaration.base_class() {
        output.push_str(" extends ");
        output.push_str(
            context
                .import_references
                .get(base_class)
                .map_or(base_class, String::as_str),
        );
    }
    output.push_str(" {\n");
    for member in declaration.members() {
        match member {
            ClassMember::Method(method) => {
                let is_constructor = method.name() == "constructor";
                if is_constructor {
                    emit_parameter_property_fields(method.parameters(), output);
                }
                output.push_str("  ");
                if method.is_static() {
                    output.push_str("static ");
                }
                output.push_str(method.name());
                parameters::emit_parameters(method.parameters(), true, context, target, 1, output);
                output.push_str(" {\n");
                let mut parameter_properties_initialized = false;
                if is_constructor && declaration.base_class().is_none() {
                    emit_parameter_property_initializers(method.parameters(), output);
                    parameter_properties_initialized = true;
                }
                for statement in method.body() {
                    emit_function_body_statement(statement, 2, target, context, output);
                    if is_constructor
                        && !parameter_properties_initialized
                        && is_super_call(statement)
                    {
                        emit_parameter_property_initializers(method.parameters(), output);
                        parameter_properties_initialized = true;
                    }
                }
                if is_constructor && !parameter_properties_initialized {
                    emit_parameter_property_initializers(method.parameters(), output);
                }
                output.push_str("  }\n");
            }
            ClassMember::Property(property) => {
                output.push_str("  ");
                if property.is_static() {
                    output.push_str("static ");
                }
                output.push_str(property.name());
                if let Some(initializer) = property.initializer() {
                    output.push_str(" = ");
                    output.push_str(&super::emit_expression(initializer, context, target, 1));
                }
                output.push_str(";\n");
            }
        }
    }
    output.push_str("}\n");
}

fn is_super_call(statement: &crate::syntax::FunctionBodyStatement) -> bool {
    matches!(
        statement,
        crate::syntax::FunctionBodyStatement::Expression(
            crate::syntax::Expression::CallExpression { callee, .. }
        ) if matches!(
            callee.as_ref(),
            crate::syntax::Expression::Identifier { name, .. } if name == "super"
        )
    )
}

fn emit_parameter_property_fields(
    parameters: &[crate::syntax::FunctionParameter],
    output: &mut String,
) {
    for parameter in parameters
        .iter()
        .filter(|parameter| parameter.is_parameter_property())
    {
        output.push_str("  ");
        output.push_str(parameter.name());
        output.push_str(";\n");
    }
}

fn emit_parameter_property_initializers(
    parameters: &[crate::syntax::FunctionParameter],
    output: &mut String,
) {
    for parameter in parameters
        .iter()
        .filter(|parameter| parameter.is_parameter_property())
    {
        output.push_str("    this.");
        output.push_str(parameter.name());
        output.push_str(" = ");
        output.push_str(parameter.name());
        output.push_str(";\n");
    }
}
