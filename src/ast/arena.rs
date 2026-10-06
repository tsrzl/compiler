//! The per-file node arena: identities, ranges, lists, and the immutable tree.

use super::nodes::NodeData;
use super::visitor::EachChild;
use super::{ModifierFlags, NodeFlags, SyntaxKind};

/// A node's identity within one file's [`Ast`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(u32);

impl NodeId {
    /// Returns the node's zero-based index in its arena.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }

    const fn as_usize(self) -> usize {
        self.0 as usize
    }
}

/// A source-ranged list of child nodes stored in the arena's shared list pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeList {
    pos: u32,
    end: u32,
    start: u32,
    len: u32,
    missing: bool,
}

impl NodeList {
    /// Returns the UTF-8 byte offset where the list's text starts.
    #[must_use]
    pub const fn pos(self) -> u32 {
        self.pos
    }

    /// Returns the UTF-8 byte offset where the list's text ends.
    #[must_use]
    pub const fn end(self) -> u32 {
        self.end
    }

    /// Returns the number of nodes in the list.
    #[must_use]
    pub const fn len(self) -> usize {
        self.len as usize
    }

    /// Returns whether the list is missing because its opening token was absent, as opposed to
    /// present but empty.
    #[must_use]
    pub const fn is_missing(self) -> bool {
        self.missing
    }

    /// Returns whether the list has no nodes.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.len == 0
    }

    pub(super) fn nodes(self, pool: &[NodeId]) -> &[NodeId] {
        &pool[self.start as usize..(self.start + self.len) as usize]
    }
}

/// A list of modifiers and decorators with their combined flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModifierList {
    list: NodeList,
    flags: ModifierFlags,
}

impl ModifierList {
    /// Creates a modifier list from its nodes and combined flags.
    #[must_use]
    pub const fn new(list: NodeList, flags: ModifierFlags) -> Self {
        Self { list, flags }
    }

    /// Returns the underlying node list.
    #[must_use]
    pub const fn list(&self) -> &NodeList {
        &self.list
    }

    /// Returns the combined modifier flags.
    #[must_use]
    pub const fn flags(&self) -> ModifierFlags {
        self.flags
    }
}

/// A syntax node: its kind, parse flags, source range, parent, and kind-specific data.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    kind: SyntaxKind,
    flags: NodeFlags,
    pos: u32,
    end: u32,
    parent: Option<NodeId>,
    data: NodeData,
}

impl Node {
    /// Returns the node kind.
    #[must_use]
    pub const fn kind(&self) -> SyntaxKind {
        self.kind
    }

    /// Returns the flags recorded by the parser.
    #[must_use]
    pub const fn flags(&self) -> NodeFlags {
        self.flags
    }

    /// Returns the UTF-8 byte offset where the node's text, including leading trivia, starts.
    #[must_use]
    pub const fn pos(&self) -> u32 {
        self.pos
    }

    /// Returns the UTF-8 byte offset where the node's text ends.
    #[must_use]
    pub const fn end(&self) -> u32 {
        self.end
    }

    /// Returns the node's parent, or `None` for the root.
    #[must_use]
    pub const fn parent(&self) -> Option<NodeId> {
        self.parent
    }

    /// Returns the kind-specific data.
    #[must_use]
    pub const fn data(&self) -> &NodeData {
        &self.data
    }
}

/// An immutable syntax tree for one file. Nodes are referenced by [`NodeId`].
#[derive(Debug, Clone, PartialEq)]
pub struct Ast {
    nodes: Box<[Node]>,
    list_nodes: Box<[NodeId]>,
    root: NodeId,
}

impl Ast {
    /// Returns the root node.
    #[must_use]
    pub const fn root(&self) -> NodeId {
        self.root
    }

