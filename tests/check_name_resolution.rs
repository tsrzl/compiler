use tsrzl::ast::{Ast, NodeId, SymbolFlags, SyntaxKind};
use tsrzl::check::{Checker, CheckerOptions, NodeRef, SymbolRef};
use tsrzl::diagnostics;
use tsrzl::parser::{ParseOptions, ScriptKind, parse_source_file};
use tsrzl::program::ProgramFile;

fn file(name: &str, text: &str) -> ProgramFile {
    ProgramFile::new(parse_source_file(
        &ParseOptions::new(name, ScriptKind::Ts),
        text,
    ))
}

fn all(ast: &Ast, id: NodeId, found: &mut Vec<NodeId>) {
    found.push(id);
    for child in ast.children(id) {
        all(ast, child, found);
    }
}

/// Returns the identifier `name` at `occurrence` in source order in file 0.
fn identifier(files: &[ProgramFile], name: &str, occurrence: usize) -> NodeRef {
    let ast = files[0].parsed().ast();
    let mut nodes = Vec::new();
    all(ast, ast.root(), &mut nodes);
    let node = nodes
        .into_iter()
        .filter(|&id| {
            ast.node(id).kind() == SyntaxKind::Identifier && ast.identifier_text(id) == Some(name)
        })
        .nth(occurrence)
        .expect("the identifier occurs in the source");
    NodeRef { file: 0, node }
}

fn declaration_kind(checker: &Checker<'_>, files: &[ProgramFile], symbol: SymbolRef) -> SyntaxKind {
    let declaration = checker.symbol_declarations(symbol)[0];
    files[declaration.file]
        .parsed()
        .ast()
        .node(declaration.node)
        .kind()
}

#[test]
fn should_resolve_function_local_given_reference_in_body_when_resolving_name() {
    // Arrange
    let files = [file("a.ts", "function run() { let count = 1; count; }")];
    let checker = Checker::new(&files, CheckerOptions::default());
    let reference = identifier(&files, "count", 1);

    // Act
    let symbol = checker.resolve_name(reference, "count", SymbolFlags::VALUE, None);

    // Assert
    assert!(symbol.is_some_and(
        |symbol| declaration_kind(&checker, &files, symbol) == SyntaxKind::VariableDeclaration
    ));
}

#[test]
fn should_resolve_parameter_given_reference_in_body_when_resolving_name() {
    // Arrange
    let files = [file("a.ts", "function run(input: number) { input; }")];
    let checker = Checker::new(&files, CheckerOptions::default());
    let reference = identifier(&files, "input", 1);

    // Act
    let symbol = checker.resolve_name(reference, "input", SymbolFlags::VALUE, None);

    // Assert
    assert!(
        symbol.is_some_and(
            |symbol| declaration_kind(&checker, &files, symbol) == SyntaxKind::Parameter
        )
    );
}

#[test]
fn should_resolve_type_parameter_given_reference_in_return_type_when_resolving_name() {
    // Arrange
    let files = [file(
        "a.ts",
        "function identity<T>(value: T): T { return value; }",
    )];
    let checker = Checker::new(&files, CheckerOptions::default());
    let return_type = identifier(&files, "T", 2);

    // Act
    let symbol = checker.resolve_name(return_type, "T", SymbolFlags::TYPE, None);

    // Assert
    assert!(symbol.is_some_and(
        |symbol| declaration_kind(&checker, &files, symbol) == SyntaxKind::TypeParameter
    ));
}

#[test]
fn should_not_resolve_body_type_given_reference_in_parameter_list_when_resolving_name() {
    // Arrange
    let files = [file(
        "a.ts",
        "function run(value: Local) { type Local = number; }",
    )];
    let checker = Checker::new(&files, CheckerOptions::default());
    let parameter_type = identifier(&files, "Local", 0);

    // Act
    let symbol = checker.resolve_name(parameter_type, "Local", SymbolFlags::TYPE, None);

    // Assert
    assert_eq!(symbol, None);
}

