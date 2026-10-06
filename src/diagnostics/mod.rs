//! TypeScript diagnostic messages, generated from the pinned TypeScript-Go message table.

mod messages;

pub use messages::*;

/// The severity class of a diagnostic message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    /// A warning.
    Warning,
    /// An error.
    Error,
    /// An editor suggestion.
    Suggestion,
    /// An informational message.
    Message,
}

/// A diagnostic message template with its stable TypeScript code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Message {
    code: u32,
    category: Category,
    text: &'static str,
    reports_unnecessary: bool,
    elided_in_compatibility_pyramid: bool,
    reports_deprecated: bool,
}

impl Message {
    const fn new(code: u32, category: Category, text: &'static str) -> Self {
        Self {
            code,
            category,
            text,
            reports_unnecessary: false,
            elided_in_compatibility_pyramid: false,
            reports_deprecated: false,
        }
    }

    const fn reporting_unnecessary(mut self) -> Self {
        self.reports_unnecessary = true;
        self
    }

    const fn elided_in_compatibility_pyramid(mut self) -> Self {
        self.elided_in_compatibility_pyramid = true;
        self
    }

    const fn reporting_deprecated(mut self) -> Self {
        self.reports_deprecated = true;
        self
    }

    /// Returns the TypeScript diagnostic code.
    #[must_use]
    pub const fn code(self) -> u32 {
        self.code
    }

    /// Returns the message category.
    #[must_use]
    pub const fn category(self) -> Category {
        self.category
    }

    /// Returns the unformatted message template.
    #[must_use]
    pub const fn text(self) -> &'static str {
        self.text
    }

    /// Returns whether the diagnostic marks unnecessary code.
    #[must_use]
    pub const fn reports_unnecessary(self) -> bool {
        self.reports_unnecessary
    }

    /// Returns whether the diagnostic is elided from the compatibility pyramid.
    #[must_use]
    pub const fn is_elided_in_compatibility_pyramid(self) -> bool {
        self.elided_in_compatibility_pyramid
    }

    /// Returns whether the diagnostic marks deprecated usage.
    #[must_use]
    pub const fn reports_deprecated(self) -> bool {
        self.reports_deprecated
    }

    /// Formats the template, replacing each `{n}` placeholder with the `n`th argument.
    #[must_use]
    pub fn format(self, arguments: &[&str]) -> String {
        let mut formatted = String::with_capacity(self.text.len());
        let mut rest = self.text;
        while let Some(open) = rest.find('{') {
            formatted.push_str(&rest[..open]);
            let after_open = &rest[open + 1..];
            let argument = after_open.find('}').and_then(|close| {
                let index = after_open[..close].parse::<usize>().ok()?;
                Some((arguments.get(index)?, close))
            });
            match argument {
                Some((argument, close)) => {
                    formatted.push_str(argument);
                    rest = &after_open[close + 1..];
                }
                None => {
                    formatted.push('{');
                    rest = after_open;
                }
            }
        }
        formatted.push_str(rest);
        formatted
    }
}
