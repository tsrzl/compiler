use tsrzl::ast::SymbolFlags;
use tsrzl::check::{Checker, CheckerOptions};
use tsrzl::parser::{ParseOptions, ScriptKind, parse_source_file};
use tsrzl::program::ProgramFile;

fn file(name: &str, text: &str) -> ProgramFile {
    ProgramFile::new(parse_source_file(
        &ParseOptions::new(name, ScriptKind::Ts),
        text,
    ))
}

fn diagnostics(checker: &Checker<'_>) -> Vec<(usize, String)> {
    checker
        .diagnostics()
        .iter()
        .map(|diagnostic| (diagnostic.file(), diagnostic.diagnostic().text()))
        .collect()
}

#[test]
fn should_merge_script_interfaces_into_one_global_given_two_files_when_initializing_checker() {
    // Arrange
    let files = [
        file("a.ts", "interface Shape { a: number }"),
        file("b.ts", "interface Shape { b: number }"),
    ];

    // Act
    let checker = Checker::new(&files, CheckerOptions::default());

    // Assert
    let shape = checker.globals().get("Shape").expect("Shape is global");
    assert_eq!(checker.symbol_declarations(shape).len(), 2);
}

#[test]
fn should_keep_module_locals_out_of_globals_given_module_file_when_initializing_checker() {
    // Arrange
    let files = [file("a.ts", "export const value = 1;\nconst hidden = 2;")];

    // Act
    let checker = Checker::new(&files, CheckerOptions::default());

    // Assert
    assert!(checker.globals().get("hidden").is_none());
}

#[test]
fn should_report_ts2451_in_both_files_given_global_let_redeclaration_when_initializing_checker() {
    // Arrange
    let files = [
        file("a.ts", "let value = 1;"),
        file("b.ts", "let value = 2;"),
    ];

    // Act
    let checker = Checker::new(&files, CheckerOptions::default());

    // Assert
    assert_eq!(
        diagnostics(&checker),
        [
            (
                1,
                "Cannot redeclare block-scoped variable 'value'.".to_owned()
            ),
            (
                0,
                "Cannot redeclare block-scoped variable 'value'.".to_owned()
            )
        ]
    );
}

#[test]
fn should_relate_other_declaration_given_global_redeclaration_when_initializing_checker() {
    // Arrange
    let files = [
        file("a.ts", "let value = 1;"),
        file("b.ts", "let value = 2;"),
    ];

    // Act
    let checker = Checker::new(&files, CheckerOptions::default());

    // Assert
    let related: Vec<_> = checker.diagnostics()[0]
        .related()
        .iter()
        .map(|related| (related.file(), related.diagnostic().text()))
        .collect();
    assert_eq!(related, [(0, "'value' was also declared here.".to_owned())]);
}

#[test]
fn should_report_ts2397_given_script_declaring_global_this_when_initializing_checker() {
    // Arrange
    let files = [file("a.ts", "var globalThis = 1;")];

    // Act
    let checker = Checker::new(&files, CheckerOptions::default());

    // Assert
    assert_eq!(
        diagnostics(&checker),
        [(
            0,
            "Declaration name conflicts with built-in global identifier 'globalThis'.".to_owned()
        )]
    );
}

#[test]
fn should_report_ts2397_given_script_declaring_undefined_when_initializing_checker() {
    // Arrange
    let files = [file("a.ts", "var undefined = 1;")];

    // Act
    let checker = Checker::new(&files, CheckerOptions::default());

    // Assert
    assert_eq!(
        diagnostics(&checker),
        [(
            0,
            "Declaration name conflicts with built-in global identifier 'undefined'.".to_owned()
        )]
    );
}

#[test]
fn should_add_undefined_symbol_given_no_conflicting_global_when_initializing_checker() {
    // Arrange
    let files = [file("a.ts", "let value = 1;")];

    // Act
    let checker = Checker::new(&files, CheckerOptions::default());

    // Assert
    let undefined = checker
        .globals()
        .get("undefined")
        .expect("undefined is global");
    assert_eq!(
        checker.symbol_flags(undefined),
        SymbolFlags::PROPERTY | SymbolFlags::TRANSIENT
    );
}

#[test]
fn should_expose_globals_through_global_this_given_script_global_when_initializing_checker() {
    // Arrange
    let files = [file("a.ts", "var value = 1;")];

    // Act
    let checker = Checker::new(&files, CheckerOptions::default());

    // Assert
    let global_this = checker
        .globals()
        .get("globalThis")
        .expect("globalThis is global");
    assert_eq!(
        checker.export_of(global_this, "value"),
        checker.globals().get("value")
    );
}

#[test]
fn should_merge_namespace_exports_given_namespace_in_two_files_when_initializing_checker() {
    // Arrange
    let files = [
        file("a.ts", "namespace Shapes { export const sides = 4; }"),
        file("b.ts", "namespace Shapes { export const corners = 4; }"),
    ];

    // Act
    let checker = Checker::new(&files, CheckerOptions::default());

    // Assert
    let shapes = checker.globals().get("Shapes").expect("Shapes is global");
    let names: Vec<_> = checker.export_names(shapes);
    assert_eq!(names, ["sides", "corners"]);
}

#[test]
fn should_merge_lib_globals_without_conflicts_given_default_libs_when_initializing_checker() {
    // Arrange
    let roots = vec![parse_source_file(
        &ParseOptions::new("/app/main.ts", ScriptKind::Ts),
        "let value = 1;",
    )];
    let program = tsrzl::program::Program::load(roots, &tsrzl::program::ProgramOptions::default());

    // Act
    let checker = Checker::new(program.files(), CheckerOptions::default());

    // Assert
    assert!(checker.globals().get("Array").is_some() && checker.diagnostics().is_empty());
}
