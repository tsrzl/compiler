//! The library crate for the TSRZL TypeScript compiler.
//!
//! TypeScript compiler core built from the Rust standard library.
//!
//! Compiler behavior is added in focused, behavior-tested increments.

pub mod analyzer;
pub mod ast;
mod binder;
pub mod compiler;
mod emit;
mod enum_values;
pub mod generator;
pub mod module_resolver;
pub mod source_file;
pub mod source_text;
pub mod syntax;
mod type_checker;
mod type_system;
