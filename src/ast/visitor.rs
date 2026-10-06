//! Child traversal over the arena syntax tree.

use super::arena::{ModifierList, NodeId, NodeList};

/// Receives a node's direct children in TypeScript-Go `ForEachChild` order.
///
/// Each method returns `true` to stop the traversal early.
pub trait ChildVisitor {
    /// Visits a single child node.
    fn visit_node(&mut self, node: NodeId) -> bool;

    /// Visits a child node list.
    fn visit_list(&mut self, list: &NodeList) -> bool;

    /// Visits a child modifier list.
    fn visit_modifiers(&mut self, modifiers: &ModifierList) -> bool;

    /// Visits a child list without its own source range.
    fn visit_raw(&mut self, nodes: &[NodeId]) -> bool;
}

/// Adapts a per-node callback into a [`ChildVisitor`] that expands lists into their elements.
pub(super) struct EachChild<'ast, F> {
    pub(super) list_nodes: &'ast [NodeId],
    pub(super) callback: F,
}

impl<F: FnMut(NodeId) -> bool> ChildVisitor for EachChild<'_, F> {
    fn visit_node(&mut self, node: NodeId) -> bool {
        (self.callback)(node)
    }

    fn visit_list(&mut self, list: &NodeList) -> bool {
        list.nodes(self.list_nodes)
            .iter()
            .any(|&node| (self.callback)(node))
    }

    fn visit_modifiers(&mut self, modifiers: &ModifierList) -> bool {
        self.visit_list(modifiers.list())
    }

    fn visit_raw(&mut self, nodes: &[NodeId]) -> bool {
        nodes.iter().any(|&node| (self.callback)(node))
    }
}
