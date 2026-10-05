use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

fn declaration_for(source: &str) -> String {
    let source_file = SourceFile::from_path(Path::new("contract.ts"), source)
        .expect("a TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2025).with_declaration(true);
    let result = Compiler::with_options(options).compile(source_file);
    result
        .emitted_files()
        .iter()
        .find(|file| {
            file.path().file_name().and_then(|name| name.to_str()) == Some("contract.d.ts")
        })
        .expect("declaration output is emitted")
        .text()
        .to_owned()
}

#[test]
fn should_emit_exported_function_signature_given_typed_function_when_emitting_declarations() {
    // Arrange
    let source = "export function greet(name: string): string { return name; }";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export declare function greet(name: string): string;\n"
    );
}

#[test]
fn should_emit_exported_class_method_signature_given_typed_class_when_emitting_declarations() {
    // Arrange
    let source = "export class Greeter { greet(name: string): string { return name; } }";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export declare class Greeter {\n    greet(name: string): string;\n}\n"
    );
}

#[test]
fn should_preserve_static_class_method_given_exported_class_when_emitting_declarations() {
    // Arrange
    let source = "export class Counter { static current(): number { return 0; } }";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export declare class Counter {\n    static current(): number;\n}\n"
    );
}

#[test]
fn should_preserve_base_class_given_extends_clause_when_emitting_declarations() {
    // Arrange
    let source = "export class Animal {} export class Dog extends Animal {}";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export declare class Animal {\n}\nexport declare class Dog extends Animal {\n}\n"
    );
}

#[test]
fn should_emit_class_property_signature_given_initialized_typed_property_when_emitting_declarations()
 {
    // Arrange
    let source = "export class Greeter { name: string = \"world\"; }";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export declare class Greeter {\n    name: string;\n}\n"
    );
}

#[test]
fn should_preserve_static_class_property_given_exported_class_when_emitting_declarations() {
    // Arrange
    let source = "export class Counter { static count: number = 0; }";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export declare class Counter {\n    static count: number;\n}\n"
    );
}

#[test]
fn should_emit_exported_enum_given_auto_incremented_members_when_emitting_declarations() {
    // Arrange
    let source = "export enum Direction { Up, Down }";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export declare enum Direction {\n    Up = 0,\n    Down = 1\n}\n"
    );
}

#[test]
fn should_emit_parameter_property_signature_given_public_constructor_parameter_when_emitting_declarations()
 {
    // Arrange
    let source = "export class Greeter { constructor(public name: string) {} }";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export declare class Greeter {\n    name: string;\n    constructor(name: string);\n}\n"
    );
}

#[test]
fn should_preserve_parameter_property_modifiers_given_private_readonly_constructor_parameter() {
    // Arrange
    let source = "export class Greeter { constructor(private readonly name: string) {} }";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export declare class Greeter {\n    private readonly name;\n    constructor(name: string);\n}\n"
    );
}

#[test]
fn should_preserve_private_readonly_class_property_given_exported_class_when_emitting_declarations()
{
    // Arrange
    let source = "export class Secret { private readonly token: string = \"x\"; }";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export declare class Secret {\n    private readonly token;\n}\n"
    );
}

#[test]
fn should_preserve_private_method_given_exported_class_when_emitting_declarations() {
    // Arrange
    let source = "export class Secret { private reveal(): string { return \"x\"; } }";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export declare class Secret {\n    private reveal;\n}\n"
    );
}

#[test]
fn should_preserve_protected_class_property_given_exported_class_when_emitting_declarations() {
    // Arrange
    let source = "export class Base { protected count: number = 0; }";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export declare class Base {\n    protected count: number;\n}\n"
    );
}

#[test]
fn should_emit_class_instance_type_given_exported_constructed_value_when_emitting_declarations() {
    // Arrange
    let source = "export class Greeter {} export const greeter = new Greeter();";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export declare class Greeter {\n}\nexport declare const greeter: Greeter;\n"
    );
}

