use tsrzl::ast::{Ast, NodeId, SyntaxKind};
use tsrzl::parser::{ParseOptions, ParsedSourceFile, ScriptKind, parse_source_file};

fn parse_tsx(text: &str) -> ParsedSourceFile {
    parse_source_file(&ParseOptions::new("test.tsx", ScriptKind::Tsx), text)
}

/// Returns the first node of `kind` in a pre-order walk.
fn find(ast: &Ast, id: NodeId, kind: SyntaxKind) -> Option<NodeId> {
    if ast.node(id).kind() == kind {
        return Some(id);
    }
    ast.children(id)
        .into_iter()
        .find_map(|child| find(ast, child, kind))
}

#[test]
fn should_parse_element_with_children_given_nested_tags_when_parsing_tsx() {
    // Arrange
    let parsed = parse_tsx("const x = <div><span>hi</span></div>;");
    let ast = parsed.ast();
    let element = find(ast, ast.root(), SyntaxKind::JsxElement).expect("an element is parsed");

    // Act
    let actual = ast
        .node(element)
        .data()
        .as_jsx_element()
        .map(|element| element.children.len());

    // Assert
    assert_eq!(actual, Some(1));
}

#[test]
fn should_parse_self_closing_element_given_attributes_when_parsing_tsx() {
    // Arrange
    let parsed = parse_tsx(r#"const x = <input value="a" {...rest} />;"#);

    // Act
    let actual = find(
        parsed.ast(),
        parsed.ast().root(),
        SyntaxKind::JsxSpreadAttribute,
    );

    // Assert
    assert!(actual.is_some());
}

#[test]
fn should_report_missing_closing_tag_given_mismatched_tags_when_parsing_tsx() {
    // Arrange
    let text = "const x = <div></span>;";

    // Act
    let parsed = parse_tsx(text);

    // Assert
    let codes = parsed
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.message().code())
        .collect::<Vec<_>>();
    assert!(codes.contains(&17002), "{codes:?}");
}
