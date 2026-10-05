use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions};
use tsrzl::source_file::SourceFile;
use tsrzl::source_text::Utf16Offset;

#[test]
fn should_report_assignability_error_given_string_initializer_for_number_annotation_when_checking_types()
 {
    // Arrange
    let source = SourceFile::from_path(Path::new("answer.ts"), "const answer: number = 'wrong';")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2322)
    );
}

#[test]
fn should_accept_indexed_access_type_given_interface_property_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("person.ts"),
        "interface Person { name: string; } type Name = Person[\"name\"]; const name: Name = \"Ada\";",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_resolve_overload_given_matching_string_argument_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("parse.ts"),
        "function parse(value: string): string; function parse(value: number): number; function parse(value: string | number): string | number { return value; } const text: string = parse(\"answer\");",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_narrow_union_given_typeof_guard_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("format.ts"),
        "function toNumber(value: string | number): number { if (typeof value === \"number\") return value; return 0; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_accept_tuple_type_given_matching_array_literal_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("pair.ts"),
        "const pair: [string, number] = [\"answer\", 42];",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_accept_generic_constraint_given_matching_argument_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("echo.ts"),
        "function echo<T extends string>(value: T): T { return value; } const literal: \"answer\" = echo(\"answer\");",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_infer_conditional_type_member_given_array_type_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("element.ts"),
        "type ItemOf<T> = T extends (infer Item)[] ? Item : never; const element: ItemOf<number[]> = 42;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_accept_keyof_type_given_interface_property_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("person.ts"),
        "interface Person { name: string; age: number; } type PersonKey = keyof Person; const key: PersonKey = \"name\";",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_narrow_discriminated_union_given_literal_property_guard_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("shape.ts"),
        "type Shape = { kind: \"circle\"; radius: number } | { kind: \"square\"; side: number }; function area(shape: Shape): number { if (shape.kind === \"circle\") return shape.radius; return shape.side; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_accept_intersection_type_given_matching_object_properties_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("person.ts"),
        "type NamedPerson = { name: string } & { age: number }; const person: NamedPerson = { name: \"Ada\", age: 36 }; const age: number = person.age;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_narrow_unknown_given_typeof_string_guard_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("format.ts"),
        "function format(value: unknown): string { if (typeof value === \"string\") return value; return \"unknown\"; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_reject_mapped_readonly_assignment_given_readonly_property_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("person.ts"),
        "interface Person { name: string; } type Immutable<T> = { readonly [Key in keyof T]: T[Key] }; let person: Immutable<Person> = { name: \"Ada\" }; person.name = \"Grace\";",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2540)
    );
}

#[test]
fn should_assign_bigint_literal_given_bigint_annotation_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(Path::new("value.ts"), "const value: bigint = 1n;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_report_incompatible_satisfies_type_given_string_expression_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "const answer = 'wrong' satisfies number;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 1360);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'string' does not satisfy the expected type 'number'."
    );
}

#[test]
fn should_accept_const_assertion_given_string_literal_when_checking_types() {
    // Arrange
    let source =
        SourceFile::from_path(Path::new("answer.ts"), "const answer = \"ready\" as const;")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_check_generic_function_call_given_identity_function_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("identity.ts"),
        "function identity<T>(value: T): T { return value; } const answer: string = identity(\"Ada\");",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_report_unknown_identifier_given_template_interpolation_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("message.ts"),
        "const message = `hello ${missing}`;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2304);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Cannot find name 'missing'."
    );
}

#[test]
fn should_accept_conditional_type_given_generic_type_parameter_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("result.ts"),
        "type Result<T> = T extends string ? string : number;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_accept_mapped_type_given_keyof_type_parameter_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("readonly-fields.ts"),
        "type ReadonlyFields<T> = { readonly [Key in keyof T]: T[Key] };",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_accept_template_literal_type_given_string_type_interpolation_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greeting.ts"),
        r"type Greeting = `hello ${string}`;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_assign_hexadecimal_bigint_literal_given_bigint_annotation_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(Path::new("value.ts"), "const value: bigint = 0x1n;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_report_bigint_arithmetic_assignment_given_string_annotation_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(Path::new("total.ts"), "const total: string = 1n + 2n;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'bigint' is not assignable to type 'string'."
    );
}

#[test]
fn should_reject_mixed_numeric_kinds_given_bigint_addition_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(Path::new("invalid.ts"), "const invalid = 1n + 2;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2365)
    );
}

