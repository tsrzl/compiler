use tsrzl::parser::{
    ParseDiagnostic, ParseOptions, ParsedSourceFile, ResolutionMode, ScriptKind, parse_source_file,
};

fn parse(text: &str) -> ParsedSourceFile {
    parse_source_file(&ParseOptions::new("test.ts", ScriptKind::Ts), text)
}

#[test]
fn should_collect_lib_reference_given_reference_lib_directive_when_parsing() {
    // Arrange
    let text = "/// <reference lib=\"es2015.promise\" />\nlet value = 1;";

    // Act
    let parsed = parse(text);

    // Assert
    let libs: Vec<_> = parsed
        .lib_reference_directives()
        .iter()
        .map(|reference| reference.file_name.as_str())
        .collect();
    assert_eq!(libs, ["es2015.promise"]);
}

#[test]
fn should_locate_reference_value_given_reference_path_directive_when_parsing() {
    // Arrange
    let text = "/// <reference path='./types.d.ts' />";

    // Act
    let parsed = parse(text);

    // Assert
    let reference = &parsed.referenced_files()[0];
    assert_eq!(&text[reference.pos..reference.end], "./types.d.ts");
}

#[test]
fn should_parse_resolution_mode_given_types_directive_when_parsing() {
    // Arrange
    let text = "/// <reference types=\"node\" resolution-mode=\"require\" />";

    // Act
    let parsed = parse(text);

    // Assert
    assert_eq!(
        parsed.type_reference_directives()[0].resolution_mode,
        Some(ResolutionMode::CommonJs)
    );
}

#[test]
fn should_ignore_directive_given_no_default_lib_reference_when_parsing() {
    // Arrange
    let text = "/// <reference no-default-lib=\"true\" />";

    // Act
    let parsed = parse(text);

    // Assert
    assert!(parsed.referenced_files().is_empty() && parsed.diagnostics().is_empty());
}

#[test]
fn should_report_ts1084_given_reference_without_known_attribute_when_parsing() {
    // Arrange
    let text = "/// <reference unknown=\"x\" />";

    // Act
    let parsed = parse(text);

    // Assert
    let texts: Vec<_> = parsed
        .diagnostics()
        .iter()
        .map(ParseDiagnostic::text)
        .collect();
    assert_eq!(texts, ["Invalid 'reference' directive syntax."]);
}

#[test]
fn should_take_last_directive_given_check_and_nocheck_comments_when_parsing() {
    // Arrange
    let text = "// @ts-check\n// @ts-nocheck\nlet value = 1;";

    // Act
    let parsed = parse(text);

    // Assert
    assert_eq!(
        parsed
            .check_js_directive()
            .map(|directive| directive.enabled),
        Some(false)
    );
}

#[test]
fn should_ignore_references_given_directive_after_first_statement_when_parsing() {
    // Arrange
    let text = "let value = 1;\n/// <reference lib=\"dom\" />";

    // Act
    let parsed = parse(text);

    // Assert
    assert_eq!(parsed.lib_reference_directives(), []);
}
