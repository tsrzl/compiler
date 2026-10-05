use std::path::Path;

use tsrzl::compiler::{CompilationResult, Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;

fn compile_export_assignment(exporter_text: &str, consumer_text: &str) -> CompilationResult {
    let exporter = SourceFile::from_path(Path::new("exp.ts"), exporter_text)
        .expect("an export-assignment path has a supported source kind");
    let consumer = SourceFile::from_path(Path::new("consumer.ts"), consumer_text)
        .expect("an import-assignment path has a supported source kind");
    let compiler = Compiler::with_options(
        CompilerOptions::new(ScriptTarget::Es2015).with_module(ModuleKind::CommonJs),
    );

    compiler.compile_sources([exporter, consumer])
}

#[test]
fn should_preserve_export_assignment_type_given_relative_import_equals_when_compiling_sources() {
    // Arrange
    // Pinned TypeScript 7.0.2 case: conformance/externalModules/exportAssignTypes.ts.
    // TS-Go accepts a relative import-equals consumer of a CommonJS export assignment.
    let exporter = SourceFile::from_path(
        Path::new("expString.ts"),
        "const answer: string = \"test\"; export = answer;",
    )
    .expect("an export-assignment path has a supported source kind");
    let consumer = SourceFile::from_path(
        Path::new("consumer.ts"),
        "import exported = require(\"./expString\"); const value: string = exported;",
    )
    .expect("an import-assignment path has a supported source kind");
    let compiler = Compiler::with_options(
        CompilerOptions::new(ScriptTarget::Es2015).with_module(ModuleKind::CommonJs),
    );

    // Act
    let result = compiler.compile_sources([exporter, consumer]);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_preserve_numeric_export_type_given_relative_import_equals_when_compiling_sources() {
    // Arrange
    // Pinned TypeScript 7.0.2 case: conformance/externalModules/exportAssignTypes.ts.
    let exporter = "var x = 42; export = x;";
    let consumer = "import iNumber = require('./exp'); const value: number = iNumber;";

    // Act
    let result = compile_export_assignment(exporter, consumer);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_preserve_boolean_export_type_given_relative_import_equals_when_compiling_sources() {
    // Arrange
    // Pinned TypeScript 7.0.2 case: conformance/externalModules/exportAssignTypes.ts.
    let exporter = "var x = true; export = x;";
    let consumer = "import iBoolean = require('./exp'); const value: boolean = iBoolean;";

    // Act
    let result = compile_export_assignment(exporter, consumer);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_preserve_array_element_type_given_relative_import_equals_when_compiling_sources() {
    // Arrange
    // Pinned TypeScript 7.0.2 case: conformance/externalModules/exportAssignTypes.ts.
    let exporter = "var x = [1, 2]; export = x;";
    let consumer = "import iArray = require('./exp'); const value: Array<number> = iArray;";

    // Act
    let result = compile_export_assignment(exporter, consumer);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_preserve_object_property_type_given_relative_import_equals_when_compiling_sources() {
    // Arrange
    // Pinned TypeScript 7.0.2 case: conformance/externalModules/exportAssignTypes.ts.
    let exporter = "var x = { answer: 42, when: 1776 }; export = x;";
    let consumer = "import iObject = require('./exp'); const value: number = iObject.answer;";

    // Act
    let result = compile_export_assignment(exporter, consumer);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_preserve_any_value_type_given_relative_import_equals_when_compiling_sources() {
    // Arrange
    // Pinned TypeScript 7.0.2 case: conformance/externalModules/exportAssignTypes.ts.
    let exporter = "var x; export = x;";
    let consumer = "import iAny = require('./exp'); const value: string = iAny;";

    // Act
    let result = compile_export_assignment(exporter, consumer);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_preserve_function_return_type_given_relative_import_equals_when_compiling_sources() {
    // Arrange
    // Pinned TypeScript 7.0.2 case: conformance/externalModules/exportAssignTypes.ts.
    let exporter = "function x(a: number): number { return a; } export = x;";
    let consumer = "import iFunction = require('./exp'); const value: number = iFunction(42);";

    // Act
    let result = compile_export_assignment(exporter, consumer);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_infer_generic_call_result_given_relative_import_equals_when_compiling_sources() {
    // Arrange
    // Pinned TypeScript 7.0.2 case: conformance/externalModules/exportAssignTypes.ts.
    let exporter = "function x<T>(a: T): T { return a; } export = x;";
    let consumer = "import iGeneric = require('./exp'); const value: number = iGeneric(42);";

    // Act
    let result = compile_export_assignment(exporter, consumer);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_report_string_to_number_mismatch_given_relative_import_equals_when_compiling_sources() {
    // Arrange
    // Pinned TypeScript 7.0.2 case: conformance/externalModules/exportAssignTypes.ts.
    let exporter = "var x = \"test\"; export = x;";
    let consumer = "import iString = require('./exp'); const value: number = iString;";

    // Act
    let result = compile_export_assignment(exporter, consumer);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2322),
        "expected TS2322 when assigning the exported string value to number, got {:?}",
        result.diagnostics()
    );
}