#[test]
fn should_report_unknown_type_given_type_assertion_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(Path::new("answer.ts"), "const answer = 42 as Missing;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2304);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Cannot find name 'Missing'."
    );
}

#[test]
fn should_report_method_return_mismatch_given_typed_class_method_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greeter.ts"),
        "class Greeter { greet(name: number): string { return name; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'number' is not assignable to type 'string'."
    );
}

#[test]
fn should_resolve_class_type_given_class_parameter_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greeter.ts"),
        "class Greeter {} function accept(value: Greeter): void {}",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_resolve_this_property_given_class_method_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("counter.ts"),
        "class Counter { value: number = 0; increment(): number { return this.value + 1; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_resolve_this_property_given_class_field_initializer_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("counter.ts"),
        "class Counter { value: number = 1; snapshot: number = this.value; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_resolve_static_property_type_given_class_value_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("counter.ts"),
        "class Counter { static count: number = 0; } const label: string = Counter.count;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'number' is not assignable to type 'string'."
    );
}

#[test]
fn should_resolve_enum_member_given_enum_typed_variable_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("direction.ts"),
        "enum Direction { Up, Down } const current: Direction = Direction.Up;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_assign_string_enum_member_to_string_given_member_access_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("direction.ts"),
        "enum Direction { Up = \"UP\" } const name: string = Direction.Up;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_assign_numeric_enum_member_to_number_given_member_access_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("level.ts"),
        "enum Level { Low = 1 } const value: number = Level.Low;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_assign_numeric_enum_variable_to_number_given_numeric_members_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("level.ts"),
        "enum Level { Low = 1 } let level: Level = Level.Low; const value: number = level;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_report_missing_enum_initializer_given_string_member_predecessor_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("direction.ts"),
        "enum Direction { Up = \"UP\", Down }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 1061);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Enum member must have initializer."
    );
}

#[test]
fn should_reject_missing_static_property_given_class_value_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("counter.ts"),
        "class Counter { static count: number = 0; } Counter.missing;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2339);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Property 'missing' does not exist on type 'typeof Counter'."
    );
}

#[test]
fn should_resolve_static_property_given_static_method_this_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("counter.ts"),
        "class Counter { static count: number = 0; static next(): number { return this.count + 1; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_reject_static_property_access_given_instance_value_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("counter.ts"),
        "class Counter { static count: number = 0; } const counter = new Counter(); const count: number = counter.count;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2339);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Property 'count' does not exist on type 'Counter'."
    );
}

#[test]
fn should_resolve_inherited_static_property_given_derived_class_value_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("counter.ts"),
        "class BaseCounter { static count: number = 0; } class Counter extends BaseCounter {} const label: string = Counter.count;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'number' is not assignable to type 'string'."
    );
}

#[test]
fn should_resolve_inherited_class_property_given_constructed_subclass_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("animals.ts"),
        "class Animal { name: string = \"Ada\"; } class Dog extends Animal {} const dog = new Dog(); const label: string = dog.name;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_report_incompatible_inherited_property_given_subclass_override_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("animals.ts"),
        "class Animal { name: string = \"\"; } class Dog extends Animal { name: number = 42; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2416);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Property 'name' in type 'Dog' is not assignable to the same property in base type 'Animal'.\n  Type 'number' is not assignable to type 'string'."
    );
}

#[test]
fn should_report_unresolved_base_class_given_missing_extends_name_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(Path::new("animals.ts"), "class Dog extends Missing {}")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2304);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Cannot find name 'Missing'."
    );
}

#[test]
fn should_allow_super_call_given_derived_constructor_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("animals.ts"),
        "class Animal {} class Dog extends Animal { constructor() { super(); } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_report_missing_super_argument_given_required_base_constructor_parameter_when_checking_types()
 {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("animals.ts"),
        "class Animal { constructor(name: string) {} } class Dog extends Animal { constructor() { super(); } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2554);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Expected 1 arguments, but got 0."
    );
}

#[test]
fn should_report_property_initializer_mismatch_given_typed_class_property_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("counter.ts"),
        "class Counter { value: number = \"wrong\"; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'string' is not assignable to type 'number'."
    );
}

