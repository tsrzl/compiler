use tsrzl::ast::{Ast, ModifierFlags, NodeFlags, NodeId, SyntaxKind};
use tsrzl::parser::{ParseOptions, ParsedSourceFile, ScriptKind, parse_source_file};

fn parse(text: &str) -> ParsedSourceFile {
    parse_source_file(&ParseOptions::new("test.ts", ScriptKind::Ts), text)
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

fn first(parsed: &ParsedSourceFile, kind: SyntaxKind) -> NodeId {
    let ast = parsed.ast();
    find(ast, ast.root(), kind).expect("the source contains the node kind")
}

#[test]
fn should_combine_list_flags_given_const_declaration_when_reading_combined_node_flags() {
    // Arrange
    let parsed = parse("const value = 1;");
    let declaration = first(&parsed, SyntaxKind::VariableDeclaration);

    // Act
    let flags = parsed.ast().combined_node_flags(declaration);

    // Assert
    assert!(flags.contains(NodeFlags::CONST));
}

#[test]
fn should_combine_statement_modifiers_given_exported_variable_when_reading_combined_modifier_flags()
{
    // Arrange
    let parsed = parse("export let value = 1;");
    let declaration = first(&parsed, SyntaxKind::VariableDeclaration);

    // Act
    let flags = parsed.ast().combined_modifier_flags(declaration);

    // Assert
    assert!(flags.contains(ModifierFlags::EXPORT));
}

#[test]
fn should_report_block_scope_given_let_declaration_when_checking_block_or_catch_scope() {
    // Arrange
    let parsed = parse("let value = 1;");
    let declaration = first(&parsed, SyntaxKind::VariableDeclaration);

    // Act
    let scoped = parsed.ast().is_block_or_catch_scoped(declaration);

    // Assert
    assert!(scoped);
}

#[test]
fn should_not_report_block_scope_given_var_declaration_when_checking_block_or_catch_scope() {
    // Arrange
    let parsed = parse("var value = 1;");
    let declaration = first(&parsed, SyntaxKind::VariableDeclaration);

    // Act
    let scoped = parsed.ast().is_block_or_catch_scoped(declaration);

    // Assert
    assert!(!scoped);
}

#[test]
fn should_report_catch_scope_given_catch_variable_when_checking_block_or_catch_scope() {
    // Arrange
    let parsed = parse("try {} catch (error) {}");
    let declaration = first(&parsed, SyntaxKind::VariableDeclaration);

    // Act
    let scoped = parsed.ast().is_block_or_catch_scoped(declaration);

    // Assert
    assert!(scoped);
}

#[test]
fn should_return_declared_identifier_given_function_declaration_when_reading_declaration_name() {
    // Arrange
    let parsed = parse("function run() {}");
    let function = first(&parsed, SyntaxKind::FunctionDeclaration);

    // Act
    let name = parsed.ast().name_of_declaration(function);

    // Assert
    assert_eq!(
        name.map(|name| parsed.ast().identifier_text(name)),
        Some(Some("run"))
    );
}

#[test]
fn should_return_variable_name_given_assigned_arrow_function_when_reading_declaration_name() {
    // Arrange
    let parsed = parse("const run = () => {};");
    let arrow = first(&parsed, SyntaxKind::ArrowFunction);

    // Act
    let name = parsed.ast().name_of_declaration(arrow);

    // Assert
    assert_eq!(
        name.map(|name| parsed.ast().identifier_text(name)),
        Some(Some("run"))
    );
}

#[test]
fn should_report_dynamic_name_given_computed_identifier_key_when_checking_declaration() {
    // Arrange
    let parsed = parse("class C { [key]() {} }");
    let method = first(&parsed, SyntaxKind::MethodDeclaration);

    // Act
    let dynamic = parsed.ast().has_dynamic_name(method);

    // Assert
    assert!(dynamic);
}

#[test]
fn should_not_report_dynamic_name_given_computed_string_key_when_checking_declaration() {
    // Arrange
    let parsed = parse("class C { [\"key\"]() {} }");
    let method = first(&parsed, SyntaxKind::MethodDeclaration);

    // Act
    let dynamic = parsed.ast().has_dynamic_name(method);

    // Assert
    assert!(!dynamic);
}

#[test]
fn should_report_ambient_module_given_quoted_module_name_when_checking_module_declaration() {
    // Arrange
    let parsed = parse("declare module \"feature\" {}");
    let module = first(&parsed, SyntaxKind::ModuleDeclaration);

    // Act
    let ambient = parsed.ast().is_ambient_module(module);

    // Assert
    assert!(ambient);
}

#[test]
fn should_not_report_ambient_module_given_namespace_identifier_when_checking_module_declaration() {
    // Arrange
    let parsed = parse("namespace Feature {}");
    let module = first(&parsed, SyntaxKind::ModuleDeclaration);

    // Act
    let ambient = parsed.ast().is_ambient_module(module);

    // Assert
    assert!(!ambient);
}
