//! The files a checker reads: each parsed file with its binding.

use crate::ast::NodeId;
use crate::bind::{BoundFile, bind_source_file};
use crate::diagnostics::Diagnostic;
use crate::parser::{ExternalModuleIndicatorOptions, ParsedSourceFile};
use crate::scanner::error_range_for_node;

/// A parsed and bound source file in a program.
#[derive(Debug, Clone)]
pub struct ProgramFile {
    parsed: ParsedSourceFile,
    bound: BoundFile,
}

impl ProgramFile {
    /// Binds `parsed` with default module detection.
    #[must_use]
    pub fn new(parsed: ParsedSourceFile) -> Self {
        Self::with_options(parsed, ExternalModuleIndicatorOptions::default())
    }

    /// Binds `parsed` with the given module detection options.
    #[must_use]
    pub fn with_options(parsed: ParsedSourceFile, options: ExternalModuleIndicatorOptions) -> Self {
        let bound = bind_source_file(&parsed, options);
        Self { parsed, bound }
    }

    /// Returns the parsed file.
    #[must_use]
    pub const fn parsed(&self) -> &ParsedSourceFile {
        &self.parsed
    }

    /// Returns the file's binding.
    #[must_use]
    pub const fn bound(&self) -> &BoundFile {
        &self.bound
    }

    /// Returns whether the file is an external module rather than a global script.
    #[must_use]
    pub const fn is_external_module(&self) -> bool {
        self.bound.external_module_indicator().is_some()
    }

    /// Returns a diagnostic covering `node`'s error range.
    pub(super) fn diagnostic_for_node(
        &self,
        node: NodeId,
        message: crate::diagnostics::Message,
        arguments: &[&str],
    ) -> Diagnostic {
        let parsed = &self.parsed;
        let range =
            error_range_for_node(parsed.ast(), parsed.text(), parsed.language_variant(), node);
        Diagnostic::new(message, range, arguments)
    }
}

/// A node in a specific program file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeRef {
    /// The index of the file in the program.
    pub file: usize,
    /// The node within that file.
    pub node: NodeId,
}

/// A checker diagnostic attributed to a program file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckDiagnostic {
    file: usize,
    diagnostic: Diagnostic,
    related: Vec<CheckDiagnostic>,
}

impl CheckDiagnostic {
    pub(super) const fn new(file: usize, diagnostic: Diagnostic) -> Self {
        Self {
            file,
            diagnostic,
            related: Vec::new(),
        }
    }

    /// Returns the index of the file the diagnostic reports on.
    #[must_use]
    pub const fn file(&self) -> usize {
        self.file
    }

    /// Returns the positioned diagnostic.
    #[must_use]
    pub const fn diagnostic(&self) -> &Diagnostic {
        &self.diagnostic
    }

    /// Returns related locations, possibly in other files.
    #[must_use]
    pub fn related(&self) -> &[CheckDiagnostic] {
        &self.related
    }

    pub(super) fn add_related(&mut self, related: Self) {
        self.related.push(related);
    }

    /// Returns whether the diagnostic reports the same message, arguments, and location.
    pub(super) fn same_report(&self, other: &Self) -> bool {
        self.file == other.file
            && self.diagnostic.range() == other.diagnostic.range()
            && self.diagnostic.text() == other.diagnostic.text()
    }
}