#[test]
fn should_report_constructor_argument_mismatch_given_typed_class_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greeter.ts"),
        "class Greeter { constructor(name: string) {} } new Greeter(42);",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2345);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Argument of type 'number' is not assignable to parameter of type 'string'."
    );
}

#[test]
fn should_infer_class_instance_type_given_new_expression_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greeter.ts"),
        "class Greeter {} let greeter = new Greeter(); greeter = \"wrong\";",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'string' is not assignable to type 'Greeter'."
    );
}

#[test]
fn should_infer_class_method_return_type_given_constructed_instance_call_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greeter.ts"),
        "class Greeter { greet(): string { return \"hello\"; } } const message: number = new Greeter().greet();",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'string' is not assignable to type 'number'."
    );
}

#[test]
fn should_infer_unannotated_class_method_return_given_string_return_expression_when_checking_types()
{
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greeter.ts"),
        "class Greeter { greet() { return \"hello\"; } } const message: number = new Greeter().greet();",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'string' is not assignable to type 'number'."
    );
}

#[test]
fn should_report_incompatible_class_method_argument_given_typed_method_call_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greeter.ts"),
        "class Greeter { greet(name: string): string { return name; } } new Greeter().greet(42);",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2345);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Argument of type 'number' is not assignable to parameter of type 'string'."
    );
}

#[test]
fn should_report_argument_given_default_constructor_without_parameters_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greeter.ts"),
        "class Greeter {} new Greeter(\"Ada\");",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2554);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Expected 0 arguments, but got 1."
    );
}

#[test]
fn should_check_class_property_type_given_constructed_instance_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greeter.ts"),
        "class Greeter { name: string = \"world\"; } const greeter = new Greeter(); const size: number = greeter.name;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'string' is not assignable to type 'number'."
    );
}

#[test]
fn should_check_constructor_parameter_property_type_given_constructed_instance_when_checking_types()
{
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greeter.ts"),
        "class Greeter { constructor(public name: string) {} } const greeter = new Greeter(\"Ada\"); const size: number = greeter.name;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'string' is not assignable to type 'number'."
    );
}

#[test]
fn should_reject_write_to_readonly_parameter_property_given_constructed_instance_when_checking_types()
 {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greeter.ts"),
        "class Greeter { constructor(public readonly name: string) {} } const greeter = new Greeter(\"Ada\"); greeter.name = \"Grace\";",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2540);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Cannot assign to 'name' because it is a read-only property."
    );
}

#[test]
fn should_allow_readonly_parameter_property_assignment_given_its_constructor_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greeter.ts"),
        "class Greeter { constructor(public readonly name: string) { this.name = name; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_reject_readonly_parameter_property_assignment_given_class_method_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greeter.ts"),
        "class Greeter { constructor(public readonly name: string) {} rename(): void { this.name = \"Grace\"; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2540);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Cannot assign to 'name' because it is a read-only property."
    );
}

#[test]
fn should_reject_private_parameter_property_access_given_external_instance_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("secret.ts"),
        "class Secret { constructor(private readonly token: string) {} } const secret = new Secret(\"x\"); const token: string = secret.token;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2341);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Property 'token' is private and only accessible within class 'Secret'."
    );
}

#[test]
fn should_allow_private_parameter_property_access_given_same_class_method_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("secret.ts"),
        "class Secret { constructor(private readonly token: string) {} copy(other: Secret): string { return other.token; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_reject_protected_parameter_property_access_given_external_instance_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("secret.ts"),
        "class Base { constructor(protected readonly token: string) {} } class Child extends Base { reveal(): string { return this.token; } } const base = new Base(\"x\"); const token: string = base.token;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2445);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Property 'token' is protected and only accessible within class 'Base' and its subclasses."
    );
}

#[test]
fn should_allow_protected_parameter_property_access_given_subclass_method_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("secret.ts"),
        "class Base { constructor(protected readonly token: string) {} } class Child extends Base { reveal(): string { return this.token; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_reject_private_class_property_access_given_external_instance_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("secret.ts"),
        "class Secret { private token: string = \"x\"; } const secret = new Secret(); const token: string = secret.token;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2341);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Property 'token' is private and only accessible within class 'Secret'."
    );
}

#[test]
fn should_reject_protected_class_property_access_given_external_instance_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("base.ts"),
        "class Base { protected token: string = \"x\"; } const base = new Base(); const token: string = base.token;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2445);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Property 'token' is protected and only accessible within class 'Base' and its subclasses."
    );
}

