//! Program construction modeled on TypeScript-Go's `compiler` file loader.
//!
//! A program is its root files plus the bundled lib files they need: the target's default lib,
//! or the `lib` option's libs, and every lib named by a `/// <reference lib="..." />`
//! directive. Files referenced by path or type directive, and imports, need module resolution
//! and are loaded once it is ported.

mod file;

use std::collections::HashSet;

pub use file::ProgramFile;

use crate::bundled::{
    default_lib_file_name, lib_file_name, lib_option_names, lib_priority, lib_text,
};
use crate::diagnostics::{self, Diagnostic};
use crate::parser::{ParseOptions, ParsedSourceFile, ScriptKind, parse_source_file};
use crate::spelling::spelling_suggestion;

/// The path prefix of bundled lib files, as TypeScript-Go names them.
pub const BUNDLED_LIB_PREFIX: &str = "bundled:///libs/";

/// Compiler options that decide which files a program loads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramOptions {
    /// The `target` option value, such as `es2022`, which selects the default lib.
    pub target: String,
    /// The `lib` option values, replacing the target's default lib.
    pub lib: Option<Vec<String>>,
    /// Whether no lib files are loaded.
    pub no_lib: bool,
}

impl Default for ProgramOptions {
    fn default() -> Self {
        Self {
            target: "esnext".to_owned(),
            lib: None,
            no_lib: false,
        }
    }
}

/// A diagnostic about a program file, reported while loading the program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramDiagnostic {
    file: usize,
    diagnostic: Diagnostic,
}

impl ProgramDiagnostic {
    /// Returns the index of the file the diagnostic reports on.
    #[must_use]
    pub const fn file(&self) -> usize {
        self.file
    }

    /// Returns the positioned diagnostic.
    #[must_use]
    pub const fn diagnostic(&self) -> &Diagnostic {
        &self.diagnostic
    }
}

/// The files of a compilation: lib files first, then the other files in discovery order.
#[derive(Debug, Clone)]
pub struct Program {
    files: Vec<ProgramFile>,
    diagnostics: Vec<ProgramDiagnostic>,
}

/// A file awaiting inclusion, with the libs it references.
struct Task {
    parsed: ParsedSourceFile,
    is_lib: bool,
    references: Vec<usize>,
    unknown_libs: Vec<(usize, usize, String)>,
}

impl Program {
    /// Loads `roots` and the lib files they need.
    #[must_use]
    pub fn load(roots: Vec<ParsedSourceFile>, options: &ProgramOptions) -> Self {
        let mut loader = Loader::default();
        let has_roots = !roots.is_empty();
        let mut root_tasks: Vec<usize> = roots
            .into_iter()
            .map(|parsed| loader.add_task(parsed, false, options))
            .collect();
        if has_roots && !options.no_lib {
            let libs = match &options.lib {
                None => vec![default_lib_file_name(&options.target)],
                Some(names) => names
                    .iter()
                    .filter_map(|name| lib_file_name(name))
                    .collect(),
            };
            root_tasks.extend(libs.into_iter().map(|name| loader.lib_task(name, options)));
        }
        loader.finish(&root_tasks)
    }

    /// Returns the files in checking order.
    #[must_use]
    pub fn files(&self) -> &[ProgramFile] {
        &self.files
    }

    /// Returns the diagnostics found while loading.
    #[must_use]
    pub fn diagnostics(&self) -> &[ProgramDiagnostic] {
        &self.diagnostics
    }
}

#[derive(Default)]
struct Loader {
    tasks: Vec<Task>,
    lib_tasks: Vec<(&'static str, usize)>,
}

impl Loader {
    fn add_task(
        &mut self,
        parsed: ParsedSourceFile,
        is_lib: bool,
        options: &ProgramOptions,
    ) -> usize {
        let index = self.tasks.len();
        let directives = parsed.lib_reference_directives().to_vec();
        self.tasks.push(Task {
            parsed,
            is_lib,
            references: Vec::new(),
            unknown_libs: Vec::new(),
        });
        if !options.no_lib {
            for reference in directives {
                match lib_file_name(&reference.file_name) {
                    Some(name) => {
                        let task = self.lib_task(name, options);
                        self.tasks[index].references.push(task);
                    }
                    None => self.tasks[index].unknown_libs.push((
                        reference.pos,
                        reference.end,
                        reference.file_name,
                    )),
                }
            }
        }
        index
    }

