//! The library crate for the TSRZL TypeScript compiler.
//!
//! TypeScript compiler core built from the Rust standard library.
//!
//! Compiler behavior is added in focused, behavior-tested increments.

pub mod analyzer;
pub mod ast;
mod binder;
pub mod compiler;
pub mod diagnostics;
mod emit;
mod enum_values;
pub mod generator;
pub mod jsnum;
pub mod module_resolver;
// Helpers ported ahead of their callers; remove once statement and expression parsing land.
#[allow(dead_code)]
pub mod parser;
pub mod scanner;
pub mod source_file;
pub mod source_text;
pub mod syntax;
pub mod tspath;
mod type_checker;
mod type_system;
