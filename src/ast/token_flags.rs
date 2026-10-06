//! Flags describing how a token was written, mirroring TypeScript-Go's `ast.TokenFlags`.

use std::ops::{BitAnd, BitOr, BitOrAssign};

/// A set of token flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct TokenFlags(u32);

#[allow(missing_docs)]
impl TokenFlags {
    pub const NONE: Self = Self(0);
    pub const PRECEDING_LINE_BREAK: Self = Self(1 << 0);
    pub const PRECEDING_JSDOC_COMMENT: Self = Self(1 << 1);
    pub const UNTERMINATED: Self = Self(1 << 2);
    pub const EXTENDED_UNICODE_ESCAPE: Self = Self(1 << 3);
    pub const SCIENTIFIC: Self = Self(1 << 4);
    pub const OCTAL: Self = Self(1 << 5);
    pub const HEX_SPECIFIER: Self = Self(1 << 6);
    pub const BINARY_SPECIFIER: Self = Self(1 << 7);
    pub const OCTAL_SPECIFIER: Self = Self(1 << 8);
    pub const CONTAINS_SEPARATOR: Self = Self(1 << 9);
    pub const UNICODE_ESCAPE: Self = Self(1 << 10);
    pub const CONTAINS_INVALID_ESCAPE: Self = Self(1 << 11);
    pub const HEX_ESCAPE: Self = Self(1 << 12);
    pub const CONTAINS_LEADING_ZERO: Self = Self(1 << 13);
    pub const CONTAINS_INVALID_SEPARATOR: Self = Self(1 << 14);
    pub const PRECEDING_JSDOC_LEADING_ASTERISKS: Self = Self(1 << 15);
    pub const SINGLE_QUOTE: Self = Self(1 << 16);
    pub const PRECEDING_JSDOC_WITH_DEPRECATED: Self = Self(1 << 17);
    pub const PRECEDING_JSDOC_WITH_SEE_OR_LINK: Self = Self(1 << 18);
    pub const BINARY_OR_OCTAL_SPECIFIER: Self =
        Self(Self::BINARY_SPECIFIER.0 | Self::OCTAL_SPECIFIER.0);
    pub const WITH_SPECIFIER: Self =
        Self(Self::HEX_SPECIFIER.0 | Self::BINARY_OR_OCTAL_SPECIFIER.0);
    pub const IS_INVALID: Self = Self(
        Self::OCTAL.0
            | Self::CONTAINS_LEADING_ZERO.0
            | Self::CONTAINS_INVALID_SEPARATOR.0
            | Self::CONTAINS_INVALID_ESCAPE.0,
    );

    /// Returns the flags as TypeScript-Go's `ast.TokenFlags` bit values.
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Returns whether any flag in `other` is set.
    #[must_use]
    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}

impl BitOr for TokenFlags {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl BitOrAssign for TokenFlags {
    fn bitor_assign(&mut self, other: Self) {
        self.0 |= other.0;
    }
}

impl BitAnd for TokenFlags {
    type Output = Self;

    fn bitand(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}
