use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_erase_type_annotation_given_typed_variable_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(Path::new("answer.ts"), "const answer: number = 42;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(result.emitted_files()[0].text(), "const answer = 42;\n");
}

#[test]
fn should_preserve_negative_decimal_exponent_given_numeric_initializer_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(Path::new("ratio.ts"), "const ratio = 1e-3;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(result.emitted_files()[0].text(), "const ratio = 1e-3;\n");
}

#[test]
fn should_preserve_leading_decimal_point_given_fractional_initializer_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(Path::new("fraction.ts"), "const fraction = .5;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(result.emitted_files()[0].text(), "const fraction = .5;\n");
}

#[test]
fn should_erase_type_assertion_given_numeric_initializer_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(Path::new("answer.ts"), "const answer = 42 as number;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(result.emitted_files()[0].text(), "const answer = 42;\n");
}

#[test]
fn should_erase_satisfies_operator_given_numeric_initializer_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "const answer = 42 satisfies number;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(result.emitted_files()[0].text(), "const answer = 42;\n");
}

#[test]
fn should_preserve_optional_property_access_given_optional_chain_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("person.ts"),
        "const person = { name: \"Ada\" }; const name = person?.name;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "const person = { name: \"Ada\" };\nconst name = person?.name;\n"
    );
}

#[test]
fn should_emit_async_function_given_async_function_declaration_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "async function loadAnswer() {\n    return 42;\n}",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "async function loadAnswer() {\n    return 42;\n}\n"
    );
}

#[test]
fn should_emit_generator_function_given_generator_declaration_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answers.ts"),
        "function* answers() {\n    yield 42;\n}",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function* answers() {\n    yield 42;\n}\n"
    );
}

#[test]
fn should_preserve_regular_expression_literal_given_variable_initializer_when_emitting_javascript()
{
    // Arrange
    let source = SourceFile::from_path(Path::new("pattern.ts"), "const pattern = /answer/i;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert!(
        result.emitted_files()[0]
            .text()
            .contains("const pattern = /answer/i;")
    );
}

#[test]
fn should_emit_private_identifier_given_typed_class_field_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("person.ts"),
        "class Person { #name: string; read(): string { return this.#name; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert!(result.emitted_files()[0].text().contains("#name;"));
    assert!(result.emitted_files()[0].text().contains("this.#name"));
}

#[test]
fn should_preserve_array_destructuring_given_array_initializer_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(Path::new("answer.ts"), "const [answer, rest] = [42, 24];")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert!(
        result.emitted_files()[0]
            .text()
            .contains("[answer, rest] = [42, 24]")
    );
}

#[test]
fn should_preserve_optional_call_given_optional_function_expression_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "function getAnswer() { return 42; } const answer = getAnswer?.();",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert!(result.emitted_files()[0].text().contains("getAnswer?.()"));
}

#[test]
fn should_preserve_object_destructuring_given_object_initializer_when_emitting_javascript() {
    // Arrange
    let source =
        SourceFile::from_path(Path::new("answer.ts"), "const { answer } = { answer: 42 };")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "const { answer } = { answer: 42 };\n"
    );
}

#[test]
fn should_emit_namespace_given_exported_namespace_value_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("utilities.ts"),
        "namespace Utilities { export const answer = 42; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_emit_assignment_given_mutable_typed_variable_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(Path::new("count.ts"), "let count: number = 1; count = 2;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "let count = 1;\ncount = 2;\n"
    );
}

#[test]
fn should_emit_compound_assignment_given_mutable_typed_variable_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(Path::new("count.ts"), "let count: number = 1; count += 2;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "let count = 1;\ncount += 2;\n"
    );
}

#[test]
fn should_emit_property_assignment_given_const_typed_object_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("options.ts"),
        "interface Options { count: number; } const options: Options = { count: 1 }; options.count = 2;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "const options = { count: 1 };\noptions.count = 2;\n"
    );
}

