//! The library crate for the TSRZL TypeScript compiler.
//!
//! TypeScript compiler core built from the Rust standard library.
//!
//! Compiler behavior is added in focused, behavior-tested increments.

pub mod analyzer;
pub mod ast;
pub mod bind;
mod binder;
pub mod bundled;
pub mod check;
pub mod compiler;
pub mod diagnostics;
mod emit;
mod enum_values;
pub mod generator;
pub mod jsnum;
pub mod module_resolver;
pub mod parser;
pub mod program;
pub mod scanner;
pub mod source_file;
pub mod source_text;
pub mod spelling;
pub mod symbols;
pub mod syntax;
pub mod tspath;
mod type_checker;
mod type_system;
