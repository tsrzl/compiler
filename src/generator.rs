//! Immutable input and owned output contracts for source generators.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::syntax::{Diagnostic, SyntaxTree};

/// Generates owned virtual source files from an immutable compilation view.
pub trait Generator: Send + Sync {
    /// Returns generated sources and any diagnostics reported by the generator.
    fn generate(&self, context: &GenerationContext<'_>) -> GenerationOutput;
}

/// An immutable view supplied to source generators.
#[derive(Debug, Clone, Copy)]
pub struct GenerationContext<'compilation> {
    syntax_trees: &'compilation [SyntaxTree],
    diagnostics: &'compilation [Diagnostic],
}

impl<'compilation> GenerationContext<'compilation> {
    pub(crate) const fn new(
        syntax_trees: &'compilation [SyntaxTree],
        diagnostics: &'compilation [Diagnostic],
    ) -> Self {
        Self {
            syntax_trees,
            diagnostics,
        }
    }

    /// Returns the immutable syntax trees from the generator input pass.
    #[must_use]
    pub const fn syntax_trees(&self) -> &'compilation [SyntaxTree] {
        self.syntax_trees
    }

    /// Returns diagnostics produced before source generation.
    #[must_use]
    pub const fn diagnostics(&self) -> &'compilation [Diagnostic] {
        self.diagnostics
    }
}

/// A generated source file owned by the generator result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedSource {
    path: PathBuf,
    text: Arc<str>,
}

impl GeneratedSource {
    /// Creates a generated source file with an owned path and text.
    #[must_use]
    pub fn new(path: impl AsRef<Path>, text: impl Into<Arc<str>>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            text: text.into(),
        }
    }

    /// Returns the generated source path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the generated source text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// The owned results returned by one generator.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GenerationOutput {
    sources: Vec<GeneratedSource>,
    diagnostics: Vec<Diagnostic>,
}

impl GenerationOutput {
    /// Creates an empty generator output.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            sources: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    /// Adds an owned generated source and returns the updated output.
    #[must_use]
    pub fn with_source(mut self, source: GeneratedSource) -> Self {
        self.sources.push(source);
        self
    }

    /// Adds an owned diagnostic and returns the updated output.
    #[must_use]
    pub fn with_diagnostic(mut self, diagnostic: Diagnostic) -> Self {
        self.diagnostics.push(diagnostic);
        self
    }

    /// Returns generated sources in generator order.
    #[must_use]
    pub fn sources(&self) -> &[GeneratedSource] {
        &self.sources
    }

    /// Returns diagnostics in generator order.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}
