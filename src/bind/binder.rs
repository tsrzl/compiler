//! The binder walk: per-kind declaration binding and container tracking.

use std::collections::{HashMap, HashSet};

use super::container_flags::is_object_literal_or_class_expression_method_or_accessor;
use super::flow::FlowPayload;
use super::flow::{FlowGraph, FlowId};
use super::{BoundFile, PatternAmbientModule};
use crate::ast::{
    Ast, FlowFlags, ModifierFlags, NodeData, NodeFlags, NodeId, SymbolFlags, SyntaxKind,
};
use crate::diagnostics::{self, Diagnostic, Message};
use crate::parser::ParsedSourceFile;
use crate::scanner::{LanguageVariant, error_range_for_node, range_of_token_at_position};
use crate::symbols::{
    INTERNAL_SYMBOL_NAME_CLASS, INTERNAL_SYMBOL_NAME_COMPUTED, INTERNAL_SYMBOL_NAME_FUNCTION,
    INTERNAL_SYMBOL_NAME_JSX_ATTRIBUTES, INTERNAL_SYMBOL_NAME_OBJECT, INTERNAL_SYMBOL_NAME_TYPE,
    SymbolArena, SymbolId, SymbolTable,
};
use crate::tspath::remove_file_extension;

/// Where a declaration's symbol is recorded.
#[derive(Debug, Clone, Copy)]
pub(super) enum Table {
    /// The locals of a container node.
    Locals(NodeId),
    /// The members of a class, interface, or literal symbol.
    Members(SymbolId),
    /// The exports of a module, class, or enum symbol.
    Exports(SymbolId),
    /// The UMD globals of the file, declared by `export as namespace`.
    GlobalExports,
}

/// A label in scope for `break` and `continue` statements.
pub(super) struct ActiveLabel {
    pub(super) name: String,
    pub(super) break_target: FlowId,
    pub(super) continue_target: Option<FlowId>,
    pub(super) referenced: bool,
}

/// The mutable state of one binding walk.
#[expect(
    clippy::struct_excessive_bools,
    reason = "the walk flags mirror TypeScript-Go's independent binder state one for one"
)]
pub(super) struct Binder<'file> {
    pub(super) ast: &'file Ast,
    text: &'file str,
    file_name: &'file str,
    language_variant: LanguageVariant,
    pub(super) external_module_indicator: Option<NodeId>,
    pub(super) container: NodeId,
    pub(super) block_scope_container: NodeId,
    pub(super) this_container: NodeId,
    pub(super) symbols: SymbolArena,
    pub(super) node_symbols: HashMap<NodeId, SymbolId>,
    pub(super) local_symbols: HashMap<NodeId, SymbolId>,
    pub(super) locals: HashMap<NodeId, SymbolTable>,
    pub(super) export_contexts: HashSet<NodeId>,
    pub(super) not_const_enum_only_modules: HashSet<SymbolId>,
    pub(super) classifiable_names: HashSet<String>,
    pub(super) global_exports: SymbolTable,
    pub(super) pattern_ambient_modules: Vec<PatternAmbientModule>,
    is_declaration_file: bool,
    diagnostics: Vec<Diagnostic>,
    pub(super) flow: FlowGraph,
    pub(super) unreachable_flow: FlowId,
    pub(super) current_flow: FlowId,
    pub(super) current_break_target: Option<FlowId>,
    pub(super) current_continue_target: Option<FlowId>,
    pub(super) current_return_target: Option<FlowId>,
    pub(super) current_true_target: Option<FlowId>,
    pub(super) current_false_target: Option<FlowId>,
    pub(super) current_exception_target: Option<FlowId>,
    pub(super) pre_switch_case_flow: Option<FlowId>,
    pub(super) active_labels: Vec<ActiveLabel>,
    pub(super) emit_flags: NodeFlags,
    pub(super) seen_this_keyword: bool,
    pub(super) has_explicit_return: bool,
    pub(super) has_flow_effects: bool,
    pub(super) in_assignment_pattern: bool,
    pub(super) seen_parse_error: bool,
    pub(super) flow_nodes: HashMap<NodeId, FlowId>,
    pub(super) end_flow_nodes: HashMap<NodeId, FlowId>,
    pub(super) return_flow_nodes: HashMap<NodeId, FlowId>,
    pub(super) fallthrough_flow_nodes: HashMap<NodeId, FlowId>,
    pub(super) node_flags: HashMap<NodeId, NodeFlags>,
}

