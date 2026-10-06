//! File-level comment directives, modeled on TypeScript-Go's `extractPragmas` and
//! `processPragmasIntoFields`.
//!
//! Only comments before the first token are directives: triple-slash `<reference />` tags and
//! `@ts-check` or `@ts-nocheck` comments. JSX factory pragmas are extracted with JSX emit.

use super::ParseDiagnostic;
use crate::ast::SyntaxKind;
use crate::diagnostics;
use crate::scanner::{CommentRange, leading_comment_ranges};

/// The module format a `resolution-mode` reference attribute selects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResolutionMode {
    /// `resolution-mode="require"`.
    CommonJs,
    /// `resolution-mode="import"`.
    EsNext,
}

/// A file named by a `<reference />` directive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileReference {
    /// The UTF-8 byte offset where the quoted value starts.
    pub pos: usize,
    /// The UTF-8 byte offset where the quoted value ends.
    pub end: usize,
    /// The referenced path, package, or lib name.
    pub file_name: String,
    /// The module format requested by `resolution-mode`.
    pub resolution_mode: Option<ResolutionMode>,
    /// Whether `preserve="true"` keeps the directive in emitted output.
    pub preserve: bool,
}

/// The winning `@ts-check` or `@ts-nocheck` comment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckJsDirective {
    /// Whether the comment enables checking.
    pub enabled: bool,
    /// The comment's UTF-8 byte range start.
    pub pos: usize,
    /// The comment's UTF-8 byte range end.
    pub end: usize,
}

/// The directives of one file and the errors found reading them.
#[derive(Debug, Clone, Default)]
pub(super) struct FileDirectives {
    pub(super) referenced_files: Vec<FileReference>,
    pub(super) type_reference_directives: Vec<FileReference>,
    pub(super) lib_reference_directives: Vec<FileReference>,
    pub(super) check_js_directive: Option<CheckJsDirective>,
    pub(super) diagnostics: Vec<ParseDiagnostic>,
}

/// A `<reference />` attribute value and its location.
struct Argument<'text> {
    value: &'text str,
    pos: usize,
    end: usize,
}

/// Reads the directives in the comments that lead the file.
pub(super) fn file_directives(text: &str) -> FileDirectives {
    let mut directives = FileDirectives::default();
    for comment in leading_comment_ranges(text, 0) {
        if comment.kind != SyntaxKind::SingleLineCommentTrivia {
            continue;
        }
        let comment_text = &text[comment.pos..comment.end];
        if let Some(arguments) = reference_arguments(comment, comment_text) {
            add_reference(&mut directives, comment, &arguments);
        } else if let Some(enabled) = check_directive(comment_text) {
            let replaces = directives
                .check_js_directive
                .is_none_or(|existing| comment.pos > existing.pos);
            if replaces {
                directives.check_js_directive = Some(CheckJsDirective {
                    enabled,
                    pos: comment.pos,
                    end: comment.end,
                });
            }
        }
    }
    directives
}

fn add_reference(
    directives: &mut FileDirectives,
    comment: CommentRange,
    arguments: &[(String, Argument<'_>)],
) {
    let argument = |name: &str| {
        arguments
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value)
    };
    let preserve = argument("preserve").is_some_and(|value| value.value == "true");
    let reference = |argument: &Argument<'_>, resolution_mode| FileReference {
        pos: argument.pos,
        end: argument.end,
        file_name: argument.value.to_owned(),
        resolution_mode,
        preserve,
    };
    if argument("no-default-lib").is_some_and(|value| value.value == "true") {
        // Ignored, as in TypeScript-Go.
    } else if let Some(types) = argument("types") {
        let resolution_mode = argument("resolution-mode").and_then(|mode| match mode.value {
            "import" => Some(ResolutionMode::EsNext),
            "require" => Some(ResolutionMode::CommonJs),
            _ => {
                directives.diagnostics.push(ParseDiagnostic::new(
                    diagnostics::X_RESOLUTION_MODE_SHOULD_BE_EITHER_REQUIRE_OR_IMPORT,
                    mode.pos,
                    mode.end,
                    &[],
                ));
                None
            }
        });
        directives
            .type_reference_directives
            .push(reference(types, resolution_mode));
    } else if let Some(lib) = argument("lib") {
        directives
            .lib_reference_directives
            .push(reference(lib, None));
    } else if let Some(path) = argument("path") {
        directives.referenced_files.push(reference(path, None));
    } else {
        directives.diagnostics.push(ParseDiagnostic::new(
            diagnostics::INVALID_REFERENCE_DIRECTIVE_SYNTAX,
            comment.pos,
            comment.end,
            &[],
        ));
    }
}

/// Reads the attributes of a `/// <reference ... />` comment, or `None` for other comments.
fn reference_arguments(comment: CommentRange, text: &str) -> Option<Vec<(String, Argument<'_>)>> {
    let mut pos = 2;
    if !text[pos..].starts_with('/') {
        return None;
    }
    pos = skip_blanks(text, pos + 1);
    if !text[pos..].starts_with('<') || extract_name(text, pos + 1) != "reference" {
        return None;
    }
    pos += "<reference".len();
    let mut arguments = Vec::new();
    loop {
        pos = skip_blanks(text, pos);
        if text[pos..].starts_with("/>") {
            break;
        }
        let name = extract_name(text, pos);
        if name.is_empty() {
            break;
        }
        pos = skip_blanks(text, pos + name.len());
        if !text[pos..].starts_with('=') {
            break;
        }
        pos = skip_blanks(text, pos + 1);
        let Some(value) = extract_quoted_string(text, pos) else {
            break;
        };
        let value_pos = comment.pos + pos + 1;
        arguments.push((
            name,
            Argument {
                value,
                pos: value_pos,
                end: value_pos + value.len(),
            },
        ));
        pos += value.len() + 2;
    }
    Some(arguments)
}

/// Returns whether a `// @ts-check` or `// @ts-nocheck` comment enables checking.
fn check_directive(text: &str) -> Option<bool> {
    let pos = skip_blanks(text, 2);
    if !text[pos..].starts_with('@') {
        return None;
    }
    match extract_name(text, pos + 1).as_str() {
        "ts-check" => Some(true),
        "ts-nocheck" => Some(false),
        _ => None,
    }
}

fn skip_blanks(text: &str, mut pos: usize) -> usize {
    let bytes = text.as_bytes();
    while pos < bytes.len() && matches!(bytes[pos], b' ' | b'\t') {
        pos += 1;
    }
    pos
}

/// Returns the lowercased run of ASCII letters and hyphens at `pos`.
fn extract_name(text: &str, pos: usize) -> String {
    let rest = &text.as_bytes()[pos.min(text.len())..];
    let length = rest
        .iter()
        .take_while(|&&byte| byte.is_ascii_alphabetic() || byte == b'-')
        .count();
    text[pos..pos + length].to_ascii_lowercase()
}

/// Returns the contents of a single- or double-quoted string starting at `pos`.
fn extract_quoted_string(text: &str, pos: usize) -> Option<&str> {
    let quote = *text.as_bytes().get(pos)?;
    if quote != b'\'' && quote != b'"' {
        return None;
    }
    let start = pos + 1;
    let length = text.as_bytes()[start..]
        .iter()
        .position(|&byte| byte == quote)?;
    Some(&text[start..start + length])
}