#[test]
fn should_erase_function_return_type_given_typed_function_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "function answer(): number { return 42; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function answer() {\n  return 42;\n}\n"
    );
}

#[test]
fn should_emit_class_method_given_typed_class_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greeter.ts"),
        "class Greeter { greet(name: string): string { return name; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "class Greeter {\n  greet(name) {\n    return name;\n  }\n}\n"
    );
}

#[test]
fn should_emit_public_constructor_parameter_property_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greeter.ts"),
        "class Greeter { constructor(public name: string) {} }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "class Greeter {\n  name;\n  constructor(name) {\n    this.name = name;\n  }\n}\n"
    );
}

#[test]
fn should_initialize_parameter_property_after_super_call_in_derived_constructor() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greeter.ts"),
        "class Base {} class Greeter extends Base { constructor(public name: string) { super(); } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "class Base {\n}\nclass Greeter extends Base {\n  name;\n  constructor(name) {\n    super();\n    this.name = name;\n  }\n}\n"
    );
}

#[test]
fn should_emit_base_class_given_extends_clause_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("animals.ts"),
        "export class Animal {} export class Dog extends Animal {}",
    )
    .expect("a TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2025).with_module(ModuleKind::EsNext);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "export class Animal {\n}\nexport class Dog extends Animal {\n}\n"
    );
}

#[test]
fn should_erase_class_property_type_given_initialized_typed_property_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greeter.ts"),
        "class Greeter { name: string = \"world\"; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "class Greeter {\n  name = \"world\";\n}\n"
    );
}

#[test]
fn should_erase_class_property_modifiers_given_private_readonly_field_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("secret.ts"),
        "class Secret { private readonly token: string = \"x\"; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "class Secret {\n  token = \"x\";\n}\n"
    );
}

#[test]
fn should_emit_static_class_property_given_initialized_field_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("counter.ts"),
        "class Counter { static count: number = 0; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "class Counter {\n  static count = 0;\n}\n"
    );
}

#[test]
fn should_emit_static_class_method_given_typed_method_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("counter.ts"),
        "class Counter { static current(): number { return 0; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "class Counter {\n  static current() {\n    return 0;\n  }\n}\n"
    );
}

#[test]
fn should_emit_numeric_enum_given_auto_incremented_members_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("direction.ts"),
        "export enum Direction { Up, Down }",
    )
    .expect("a TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2025).with_module(ModuleKind::EsNext);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "export var Direction;\n(function (Direction) {\n    Direction[Direction[\"Up\"] = 0] = \"Up\";\n    Direction[Direction[\"Down\"] = 1] = \"Down\";\n})(Direction || (Direction = {}));\n"
    );
}

#[test]
fn should_emit_string_enum_given_string_initialized_members_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("direction.ts"),
        "export enum Direction { Up = \"UP\", Down = \"DOWN\" }",
    )
    .expect("a TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2025).with_module(ModuleKind::EsNext);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "export var Direction;\n(function (Direction) {\n    Direction[\"Up\"] = \"UP\";\n    Direction[\"Down\"] = \"DOWN\";\n})(Direction || (Direction = {}));\n"
    );
}

#[test]
fn should_fold_numeric_enum_constant_given_arithmetic_member_initializer_when_emitting_javascript()
{
    // Arrange
    let source = SourceFile::from_path(
        Path::new("count.ts"),
        "export enum Count { First = 2 + 3, Second }",
    )
    .expect("a TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2025).with_module(ModuleKind::EsNext);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "export var Count;\n(function (Count) {\n    Count[Count[\"First\"] = 5] = \"First\";\n    Count[Count[\"Second\"] = 6] = \"Second\";\n})(Count || (Count = {}));\n"
    );
}

#[test]
fn should_fold_numeric_enum_remainder_given_member_initializer_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("count.ts"),
        "export enum Count { Remainder = 5 % 2, Next }",
    )
    .expect("a TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2025).with_module(ModuleKind::EsNext);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "export var Count;\n(function (Count) {\n    Count[Count[\"Remainder\"] = 1] = \"Remainder\";\n    Count[Count[\"Next\"] = 2] = \"Next\";\n})(Count || (Count = {}));\n"
    );
}

#[test]
fn should_emit_class_construction_given_class_declaration_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greeter.ts"),
        "class Greeter {} const greeter = new Greeter();",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "class Greeter {\n}\nconst greeter = new Greeter();\n"
    );
}

#[test]
fn should_preserve_default_parameter_given_typed_function_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greet.ts"),
        "function greet(name: string = 'world'): string { return name; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function greet(name = 'world') {\n  return name;\n}\n"
    );
}

#[test]
fn should_emit_if_else_statement_given_typed_function_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "function answer(enabled: boolean): number { if (enabled) { return 1; } else { return 0; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function answer(enabled) {\n  if (enabled) {\n    return 1;\n  } else {\n    return 0;\n  }\n}\n"
    );
}

#[test]
fn should_emit_switch_clauses_given_typed_function_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("label.ts"),
        "function label(code: number): string { switch (code) { case 1: return 'one'; default: return 'other'; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function label(code) {\n  switch (code) {\n    case 1:\n      return 'one';\n    default:\n      return 'other';\n  }\n}\n"
    );
}

#[test]
fn should_emit_try_catch_finally_given_typed_function_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("execute.ts"),
        "function run(): void {} function recover(error: any): void {} function close(): void {} function execute(): void { try { run(); } catch (error: any) { recover(error); } finally { close(); } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function run() {\n}\nfunction recover(error) {\n}\nfunction close() {\n}\nfunction execute() {\n  try {\n    run();\n  } catch (error) {\n    recover(error);\n  } finally {\n    close();\n  }\n}\n"
    );
}