#[test]
fn should_mark_default_parameter_optional_given_exported_function_when_emitting_declarations() {
    // Arrange
    let source = "export function greet(name: string = 'world'): string { return name; }";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export declare function greet(name?: string): string;\n"
    );
}

#[test]
fn should_infer_default_parameter_type_given_exported_function_when_emitting_declarations() {
    // Arrange
    let source = "export function greet(name = 'world') { return name; }";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export declare function greet(name?: string): string;\n"
    );
}

#[test]
fn should_infer_default_parameter_type_given_exported_arrow_when_emitting_declarations() {
    // Arrange
    let source = "export const greet = (name = 'world') => name;";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export declare const greet: (name?: string) => string;\n"
    );
}

#[test]
fn should_infer_boolean_logical_return_given_boolean_parameters_when_emitting_declarations() {
    // Arrange
    let source =
        "export function areBothReady(first: boolean, second: boolean) { return first && second; }";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export declare function areBothReady(first: boolean, second: boolean): boolean;\n"
    );
}

#[test]
fn should_infer_string_nullish_return_given_optional_string_parameter_when_emitting_declarations() {
    // Arrange
    let source = "export function label(value: string | undefined) { return value ?? 'fallback'; }";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export declare function label(value: string | undefined): string;\n"
    );
}

#[test]
fn should_infer_string_conditional_return_given_boolean_parameter_when_emitting_declarations() {
    // Arrange
    let source = "export function label(enabled: boolean) { return enabled ? 'on' : 'off'; }";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export declare function label(enabled: boolean): string;\n"
    );
}

#[test]
fn should_emit_array_type_given_exported_array_variable_when_emitting_declarations() {
    // Arrange
    let source = "export const values: number[] = [1, 2];";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(declaration, "export declare const values: number[];\n");
}

#[test]
fn should_emit_typed_arrow_signature_given_exported_arrow_variable_when_emitting_declarations() {
    // Arrange
    let source = "export const double = (value: number): number => value * 2;";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export declare const double: (value: number) => number;\n"
    );
}

#[test]
fn should_infer_arrow_signature_given_unannotated_return_when_emitting_declarations() {
    // Arrange
    let source = "export const double = (value: number) => value * 2;";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export declare const double: (value: number) => number;\n"
    );
}

#[test]
fn should_infer_arrow_signature_given_typed_block_body_when_emitting_declarations() {
    // Arrange
    let source = "export const double = (value: number) => { return value * 2; };";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export declare const double: (value: number) => number;\n"
    );
}

#[test]
fn should_infer_array_type_given_exported_array_variable_when_emitting_declarations() {
    // Arrange
    let source = "export const values = [1, 2];";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(declaration, "export declare const values: number[];\n");
}

#[test]
fn should_infer_bigint_given_exported_bigint_arithmetic_when_emitting_declarations() {
    // Arrange
    let source = "export const total = 1n + 2n;";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(declaration, "export declare const total: bigint;\n");
}

#[test]
fn should_emit_exported_interface_given_interface_declaration_when_emitting_declarations() {
    // Arrange
    let source = "export interface User { name: string; age?: number; }";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export interface User {\n    name: string;\n    age?: number;\n}\n"
    );
}

#[test]
fn should_emit_exported_type_alias_given_union_alias_when_emitting_declarations() {
    // Arrange
    let source = "export type Answer = string | number;";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(declaration, "export type Answer = string | number;\n");
}

#[test]
fn should_preserve_named_reexport_given_declaration_emit_when_emitting_declarations() {
    // Arrange
    let source = "export { answer as renamed } from './answer';";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export { answer as renamed } from './answer';\n"
    );
}

#[test]
fn should_preserve_type_only_named_reexport_given_declaration_emit_when_emitting_declarations() {
    // Arrange
    let source = "export type { Answer as Renamed } from './answer';";

    // Act
    let declaration = declaration_for(source);

    // Assert
    assert_eq!(
        declaration,
        "export type { Answer as Renamed } from './answer';\n"
    );
}
