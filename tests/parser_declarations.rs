use tsrzl::ast::{Ast, NodeId, SyntaxKind};
use tsrzl::parser::{ParseOptions, ParsedSourceFile, ScriptKind, parse_source_file};

fn parse(text: &str) -> ParsedSourceFile {
    parse_source_file(&ParseOptions::new("test.ts", ScriptKind::Ts), text)
}

fn statements(ast: &Ast) -> Vec<NodeId> {
    let source_file = ast
        .node(ast.root())
        .data()
        .as_source_file()
        .expect("the root is a source file");
    ast.list(source_file.statements).to_vec()
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
fn should_parse_constructor_member_given_class_with_constructor_when_parsing_declaration() {
    // Arrange
    let parsed = parse("class A { constructor(public x: number) {} }");
    let ast = parsed.ast();
    let class = ast
        .node(statements(ast)[0])
        .data()
        .as_class_declaration()
        .expect("a class declaration is parsed");

    // Act
    let actual = ast.node(ast.list(class.members)[0]).kind();

    // Assert
    assert_eq!(actual, SyntaxKind::Constructor);
}

#[test]
fn should_nest_module_declarations_given_dotted_namespace_name_when_parsing_declaration() {
    // Arrange
    let parsed = parse("namespace A.B { }");
    let ast = parsed.ast();
    let outer = ast
        .node(statements(ast)[0])
        .data()
        .as_module_declaration()
        .expect("a module declaration is parsed");

    // Act
    let actual = outer.body.map(|body| ast.node(body).kind());

    // Assert
    assert_eq!(actual, Some(SyntaxKind::ModuleDeclaration));
}

#[test]
fn should_mark_specifier_type_only_given_type_modifier_in_named_import_when_parsing_declaration() {
    // Arrange
    let parsed = parse(r#"import { type A } from "m";"#);
    let ast = parsed.ast();
    let specifier =
        find(ast, ast.root(), SyntaxKind::ImportSpecifier).expect("an import specifier is parsed");

    // Act
    let actual = ast
        .node(specifier)
        .data()
        .as_import_specifier()
        .map(|specifier| specifier.is_type_only);

    // Assert
    assert_eq!(actual, Some(true));
}

#[test]
fn should_parse_export_assignment_given_export_default_expression_when_parsing_declaration() {
    // Arrange
    let parsed = parse("export default 42;");

    // Act
    let actual = parsed.ast().node(statements(parsed.ast())[0]).kind();

    // Assert
    assert_eq!(actual, SyntaxKind::ExportAssignment);
}
