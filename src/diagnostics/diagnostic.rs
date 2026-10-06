//! A positioned diagnostic reported by a compiler phase.

use std::ops::Range;

use super::{Category, Message};

/// A diagnostic located by UTF-8 byte offsets within one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    message: Message,
    category: Category,
    range: Range<usize>,
    arguments: Vec<String>,
    related: Vec<Diagnostic>,
}

impl Diagnostic {
    /// Creates a diagnostic with the message's own category.
    #[must_use]
    pub fn new(message: Message, range: Range<usize>, arguments: &[&str]) -> Self {
        Self {
            message,
            category: message.category(),
            range,
            arguments: arguments
                .iter()
                .map(|&argument| argument.to_owned())
                .collect(),
            related: Vec::new(),
        }
    }

    /// Returns the diagnostic message template.
    #[must_use]
    pub const fn message(&self) -> Message {
        self.message
    }

    /// Returns the TypeScript diagnostic code.
    #[must_use]
    pub const fn code(&self) -> u32 {
        self.message.code()
    }

    /// Returns the reported category.
    #[must_use]
    pub const fn category(&self) -> Category {
        self.category
    }

    /// Returns the UTF-8 byte range the diagnostic covers.
    #[must_use]
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }

    /// Returns the formatted message text.
    #[must_use]
    pub fn text(&self) -> String {
        let arguments = self
            .arguments
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        self.message.format(&arguments)
    }

    /// Returns related locations that explain this diagnostic.
    #[must_use]
    pub fn related(&self) -> &[Diagnostic] {
        &self.related
    }

    /// Adds a related location.
    pub fn add_related(&mut self, related: Diagnostic) {
        self.related.push(related);
    }
}