#[test]
fn should_reject_write_to_readonly_class_property_given_external_instance_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greeter.ts"),
        "class Greeter { readonly name: string = \"Ada\"; } const greeter = new Greeter(); greeter.name = \"Grace\";",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2540);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Cannot assign to 'name' because it is a read-only property."
    );
}

#[test]
fn should_allow_readonly_class_property_assignment_given_its_constructor_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greeter.ts"),
        "class Greeter { readonly name: string = \"Ada\"; constructor() { this.name = \"Grace\"; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_allow_private_class_property_access_given_same_class_method_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("secret.ts"),
        "class Secret { private token: string = \"x\"; copy(other: Secret): string { return other.token; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_allow_protected_class_property_access_given_subclass_method_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("base.ts"),
        "class Base { protected token: string = \"x\"; } class Child extends Base { reveal(): string { return this.token; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_reject_private_class_method_call_given_external_instance_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("secret.ts"),
        "class Secret { private reveal(): string { return \"x\"; } } const secret = new Secret(); secret.reveal();",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2341);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Property 'reveal' is private and only accessible within class 'Secret'."
    );
}

#[test]
fn should_infer_number_for_of_binding_given_number_array_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("values.ts"),
        "function inspect(values: number[]): void { for (const value of values) { const text: string = value; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2322)
    );
}

#[test]
fn should_report_non_iterable_given_for_of_expression_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("values.ts"),
        "function inspect(count: number): void { for (const value of count) { } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2488)
    );
}

#[test]
fn should_report_non_object_given_for_in_expression_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("keys.ts"),
        "function iterate(value: number): void { for (const key in value) { } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2407)
    );
}

#[test]
fn should_allow_break_given_switch_clause_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("choice.ts"),
        "function choose(code: number): number { switch (code) { case 0: break; default: return 1; } return 0; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_check_catch_binding_type_given_unknown_annotation_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("catch.ts"),
        "function inspect(): void { try { } catch (error: unknown) { const text: string = error; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'unknown' is not assignable to type 'string'."
    );
}

#[test]
fn should_reject_invalid_catch_binding_type_given_number_annotation_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("catch.ts"),
        "function inspect(): void { try { } catch (error: number) { } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 1196);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Catch clause variable type annotation must be 'any' or 'unknown' if specified."
    );
}

#[test]
fn should_allow_catch_binding_given_unannotated_error_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("catch.ts"),
        "function recover(error: any): void {} function inspect(): void { try { } catch (error) { recover(error); } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_report_arrow_return_mismatch_given_typed_expression_body_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("double.ts"),
        "const double = (value: number): string => value * 2;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'number' is not assignable to type 'string'."
    );
}

#[test]
fn should_report_arrow_return_mismatch_given_typed_block_body_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("double.ts"),
        "const double = (value: number): string => { return value * 2; };",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'number' is not assignable to type 'string'."
    );
}

#[test]
fn should_report_incompatible_arrow_argument_given_typed_arrow_variable_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("double.ts"),
        "const double = (value: number): number => value * 2; double(\"wrong\");",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2345);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Argument of type 'string' is not assignable to parameter of type 'number'."
    );
}

#[test]
fn should_infer_arrow_call_return_type_given_typed_expression_body_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("double.ts"),
        "const double = (value: number) => value * 2; const label: string = double(2);",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'number' is not assignable to type 'string'."
    );
}

#[test]
fn should_infer_arrow_call_return_type_given_typed_block_body_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("double.ts"),
        "const double = (value: number) => { return value * 2; }; const label: string = double(2);",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'number' is not assignable to type 'string'."
    );
}

#[test]
fn should_report_missing_arrow_argument_given_typed_arrow_variable_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("double.ts"),
        "const double = (value: number) => value * 2; double();",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2554);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Expected 1 arguments, but got 0."
    );
}

#[test]
fn should_report_incompatible_local_arrow_argument_given_typed_arrow_variable_when_checking_types()
{
    // Arrange
    let source = SourceFile::from_path(
        Path::new("double.ts"),
        "function inspect(): void { const double = (value: number): number => value * 2; double(\"wrong\"); }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2345);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Argument of type 'string' is not assignable to parameter of type 'number'."
    );
}

#[test]
fn should_report_return_given_top_level_conditional_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(Path::new("return.ts"), "if (true) { return; }")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 1108)
    );
}

