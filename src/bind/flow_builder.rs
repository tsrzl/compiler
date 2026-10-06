//! Flow graph construction primitives modeled on TypeScript-Go's binder.

use super::binder::Binder;
use super::flow::{FlowId, FlowListId, FlowPayload};
use super::narrowing::is_narrowing_expression;
use crate::ast::{FlowFlags, NodeId, SyntaxKind};

impl Binder<'_> {
    pub(super) fn new_flow_node(&mut self, flags: FlowFlags) -> FlowId {
        self.flow.create(flags, FlowPayload::None, None)
    }

    pub(super) fn create_loop_label(&mut self) -> FlowId {
        self.new_flow_node(FlowFlags::LOOP_LABEL)
    }

    pub(super) fn create_branch_label(&mut self) -> FlowId {
        self.new_flow_node(FlowFlags::BRANCH_LABEL)
    }

    pub(super) fn create_reduce_label(
        &mut self,
        target: FlowId,
        antecedents: Option<FlowListId>,
        antecedent: FlowId,
    ) -> FlowId {
        self.flow.create(
            FlowFlags::REDUCE_LABEL,
            FlowPayload::ReduceLabel {
                target,
                antecedents,
            },
            Some(antecedent),
        )
    }

    /// Returns the flow after `expression` evaluates to the branch `flags` selects: unchanged
    /// when the expression cannot narrow, unreachable when a literal rules the branch out.
    pub(super) fn create_flow_condition(
        &mut self,
        flags: FlowFlags,
        antecedent: FlowId,
        expression: Option<NodeId>,
    ) -> FlowId {
        if self.is_unreachable(antecedent) {
            return antecedent;
        }
        let Some(expression) = expression else {
            return if flags.intersects(FlowFlags::TRUE_CONDITION) {
                antecedent
            } else {
                self.unreachable_flow
            };
        };
        let ast = self.ast;
        let kind = ast.node(expression).kind();
        let literal_rules_out_branch = kind == SyntaxKind::TrueKeyword
            && flags.intersects(FlowFlags::FALSE_CONDITION)
            || kind == SyntaxKind::FalseKeyword && flags.intersects(FlowFlags::TRUE_CONDITION);
        let parent_is_nullish_coalesce = ast
            .node(expression)
            .parent()
            .is_some_and(|parent| ast.is_nullish_coalesce(parent));
        if literal_rules_out_branch
            && !ast.is_expression_of_optional_chain_root(expression)
            && !parent_is_nullish_coalesce
        {
            return self.unreachable_flow;
        }
        if !is_narrowing_expression(ast, expression) {
            return antecedent;
        }
        self.set_flow_node_referenced(antecedent);
        self.flow
            .create(flags, FlowPayload::Node(expression), Some(antecedent))
    }

    pub(super) fn create_flow_mutation(
        &mut self,
        flags: FlowFlags,
        antecedent: FlowId,
        node: NodeId,
    ) -> FlowId {
        self.set_flow_node_referenced(antecedent);
        self.has_flow_effects = true;
        let result = self
            .flow
            .create(flags, FlowPayload::Node(node), Some(antecedent));
        if let Some(exception_target) = self.current_exception_target {
            self.add_antecedent(exception_target, result);
        }
        result
    }

    pub(super) fn create_flow_switch_clause(
        &mut self,
        antecedent: FlowId,
        switch_statement: NodeId,
        clause_start: usize,
        clause_end: usize,
    ) -> FlowId {
        self.set_flow_node_referenced(antecedent);
        let payload = FlowPayload::SwitchClause {
            switch_statement,
            clause_start: u32::try_from(clause_start).expect("clause index fits in u32"),
            clause_end: u32::try_from(clause_end).expect("clause index fits in u32"),
        };
        self.flow
            .create(FlowFlags::SWITCH_CLAUSE, payload, Some(antecedent))
    }

    pub(super) fn create_flow_call(&mut self, antecedent: FlowId, node: NodeId) -> FlowId {
        self.set_flow_node_referenced(antecedent);
        self.has_flow_effects = true;
        self.flow
            .create(FlowFlags::CALL, FlowPayload::Node(node), Some(antecedent))
    }

    /// Marks a flow node referenced on first use and shared on later uses.
    fn set_flow_node_referenced(&mut self, flow: FlowId) {
        let flags = if self
            .flow
            .node(flow)
            .flags()
            .intersects(FlowFlags::REFERENCED)
        {
            FlowFlags::SHARED
        } else {
            FlowFlags::REFERENCED
        };
        self.flow.add_flags(flow, flags);
    }

    pub(super) fn add_antecedent(&mut self, label: FlowId, antecedent: FlowId) {
        if self.is_unreachable(antecedent) {
            return;
        }
        if self.flow.push_antecedent(label, antecedent) {
            self.set_flow_node_referenced(antecedent);
        }
    }

    /// Returns the flow after a label: unreachable without antecedents, the sole antecedent
    /// when there is one, and the label itself otherwise.
    pub(super) fn finish_flow_label(&self, label: FlowId) -> FlowId {
        let mut antecedents = self.flow.antecedents(label);
        match (antecedents.next(), antecedents.next()) {
            (None, _) => self.unreachable_flow,
            (Some(only), None) => only,
            (Some(_), Some(_)) => label,
        }
    }

    pub(super) fn is_unreachable(&self, flow: FlowId) -> bool {
        self.flow
            .node(flow)
            .flags()
            .intersects(FlowFlags::UNREACHABLE)
    }
}
