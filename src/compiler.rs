//! Compiler orchestration over parsing, checking, extensions, and emit.

use std::path::PathBuf;

use crate::analyzer::{AnalysisContext, Analyzer};
use crate::binder;
use crate::emit::declaration;
use crate::emit::javascript;
use crate::generator::{GenerationContext, Generator};
use crate::module_resolver;
use crate::source_file::{FileId, ScriptKind, SourceFile};
use crate::source_text::Utf16Offset;
use crate::syntax::{Diagnostic, SyntaxTree, TextSpan};
use crate::type_checker;

/// A JavaScript language target supported by the emitter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptTarget {
    /// ECMAScript 5.
    Es5,
    /// ECMAScript 2015.
    Es2015,
    /// ECMAScript 2016.
    Es2016,
    /// ECMAScript 2017.
    Es2017,
    /// ECMAScript 2018.
    Es2018,
    /// ECMAScript 2019.
    Es2019,
    /// ECMAScript 2020.
    Es2020,
    /// ECMAScript 2021.
    Es2021,
    /// ECMAScript 2022.
    Es2022,
    /// ECMAScript 2023.
    Es2023,
    /// ECMAScript 2024.
    Es2024,
    /// ECMAScript 2025.
    Es2025,
    /// The latest supported ECMAScript syntax.
    Latest,
}

/// A module format for emitted JavaScript.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModuleKind {
    /// `CommonJS` modules.
    CommonJs,
    /// ECMAScript modules with ES2015 syntax.
    Es2015,
    /// ECMAScript modules with ES2020 syntax.
    Es2020,
    /// ECMAScript modules with ES2022 syntax.
    Es2022,
    /// Latest ECMAScript module syntax.
    EsNext,
    /// Preserve the source module syntax.
    Preserve,
}

impl ModuleKind {
    /// Returns whether this format emits ECMAScript module syntax.
    #[must_use]
    pub const fn is_ecma_script(self) -> bool {
        matches!(
            self,
            Self::Es2015 | Self::Es2020 | Self::Es2022 | Self::EsNext | Self::Preserve
        )
    }
}

/// Options that control compilation behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompilerOptions {
    target: ScriptTarget,
    module: Option<ModuleKind>,
    declaration: bool,
    emit_declaration_only: bool,
    strict_null_checks: bool,
}

impl CompilerOptions {
    /// Creates compiler options for an ECMAScript target.
    #[must_use]
    pub const fn new(target: ScriptTarget) -> Self {
        Self {
            target,
            module: None,
            declaration: false,
            emit_declaration_only: false,
            strict_null_checks: true,
        }
    }

    /// Returns the configured ECMAScript target.
    #[must_use]
    pub const fn target(self) -> ScriptTarget {
        self.target
    }

    /// Sets an explicit module format.
    #[must_use]
    pub const fn with_module(mut self, module: ModuleKind) -> Self {
        self.module = Some(module);
        self
    }

    /// Returns the explicit module format, if present.
    #[must_use]
    pub const fn module(self) -> Option<ModuleKind> {
        self.module
    }

    /// Sets whether declaration files are emitted alongside JavaScript.
    #[must_use]
    pub const fn with_declaration(mut self, declaration: bool) -> Self {
        self.declaration = declaration;
        self
    }

    /// Returns whether declaration files are emitted alongside JavaScript.
    #[must_use]
    pub const fn declaration(self) -> bool {
        self.declaration
    }

    /// Sets whether compilation emits only declaration files.
    #[must_use]
    pub const fn with_emit_declaration_only(mut self, emit_declaration_only: bool) -> Self {
        self.emit_declaration_only = emit_declaration_only;
        self
    }

    /// Sets whether `null` and `undefined` are checked as distinct types.
    #[must_use]
    pub const fn with_strict_null_checks(mut self, strict_null_checks: bool) -> Self {
        self.strict_null_checks = strict_null_checks;
        self
    }

    /// Returns whether `null` and `undefined` are checked as distinct types.
    #[must_use]
    pub const fn strict_null_checks(self) -> bool {
        self.strict_null_checks
    }

