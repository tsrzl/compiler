use tsrzl::ast::SyntaxKind;
use tsrzl::parser::{
    ExternalModuleIndicatorOptions, ParseOptions, ParsedSourceFile, ScriptKind, parse_source_file,
};

fn parse(file_name: &str, script_kind: ScriptKind, text: &str) -> ParsedSourceFile {
    parse_source_file(&ParseOptions::new(file_name, script_kind), text)
}

#[test]
fn should_indicate_exported_statement_given_export_modifier_when_detecting_external_module() {
    // Arrange
    let parsed = parse(
        "test.ts",
        ScriptKind::Ts,
        "let a = 1;\nexport function f() {}",
    );

    // Act
    let indicator = parsed.external_module_indicator(ExternalModuleIndicatorOptions::default());

    // Assert
    let kind = indicator.map(|node| parsed.ast().node(node).kind());
    assert_eq!(kind, Some(SyntaxKind::FunctionDeclaration));
}

#[test]
fn should_not_indicate_module_given_script_statements_when_detecting_external_module() {
    // Arrange
    let parsed = parse("test.ts", ScriptKind::Ts, "let a = 1;\nfunction f() {}");

    // Act
    let indicator = parsed.external_module_indicator(ExternalModuleIndicatorOptions::default());

    // Assert
    assert_eq!(indicator, None);
}

#[test]
fn should_indicate_import_declaration_given_import_when_detecting_external_module() {
    // Arrange
    let parsed = parse("test.ts", ScriptKind::Ts, "import { a } from \"./a\";");

    // Act
    let indicator = parsed.external_module_indicator(ExternalModuleIndicatorOptions::default());

    // Assert
    let kind = indicator.map(|node| parsed.ast().node(node).kind());
    assert_eq!(kind, Some(SyntaxKind::ImportDeclaration));
}

#[test]
fn should_indicate_import_meta_given_meta_property_when_detecting_external_module() {
    // Arrange
    let parsed = parse("test.ts", ScriptKind::Ts, "const url = import.meta.url;");

    // Act
    let indicator = parsed.external_module_indicator(ExternalModuleIndicatorOptions::default());

    // Assert
    let kind = indicator.map(|node| parsed.ast().node(node).kind());
    assert_eq!(kind, Some(SyntaxKind::MetaProperty));
}

#[test]
fn should_indicate_source_file_given_forced_detection_when_detecting_external_module() {
    // Arrange
    let parsed = parse("test.ts", ScriptKind::Ts, "let a = 1;");
    let options = ExternalModuleIndicatorOptions {
        force: true,
        ..ExternalModuleIndicatorOptions::default()
    };

    // Act
    let indicator = parsed.external_module_indicator(options);

    // Assert
    assert_eq!(indicator, Some(parsed.ast().root()));
}

#[test]
fn should_indicate_jsx_element_given_jsx_detection_when_detecting_external_module() {
    // Arrange
    let parsed = parse("test.tsx", ScriptKind::Tsx, "const view = <div />;");
    let options = ExternalModuleIndicatorOptions {
        jsx: true,
        ..ExternalModuleIndicatorOptions::default()
    };

    // Act
    let indicator = parsed.external_module_indicator(options);

    // Assert
    let kind = indicator.map(|node| parsed.ast().node(node).kind());
    assert_eq!(kind, Some(SyntaxKind::JsxSelfClosingElement));
}

#[test]
fn should_not_indicate_module_given_json_file_when_detecting_external_module() {
    // Arrange
    let parsed = parse("data.json", ScriptKind::Json, "{ \"a\": 1 }");
    let options = ExternalModuleIndicatorOptions {
        force: true,
        ..ExternalModuleIndicatorOptions::default()
    };

    // Act
    let indicator = parsed.external_module_indicator(options);

    // Assert
    assert_eq!(indicator, None);
}