    /// Returns the node with identity `id`.
    ///
    /// # Panics
    ///
    /// Panics when `id` was not produced by this tree.
    #[must_use]
    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id.as_usize()]
    }

    /// Returns the nodes of a list belonging to this tree.
    #[must_use]
    pub fn list(&self, list: NodeList) -> &[NodeId] {
        list.nodes(&self.list_nodes)
    }

    /// Visits the direct children of `id` in source order until `callback` returns `true`.
    pub fn for_each_child(&self, id: NodeId, callback: impl FnMut(NodeId) -> bool) -> bool {
        let mut visitor = EachChild {
            list_nodes: &self.list_nodes,
            callback,
        };
        self.node(id).data.visit_children(&mut visitor)
    }

    /// Returns the direct children of `id` in source order.
    #[must_use]
    pub fn children(&self, id: NodeId) -> Vec<NodeId> {
        let mut children = Vec::new();
        self.for_each_child(id, |child| {
            children.push(child);
            false
        });
        children
    }
}

/// The size of an [`AstBuilder`] at a point in time, used to discard speculative nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AstBuilderMark {
    nodes: usize,
    list_nodes: usize,
}

/// Builds an [`Ast`] bottom-up; parent links are assigned when the tree is finished.
#[derive(Debug, Default)]
pub struct AstBuilder {
    nodes: Vec<Node>,
    list_nodes: Vec<NodeId>,
}

impl AstBuilder {
    /// Creates an empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a node and returns its identity.
    ///
    /// # Panics
    ///
    /// Panics when `data` is not valid for `kind`, or when the arena exceeds `u32::MAX` nodes.
    pub fn add_node(
        &mut self,
        kind: SyntaxKind,
        pos: u32,
        end: u32,
        flags: NodeFlags,
        data: NodeData,
    ) -> NodeId {
        assert!(data.accepts_kind(kind), "{kind:?} cannot hold {data:?}");
        let id = NodeId(u32::try_from(self.nodes.len()).expect("node count fits in u32"));
        self.nodes.push(Node {
            kind,
            flags,
            pos,
            end,
            parent: None,
            data,
        });
        id
    }

    /// Adds `flags` to a node that has already been added.
    pub fn add_flags(&mut self, id: NodeId, flags: NodeFlags) {
        self.nodes[id.as_usize()].flags |= flags;
    }

    /// Returns a node that has already been added.
    #[must_use]
    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id.as_usize()]
    }

    /// Returns the nodes of a list that has already been added.
    #[must_use]
    pub fn list(&self, list: NodeList) -> &[NodeId] {
        list.nodes(&self.list_nodes)
    }

    /// Captures the builder size so that speculatively added nodes can be discarded.
    #[must_use]
    pub fn mark(&self) -> AstBuilderMark {
        AstBuilderMark {
            nodes: self.nodes.len(),
            list_nodes: self.list_nodes.len(),
        }
    }

    /// Discards nodes and lists added after `mark`.
    pub fn rewind(&mut self, mark: AstBuilderMark) {
        self.nodes.truncate(mark.nodes);
        self.list_nodes.truncate(mark.list_nodes);
    }

    /// Adds a list of previously added nodes covering `pos..end`.
    ///
    /// # Panics
    ///
    /// Panics when the list pool exceeds `u32::MAX` entries.
    pub fn add_list(
        &mut self,
        pos: u32,
        end: u32,
        nodes: impl IntoIterator<Item = NodeId>,
    ) -> NodeList {
        let start = u32::try_from(self.list_nodes.len()).expect("list pool fits in u32");
        self.list_nodes.extend(nodes);
        let len = u32::try_from(self.list_nodes.len()).expect("list pool fits in u32") - start;
        NodeList {
            pos,
            end,
            start,
            len,
            missing: false,
        }
    }

    /// Adds an empty list at `pos` that records that its opening token was missing.
    pub fn add_missing_list(&mut self, pos: u32) -> NodeList {
        NodeList {
            missing: true,
            ..self.add_list(pos, pos, [])
        }
    }

    /// Finishes the tree rooted at `root`, linking every node reachable from the root to its
    /// parent. Unreachable nodes, such as those replaced during reparsing, keep no parent.
    #[must_use]
    pub fn finish(self, root: NodeId) -> Ast {
        let mut ast = Ast {
            nodes: self.nodes.into_boxed_slice(),
            list_nodes: self.list_nodes.into_boxed_slice(),
            root,
        };
        let mut pending = vec![root];
        while let Some(parent) = pending.pop() {
            for child in ast.children(parent) {
                ast.nodes[child.as_usize()].parent = Some(parent);
                pending.push(child);
            }
        }
        ast
    }
}