#[test]
fn should_emit_throw_given_function_with_typed_parameter_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("fail.ts"),
        "function fail(reason: string): void { throw reason; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function fail(reason) {\n  throw reason;\n}\n"
    );
}

#[test]
fn should_emit_typed_arrow_function_given_expression_body_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("double.ts"),
        "const double = (value: number): number => value * 2;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "const double = (value) => value * 2;\n"
    );
}

#[test]
fn should_preserve_default_parameter_given_typed_arrow_function_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greet.ts"),
        "const greet = (name: string = 'world') => name;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "const greet = (name = 'world') => name;\n"
    );
}

#[test]
fn should_emit_unparenthesized_arrow_parameter_given_single_parameter_when_compiling() {
    // Arrange
    let source =
        SourceFile::from_path(Path::new("double.ts"), "const double = value => value * 2;")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "const double = value => value * 2;\n"
    );
}

#[test]
fn should_emit_typed_arrow_function_given_block_body_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("double.ts"),
        "const double = (value: number): number => { return value * 2; };",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "const double = (value) => {\n  return value * 2;\n};\n"
    );
}

#[test]
fn should_emit_top_level_if_statement_given_mutable_boolean_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("enabled.ts"),
        "let enabled: boolean = true; if (enabled) { enabled = false; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "let enabled = true;\nif (enabled) {\n  enabled = false;\n}\n"
    );
}

#[test]
fn should_emit_top_level_for_loop_given_mutable_initializer_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("loop.ts"),
        "for (let index = 0; index < 1; index += 1) { break; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "for (let index = 0; index < 1; index += 1) {\n  break;\n}\n"
    );
}

#[test]
fn should_emit_unbraced_if_else_statement_given_typed_function_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "function answer(enabled: boolean): number { if (enabled) return 1; else return 0; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function answer(enabled) {\n  if (enabled) {\n    return 1;\n  } else {\n    return 0;\n  }\n}\n"
    );
}

#[test]
fn should_emit_expression_statement_given_typed_function_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("notify.ts"),
        "function notify(): void { notify(); }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function notify() {\n  notify();\n}\n"
    );
}

