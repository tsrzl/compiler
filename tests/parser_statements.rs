use tsrzl::ast::{NodeFlags, SyntaxKind};
use tsrzl::parser::{ParseOptions, ScriptKind, parse_source_file};

fn statement_kinds(text: &str) -> Vec<SyntaxKind> {
    let parsed = parse_source_file(&ParseOptions::new("test.ts", ScriptKind::Ts), text);
    let ast = parsed.ast();
    let source_file = ast
        .node(ast.root())
        .data()
        .as_source_file()
        .expect("the root is a source file");
    ast.list(source_file.statements)
        .iter()
        .map(|&statement| ast.node(statement).kind())
        .collect()
}

#[test]
fn should_parse_empty_statements_given_semicolons_when_parsing_source_file() {
    // Arrange
    let text = ";\n;";

    // Act
    let actual = statement_kinds(text);

    // Assert
    assert_eq!(
        actual,
        [SyntaxKind::EmptyStatement, SyntaxKind::EmptyStatement]
    );
}

#[test]
fn should_flag_possible_dynamic_import_on_source_file_given_import_call_when_parsing_source_file() {
    // Arrange
    let text = r#"import("m");"#;

    // Act
    let parsed = parse_source_file(&ParseOptions::new("test.ts", ScriptKind::Ts), text);

    // Assert
    let flags = parsed.ast().node(parsed.ast().root()).flags();
    assert!(flags.intersects(NodeFlags::POSSIBLY_CONTAINS_DYNAMIC_IMPORT));
}

#[test]
fn should_keep_source_text_given_parsed_file_when_parsing_source_file() {
    // Arrange
    let text = "let value = 1;";

    // Act
    let parsed = parse_source_file(&ParseOptions::new("test.ts", ScriptKind::Ts), text);

    // Assert
    assert_eq!(parsed.text(), text);
}
