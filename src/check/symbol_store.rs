//! Program-wide symbol identity.
//!
//! Bound symbols stay in their files' arenas; the checker refers to them by file and ID. Symbols
//! the checker creates, such as clones that receive merged declarations, are transient and owned
//! by the checker.

use super::program::NodeRef;
use crate::ast::{CheckFlags, SymbolFlags};
use crate::program::ProgramFile;
use crate::symbols::{SymbolId, SymbolTable};

/// A symbol anywhere in a program.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SymbolRef {
    /// A symbol a file's binder created.
    Bound {
        /// The index of the file in the program.
        file: usize,
        /// The symbol within that file's binding.
        symbol: SymbolId,
    },
    /// A symbol the checker created.
    Transient(u32),
}

/// A symbol the checker created.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TransientSymbol {
    pub(super) flags: SymbolFlags,
    pub(super) check_flags: CheckFlags,
    pub(super) name: String,
    pub(super) declarations: Vec<NodeRef>,
    pub(super) value_declaration: Option<NodeRef>,
    pub(super) members: SymbolTable<SymbolRef>,
    pub(super) exports: SymbolTable<SymbolRef>,
    pub(super) parent: Option<SymbolRef>,
}

/// The transient symbols of one checker and read access to bound symbols.
#[derive(Debug, Default)]
pub(super) struct SymbolStore {
    transients: Vec<TransientSymbol>,
}

impl SymbolStore {
    /// Creates a transient symbol; the transient flag is always set.
    pub(super) fn create(
        &mut self,
        flags: SymbolFlags,
        name: impl Into<String>,
        check_flags: CheckFlags,
    ) -> SymbolRef {
        let id = u32::try_from(self.transients.len()).expect("transient symbol count fits in u32");
        self.transients.push(TransientSymbol {
            flags: flags | SymbolFlags::TRANSIENT,
            check_flags,
            name: name.into(),
            declarations: Vec::new(),
            value_declaration: None,
            members: SymbolTable::default(),
            exports: SymbolTable::default(),
            parent: None,
        });
        SymbolRef::Transient(id)
    }

    pub(super) fn transient(&self, id: u32) -> &TransientSymbol {
        &self.transients[id as usize]
    }

    pub(super) fn transient_mut(&mut self, id: u32) -> &mut TransientSymbol {
        &mut self.transients[id as usize]
    }

    /// Returns a transient copy of `symbol` with its declarations, parent, members, and exports.
    pub(super) fn clone_symbol(&mut self, files: &[ProgramFile], symbol: SymbolRef) -> SymbolRef {
        let view = self.view(files, symbol);
        let clone = TransientSymbol {
            flags: view.flags | SymbolFlags::TRANSIENT,
            check_flags: CheckFlags::NONE,
            name: view.name.to_owned(),
            declarations: view.declarations,
            value_declaration: view.value_declaration,
            members: view.members,
            exports: view.exports,
            parent: view.parent,
        };
        let id = u32::try_from(self.transients.len()).expect("transient symbol count fits in u32");
        self.transients.push(clone);
        SymbolRef::Transient(id)
    }

    /// Returns an owned view of any symbol's declaration facts.
    pub(super) fn view<'a>(
        &'a self,
        files: &'a [ProgramFile],
        symbol: SymbolRef,
    ) -> SymbolView<'a> {
        match symbol {
            SymbolRef::Bound { file, symbol } => {
                let bound = files[file].bound().symbols().symbol(symbol);
                let to_ref = |symbol| SymbolRef::Bound { file, symbol };
                let to_node = |node| NodeRef { file, node };
                SymbolView {
                    flags: bound.flags,
                    name: &bound.name,
                    declarations: bound.declarations.iter().copied().map(to_node).collect(),
                    value_declaration: bound.value_declaration.map(to_node),
                    members: convert_table(&bound.members, to_ref),
                    exports: convert_table(&bound.exports, to_ref),
                    parent: bound.parent.map(to_ref),
                }
            }
            SymbolRef::Transient(id) => {
                let transient = self.transient(id);
                SymbolView {
                    flags: transient.flags,
                    name: &transient.name,
                    declarations: transient.declarations.clone(),
                    value_declaration: transient.value_declaration,
                    members: transient.members.clone(),
                    exports: transient.exports.clone(),
                    parent: transient.parent,
                }
            }
        }
    }
}

/// The declaration facts of a bound or transient symbol.
#[derive(Debug, Clone)]
pub(super) struct SymbolView<'a> {
    pub(super) flags: SymbolFlags,
    pub(super) name: &'a str,
    pub(super) declarations: Vec<NodeRef>,
    pub(super) value_declaration: Option<NodeRef>,
    pub(super) members: SymbolTable<SymbolRef>,
    pub(super) exports: SymbolTable<SymbolRef>,
    pub(super) parent: Option<SymbolRef>,
}

fn convert_table(
    table: &SymbolTable,
    to_ref: impl Fn(SymbolId) -> SymbolRef,
) -> SymbolTable<SymbolRef> {
    let mut converted = SymbolTable::default();
    for (name, symbol) in table.iter() {
        converted.insert(name, to_ref(symbol));
    }
    converted
}
