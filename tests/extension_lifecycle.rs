use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use tsrzl::analyzer::{AnalysisContext, Analyzer};
use tsrzl::compiler::Compiler;
use tsrzl::generator::{GeneratedSource, GenerationContext, GenerationOutput, Generator};
use tsrzl::source_file::SourceFile;
use tsrzl::source_text::Utf16Offset;
use tsrzl::syntax::{Diagnostic, TextSpan};

fn source_file(path: &str, text: &str) -> SourceFile {
    SourceFile::from_path(Path::new(path), text)
        .expect("a TypeScript path has a supported source kind")
}

fn diagnostic(code: u32, message: &'static str) -> Diagnostic {
    Diagnostic::new(code, message, TextSpan::new(Utf16Offset::new(0), 0))
}

fn output_with_sources(sources: &[GeneratedSource]) -> GenerationOutput {
    sources
        .iter()
        .cloned()
        .fold(GenerationOutput::new(), GenerationOutput::with_source)
}

#[derive(Debug, PartialEq, Eq)]
struct GenerationObservation {
    paths: Vec<PathBuf>,
    diagnostic_codes: Vec<u32>,
}

struct SnapshotGenerator {
    observations: Arc<Mutex<Vec<GenerationObservation>>>,
    generated_source: Option<GeneratedSource>,
}

impl Generator for SnapshotGenerator {
    fn generate(&self, context: &GenerationContext<'_>) -> GenerationOutput {
        self.observations
            .lock()
            .expect("the observation list is not poisoned")
            .push(GenerationObservation {
                paths: context
                    .syntax_trees()
                    .iter()
                    .map(|tree| tree.source_file().path().to_path_buf())
                    .collect(),
                diagnostic_codes: context.diagnostics().iter().map(Diagnostic::code).collect(),
            });
        self.generated_source
            .as_ref()
            .map_or_else(GenerationOutput::new, |source| {
                GenerationOutput::new().with_source(source.clone())
            })
    }
}

struct EventGenerator {
    event: &'static str,
    events: Arc<Mutex<Vec<&'static str>>>,
}

impl Generator for EventGenerator {
    fn generate(&self, _context: &GenerationContext<'_>) -> GenerationOutput {
        self.events
            .lock()
            .expect("the event list is not poisoned")
            .push(self.event);
        GenerationOutput::new()
    }
}

struct EventAnalyzer {
    events: Arc<Mutex<Vec<&'static str>>>,
}

impl Analyzer for EventAnalyzer {
    fn analyze(&self, _context: &AnalysisContext<'_>) -> Vec<Diagnostic> {
        self.events
            .lock()
            .expect("the event list is not poisoned")
            .push("analyzer");
        Vec::new()
    }
}

struct SourceListGenerator(Vec<GeneratedSource>);

impl Generator for SourceListGenerator {
    fn generate(&self, _context: &GenerationContext<'_>) -> GenerationOutput {
        output_with_sources(&self.0)
    }
}

struct DiagnosticGenerator(Diagnostic);

impl Generator for DiagnosticGenerator {
    fn generate(&self, _context: &GenerationContext<'_>) -> GenerationOutput {
        GenerationOutput::new().with_diagnostic(self.0.clone())
    }
}

struct DiagnosticAnalyzer(Diagnostic);

impl Analyzer for DiagnosticAnalyzer {
    fn analyze(&self, _context: &AnalysisContext<'_>) -> Vec<Diagnostic> {
        vec![self.0.clone()]
    }
}

struct OrderedDiagnosticAnalyzer {
    code: u32,
}

impl Analyzer for OrderedDiagnosticAnalyzer {
    fn analyze(&self, _context: &AnalysisContext<'_>) -> Vec<Diagnostic> {
        vec![diagnostic(self.code, "ordered analyzer diagnostic")]
    }
}

