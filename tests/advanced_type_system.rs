use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

fn compile(source: &str) -> tsrzl::compiler::CompilationResult {
    let source_file = SourceFile::from_path(Path::new("advanced.ts"), source)
        .expect("a TypeScript path has a supported source kind");
    Compiler::new().compile(source_file)
}

fn declaration_for(source: &str) -> String {
    let source_file = SourceFile::from_path(Path::new("advanced.ts"), source)
        .expect("a TypeScript path has a supported source kind");
    let compiler =
        Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2025).with_declaration(true));
    let result = compiler.compile(source_file);
    result
        .emitted_files()
        .iter()
        .find(|file| {
            file.path().file_name().and_then(|name| name.to_str()) == Some("advanced.d.ts")
        })
        .expect("declaration output is emitted")
        .text()
        .to_owned()
}

#[test]
fn should_instantiate_generic_interface_given_structural_assignment_when_checking_types() {
    // TypeScript 7 case: conformance/types/namedTypes/genericInstantiationEquivalentToObjectLiteral.ts
    // Arrange
    let source = "interface Pair<Left, Right> { first: Left; second: Right; } const pair: Pair<string, number> = { first: \"Ada\", second: 36 }; const copy: { first: string; second: number } = pair;";

    // Act
    let result = compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_inherit_interface_members_given_extends_clause_when_checking_types() {
    // TypeScript 7 case: conformance/interfaces/interfaceDeclarations/interfaceExtendsObjectIntersection.ts
    // Arrange
    let source = "interface Base { id: number; } interface Person extends Base { name: string; } const person: Person = { id: 1, name: \"Ada\" }; const id: number = person.id;";

    // Act
    let result = compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_narrow_nullable_string_given_truthy_guard_when_checking_types() {
    // TypeScript 7 case: conformance/controlFlow/controlFlowTruthiness.ts
    // Arrange
    let source = "function format(value: string | undefined): string { if (value) { return value; } return \"empty\"; }";
    let source_file = SourceFile::from_path(Path::new("advanced.ts"), source)
        .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(
        CompilerOptions::new(ScriptTarget::Es2025).with_strict_null_checks(true),
    );

    // Act
    let result = compiler.compile(source_file);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_report_no_matching_overload_given_boolean_argument_when_checking_types() {
    // TypeScript 7 case: conformance/expressions/functionCalls/overloadResolution.ts
    // Arrange
    let source = "function parse(value: string): string; function parse(value: number): number; function parse(value: string | number): string | number { return value; } parse(true);";

    // Act
    let result = compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2769)
    );
}

#[test]
fn should_preserve_generic_parameters_given_exported_function_when_emitting_declarations() {
    // TypeScript 7 case: conformance/types/typeRelationships/typeInference/genericFunctionParameters.ts
    // Arrange
    let source = "export function identity<T>(value: T): T { return value; }";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export declare function identity<T>(value: T): T;\n"
    );
}

#[test]
fn should_preserve_readonly_interface_property_given_exported_interface_when_emitting_declarations()
{
    // TypeScript 7 case: conformance/controlFlow/typeGuardsAsAssertions.ts
    // Arrange
    let source = "export interface None { readonly none: string; }";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export interface None {\n    readonly none: string;\n}\n"
    );
}
