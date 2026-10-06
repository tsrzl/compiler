//! The binder walk, modeled on TypeScript-Go's `bind`, `bindContainer`, and `bindChildren`.
//!
//! Each node binds its declaration, records the control flow reaching it, and then binds its
//! children, entering a new scope or control flow graph when it is a container.

use super::binder::Binder;
use super::container_flags::{ContainerFlags, container_flags};
use super::narrowing::is_narrowable_reference;
use crate::ast::{FlowFlags, ModifierFlags, NodeData, NodeFlags, NodeId, SyntaxKind};

impl Binder<'_> {
    pub(super) fn bind(&mut self, node: NodeId) {
        self.record_reference_flow(node);
        self.check_strict_mode_node(node);
        self.bind_declaration(node);
        let ast = self.ast;
        let mut has_error = ast
            .node(node)
            .flags()
            .intersects(NodeFlags::THIS_NODE_HAS_ERROR);
        if ast.node(node).kind() > SyntaxKind::LAST_TOKEN {
            let save_seen_parse_error = self.seen_parse_error;
            self.seen_parse_error = false;
            let flags = container_flags(ast, node);
            if flags == ContainerFlags::NONE {
                self.bind_children(node);
            } else {
                self.bind_container(node, flags);
            }
            has_error |= self.seen_parse_error;
            self.seen_parse_error = save_seen_parse_error;
        }
        if has_error {
            self.add_node_flags(node, NodeFlags::THIS_NODE_OR_ANY_SUB_NODES_HAS_ERROR);
            self.seen_parse_error = true;
        }
    }

    pub(super) fn bind_opt(&mut self, node: Option<NodeId>) {
        if let Some(node) = node {
            self.bind(node);
        }
    }

    pub(super) fn bind_each_child(&mut self, node: NodeId) {
        for child in self.ast.children(node) {
            self.bind(child);
        }
    }

    /// Records the flow reaching references that control flow analysis may narrow.
    fn record_reference_flow(&mut self, node: NodeId) {
        let ast = self.ast;
        match ast.node(node).kind() {
            SyntaxKind::Identifier
            | SyntaxKind::SuperKeyword
            | SyntaxKind::MetaProperty
            | SyntaxKind::BindingElement => {
                self.set_flow_node(node);
            }
            SyntaxKind::ThisKeyword => {
                self.seen_this_keyword = true;
                self.set_flow_node(node);
            }
            SyntaxKind::QualifiedName if is_part_of_type_query(self, node) => {
                self.set_flow_node(node);
            }
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression
                if is_narrowable_reference(ast, node) =>
            {
                self.set_flow_node(node);
            }
            SyntaxKind::ThisType => self.seen_this_keyword = true,
            _ => {}
        }
    }

    pub(super) fn set_flow_node(&mut self, node: NodeId) {
        self.flow_nodes.insert(node, self.current_flow);
    }

    pub(super) fn add_node_flags(&mut self, node: NodeId, flags: NodeFlags) {
        *self.node_flags.entry(node).or_default() |= flags;
    }

    fn remove_node_flags(&mut self, node: NodeId, flags: NodeFlags) {
        if let Some(current) = self.node_flags.get_mut(&node) {
            *current = current.without(flags);
        }
    }

    /// Notes an `async` function outside ambient and declaration contexts, which the emitter
    /// needs to lower.
    pub(super) fn note_async_function(&mut self, node: NodeId) {
        let ast = self.ast;
        let is_async_function = matches!(
            ast.node(node).kind(),
            SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::ArrowFunction
                | SyntaxKind::MethodDeclaration
        ) && ast.node(node).data().body().is_some()
            && ast.node(node).data().asterisk_token().is_none()
            && ast.has_syntactic_modifier(node, ModifierFlags::ASYNC);
        if !self.is_declaration_file()
            && !ast.node(node).flags().intersects(NodeFlags::AMBIENT)
            && is_async_function
        {
            self.emit_flags |= NodeFlags::HAS_ASYNC_FUNCTIONS;
        }
    }

    fn bind_container(&mut self, node: NodeId, flags: ContainerFlags) {
        let save_container = self.container;
        let save_this_container = self.this_container;
        let save_block_scope_container = self.block_scope_container;
        if flags.intersects(ContainerFlags::IS_CONTAINER) {
            self.container = node;
            self.block_scope_container = node;
        } else if flags.intersects(ContainerFlags::IS_BLOCK_SCOPED_CONTAINER) {
            self.block_scope_container = node;
        }
        if flags.intersects(ContainerFlags::IS_THIS_CONTAINER) {
            self.this_container = node;
        }
        if flags.intersects(ContainerFlags::IS_CONTROL_FLOW_CONTAINER) {
            self.bind_control_flow_container(node, flags);
        } else if flags.intersects(ContainerFlags::IS_INTERFACE) {
            let save_seen_this_keyword = self.seen_this_keyword;
            self.seen_this_keyword = false;
            self.bind_children(node);
            if self.seen_this_keyword {
                self.add_node_flags(node, NodeFlags::CONTAINS_THIS);
            } else {
                self.remove_node_flags(node, NodeFlags::CONTAINS_THIS);
            }
            self.seen_this_keyword = save_seen_this_keyword;
        } else {
            self.bind_children(node);
        }
        self.container = save_container;
        self.this_container = save_this_container;
        self.block_scope_container = save_block_scope_container;
    }

    /// Binds a function-like body, source file, or initializer in a fresh control flow graph.
    /// A non-async, non-generator IIFE and a class static block instead stay part of the
    /// enclosing flow, with returns acting like breaks past the body.
    fn bind_control_flow_container(&mut self, node: NodeId, flags: ContainerFlags) {
        let ast = self.ast;
        let save_current_flow = self.current_flow;
        let save_break_target = self.current_break_target;
        let save_continue_target = self.current_continue_target;
        let save_return_target = self.current_return_target;
        let save_exception_target = self.current_exception_target;
        let save_active_labels = std::mem::take(&mut self.active_labels);
        let save_has_explicit_return = self.has_explicit_return;
        let save_seen_this_keyword = self.seen_this_keyword;
        let is_generator_function_expression = ast
            .node(node)
            .data()
            .as_function_expression()
            .is_some_and(|function| function.asterisk_token.is_some());
        let is_immediately_invoked = flags.intersects(ContainerFlags::IS_FUNCTION_EXPRESSION)
            && !ast.has_syntactic_modifier(node, ModifierFlags::ASYNC)
            && !is_generator_function_expression
            && ast.immediately_invoked_function_expression(node).is_some()
            || ast.node(node).kind() == SyntaxKind::ClassStaticBlockDeclaration;
        if !is_immediately_invoked {
            let starts_at_node = flags
                .intersects(ContainerFlags::IS_FUNCTION_EXPRESSION.with(
                    ContainerFlags::IS_OBJECT_LITERAL_OR_CLASS_EXPRESSION_METHOD_OR_ACCESSOR,
                ));
            let payload = if starts_at_node {
                super::flow::FlowPayload::Node(node)
            } else {
                super::flow::FlowPayload::None
            };
            self.current_flow = self.flow.create(FlowFlags::START, payload, None);
        }
        // IIFEs and constructors get a return target: IIFE returns rejoin the enclosing flow, and
        // constructor returns feed strict property initialization checks.
        self.current_return_target = (is_immediately_invoked
            || ast.node(node).kind() == SyntaxKind::Constructor)
            .then(|| self.new_flow_node(FlowFlags::BRANCH_LABEL));
        self.current_exception_target = None;
        self.current_break_target = None;
        self.current_continue_target = None;
        self.has_explicit_return = false;
        self.seen_this_keyword = false;
        self.bind_children(node);
        self.remove_node_flags(
            node,
            NodeFlags::REACHABILITY_AND_EMIT_FLAGS | NodeFlags::CONTAINS_THIS,
        );
        if !self.is_unreachable(self.current_flow)
            && flags.intersects(ContainerFlags::IS_FUNCTION_LIKE)
            && ast
                .node(node)
                .data()
                .body()
                .is_some_and(|body| !ast.node_is_missing(body))
        {
            self.add_node_flags(node, NodeFlags::HAS_IMPLICIT_RETURN);
            if self.has_explicit_return {
                self.add_node_flags(node, NodeFlags::HAS_EXPLICIT_RETURN);
            }
            self.end_flow_nodes.insert(node, self.current_flow);
        }
        if self.seen_this_keyword {
            self.add_node_flags(node, NodeFlags::CONTAINS_THIS);
        }
        if ast.node(node).kind() == SyntaxKind::SourceFile {
            self.add_node_flags(node, self.emit_flags);
            self.end_flow_nodes.insert(node, self.current_flow);
        }
        if let Some(return_target) = self.current_return_target {
            self.add_antecedent(return_target, self.current_flow);
            self.current_flow = self.finish_flow_label(return_target);
            if matches!(
                ast.node(node).kind(),
                SyntaxKind::Constructor | SyntaxKind::ClassStaticBlockDeclaration
            ) {
                self.return_flow_nodes.insert(node, self.current_flow);
            }
        }
        if !is_immediately_invoked {
            self.current_flow = save_current_flow;
        }
        self.current_break_target = save_break_target;
        self.current_continue_target = save_continue_target;
        self.current_return_target = save_return_target;
        self.current_exception_target = save_exception_target;
        self.active_labels = save_active_labels;
        self.has_explicit_return = save_has_explicit_return;
        self.seen_this_keyword = if flags.intersects(ContainerFlags::PROPAGATES_THIS_KEYWORD) {
            save_seen_this_keyword || self.seen_this_keyword
        } else {
            save_seen_this_keyword
        };
    }

    /// Binds children, routing statements and expressions through their control flow binders.
    /// Unreachable code binds its children without flow and is flagged for reporting.
    fn bind_children(&mut self, node: NodeId) {
        let ast = self.ast;
        let save_in_assignment_pattern = self.in_assignment_pattern;
        // Most nodes are not valid in an assignment pattern, so only the nodes that can be part
        // of one restore it below.
        self.in_assignment_pattern = false;
        let kind = ast.node(node).kind();
        if self.current_flow == self.unreachable_flow {
            if has_flow_node_data(kind) {
                self.flow_nodes.remove(&node);
            }
            if ast.is_potentially_executable_node(node) {
                self.add_node_flags(node, NodeFlags::UNREACHABLE);
            }
            self.bind_each_child(node);
            self.in_assignment_pattern = save_in_assignment_pattern;
            return;
        }
        if (SyntaxKind::FIRST_STATEMENT..=SyntaxKind::LAST_STATEMENT).contains(&kind)
            && has_flow_node_data(kind)
        {
            self.set_flow_node(node);
        }
        match kind {
            SyntaxKind::WhileStatement => self.bind_while_statement(node),
            SyntaxKind::DoStatement => self.bind_do_statement(node),
            SyntaxKind::ForStatement => self.bind_for_statement(node),
            SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement => {
                self.bind_for_in_or_of_statement(node);
            }
            SyntaxKind::IfStatement => self.bind_if_statement(node),
            SyntaxKind::ReturnStatement => self.bind_return_statement(node),
            SyntaxKind::ThrowStatement => self.bind_throw_statement(node),
            SyntaxKind::BreakStatement => self.bind_break_or_continue_statement(node, true),
            SyntaxKind::ContinueStatement => self.bind_break_or_continue_statement(node, false),
            SyntaxKind::TryStatement => self.bind_try_statement(node),
            SyntaxKind::SwitchStatement => self.bind_switch_statement(node),
            SyntaxKind::CaseBlock => self.bind_case_block(node),
            SyntaxKind::CaseClause | SyntaxKind::DefaultClause => {
                self.bind_case_or_default_clause(node);
            }
            SyntaxKind::ExpressionStatement => self.bind_expression_statement(node),
            SyntaxKind::LabeledStatement => self.bind_labeled_statement(node),
            SyntaxKind::PrefixUnaryExpression => self.bind_prefix_unary_expression_flow(node),
            SyntaxKind::PostfixUnaryExpression => self.bind_postfix_unary_expression_flow(node),
            SyntaxKind::BinaryExpression if ast.is_destructuring_assignment(node) => {
                // A destructuring assignment carries the pattern context into its initializers.
                self.in_assignment_pattern = save_in_assignment_pattern;
                self.bind_destructuring_assignment_flow(node);
                return;
            }
            SyntaxKind::BinaryExpression => self.bind_binary_expression_flow(node),
            SyntaxKind::DeleteExpression => self.bind_delete_expression_flow(node),
            SyntaxKind::ConditionalExpression => self.bind_conditional_expression_flow(node),
            SyntaxKind::VariableDeclaration => self.bind_variable_declaration_flow(node),
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression => {
                self.bind_access_expression_flow(node);
            }
            SyntaxKind::CallExpression => self.bind_call_expression_flow(node),
            SyntaxKind::NonNullExpression => self.bind_non_null_expression_flow(node),
            SyntaxKind::SourceFile => {
                let NodeData::SourceFile(file) = ast.node(node).data() else {
                    unreachable!("a source file node carries source file data");
                };
                self.bind_each_statement_functions_first(ast.list(file.statements));
                self.bind(file.end_of_file_token);
            }
            SyntaxKind::Block | SyntaxKind::ModuleBlock => {
                let statements = ast
                    .node(node)
                    .data()
                    .statements()
                    .map_or(&[][..], |statements| ast.list(statements));
                self.bind_each_statement_functions_first(statements);
            }
            SyntaxKind::BindingElement => self.bind_binding_element_flow(node),
            SyntaxKind::Parameter => self.bind_parameter_flow(node),
            SyntaxKind::ObjectLiteralExpression
            | SyntaxKind::ArrayLiteralExpression
            | SyntaxKind::PropertyAssignment
            | SyntaxKind::SpreadElement => {
                self.in_assignment_pattern = save_in_assignment_pattern;
                self.bind_each_child(node);
            }
            _ => self.bind_each_child(node),
        }
        self.in_assignment_pattern = save_in_assignment_pattern;
    }

    /// Binds function declarations before other statements so hoisted functions are declared
    /// before the statements that use them.
    fn bind_each_statement_functions_first(&mut self, statements: &[NodeId]) {
        let ast = self.ast;
        let is_function =
            |statement: NodeId| ast.node(statement).kind() == SyntaxKind::FunctionDeclaration;
        for &statement in statements
            .iter()
            .filter(|&&statement| is_function(statement))
        {
            self.bind(statement);
        }
        for &statement in statements
            .iter()
            .filter(|&&statement| !is_function(statement))
        {
            self.bind(statement);
        }
    }
}