impl<'file> Binder<'file> {
    pub(super) fn new(
        file: &'file ParsedSourceFile,
        external_module_indicator: Option<NodeId>,
    ) -> Self {
        let ast = file.ast();
        let mut flow = FlowGraph::default();
        let unreachable_flow = flow.create(FlowFlags::UNREACHABLE, FlowPayload::None, None);
        Self {
            ast,
            text: file.text(),
            file_name: file.options().file_name(),
            language_variant: file.language_variant(),
            external_module_indicator,
            container: ast.root(),
            block_scope_container: ast.root(),
            this_container: ast.root(),
            symbols: SymbolArena::default(),
            node_symbols: HashMap::new(),
            local_symbols: HashMap::new(),
            locals: HashMap::new(),
            export_contexts: HashSet::new(),
            not_const_enum_only_modules: HashSet::new(),
            classifiable_names: HashSet::new(),
            global_exports: SymbolTable::default(),
            pattern_ambient_modules: Vec::new(),
            is_declaration_file: file.is_declaration_file(),
            diagnostics: Vec::new(),
            flow,
            unreachable_flow,
            current_flow: unreachable_flow,
            current_break_target: None,
            current_continue_target: None,
            current_return_target: None,
            current_true_target: None,
            current_false_target: None,
            current_exception_target: None,
            pre_switch_case_flow: None,
            active_labels: Vec::new(),
            emit_flags: NodeFlags::NONE,
            seen_this_keyword: false,
            has_explicit_return: false,
            has_flow_effects: false,
            in_assignment_pattern: false,
            seen_parse_error: false,
            flow_nodes: HashMap::new(),
            end_flow_nodes: HashMap::new(),
            return_flow_nodes: HashMap::new(),
            fallthrough_flow_nodes: HashMap::new(),
            node_flags: HashMap::new(),
        }
    }

    pub(super) fn bind_file(mut self) -> BoundFile {
        self.bind(self.ast.root());
        BoundFile {
            symbols: self.symbols,
            node_symbols: self.node_symbols,
            local_symbols: self.local_symbols,
            locals: self.locals,
            diagnostics: self.diagnostics,
            external_module_indicator: self.external_module_indicator,
            classifiable_names: self.classifiable_names,
            global_exports: self.global_exports,
            pattern_ambient_modules: self.pattern_ambient_modules,
            flow: self.flow,
            flow_nodes: self.flow_nodes,
            end_flow_nodes: self.end_flow_nodes,
            return_flow_nodes: self.return_flow_nodes,
            fallthrough_flow_nodes: self.fallthrough_flow_nodes,
            node_flags: self.node_flags,
        }
    }

    /// Returns whether the file is an external module.
    pub(super) const fn is_external_module(&self) -> bool {
        self.external_module_indicator.is_some()
    }

    /// Binds the declaration `node` introduces, if any, before its children are bound.
    pub(super) fn bind_declaration(&mut self, node: NodeId) {
        self.bind_member_declaration(node);
        self.bind_type_or_value_declaration(node);
        self.bind_module_declaration_syntax(node);
    }

