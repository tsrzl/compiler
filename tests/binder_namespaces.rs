use tsrzl::ast::{Ast, NodeId, SymbolFlags, SyntaxKind};
use tsrzl::bind::{BoundFile, bind_source_file};
use tsrzl::diagnostics::Diagnostic;
use tsrzl::parser::{
    ExternalModuleIndicatorOptions, ParseOptions, ParsedSourceFile, ScriptKind, parse_source_file,
};
use tsrzl::symbols::SymbolId;

fn parse_named(file_name: &str, text: &str) -> ParsedSourceFile {
    parse_source_file(&ParseOptions::new(file_name, ScriptKind::Ts), text)
}

fn parse(text: &str) -> ParsedSourceFile {
    parse_named("test.ts", text)
}

fn bind(parsed: &ParsedSourceFile) -> BoundFile {
    bind_source_file(parsed, ExternalModuleIndicatorOptions::default())
}

fn find(ast: &Ast, id: NodeId, kind: SyntaxKind) -> Option<NodeId> {
    if ast.node(id).kind() == kind {
        return Some(id);
    }
    ast.children(id)
        .into_iter()
        .find_map(|child| find(ast, child, kind))
}

fn file_local(parsed: &ParsedSourceFile, bound: &BoundFile, name: &str) -> SymbolId {
    bound
        .locals(parsed.ast().root())
        .and_then(|locals| locals.get(name))
        .expect("the name is a file local")
}

fn diagnostic_texts(bound: &BoundFile) -> Vec<String> {
    bound.diagnostics().iter().map(Diagnostic::text).collect()
}

#[test]
fn should_declare_value_module_given_namespace_with_value_when_binding() {
    // Arrange
    let parsed = parse("namespace Shapes { export const sides = 4; }");

    // Act
    let bound = bind(&parsed);

    // Assert
    let shapes = file_local(&parsed, &bound, "Shapes");
    assert_eq!(
        bound.symbols().symbol(shapes).flags,
        SymbolFlags::VALUE_MODULE
    );
}

#[test]
fn should_declare_namespace_module_given_type_only_namespace_when_binding() {
    // Arrange
    let parsed = parse("namespace Shapes { export interface Square {} }");

    // Act
    let bound = bind(&parsed);

    // Assert
    let shapes = file_local(&parsed, &bound, "Shapes");
    assert_eq!(
        bound.symbols().symbol(shapes).flags,
        SymbolFlags::NAMESPACE_MODULE
    );
}

#[test]
fn should_export_member_from_namespace_symbol_given_exported_member_when_binding() {
    // Arrange
    let parsed = parse("namespace Shapes { export const sides = 4; }");

    // Act
    let bound = bind(&parsed);

    // Assert
    let shapes = file_local(&parsed, &bound, "Shapes");
    assert!(
        bound
            .symbols()
            .symbol(shapes)
            .exports
            .get("sides")
            .is_some()
    );
}

#[test]
fn should_keep_unexported_member_in_namespace_locals_given_local_member_when_binding() {
    // Arrange
    let parsed = parse("namespace Shapes { const hidden = 4; }");

    // Act
    let bound = bind(&parsed);

    // Assert
    let ast = parsed.ast();
    let namespace = find(ast, ast.root(), SyntaxKind::ModuleDeclaration).expect("a namespace");
    assert!(
        bound
            .locals(namespace)
            .and_then(|locals| locals.get("hidden"))
            .is_some()
    );
}

#[test]
fn should_mark_const_enum_only_module_given_only_const_enums_when_binding() {
    // Arrange
    let parsed = parse("namespace Flags { export const enum Kind { A } }");

    // Act
    let bound = bind(&parsed);

    // Assert
    let flags = bound
        .symbols()
        .symbol(file_local(&parsed, &bound, "Flags"))
        .flags;
    assert!(flags.contains(SymbolFlags::CONST_ENUM_ONLY_MODULE));
}

#[test]
fn should_merge_namespace_with_function_given_same_name_when_binding() {
    // Arrange
    let parsed = parse("function area() {}\nnamespace area { export const unit = \"m\"; }");

    // Act
    let bound = bind(&parsed);

    // Assert
    let flags = bound
        .symbols()
        .symbol(file_local(&parsed, &bound, "area"))
        .flags;
    assert_eq!(flags, SymbolFlags::FUNCTION | SymbolFlags::VALUE_MODULE);
}

#[test]
fn should_name_ambient_module_with_quotes_given_string_module_name_when_binding() {
    // Arrange
    let parsed = parse("declare module \"feature\" { export const value: number; }");

    // Act
    let bound = bind(&parsed);

    // Assert
    assert_eq!(
        bound
            .symbols()
            .symbol(file_local(&parsed, &bound, "\"feature\""))
            .flags,
        SymbolFlags::VALUE_MODULE
    );
}

#[test]
fn should_report_ts2668_given_exported_ambient_module_when_binding() {
    // Arrange
    let parsed = parse("export declare module \"feature\" {}");

    // Act
    let bound = bind(&parsed);

    // Assert
    assert_eq!(
        diagnostic_texts(&bound),
        [
            "'export' modifier cannot be applied to ambient modules and module augmentations since they are always visible."
        ]
    );
}

#[test]
fn should_report_ts5061_given_ambient_module_pattern_with_two_asterisks_when_binding() {
    // Arrange
    let parsed = parse("declare module \"a*b*\" {}");

    // Act
    let bound = bind(&parsed);

    // Assert
    assert_eq!(
        diagnostic_texts(&bound),
        ["Pattern 'a*b*' can have at most one '*' character."]
    );
}

#[test]
fn should_record_pattern_ambient_module_given_wildcard_module_name_when_binding() {
    // Arrange
    let parsed = parse("declare module \"*.css\" {}");

    // Act
    let bound = bind(&parsed);

    // Assert
    let patterns: Vec<_> = bound
        .pattern_ambient_modules()
        .iter()
        .map(|module| module.pattern.as_str())
        .collect();
    assert_eq!(patterns, ["*.css"]);
}

#[test]
fn should_report_module_file_requirement_given_global_export_in_script_when_binding() {
    // Arrange
    let parsed = parse_named("test.d.ts", "export as namespace Lib;");

    // Act
    let bound = bind(&parsed);

    // Assert
    assert_eq!(
        diagnostic_texts(&bound),
        ["Global module exports may only appear in module files."]
    );
}

#[test]
fn should_declare_global_export_given_umd_declaration_file_when_binding() {
    // Arrange
    let parsed = parse_named(
        "test.d.ts",
        "export declare const value: number;\nexport as namespace Lib;",
    );

    // Act
    let bound = bind(&parsed);

    // Assert
    assert!(bound.global_exports().get("Lib").is_some());
}
