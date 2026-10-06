//! Binder symbols modeled on TypeScript-Go's `internal/ast/symbol.go`.
//!
//! Symbols live in a per-binding [`SymbolArena`] and refer to each other and to declarations by
//! explicit IDs, so bound results never hold references into the syntax tree.

use std::collections::HashMap;

use crate::ast::{NodeId, SymbolFlags};

/// Marks binder-internal symbol names.
///
/// TypeScript-Go uses the invalid UTF-8 byte `0xFE`. Rust strings must be valid UTF-8, so the
/// noncharacter U+FFFE stands in: it cannot occur in an identifier, and a string-literal property
/// name collides only when it begins with U+FFFE followed by an internal name.
pub const INTERNAL_SYMBOL_NAME_PREFIX: &str = "\u{FFFE}";

/// Call signatures.
pub const INTERNAL_SYMBOL_NAME_CALL: &str = "\u{FFFE}call";
/// Constructor implementations.
pub const INTERNAL_SYMBOL_NAME_CONSTRUCTOR: &str = "\u{FFFE}constructor";
/// Constructor signatures.
pub const INTERNAL_SYMBOL_NAME_NEW: &str = "\u{FFFE}new";
/// Index signatures.
pub const INTERNAL_SYMBOL_NAME_INDEX: &str = "\u{FFFE}index";
/// Module `export *` declarations.
pub const INTERNAL_SYMBOL_NAME_EXPORT_STAR: &str = "\u{FFFE}export";
/// Global self-reference.
pub const INTERNAL_SYMBOL_NAME_GLOBAL: &str = "\u{FFFE}global";
/// A missing symbol.
pub const INTERNAL_SYMBOL_NAME_MISSING: &str = "\u{FFFE}missing";
/// An anonymous type literal.
pub const INTERNAL_SYMBOL_NAME_TYPE: &str = "\u{FFFE}type";
/// An anonymous object literal.
pub const INTERNAL_SYMBOL_NAME_OBJECT: &str = "\u{FFFE}object";
/// An anonymous JSX attributes object literal.
pub const INTERNAL_SYMBOL_NAME_JSX_ATTRIBUTES: &str = "\u{FFFE}jsxAttributes";
/// An unnamed class expression.
pub const INTERNAL_SYMBOL_NAME_CLASS: &str = "\u{FFFE}class";
/// An unnamed function expression.
pub const INTERNAL_SYMBOL_NAME_FUNCTION: &str = "\u{FFFE}function";
/// A computed property name with a dynamic name.
pub const INTERNAL_SYMBOL_NAME_COMPUTED: &str = "\u{FFFE}computed";
/// Assignment declarations.
pub const INTERNAL_SYMBOL_NAME_ASSIGNMENT_DECLARATION: &str = "\u{FFFE}assignment";
/// Instantiation expressions.
pub const INTERNAL_SYMBOL_NAME_INSTANTIATION_EXPRESSION: &str = "\u{FFFE}instantiationExpression";
/// Import attributes.
pub const INTERNAL_SYMBOL_NAME_IMPORT_ATTRIBUTES: &str = "\u{FFFE}importAttributes";
/// The export assignment symbol.
pub const INTERNAL_SYMBOL_NAME_EXPORT_EQUALS: &str = "export=";
/// The default export symbol.
pub const INTERNAL_SYMBOL_NAME_DEFAULT: &str = "default";
/// The `this` symbol.
pub const INTERNAL_SYMBOL_NAME_THIS: &str = "this";
/// The `CommonJS` `module.exports` symbol.
pub const INTERNAL_SYMBOL_NAME_MODULE_EXPORTS: &str = "module.exports";

/// A symbol's identity within its [`SymbolArena`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolId(u32);

impl SymbolId {
    /// Returns the symbol's zero-based index in its arena.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

/// A named entity declared by one or more declarations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    /// The kinds of declarations merged into this symbol.
    pub flags: SymbolFlags,
    /// The symbol name; internal names begin with [`INTERNAL_SYMBOL_NAME_PREFIX`].
    pub name: String,
    /// The declarations that contribute to this symbol.
    pub declarations: Vec<NodeId>,
    /// The first value declaration, if any.
    pub value_declaration: Option<NodeId>,
    /// Class, interface, and literal members.
    pub members: SymbolTable,
    /// Module and namespace exports.
    pub exports: SymbolTable,
    /// The containing symbol.
    pub parent: Option<SymbolId>,
    /// The export symbol paired with a local declaration symbol.
    pub export_symbol: Option<SymbolId>,
}

