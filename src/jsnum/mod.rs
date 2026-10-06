//! ECMAScript number conversions, matching TypeScript-Go's `internal/jsnum` package.

mod bigint;
mod format;
mod parse;

pub use bigint::parse_pseudo_big_int;
pub use format::number_to_string;
pub use parse::from_string;
