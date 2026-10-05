//! Read-only analyzer extension points for compiler diagnostics.

use crate::syntax::{Diagnostic, SyntaxTree};

/// Analyzes an immutable view of a completed compilation pass.
pub trait Analyzer: Send + Sync {
    /// Returns diagnostics owned by this analyzer.
    fn analyze(&self, context: &AnalysisContext<'_>) -> Vec<Diagnostic>;
}

/// An immutable view supplied to registered analyzers.
#[derive(Debug, Clone, Copy)]
pub struct AnalysisContext<'compilation> {
    syntax_trees: &'compilation [SyntaxTree],
    diagnostics: &'compilation [Diagnostic],
}

impl<'compilation> AnalysisContext<'compilation> {
    pub(crate) const fn new(
        syntax_trees: &'compilation [SyntaxTree],
        diagnostics: &'compilation [Diagnostic],
    ) -> Self {
        Self {
            syntax_trees,
            diagnostics,
        }
    }

    /// Returns the immutable syntax trees in the compilation pass.
    #[must_use]
    pub const fn syntax_trees(&self) -> &'compilation [SyntaxTree] {
        self.syntax_trees
    }

    /// Returns diagnostics produced by parsing and type checking.
    #[must_use]
    pub const fn diagnostics(&self) -> &'compilation [Diagnostic] {
        self.diagnostics
    }
}
