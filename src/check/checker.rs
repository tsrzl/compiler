//! The checker: program-wide symbols, types, and diagnostics.

use std::collections::HashMap;

use super::program::{CheckDiagnostic, NodeRef};
use super::symbol_store::{SymbolRef, SymbolStore};
use super::type_table::TypeTable;
use super::types::TypeId;
use crate::ast::{CheckFlags, SymbolFlags};
use crate::program::ProgramFile;
use crate::symbols::SymbolTable;

/// Compiler options that change checking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckerOptions {
    /// Whether `null` and `undefined` are distinct types rather than members of every type.
    pub strict_null_checks: bool,
}

impl Default for CheckerOptions {
    fn default() -> Self {
        Self {
            strict_null_checks: true,
        }
    }
}

/// The checker's built-in symbols.
#[derive(Debug, Clone, Copy)]
pub(super) struct SpecialSymbols {
    pub(super) undefined: SymbolRef,
    pub(super) arguments: SymbolRef,
    #[expect(dead_code, reason = "name resolution reads it once ported")]
    pub(super) require: SymbolRef,
    pub(super) unknown: SymbolRef,
    pub(super) global_this: SymbolRef,
}

/// Checks a program, owning every type and transient symbol it creates.
#[derive(Debug)]
pub struct Checker<'program> {
    pub(super) files: &'program [ProgramFile],
    pub(super) types: TypeTable,
    pub(super) symbols: SymbolStore,
    pub(super) special: SpecialSymbols,
    pub(super) globals: SymbolTable<SymbolRef>,
    pub(super) merged_symbols: HashMap<SymbolRef, SymbolRef>,
    pub(super) type_node_links: HashMap<NodeRef, TypeId>,
    diagnostics: Vec<CheckDiagnostic>,
}

impl<'program> Checker<'program> {
    /// Creates a checker for `files` and merges their global declarations.
    #[must_use]
    pub fn new(files: &'program [ProgramFile], options: CheckerOptions) -> Self {
        let mut symbols = SymbolStore::default();
        let special = SpecialSymbols {
            undefined: symbols.create(SymbolFlags::PROPERTY, "undefined", CheckFlags::NONE),
            arguments: symbols.create(SymbolFlags::PROPERTY, "arguments", CheckFlags::NONE),
            require: symbols.create(SymbolFlags::PROPERTY, "require", CheckFlags::NONE),
            unknown: symbols.create(SymbolFlags::PROPERTY, "unknown", CheckFlags::NONE),
            global_this: symbols.create(SymbolFlags::MODULE, "globalThis", CheckFlags::READONLY),
        };
        let mut globals = SymbolTable::default();
        globals.insert("globalThis", special.global_this);
        let mut checker = Self {
            files,
            types: TypeTable::new(options.strict_null_checks),
            symbols,
            special,
            globals,
            merged_symbols: HashMap::new(),
            type_node_links: HashMap::new(),
            diagnostics: Vec::new(),
        };
        checker.initialize();
        checker
    }

    /// Returns the merged global symbols.
    #[must_use]
    pub const fn globals(&self) -> &SymbolTable<SymbolRef> {
        &self.globals
    }

    /// Returns the checker's types.
    #[must_use]
    pub const fn types(&self) -> &TypeTable {
        &self.types
    }

    /// Returns the diagnostics reported so far, in report order.
    #[must_use]
    pub fn diagnostics(&self) -> &[CheckDiagnostic] {
        &self.diagnostics
    }

    /// Returns a symbol's flags.
    #[must_use]
    pub fn symbol_flags(&self, symbol: SymbolRef) -> SymbolFlags {
        self.symbols.view(self.files, symbol).flags
    }

    /// Returns a symbol's name.
    #[must_use]
    pub fn symbol_name(&self, symbol: SymbolRef) -> &str {
        self.symbols.view(self.files, symbol).name
    }

    /// Returns a symbol's declarations.
    #[must_use]
    pub fn symbol_declarations(&self, symbol: SymbolRef) -> Vec<NodeRef> {
        self.symbols.view(self.files, symbol).declarations
    }

    /// Returns the export named `name`; the exports of `globalThis` are the globals.
    #[must_use]
    pub fn export_of(&self, symbol: SymbolRef, name: &str) -> Option<SymbolRef> {
        if symbol == self.special.global_this {
            return self.globals.get(name);
        }
        self.symbols.view(self.files, symbol).exports.get(name)
    }

    /// Returns the names of a symbol's exports in declaration order.
    #[must_use]
    pub fn export_names(&self, symbol: SymbolRef) -> Vec<String> {
        if symbol == self.special.global_this {
            return self
                .globals
                .iter()
                .map(|(name, _)| name.to_owned())
                .collect();
        }
        self.symbols
            .view(self.files, symbol)
            .exports
            .iter()
            .map(|(name, _)| name.to_owned())
            .collect()
    }

    /// Returns the symbol that `symbol` was merged into, or `symbol` itself.
    #[must_use]
    pub fn merged_symbol(&self, symbol: SymbolRef) -> SymbolRef {
        self.merged_symbols.get(&symbol).copied().unwrap_or(symbol)
    }

    /// Reports a diagnostic unless an identical one was already reported, and returns its index.
    pub(super) fn lookup_or_issue_error(&mut self, diagnostic: CheckDiagnostic) -> usize {
        if let Some(index) = self
            .diagnostics
            .iter()
            .position(|existing| existing.same_report(&diagnostic))
        {
            return index;
        }
        self.diagnostics.push(diagnostic);
        self.diagnostics.len() - 1
    }

    pub(super) fn add_diagnostic(&mut self, diagnostic: CheckDiagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub(super) fn diagnostic_mut(&mut self, index: usize) -> &mut CheckDiagnostic {
        &mut self.diagnostics[index]
    }
}