    /// Binds parameters, variables, and class, interface, literal, and enum members.
    fn bind_member_declaration(&mut self, node: NodeId) {
        let ast = self.ast;
        match ast.node(node).kind() {
            SyntaxKind::TypeParameter => self.bind_type_parameter(node),
            SyntaxKind::Parameter => self.bind_parameter(node),
            SyntaxKind::VariableDeclaration | SyntaxKind::BindingElement => {
                self.bind_variable_declaration_or_binding_element(node);
            }
            SyntaxKind::PropertyDeclaration | SyntaxKind::PropertySignature => {
                self.bind_property(node);
            }
            SyntaxKind::PropertyAssignment | SyntaxKind::ShorthandPropertyAssignment => {
                self.bind_property_or_method_or_accessor(
                    node,
                    SymbolFlags::PROPERTY,
                    SymbolFlags::PROPERTY_EXCLUDES,
                );
            }
            SyntaxKind::EnumMember => self.bind_property_or_method_or_accessor(
                node,
                SymbolFlags::ENUM_MEMBER,
                SymbolFlags::ENUM_MEMBER_EXCLUDES,
            ),
            SyntaxKind::CallSignature
            | SyntaxKind::ConstructSignature
            | SyntaxKind::IndexSignature => {
                self.declare_symbol_and_add_to_symbol_table(
                    node,
                    SymbolFlags::SIGNATURE,
                    SymbolFlags::NONE,
                );
            }
            SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature => {
                let is_object_literal_method = ast.node(node).kind()
                    == SyntaxKind::MethodDeclaration
                    && self.parent_kind(node) == Some(SyntaxKind::ObjectLiteralExpression);
                let excludes = if is_object_literal_method {
                    SymbolFlags::VALUE
                } else {
                    SymbolFlags::METHOD_EXCLUDES
                };
                self.bind_property_or_method_or_accessor(
                    node,
                    SymbolFlags::METHOD | self.optional_symbol_flag(node),
                    excludes,
                );
            }
            SyntaxKind::Constructor => {
                self.declare_symbol_and_add_to_symbol_table(
                    node,
                    SymbolFlags::CONSTRUCTOR,
                    SymbolFlags::NONE,
                );
            }
            SyntaxKind::GetAccessor => self.bind_property_or_method_or_accessor(
                node,
                SymbolFlags::GET_ACCESSOR,
                SymbolFlags::GET_ACCESSOR_EXCLUDES,
            ),
            SyntaxKind::SetAccessor => self.bind_property_or_method_or_accessor(
                node,
                SymbolFlags::SET_ACCESSOR,
                SymbolFlags::SET_ACCESSOR_EXCLUDES,
            ),
            _ => {}
        }
    }

    /// Binds functions, classes, interfaces, type aliases, enums, and type or object literals.
    fn bind_type_or_value_declaration(&mut self, node: NodeId) {
        let ast = self.ast;
        match ast.node(node).kind() {
            SyntaxKind::FunctionDeclaration => {
                self.note_async_function(node);
                self.bind_block_scoped_declaration(
                    node,
                    SymbolFlags::FUNCTION,
                    SymbolFlags::FUNCTION_EXCLUDES,
                );
            }
            SyntaxKind::FunctionType | SyntaxKind::ConstructorType => {
                self.bind_function_or_constructor_type(node);
            }
            SyntaxKind::TypeLiteral | SyntaxKind::MappedType => {
                self.bind_anonymous_declaration(
                    node,
                    SymbolFlags::TYPE_LITERAL,
                    INTERNAL_SYMBOL_NAME_TYPE,
                );
            }
            SyntaxKind::ObjectLiteralExpression => self.bind_anonymous_declaration(
                node,
                SymbolFlags::OBJECT_LITERAL,
                INTERNAL_SYMBOL_NAME_OBJECT,
            ),
            SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction => {
                self.bind_function_expression(node);
            }
            SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => {
                self.bind_class_like_declaration(node);
            }
            SyntaxKind::InterfaceDeclaration => self.bind_block_scoped_declaration(
                node,
                SymbolFlags::INTERFACE,
                SymbolFlags::INTERFACE_EXCLUDES,
            ),
            SyntaxKind::TypeAliasDeclaration => self.bind_block_scoped_declaration(
                node,
                SymbolFlags::TYPE_ALIAS,
                SymbolFlags::TYPE_ALIAS_EXCLUDES,
            ),
            SyntaxKind::EnumDeclaration => {
                let (includes, excludes) = if ast
                    .combined_modifier_flags(node)
                    .intersects(ModifierFlags::CONST)
                {
                    (SymbolFlags::CONST_ENUM, SymbolFlags::CONST_ENUM_EXCLUDES)
                } else {
                    (
                        SymbolFlags::REGULAR_ENUM,
                        SymbolFlags::REGULAR_ENUM_EXCLUDES,
                    )
                };
                self.bind_block_scoped_declaration(node, includes, excludes);
            }
            _ => {}
        }
    }

