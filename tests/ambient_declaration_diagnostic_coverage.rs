use std::path::Path;

use tsrzl::compiler::{CompilationResult, Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;

fn compile_source(text: &str) -> CompilationResult {
    let source = SourceFile::from_path(Path::new("ambient.ts"), text)
        .expect("the ambient declaration path has a supported source kind");
    let compiler = Compiler::with_options(
        CompilerOptions::new(ScriptTarget::Es2015).with_module(ModuleKind::CommonJs),
    );

    compiler.compile(source)
}

#[test]
fn should_report_initializer_given_ambient_variable_declaration_when_compiling_sources() {
    // Pinned TypeScript 7.0.2 case: conformance/ambient/ambientErrors.ts.
    // TS-Go reports TS1039 for an initializer on an ambient variable.
    // Arrange
    let source = "declare var ambientValue = 4;";

    // Act
    let result = compile_source(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 1039),
        "expected TS1039 for an ambient variable initializer, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_report_function_body_given_ambient_function_declaration_when_compiling_sources() {
    // Pinned TypeScript 7.0.2 case: conformance/ambient/ambientErrors.ts.
    // TS-Go reports TS1183 when an ambient function declares an implementation.
    // Arrange
    let source = "declare function ambientFunction() {};";

    // Act
    let result = compile_source(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 1183),
        "expected TS1183 for an ambient function body, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_report_parameter_initializer_given_ambient_function_signature_when_compiling_sources() {
    // Pinned TypeScript 7.0.2 case: conformance/ambient/ambientErrors.ts.
    // TS-Go reports TS2371 for a default parameter on an ambient signature.
    // Arrange
    let source = "declare function ambientFunction(value = 3): void;";

    // Act
    let result = compile_source(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2371),
        "expected TS2371 for an ambient parameter initializer, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_report_nonconstant_initializer_given_ambient_enum_member_when_compiling_sources() {
    // Pinned TypeScript 7.0.2 case: conformance/ambient/ambientErrors.ts.
    // TS-Go reports TS1066 for a computed ambient enum member initializer.
    // Arrange
    let source = "declare enum AmbientEnum { value = 'foo'.length }";

    // Act
    let result = compile_source(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 1066),
        "expected TS1066 for a computed ambient enum initializer, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_accept_exported_ambient_variable_given_namespace_member_when_compiling_sources() {
    // Pinned TypeScript 7.0.2 case: conformance/ambient/ambientInsideNonAmbient.ts.
    // TS-Go accepts an exported ambient variable inside a namespace declaration.
    // Arrange
    let source = "namespace Container { export declare var value: string; }";

    // Act
    let result = compile_source(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_report_nested_module_given_ambient_module_inside_namespace_when_compiling_sources() {
    // Pinned TypeScript 7.0.2 case: conformance/ambient/ambientErrors.ts.
    // TS-Go reports TS2435 when an ambient module is nested in a namespace.
    // Arrange
    let source = "namespace Container { declare module \"nested\" {} }";

    // Act
    let result = compile_source(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2435),
        "expected TS2435 for a nested ambient module, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_report_relative_name_given_ambient_module_declaration_when_compiling_sources() {
    // Pinned TypeScript 7.0.2 case: conformance/ambient/ambientErrors.ts.
    // TS-Go reports TS2436 because ambient module names must be non-relative.
    // Arrange
    let source = "declare module \"../relative\" {}";

    // Act
    let result = compile_source(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2436),
        "expected TS2436 for a relative ambient module name, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_report_mixed_ambient_exports_given_export_assignment_when_compiling_sources() {
    // Pinned TypeScript 7.0.2 case: conformance/ambient/ambientErrors.ts.
    // TS-Go reports TS2309 when an ambient module mixes export = with named exports.
    // Arrange
    let source =
        "declare module \"bar\" { var value: number; export var extra: number; export = value; }";

    // Act
    let result = compile_source(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2309),
        "expected TS2309 for mixed ambient module exports, got {:?}",
        result.diagnostics()
    );
}