#[test]
fn should_emit_top_level_call_given_function_declaration_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("notify.ts"),
        "function notify(): void { return; } notify();",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function notify() {\n  return;\n}\nnotify();\n"
    );
}

#[test]
fn should_emit_numeric_comparison_given_typed_function_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("positive.ts"),
        "function isPositive(score: number): boolean { return score > 0; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function isPositive(score) {\n  return score > 0;\n}\n"
    );
}

#[test]
fn should_emit_less_than_comparison_given_typed_function_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("within-limit.ts"),
        "function isWithinLimit(value: number): boolean { return value < 10; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function isWithinLimit(value) {\n  return value < 10;\n}\n"
    );
}

#[test]
fn should_emit_strict_equality_given_typed_function_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("same.ts"),
        "function same(left: number, right: number): boolean { return left === right; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function same(left, right) {\n  return left === right;\n}\n"
    );
}

#[test]
fn should_emit_strict_inequality_given_typed_function_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("different.ts"),
        "function differs(left: string, right: string): boolean { return left !== right; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function differs(left, right) {\n  return left !== right;\n}\n"
    );
}

#[test]
fn should_emit_logical_negation_given_boolean_parameter_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("enabled.ts"),
        "function isDisabled(enabled: boolean): boolean { return !enabled; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function isDisabled(enabled) {\n  return !enabled;\n}\n"
    );
}

#[test]
fn should_emit_logical_conjunction_given_boolean_parameters_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("ready.ts"),
        "function areBothReady(first: boolean, second: boolean): boolean { return first && second; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function areBothReady(first, second) {\n  return first && second;\n}\n"
    );
}

#[test]
fn should_emit_logical_disjunction_given_boolean_parameters_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("ready.ts"),
        "function isReady(first: boolean, second: boolean): boolean { return first || second; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function isReady(first, second) {\n  return first || second;\n}\n"
    );
}

#[test]
fn should_emit_nullish_coalescing_given_optional_string_parameter_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("label.ts"),
        "function label(value: string | undefined): string { return value ?? 'fallback'; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function label(value) {\n  return value ?? 'fallback';\n}\n"
    );
}

#[test]
fn should_allow_parenthesized_nullish_expression_given_logical_disjunction_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("label.ts"),
        "function label(value: string | undefined, fallback: string): string { return (value ?? fallback) || 'empty'; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function label(value, fallback) {\n  return (value ?? fallback) || 'empty';\n}\n"
    );
}

#[test]
fn should_emit_conditional_expression_given_boolean_parameter_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("label.ts"),
        "function label(enabled: boolean): string { return enabled ? 'on' : 'off'; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function label(enabled) {\n  return enabled ? 'on' : 'off';\n}\n"
    );
}

#[test]
fn should_emit_while_loop_given_boolean_parameter_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("process.ts"),
        "function process(ready: boolean): void { while (ready) { ready = false; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function process(ready) {\n  while (ready) {\n    ready = false;\n  }\n}\n"
    );
}

#[test]
fn should_emit_do_while_loop_given_boolean_parameter_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("process.ts"),
        "function process(ready: boolean): void { do { ready = false; } while (ready); }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function process(ready) {\n  do {\n    ready = false;\n  } while (ready);\n}\n"
    );
}

#[test]
fn should_emit_for_loop_given_typed_function_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("count.ts"),
        "function count(limit: number): void { for (let index = 0; index < limit; index += 1) { continue; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function count(limit) {\n  for (let index = 0; index < limit; index += 1) {\n    continue;\n  }\n}\n"
    );
}

#[test]
fn should_emit_multiple_for_initializers_given_typed_function_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("count.ts"),
        "function count(limit: number): void { for (let index = 0, total = 0; index < limit; index += 1) { total += index; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function count(limit) {\n  for (let index = 0, total = 0; index < limit; index += 1) {\n    total += index;\n  }\n}\n"
    );
}

