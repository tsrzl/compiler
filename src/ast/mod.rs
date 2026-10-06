//! The arena syntax tree modeled on TypeScript-Go's `internal/ast` package.

mod kind;
mod token_flags;

pub use kind::SyntaxKind;
pub use token_flags::TokenFlags;

impl SyntaxKind {
    /// Returns whether the kind is a keyword that cannot be used as an identifier.
    #[must_use]
    pub fn is_reserved_word(self) -> bool {
        (Self::FIRST_RESERVED_WORD..=Self::LAST_RESERVED_WORD).contains(&self)
    }
}
