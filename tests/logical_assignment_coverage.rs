use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_preserve_conjunction_assignment_given_es2021_target_when_emitting_javascript() {
    // TypeScript 7.0.2 case: conformance/es2021/logicalAssignment/logicalAssignment1.ts.
    // The ES2021 baseline preserves &&= syntax.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("logicalAssignment.ts"),
        "let value: number | undefined = undefined; value &&= 42;",
    )
    .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2021));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert!(result.emitted_files()[0].text().contains("value &&= 42"));
}

#[test]
fn should_preserve_disjunction_assignment_given_es2021_target_when_emitting_javascript() {
    // TypeScript 7.0.2 case: conformance/es2021/logicalAssignment/logicalAssignment1.ts.
    // The ES2021 baseline preserves ||= syntax.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("logicalAssignment.ts"),
        "let value: number | undefined = undefined; value ||= 42;",
    )
    .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2021));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert!(result.emitted_files()[0].text().contains("value ||= 42"));
}

#[test]
fn should_preserve_nullish_assignment_given_es2021_target_when_emitting_javascript() {
    // TypeScript 7.0.2 case: conformance/es2021/logicalAssignment/logicalAssignment1.ts.
    // The ES2021 baseline preserves ??= syntax.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("logicalAssignment.ts"),
        "let value: number | undefined = undefined; value ??= 42;",
    )
    .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2021));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert!(result.emitted_files()[0].text().contains("value ??= 42"));
}

#[test]
fn should_lower_conjunction_assignment_given_es2015_target_when_emitting_javascript() {
    // TypeScript 7.0.2 case: conformance/es2021/logicalAssignment/logicalAssignment1.ts.
    // The ES2015 baseline lowers &&= to a short-circuit check followed by assignment.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("logicalAssignment.ts"),
        "let value: number | undefined = undefined; value &&= 42;",
    )
    .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert!(
        result.emitted_files()[0]
            .text()
            .contains("value && (value = 42)")
    );
}

#[test]
fn should_lower_disjunction_assignment_given_es2015_target_when_emitting_javascript() {
    // TypeScript 7.0.2 case: conformance/es2021/logicalAssignment/logicalAssignment1.ts.
    // The ES2015 baseline lowers ||= to a short-circuit check followed by assignment.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("logicalAssignment.ts"),
        "let value: number | undefined = undefined; value ||= 42;",
    )
    .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert!(
        result.emitted_files()[0]
            .text()
            .contains("value || (value = 42)")
    );
}

#[test]
fn should_lower_nullish_assignment_given_es2015_target_when_emitting_javascript() {
    // TypeScript 7.0.2 case: conformance/es2021/logicalAssignment/logicalAssignment1.ts.
    // The ES2015 baseline lowers ??= to a nullish check followed by assignment.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("logicalAssignment.ts"),
        "let value: number | undefined = undefined; value ??= 42;",
    )
    .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert!(
        result.emitted_files()[0]
            .text()
            .contains("value !== null && value !== void 0 ? value : (value = 42)")
    );
}
