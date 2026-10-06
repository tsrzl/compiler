use tsrzl::ast::SyntaxKind;
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