    /// Returns the task for a bundled lib file, parsing it the first time it is named.
    fn lib_task(&mut self, name: &'static str, options: &ProgramOptions) -> usize {
        if let Some(&(_, task)) = self.lib_tasks.iter().find(|(lib, _)| *lib == name) {
            return task;
        }
        let text = lib_text(name).expect("lib names come from the bundled lib table");
        let parsed = parse_source_file(
            &ParseOptions::new(format!("{BUNDLED_LIB_PREFIX}{name}"), ScriptKind::Ts),
            text,
        );
        // Record the task before following its references so reference cycles terminate.
        let index = self.tasks.len();
        self.lib_tasks.push((name, index));
        let added = self.add_task(parsed, true, options);
        debug_assert_eq!(added, index);
        index
    }

    /// Orders files as TypeScript-Go does: depth-first with references before the file that
    /// names them, then libs sorted by priority ahead of all other files.
    fn finish(self, root_tasks: &[usize]) -> Program {
        let mut order = Vec::new();
        let mut seen = HashSet::new();
        for &task in root_tasks {
            collect(&self.tasks, task, &mut seen, &mut order);
        }
        let (mut libs, others): (Vec<usize>, Vec<usize>) =
            order.into_iter().partition(|&task| self.tasks[task].is_lib);
        libs.sort_by_key(|&task| {
            let file_name = self.tasks[task].parsed.options().file_name();
            lib_priority(
                file_name
                    .strip_prefix(BUNDLED_LIB_PREFIX)
                    .unwrap_or(file_name),
            )
        });
        let ordered: Vec<usize> = libs.into_iter().chain(others).collect();
        let mut diagnostics = Vec::new();
        for (file, &task) in ordered.iter().enumerate() {
            for (pos, end, name) in &self.tasks[task].unknown_libs {
                diagnostics.push(ProgramDiagnostic {
                    file,
                    diagnostic: unknown_lib_diagnostic(*pos, *end, name),
                });
            }
        }
        let mut tasks: Vec<Option<Task>> = self.tasks.into_iter().map(Some).collect();
        let files = ordered
            .into_iter()
            .map(|task| {
                ProgramFile::new(
                    tasks[task]
                        .take()
                        .expect("each task is ordered once")
                        .parsed,
                )
            })
            .collect();
        Program { files, diagnostics }
    }
}

fn collect(tasks: &[Task], task: usize, seen: &mut HashSet<usize>, order: &mut Vec<usize>) {
    if !seen.insert(task) {
        return;
    }
    for &reference in &tasks[task].references {
        collect(tasks, reference, seen, order);
    }
    order.push(task);
}

/// Reports an unknown lib, suggesting the closest lib option name.
fn unknown_lib_diagnostic(pos: usize, end: usize, name: &str) -> Diagnostic {
    let lib_name = name.to_ascii_lowercase();
    let unqualified = lib_name
        .strip_prefix("lib.")
        .unwrap_or(&lib_name)
        .strip_suffix(".d.ts")
        .unwrap_or_else(|| lib_name.strip_prefix("lib.").unwrap_or(&lib_name));
    match spelling_suggestion(unqualified, lib_option_names()) {
        Some(suggestion) => Diagnostic::new(
            diagnostics::CANNOT_FIND_LIB_DEFINITION_FOR_0_DID_YOU_MEAN_1,
            pos..end,
            &[&lib_name, suggestion],
        ),
        None => Diagnostic::new(
            diagnostics::CANNOT_FIND_LIB_DEFINITION_FOR_0,
            pos..end,
            &[&lib_name],
        ),
    }
}
