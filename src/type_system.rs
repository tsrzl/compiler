//! Shared type-spelling operations for compiler phases.

pub(crate) fn array_type(element_types: impl IntoIterator<Item = String>) -> String {
    let mut unique_element_types = Vec::new();
    for element_type in element_types {
        if !unique_element_types.contains(&element_type) {
            unique_element_types.push(element_type);
        }
    }

    let element_type = match unique_element_types.as_slice() {
        [] => "any".to_owned(),
        [element_type] => element_type.clone(),
        element_types => format!("({})", element_types.join(" | ")),
    };
    format!("{element_type}[]")
}

pub(crate) fn numeric_literal_type(spelling: &str) -> &'static str {
    if spelling.ends_with('n') {
        "bigint"
    } else {
        "number"
    }
}
