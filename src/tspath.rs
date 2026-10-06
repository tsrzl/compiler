//! File-name rules, matching TypeScript-Go's `internal/tspath` package.

const SUPPORTED_DECLARATION_EXTENSIONS: [&str; 3] = [".d.ts", ".d.cts", ".d.mts"];

/// Returns the final path component of `path`.
#[must_use]
pub fn base_file_name(path: &str) -> &str {
    let trimmed = path.trim_end_matches(['/', '\\']);
    trimmed.rsplit(['/', '\\']).next().unwrap_or(trimmed)
}

/// Returns the declaration-file extension of `file_name`, such as `.d.ts` or `.d.css.ts`.
#[must_use]
pub fn declaration_file_extension(file_name: &str) -> Option<&str> {
    let base = base_file_name(file_name);
    if let Some(extension) = SUPPORTED_DECLARATION_EXTENSIONS
        .iter()
        .find(|extension| base.ends_with(*extension))
    {
        return Some(&base[base.len() - extension.len()..]);
    }
    // TypeScript-Go compares extensions case-sensitively; matching it is the intent here.
    #[allow(clippy::case_sensitive_file_extension_comparisons)]
    if base.ends_with(".ts") {
        return base.find(".d.").map(|index| &base[index..]);
    }
    None
}

/// Returns whether `file_name` names a declaration file.
#[must_use]
pub fn is_declaration_file_name(file_name: &str) -> bool {
    declaration_file_extension(file_name).is_some()
}

/// Known extensions in removal order; declaration extensions precede their shorter suffixes.
const EXTENSIONS_TO_REMOVE: [&str; 12] = [
    ".d.ts", ".d.mts", ".d.cts", ".mjs", ".mts", ".cjs", ".cts", ".ts", ".js", ".tsx", ".jsx",
    ".json",
];

/// Returns `path` without a known TypeScript, JavaScript, or JSON extension.
#[must_use]
pub fn remove_file_extension(path: &str) -> &str {
    EXTENSIONS_TO_REMOVE
        .iter()
        .find_map(|extension| path.strip_suffix(extension))
        .unwrap_or(path)
}
