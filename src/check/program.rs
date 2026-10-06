//! Checker locations and diagnostics that span a program's files.

use crate::ast::NodeId;
use crate::diagnostics::Diagnostic;

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
