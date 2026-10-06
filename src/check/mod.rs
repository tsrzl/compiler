//! Type checking modeled on TypeScript-Go's `internal/checker` package.
//!
//! The checker consumes immutable parsed and bound files and owns every type it creates in a
//! per-compilation arena, so types refer to each other by ID rather than by reference.

mod compare;
mod flags;
mod type_table;
mod types;

pub use compare::{compare_type_lists, compare_types};
pub use flags::{ObjectFlags, TypeFlags};
pub use type_table::{Intrinsics, TypeTable};
pub use types::{LiteralValue, Type, TypeData, TypeId};