    /// Binds imports, exports, the external-module symbol, and JSX attributes.
    fn bind_module_declaration_syntax(&mut self, node: NodeId) {
        let ast = self.ast;
        match ast.node(node).kind() {
            SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::NamespaceImport
            | SyntaxKind::ImportSpecifier
            | SyntaxKind::ExportSpecifier => {
                self.declare_symbol_and_add_to_symbol_table(
                    node,
                    SymbolFlags::ALIAS,
                    SymbolFlags::ALIAS_EXCLUDES,
                );
            }
            SyntaxKind::ImportClause => {
                if ast.node(node).data().name().is_some() {
                    self.declare_symbol_and_add_to_symbol_table(
                        node,
                        SymbolFlags::ALIAS,
                        SymbolFlags::ALIAS_EXCLUDES,
                    );
                }
            }
            SyntaxKind::ModuleDeclaration => self.bind_module_declaration(node),
            SyntaxKind::NamespaceExportDeclaration => self.bind_namespace_export_declaration(node),
            SyntaxKind::ExportDeclaration => self.bind_export_declaration(node),
            SyntaxKind::ExportAssignment => self.bind_export_assignment(node),
            SyntaxKind::SourceFile => self.bind_source_file_if_external_module(),
            SyntaxKind::JsxAttributes => self.bind_anonymous_declaration(
                node,
                SymbolFlags::OBJECT_LITERAL,
                INTERNAL_SYMBOL_NAME_JSX_ATTRIBUTES,
            ),
            SyntaxKind::JsxAttribute => {
                self.declare_symbol_and_add_to_symbol_table(
                    node,
                    SymbolFlags::PROPERTY,
                    SymbolFlags::PROPERTY_EXCLUDES,
                );
            }

            _ => {}
        }
    }

    fn bind_source_file_if_external_module(&mut self) {
        let root = self.ast.root();
        self.set_export_context_flag(root);
        if self.is_external_module() {
            let name = format!("\"{}\"", remove_file_extension(self.file_name));
            self.bind_anonymous_declaration(root, SymbolFlags::VALUE_MODULE, &name);
        }
    }

    /// Records whether an ambient file or module without export declarations implicitly exports
    /// its declarations.
    pub(super) fn set_export_context_flag(&mut self, node: NodeId) {
        if self.ast.node(node).flags().intersects(NodeFlags::AMBIENT)
            && !self.has_export_declarations(node)
        {
            self.export_contexts.insert(node);
        } else {
            self.export_contexts.remove(&node);
        }
    }

    fn has_export_declarations(&self, node: NodeId) -> bool {
        let ast = self.ast;
        let statements = match ast.node(node).data() {
            NodeData::SourceFile(file) => Some(file.statements),
            NodeData::ModuleDeclaration(module) => module.body.and_then(|body| {
                ast.node(body)
                    .data()
                    .as_module_block()
                    .map(|block| block.statements)
            }),
            _ => None,
        };
        statements.is_some_and(|statements| {
            ast.list(statements).iter().any(|&statement| {
                matches!(
                    ast.node(statement).kind(),
                    SyntaxKind::ExportDeclaration | SyntaxKind::ExportAssignment
                )
            })
        })
    }

    fn bind_type_parameter(&mut self, node: NodeId) {
        if self.parent_kind(node) == Some(SyntaxKind::InferType) {
            if let Some(container) = self.infer_type_container(node) {
                self.declare_symbol(
                    Table::Locals(container),
                    None,
                    node,
                    SymbolFlags::TYPE_PARAMETER,
                    SymbolFlags::TYPE_PARAMETER_EXCLUDES,
                );
            } else {
                let name = self.declaration_name(node);
                self.bind_anonymous_declaration(node, SymbolFlags::TYPE_PARAMETER, &name);
            }
        } else {
            self.declare_symbol_and_add_to_symbol_table(
                node,
                SymbolFlags::TYPE_PARAMETER,
                SymbolFlags::TYPE_PARAMETER_EXCLUDES,
            );
        }
    }