#[test]
fn should_report_incompatible_assignment_given_typed_mutable_variable_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("count.ts"),
        "let count: number = 1; count = \"wrong\";",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'string' is not assignable to type 'number'."
    );
}

#[test]
fn should_report_assignment_to_constant_given_const_variable_when_checking_types() {
    // Arrange
    let source =
        SourceFile::from_path(Path::new("count.ts"), "const count: number = 1; count = 2;")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2588);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Cannot assign to 'count' because it is a constant."
    );
}

#[test]
fn should_report_assignment_to_local_constant_given_const_variable_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("update.ts"),
        "function update(): void { const count = 1; count = 2; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2588);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Cannot assign to 'count' because it is a constant."
    );
}

#[test]
fn should_report_invalid_assignment_target_given_literal_left_hand_side_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(Path::new("invalid.ts"), "1 = 2;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2364);
    assert_eq!(
        result.diagnostics()[0].message(),
        "The left-hand side of an assignment expression must be a variable or a property access."
    );
}

#[test]
fn should_accept_string_concatenation_given_string_add_assignment_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("label.ts"),
        "let label: string = 'value'; label += 1;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_report_incompatible_add_assignment_given_number_target_and_string_operand_when_checking_types()
 {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("count.ts"),
        "let count: number = 1; count += 'wrong';",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'string' is not assignable to type 'number'."
    );
}

#[test]
fn should_report_disjoint_strict_equality_given_number_and_string_literals_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(Path::new("comparison.ts"), "const result = 1 === '1';")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2367);
    assert_eq!(
        result.diagnostics()[0].message(),
        "This comparison appears to be unintentional because the types 'number' and 'string' have no overlap."
    );
}

#[test]
fn should_report_boolean_logical_result_assigned_to_number_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("result.ts"),
        "const result: number = true && false;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'boolean' is not assignable to type 'number'."
    );
}

#[test]
fn should_report_boolean_disjunction_result_assigned_to_number_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("result.ts"),
        "const result: number = true || false;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'boolean' is not assignable to type 'number'."
    );
}

#[test]
fn should_report_string_right_operand_given_numeric_subtraction_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("difference.ts"),
        "const difference = 1 - 'wrong';",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2363);
    assert_eq!(
        result.diagnostics()[0].message(),
        "The right-hand side of an arithmetic operation must be of type 'any', 'number', 'bigint' or an enum type."
    );
}

#[test]
fn should_report_string_nullish_result_given_number_return_type_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("value.ts"),
        "function requireNumber(value: string | undefined): number { return value ?? 'fallback'; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'string' is not assignable to type 'number'."
    );
}

#[test]
fn should_report_unreachable_nullish_fallback_given_non_nullable_string_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("value.ts"),
        "const value = 'present' ?? 'fallback';",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2869);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Right operand of ?? is unreachable because the left operand is never nullish."
    );
}

#[test]
fn should_report_string_conditional_branch_given_number_return_type_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("value.ts"),
        "function valueOrZero(enabled: boolean): number { return enabled ? 'wrong' : 0; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'string' is not assignable to type 'number'."
    );
}

#[test]
fn should_report_incompatible_assignment_given_while_loop_body_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("update.ts"),
        "function update(value: number): void { while (value > 0) { value = 'wrong'; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'string' is not assignable to type 'number'."
    );
}

#[test]
fn should_report_break_outside_iteration_given_top_level_statement_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(Path::new("invalid.ts"), "break;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 1105);
    assert_eq!(
        result.diagnostics()[0].message(),
        "A 'break' statement can only be used within an enclosing iteration or switch statement."
    );
}

#[test]
fn should_report_continue_outside_iteration_given_top_level_statement_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(Path::new("invalid.ts"), "continue;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 1104);
    assert_eq!(
        result.diagnostics()[0].message(),
        "A 'continue' statement can only be used within an enclosing iteration statement."
    );
}

#[test]
fn should_report_null_for_non_nullable_type_given_strict_null_checks_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(Path::new("value.ts"), "const value: number = null;")
        .expect("a TypeScript path has a supported source kind");
    let binding_start = source
        .text()
        .as_str()
        .find("value:")
        .expect("the variable declaration contains its binding name");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'null' is not assignable to type 'number'."
    );
    assert_eq!(
        result.diagnostics()[0].span().start(),
        Utf16Offset::new(binding_start)
    );
}

