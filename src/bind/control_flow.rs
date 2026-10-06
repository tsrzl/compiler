//! Statement control flow, modeled on TypeScript-Go's `bind*Statement` functions.

use super::binder::{ActiveLabel, Binder};
use super::flow::FlowId;
use super::narrowing::is_narrowing_expression;
use crate::ast::{FlowFlags, NodeData, NodeFlags, NodeId, SyntaxKind};

impl Binder<'_> {
    /// Binds `node` with `true_target` and `false_target` as the branches of the current
    /// condition.
    pub(super) fn bind_with_conditional_branches(
        &mut self,
        node: Option<NodeId>,
        true_target: Option<FlowId>,
        false_target: Option<FlowId>,
    ) {
        let saved_true = self.current_true_target;
        let saved_false = self.current_false_target;
        self.current_true_target = true_target;
        self.current_false_target = false_target;
        self.bind_opt(node);
        self.current_true_target = saved_true;
        self.current_false_target = saved_false;
    }

    /// Binds a condition and routes its true and false outcomes to the given labels; logical
    /// expressions and optional chains route their own outcomes.
    pub(super) fn bind_condition(
        &mut self,
        node: Option<NodeId>,
        true_target: FlowId,
        false_target: FlowId,
    ) {
        self.bind_with_conditional_branches(node, Some(true_target), Some(false_target));
        let ast = self.ast;
        let routes_own_outcomes = node.is_some_and(|node| {
            ast.is_logical_or_coalescing_assignment_expression(ast.skip_parentheses(node))
                || ast.is_logical_expression(node)
                || ast.is_optional_chain(node) && ast.is_outermost_optional_chain(node)
        });
        if !routes_own_outcomes {
            let when_true =
                self.create_flow_condition(FlowFlags::TRUE_CONDITION, self.current_flow, node);
            self.add_antecedent(true_target, when_true);
            let when_false =
                self.create_flow_condition(FlowFlags::FALSE_CONDITION, self.current_flow, node);
            self.add_antecedent(false_target, when_false);
        }
    }

    fn bind_iterative_statement(
        &mut self,
        node: NodeId,
        break_target: FlowId,
        continue_target: FlowId,
    ) {
        let save_break = self.current_break_target;
        let save_continue = self.current_continue_target;
        self.current_break_target = Some(break_target);
        self.current_continue_target = Some(continue_target);
        self.bind(node);
        self.current_break_target = save_break;
        self.current_continue_target = save_continue;
    }

    /// Makes `target` the continue target of every label directly enclosing the loop `node`.
    fn set_continue_target(&mut self, mut node: NodeId, target: FlowId) -> FlowId {
        let ast = self.ast;
        for label in self.active_labels.iter_mut().rev() {
            let Some(parent) = ast.node(node).parent() else {
                break;
            };
            if ast.node(parent).kind() != SyntaxKind::LabeledStatement {
                break;
            }
            label.continue_target = Some(target);
            node = parent;
        }
        target
    }

    pub(super) fn bind_while_statement(&mut self, node: NodeId) {
        let NodeData::WhileStatement(statement) = self.ast.node(node).data() else {
            unreachable!("a while statement node carries while data");
        };
        let loop_label = self.create_loop_label();
        let pre_while = self.set_continue_target(node, loop_label);
        let pre_body = self.create_branch_label();
        let post_while = self.create_branch_label();
        self.add_antecedent(pre_while, self.current_flow);
        self.current_flow = pre_while;
        self.bind_condition(Some(statement.expression), pre_body, post_while);
        self.current_flow = self.finish_flow_label(pre_body);
        self.bind_iterative_statement(statement.statement, post_while, pre_while);
        self.add_antecedent(pre_while, self.current_flow);
        self.current_flow = self.finish_flow_label(post_while);
    }

    pub(super) fn bind_do_statement(&mut self, node: NodeId) {
        let NodeData::DoStatement(statement) = self.ast.node(node).data() else {
            unreachable!("a do statement node carries do data");
        };
        let pre_do = self.create_loop_label();
        let condition_label = self.create_branch_label();
        let pre_condition = self.set_continue_target(node, condition_label);
        let post_do = self.create_branch_label();
        self.add_antecedent(pre_do, self.current_flow);
        self.current_flow = pre_do;
        self.bind_iterative_statement(statement.statement, post_do, pre_condition);
        self.add_antecedent(pre_condition, self.current_flow);
        self.current_flow = self.finish_flow_label(pre_condition);
        self.bind_condition(Some(statement.expression), pre_do, post_do);
        self.current_flow = self.finish_flow_label(post_do);
    }

    pub(super) fn bind_for_statement(&mut self, node: NodeId) {
        let NodeData::ForStatement(statement) = self.ast.node(node).data() else {
            unreachable!("a for statement node carries for data");
        };
        let loop_label = self.create_loop_label();
        let pre_loop = self.set_continue_target(node, loop_label);
        let pre_body = self.create_branch_label();
        let pre_incrementor = self.create_branch_label();
        let post_loop = self.create_branch_label();
        self.bind_opt(statement.initializer);
        self.add_antecedent(pre_loop, self.current_flow);
        self.current_flow = pre_loop;
        self.bind_condition(statement.condition, pre_body, post_loop);
        self.current_flow = self.finish_flow_label(pre_body);
        self.bind_iterative_statement(statement.statement, post_loop, pre_incrementor);
        self.add_antecedent(pre_incrementor, self.current_flow);
        self.current_flow = self.finish_flow_label(pre_incrementor);
        self.bind_opt(statement.incrementor);
        self.add_antecedent(pre_loop, self.current_flow);
        self.current_flow = self.finish_flow_label(post_loop);
    }

    pub(super) fn bind_for_in_or_of_statement(&mut self, node: NodeId) {
        let ast = self.ast;
        let NodeData::ForInOrOfStatement(statement) = ast.node(node).data() else {
            unreachable!("a for-in or for-of statement node carries loop data");
        };
        let loop_label = self.create_loop_label();
        let pre_loop = self.set_continue_target(node, loop_label);
        let post_loop = self.create_branch_label();
        self.bind(statement.expression);
        self.add_antecedent(pre_loop, self.current_flow);
        self.current_flow = pre_loop;
        if ast.node(node).kind() == SyntaxKind::ForOfStatement {
            self.bind_opt(statement.await_modifier);
        }
        self.add_antecedent(post_loop, self.current_flow);
        self.bind(statement.initializer);
        if ast.node(statement.initializer).kind() != SyntaxKind::VariableDeclarationList {
            self.bind_assignment_target_flow(statement.initializer);
        }
        self.bind_iterative_statement(statement.statement, post_loop, pre_loop);
        self.add_antecedent(pre_loop, self.current_flow);
        self.current_flow = self.finish_flow_label(post_loop);
    }

    pub(super) fn bind_if_statement(&mut self, node: NodeId) {
        let NodeData::IfStatement(statement) = self.ast.node(node).data() else {
            unreachable!("an if statement node carries if data");
        };
        let then_label = self.create_branch_label();
        let else_label = self.create_branch_label();
        let post_if = self.create_branch_label();
        self.bind_condition(Some(statement.expression), then_label, else_label);
        self.current_flow = self.finish_flow_label(then_label);
        self.bind(statement.then_statement);
        self.add_antecedent(post_if, self.current_flow);
        self.current_flow = self.finish_flow_label(else_label);
        self.bind_opt(statement.else_statement);
        self.add_antecedent(post_if, self.current_flow);
        self.current_flow = self.finish_flow_label(post_if);
    }

    pub(super) fn bind_return_statement(&mut self, node: NodeId) {
        self.bind_opt(self.ast.node(node).data().expression());
        if let Some(return_target) = self.current_return_target {
            self.add_antecedent(return_target, self.current_flow);
        }
        self.current_flow = self.unreachable_flow;
        self.has_explicit_return = true;
        self.has_flow_effects = true;
    }

    pub(super) fn bind_throw_statement(&mut self, node: NodeId) {
        self.bind_opt(self.ast.node(node).data().expression());
        self.current_flow = self.unreachable_flow;
        self.has_flow_effects = true;
    }

    /// Binds `break` or `continue`, jumping to the labeled statement's target or, without a
    /// label, to the innermost loop or switch target.
    pub(super) fn bind_break_or_continue_statement(&mut self, node: NodeId, is_break: bool) {
        let ast = self.ast;
        let label = ast.node(node).data().label();
        self.bind_opt(label);
        if let Some(label) = label {
            let name = ast.identifier_text(label).unwrap_or_default();
            if let Some(active) = self
                .active_labels
                .iter_mut()
                .rev()
                .find(|active| active.name == name)
            {
                active.referenced = true;
                let target = if is_break {
                    Some(active.break_target)
                } else {
                    active.continue_target
                };
                self.bind_break_or_continue_flow(target);
            }
        } else {
            let target = if is_break {
                self.current_break_target
            } else {
                self.current_continue_target
            };
            self.bind_break_or_continue_flow(target);
        }
    }

    fn bind_break_or_continue_flow(&mut self, target: Option<FlowId>) {
        if let Some(target) = target {
            self.add_antecedent(target, self.current_flow);
            self.current_flow = self.unreachable_flow;
            self.has_flow_effects = true;
        }
    }

    /// Binds a `try` statement. Any code in the `try` block may throw, so every mutation is an
    /// antecedent of the exception label that starts the `catch` or `finally` block.
    pub(super) fn bind_try_statement(&mut self, node: NodeId) {
        let NodeData::TryStatement(statement) = self.ast.node(node).data() else {
            unreachable!("a try statement node carries try data");
        };
        let save_return_target = self.current_return_target;
        let save_exception_target = self.current_exception_target;
        let normal_exit = self.create_branch_label();
        let return_label = self.create_branch_label();
        let mut exception_label = self.create_branch_label();
        if statement.finally_block.is_some() {
            self.current_return_target = Some(return_label);
        }
        self.add_antecedent(exception_label, self.current_flow);
        self.current_exception_target = Some(exception_label);
        self.bind(statement.try_block);
        self.add_antecedent(normal_exit, self.current_flow);
        if let Some(catch_clause) = statement.catch_clause {
            // Exceptions from the try block start the catch clause, which then acts like a
            // second try block for exceptions it raises.
            self.current_flow = self.finish_flow_label(exception_label);
            exception_label = self.create_branch_label();
            self.add_antecedent(exception_label, self.current_flow);
            self.current_exception_target = Some(exception_label);
            self.bind(catch_clause);
            self.add_antecedent(normal_exit, self.current_flow);
        }
        self.current_return_target = save_return_target;
        self.current_exception_target = save_exception_target;
        if let Some(finally_block) = statement.finally_block {
            self.bind_finally_block(finally_block, normal_exit, exception_label, return_label);
        } else {
            self.current_flow = self.finish_flow_label(normal_exit);
        }
    }

    /// Binds a `finally` block reached by normal completion, returns, and exceptions, then uses
    /// reduce labels so analysis past the block, or back through returns or outer exception
    /// handlers, sees only the matching subset of those paths.
    fn bind_finally_block(
        &mut self,
        finally_block: NodeId,
        normal_exit: FlowId,
        exception_label: FlowId,
        return_label: FlowId,
    ) {
        let finally_label = self.create_branch_label();
        let exits = self.flow.antecedent_list(normal_exit);
        let exceptions = self.flow.antecedent_list(exception_label);
        let returns = self.flow.antecedent_list(return_label);
        let abrupt = self.flow.combine_lists(exceptions, returns);
        let all = self.flow.combine_lists(exits, abrupt);
        self.flow.set_antecedent_list(finally_label, all);
        self.current_flow = finally_label;
        self.bind(finally_block);
        if self.is_unreachable(self.current_flow) {
            // An unreachable end of the finally block makes the end of the try statement unreachable.
            self.current_flow = self.unreachable_flow;
            return;
        }
        if let Some(return_target) = self.current_return_target
            && returns.is_some()
        {
            let reduced = self.create_reduce_label(finally_label, returns, self.current_flow);
            self.add_antecedent(return_target, reduced);
        }
        if let Some(exception_target) = self.current_exception_target
            && exceptions.is_some()
        {
            let reduced = self.create_reduce_label(finally_label, exceptions, self.current_flow);
            self.add_antecedent(exception_target, reduced);
        }
        // A reachable finally end after try and catch blocks that never complete normally, as in
        // `try { return 1; } finally { ... }`, still leaves the statement's end unreachable.
        self.current_flow = if exits.is_some() {
            self.create_reduce_label(finally_label, exits, self.current_flow)
        } else {
            self.unreachable_flow
        };
    }

    pub(super) fn bind_switch_statement(&mut self, node: NodeId) {
        let ast = self.ast;
        let NodeData::SwitchStatement(statement) = ast.node(node).data() else {
            unreachable!("a switch statement node carries switch data");
        };
        let post_switch = self.create_branch_label();
        self.bind(statement.expression);
        let save_break_target = self.current_break_target;
        let save_pre_switch_case_flow = self.pre_switch_case_flow;
        self.current_break_target = Some(post_switch);
        self.pre_switch_case_flow = Some(self.current_flow);
        self.bind(statement.case_block);
        self.add_antecedent(post_switch, self.current_flow);
        let clauses = ast
            .node(statement.case_block)
            .data()
            .as_case_block()
            .expect("a switch statement owns a case block")
            .clauses;
        let has_default = ast
            .list(clauses)
            .iter()
            .any(|&clause| ast.node(clause).kind() == SyntaxKind::DefaultClause);
        if !has_default {
            let pre_switch = self.pre_switch_case_flow();
            let implicit_default = self.create_flow_switch_clause(pre_switch, node, 0, 0);
            self.add_antecedent(post_switch, implicit_default);
        }
        self.current_break_target = save_break_target;
        self.pre_switch_case_flow = save_pre_switch_case_flow;
        self.current_flow = self.finish_flow_label(post_switch);
    }

    /// Binds case clauses. Consecutive empty clauses share one narrowing range, and each clause
    /// is reached from the switch expression or by falling through the previous clause.
    pub(super) fn bind_case_block(&mut self, node: NodeId) {
        let ast = self.ast;
        let switch_statement = ast
            .node(node)
            .parent()
            .expect("a case block belongs to a switch");
        let expression = ast
            .node(switch_statement)
            .data()
            .expression()
            .expect("a switch statement has an expression");
        let is_narrowing_switch = ast.node(expression).kind() == SyntaxKind::TrueKeyword
            || is_narrowing_expression(ast, expression);
        let clauses = ast
            .node(node)
            .data()
            .as_case_block()
            .expect("a case block node carries case block data")
            .clauses;
        let clauses = ast.list(clauses);
        let mut fallthrough_flow = self.unreachable_flow;
        let mut index = 0;
        while index < clauses.len() {
            let clause_start = index;
            while clause_statements(self, clauses[index]).is_empty() && index + 1 < clauses.len() {
                if fallthrough_flow == self.unreachable_flow {
                    self.current_flow = self.pre_switch_case_flow();
                }
                self.bind(clauses[index]);
                index += 1;
            }
            let pre_case = self.create_branch_label();
            let pre_switch = self.pre_switch_case_flow();
            let pre_case_flow = if is_narrowing_switch {
                self.create_flow_switch_clause(
                    pre_switch,
                    switch_statement,
                    clause_start,
                    index + 1,
                )
            } else {
                pre_switch
            };
            self.add_antecedent(pre_case, pre_case_flow);
            self.add_antecedent(pre_case, fallthrough_flow);
            self.current_flow = self.finish_flow_label(pre_case);
            let clause = clauses[index];
            self.bind(clause);
            fallthrough_flow = self.current_flow;
            if !self.is_unreachable(self.current_flow) && index != clauses.len() - 1 {
                self.fallthrough_flow_nodes
                    .insert(clause, self.current_flow);
            }
            index += 1;
        }
    }

    pub(super) fn bind_case_or_default_clause(&mut self, node: NodeId) {
        let NodeData::CaseOrDefaultClause(clause) = self.ast.node(node).data() else {
            unreachable!("a case or default clause node carries clause data");
        };
        if let Some(expression) = clause.expression {
            let save_current_flow = self.current_flow;
            self.current_flow = self.pre_switch_case_flow();
            self.bind(expression);
            self.current_flow = save_current_flow;
        }
        for &statement in self.ast.list(clause.statements) {
            self.bind(statement);
        }
    }

    fn pre_switch_case_flow(&self) -> FlowId {
        self.pre_switch_case_flow
            .expect("case clauses are bound inside a switch statement")
    }

    pub(super) fn bind_expression_statement(&mut self, node: NodeId) {
        let expression = self
            .ast
            .node(node)
            .data()
            .expression()
            .expect("an expression statement has an expression");
        self.bind(expression);
        self.maybe_bind_expression_flow_if_call(expression);
    }

    /// Records a call with a dotted callee as a potential assertion in the control flow.
    pub(super) fn maybe_bind_expression_flow_if_call(&mut self, node: NodeId) {
        let ast = self.ast;
        if let Some(call) = ast.node(node).data().as_call_expression()
            && ast.node(call.expression).kind() != SyntaxKind::SuperKeyword
            && ast.is_dotted_name(call.expression)
        {
            self.current_flow = self.create_flow_call(self.current_flow, node);
        }
    }

    pub(super) fn bind_labeled_statement(&mut self, node: NodeId) {
        let ast = self.ast;
        let NodeData::LabeledStatement(statement) = ast.node(node).data() else {
            unreachable!("a labeled statement node carries label data");
        };
        let post_statement = self.create_branch_label();
        self.active_labels.push(ActiveLabel {
            name: ast
                .identifier_text(statement.label)
                .unwrap_or_default()
                .to_owned(),
            break_target: post_statement,
            continue_target: None,
            referenced: false,
        });
        self.bind(statement.label);
        self.bind(statement.statement);
        let label = self.active_labels.pop().expect("the label pushed above");
        if !label.referenced {
            // The checker decides whether to report the unused label.
            self.add_node_flags(statement.label, NodeFlags::UNREACHABLE);
        }
        self.add_antecedent(post_statement, self.current_flow);
        self.current_flow = self.finish_flow_label(post_statement);
    }
}

fn clause_statements<'a>(binder: &Binder<'a>, clause: NodeId) -> &'a [NodeId] {
    let ast = binder.ast;
    ast.node(clause)
        .data()
        .statements()
        .map_or(&[][..], |statements| ast.list(statements))
}