    /// Returns the conditional type whose `extends` clause contains `node`.
    fn infer_type_container(&self, node: NodeId) -> Option<NodeId> {
        let ast = self.ast;
        let mut current = Some(node);
        while let Some(id) = current {
            let parent = ast.node(id).parent()?;
            if ast
                .node(parent)
                .data()
                .as_conditional_type_node()
                .is_some_and(|conditional| conditional.extends_type == id)
            {
                return Some(parent);
            }
            current = Some(parent);
        }
        None
    }

    fn bind_parameter(&mut self, node: NodeId) {
        let ast = self.ast;
        let NodeData::ParameterDeclaration(parameter) = ast.node(node).data() else {
            unreachable!("a parameter node carries parameter data");
        };
        let parent = ast
            .node(node)
            .parent()
            .expect("a parameter has a parent signature");
        if is_binding_pattern(ast, parameter.name) {
            let index = ast
                .node(parent)
                .data()
                .parameters()
                .and_then(|parameters| ast.list(parameters).iter().position(|&id| id == node))
                .expect("a parameter is listed by its parent");
            let name = format!("__{index}");
            self.bind_anonymous_declaration(node, SymbolFlags::FUNCTION_SCOPED_VARIABLE, &name);
        } else {
            self.declare_symbol_and_add_to_symbol_table(
                node,
                SymbolFlags::FUNCTION_SCOPED_VARIABLE,
                SymbolFlags::PARAMETER_EXCLUDES,
            );
        }
        if ast.is_parameter_property_declaration(node, parent) {
            let class = ast
                .node(parent)
                .parent()
                .expect("a constructor belongs to a class");
            let class_symbol = self
                .symbol_of(class)
                .expect("a class is bound before its members");
            let optional = if parameter.question_token.is_some() {
                SymbolFlags::OPTIONAL
            } else {
                SymbolFlags::NONE
            };
            self.declare_symbol(
                Table::Members(class_symbol),
                Some(class_symbol),
                node,
                SymbolFlags::PROPERTY | optional,
                SymbolFlags::PROPERTY_EXCLUDES,
            );
        }
    }

    fn bind_variable_declaration_or_binding_element(&mut self, node: NodeId) {
        let ast = self.ast;
        let Some(name) = ast.node(node).data().name() else {
            return;
        };
        if is_binding_pattern(ast, name) {
            return;
        }
        if ast.is_block_or_catch_scoped(node) {
            self.bind_block_scoped_declaration(
                node,
                SymbolFlags::BLOCK_SCOPED_VARIABLE,
                SymbolFlags::BLOCK_SCOPED_VARIABLE_EXCLUDES,
            );
        } else if ast.is_part_of_parameter_declaration(node) {
            self.declare_symbol_and_add_to_symbol_table(
                node,
                SymbolFlags::FUNCTION_SCOPED_VARIABLE,
                SymbolFlags::PARAMETER_EXCLUDES,
            );
        } else {
            self.declare_symbol_and_add_to_symbol_table(
                node,
                SymbolFlags::FUNCTION_SCOPED_VARIABLE,
                SymbolFlags::FUNCTION_SCOPED_VARIABLE_EXCLUDES,
            );
        }
    }

    fn bind_property(&mut self, node: NodeId) {
        let is_auto_accessor = self.ast.node(node).kind() == SyntaxKind::PropertyDeclaration
            && self
                .ast
                .has_syntactic_modifier(node, ModifierFlags::ACCESSOR);
        let (includes, excludes) = if is_auto_accessor {
            (SymbolFlags::ACCESSOR, SymbolFlags::ACCESSOR_EXCLUDES)
        } else {
            (SymbolFlags::PROPERTY, SymbolFlags::PROPERTY_EXCLUDES)
        };
        self.bind_property_or_method_or_accessor(
            node,
            includes | self.optional_symbol_flag(node),
            excludes,
        );
    }

