use tsrzl::ast::{Ast, ModuleInstanceState, NodeId, SyntaxKind};
use tsrzl::parser::{ParseOptions, ParsedSourceFile, ScriptKind, parse_source_file};

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

fn state(text: &str) -> ModuleInstanceState {
    let parsed = parse(text);
    let ast = parsed.ast();
    let module = find(ast, ast.root(), SyntaxKind::ModuleDeclaration).expect("a namespace");
    ast.module_instance_state(module)
}

#[test]
fn should_be_non_instantiated_given_only_types_when_reading_module_instance_state() {
    // Arrange
    let text = "namespace N { interface I {} type T = number; }";

    // Act
    let actual = state(text);

    // Assert
    assert_eq!(actual, ModuleInstanceState::NonInstantiated);
}

#[test]
fn should_be_instantiated_given_variable_when_reading_module_instance_state() {
    // Arrange
    let text = "namespace N { export const value = 1; }";

    // Act
    let actual = state(text);

    // Assert
    assert_eq!(actual, ModuleInstanceState::Instantiated);
}

#[test]
fn should_be_const_enum_only_given_const_enum_when_reading_module_instance_state() {
    // Arrange
    let text = "namespace N { export const enum E { A } }";

    // Act
    let actual = state(text);

    // Assert
    assert_eq!(actual, ModuleInstanceState::ConstEnumOnly);
}

#[test]
fn should_be_non_instantiated_given_reexported_interface_when_reading_module_instance_state() {
    // Arrange
    let text = "namespace N { interface I {} export { I }; }";

    // Act
    let actual = state(text);

    // Assert
    assert_eq!(actual, ModuleInstanceState::NonInstantiated);
}

#[test]
fn should_be_instantiated_given_reexported_variable_when_reading_module_instance_state() {
    // Arrange
    let text = "namespace N { const value = 1; export { value }; }";

    // Act
    let actual = state(text);

    // Assert
    assert_eq!(actual, ModuleInstanceState::Instantiated);
}

#[test]
fn should_be_instantiated_given_bodiless_module_when_reading_module_instance_state() {
    // Arrange
    let text = "declare module \"feature\";";

    // Act
    let actual = state(text);

    // Assert
    assert_eq!(actual, ModuleInstanceState::Instantiated);
}
