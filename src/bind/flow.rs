//! The control flow graph modeled on TypeScript-Go's `internal/ast/flow.go`.
//!
//! Flow nodes and antecedent lists live in one [`FlowGraph`] per bound file and refer to each
//! other by [`FlowId`]. Labels collect their antecedents while the binder walks the file.

use crate::ast::{FlowFlags, NodeId};

/// A flow node's identity within its [`FlowGraph`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FlowId(u32);

impl FlowId {
    /// Returns the flow node's zero-based index in its graph.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

/// An antecedent list entry's identity within its [`FlowGraph`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FlowListId(u32);

/// What a flow node refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowPayload {
    /// Start nodes without a function, labels, and the unreachable node.
    None,
    /// The condition, assignment target, declaration, or call the node describes.
    Node(NodeId),
    /// The `case` and `default` clauses `[clause_start, clause_end)` of a narrowing switch; an
    /// empty range is the implicit default of a switch without one.
    SwitchClause {
        /// The switch statement.
        switch_statement: NodeId,
        /// The first clause index.
        clause_start: u32,
        /// One past the last clause index.
        clause_end: u32,
    },
    /// A temporary reduction of a `finally` label's antecedents while analysis passes through it.
    ReduceLabel {
        /// The label whose antecedents are reduced.
        target: FlowId,
        /// The reduced antecedent list.
        antecedents: Option<FlowListId>,
    },
}

/// A node in the control flow graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlowNode {
    flags: FlowFlags,
    payload: FlowPayload,
    antecedent: Option<FlowId>,
    antecedents: Option<FlowListId>,
}

impl FlowNode {
    /// Returns the flow node's flags.
    #[must_use]
    pub const fn flags(&self) -> FlowFlags {
        self.flags
    }

    /// Returns what the flow node refers to.
    #[must_use]
    pub const fn payload(&self) -> FlowPayload {
        self.payload
    }

    /// Returns the single antecedent of a non-label node.
    #[must_use]
    pub const fn antecedent(&self) -> Option<FlowId> {
        self.antecedent
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FlowListEntry {
    flow: FlowId,
    next: Option<FlowListId>,
}

/// The control flow graph of one bound file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FlowGraph {
    nodes: Vec<FlowNode>,
    lists: Vec<FlowListEntry>,
}

impl FlowGraph {
    /// Returns the flow node with identity `id`.
    ///
    /// # Panics
    ///
    /// Panics when `id` was not produced by this graph.
    #[must_use]
    pub fn node(&self, id: FlowId) -> &FlowNode {
        &self.nodes[id.0 as usize]
    }

    /// Returns a label's antecedents in the order they were added.
    pub fn antecedents(&self, id: FlowId) -> impl Iterator<Item = FlowId> + '_ {
        self.list(self.node(id).antecedents)
    }

    /// Returns the flow nodes of an antecedent list.
    pub fn list(&self, head: Option<FlowListId>) -> impl Iterator<Item = FlowId> + '_ {
        std::iter::successors(head, |&entry| self.lists[entry.0 as usize].next)
            .map(|entry| self.lists[entry.0 as usize].flow)
    }

    /// Returns the number of flow nodes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns whether the graph holds no flow nodes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub(super) fn create(
        &mut self,
        flags: FlowFlags,
        payload: FlowPayload,
        antecedent: Option<FlowId>,
    ) -> FlowId {
        let id = FlowId(u32::try_from(self.nodes.len()).expect("flow node count fits in u32"));
        self.nodes.push(FlowNode {
            flags,
            payload,
            antecedent,
            antecedents: None,
        });
        id
    }

    pub(super) fn add_flags(&mut self, id: FlowId, flags: FlowFlags) {
        self.nodes[id.0 as usize].flags |= flags;
    }

    pub(super) fn antecedent_list(&self, id: FlowId) -> Option<FlowListId> {
        self.nodes[id.0 as usize].antecedents
    }

    pub(super) fn set_antecedent_list(&mut self, id: FlowId, list: Option<FlowListId>) {
        self.nodes[id.0 as usize].antecedents = list;
    }

    /// Appends `flow` to a label's antecedents unless it is already listed; returns whether it
    /// was added.
    pub(super) fn push_antecedent(&mut self, label: FlowId, flow: FlowId) -> bool {
        let mut last = None;
        let mut current = self.antecedent_list(label);
        while let Some(entry) = current {
            let listed = self.lists[entry.0 as usize];
            if listed.flow == flow {
                return false;
            }
            last = Some(entry);
            current = listed.next;
        }
        let entry = self.new_list(flow, None);
        match last {
            Some(last) => self.lists[last.0 as usize].next = Some(entry),
            None => self.set_antecedent_list(label, Some(entry)),
        }
        true
    }

    /// Returns a new list holding the entries of `head` followed by `tail`.
    pub(super) fn combine_lists(
        &mut self,
        head: Option<FlowListId>,
        tail: Option<FlowListId>,
    ) -> Option<FlowListId> {
        let head_flows: Vec<FlowId> = self.list(head).collect();
        head_flows
            .into_iter()
            .rev()
            .fold(tail, |next, flow| Some(self.new_list(flow, next)))
    }

    fn new_list(&mut self, flow: FlowId, next: Option<FlowListId>) -> FlowListId {
        let id = FlowListId(u32::try_from(self.lists.len()).expect("flow list count fits in u32"));
        self.lists.push(FlowListEntry { flow, next });
        id
    }
}