    fn bind_property_or_method_or_accessor(
        &mut self,
        node: NodeId,
        includes: SymbolFlags,
        excludes: SymbolFlags,
    ) {
        self.note_async_function(node);
        if is_object_literal_or_class_expression_method_or_accessor(self.ast, node) {
            self.set_flow_node(node);
        }
        if self.ast.has_dynamic_name(node) {
            self.bind_anonymous_declaration(node, includes, INTERNAL_SYMBOL_NAME_COMPUTED);
        } else {
            self.declare_symbol_and_add_to_symbol_table(node, includes, excludes);
        }
    }

    /// Binds a function or constructor type as a type literal whose only member is the
    /// signature, matching the symbol of the equivalent `{ (...): T }` form.
    fn bind_function_or_constructor_type(&mut self, node: NodeId) {
        let name = self.declaration_name(node);
        let signature = self.symbols.create(SymbolFlags::SIGNATURE, name.clone());
        self.add_declaration_to_symbol(signature, node, SymbolFlags::SIGNATURE);
        let type_literal = self
            .symbols
            .create(SymbolFlags::TYPE_LITERAL, INTERNAL_SYMBOL_NAME_TYPE);
        self.add_declaration_to_symbol(type_literal, node, SymbolFlags::TYPE_LITERAL);
        self.symbols
            .symbol_mut(type_literal)
            .members
            .insert(name, signature);
    }

    fn bind_function_expression(&mut self, node: NodeId) {
        let ast = self.ast;
        self.note_async_function(node);
        self.set_flow_node(node);
        let name = ast
            .node(node)
            .data()
            .as_function_expression()
            .and_then(|function| function.name)
            .and_then(|name| ast.node_text(name))
            .unwrap_or(INTERNAL_SYMBOL_NAME_FUNCTION);
        self.bind_anonymous_declaration(node, SymbolFlags::FUNCTION, name);
    }

    fn bind_class_like_declaration(&mut self, node: NodeId) {
        let ast = self.ast;
        if ast.node(node).kind() == SyntaxKind::ClassDeclaration {
            self.bind_block_scoped_declaration(
                node,
                SymbolFlags::CLASS,
                SymbolFlags::CLASS_EXCLUDES,
            );
        } else {
            let name = ast
                .node(node)
                .data()
                .name()
                .and_then(|name| ast.node_text(name));
            if let Some(name) = name {
                self.classifiable_names.insert(name.to_owned());
            }
            self.bind_anonymous_declaration(
                node,
                SymbolFlags::CLASS,
                name.unwrap_or(INTERNAL_SYMBOL_NAME_CLASS),
            );
        }
        let Some(class) = self.symbol_of(node) else {
            return;
        };
        // Every class has a static `prototype` property, so an explicit static `prototype`
        // member, including one merged from a namespace export, is a duplicate.
        let prototype = self
            .symbols
            .create(SymbolFlags::PROPERTY | SymbolFlags::PROTOTYPE, "prototype");
        if let Some(existing) = self.symbols.symbol(class).exports.get("prototype") {
            let declaration = self.symbols.symbol(existing).declarations[0];
            self.error_on_node(
                declaration,
                diagnostics::DUPLICATE_IDENTIFIER_0,
                &["prototype"],
            );
        }
        self.symbols
            .symbol_mut(class)
            .exports
            .insert("prototype", prototype);
        self.symbols.symbol_mut(prototype).parent = Some(class);
    }

    fn bind_export_declaration(&mut self, node: NodeId) {
        let ast = self.ast;
        let NodeData::ExportDeclaration(declaration) = ast.node(node).data() else {
            unreachable!("an export declaration node carries export declaration data");
        };
        let Some(container) = self.symbol_of(self.container) else {
            let name = self.declaration_name(node);
            self.bind_anonymous_declaration(node, SymbolFlags::EXPORT_STAR, &name);
            return;
        };
        match declaration.export_clause {
            None => {
                self.declare_symbol(
                    Table::Exports(container),
                    Some(container),
                    node,
                    SymbolFlags::EXPORT_STAR,
                    SymbolFlags::NONE,
                );
            }
            Some(clause) if ast.node(clause).kind() == SyntaxKind::NamespaceExport => {
                self.declare_symbol(
                    Table::Exports(container),
                    Some(container),
                    clause,
                    SymbolFlags::ALIAS,
                    SymbolFlags::ALIAS_EXCLUDES,
                );
            }
            Some(_) => {}
        }
    }

