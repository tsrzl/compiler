//! A declarative macro for TypeScript-Go compatible bit-flag sets.

/// Declares a copyable bit-flag set with TypeScript-Go bit values and set operators.
macro_rules! flags_type {
    ($(#[$meta:meta])* $name:ident { $($(#[$flag_meta:meta])* $flag:ident = $value:expr;)* }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
        pub struct $name(u32);

        #[allow(missing_docs)]
        impl $name {
            $($(#[$flag_meta])* pub const $flag: Self = Self($value);)*

            /// Returns the TypeScript-Go bit values.
            #[must_use]
            pub const fn bits(self) -> u32 {
                self.0
            }

            /// Returns whether any flag in `other` is set.
            #[must_use]
            pub const fn intersects(self, other: Self) -> bool {
                self.0 & other.0 != 0
            }

            /// Returns whether every flag in `other` is set.
            #[must_use]
            pub const fn contains(self, other: Self) -> bool {
                self.0 & other.0 == other.0
            }

            /// Returns these flags without any flag in `other`.
            #[must_use]
            pub const fn without(self, other: Self) -> Self {
                Self(self.0 & !other.0)
            }
        }

        impl std::ops::BitOr for $name {
            type Output = Self;

            fn bitor(self, other: Self) -> Self {
                Self(self.0 | other.0)
            }
        }

        impl std::ops::BitOrAssign for $name {
            fn bitor_assign(&mut self, other: Self) {
                self.0 |= other.0;
            }
        }

        impl std::ops::BitAnd for $name {
            type Output = Self;

            fn bitand(self, other: Self) -> Self {
                Self(self.0 & other.0)
            }
        }
    };
}

pub(crate) use flags_type;
