use super::EmitContext;
use crate::compiler::{ModuleKind, ScriptTarget};
use crate::enum_values::{EnumValue, format_enum_number, values_for_enum};
use crate::syntax::{EnumDeclaration, EnumMember};

pub(super) fn emit_enum_declaration(
    declaration: &EnumDeclaration,
    exported: bool,
    module: ModuleKind,
    target: ScriptTarget,
    context: &EmitContext,
    output: &mut String,
) {
    if declaration.is_const() {
        return;
    }
    if exported && module.is_ecma_script() {
        output.push_str("export ");
    }
    output.push_str("var ");
    output.push_str(declaration.name());
    output.push_str(";\n(function (");
    output.push_str(declaration.name());
    output.push_str(") {\n");

    for (member, value) in declaration
        .members()
        .iter()
        .zip(values_for_enum(declaration))
    {
        emit_enum_member(declaration, member, &value, context, target, 2, output);
    }

    output.push_str("})(");
    output.push_str(declaration.name());
    output.push_str(" || (");
    output.push_str(declaration.name());
    output.push_str(" = {}));\n");
    if exported && module == ModuleKind::CommonJs {
        output.push_str("exports.");
        output.push_str(declaration.name());
        output.push_str(" = ");
        output.push_str(declaration.name());
        output.push_str(";\n");
    }
}

pub(super) fn emit_namespace_enum(
    declaration: &EnumDeclaration,
    namespace_name: &str,
    exported: bool,
    target: ScriptTarget,
    context: &EmitContext,
    indentation: usize,
    output: &mut String,
) {
    if declaration.is_const() {
        return;
    }
    super::write_indentation(output, indentation);
    output.push_str(super::declaration_keyword(
        crate::syntax::VariableDeclarationKind::Let,
        target,
    ));
    output.push(' ');
    output.push_str(declaration.name());
    output.push_str(";\n");
    super::write_indentation(output, indentation);
    output.push_str("(function (");
    output.push_str(declaration.name());
    output.push_str(") {\n");

    for (member, value) in declaration
        .members()
        .iter()
        .zip(values_for_enum(declaration))
    {
        emit_enum_member(
            declaration,
            member,
            &value,
            context,
            target,
            indentation + 2,
            output,
        );
    }

    super::write_indentation(output, indentation);
    output.push_str("})(");
    output.push_str(declaration.name());
    if exported {
        output.push_str(" = ");
        output.push_str(namespace_name);
        output.push('.');
        output.push_str(declaration.name());
        output.push_str(" || (");
        output.push_str(namespace_name);
        output.push('.');
        output.push_str(declaration.name());
        output.push_str(" = {}));\n");
    } else {
        output.push_str(" || (");
        output.push_str(declaration.name());
        output.push_str(" = {}));\n");
    }
}

fn emit_enum_member(
    declaration: &EnumDeclaration,
    member: &EnumMember,
    value: &EnumValue,
    context: &EmitContext,
    target: ScriptTarget,
    indentation: usize,
    output: &mut String,
) {
    match value {
        EnumValue::Number(value) => emit_numeric_enum_member(
            declaration.name(),
            member.name(),
            &format_enum_number(*value),
            indentation,
            output,
        ),
        EnumValue::String(raw) => {
            emit_string_enum_member(declaration.name(), member.name(), raw, indentation, output);
        }
        EnumValue::Computed => {
            let value = member.initializer().map_or_else(
                || "0".to_owned(),
                |initializer| super::emit_expression(initializer, context, target, 1),
            );
            emit_numeric_enum_member(
                declaration.name(),
                member.name(),
                &value,
                indentation,
                output,
            );
        }
    }
}

fn emit_numeric_enum_member(
    enum_name: &str,
    member_name: &str,
    value: &str,
    indentation: usize,
    output: &mut String,
) {
    super::write_indentation(output, indentation);
    output.push_str(enum_name);
    output.push('[');
    output.push_str(enum_name);
    output.push_str("[\"");
    output.push_str(member_name);
    output.push_str("\"] = ");
    output.push_str(value);
    output.push_str("] = \"");
    output.push_str(member_name);
    output.push_str("\";\n");
}

fn emit_string_enum_member(
    enum_name: &str,
    member_name: &str,
    raw_value: &str,
    indentation: usize,
    output: &mut String,
) {
    super::write_indentation(output, indentation);
    output.push_str(enum_name);
    output.push_str("[\"");
    output.push_str(member_name);
    output.push_str("\"] = ");
    output.push_str(raw_value);
    output.push_str(";\n");
}