// Project-owned extension contract; the TypeScript corpus has no generator snapshot equivalent.
#[test]
fn should_share_authored_snapshot_given_multiple_generators_when_compiling() {
    // Arrange
    let authored = source_file("authored.ts", "const answer: number = \"wrong\";");
    let observations = Arc::new(Mutex::new(Vec::new()));
    let first = SnapshotGenerator {
        observations: Arc::clone(&observations),
        generated_source: Some(GeneratedSource::new(
            "first.ts",
            "const firstGenerated = 1;",
        )),
    };
    let second = SnapshotGenerator {
        observations: Arc::clone(&observations),
        generated_source: None,
    };
    let compiler = Compiler::new().with_generator(first).with_generator(second);

    // Act
    let _result = compiler.compile(authored);

    // Assert
    let expected = GenerationObservation {
        paths: vec![PathBuf::from("authored.ts")],
        diagnostic_codes: vec![2322],
    };
    assert_eq!(
        *observations
            .lock()
            .expect("the observation list is not poisoned"),
        vec![
            expected,
            GenerationObservation {
                paths: vec![PathBuf::from("authored.ts")],
                diagnostic_codes: vec![2322],
            }
        ]
    );
}

// Project-owned extension contract; the TypeScript corpus has no stage-order equivalent.
#[test]
fn should_run_generators_before_analyzers_given_registered_extensions_when_compiling() {
    // Arrange
    let events = Arc::new(Mutex::new(Vec::new()));
    let compiler = Compiler::new()
        .with_generator(EventGenerator {
            event: "generator one",
            events: Arc::clone(&events),
        })
        .with_generator(EventGenerator {
            event: "generator two",
            events: Arc::clone(&events),
        })
        .with_analyzer(EventAnalyzer {
            events: Arc::clone(&events),
        });

    // Act
    let _result = compiler.compile(source_file("input.ts", "const input = 1;"));

    // Assert
    assert_eq!(
        *events.lock().expect("the event list is not poisoned"),
        ["generator one", "generator two", "analyzer"]
    );
}

// Project-owned extension contract; the TypeScript corpus has no generated-output ordering case.
#[test]
fn should_preserve_generator_order_given_multiple_outputs_when_emitting() {
    // Arrange
    let first = SourceListGenerator(vec![
        GeneratedSource::new("first-a.ts", "const firstA = 1;"),
        GeneratedSource::new("first-b.ts", "const firstB = 2;"),
    ]);
    let second = SourceListGenerator(vec![GeneratedSource::new("second.ts", "const second = 3;")]);
    let compiler = Compiler::new().with_generator(first).with_generator(second);

    // Act
    let result = compiler.compile(source_file("input.ts", "const input = 0;"));

    // Assert
    let paths = result
        .emitted_files()
        .iter()
        .map(|file| file.path().to_path_buf())
        .collect::<Vec<_>>();
    assert_eq!(
        paths,
        [
            PathBuf::from("input.js"),
            PathBuf::from("first-a.js"),
            PathBuf::from("first-b.js"),
            PathBuf::from("second.js"),
        ]
    );
}

// Project-owned extension contract; the TypeScript corpus has no analyzer ordering case.
#[test]
fn should_preserve_analyzer_registration_order_given_equal_span_diagnostics_when_compiling() {
    // Arrange
    let compiler = Compiler::new()
        .with_analyzer(OrderedDiagnosticAnalyzer { code: 9201 })
        .with_analyzer(OrderedDiagnosticAnalyzer { code: 9202 });

    // Act
    let result = compiler.compile(source_file("input.ts", "const input = 0;"));

    // Assert
    let analyzer_codes = result
        .diagnostics()
        .iter()
        .map(Diagnostic::code)
        .collect::<Vec<_>>();
    assert_eq!(analyzer_codes, [9201, 9202]);
}

// Project-owned extension contract; the TypeScript corpus has no generator diagnostic equivalent.
#[test]
fn should_propagate_generator_diagnostic_given_generator_output_when_compiling() {
    // Arrange
    let expected = diagnostic(9301, "generator reported a problem");
    let compiler = Compiler::new().with_generator(DiagnosticGenerator(expected.clone()));

    // Act
    let result = compiler.compile(source_file("input.ts", "const input = 0;"));

    // Assert
    assert!(result.diagnostics().contains(&expected));
}