#[test]
fn should_resolve_global_from_other_script_given_script_reference_when_resolving_name() {
    // Arrange
    let files = [file("a.ts", "shared;"), file("b.ts", "var shared = 1;")];
    let checker = Checker::new(&files, CheckerOptions::default());
    let reference = identifier(&files, "shared", 0);

    // Act
    let symbol = checker.resolve_name(reference, "shared", SymbolFlags::VALUE, None);

    // Assert
    assert_eq!(symbol, checker.globals().get("shared"));
}

#[test]
fn should_not_resolve_block_scoped_let_given_reference_outside_block_when_resolving_name() {
    // Arrange
    let files = [file("a.ts", "{ let inner = 1; }\ninner;")];
    let checker = Checker::new(&files, CheckerOptions::default());
    let reference = identifier(&files, "inner", 1);

    // Act
    let symbol = checker.resolve_name(reference, "inner", SymbolFlags::VALUE, None);

    // Assert
    assert_eq!(symbol, None);
}

#[test]
fn should_resolve_export_symbol_given_exported_const_reference_in_module_when_resolving_name() {
    // Arrange
    let files = [file("a.ts", "export const value = 1;\nvalue;")];
    let checker = Checker::new(&files, CheckerOptions::default());
    let reference = identifier(&files, "value", 1);

    // Act
    let symbol = checker.resolve_name(reference, "value", SymbolFlags::VALUE, None);

    // Assert
    assert!(
        symbol.is_some_and(
            |symbol| checker.symbol_flags(symbol) == SymbolFlags::BLOCK_SCOPED_VARIABLE
        )
    );
}

#[test]
fn should_resolve_sibling_member_given_enum_initializer_when_resolving_name() {
    // Arrange
    let files = [file("a.ts", "enum Flags { A = 1, B = A }")];
    let checker = Checker::new(&files, CheckerOptions::default());
    let reference = identifier(&files, "A", 1);

    // Act
    let symbol = checker.resolve_name(reference, "A", SymbolFlags::VALUE, None);

    // Assert
    assert!(symbol.is_some_and(|symbol| checker.symbol_flags(symbol) == SymbolFlags::ENUM_MEMBER));
}

#[test]
fn should_resolve_arguments_symbol_given_reference_in_function_when_resolving_name() {
    // Arrange
    let files = [file("a.ts", "function run() { arguments; }")];
    let checker = Checker::new(&files, CheckerOptions::default());
    let reference = identifier(&files, "arguments", 0);

    // Act
    let symbol = checker.resolve_name(reference, "arguments", SymbolFlags::VALUE, None);

    // Assert
    assert!(symbol.is_some_and(|symbol| {
        checker.symbol_name(symbol) == "arguments"
            && checker
                .symbol_flags(symbol)
                .contains(SymbolFlags::TRANSIENT)
    }));
}

#[test]
fn should_resolve_own_name_given_named_function_expression_body_when_resolving_name() {
    // Arrange
    let files = [file("a.ts", "const run = function loop() { loop; };")];
    let checker = Checker::new(&files, CheckerOptions::default());
    let reference = identifier(&files, "loop", 1);

    // Act
    let symbol = checker.resolve_name(reference, "loop", SymbolFlags::VALUE, None);

    // Assert
    assert!(symbol.is_some_and(
        |symbol| declaration_kind(&checker, &files, symbol) == SyntaxKind::FunctionExpression
    ));
}