#[test]
fn should_infer_for_of_binding_given_array_element_type_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("sum.ts"),
        "function sum(values: number[]): number { let total = 0; for (const value of values) { total += value; } return total; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function sum(values) {\n  let total = 0;\n  for (const value of values) {\n    total += value;\n  }\n  return total;\n}\n"
    );
}

#[test]
fn should_emit_for_in_loop_given_typed_function_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("keys.ts"),
        "function copyKeys(options: any): void { for (const key in options) { const copied: string = key; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function copyKeys(options) {\n  for (const key in options) {\n    const copied = key;\n  }\n}\n"
    );
}

#[test]
fn should_emit_empty_for_clauses_given_unconditional_loop_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("spin.ts"),
        "function spin(): void { for (;;) { break; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function spin() {\n  for (;;) {\n    break;\n  }\n}\n"
    );
}

#[test]
fn should_emit_break_statement_given_while_body_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("process.ts"),
        "function process(): void { while (true) { break; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function process() {\n  while (true) {\n    break;\n  }\n}\n"
    );
}

#[test]
fn should_emit_continue_statement_given_while_body_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("process.ts"),
        "function process(): void { while (true) { continue; } }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function process() {\n  while (true) {\n    continue;\n  }\n}\n"
    );
}

#[test]
fn should_emit_less_than_or_equal_comparison_given_typed_function_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("within-limit.ts"),
        "function isWithinLimit(value: number): boolean { return value <= 10; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function isWithinLimit(value) {\n  return value <= 10;\n}\n"
    );
}

#[test]
fn should_emit_local_variable_given_function_body_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "function answer(): number { const result = 42; return result; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function answer() {\n  const result = 42;\n  return result;\n}\n"
    );
}

#[test]
fn should_emit_each_local_variable_given_comma_separated_declarations_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("total.ts"),
        "function total(): number { const left = 20, right = 22; return left + right; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function total() {\n  const left = 20;\n  const right = 22;\n  return left + right;\n}\n"
    );
}

#[test]
fn should_emit_typed_array_given_array_literal_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(Path::new("values.ts"), "const values: number[] = [1, 2];")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(result.emitted_files()[0].text(), "const values = [1, 2];\n");
}

#[test]
fn should_emit_null_literal_given_null_typed_variable_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(Path::new("value.ts"), "const value: null = null;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(result.emitted_files()[0].text(), "const value = null;\n");
}

#[test]
fn should_emit_undefined_global_given_undefined_typed_variable_when_compiling() {
    // Arrange
    let source =
        SourceFile::from_path(Path::new("value.ts"), "const value: undefined = undefined;")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "const value = undefined;\n"
    );
}

#[test]
fn should_emit_array_element_access_given_typed_array_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("values.ts"),
        "const values: number[] = [1, 2]; const first: number = values[0];",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "const values = [1, 2];\nconst first = values[0];\n"
    );
}

#[test]
fn should_preserve_addition_given_numeric_initializer_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(Path::new("total.ts"), "const total: number = 1 + 2;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(result.emitted_files()[0].text(), "const total = 1 + 2;\n");
}

#[test]
fn should_emit_unary_negation_given_negative_initializer_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(Path::new("offset.ts"), "const offset: number = -1;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(result.emitted_files()[0].text(), "const offset = -1;\n");
}

#[test]
fn should_preserve_parentheses_given_grouped_initializer_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(Path::new("total.ts"), "const total: number = (1 + 2) * 3;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "const total = (1 + 2) * 3;\n"
    );
}

#[test]
fn should_emit_function_call_given_typed_call_initializer_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "function compute(left: number, right: number): number { return left + right; } const answer: number = compute(40, 2);",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "function compute(left, right) {\n  return left + right;\n}\nconst answer = compute(40, 2);\n"
    );
}

#[test]
fn should_preserve_export_keyword_given_typescript_seven_default_module_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(Path::new("answer.ts"), "export const answer: number = 42;")
        .expect("a TypeScript path has a supported source kind");
    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "export const answer = 42;\n"
    );
}