impl Symbol {
    /// Returns whether the symbol is an external module, whose name is a quoted specifier.
    #[must_use]
    pub fn is_external_module(&self) -> bool {
        self.flags.intersects(SymbolFlags::MODULE) && self.name.starts_with('"')
    }
}

/// Symbols owned by one binding, addressed by [`SymbolId`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SymbolArena {
    symbols: Vec<Symbol>,
}

impl SymbolArena {
    /// Creates a symbol with no declarations and returns its identity.
    ///
    /// # Panics
    ///
    /// Panics when the arena already holds `u32::MAX` symbols.
    pub fn create(&mut self, flags: SymbolFlags, name: impl Into<String>) -> SymbolId {
        let id = SymbolId(u32::try_from(self.symbols.len()).expect("symbol count fits in u32"));
        self.symbols.push(Symbol {
            flags,
            name: name.into(),
            declarations: Vec::new(),
            value_declaration: None,
            members: SymbolTable::default(),
            exports: SymbolTable::default(),
            parent: None,
            export_symbol: None,
        });
        id
    }

    /// Returns the symbol with identity `id`.
    ///
    /// # Panics
    ///
    /// Panics when `id` was not produced by this arena.
    #[must_use]
    pub fn symbol(&self, id: SymbolId) -> &Symbol {
        &self.symbols[id.0 as usize]
    }

    /// Returns the symbol with identity `id` for mutation.
    ///
    /// # Panics
    ///
    /// Panics when `id` was not produced by this arena.
    pub fn symbol_mut(&mut self, id: SymbolId) -> &mut Symbol {
        &mut self.symbols[id.0 as usize]
    }

    /// Returns the number of symbols.
    #[must_use]
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Returns whether the arena holds no symbols.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// Returns the symbol's flags combined with those of its export symbol.
    ///
    /// See the comment on `declareModuleMember` in TypeScript-Go's binder.
    #[must_use]
    pub fn combined_local_and_export_symbol_flags(&self, id: SymbolId) -> SymbolFlags {
        let symbol = self.symbol(id);
        symbol.export_symbol.map_or(symbol.flags, |export| {
            symbol.flags | self.symbol(export).flags
        })
    }
}

/// Symbols by name, iterated in insertion order.
///
/// Bound files store [`SymbolId`]s; the checker stores references that span files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolTable<S = SymbolId> {
    entries: Vec<(String, S)>,
    index: HashMap<String, usize>,
}

impl<S> Default for SymbolTable<S> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            index: HashMap::new(),
        }
    }
}

impl<S: Copy> SymbolTable<S> {
    /// Returns the symbol named `name`.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<S> {
        self.index
            .get(name)
            .map(|&position| self.entries[position].1)
    }

    /// Sets the symbol named `name`, keeping its original position when it is replaced.
    pub fn insert(&mut self, name: impl Into<String>, symbol: S) {
        let name = name.into();
        if let Some(&position) = self.index.get(&name) {
            self.entries[position].1 = symbol;
        } else {
            self.index.insert(name.clone(), self.entries.len());
            self.entries.push((name, symbol));
        }
    }

    /// Returns the names and symbols in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, S)> {
        self.entries
            .iter()
            .map(|(name, symbol)| (name.as_str(), *symbol))
    }

    /// Returns the number of symbols.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the table holds no symbols.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Replaces an internal-name prefix with `__`.
#[must_use]
pub fn escape_internal_symbol_name(name: &str) -> String {
    name.strip_prefix(INTERNAL_SYMBOL_NAME_PREFIX)
        .map_or_else(|| name.to_owned(), |rest| format!("__{rest}"))
}

/// Converts a binder symbol name into TypeScript's escaped `__String` form.
///
/// Internal names gain a `__` prefix, and user names that already begin with `__` gain an extra
/// leading underscore so they stay distinct from internal names.
#[must_use]
pub fn escape_symbol_name(name: &str) -> String {
    if let Some(rest) = name.strip_prefix(INTERNAL_SYMBOL_NAME_PREFIX) {
        format!("__{rest}")
    } else if name.starts_with("__") {
        format!("_{name}")
    } else {
        name.to_owned()
    }
}
