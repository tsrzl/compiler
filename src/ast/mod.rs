//! The arena syntax tree modeled on TypeScript-Go's `internal/ast` package.

mod flags;
mod flags_type;
mod kind;

pub use flags::{ModifierFlags, NodeFlags, TokenFlags};
pub use kind::SyntaxKind;

impl SyntaxKind {
    /// Returns whether the kind is a keyword that cannot be used as an identifier.
    #[must_use]
    pub fn is_reserved_word(self) -> bool {
        (Self::FIRST_RESERVED_WORD..=Self::LAST_RESERVED_WORD).contains(&self)
    }

    /// Returns whether the kind is an identifier or any keyword.
    #[must_use]
    pub fn is_identifier_or_keyword(self) -> bool {
        self >= Self::Identifier
    }
}