#[test]
fn should_report_ts2302_given_class_type_parameter_in_static_member_when_resolving_name() {
    // Arrange
    let files = [file("a.ts", "class Box<T> { static empty: T; }")];
    let mut checker = Checker::new(&files, CheckerOptions::default());
    let reference = identifier(&files, "T", 1);

    // Act
    let symbol = checker.resolve_name_reporting(
        reference,
        "T",
        SymbolFlags::TYPE,
        diagnostics::CANNOT_FIND_NAME_0,
    );

    // Assert
    let texts: Vec<_> = checker
        .diagnostics()
        .iter()
        .map(|d| d.diagnostic().text())
        .collect();
    assert_eq!(
        (symbol, texts),
        (
            None,
            vec!["Static members cannot reference class type parameters.".to_owned()]
        )
    );
}

#[test]
fn should_report_at_reference_given_class_type_parameter_in_static_member_when_resolving_name() {
    // Arrange
    let files = [file("a.ts", "class Box<T> { static empty: T; }")];
    let mut checker = Checker::new(&files, CheckerOptions::default());
    let reference = identifier(&files, "T", 1);

    // Act
    checker.resolve_name_reporting(
        reference,
        "T",
        SymbolFlags::TYPE,
        diagnostics::CANNOT_FIND_NAME_0,
    );

    // Assert
    assert_eq!(checker.diagnostics()[0].diagnostic().range(), 29..30);
}

fn report_unresolved(
    text: &str,
    name: &str,
    occurrence: usize,
    meaning: SymbolFlags,
) -> Vec<String> {
    let files = [file("a.ts", text)];
    let mut checker = Checker::new(&files, CheckerOptions::default());
    let reference = identifier(&files, name, occurrence);
    let message = checker.cannot_find_name_message(reference);
    checker.resolve_name_reporting(reference, name, meaning, message);
    checker
        .diagnostics()
        .iter()
        .map(|d| d.diagnostic().text())
        .collect()
}

#[test]
fn should_report_ts2304_given_unknown_name_when_resolving_name() {
    // Arrange
    let text = "missing;";

    // Act
    let texts = report_unresolved(text, "missing", 0, SymbolFlags::VALUE);

    // Assert
    assert_eq!(texts, ["Cannot find name 'missing'."]);
}

#[test]
fn should_report_ts2552_given_misspelled_local_when_resolving_name() {
    // Arrange
    let text = "function run() { let counter = 1; countr; }";

    // Act
    let texts = report_unresolved(text, "countr", 0, SymbolFlags::VALUE);

    // Assert
    assert_eq!(
        texts,
        ["Cannot find name 'countr'. Did you mean 'counter'?"]
    );
}

#[test]
fn should_relate_declaration_given_spelling_suggestion_when_resolving_name() {
    // Arrange
    let files = [file("a.ts", "function run() { let counter = 1; countr; }")];
    let mut checker = Checker::new(&files, CheckerOptions::default());
    let reference = identifier(&files, "countr", 0);
    let message = checker.cannot_find_name_message(reference);

    // Act
    checker.resolve_name_reporting(reference, "countr", SymbolFlags::VALUE, message);

    // Assert
    let related: Vec<_> = checker.diagnostics()[0]
        .related()
        .iter()
        .map(|d| d.diagnostic().text())
        .collect();
    assert_eq!(related, ["'counter' is declared here."]);
}

#[test]
fn should_suggest_dom_lib_given_unresolved_document_when_resolving_name() {
    // Arrange
    let text = "document;";

    // Act
    let texts = report_unresolved(text, "document", 0, SymbolFlags::VALUE);

    // Assert
    assert_eq!(
        texts,
        [
            "Cannot find name 'document'. Do you need to change your target library? Try changing the 'lib' compiler option to include 'dom'."
        ]
    );
}

#[test]
fn should_suggest_lib_given_unresolved_es2015_global_when_resolving_name() {
    // Arrange
    let text = "Promise;";

    // Act
    let texts = report_unresolved(text, "Promise", 0, SymbolFlags::VALUE);

    // Assert
    assert_eq!(
        texts,
        [
            "Cannot find name 'Promise'. Do you need to change your target library? Try changing the 'lib' compiler option to 'es2015' or later."
        ]
    );
}