fn is_part_of_type_query(binder: &Binder<'_>, mut node: NodeId) -> bool {
    let ast = binder.ast;
    while matches!(
        ast.node(node).kind(),
        SyntaxKind::QualifiedName | SyntaxKind::Identifier
    ) {
        let Some(parent) = ast.node(node).parent() else {
            return false;
        };
        node = parent;
    }
    ast.node(node).kind() == SyntaxKind::TypeQuery
}

/// Returns whether nodes of `kind` record the control flow that reaches them, as TypeScript-Go
/// nodes embedding `FlowNodeBase` do.
const fn has_flow_node_data(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Identifier
            | SyntaxKind::ThisKeyword
            | SyntaxKind::SuperKeyword
            | SyntaxKind::QualifiedName
            | SyntaxKind::MetaProperty
            | SyntaxKind::PropertyAccessExpression
            | SyntaxKind::ElementAccessExpression
            | SyntaxKind::BindingElement
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::Block
            | SyntaxKind::BreakStatement
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::ContinueStatement
            | SyntaxKind::DebuggerStatement
            | SyntaxKind::EmptyStatement
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::ExportAssignment
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::ExpressionStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::ForStatement
            | SyntaxKind::DoStatement
            | SyntaxKind::WhileStatement
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::IfStatement
            | SyntaxKind::ImportDeclaration
            | SyntaxKind::JSImportDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::LabeledStatement
            | SyntaxKind::MissingDeclaration
            | SyntaxKind::ModuleBlock
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::NamespaceExportDeclaration
            | SyntaxKind::NotEmittedStatement
            | SyntaxKind::ReturnStatement
            | SyntaxKind::SwitchStatement
            | SyntaxKind::ThrowStatement
            | SyntaxKind::TryStatement
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::JSTypeAliasDeclaration
            | SyntaxKind::VariableStatement
            | SyntaxKind::WithStatement
    )
}