#[test]
fn should_preserve_const_given_typescript_seven_default_target_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(Path::new("answer.ts"), "const answer: number = 42;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.emitted_files()[0].text(), "const answer = 42;\n");
}

#[test]
fn should_emit_commonjs_export_given_commonjs_module_option_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(Path::new("answer.ts"), "export const answer: number = 42;")
        .expect("a TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2020).with_module(ModuleKind::CommonJs);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    assert_eq!(
        result.emitted_files()[0].text(),
        "\"use strict\";\nObject.defineProperty(exports, \"__esModule\", { value: true });\nexports.answer = void 0;\nexports.answer = 42;\n"
    );
}

#[test]
fn should_export_commonjs_function_given_commonjs_module_option_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "export function answer(): number { return 42; }",
    )
    .expect("a TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2020).with_module(ModuleKind::CommonJs);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    assert_eq!(
        result.emitted_files()[0].text(),
        "\"use strict\";\nObject.defineProperty(exports, \"__esModule\", { value: true });\nexports.answer = answer;\nfunction answer() {\n  return 42;\n}\n"
    );
}

#[test]
fn should_preserve_side_effect_import_given_ecmascript_module_when_emitting_javascript() {
    // Arrange
    let importing_source = SourceFile::from_path(Path::new("main.ts"), "import './polyfill';")
        .expect("a TypeScript path has a supported source kind");
    let imported_source =
        SourceFile::from_path(Path::new("polyfill.ts"), "const ready: boolean = true;")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([importing_source, imported_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(result.emitted_files()[0].text(), "import './polyfill';\n");
}

#[test]
fn should_emit_require_given_side_effect_import_and_commonjs_module_when_emitting_javascript() {
    // Arrange
    let importing_source = SourceFile::from_path(Path::new("main.ts"), "import './polyfill';")
        .expect("a TypeScript path has a supported source kind");
    let imported_source =
        SourceFile::from_path(Path::new("polyfill.ts"), "const ready: boolean = true;")
            .expect("a TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2020).with_module(ModuleKind::CommonJs);

    // Act
    let result =
        Compiler::with_options(options).compile_sources([importing_source, imported_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "\"use strict\";\nObject.defineProperty(exports, \"__esModule\", { value: true });\nrequire(\"./polyfill\");\n"
    );
}

#[test]
fn should_emit_default_export_given_ecmascript_module_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(Path::new("answer.ts"), "export default 42;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(result.emitted_files()[0].text(), "export default 42;\n");
}

#[test]
fn should_emit_commonjs_default_export_given_commonjs_module_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(Path::new("answer.ts"), "export default 42;")
        .expect("a TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2020).with_module(ModuleKind::CommonJs);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "\"use strict\";\nObject.defineProperty(exports, \"__esModule\", { value: true });\nexports.default = 42;\n"
    );
}

#[test]
fn should_emit_named_export_given_ecmascript_module_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "const answer: number = 42; export { answer };",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "const answer = 42;\nexport { answer };\n"
    );
}

#[test]
fn should_emit_commonjs_named_export_given_commonjs_module_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "const answer: number = 42; export { answer };",
    )
    .expect("a TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2020).with_module(ModuleKind::CommonJs);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "\"use strict\";\nObject.defineProperty(exports, \"__esModule\", { value: true });\nexports.answer = void 0;\nconst answer = 42;\nexports.answer = answer;\n"
    );
}

#[test]
fn should_emit_property_access_given_interface_member_initializer_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("person.ts"),
        "interface Person { name: string; } const person: Person = { name: 'Ada' }; const name: string = person.name;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "const person = { name: 'Ada' };\nconst name = person.name;\n"
    );
}

#[test]
fn should_emit_commonjs_named_function_export_given_commonjs_module_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "function answer(): number { return 42; } export { answer };",
    )
    .expect("a TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2020).with_module(ModuleKind::CommonJs);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "\"use strict\";\nObject.defineProperty(exports, \"__esModule\", { value: true });\nexports.answer = answer;\nfunction answer() {\n  return 42;\n}\n"
    );
}

