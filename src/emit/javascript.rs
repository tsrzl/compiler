//! JavaScript output for the currently supported syntax nodes.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::compiler::{ModuleKind, ScriptTarget};
use crate::enum_values::{EnumValue, format_enum_number, values_for_enum};
use crate::syntax::{
    ArrowFunctionBody, ExportAllDeclaration, ExportNamedFromDeclaration, Expression,
    FunctionBodyStatement, FunctionDeclaration, Program, Statement, UnaryOperator,
    VariableDeclaration, VariableDeclarationKind,
};

mod classes;
mod enums;
mod expressions;
mod parameters;

use expressions::emit_expression;

#[derive(Default)]
struct CommonJsImportBindings {
    module_variables: HashMap<String, String>,
    reexport_module_variables: HashMap<String, String>,
    local_references: HashMap<String, String>,
    default_modules: HashSet<String>,
    namespace_modules: HashSet<String>,
}

struct EmitContext {
    import_references: HashMap<String, String>,
    const_enum_members: HashMap<(String, String), String>,
}

impl EmitContext {
    fn new(program: &Program, import_references: &HashMap<String, String>) -> Self {
        let mut context = Self {
            import_references: import_references.clone(),
            const_enum_members: HashMap::new(),
        };
        for statement in program.statements() {
            context.collect_const_enum_members(statement.declaration());
        }
        context
    }

