use crate::enum_values::{EnumValue, values_for_enum};
use crate::syntax::{Diagnostic, EnumDeclaration};

pub(super) fn check_enum_declaration(declaration: &EnumDeclaration) -> Vec<Diagnostic> {
    declaration
        .members()
        .iter()
        .zip(values_for_enum(declaration))
        .filter(|(member, value)| {
            member.initializer().is_none() && !matches!(value, EnumValue::Number(_))
        })
        .map(|(member, _)| {
            Diagnostic::new(
                1061,
                "Enum member must have initializer.",
                member.name_span(),
            )
        })
        .collect()
}