#[test]
fn should_report_undefined_for_non_nullable_type_given_strict_null_checks_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(Path::new("value.ts"), "const value: number = undefined;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'undefined' is not assignable to type 'number'."
    );
}

#[test]
fn should_allow_null_assignment_given_disabled_strict_null_checks_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(Path::new("value.ts"), "const value: number = null;")
        .expect("a TypeScript path has a supported source kind");
    let compiler =
        Compiler::with_options(CompilerOptions::default().with_strict_null_checks(false));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_allow_undefined_assignment_given_disabled_strict_null_checks_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(Path::new("value.ts"), "const value: number = undefined;")
        .expect("a TypeScript path has a supported source kind");
    let compiler =
        Compiler::with_options(CompilerOptions::default().with_strict_null_checks(false));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_resolve_type_alias_given_typed_variable_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "type Answer = number; const answer: Answer = 42;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
}

#[test]
fn should_accept_union_member_given_union_type_alias_and_matching_literal_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("value.ts"),
        "type Value = number | string; const value: Value = 'text';",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
}

#[test]
fn should_accept_union_member_given_direct_union_annotation_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("value.ts"),
        "const value: number | string = 'text';",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
}

#[test]
fn should_accept_required_interface_properties_given_matching_object_literal_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("person.ts"),
        "interface Person { name: string; } const person: Person = { name: 'Ada' };",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
}

#[test]
fn should_report_unknown_object_property_given_typed_object_literal_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("person.ts"),
        "interface Person { name: string; } const person: Person = { name: 'Ada', age: 37 };",
    )
    .expect("a TypeScript path has a supported source kind");
    let property_start = source
        .text()
        .as_str()
        .find("age:")
        .expect("the object literal contains its extra property");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2353);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Object literal may only specify known properties, and 'age' does not exist in type 'Person'."
    );
    assert_eq!(
        result.diagnostics()[0].span().start(),
        Utf16Offset::new(property_start)
    );
    assert_eq!(result.diagnostics()[0].span().length(), 3);
}

#[test]
fn should_accept_empty_array_given_array_annotation_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(Path::new("values.ts"), "const values: number[] = [];")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
}

#[test]
fn should_resolve_array_type_alias_given_array_initializer_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("values.ts"),
        "type Values = number[]; const values: Values = [1, 2];",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
}

#[test]
fn should_report_array_element_type_error_given_incompatible_array_initializer_when_checking_types()
{
    // Arrange
    let source = SourceFile::from_path(
        Path::new("values.ts"),
        "const values: number[] = [\"wrong\"];",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'string' is not assignable to type 'number'."
    );
    assert_eq!(result.diagnostics()[0].span().start(), Utf16Offset::new(26));
    assert_eq!(result.diagnostics()[0].span().length(), 7);
}

#[test]
fn should_report_array_length_type_error_given_string_annotation_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("values.ts"),
        "const values: number[] = [1]; const size: string = values.length;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'number' is not assignable to type 'string'."
    );
    assert_eq!(result.diagnostics()[0].span().start(), Utf16Offset::new(36));
    assert_eq!(result.diagnostics()[0].span().length(), 4);
}

#[test]
fn should_report_assignability_error_given_boolean_initializer_for_number_annotation_when_checking_types()
 {
    // Arrange
    let source = SourceFile::from_path(Path::new("answer.ts"), "const answer: number = true;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2322)
    );
}

#[test]
fn should_report_assignability_error_given_variable_initializer_for_incompatible_annotation_when_checking_types()
 {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("values.ts"),
        "const answer: number = 42; const label: string = answer;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2322)
    );
}

#[test]
fn should_report_unresolved_identifier_given_missing_variable_initializer_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(Path::new("missing.ts"), "const answer: number = missing;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2304)
    );
}

#[test]
fn should_check_function_call_result_given_incompatible_annotation_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "function answer(): number { return 42; } const label: string = answer();",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'number' is not assignable to type 'string'."
    );
}

#[test]
fn should_report_incompatible_function_argument_given_mismatched_parameter_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "function answer(value: number): number { return value; } const result = answer('wrong');",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2345);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Argument of type 'string' is not assignable to parameter of type 'number'."
    );
}

#[test]
fn should_report_incompatible_argument_given_expression_statement_call_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "function accept(value: number): void { return; } function run(): void { accept('wrong'); }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2345);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Argument of type 'string' is not assignable to parameter of type 'number'."
    );
}

