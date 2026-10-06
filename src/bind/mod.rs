//! Symbol binding modeled on TypeScript-Go's `internal/binder` package.
//!
//! [`bind_source_file`] walks one parsed file and produces an immutable [`BoundFile`]: the
//! symbols its declarations introduce, the symbol tables of each container, and binder
//! diagnostics such as duplicate declarations. The syntax tree is never mutated; every binding
//! fact is a side table keyed by [`NodeId`].
//!
//! Ported so far: declarations, containers and locals, class, interface, enum, and literal
//! members, namespaces and ambient modules, module exports, imports, JSX attributes, the
//! external-module symbol, the control flow graph with reachability, and strict-mode grammar
//! checks. JavaScript assignment declarations are ported in a later increment.

mod binder;
mod container_flags;
mod control_flow;
mod declarations;
mod expression_flow;
mod flow;
mod flow_builder;
mod modules;
mod narrowing;
mod strict_mode;
mod walk;

pub use flow::{FlowGraph, FlowId, FlowListId, FlowNode, FlowPayload};

use std::collections::{HashMap, HashSet};

use crate::ast::{NodeFlags, NodeId};
use crate::diagnostics::Diagnostic;
use crate::parser::{ExternalModuleIndicatorOptions, ParsedSourceFile};
use crate::symbols::{SymbolArena, SymbolId, SymbolTable};

/// The immutable binding facts of one source file.
#[derive(Debug, Clone, Default)]
pub struct BoundFile {
    symbols: SymbolArena,
    node_symbols: HashMap<NodeId, SymbolId>,
    local_symbols: HashMap<NodeId, SymbolId>,
    locals: HashMap<NodeId, SymbolTable>,
    diagnostics: Vec<Diagnostic>,
    external_module_indicator: Option<NodeId>,
    classifiable_names: HashSet<String>,
    global_exports: SymbolTable,
    pattern_ambient_modules: Vec<PatternAmbientModule>,
    flow: FlowGraph,
    flow_nodes: HashMap<NodeId, FlowId>,
    end_flow_nodes: HashMap<NodeId, FlowId>,
    return_flow_nodes: HashMap<NodeId, FlowId>,
    fallthrough_flow_nodes: HashMap<NodeId, FlowId>,
    node_flags: HashMap<NodeId, NodeFlags>,
}

/// An ambient module whose name contains one `*` wildcard, such as `declare module "*.css"`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternAmbientModule {
    /// The module name pattern.
    pub pattern: String,
    /// The byte index of the `*` wildcard in the pattern.
    pub star_index: usize,
    /// The ambient module's symbol.
    pub symbol: SymbolId,
}

impl BoundFile {
    /// Returns the symbols created while binding.
    #[must_use]
    pub const fn symbols(&self) -> &SymbolArena {
        &self.symbols
    }

    /// Returns the symbol a declaration contributes to.
    #[must_use]
    pub fn symbol_of(&self, node: NodeId) -> Option<SymbolId> {
        self.node_symbols.get(&node).copied()
    }

    /// Returns the local symbol paired with an exported declaration's export symbol.
    #[must_use]
    pub fn local_symbol_of(&self, node: NodeId) -> Option<SymbolId> {
        self.local_symbols.get(&node).copied()
    }

    /// Returns the locals table of a container, if it declares any locals.
    #[must_use]
    pub fn locals(&self, container: NodeId) -> Option<&SymbolTable> {
        self.locals.get(&container)
    }

    /// Returns the binder diagnostics in report order.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Returns the node that made the file an external module, if any.
    #[must_use]
    pub const fn external_module_indicator(&self) -> Option<NodeId> {
        self.external_module_indicator
    }

    /// Returns the UMD globals declared by `export as namespace` in a declaration file.
    #[must_use]
    pub const fn global_exports(&self) -> &SymbolTable {
        &self.global_exports
    }

    /// Returns the wildcard ambient modules declared in the file.
    #[must_use]
    pub fn pattern_ambient_modules(&self) -> &[PatternAmbientModule] {
        &self.pattern_ambient_modules
    }

    /// Returns the file's control flow graph.
    #[must_use]
    pub const fn flow(&self) -> &FlowGraph {
        &self.flow
    }

    /// Returns the flow reaching a reference, statement, or function expression.
    #[must_use]
    pub fn flow_node_of(&self, node: NodeId) -> Option<FlowId> {
        self.flow_nodes.get(&node).copied()
    }

    /// Returns the flow at the end of a source file or a function-like body with a reachable end.
    #[must_use]
    pub fn end_flow_node_of(&self, node: NodeId) -> Option<FlowId> {
        self.end_flow_nodes.get(&node).copied()
    }

    /// Returns the flow after every return of a constructor or class static block.
    #[must_use]
    pub fn return_flow_node_of(&self, node: NodeId) -> Option<FlowId> {
        self.return_flow_nodes.get(&node).copied()
    }

    /// Returns the flow falling through the end of a non-final `case` or `default` clause.
    #[must_use]
    pub fn fallthrough_flow_node_of(&self, clause: NodeId) -> Option<FlowId> {
        self.fallthrough_flow_nodes.get(&clause).copied()
    }

    /// Returns the flags the binder computed for a node, such as reachability, implicit and
    /// explicit returns, and `this` usage; parser flags remain on the syntax tree.
    #[must_use]
    pub fn node_flags(&self, node: NodeId) -> NodeFlags {
        self.node_flags.get(&node).copied().unwrap_or_default()
    }

    /// Returns the names of classifiable declarations, such as classes, enums, and aliases.
    #[must_use]
    pub const fn classifiable_names(&self) -> &HashSet<String> {
        &self.classifiable_names
    }
}

/// Binds the declarations of `file`.
///
/// `options` supplies the compiler facts that can make a file a module without import or export
/// syntax.
#[must_use]
pub fn bind_source_file(
    file: &ParsedSourceFile,
    options: ExternalModuleIndicatorOptions,
) -> BoundFile {
    binder::Binder::new(file, file.external_module_indicator(options)).bind_file()
}
