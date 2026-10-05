//! Source files and their language-specific script kinds.

use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::source_text::SourceText;

/// The syntax mode selected for a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptKind {
    /// A TypeScript source file.
    TypeScript,
    /// A TypeScript source file containing JSX syntax.
    TypeScriptJsx,
    /// A TypeScript declaration source file.
    TypeScriptDeclaration,
    /// A JavaScript source file.
    JavaScript,
    /// A JavaScript source file containing JSX syntax.
    JavaScriptJsx,
}

/// An immutable compiler input with an owned path and source text.
#[derive(Debug, Clone)]
pub struct SourceFile {
    path: PathBuf,
    text: SourceText,
    script_kind: ScriptKind,
}

impl SourceFile {
    /// Creates a source file by inferring its script kind from its path.
    ///
    /// # Errors
    ///
    /// Returns an error when the path has an unsupported source file extension.
    pub fn from_path(
        path: impl AsRef<Path>,
        text: impl Into<std::sync::Arc<str>>,
    ) -> Result<Self, UnsupportedSourceExtension> {
        let path = path.as_ref();
        let script_kind = Self::script_kind_for_path(path)?;

        Ok(Self {
            path: path.to_path_buf(),
            text: SourceText::new(text),
            script_kind,
        })
    }

    /// Returns the source file path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the immutable source text.
    #[must_use]
    pub const fn text(&self) -> &SourceText {
        &self.text
    }

    /// Returns the script kind inferred from the source file path.
    #[must_use]
    pub const fn script_kind(&self) -> ScriptKind {
        self.script_kind
    }

    fn script_kind_for_path(path: &Path) -> Result<ScriptKind, UnsupportedSourceExtension> {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        let extension = path.extension().and_then(|extension| extension.to_str());

        if name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts") {
            return Ok(ScriptKind::TypeScriptDeclaration);
        }

        match extension {
            Some("ts" | "mts" | "cts") => Ok(ScriptKind::TypeScript),
            Some("tsx") => Ok(ScriptKind::TypeScriptJsx),
            Some("js" | "mjs" | "cjs") => Ok(ScriptKind::JavaScript),
            Some("jsx") => Ok(ScriptKind::JavaScriptJsx),
            _ => Err(UnsupportedSourceExtension {
                path: path.to_path_buf(),
            }),
        }
    }
}

/// An error returned when a path does not identify a supported source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedSourceExtension {
    path: PathBuf,
}

impl UnsupportedSourceExtension {
    /// Returns the path whose extension was not supported.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl fmt::Display for UnsupportedSourceExtension {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "unsupported source file extension: {}",
            self.path.display()
        )
    }
}

impl Error for UnsupportedSourceExtension {}