    const fn emit_module_kind(self) -> ModuleKind {
        if let Some(module) = self.module {
            return module;
        }
        match self.target {
            ScriptTarget::Es5 => ModuleKind::CommonJs,
            ScriptTarget::Es2015
            | ScriptTarget::Es2016
            | ScriptTarget::Es2017
            | ScriptTarget::Es2018
            | ScriptTarget::Es2019 => ModuleKind::Es2015,
            ScriptTarget::Es2020 | ScriptTarget::Es2021 => ModuleKind::Es2020,
            ScriptTarget::Es2022
            | ScriptTarget::Es2023
            | ScriptTarget::Es2024
            | ScriptTarget::Es2025 => ModuleKind::Es2022,
            ScriptTarget::Latest => ModuleKind::EsNext,
        }
    }
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self::new(ScriptTarget::Es2025)
    }
}

/// A configured TypeScript compiler.
#[derive(Default)]
pub struct Compiler {
    options: CompilerOptions,
    analyzers: Vec<Box<dyn Analyzer>>,
    generators: Vec<Box<dyn Generator>>,
}

impl Compiler {
    /// Creates a compiler with TypeScript-compatible default target settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a compiler with explicit options.
    #[must_use]
    pub const fn with_options(options: CompilerOptions) -> Self {
        Self {
            options,
            analyzers: Vec::new(),
            generators: Vec::new(),
        }
    }

    /// Registers an analyzer that receives immutable compilation views.
    #[must_use]
    pub fn with_analyzer(mut self, analyzer: impl Analyzer + 'static) -> Self {
        self.analyzers.push(Box::new(analyzer));
        self
    }

    /// Registers a source generator that returns owned virtual files.
    #[must_use]
    pub fn with_generator(mut self, generator: impl Generator + 'static) -> Self {
        self.generators.push(Box::new(generator));
        self
    }

    /// Parses and emits one source file.
    #[must_use]
    pub fn compile(&self, source_file: SourceFile) -> CompilationResult {
        self.compile_sources([source_file])
    }

    /// Parses and emits source files in input order as one compilation.
    #[must_use]
    pub fn compile_sources(
        &self,
        source_files: impl IntoIterator<Item = SourceFile>,
    ) -> CompilationResult {
        let mut syntax_trees = source_files
            .into_iter()
            .zip(0..)
            .map(|(source_file, index)| SyntaxTree::parse_file(FileId::new(index), source_file))
            .collect::<Vec<_>>();
        let authored_symbols = binder::bind(&syntax_trees);
        let authored_diagnostics = collect_diagnostics(
            &syntax_trees,
            &authored_symbols,
            self.options.strict_null_checks(),
        );

        let generator_outputs = {
            let context = GenerationContext::new(&syntax_trees, &authored_diagnostics);
            self.generators
                .iter()
                .map(|generator| generator.generate(&context))
                .collect::<Vec<_>>()
        };
        let mut generated_sources = Vec::new();
        let mut generator_diagnostics = Vec::new();
        for output in generator_outputs {
            generated_sources.extend(output.sources().iter().cloned());
            generator_diagnostics.extend(output.diagnostics().iter().cloned());
        }

        let mut generated_trees = Vec::new();
        for generated_source in generated_sources {
            match SourceFile::from_path(generated_source.path(), generated_source.text().to_owned())
            {
                Ok(source_file) => {
                    let index = syntax_trees.len() + generated_trees.len();
                    let file_id = FileId::new(
                        u32::try_from(index).expect("compilation inputs fit in a file id"),
                    );
                    generated_trees.push(SyntaxTree::parse_file(file_id, source_file));
                }
                Err(error) => generator_diagnostics.push(Diagnostic::new(
                    9002,
                    format!(
                        "generated source has an unsupported path: {}",
                        error.path().display()
                    ),
                    TextSpan::new(Utf16Offset::new(0), 0),
                )),
            }
        }
        syntax_trees.extend(generated_trees);
        let symbols = binder::bind(&syntax_trees);
        let mut diagnostics =
            collect_diagnostics(&syntax_trees, &symbols, self.options.strict_null_checks());
        diagnostics.extend(module_resolver::check(&syntax_trees));
        diagnostics.extend(generator_diagnostics);

        let analyzer_diagnostics = {
            let context = AnalysisContext::new(&syntax_trees, &diagnostics);
            self.analyzers
                .iter()
                .flat_map(|analyzer| analyzer.analyze(&context))
                .collect::<Vec<_>>()
        };
        diagnostics.extend(analyzer_diagnostics);
        diagnostics.sort_by_key(|diagnostic| (diagnostic.file(), diagnostic.span().start()));
        let emitted_files = syntax_trees
            .iter()
            .flat_map(|syntax_tree| {
                emit_source(
                    syntax_tree,
                    self.options.target,
                    self.options.emit_module_kind(),
                    self.options.declaration,
                    self.options.emit_declaration_only,
                )
            })
            .collect();
        CompilationResult {
            syntax_trees,
            diagnostics,
            emitted_files,
        }
    }
}