    fn collect_const_enum_members(&mut self, statement: &Statement) {
        match statement {
            Statement::ExportedDeclaration(declaration) => {
                self.collect_const_enum_members(declaration);
            }
            Statement::EnumDeclaration(declaration) if declaration.is_const() => {
                for (member, value) in declaration
                    .members()
                    .iter()
                    .zip(values_for_enum(declaration))
                {
                    let emitted_value = match value {
                        EnumValue::Number(value) => format_enum_number(value),
                        EnumValue::String(raw) => raw,
                        EnumValue::Computed => continue,
                    };
                    self.const_enum_members.insert(
                        (declaration.name().to_owned(), member.name().to_owned()),
                        format!(
                            "{emitted_value} /* {}.{} */",
                            declaration.name(),
                            member.name()
                        ),
                    );
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn emit(program: &Program, target: ScriptTarget, module: ModuleKind) -> String {
    let mut output = String::new();
    let has_default_import = program.statements().iter().any(|statement| {
        matches!(statement.declaration(), Statement::ImportDeclaration(import) if !import.is_type_only() && import.default_import().is_some())
    });
    let has_namespace_import = program.statements().iter().any(|statement| {
        matches!(statement.declaration(), Statement::ImportDeclaration(import) if !import.is_type_only() && import.namespace_import().is_some())
    });
    let has_export_all = program
        .statements()
        .iter()
        .any(|statement| matches!(statement.declaration(), Statement::ExportAll(_)));

    if module == ModuleKind::CommonJs
        && program
            .statements()
            .iter()
            .any(Statement::is_external_module_indicator)
    {
        output.push_str("\"use strict\";\n");
        if has_namespace_import || has_export_all {
            emit_commonjs_create_binding_helper(&mut output);
        }
        if has_namespace_import {
            emit_commonjs_import_star_helpers(&mut output);
        }
        if has_export_all {
            emit_commonjs_export_star_helper(&mut output);
        }
        if has_default_import {
            emit_commonjs_default_import_helper(&mut output);
        }
        output.push_str("Object.defineProperty(exports, \"__esModule\", { value: true });\n");
    }
    let preinitialized_functions = if module == ModuleKind::CommonJs {
        emit_commonjs_export_initializers(program, &mut output)
    } else {
        Vec::new()
    };
    let import_bindings = if module == ModuleKind::CommonJs {
        commonjs_import_bindings(program)
    } else {
        CommonJsImportBindings::default()
    };
    let emit_context = EmitContext::new(program, &import_bindings.local_references);
    let mut emitter = JavaScriptEmitter {
        target,
        module,
        preinitialized_functions,
        import_bindings,
        emit_context,
        emitted_module_imports: HashSet::new(),
        emitted_reexport_modules: HashSet::new(),
    };

    for statement in program.statements() {
        emitter.emit_statement(statement, false, &mut output);
    }

    output
}

fn emit_commonjs_default_import_helper(output: &mut String) {
    output.push_str("var __importDefault = (this && this.__importDefault) || function (mod) {\n");
    output.push_str("    return (mod && mod.__esModule) ? mod : { \"default\": mod };\n");
    output.push_str("};\n");
}

fn emit_commonjs_create_binding_helper(output: &mut String) {
    output.push_str(
        r#"var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
"#,
    );
}

fn emit_commonjs_import_star_helpers(output: &mut String) {
    output.push_str(
        r#"var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
"#,
    );
}

fn emit_commonjs_export_star_helper(output: &mut String) {
    output.push_str(
        r#"var __exportStar = (this && this.__exportStar) || function(m, exports) {
    for (var p in m) if (p !== "default" && !Object.prototype.hasOwnProperty.call(exports, p)) __createBinding(exports, m, p);
};
"#,
    );
}

fn emit_commonjs_export_initializers(program: &Program, output: &mut String) -> Vec<String> {
    let mut exports = Vec::new();
    let mut initialized_names = HashSet::new();
    for statement in program.statements() {
        if let Statement::ExportNamedFrom(export) = statement.declaration() {
            if export.is_type_only() {
                continue;
            }
            for specifier in export.specifiers() {
                if initialized_names.insert(specifier.exported_name().to_owned()) {
                    output.push_str("exports.");
                    output.push_str(specifier.exported_name());
                    output.push_str(" = void 0;\n");
                }
            }
        }
        if statement.is_exported()
            && let Some(variable) = statement.as_variable_declaration()
        {
            exports.push((variable.name().to_owned(), variable.name().to_owned()));
        }
        if let Statement::ExportNamed(specifiers) = statement {
            exports.extend(specifiers.iter().map(|specifier| {
                (
                    specifier.local_name().to_owned(),
                    specifier.exported_name().to_owned(),
                )
            }));
        }
    }

    let mut initialized_variables = Vec::new();
    let mut initialized_functions = Vec::new();
    for (local_name, exported_name) in exports {
        let is_variable = program.statements().iter().any(|candidate| {
            candidate
                .as_variable_declaration()
                .is_some_and(|declaration| declaration.name() == local_name)
        });
        let is_function = program.statements().iter().any(|candidate| {
            candidate
                .as_function_declaration()
                .is_some_and(|declaration| declaration.name() == local_name)
        });
        let is_direct_function_export = program.statements().iter().any(|candidate| {
            candidate.is_exported()
                && candidate
                    .as_function_declaration()
                    .is_some_and(|declaration| declaration.name() == local_name)
        });
        if is_function
            && !is_direct_function_export
            && !initialized_functions
                .iter()
                .any(|name| name == &exported_name)
        {
            output.push_str("exports.");
            output.push_str(&exported_name);
            output.push_str(" = ");
            output.push_str(&local_name);
            output.push_str(";\n");
            initialized_functions.push(exported_name);
        } else if is_variable
            && !initialized_variables
                .iter()
                .any(|name| name == &exported_name)
        {
            output.push_str("exports.");
            output.push_str(&exported_name);
            output.push_str(" = void 0;\n");
            initialized_variables.push(exported_name);
        }
    }
    initialized_functions
}

fn commonjs_import_bindings(program: &Program) -> CommonJsImportBindings {
    let mut bindings = CommonJsImportBindings::default();
    let mut occupied_names = HashSet::new();

    for statement in program.statements() {
        if let Some(variable) = statement.as_variable_declaration() {
            occupied_names.insert(variable.name().to_owned());
        }
        if let Some(function) = statement.as_function_declaration() {
            occupied_names.insert(function.name().to_owned());
        }
        if let Statement::ImportDeclaration(import) = statement.declaration()
            && !import.is_type_only()
        {
            occupied_names.extend(
                import
                    .imported_bindings()
                    .map(|specifier| specifier.local_name().to_owned()),
            );
        }
    }

    for statement in program.statements() {
        let Statement::ImportDeclaration(import) = statement.declaration() else {
            continue;
        };
        if import.is_type_only() {
            continue;
        }
        if import.imported_bindings().next().is_none() {
            continue;
        }

        let module_variable =
            if let Some(existing) = bindings.module_variables.get(import.module_specifier()) {
                existing.clone()
            } else {
                let name = import.namespace_import().map_or_else(
                    || module_binding_name(import.module_specifier(), &mut occupied_names),
                    |namespace_import| namespace_import.local_name().to_owned(),
                );
                bindings
                    .module_variables
                    .insert(import.module_specifier().to_owned(), name.clone());
                name
            };
        for specifier in import.imported_bindings() {
            bindings.local_references.insert(
                specifier.local_name().to_owned(),
                if specifier.imported_name() == "*" {
                    module_variable.clone()
                } else {
                    format!("{module_variable}.{}", specifier.imported_name())
                },
            );
        }
        if import.default_import().is_some() {
            bindings
                .default_modules
                .insert(import.module_specifier().to_owned());
        }
        if import.namespace_import().is_some() {
            bindings
                .namespace_modules
                .insert(import.module_specifier().to_owned());
        }
    }

    for statement in program.statements() {
        let Statement::ExportNamedFrom(export) = statement.declaration() else {
            continue;
        };
        if export.is_type_only() {
            continue;
        }
        if !bindings
            .reexport_module_variables
            .contains_key(export.module_specifier())
        {
            let name = module_binding_name(export.module_specifier(), &mut occupied_names);
            bindings
                .reexport_module_variables
                .insert(export.module_specifier().to_owned(), name);
        }
    }

    bindings
}

fn module_binding_name(module_specifier: &str, occupied_names: &mut HashSet<String>) -> String {
    let stem = Path::new(module_specifier)
        .file_stem()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("module");
    let mut base = String::new();
    for (index, character) in stem.chars().enumerate() {
        let valid = if index == 0 {
            character == '_' || character == '$' || character.is_alphabetic()
        } else {
            character == '_' || character == '$' || character.is_alphanumeric()
        };
        base.push(if valid { character } else { '_' });
    }
    if base.is_empty() {
        base.push_str("module");
    }

    let mut suffix = 1;
    loop {
        let candidate = format!("{base}_{suffix}");
        if occupied_names.insert(candidate.clone()) {
            return candidate;
        }
        suffix += 1;
    }
}

struct JavaScriptEmitter {
    target: ScriptTarget,
    module: ModuleKind,
    preinitialized_functions: Vec<String>,
    import_bindings: CommonJsImportBindings,
    emit_context: EmitContext,
    emitted_module_imports: HashSet<String>,
    emitted_reexport_modules: HashSet<String>,
}

impl JavaScriptEmitter {
    fn emit_statement(&mut self, statement: &Statement, exported: bool, output: &mut String) {
        match statement {
            Statement::ExportedDeclaration(declaration) => {
                self.emit_statement(declaration, true, output);
            }
            Statement::VariableDeclaration(declaration) => {
                self.emit_variable(declaration, exported, output);
            }
            Statement::FunctionDeclaration(function) => {
                self.emit_function_declaration(function, exported, output);
            }
            Statement::ClassDeclaration(declaration) => {
                classes::emit_class_declaration(self, declaration, exported, output);
            }
            Statement::NamespaceDeclaration(_) => {}
            Statement::EnumDeclaration(declaration) => enums::emit_enum_declaration(
                declaration,
                exported,
                self.module,
                self.target,
                &self.emit_context,
                output,
            ),
            Statement::ExpressionStatement(expression) => {
                output.push_str(&emit_expression(
                    expression,
                    &self.emit_context,
                    self.target,
                    0,
                ));
                output.push_str(";\n");
            }
            Statement::ControlFlowStatement(control_flow) => emit_function_body_statement(
                control_flow,
                0,
                self.target,
                &self.emit_context,
                output,
            ),
            Statement::Break { .. } => output.push_str("break;\n"),
            Statement::Continue { .. } => output.push_str("continue;\n"),
            Statement::ImportDeclaration(import) => self.emit_import(import, output),
            Statement::ExportDefault(expression) => self.emit_default_export(expression, output),
            Statement::ExportNamed(specifiers) => self.emit_named_exports(specifiers, output),
            Statement::ExportTypeNamed(_)
            | Statement::InterfaceDeclaration(_)
            | Statement::TypeAliasDeclaration(_) => {}
            Statement::ExportNamedFrom(export) => self.emit_named_reexport(export, output),
            Statement::ExportAll(export) => self.emit_export_all(export, output),
        }
    }

    fn emit_variable(
        &self,
        declaration: &VariableDeclaration,
        exported: bool,
        output: &mut String,
    ) {
        if exported && self.module == ModuleKind::CommonJs {
            emit_commonjs_variable(declaration, &self.emit_context, self.target, output);
            return;
        }
        if exported && self.module.is_ecma_script() {
            output.push_str("export ");
        }
        emit_variable_declaration(declaration, self.target, &self.emit_context, 0, output);
        output.push_str(";\n");
    }

    fn emit_function_declaration(
        &self,
        function: &FunctionDeclaration,
        exported: bool,
        output: &mut String,
    ) {
        if exported && self.module == ModuleKind::CommonJs {
            output.push_str("exports.");
            output.push_str(function.name());
            output.push_str(" = ");
            output.push_str(function.name());
            output.push_str(";\n");
        }
        if exported && self.module.is_ecma_script() {
            output.push_str("export ");
        }
        emit_function(function, self.target, &self.emit_context, output);
    }

    fn emit_import(&mut self, import: &crate::syntax::ImportDeclaration, output: &mut String) {
        if import.is_type_only() {
            return;
        }
        if self.module.is_ecma_script() {
            output.push_str("import ");
            if let Some(default_import) = import.default_import() {
                output.push_str(default_import.local_name());
                if import.namespace_import().is_some() || !import.named_imports().is_empty() {
                    output.push_str(", ");
                }
            }
            if let Some(namespace_import) = import.namespace_import() {
                output.push_str("* as ");
                output.push_str(namespace_import.local_name());
                output.push_str(" from ");
            } else if !import.named_imports().is_empty() {
                output.push_str("{ ");
                for (index, specifier) in import.named_imports().iter().enumerate() {
                    if index > 0 {
                        output.push_str(", ");
                    }
                    output.push_str(specifier.imported_name());
                    if specifier.imported_name() != specifier.local_name() {
                        output.push_str(" as ");
                        output.push_str(specifier.local_name());
                    }
                }
                output.push_str(" } from ");
            } else if import.default_import().is_some() {
                output.push_str(" from ");
            }
            output.push_str(import.raw_module_specifier());
            output.push_str(";\n");
        } else if import.imported_bindings().next().is_none() {
            output.push_str("require(\"");
            output.push_str(&escape_javascript_string(import.module_specifier()));
            output.push_str("\");\n");
        } else if self
            .emitted_module_imports
            .insert(import.module_specifier().to_owned())
        {
            let module_variable = self
                .import_bindings
                .module_variables
                .get(import.module_specifier())
                .expect("named imports have a generated CommonJS module binding");
            output.push_str(declaration_keyword(
                VariableDeclarationKind::Const,
                self.target,
            ));
            output.push(' ');
            output.push_str(module_variable);
            output.push_str(" = ");
            let uses_default_import = self
                .import_bindings
                .default_modules
                .contains(import.module_specifier());
            let uses_namespace_import = self
                .import_bindings
                .namespace_modules
                .contains(import.module_specifier());
            if uses_namespace_import {
                output.push_str("__importStar(");
            } else if uses_default_import {
                output.push_str("__importDefault(");
            }
            output.push_str("require(\"");
            output.push_str(&escape_javascript_string(import.module_specifier()));
            output.push_str("\")");
            if uses_namespace_import || uses_default_import {
                output.push(')');
            }
            output.push_str(";\n");
        }
    }

    fn emit_default_export(&self, expression: &Expression, output: &mut String) {
        if self.module.is_ecma_script() {
            output.push_str("export default ");
        } else {
            output.push_str("exports.default = ");
        }
        output.push_str(&emit_expression(
            expression,
            &self.emit_context,
            self.target,
            0,
        ));
        output.push_str(";\n");
    }

    fn emit_named_exports(
        &self,
        specifiers: &[crate::syntax::ExportSpecifier],
        output: &mut String,
    ) {
        if self.module.is_ecma_script() {
            output.push_str("export { ");
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
            output.push_str(" };\n");
        } else {
            for specifier in specifiers {
                if self
                    .preinitialized_functions
                    .iter()
                    .any(|name| name == specifier.exported_name())
                {
                    continue;
                }
                output.push_str("exports.");
                output.push_str(specifier.exported_name());
                output.push_str(" = ");
                output.push_str(specifier.local_name());
                output.push_str(";\n");
            }
        }
    }

    fn emit_export_all(&self, export: &ExportAllDeclaration, output: &mut String) {
        if self.module.is_ecma_script() {
            output.push_str("export * from ");
            output.push_str(export.raw_module_specifier());
            output.push_str(";\n");
        } else {
            output.push_str("__exportStar(require(\"");
            output.push_str(&escape_javascript_string(export.module_specifier()));
            output.push_str("\"), exports);\n");
        }
    }

    fn emit_named_reexport(&mut self, export: &ExportNamedFromDeclaration, output: &mut String) {
        if export.is_type_only() {
            return;
        }
        if self.module.is_ecma_script() {
            output.push_str("export { ");
            for (index, specifier) in export.specifiers().iter().enumerate() {
                if index > 0 {
                    output.push_str(", ");
                }
                output.push_str(specifier.local_name());
                if specifier.local_name() != specifier.exported_name() {
                    output.push_str(" as ");
                    output.push_str(specifier.exported_name());
                }
            }
            output.push_str(" } from ");
            output.push_str(export.raw_module_specifier());
            output.push_str(";\n");
        } else {
            if self
                .emitted_reexport_modules
                .insert(export.module_specifier().to_owned())
            {
                let module_variable = self
                    .import_bindings
                    .reexport_module_variables
                    .get(export.module_specifier())
                    .expect("named re-exports have a generated CommonJS module binding");
                output.push_str("var ");
                output.push_str(module_variable);
                output.push_str(" = require(\"");
                output.push_str(&escape_javascript_string(export.module_specifier()));
                output.push_str("\");\n");
            }
            let module_variable = self
                .import_bindings
                .reexport_module_variables
                .get(export.module_specifier())
                .expect("named re-exports have a generated CommonJS module binding");
            for specifier in export.specifiers() {
                output.push_str("Object.defineProperty(exports, \"");
                output.push_str(specifier.exported_name());
                output.push_str("\", { enumerable: true, get: function () { return ");
                output.push_str(module_variable);
                output.push('.');
                output.push_str(specifier.local_name());
                output.push_str("; } });\n");
            }
        }
    }
}

fn emit_commonjs_variable(
    declaration: &VariableDeclaration,
    context: &EmitContext,
    target: ScriptTarget,
    output: &mut String,
) {
    if let Some(initializer) = declaration.initializer() {
        output.push_str("exports.");
        output.push_str(declaration.name());
        output.push_str(" = ");
        output.push_str(&emit_expression(initializer, context, target, 0));
        output.push_str(";\n");
    }
}

fn emit_function(
    function: &FunctionDeclaration,
    target: ScriptTarget,
    context: &EmitContext,
    output: &mut String,
) {
    output.push_str("function ");
    output.push_str(function.name());
    parameters::emit_parameters(function.parameters(), true, context, target, 0, output);
    output.push_str(" {\n");
    for statement in function.body() {
        emit_function_body_statement(statement, 1, target, context, output);
    }
    output.push_str("}\n");
}

fn emit_function_body_statement(
    statement: &FunctionBodyStatement,
    indentation: usize,
    target: ScriptTarget,
    context: &EmitContext,
    output: &mut String,
) {
    match statement {
        FunctionBodyStatement::VariableDeclaration(declaration) => {
            emit_variable_body_statement(declaration, indentation, target, context, output);
        }
        FunctionBodyStatement::Expression(expression) => {
            emit_expression_body_statement(expression, indentation, target, context, output)
        }
        FunctionBodyStatement::Return(statement) => {
            emit_return_body_statement(statement, indentation, target, context, output);
        }
        FunctionBodyStatement::Throw(expression) => {
            emit_throw_body_statement(expression, indentation, target, context, output);
        }
        if_statement @ FunctionBodyStatement::If { .. } => {
            emit_if_statement(if_statement, indentation, target, context, output);
        }
        FunctionBodyStatement::While { condition, body } => {
            emit_while_statement(condition, body, indentation, target, context, output)
        }
        FunctionBodyStatement::DoWhile { body, condition } => {
            emit_do_while_statement(body, condition, indentation, target, context, output)
        }
        for_statement @ FunctionBodyStatement::For { .. } => {
            emit_for_loop_statement(for_statement, indentation, target, context, output)
        }
        for_of_statement @ FunctionBodyStatement::ForOf { .. } => {
            emit_for_of_loop_statement(for_of_statement, indentation, target, context, output)
        }
        for_in_statement @ FunctionBodyStatement::ForIn { .. } => {
            emit_for_in_loop_statement(for_in_statement, indentation, target, context, output)
        }
        switch_statement @ FunctionBodyStatement::Switch { .. } => {
            emit_switch_statement(switch_statement, indentation, target, context, output)
        }
        try_statement @ FunctionBodyStatement::Try { .. } => {
            emit_try_statement(try_statement, indentation, target, context, output)
        }
        FunctionBodyStatement::Break { .. } => {
            write_indentation(output, indentation);
            output.push_str("break;\n");
        }
        FunctionBodyStatement::Continue { .. } => {
            write_indentation(output, indentation);
            output.push_str("continue;\n");
        }
    }
}

fn emit_variable_body_statement(
    declaration: &VariableDeclaration,
    indentation: usize,
    target: ScriptTarget,
    context: &EmitContext,
    output: &mut String,
) {
    write_indentation(output, indentation);
    emit_variable_declaration(declaration, target, context, indentation, output);
    output.push_str(";\n");
}

fn emit_expression_body_statement(
    expression: &Expression,
    indentation: usize,
    target: ScriptTarget,
    context: &EmitContext,
    output: &mut String,
) {
    write_indentation(output, indentation);
    output.push_str(&emit_expression(expression, context, target, indentation));
    output.push_str(";\n");
}

fn emit_return_body_statement(
    statement: &crate::syntax::ReturnStatement,
    indentation: usize,
    target: ScriptTarget,
    context: &EmitContext,
    output: &mut String,
) {
    write_indentation(output, indentation);
    output.push_str("return");
    if let Some(expression) = statement.expression() {
        output.push(' ');
        output.push_str(&emit_expression(expression, context, target, indentation));
    }
    output.push_str(";\n");
}

fn emit_throw_body_statement(
    expression: &Expression,
    indentation: usize,
    target: ScriptTarget,
    context: &EmitContext,
    output: &mut String,
) {
    write_indentation(output, indentation);
    output.push_str("throw ");
    output.push_str(&emit_expression(expression, context, target, indentation));
    output.push_str(";\n");
}

fn emit_while_statement(
    condition: &Expression,
    body: &[FunctionBodyStatement],
    indentation: usize,
    target: ScriptTarget,
    context: &EmitContext,
    output: &mut String,
) {
    write_indentation(output, indentation);
    output.push_str("while (");
    output.push_str(&emit_expression(condition, context, target, indentation));
    output.push_str(") {\n");
    for statement in body {
        emit_function_body_statement(statement, indentation + 1, target, context, output);
    }
    write_indentation(output, indentation);
    output.push_str("}\n");
}

fn emit_do_while_statement(
    body: &[FunctionBodyStatement],
    condition: &Expression,
    indentation: usize,
    target: ScriptTarget,
    context: &EmitContext,
    output: &mut String,
) {
    write_indentation(output, indentation);
    output.push_str("do {\n");
    for statement in body {
        emit_function_body_statement(statement, indentation + 1, target, context, output);
    }
    write_indentation(output, indentation);
    output.push_str("} while (");
    output.push_str(&emit_expression(condition, context, target, indentation));
    output.push_str(");\n");
}

fn emit_if_statement(
    statement: &FunctionBodyStatement,
    indentation: usize,
    target: ScriptTarget,
    context: &EmitContext,
    output: &mut String,
) {
    let FunctionBodyStatement::If {
        condition,
        then_body,
        else_body,
    } = statement
    else {
        return;
    };
    write_indentation(output, indentation);
    output.push_str("if (");
    output.push_str(&emit_expression(condition, context, target, indentation));
    output.push_str(") {\n");
    for statement in then_body {
        emit_function_body_statement(statement, indentation + 1, target, context, output);
    }
    write_indentation(output, indentation);
    if let Some(else_body) = else_body {
        output.push_str("} else {\n");
        for statement in else_body {
            emit_function_body_statement(statement, indentation + 1, target, context, output);
        }
        write_indentation(output, indentation);
    }
    output.push_str("}\n");
}

fn emit_for_loop_statement(
    statement: &FunctionBodyStatement,
    indentation: usize,
    target: ScriptTarget,
    context: &EmitContext,
    output: &mut String,
) {
    let FunctionBodyStatement::For {
        initializer,
        condition,
        incrementor,
        body,
    } = statement
    else {
        return;
    };
    write_indentation(output, indentation);
    output.push_str("for (");
    if let Some(initializer) = initializer {
        match initializer {
            crate::syntax::ForInitializer::VariableDeclarations(declarations) => {
                if let Some((first, remaining)) = declarations.split_first() {
                    emit_variable_declaration(first, target, context, indentation, output);
                    for declaration in remaining {
                        output.push_str(", ");
                        emit_variable_declarator(declaration, context, target, indentation, output);
                    }
                }
            }
            crate::syntax::ForInitializer::Expression(expression) => {
                output.push_str(&emit_expression(expression, context, target, indentation));
            }
        }
    }
    output.push(';');
    if let Some(condition) = condition {
        output.push(' ');
        output.push_str(&emit_expression(condition, context, target, indentation));
    }
    output.push(';');
    if let Some(incrementor) = incrementor {
        output.push(' ');
        output.push_str(&emit_expression(incrementor, context, target, indentation));
    }
    output.push_str(") {\n");
    for statement in body {
        emit_function_body_statement(statement, indentation + 1, target, context, output);
    }
    write_indentation(output, indentation);
    output.push_str("}\n");
}

fn emit_for_of_loop_statement(
    statement: &FunctionBodyStatement,
    indentation: usize,
    target: ScriptTarget,
    context: &EmitContext,
    output: &mut String,
) {
    let FunctionBodyStatement::ForOf {
        initializer,
        iterable,
        body,
    } = statement
    else {
        return;
    };
    write_indentation(output, indentation);
    output.push_str("for (");
    match initializer {
        crate::syntax::ForInitializer::VariableDeclarations(declarations) => {
            if let Some(declaration) = declarations.first() {
                emit_variable_declaration(declaration, target, context, indentation, output);
            }
        }
        crate::syntax::ForInitializer::Expression(expression) => {
            output.push_str(&emit_expression(expression, context, target, indentation));
        }
    }
    output.push_str(" of ");
    output.push_str(&emit_expression(iterable, context, target, indentation));
    output.push_str(") {\n");
    for statement in body {
        emit_function_body_statement(statement, indentation + 1, target, context, output);
    }
    write_indentation(output, indentation);
    output.push_str("}\n");
}

fn emit_for_in_loop_statement(
    statement: &FunctionBodyStatement,
    indentation: usize,
    target: ScriptTarget,
    context: &EmitContext,
    output: &mut String,
) {
    let FunctionBodyStatement::ForIn {
        initializer,
        object,
        body,
    } = statement
    else {
        return;
    };
    write_indentation(output, indentation);
    output.push_str("for (");
    match initializer {
        crate::syntax::ForInitializer::VariableDeclarations(declarations) => {
            if let Some(declaration) = declarations.first() {
                emit_variable_declaration(declaration, target, context, indentation, output);
            }
        }
        crate::syntax::ForInitializer::Expression(expression) => {
            output.push_str(&emit_expression(expression, context, target, indentation));
        }
    }
    output.push_str(" in ");
    output.push_str(&emit_expression(object, context, target, indentation));
    output.push_str(") {\n");
    for statement in body {
        emit_function_body_statement(statement, indentation + 1, target, context, output);
    }
    write_indentation(output, indentation);
    output.push_str("}\n");
}

fn emit_switch_statement(
    statement: &FunctionBodyStatement,
    indentation: usize,
    target: ScriptTarget,
    context: &EmitContext,
    output: &mut String,
) {
    let FunctionBodyStatement::Switch {
        expression,
        clauses,
    } = statement
    else {
        return;
    };
    write_indentation(output, indentation);
    output.push_str("switch (");
    output.push_str(&emit_expression(expression, context, target, indentation));
    output.push_str(") {\n");
    for clause in clauses {
        write_indentation(output, indentation + 1);
        if let Some(expression) = clause.expression() {
            output.push_str("case ");
            output.push_str(&emit_expression(expression, context, target, indentation));
        } else {
            output.push_str("default");
        }
        output.push_str(":\n");
        for statement in clause.statements() {
            emit_function_body_statement(statement, indentation + 2, target, context, output);
        }
    }
    write_indentation(output, indentation);
    output.push_str("}\n");
}

fn emit_try_statement(
    statement: &FunctionBodyStatement,
    indentation: usize,
    target: ScriptTarget,
    context: &EmitContext,
    output: &mut String,
) {
    let FunctionBodyStatement::Try {
        try_body,
        catch_clause,
        finally_body,
    } = statement
    else {
        return;
    };
    write_indentation(output, indentation);
    output.push_str("try {\n");
    for statement in try_body {
        emit_function_body_statement(statement, indentation + 1, target, context, output);
    }
    write_indentation(output, indentation);
    output.push('}');
    if let Some(catch_clause) = catch_clause {
        output.push_str(" catch");
        if let Some(variable) = catch_clause.variable() {
            output.push_str(" (");
            output.push_str(variable.name());
            output.push(')');
        }
        output.push_str(" {\n");
        for statement in catch_clause.body() {
            emit_function_body_statement(statement, indentation + 1, target, context, output);
        }
        write_indentation(output, indentation);
        output.push('}');
    }
    if let Some(finally_body) = finally_body {
        output.push_str(" finally {\n");
        for statement in finally_body {
            emit_function_body_statement(statement, indentation + 1, target, context, output);
        }
        write_indentation(output, indentation);
        output.push('}');
    }
    output.push('\n');
}

fn write_indentation(output: &mut String, indentation: usize) {
    for _ in 0..indentation {
        output.push_str("  ");
    }
}

fn emit_variable_declaration(
    declaration: &VariableDeclaration,
    target: ScriptTarget,
    context: &EmitContext,
    indentation: usize,
    output: &mut String,
) {
    output.push_str(declaration_keyword(declaration.declaration_kind(), target));
    output.push(' ');
    emit_variable_declarator(declaration, context, target, indentation, output);
}

fn emit_variable_declarator(
    declaration: &VariableDeclaration,
    context: &EmitContext,
    target: ScriptTarget,
    indentation: usize,
    output: &mut String,
) {
    output.push_str(declaration.name());
    if let Some(initializer) = declaration.initializer() {
        output.push_str(" = ");
        output.push_str(&emit_expression(initializer, context, target, indentation));
    }
}

fn declaration_keyword(kind: VariableDeclarationKind, target: ScriptTarget) -> &'static str {
    match (kind, target) {
        (VariableDeclarationKind::Var, _) | (_, ScriptTarget::Es5) => "var",
        (VariableDeclarationKind::Const, _) => "const",
        (VariableDeclarationKind::Let, _) => "let",
    }
}

fn emit_call_expression(
    callee: &Expression,
    arguments: &[Expression],
    context: &EmitContext,
    target: ScriptTarget,
    indentation: usize,
) -> String {
    let mut output = emit_expression(callee, context, target, indentation);
    output.push('(');
    for (index, argument) in arguments.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push_str(&emit_expression(argument, context, target, indentation));
    }
    output.push(')');
    output
}

fn emit_property_access_expression(
    receiver: &Expression,
    name: &str,
    context: &EmitContext,
    target: ScriptTarget,
    indentation: usize,
) -> String {
    if let Expression::Identifier {
        name: enum_name, ..
    } = receiver
        && let Some(value) = context
            .const_enum_members
            .get(&(enum_name.clone(), name.to_owned()))
    {
        return value.clone();
    }
    format!(
        "{}.{}",
        emit_expression(receiver, context, target, indentation),
        name
    )
}

fn emit_element_access_expression(
    receiver: &Expression,
    argument: &Expression,
    context: &EmitContext,
    target: ScriptTarget,
    indentation: usize,
) -> String {
    format!(
        "{}[{}]",
        emit_expression(receiver, context, target, indentation),
        emit_expression(argument, context, target, indentation)
    )
}

fn emit_unary_expression(
    operator: UnaryOperator,
    operand: &Expression,
    context: &EmitContext,
    target: ScriptTarget,
    indentation: usize,
) -> String {
    let operand = emit_expression(operand, context, target, indentation);
    let needs_separator = matches!(
        (operator, operand.as_bytes().first()),
        (UnaryOperator::Plus, Some(b'+')) | (UnaryOperator::Negate, Some(b'-'))
    );
    format!(
        "{}{}{}",
        operator.as_str(),
        if needs_separator { " " } else { "" },
        operand
    )
}

fn emit_object_literal(
    properties: &[crate::syntax::ObjectProperty],
    context: &EmitContext,
    target: ScriptTarget,
    indentation: usize,
) -> String {
    let mut output = String::from("{");
    for (index, property) in properties.iter().enumerate() {
        if index == 0 {
            output.push(' ');
        } else {
            output.push_str(", ");
        }
        output.push_str(property.name());
        output.push_str(": ");
        output.push_str(&emit_expression(
            property.value(),
            context,
            target,
            indentation,
        ));
    }
    if !properties.is_empty() {
        output.push(' ');
    }
    output.push('}');
    output
}

fn emit_array_literal(
    elements: &[Expression],
    context: &EmitContext,
    target: ScriptTarget,
    indentation: usize,
) -> String {
    let mut output = String::from("[");
    for (index, element) in elements.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push_str(&emit_expression(element, context, target, indentation));
    }
    output.push(']');
    output
}

fn emit_arrow_function(
    parameters: &[crate::syntax::FunctionParameter],
    parameters_parenthesized: bool,
    body: &ArrowFunctionBody,
    context: &EmitContext,
    target: ScriptTarget,
    indentation: usize,
) -> String {
    let mut output = String::new();
    if target == ScriptTarget::Es5 {
        output.push_str("function ");
    }
    let wrap_parameters =
        target == ScriptTarget::Es5 || parameters_parenthesized || parameters.len() != 1;
    parameters::emit_parameters(
        parameters,
        wrap_parameters,
        context,
        target,
        indentation,
        &mut output,
    );
    if target == ScriptTarget::Es5 {
        match body {
            ArrowFunctionBody::Expression(expression) => {
                output.push_str(" { return ");
                output.push_str(&emit_expression(expression, context, target, indentation));
                output.push_str("; }");
            }
            ArrowFunctionBody::Block(statements) => {
                emit_arrow_block_body(statements, context, target, indentation, &mut output)
            }
        }
    } else {
        output.push_str(" => ");
        match body {
            ArrowFunctionBody::Expression(expression) => {
                output.push_str(&emit_expression(expression, context, target, indentation))
            }
            ArrowFunctionBody::Block(statements) => {
                emit_arrow_block_body(statements, context, target, indentation, &mut output)
            }
        }
    }
    output
}

fn emit_arrow_block_body(
    statements: &[FunctionBodyStatement],
    context: &EmitContext,
    target: ScriptTarget,
    indentation: usize,
    output: &mut String,
) {
    output.push_str("{\n");
    for statement in statements {
        emit_function_body_statement(statement, indentation + 1, target, context, output);
    }
    write_indentation(output, indentation);
    output.push('}');
}

fn emit_conditional_expression(
    condition: &Expression,
    when_true: &Expression,
    when_false: &Expression,
    context: &EmitContext,
    target: ScriptTarget,
    indentation: usize,
) -> String {
    format!(
        "{} ? {} : {}",
        emit_expression(condition, context, target, indentation),
        emit_expression(when_true, context, target, indentation),
        emit_expression(when_false, context, target, indentation)
    )
}

fn emit_infix_expression(
    left: &Expression,
    operator: &str,
    right: &Expression,
    context: &EmitContext,
    target: ScriptTarget,
    indentation: usize,
) -> String {
    format!(
        "{} {} {}",
        emit_expression(left, context, target, indentation),
        operator,
        emit_expression(right, context, target, indentation)
    )
}

fn escape_javascript_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}
