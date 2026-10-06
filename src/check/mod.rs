//! Type checking modeled on TypeScript-Go's `internal/checker` package.
//!
//! The checker consumes immutable parsed and bound files and owns every type it creates in a
//! per-compilation arena, so types refer to each other by ID rather than by reference.

mod flags;

pub use flags::{ObjectFlags, TypeFlags};