#[test]
fn should_emit_aliased_named_export_given_ecmascript_module_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "const answer: number = 42; export { answer as result };",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "const answer = 42;\nexport { answer as result };\n"
    );
}

#[test]
fn should_preserve_aliased_named_import_given_ecmascript_module_when_emitting_javascript() {
    // Arrange
    let importing_source = SourceFile::from_path(
        Path::new("main.ts"),
        "import { answer as value } from './answer';",
    )
    .expect("a TypeScript path has a supported source kind");
    let imported_source =
        SourceFile::from_path(Path::new("answer.ts"), "export const answer: number = 42;")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([importing_source, imported_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "import { answer as value } from './answer';\n"
    );
}

#[test]
fn should_emit_commonjs_named_import_given_commonjs_module_when_emitting_javascript() {
    // Arrange
    let importing_source = SourceFile::from_path(
        Path::new("main.ts"),
        "import { answer as value } from './answer'; export const result: number = value + 1;",
    )
    .expect("a TypeScript path has a supported source kind");
    let imported_source =
        SourceFile::from_path(Path::new("answer.ts"), "export const answer: number = 42;")
            .expect("a TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2020).with_module(ModuleKind::CommonJs);

    // Act
    let result =
        Compiler::with_options(options).compile_sources([importing_source, imported_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "\"use strict\";\nObject.defineProperty(exports, \"__esModule\", { value: true });\nexports.result = void 0;\nconst answer_1 = require(\"./answer\");\nexports.result = answer_1.answer + 1;\n"
    );
}

#[test]
fn should_preserve_default_import_given_ecmascript_module_when_emitting_javascript() {
    // Arrange
    let importing_source =
        SourceFile::from_path(Path::new("main.ts"), "import answer from './answer';")
            .expect("a TypeScript path has a supported source kind");
    let imported_source = SourceFile::from_path(Path::new("answer.ts"), "export default 42;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([importing_source, imported_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "import answer from './answer';\n"
    );
}

#[test]
fn should_emit_commonjs_default_import_given_commonjs_module_when_emitting_javascript() {
    // Arrange
    let importing_source = SourceFile::from_path(
        Path::new("main.ts"),
        "import answer from './answer'; export const result: number = answer + 1;",
    )
    .expect("a TypeScript path has a supported source kind");
    let imported_source = SourceFile::from_path(Path::new("answer.ts"), "export default 42;")
        .expect("a TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2020).with_module(ModuleKind::CommonJs);

    // Act
    let result =
        Compiler::with_options(options).compile_sources([importing_source, imported_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "\"use strict\";\nvar __importDefault = (this && this.__importDefault) || function (mod) {\n    return (mod && mod.__esModule) ? mod : { \"default\": mod };\n};\nObject.defineProperty(exports, \"__esModule\", { value: true });\nexports.result = void 0;\nconst answer_1 = __importDefault(require(\"./answer\"));\nexports.result = answer_1.default + 1;\n"
    );
}

#[test]
fn should_erase_type_only_import_given_exported_type_when_emitting_javascript() {
    // Arrange
    let importing_source = SourceFile::from_path(
        Path::new("main.ts"),
        "import type { Answer } from './answer'; const answer: Answer = 42;",
    )
    .expect("a TypeScript path has a supported source kind");
    let imported_source =
        SourceFile::from_path(Path::new("answer.ts"), "export type Answer = number;")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([importing_source, imported_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(result.emitted_files()[0].text(), "const answer = 42;\n");
}

#[test]
fn should_preserve_namespace_import_given_ecmascript_module_when_emitting_javascript() {
    // Arrange
    let importing_source = SourceFile::from_path(
        Path::new("main.ts"),
        "import * as answerModule from './answer'; export const result: number = answerModule.answer + 1;",
    )
    .expect("a TypeScript path has a supported source kind");
    let imported_source =
        SourceFile::from_path(Path::new("answer.ts"), "export const answer: number = 42;")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([importing_source, imported_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "import * as answerModule from './answer';\nexport const result = answerModule.answer + 1;\n"
    );
}

#[test]
fn should_lower_namespace_import_given_commonjs_module_when_emitting_javascript() {
    // Arrange
    let importing_source = SourceFile::from_path(
        Path::new("main.ts"),
        "import * as answerModule from './answer'; export const result: number = answerModule.answer + 1;",
    )
    .expect("a TypeScript path has a supported source kind");
    let imported_source =
        SourceFile::from_path(Path::new("answer.ts"), "export const answer: number = 42;")
            .expect("a TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2020).with_module(ModuleKind::CommonJs);

    // Act
    let result =
        Compiler::with_options(options).compile_sources([importing_source, imported_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        include_str!("fixtures/namespace-import-commonjs.js")
    );
}

#[test]
fn should_preserve_export_all_given_ecmascript_module_when_emitting_javascript() {
    // Arrange
    let barrel_source = SourceFile::from_path(Path::new("barrel.ts"), "export * from './answer';")
        .expect("a TypeScript path has a supported source kind");
    let exported_source =
        SourceFile::from_path(Path::new("answer.ts"), "export const answer: number = 42;")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([barrel_source, exported_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "export * from './answer';\n"
    );
}

#[test]
fn should_preserve_named_reexport_given_ecmascript_module_when_emitting_javascript() {
    // Arrange
    let barrel_source = SourceFile::from_path(
        Path::new("barrel.ts"),
        "export { answer as renamed } from './answer';",
    )
    .expect("a TypeScript path has a supported source kind");
    let exported_source =
        SourceFile::from_path(Path::new("answer.ts"), "export const answer: number = 42;")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([barrel_source, exported_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "export { answer as renamed } from './answer';\n"
    );
}

#[test]
fn should_lower_export_all_given_commonjs_module_when_emitting_javascript() {
    // Arrange
    let barrel_source = SourceFile::from_path(Path::new("barrel.ts"), "export * from './answer';")
        .expect("a TypeScript path has a supported source kind");
    let exported_source =
        SourceFile::from_path(Path::new("answer.ts"), "export const answer: number = 42;")
            .expect("a TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2020).with_module(ModuleKind::CommonJs);

    // Act
    let result = Compiler::with_options(options).compile_sources([barrel_source, exported_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        include_str!("fixtures/export-all-commonjs.js")
    );
}

#[test]
fn should_emit_live_named_reexport_given_commonjs_module_when_emitting_javascript() {
    // Arrange
    let barrel_source = SourceFile::from_path(
        Path::new("barrel.ts"),
        "export { answer as renamed } from './answer';",
    )
    .expect("a TypeScript path has a supported source kind");
    let exported_source =
        SourceFile::from_path(Path::new("answer.ts"), "export const answer: number = 42;")
            .expect("a TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2020).with_module(ModuleKind::CommonJs);

    // Act
    let result = Compiler::with_options(options).compile_sources([barrel_source, exported_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(
        result.emitted_files()[0].text(),
        "\"use strict\";\nObject.defineProperty(exports, \"__esModule\", { value: true });\nexports.renamed = void 0;\nvar answer_1 = require(\"./answer\");\nObject.defineProperty(exports, \"renamed\", { enumerable: true, get: function () { return answer_1.answer; } });\n"
    );
}

#[test]
fn should_erase_type_only_named_reexport_given_ecmascript_module_when_emitting_javascript() {
    // Arrange
    let barrel_source = SourceFile::from_path(
        Path::new("barrel.ts"),
        "export type { Answer as Renamed } from './answer';",
    )
    .expect("a TypeScript path has a supported source kind");
    let exported_source =
        SourceFile::from_path(Path::new("answer.ts"), "export type Answer = string;")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([barrel_source, exported_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 0);
    assert_eq!(result.emitted_files()[0].text(), "");
}
