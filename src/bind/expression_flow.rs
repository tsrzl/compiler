//! Expression control flow, modeled on TypeScript-Go's `bind*Flow` functions.

use super::binder::Binder;
use super::flow::FlowId;
use super::narrowing::{is_narrowable_operand, is_narrowable_reference};
use crate::ast::{FlowFlags, NodeData, NodeId, SyntaxKind};

impl Binder<'_> {
    /// Records assignments to every reference an assignment target writes, descending through
    /// array and object destructuring patterns.
    pub(super) fn bind_assignment_target_flow(&mut self, node: NodeId) {
        let ast = self.ast;
        match ast.node(node).data() {
            NodeData::ArrayLiteralExpression(array) => {
                for &element in ast.list(array.elements) {
                    if let NodeData::SpreadElement(spread) = ast.node(element).data() {
                        self.bind_assignment_target_flow(spread.expression);
                    } else {
                        self.bind_destructuring_target_flow(element);
                    }
                }
            }
            NodeData::ObjectLiteralExpression(object) => {
                for &property in ast.list(object.properties) {
                    match ast.node(property).data() {
                        NodeData::PropertyAssignment(assignment) => {
                            self.bind_destructuring_target_flow(assignment.initializer);
                        }
                        NodeData::ShorthandPropertyAssignment(shorthand) => {
                            self.bind_assignment_target_flow(shorthand.name);
                        }
                        NodeData::SpreadAssignment(spread) => {
                            self.bind_assignment_target_flow(spread.expression);
                        }
                        _ => {}
                    }
                }
            }
            _ => {
                if is_narrowable_reference(ast, node) {
                    self.current_flow =
                        self.create_flow_mutation(FlowFlags::ASSIGNMENT, self.current_flow, node);
                }
            }
        }
    }

    fn bind_destructuring_target_flow(&mut self, node: NodeId) {
        let ast = self.ast;
        match ast.node(node).data().as_binary_expression() {
            Some(binary) if ast.node(binary.operator_token).kind() == SyntaxKind::EqualsToken => {
                self.bind_assignment_target_flow(binary.left);
            }
            _ => self.bind_assignment_target_flow(node),
        }
    }

    pub(super) fn bind_prefix_unary_expression_flow(&mut self, node: NodeId) {
        let ast = self.ast;
        let NodeData::PrefixUnaryExpression(unary) = ast.node(node).data() else {
            unreachable!("a prefix unary node carries prefix unary data");
        };
        if unary.operator == SyntaxKind::ExclamationToken {
            // `!` swaps the true and false branches of the enclosing condition.
            std::mem::swap(
                &mut self.current_true_target,
                &mut self.current_false_target,
            );
            self.bind_each_child(node);
            std::mem::swap(
                &mut self.current_true_target,
                &mut self.current_false_target,
            );
        } else {
            self.bind_each_child(node);
            if matches!(
                unary.operator,
                SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
            ) {
                self.bind_assignment_target_flow(unary.operand);
            }
        }
    }

    pub(super) fn bind_postfix_unary_expression_flow(&mut self, node: NodeId) {
        let NodeData::PostfixUnaryExpression(unary) = self.ast.node(node).data() else {
            unreachable!("a postfix unary node carries postfix unary data");
        };
        self.bind_each_child(node);
        if matches!(
            unary.operator,
            SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
        ) {
            self.bind_assignment_target_flow(unary.operand);
        }
    }

    /// Binds a destructuring assignment, evaluating the right side before the pattern unless it
    /// is nested in an outer pattern.
    pub(super) fn bind_destructuring_assignment_flow(&mut self, node: NodeId) {
        let NodeData::BinaryExpression(binary) = self.ast.node(node).data() else {
            unreachable!("a destructuring assignment is a binary expression");
        };
        if self.in_assignment_pattern {
            self.in_assignment_pattern = false;
            self.bind(binary.operator_token);
            self.bind(binary.right);
            self.in_assignment_pattern = true;
            self.bind(binary.left);
            self.bind_opt(binary.type_node);
        } else {
            self.in_assignment_pattern = true;
            self.bind(binary.left);
            self.bind_opt(binary.type_node);
            self.in_assignment_pattern = false;
            self.bind(binary.operator_token);
            self.bind(binary.right);
        }
        self.bind_assignment_target_flow(binary.left);
    }

    pub(super) fn bind_binary_expression_flow(&mut self, node: NodeId) {
        let ast = self.ast;
        let NodeData::BinaryExpression(binary) = ast.node(node).data() else {
            unreachable!("a binary expression node carries binary data");
        };
        let operator = ast.node(binary.operator_token).kind();
        let is_logical = matches!(
            operator,
            SyntaxKind::BarBarToken
                | SyntaxKind::AmpersandAmpersandToken
                | SyntaxKind::QuestionQuestionToken
        ) || operator.is_logical_or_coalescing_assignment_operator();
        if is_logical {
            if self.is_top_level_logical_expression(node) {
                let post_expression = self.create_branch_label();
                self.bind_with_own_flow_effects(|binder| {
                    binder.bind_logical_like_expression(node, post_expression, post_expression);
                    post_expression
                });
            } else {
                let (true_target, false_target) = self.conditional_targets();
                self.bind_logical_like_expression(node, true_target, false_target);
            }
            return;
        }
        self.bind(binary.left);
        self.bind_opt(binary.type_node);
        if operator == SyntaxKind::CommaToken {
            self.maybe_bind_expression_flow_if_call(binary.left);
        }
        self.bind(binary.operator_token);
        self.bind(binary.right);
        if operator == SyntaxKind::CommaToken {
            self.maybe_bind_expression_flow_if_call(binary.right);
        }
        if operator.is_assignment_operator() && !ast.is_assignment_target(node) {
            self.bind_assignment_target_flow(binary.left);
            if operator == SyntaxKind::EqualsToken
                && let Some(access) = ast.node(binary.left).data().as_element_access_expression()
                && is_narrowable_operand(ast, access.expression)
            {
                self.current_flow =
                    self.create_flow_mutation(FlowFlags::ARRAY_MUTATION, self.current_flow, node);
            }
        }
    }

    /// Binds an expression whose branches rejoin at `post_label`, keeping the flow before it
    /// when the expression has no flow effects of its own.
    fn bind_with_own_flow_effects(&mut self, bind: impl FnOnce(&mut Self) -> FlowId) {
        let save_current_flow = self.current_flow;
        let save_has_flow_effects = self.has_flow_effects;
        self.has_flow_effects = false;
        let post_label = bind(self);
        self.current_flow = if self.has_flow_effects {
            self.finish_flow_label(post_label)
        } else {
            save_current_flow
        };
        self.has_flow_effects = self.has_flow_effects || save_has_flow_effects;
    }

    /// Returns the enclosing condition's branch labels, which exist whenever a logical
    /// expression or optional chain is nested in a condition rather than at the top level.
    fn conditional_targets(&self) -> (FlowId, FlowId) {
        (
            self.current_true_target
                .expect("a nested logical expression is bound inside a condition"),
            self.current_false_target
                .expect("a nested logical expression is bound inside a condition"),
        )
    }

    /// Returns whether a logical expression or optional chain is outside any condition, so it
    /// routes its own branches.
    fn is_top_level_logical_expression(&self, mut node: NodeId) -> bool {
        let ast = self.ast;
        while let Some(parent) = ast.node(node).parent() {
            let negation = ast
                .node(parent)
                .data()
                .as_prefix_unary_expression()
                .is_some_and(|unary| unary.operator == SyntaxKind::ExclamationToken);
            if ast.node(parent).kind() != SyntaxKind::ParenthesizedExpression && !negation {
                break;
            }
            node = parent;
        }
        let parent = ast.node(node).parent().expect("an expression has a parent");
        !self.is_statement_condition(node)
            && !ast.is_logical_expression(parent)
            && (!ast.is_optional_chain(parent)
                || ast.node(parent).data().expression() != Some(node))
    }

    fn is_statement_condition(&self, node: NodeId) -> bool {
        let ast = self.ast;
        let parent = ast.node(node).parent().expect("an expression has a parent");
        match ast.node(parent).data() {
            NodeData::IfStatement(_) | NodeData::WhileStatement(_) | NodeData::DoStatement(_) => {
                ast.node(parent).data().expression() == Some(node)
            }
            NodeData::ForStatement(statement) => statement.condition == Some(node),
            NodeData::ConditionalExpression(conditional) => conditional.condition == node,
            _ => false,
        }
    }

    fn bind_logical_like_expression(
        &mut self,
        node: NodeId,
        true_target: FlowId,
        false_target: FlowId,
    ) {
        let ast = self.ast;
        let NodeData::BinaryExpression(binary) = ast.node(node).data() else {
            unreachable!("a logical expression is a binary expression");
        };
        let operator = ast.node(binary.operator_token).kind();
        let pre_right = self.create_branch_label();
        if matches!(
            operator,
            SyntaxKind::AmpersandAmpersandToken | SyntaxKind::AmpersandAmpersandEqualsToken
        ) {
            self.bind_condition(Some(binary.left), pre_right, false_target);
        } else {
            self.bind_condition(Some(binary.left), true_target, pre_right);
        }
        self.current_flow = self.finish_flow_label(pre_right);
        self.bind(binary.operator_token);
        if operator.is_logical_or_coalescing_assignment_operator() {
            self.bind_with_conditional_branches(
                Some(binary.right),
                Some(true_target),
                Some(false_target),
            );
            self.bind_assignment_target_flow(binary.left);
            self.add_condition_antecedents(node, true_target, false_target);
        } else {
            self.bind_condition(Some(binary.right), true_target, false_target);
        }
    }

    fn add_condition_antecedents(
        &mut self,
        node: NodeId,
        true_target: FlowId,
        false_target: FlowId,
    ) {
        let when_true =
            self.create_flow_condition(FlowFlags::TRUE_CONDITION, self.current_flow, Some(node));
        self.add_antecedent(true_target, when_true);
        let when_false =
            self.create_flow_condition(FlowFlags::FALSE_CONDITION, self.current_flow, Some(node));
        self.add_antecedent(false_target, when_false);
    }

    pub(super) fn bind_delete_expression_flow(&mut self, node: NodeId) {
        let ast = self.ast;
        self.bind_each_child(node);
        if let Some(expression) = ast.node(node).data().expression()
            && ast.node(expression).kind() == SyntaxKind::PropertyAccessExpression
        {
            self.bind_assignment_target_flow(expression);
        }
    }

    pub(super) fn bind_conditional_expression_flow(&mut self, node: NodeId) {
        let NodeData::ConditionalExpression(conditional) = self.ast.node(node).data() else {
            unreachable!("a conditional expression node carries conditional data");
        };
        let true_label = self.create_branch_label();
        let false_label = self.create_branch_label();
        let post_expression = self.create_branch_label();
        self.bind_with_own_flow_effects(|binder| {
            binder.bind_condition(Some(conditional.condition), true_label, false_label);
            binder.current_flow = binder.finish_flow_label(true_label);
            binder.bind(conditional.question_token);
            binder.bind(conditional.when_true);
            binder.add_antecedent(post_expression, binder.current_flow);
            binder.current_flow = binder.finish_flow_label(false_label);
            binder.bind(conditional.colon_token);
            binder.bind(conditional.when_false);
            binder.add_antecedent(post_expression, binder.current_flow);
            post_expression
        });
    }

    pub(super) fn bind_variable_declaration_flow(&mut self, node: NodeId) {
        let ast = self.ast;
        self.bind_each_child(node);
        let in_for_in_or_of = ast
            .node(node)
            .parent()
            .and_then(|list| ast.node(list).parent())
            .is_some_and(|loop_statement| {
                matches!(
                    ast.node(loop_statement).kind(),
                    SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement
                )
            });
        if ast.node(node).data().initializer().is_some() || in_for_in_or_of {
            self.bind_initialized_variable_flow(node);
        }
    }

    fn bind_initialized_variable_flow(&mut self, node: NodeId) {
        let ast = self.ast;
        let name = match ast.node(node).data() {
            NodeData::VariableDeclaration(declaration) => Some(declaration.name),
            NodeData::BindingElement(element) => element.name,
            _ => None,
        };
        if let Some(pattern) = name.and_then(|name| ast.node(name).data().as_binding_pattern()) {
            for &element in ast.list(pattern.elements) {
                self.bind_initialized_variable_flow(element);
            }
        } else {
            self.current_flow =
                self.create_flow_mutation(FlowFlags::ASSIGNMENT, self.current_flow, node);
        }
    }

    pub(super) fn bind_access_expression_flow(&mut self, node: NodeId) {
        if self.ast.is_optional_chain(node) {
            self.bind_optional_chain_flow(node);
        } else {
            self.bind_each_child(node);
        }
    }

    fn bind_optional_chain_flow(&mut self, node: NodeId) {
        if self.is_top_level_logical_expression(node) {
            let post_expression = self.create_branch_label();
            let save_current_flow = self.current_flow;
            let save_has_flow_effects = self.has_flow_effects;
            self.bind_optional_chain(node, post_expression, post_expression);
            self.current_flow = if self.has_flow_effects {
                self.finish_flow_label(post_expression)
            } else {
                save_current_flow
            };
            self.has_flow_effects = self.has_flow_effects || save_has_flow_effects;
        } else {
            let (true_target, false_target) = self.conditional_targets();
            self.bind_optional_chain(node, true_target, false_target);
        }
    }

    /// Binds an optional chain like the logical expression it abbreviates: `a?.b` behaves as
    /// `a && a.b`, with the rest of the chain bound on the true branch.
    fn bind_optional_chain(&mut self, node: NodeId, true_target: FlowId, false_target: FlowId) {
        let ast = self.ast;
        let pre_chain = ast
            .is_optional_chain_root(node)
            .then(|| self.create_branch_label());
        let expression = ast
            .node(node)
            .data()
            .expression()
            .expect("an optional chain has an expression");
        self.bind_optional_expression(expression, pre_chain.unwrap_or(true_target), false_target);
        if let Some(pre_chain) = pre_chain {
            self.current_flow = self.finish_flow_label(pre_chain);
        }
        let saved_true = self.current_true_target;
        let saved_false = self.current_false_target;
        self.current_true_target = Some(true_target);
        self.current_false_target = Some(false_target);
        self.bind_optional_chain_rest(node);
        self.current_true_target = saved_true;
        self.current_false_target = saved_false;
        if ast.is_outermost_optional_chain(node) {
            self.add_condition_antecedents(node, true_target, false_target);
        }
    }

    fn bind_optional_expression(
        &mut self,
        node: NodeId,
        true_target: FlowId,
        false_target: FlowId,
    ) {
        let ast = self.ast;
        self.bind_with_conditional_branches(Some(node), Some(true_target), Some(false_target));
        if !ast.is_optional_chain(node) || ast.is_outermost_optional_chain(node) {
            self.add_condition_antecedents(node, true_target, false_target);
        }
    }

    fn bind_optional_chain_rest(&mut self, node: NodeId) {
        let ast = self.ast;
        match ast.node(node).data() {
            NodeData::PropertyAccessExpression(access) => {
                self.bind_opt(access.question_dot_token);
                self.bind(access.name);
            }
            NodeData::ElementAccessExpression(access) => {
                self.bind_opt(access.question_dot_token);
                self.bind(access.argument_expression);
            }
            NodeData::CallExpression(call) => {
                self.bind_opt(call.question_dot_token);
                if let Some(type_arguments) = call.type_arguments {
                    for &argument in ast.list(type_arguments) {
                        self.bind(argument);
                    }
                }
                for &argument in ast.list(call.arguments) {
                    self.bind(argument);
                }
            }
            _ => {}
        }
    }

    pub(super) fn bind_call_expression_flow(&mut self, node: NodeId) {
        let ast = self.ast;
        let NodeData::CallExpression(call) = ast.node(node).data() else {
            unreachable!("a call expression node carries call data");
        };
        if ast.is_optional_chain(node) {
            self.bind_optional_chain_flow(node);
        } else {
            let callee = ast.skip_parentheses(call.expression);
            if matches!(
                ast.node(callee).kind(),
                SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction
            ) {
                // An immediately invoked function starts its flow after its arguments evaluate.
                if let Some(type_arguments) = call.type_arguments {
                    for &argument in ast.list(type_arguments) {
                        self.bind(argument);
                    }
                }
                for &argument in ast.list(call.arguments) {
                    self.bind(argument);
                }
                self.bind(call.expression);
            } else {
                self.bind_each_child(node);
                if ast.node(call.expression).kind() == SyntaxKind::SuperKeyword {
                    self.current_flow = self.create_flow_call(self.current_flow, node);
                }
            }
        }
        if let Some(access) = ast
            .node(call.expression)
            .data()
            .as_property_access_expression()
            && ast
                .identifier_text(access.name)
                .is_some_and(|name| name == "push" || name == "unshift")
            && is_narrowable_operand(ast, access.expression)
        {
            self.current_flow =
                self.create_flow_mutation(FlowFlags::ARRAY_MUTATION, self.current_flow, node);
        }
    }

    pub(super) fn bind_non_null_expression_flow(&mut self, node: NodeId) {
        if self.ast.is_optional_chain(node) {
            self.bind_optional_chain_flow(node);
        } else {
            self.bind_each_child(node);
        }
    }

    /// Binds a binding element, evaluating its initializer before its name or nested pattern.
    pub(super) fn bind_binding_element_flow(&mut self, node: NodeId) {
        let NodeData::BindingElement(element) = self.ast.node(node).data() else {
            unreachable!("a binding element node carries binding element data");
        };
        self.bind_opt(element.dot_dot_dot_token);
        self.bind_opt(element.property_name);
        self.bind_initializer(element.initializer);
        self.bind_opt(element.name);
    }

    pub(super) fn bind_parameter_flow(&mut self, node: NodeId) {
        let ast = self.ast;
        let NodeData::ParameterDeclaration(parameter) = ast.node(node).data() else {
            unreachable!("a parameter node carries parameter data");
        };
        if let Some(modifiers) = parameter.modifiers {
            for &modifier in ast.list(*modifiers.list()) {
                self.bind(modifier);
            }
        }
        self.bind_opt(parameter.dot_dot_dot_token);
        self.bind_opt(parameter.question_token);
        self.bind_opt(parameter.type_node);
        self.bind_initializer(parameter.initializer);
        self.bind(parameter.name);
    }

    /// Binds a default value, which only affects flow when it is evaluated.
    fn bind_initializer(&mut self, node: Option<NodeId>) {
        let Some(node) = node else {
            return;
        };
        let entry_flow = self.current_flow;
        self.bind(node);
        if entry_flow == self.unreachable_flow || entry_flow == self.current_flow {
            return;
        }
        let exit_flow = self.create_branch_label();
        self.add_antecedent(exit_flow, entry_flow);
        self.add_antecedent(exit_flow, self.current_flow);
        self.current_flow = self.finish_flow_label(exit_flow);
    }
}