// Project-owned extension contract; the TypeScript corpus has no analyzer diagnostic equivalent.
#[test]
fn should_propagate_analyzer_diagnostic_given_registered_analyzer_when_compiling() {
    // Arrange
    let expected = diagnostic(9302, "analyzer reported a problem");
    let compiler = Compiler::new().with_analyzer(DiagnosticAnalyzer(expected.clone()));

    // Act
    let result = compiler.compile(source_file("input.ts", "const input = 0;"));

    // Assert
    assert!(result.diagnostics().contains(&expected));
}

// Project-owned extension contract; generated files must pass through the normal checker.
#[test]
fn should_check_generated_source_given_incompatible_initializer_when_compiling() {
    // Arrange
    let generated =
        GeneratedSource::new("generated.ts", "const generatedValue: number = \"wrong\";");
    let compiler = Compiler::new().with_generator(SourceListGenerator(vec![generated]));

    // Act
    let result = compiler.compile(source_file("input.ts", "const input = 0;"));

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2322)
    );
}

// Project-owned extension contract; each compile call owns a fresh generated-source set.
#[test]
fn should_isolate_generated_sources_given_repeated_compilations_when_compiling() {
    // Arrange
    let compiler = Compiler::new().with_generator(SourceListGenerator(vec![GeneratedSource::new(
        "generated.ts",
        "const generated = 1;",
    )]));

    // Act
    let first = compiler.compile(source_file("input.ts", "const input = 0;"));
    let second = compiler.compile(source_file("input.ts", "const input = 0;"));

    // Assert
    assert_eq!(first.syntax_trees().len(), 2);
    assert_eq!(second.syntax_trees().len(), 2);
    assert_eq!(first.emitted_files().len(), 2);
    assert_eq!(second.emitted_files().len(), 2);
}

// Project-owned extension contract; authored and generated paths must not collide silently.
#[test]
fn should_report_authored_generated_path_collision_given_generator_output_when_compiling() {
    // Arrange
    let generated = GeneratedSource::new("input.ts", "const generated = 1;");
    let compiler = Compiler::new().with_generator(SourceListGenerator(vec![generated]));

    // Act
    let result = compiler.compile(source_file("input.ts", "const authored = 0;"));

    // Assert
    assert_ne!(result.diagnostics(), []);
}

// Project-owned extension contract; repeated generated paths must not collide silently.
#[test]
fn should_report_duplicate_generated_path_given_multiple_sources_when_compiling() {
    // Arrange
    let generated = vec![
        GeneratedSource::new("shared.ts", "const first = 1;"),
        GeneratedSource::new("shared.ts", "const second = 2;"),
    ];
    let compiler = Compiler::new().with_generator(SourceListGenerator(generated));

    // Act
    let result = compiler.compile(source_file("input.ts", "const input = 0;"));

    // Assert
    assert_ne!(result.diagnostics(), []);
}

// Project-owned extension contract; invalid generated paths must be rejected and reported.
#[test]
fn should_report_invalid_generated_path_given_unsupported_extension_when_compiling() {
    // Arrange
    let invalid = GeneratedSource::new("generated.bin", "const generated = 1;");
    let compiler = Compiler::new().with_generator(SourceListGenerator(vec![invalid]));

    // Act
    let result = compiler.compile(source_file("input.ts", "const input = 0;"));

    // Assert
    assert_ne!(result.diagnostics(), []);
}

// Project-owned extension contract; generated files must remain inside the compilation root.
#[test]
fn should_reject_parent_directory_generated_path_given_registered_generator_when_compiling() {
    // Arrange
    let generated = GeneratedSource::new("../outside.ts", "const generated = 1;");
    let compiler = Compiler::new().with_generator(SourceListGenerator(vec![generated]));

    // Act
    let result = compiler.compile(source_file("input.ts", "const input = 0;"));

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.message().contains("generated source")),
        "an out-of-root generated file should produce a diagnostic"
    );
    assert!(
        result
            .emitted_files()
            .iter()
            .all(|file| file.path() != Path::new("../outside.js")),
        "an out-of-root generated file should not be emitted"
    );
}
