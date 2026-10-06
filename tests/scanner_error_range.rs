use tsrzl::ast::{Ast, NodeId, SyntaxKind};
use tsrzl::parser::{ParseOptions, ParsedSourceFile, ScriptKind, parse_source_file};
use tsrzl::scanner::{LanguageVariant, error_range_for_node, range_of_token_at_position};

fn parse(text: &str) -> ParsedSourceFile {
    parse_source_file(&ParseOptions::new("test.ts", ScriptKind::Ts), text)
}

fn find(ast: &Ast, id: NodeId, kind: SyntaxKind) -> Option<NodeId> {
    if ast.node(id).kind() == kind {
        return Some(id);
    }
    ast.children(id)
        .into_iter()
        .find_map(|child| find(ast, child, kind))
}

fn error_text(parsed: &ParsedSourceFile, kind: SyntaxKind) -> &str {
    let ast = parsed.ast();
    let node = find(ast, ast.root(), kind).expect("the source contains the node kind");
    let range = error_range_for_node(ast, parsed.text(), parsed.language_variant(), node);
    &parsed.text()[range]
}

#[test]
fn should_span_next_token_given_leading_trivia_when_reading_token_range() {
    // Arrange
    let text = "  /* note */ value;";

    // Act
    let range = range_of_token_at_position(text, LanguageVariant::Standard, 0);

    // Assert
    assert_eq!(&text[range], "value");
}

#[test]
fn should_span_declaration_name_given_variable_declaration_when_reading_error_range() {
    // Arrange
    let parsed = parse("let value = 1;");

    // Act
    let text = error_text(&parsed, SyntaxKind::VariableDeclaration);

    // Assert
    assert_eq!(text, "value");
}

#[test]
fn should_span_function_name_given_function_declaration_when_reading_error_range() {
    // Arrange
    let parsed = parse("function run() { return 1; }");

    // Act
    let text = error_text(&parsed, SyntaxKind::FunctionDeclaration);

    // Assert
    assert_eq!(text, "run");
}

#[test]
fn should_span_return_keyword_given_return_statement_when_reading_error_range() {
    // Arrange
    let parsed = parse("function run() { return 1; }");

    // Act
    let text = error_text(&parsed, SyntaxKind::ReturnStatement);

    // Assert
    assert_eq!(text, "return");
}

#[test]
fn should_span_first_token_given_source_file_when_reading_error_range() {
    // Arrange
    let parsed = parse("\n  let value = 1;");

    // Act
    let text = error_text(&parsed, SyntaxKind::SourceFile);

    // Assert
    assert_eq!(text, "let");
}

#[test]
fn should_span_constructor_keyword_given_constructor_when_reading_error_range() {
    // Arrange
    let parsed = parse("class C { public constructor() {} }");

    // Act
    let text = error_text(&parsed, SyntaxKind::Constructor);

    // Assert
    assert_eq!(text, "public constructor");
}

#[test]
fn should_span_first_line_given_multiline_arrow_body_when_reading_error_range() {
    // Arrange
    let parsed = parse("const run = () => {\n  return 1;\n};");

    // Act
    let text = error_text(&parsed, SyntaxKind::ArrowFunction);

    // Assert
    assert_eq!(text, "() => {\n");
}
