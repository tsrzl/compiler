use tsrzl::bind::{BoundFile, bind_source_file};
use tsrzl::diagnostics::Diagnostic;
use tsrzl::parser::{
    ExternalModuleIndicatorOptions, ParseOptions, ParsedSourceFile, ScriptKind, parse_source_file,
};

fn parse(text: &str) -> ParsedSourceFile {
    parse_source_file(&ParseOptions::new("test.ts", ScriptKind::Ts), text)
}

fn bind(parsed: &ParsedSourceFile) -> BoundFile {
    bind_source_file(parsed, ExternalModuleIndicatorOptions::default())
}

fn texts(text: &str) -> Vec<String> {
    let parsed = parse(text);
    bind(&parsed)
        .diagnostics()
        .iter()
        .map(Diagnostic::text)
        .collect()
}

#[test]
fn should_report_ts1212_given_reserved_word_variable_in_script_when_binding() {
    // Arrange
    let text = "let implements = 1;";

    // Act
    let diagnostics = texts(text);

    // Assert
    assert_eq!(
        diagnostics,
        ["Identifier expected. 'implements' is a reserved word in strict mode."]
    );
}

#[test]
fn should_report_ts1214_given_reserved_word_variable_in_module_when_binding() {
    // Arrange
    let text = "export {};\nlet implements = 1;";

    // Act
    let diagnostics = texts(text);

    // Assert
    assert_eq!(
        diagnostics,
        [
            "Identifier expected. 'implements' is a reserved word in strict mode. Modules are automatically in strict mode."
        ]
    );
}

#[test]
fn should_report_ts1213_given_reserved_word_parameter_in_class_when_binding() {
    // Arrange
    let text = "class C { run(package: number) {} }";

    // Act
    let diagnostics = texts(text);

    // Assert
    assert_eq!(
        diagnostics,
        [
            "Identifier expected. 'package' is a reserved word in strict mode. Class definitions are automatically in strict mode."
        ]
    );
}

#[test]
fn should_not_report_reserved_word_given_property_name_when_binding() {
    // Arrange
    let text = "const value = { implements: 1 };\nvalue.implements;";

    // Act
    let diagnostics = texts(text);

    // Assert
    assert_eq!(diagnostics, Vec::<String>::new());
}

#[test]
fn should_report_ts1100_given_function_named_eval_when_binding() {
    // Arrange
    let text = "function eval() {}";

    // Act
    let diagnostics = texts(text);

    // Assert
    assert_eq!(diagnostics, ["Invalid use of 'eval' in strict mode."]);
}

#[test]
fn should_report_ts1215_given_assignment_to_arguments_in_module_when_binding() {
    // Arrange
    let text = "export {};\nfunction run() { arguments = []; }";

    // Act
    let diagnostics = texts(text);

    // Assert
    assert_eq!(
        diagnostics,
        ["Invalid use of 'arguments'. Modules are automatically in strict mode."]
    );
}

#[test]
fn should_report_ts1100_given_eval_parameter_when_binding() {
    // Arrange
    let text = "function run(eval: number) {}";

    // Act
    let diagnostics = texts(text);

    // Assert
    assert_eq!(diagnostics, ["Invalid use of 'eval' in strict mode."]);
}

#[test]
fn should_report_ts1102_given_delete_of_identifier_when_binding() {
    // Arrange
    let text = "let value = 1;\ndelete value;";

    // Act
    let diagnostics = texts(text);

    // Assert
    assert_eq!(
        diagnostics,
        ["'delete' cannot be called on an identifier in strict mode."]
    );
}

#[test]
fn should_report_ts1101_given_with_statement_when_binding() {
    // Arrange
    let text = "declare const scope: object;\nwith (scope) {}";

    // Act
    let diagnostics = texts(text);

    // Assert
    assert_eq!(
        diagnostics,
        ["'with' statements are not allowed in strict mode."]
    );
}

#[test]
fn should_report_ts1344_given_labeled_function_declaration_when_binding() {
    // Arrange
    let text = "target: function run() {}";

    // Act
    let diagnostics = texts(text);

    // Assert
    assert_eq!(diagnostics, ["A label is not allowed here."]);
}

#[test]
fn should_report_ts18012_given_private_constructor_name_when_binding() {
    // Arrange
    let text = "class C { #constructor = 1; }";

    // Act
    let diagnostics = texts(text);

    // Assert
    assert_eq!(diagnostics, ["'#constructor' is a reserved word."]);
}

#[test]
fn should_skip_contextual_identifier_check_given_parse_errors_when_binding() {
    // Arrange
    let text = "let implements = ;";

    // Act
    let diagnostics = texts(text);

    // Assert
    assert_eq!(diagnostics, Vec::<String>::new());
}

#[test]
fn should_report_ts1262_given_exported_function_named_await_when_binding() {
    // Pinned TypeScript 7.0.2 case: compiler/exportDefaultAsyncFunction2.ts.
    // Arrange
    let text = "export function await(...args: any[]): any { }";

    // Act
    let diagnostics = texts(text);

    // Assert
    assert_eq!(
        diagnostics,
        ["Identifier expected. 'await' is a reserved word at the top-level of a module."]
    );
}