#[test]
fn should_report_incompatible_argument_given_top_level_call_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "function accept(value: number): void { return; } accept('wrong');",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2345);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Argument of type 'string' is not assignable to parameter of type 'number'."
    );
}

#[test]
fn should_report_incompatible_return_given_return_inside_conditional_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "function answer(enabled: boolean): number { if (enabled) { return 'wrong'; } else { return 0; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'string' is not assignable to type 'number'."
    );
}

#[test]
fn should_report_incompatible_operands_given_numeric_comparison_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("positive.ts"),
        "function isPositive(score: number): boolean { return score > \"0\"; }",
    )
    .expect("a TypeScript path has a supported source kind");
    let operator_start = source
        .text()
        .as_str()
        .find('>')
        .expect("the comparison contains a greater-than operator");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2365);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Operator '>' cannot be applied to types 'number' and 'string'."
    );
    assert_eq!(
        result.diagnostics()[0].span().start(),
        Utf16Offset::new(operator_start)
    );
    assert_eq!(result.diagnostics()[0].span().length(), 1);
}

#[test]
fn should_report_missing_required_argument_given_no_call_arguments_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "function answer(value: number): number { return value; } const result = answer();",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2554);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Expected 1 arguments, but got 0."
    );
}

#[test]
fn should_accept_omitted_optional_argument_given_function_call_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "function answer(value?: number): number { return 42; } const result = answer();",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
}

#[test]
fn should_allow_omitted_default_parameter_given_function_call_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "function answer(value: number = 42): number { return value; } const result = answer();",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_report_incompatible_default_parameter_given_string_annotation_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greet.ts"),
        "function greet(name: string = 42): string { return name; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'number' is not assignable to type 'string'."
    );
}

#[test]
fn should_report_incompatible_default_parameter_given_typed_arrow_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greet.ts"),
        "const greet = (name: string = 42) => name;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'number' is not assignable to type 'string'."
    );
}

#[test]
fn should_reject_incompatible_argument_given_unannotated_default_parameter_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greet.ts"),
        "function greet(name = 'world'): string { return name; } greet(42);",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2345);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Argument of type 'number' is not assignable to parameter of type 'string'."
    );
}

#[test]
fn should_reject_incompatible_argument_given_unannotated_arrow_default_parameter_when_checking_types()
 {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greet.ts"),
        "const greet = (name = 'world') => name; greet(42);",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2345);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Argument of type 'number' is not assignable to parameter of type 'string'."
    );
}

#[test]
fn should_report_unresolved_name_given_default_parameter_initializer_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greet.ts"),
        "function greet(name: string = missing): string { return name; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2304);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Cannot find name 'missing'."
    );
}

#[test]
fn should_report_extra_argument_given_call_with_too_many_arguments_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "function answer(value: number): number { return value; } const result = answer(42, 1);",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2554);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Expected 1 arguments, but got 2."
    );
    assert_eq!(result.diagnostics()[0].span().start(), Utf16Offset::new(83));
    assert_eq!(result.diagnostics()[0].span().length(), 1);
}

#[test]
fn should_report_unresolved_named_export_given_missing_local_binding_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(Path::new("exports.ts"), "export { missing };")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2304);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Cannot find name 'missing'."
    );
}

#[test]
fn should_check_interface_property_type_given_incompatible_assignment_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("person.ts"),
        "interface Person { name: string; } const person: Person = { name: 'Ada' }; const count: number = person.name;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'string' is not assignable to type 'number'."
    );
}

#[test]
fn should_report_missing_interface_property_given_unknown_member_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("person.ts"),
        "interface Person { name: string; } const person: Person = { name: 'Ada' }; const age = person.age;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2339);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Property 'age' does not exist on type 'Person'."
    );
}

#[test]
fn should_report_unknown_type_given_unresolved_type_annotation_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(Path::new("answer.ts"), "const answer: Missing = 42;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2304);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Cannot find name 'Missing'."
    );
}

#[test]
fn should_report_unknown_type_given_type_alias_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(Path::new("answer.ts"), "type Answer = Missing;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2304);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Cannot find name 'Missing'."
    );
}

#[test]
fn should_report_unknown_type_given_function_parameter_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "function answer(value: Missing): string { return 'answer'; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2304);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Cannot find name 'Missing'."
    );
}