    fn bind_export_assignment(&mut self, node: NodeId) {
        let ast = self.ast;
        let NodeData::ExportAssignment(assignment) = ast.node(node).data() else {
            unreachable!("an export assignment node carries export assignment data");
        };
        let Some(container) = self.symbol_of(self.container) else {
            let name = self.declaration_name(node);
            self.bind_anonymous_declaration(node, SymbolFlags::VALUE, &name);
            return;
        };
        // An `export default x;` alias cannot be combined with any other default export, unlike
        // `export default function f() {}`.
        let flags = if is_expression_alias(ast, assignment.expression) {
            SymbolFlags::ALIAS
        } else {
            SymbolFlags::PROPERTY
        };
        let symbol = self.declare_symbol(
            Table::Exports(container),
            Some(container),
            node,
            flags,
            SymbolFlags::ALL,
        );
        if assignment.is_export_equals {
            self.set_value_declaration(symbol, node);
        }
    }

    pub(super) fn bind_anonymous_declaration(
        &mut self,
        node: NodeId,
        flags: SymbolFlags,
        name: &str,
    ) {
        let symbol = self.symbols.create(flags, name);
        if flags.intersects(SymbolFlags::ENUM_MEMBER | SymbolFlags::CLASS_MEMBER) {
            self.symbols.symbol_mut(symbol).parent = self.symbol_of(self.container);
        }
        self.add_declaration_to_symbol(symbol, node, flags);
    }

    fn optional_symbol_flag(&self, node: NodeId) -> SymbolFlags {
        let ast = self.ast;
        let postfix = ast.node(node).data().postfix_token();
        if postfix.is_some_and(|token| ast.node(token).kind() == SyntaxKind::QuestionToken) {
            SymbolFlags::OPTIONAL
        } else {
            SymbolFlags::NONE
        }
    }

    pub(super) fn symbol_of(&self, node: NodeId) -> Option<SymbolId> {
        self.node_symbols.get(&node).copied()
    }

    pub(super) fn parent_kind(&self, node: NodeId) -> Option<SyntaxKind> {
        self.ast
            .node(node)
            .parent()
            .map(|parent| self.ast.node(parent).kind())
    }

    /// Returns the source text of a declaration name, without leading trivia.
    pub(super) fn declaration_name_text(&self, name: NodeId) -> &'file str {
        let range = error_range_for_node(self.ast, self.text, self.language_variant, name);
        &self.text[range]
    }

    pub(super) fn create_diagnostic_for_node(
        &self,
        node: NodeId,
        message: Message,
        arguments: &[&str],
    ) -> Diagnostic {
        Diagnostic::new(
            message,
            error_range_for_node(self.ast, self.text, self.language_variant, node),
            arguments,
        )
    }

    pub(super) fn error_on_node(&mut self, node: NodeId, message: Message, arguments: &[&str]) {
        let diagnostic = self.create_diagnostic_for_node(node, message, arguments);
        self.diagnostics.push(diagnostic);
    }

    #[allow(
        dead_code,
        reason = "used once namespace declaration diagnostics are ported"
    )]
    pub(super) fn error_on_first_token(
        &mut self,
        node: NodeId,
        message: Message,
        arguments: &[&str],
    ) {
        let range = range_of_token_at_position(
            self.text,
            self.language_variant,
            self.ast.node(node).pos() as usize,
        );
        self.diagnostics
            .push(Diagnostic::new(message, range, arguments));
    }

    pub(super) const fn is_declaration_file(&self) -> bool {
        self.is_declaration_file
    }

    pub(super) fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }
}

fn is_binding_pattern(ast: &Ast, node: NodeId) -> bool {
    matches!(
        ast.node(node).kind(),
        SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern
    )
}

/// Returns whether an exported expression names an entity or is a class expression.
fn is_expression_alias(ast: &Ast, node: NodeId) -> bool {
    ast.is_entity_name_expression(node) || ast.node(node).kind() == SyntaxKind::ClassExpression
}