fn collect_diagnostics(
    syntax_trees: &[SyntaxTree],
    symbols: &binder::SymbolTable,
    strict_null_checks: bool,
) -> Vec<Diagnostic> {
    syntax_trees
        .iter()
        .flat_map(|syntax_tree| {
            let scoped_symbols = symbols.for_source(syntax_tree.source_file().path());
            syntax_tree
                .diagnostics()
                .iter()
                .cloned()
                .chain(type_checker::check(
                    syntax_tree.program(),
                    &scoped_symbols,
                    strict_null_checks,
                ))
                .map(|diagnostic| diagnostic.in_file(syntax_tree.file_id()))
        })
        .collect()
}

/// The result of compiling source files.
#[derive(Debug, Clone)]
pub struct CompilationResult {
    syntax_trees: Vec<SyntaxTree>,
    diagnostics: Vec<Diagnostic>,
    emitted_files: Vec<EmittedFile>,
}

impl CompilationResult {
    /// Returns the parsed source trees retained by this compilation.
    #[must_use]
    pub fn syntax_trees(&self) -> &[SyntaxTree] {
        &self.syntax_trees
    }

    /// Returns parser and compiler diagnostics in source order.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Returns emitted files in deterministic source order.
    #[must_use]
    pub fn emitted_files(&self) -> &[EmittedFile] {
        &self.emitted_files
    }
}

/// A compiler output file with owned path and contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmittedFile {
    path: PathBuf,
    text: String,
}

impl EmittedFile {
    /// Returns the output file path.
    #[must_use]
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Returns the emitted file contents.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

fn emit_source(
    syntax_tree: &SyntaxTree,
    target: ScriptTarget,
    module: ModuleKind,
    emit_declaration: bool,
    emit_declaration_only: bool,
) -> Vec<EmittedFile> {
    let source_file = syntax_tree.source_file();
    if source_file.script_kind() == ScriptKind::TypeScriptDeclaration
        || matches!(
            source_file.script_kind(),
            ScriptKind::JavaScript | ScriptKind::JavaScriptJsx
        )
    {
        return Vec::new();
    }

    let extension = match source_file
        .path()
        .extension()
        .and_then(|value| value.to_str())
    {
        Some("mts") => "mjs",
        Some("cts") => "cjs",
        _ => "js",
    };
    let mut path = source_file.path().to_path_buf();
    path.set_extension(extension);

    let mut emitted_files = Vec::new();
    if !emit_declaration_only {
        emitted_files.push(EmittedFile {
            path,
            text: javascript::emit(syntax_tree.program(), target, module),
        });
    }
    if emit_declaration {
        let mut declaration_path = source_file.path().to_path_buf();
        let declaration_extension = match source_file
            .path()
            .extension()
            .and_then(|value| value.to_str())
        {
            Some("mts") => "d.mts",
            Some("cts") => "d.cts",
            _ => "d.ts",
        };
        declaration_path.set_extension(declaration_extension);
        emitted_files.push(EmittedFile {
            path: declaration_path,
            text: declaration::emit(syntax_tree.program()),
        });
    }
    emitted_files
}
