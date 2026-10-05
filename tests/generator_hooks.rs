use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::generator::{GeneratedSource, GenerationContext, GenerationOutput, Generator};
use tsrzl::source_file::SourceFile;

struct AnswerGenerator;

impl Generator for AnswerGenerator {
    fn generate(&self, _context: &GenerationContext<'_>) -> GenerationOutput {
        GenerationOutput::new().with_source(GeneratedSource::new(
            Path::new("generated.ts"),
            "const generatedAnswer: number = 42;",
        ))
    }
}

struct TypedAnswerGenerator;

impl Generator for TypedAnswerGenerator {
    fn generate(&self, _context: &GenerationContext<'_>) -> GenerationOutput {
        GenerationOutput::new().with_source(GeneratedSource::new(
            Path::new("generated.ts"),
            "export const generatedAnswer: number = 42;",
        ))
    }
}

#[test]
fn should_compile_owned_generated_source_given_generator_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(Path::new("input.ts"), "const input: number = 1;")
        .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::new().with_generator(AnswerGenerator);

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result
            .emitted_files()
            .iter()
            .any(|file| file.text() == "const generatedAnswer = 42;\n")
    );
}

#[test]
fn should_check_authored_import_against_generated_export_given_generator_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("main.ts"),
        "import { generatedAnswer } from './generated'; const label: string = generatedAnswer;",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::new().with_generator(TypedAnswerGenerator);

    // Act
    let result = compiler.compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'number' is not assignable to type 'string'."
    );
}
