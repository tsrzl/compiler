use tsrzl::ast::{Ast, NodeId, SymbolFlags, SyntaxKind};
use tsrzl::bind::{BoundFile, bind_source_file};
use tsrzl::diagnostics::Diagnostic;
use tsrzl::parser::{
    ExternalModuleIndicatorOptions, ParseOptions, ParsedSourceFile, ScriptKind, parse_source_file,
};
use tsrzl::symbols::SymbolId;

fn parse(text: &str) -> ParsedSourceFile {
    parse_source_file(&ParseOptions::new("test.ts", ScriptKind::Ts), text)
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

fn first(parsed: &ParsedSourceFile, kind: SyntaxKind) -> NodeId {
    let ast = parsed.ast();
    find(ast, ast.root(), kind).expect("the source contains the node kind")
}

fn file_local(parsed: &ParsedSourceFile, bound: &BoundFile, name: &str) -> Option<SymbolId> {
    bound
        .locals(parsed.ast().root())
        .and_then(|locals| locals.get(name))
}

fn diagnostic_texts(bound: &BoundFile) -> Vec<String> {
    bound.diagnostics().iter().map(Diagnostic::text).collect()
}

#[test]
fn should_declare_function_in_file_locals_given_script_function_when_binding() {
    // Arrange
    let parsed = parse("function run() {}");

    // Act
    let bound = bind(&parsed);

    // Assert
    let symbol = file_local(&parsed, &bound, "run").expect("the function is a file local");
    assert_eq!(bound.symbols().symbol(symbol).flags, SymbolFlags::FUNCTION);
}

#[test]
fn should_report_ts2451_on_each_declaration_given_duplicate_let_when_binding() {
    // Arrange
    let parsed = parse("let value = 1;\nlet value = 2;");

    // Act
    let bound = bind(&parsed);

    // Assert
    assert_eq!(
        diagnostic_texts(&bound),
        [
            "Cannot redeclare block-scoped variable 'value'.",
            "Cannot redeclare block-scoped variable 'value'."
        ]
    );
}

#[test]
fn should_report_duplicate_on_declaration_names_given_duplicate_let_when_binding() {
    // Arrange
    let parsed = parse("let value = 1;\nlet value = 2;");

    // Act
    let bound = bind(&parsed);

    // Assert
    let ranges: Vec<_> = bound.diagnostics().iter().map(Diagnostic::range).collect();
    assert_eq!(ranges, [4..9, 19..24]);
}

#[test]
fn should_merge_declarations_given_duplicate_var_when_binding() {
    // Arrange
    let parsed = parse("var value = 1;\nvar value = 2;");

    // Act
    let bound = bind(&parsed);

    // Assert
    let symbol = file_local(&parsed, &bound, "value").expect("the variable is a file local");
    assert_eq!(bound.symbols().symbol(symbol).declarations.len(), 2);
}

#[test]
fn should_report_ts2300_given_duplicate_class_when_binding() {
    // Arrange
    let parsed = parse("class Shape {}\nclass Shape {}");

    // Act
    let bound = bind(&parsed);

    // Assert
    assert_eq!(
        diagnostic_texts(&bound),
        [
            "Duplicate identifier 'Shape'.",
            "Duplicate identifier 'Shape'."
        ]
    );
}

#[test]
fn should_merge_interfaces_given_same_name_when_binding() {
    // Arrange
    let parsed = parse("interface Shape { a: number }\ninterface Shape { b: number }");

    // Act
    let bound = bind(&parsed);

    // Assert
    assert_eq!(bound.diagnostics(), []);
}

#[test]
fn should_export_function_from_module_symbol_given_exported_function_when_binding() {
    // Arrange
    let parsed = parse("export function run() {}");

    // Act
    let bound = bind(&parsed);

    // Assert
    let module = bound
        .symbol_of(parsed.ast().root())
        .expect("an external module has a symbol");
    let export = bound.symbols().symbol(module).exports.get("run");
    assert!(
        export.is_some_and(|export| bound.symbols().symbol(export).flags == SymbolFlags::FUNCTION)
    );
}

#[test]
fn should_pair_local_with_export_symbol_given_exported_function_when_binding() {
    // Arrange
    let parsed = parse("export function run() {}");

    // Act
    let bound = bind(&parsed);

    // Assert
    let local = file_local(&parsed, &bound, "run").expect("the export has a local symbol");
    assert_eq!(
        bound
            .symbols()
            .combined_local_and_export_symbol_flags(local),
        SymbolFlags::EXPORT_VALUE | SymbolFlags::FUNCTION
    );
}

#[test]
fn should_name_module_symbol_after_file_given_external_module_when_binding() {
    // Arrange
    let parsed = parse("export const value = 1;");

    // Act
    let bound = bind(&parsed);

    // Assert
    let module = bound
        .symbol_of(parsed.ast().root())
        .expect("an external module has a symbol");
    assert_eq!(bound.symbols().symbol(module).name, "\"test\"");
}

#[test]
fn should_declare_method_in_class_members_given_instance_method_when_binding() {
    // Arrange
    let parsed = parse("class Shape { area() { return 1; } }");

    // Act
    let bound = bind(&parsed);

    // Assert
    let class = bound
        .symbol_of(first(&parsed, SyntaxKind::ClassDeclaration))
        .expect("the class has a symbol");
    assert!(bound.symbols().symbol(class).members.get("area").is_some());
}

#[test]
fn should_declare_static_method_in_class_exports_given_static_method_when_binding() {
    // Arrange
    let parsed = parse("class Shape { static create() { return new Shape(); } }");

    // Act
    let bound = bind(&parsed);

    // Assert
    let class = bound
        .symbol_of(first(&parsed, SyntaxKind::ClassDeclaration))
        .expect("the class has a symbol");
    assert!(
        bound
            .symbols()
            .symbol(class)
            .exports
            .get("create")
            .is_some()
    );
}

#[test]
fn should_add_prototype_export_given_class_declaration_when_binding() {
    // Arrange
    let parsed = parse("class Shape {}");

    // Act
    let bound = bind(&parsed);

    // Assert
    let class = bound
        .symbol_of(first(&parsed, SyntaxKind::ClassDeclaration))
        .expect("the class has a symbol");
    let prototype = bound.symbols().symbol(class).exports.get("prototype");
    assert!(prototype.is_some_and(|prototype| {
        bound
            .symbols()
            .symbol(prototype)
            .flags
            .contains(SymbolFlags::PROTOTYPE)
    }));
}

#[test]
fn should_scope_let_to_block_given_nested_block_when_binding() {
    // Arrange
    let parsed = parse("{ let inner = 1; }");

    // Act
    let bound = bind(&parsed);

    // Assert
    let block = first(&parsed, SyntaxKind::Block);
    let inner = bound.locals(block).and_then(|locals| locals.get("inner"));
    assert!(inner.is_some() && file_local(&parsed, &bound, "inner").is_none());
}

#[test]
fn should_hoist_var_to_function_locals_given_var_in_nested_block_when_binding() {
    // Arrange
    let parsed = parse("function run() { { var hoisted = 1; } }");

    // Act
    let bound = bind(&parsed);

    // Assert
    let function = first(&parsed, SyntaxKind::FunctionDeclaration);
    assert!(
        bound
            .locals(function)
            .and_then(|locals| locals.get("hoisted"))
            .is_some()
    );
}

#[test]
fn should_declare_parameter_in_function_locals_given_parameter_when_binding() {
    // Arrange
    let parsed = parse("function run(input: number) {}");

    // Act
    let bound = bind(&parsed);

    // Assert
    let function = first(&parsed, SyntaxKind::FunctionDeclaration);
    let parameter = bound
        .locals(function)
        .and_then(|locals| locals.get("input"));
    assert!(
        parameter.is_some_and(|parameter| bound.symbols().symbol(parameter).flags
            == SymbolFlags::FUNCTION_SCOPED_VARIABLE)
    );
}

#[test]
fn should_report_ts2300_given_duplicate_parameters_when_binding() {
    // Arrange
    let parsed = parse("function run(input: number, input: string) {}");

    // Act
    let bound = bind(&parsed);

    // Assert
    assert_eq!(
        diagnostic_texts(&bound),
        [
            "Duplicate identifier 'input'.",
            "Duplicate identifier 'input'."
        ]
    );
}

#[test]
fn should_declare_enum_members_in_enum_exports_given_enum_when_binding() {
    // Arrange
    let parsed = parse("enum Color { Red, Green }");

    // Act
    let bound = bind(&parsed);

    // Assert
    let color = file_local(&parsed, &bound, "Color").expect("the enum is a file local");
    let names: Vec<_> = bound
        .symbols()
        .symbol(color)
        .exports
        .iter()
        .map(|(name, _)| name)
        .collect();
    assert_eq!(names, ["Red", "Green"]);
}

#[test]
fn should_report_enum_merge_error_given_enum_and_class_when_binding() {
    // Arrange
    let parsed = parse("enum Shape { A }\nclass Shape {}");

    // Act
    let bound = bind(&parsed);

    // Assert
    assert_eq!(
        diagnostic_texts(&bound),
        [
            "Enum declarations can only merge with namespace or other enum declarations.",
            "Enum declarations can only merge with namespace or other enum declarations."
        ]
    );
}

#[test]
fn should_report_ts2528_given_two_default_classes_when_binding() {
    // Pinned TypeScript 7.0.2 case: conformance/es6/modules/multipleDefaultExports03.ts.
    // Arrange
    let parsed = parse("export default class C {}\nexport default class C {}");

    // Act
    let bound = bind(&parsed);

    // Assert
    assert_eq!(
        diagnostic_texts(&bound),
        [
            "A module cannot have multiple default exports.",
            "A module cannot have multiple default exports."
        ]
    );
}

#[test]
fn should_bind_functions_first_given_function_after_let_when_binding() {
    // Arrange
    let parsed = parse("let value = 1;\nfunction run() {}");

    // Act
    let bound = bind(&parsed);

    // Assert
    let names: Vec<_> = bound
        .locals(parsed.ast().root())
        .expect("the file has locals")
        .iter()
        .map(|(name, _)| name)
        .collect();
    assert_eq!(names, ["run", "value"]);
}

#[test]
fn should_declare_import_alias_in_file_locals_given_named_import_when_binding() {
    // Arrange
    let parsed = parse("import { helper } from \"./helper\";");

    // Act
    let bound = bind(&parsed);

    // Assert
    let helper = file_local(&parsed, &bound, "helper").expect("the import is a file local");
    assert_eq!(bound.symbols().symbol(helper).flags, SymbolFlags::ALIAS);
}

#[test]
fn should_declare_type_parameter_in_function_locals_given_generic_function_when_binding() {
    // Arrange
    let parsed = parse("function identity<T>(value: T): T { return value; }");

    // Act
    let bound = bind(&parsed);

    // Assert
    let function = first(&parsed, SyntaxKind::FunctionDeclaration);
    let parameter = bound.locals(function).and_then(|locals| locals.get("T"));
    assert!(parameter.is_some_and(
        |parameter| bound.symbols().symbol(parameter).flags == SymbolFlags::TYPE_PARAMETER
    ));
}
