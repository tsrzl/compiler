//! Type checking modeled on TypeScript-Go's `internal/checker` package.
//!
//! The checker consumes immutable parsed and bound files and owns every type it creates in a
//! per-compilation arena, so types refer to each other by ID rather than by reference.

mod checker;
mod compare;
mod feature_map;
mod flags;
mod globals;
mod name_resolution;
mod program;
mod symbol_store;
mod type_nodes;
mod type_references;
mod type_table;
mod types;
mod unresolved;

pub use checker::{Checker, CheckerOptions};
pub use compare::{compare_type_lists, compare_types};
pub use flags::{ObjectFlags, TypeFlags};
pub use program::{CheckDiagnostic, NodeRef};
pub use symbol_store::SymbolRef;
pub use type_table::{Intrinsics, TypeTable};
pub use types::{LiteralValue, Type, TypeData, TypeId};
