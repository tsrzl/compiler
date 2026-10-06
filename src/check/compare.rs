//! The total order of types, modeled on TypeScript-Go's `CompareTypes`.
//!
//! Union constituents are kept in this order, so union identity and display order are
//! deterministic rather than dependent on type creation order.

use std::cmp::Ordering;

use super::TypeFlags;
use super::type_table::TypeTable;
use super::types::{LiteralValue, TypeData, TypeId};

/// Orders two types: by sort-order flags, then by kind-specific data, then by identity.
#[must_use]
pub fn compare_types(table: &TypeTable, left: TypeId, right: TypeId) -> Ordering {
    if left == right {
        return Ordering::Equal;
    }
    let (first, second) = (table.get(left), table.get(right));
    sort_order_flags(first.flags())
        .cmp(&sort_order_flags(second.flags()))
        .then_with(|| match (first.data(), second.data()) {
            (
                TypeData::Union { types, origin },
                TypeData::Union {
                    types: other_types,
                    origin: other_origin,
                },
            ) => match (origin, other_origin) {
                (None, None) => compare_type_lists(table, types, other_types),
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(origin), Some(other_origin)) => compare_types(table, *origin, *other_origin),
            },
            (TypeData::Literal { value, .. }, TypeData::Literal { value: other, .. }) => {
                compare_literal_values(value, other)
            }
            // Intrinsic types are distinguished only by identity.
            _ => Ordering::Equal,
        })
        .then_with(|| left.cmp(&right))
}

/// Orders type lists by length and then element by element.
#[must_use]
pub fn compare_type_lists(table: &TypeTable, left: &[TypeId], right: &[TypeId]) -> Ordering {
    left.len().cmp(&right.len()).then_with(|| {
        left.iter()
            .zip(right)
            .map(|(&first, &second)| compare_types(table, first, second))
            .find(|ordering| ordering.is_ne())
            .unwrap_or(Ordering::Equal)
    })
}

fn sort_order_flags(flags: TypeFlags) -> u32 {
    // Enum-like unit types sort as enums, then by their symbols.
    if flags.intersects(TypeFlags::ENUM_LITERAL | TypeFlags::ENUM)
        && !flags.intersects(TypeFlags::UNION)
    {
        TypeFlags::ENUM.bits()
    } else {
        flags.bits()
    }
}

fn compare_literal_values(left: &LiteralValue, right: &LiteralValue) -> Ordering {
    match (left, right) {
        (LiteralValue::String(left), LiteralValue::String(right)) => left.cmp(right),
        (LiteralValue::Number(left), LiteralValue::Number(right)) => compare_numbers(*left, *right),
        // `false` sorts before `true`.
        (LiteralValue::Boolean(left), LiteralValue::Boolean(right)) => left.cmp(right),
        _ => Ordering::Equal,
    }
}

/// Orders numbers as Go's `cmp.Compare` does: NaN sorts first and equals NaN, and `-0` equals `0`.
fn compare_numbers(left: f64, right: f64) -> Ordering {
    match (left.is_nan(), right.is_nan()) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (false, false) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
    }
}
