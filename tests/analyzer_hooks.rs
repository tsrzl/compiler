use std::path::Path;

use tsrzl::analyzer::{AnalysisContext, Analyzer};
use tsrzl::compiler::Compiler;
use tsrzl::generator::{GeneratedSource, GenerationContext, GenerationOutput, Generator};
use tsrzl::source_file::SourceFile;
use tsrzl::source_text::Utf16Offset;
use tsrzl::syntax::{Diagnostic, TextSpan};

struct DeclarationAnalyzer;

impl Analyzer for DeclarationAnalyzer {
    fn analyze(&self, context: &AnalysisContext<'_>) -> Vec<Diagnostic> {
        let contains_declaration = context
            .syntax_trees()
            .iter()
            .any(|tree| !tree.program().statements().is_empty());
        if contains_declaration {
            vec![Diagnostic::new(
                9001,
                "declaration observed by analyzer",
                TextSpan::new(Utf16Offset::new(0), 0),
            )]
        } else {
            Vec::new()
        }
    }
}

struct GeneratedSourceGenerator;

impl Generator for GeneratedSourceGenerator {
    fn generate(&self, _context: &GenerationContext<'_>) -> GenerationOutput {
        GenerationOutput::new().with_source(GeneratedSource::new(
            Path::new("generated.ts"),
            "const generatedAnswer: number = 42;",
        ))
    }
}

struct GeneratedSourceAnalyzer;

impl Analyzer for GeneratedSourceAnalyzer {
    fn analyze(&self, context: &AnalysisContext<'_>) -> Vec<Diagnostic> {
        let generated_source_exists = context
            .syntax_trees()
            .iter()
            .any(|tree| tree.source_file().path() == Path::new("generated.ts"));
        if generated_source_exists {
            vec![Diagnostic::new(
                9003,
                "generated source observed by analyzer",
                TextSpan::new(Utf16Offset::new(0), 0),
            )]
        } else {
            Vec::new()
        }
    }
}

#[test]
fn should_include_analyzer_diagnostic_given_registered_analyzer_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(Path::new("answer.ts"), "const answer: number = 42;")
        .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::new().with_analyzer(DeclarationAnalyzer);

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 9001)
    );
}

#[test]
fn should_analyze_generated_source_given_registered_generator_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(Path::new("input.ts"), "const input: number = 1;")
        .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::new()
        .with_generator(GeneratedSourceGenerator)
        .with_analyzer(GeneratedSourceAnalyzer);

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 9003)
    );
}
