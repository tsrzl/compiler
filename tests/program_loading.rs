use tsrzl::parser::{ParseOptions, ParsedSourceFile, ScriptKind, parse_source_file};
use tsrzl::program::{Program, ProgramOptions};

fn root(text: &str) -> ParsedSourceFile {
    parse_source_file(&ParseOptions::new("/app/main.ts", ScriptKind::Ts), text)
}

fn file_names(program: &Program) -> Vec<&str> {
    program
        .files()
        .iter()
        .map(|file| file.parsed().options().file_name())
        .collect()
}

fn options(target: &str) -> ProgramOptions {
    ProgramOptions {
        target: target.to_owned(),
        ..ProgramOptions::default()
    }
}

#[test]
fn should_order_libs_by_priority_before_root_given_es5_target_when_loading_program() {
    // Arrange
    let roots = vec![root("let value = 1;")];

    // Act
    let program = Program::load(roots, &options("es5"));

    // Assert
    assert_eq!(
        file_names(&program),
        [
            "bundled:///libs/lib.d.ts",
            "bundled:///libs/lib.es5.d.ts",
            "bundled:///libs/lib.es2015.d.ts",
            "bundled:///libs/lib.dom.d.ts",
            "bundled:///libs/lib.webworker.importscripts.d.ts",
            "bundled:///libs/lib.scripthost.d.ts",
            "bundled:///libs/lib.es2015.core.d.ts",
            "bundled:///libs/lib.es2015.collection.d.ts",
            "bundled:///libs/lib.es2015.generator.d.ts",
            "bundled:///libs/lib.es2015.iterable.d.ts",
            "bundled:///libs/lib.es2015.promise.d.ts",
            "bundled:///libs/lib.es2015.proxy.d.ts",
            "bundled:///libs/lib.es2015.reflect.d.ts",
            "bundled:///libs/lib.es2015.symbol.d.ts",
            "bundled:///libs/lib.es2015.symbol.wellknown.d.ts",
            "bundled:///libs/lib.es2018.asynciterable.d.ts",
            "bundled:///libs/lib.decorators.d.ts",
            "bundled:///libs/lib.decorators.legacy.d.ts",
            "/app/main.ts",
        ]
    );
}

#[test]
fn should_load_only_roots_given_no_lib_when_loading_program() {
    // Arrange
    let roots = vec![root("let value = 1;")];
    let options = ProgramOptions {
        no_lib: true,
        ..options("esnext")
    };

    // Act
    let program = Program::load(roots, &options);

    // Assert
    assert_eq!(file_names(&program), ["/app/main.ts"]);
}

#[test]
fn should_load_explicit_lib_without_dom_given_lib_option_when_loading_program() {
    // Arrange
    let roots = vec![root("let value = 1;")];
    let options = ProgramOptions {
        lib: Some(vec!["es2015.core".to_owned()]),
        ..options("esnext")
    };

    // Act
    let program = Program::load(roots, &options);

    // Assert
    assert_eq!(
        file_names(&program),
        ["bundled:///libs/lib.es2015.core.d.ts", "/app/main.ts"]
    );
}

#[test]
fn should_load_referenced_lib_given_lib_reference_directive_when_loading_program() {
    // Arrange
    let roots = vec![root(
        "/// <reference lib=\"es2015.promise\" />\nlet value = 1;",
    )];
    let options = ProgramOptions {
        no_lib: false,
        lib: Some(Vec::new()),
        ..options("esnext")
    };

    // Act
    let program = Program::load(roots, &options);

    // Assert
    assert_eq!(
        file_names(&program),
        ["bundled:///libs/lib.es2015.promise.d.ts", "/app/main.ts"]
    );
}

#[test]
fn should_report_ts2727_given_misspelled_lib_reference_when_loading_program() {
    // Arrange
    let roots = vec![root("/// <reference lib=\"es2015.promis\" />")];

    // Act
    let program = Program::load(roots, &options("esnext"));

    // Assert
    let texts: Vec<_> = program
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.diagnostic().text())
        .collect();
    assert_eq!(
        texts,
        ["Cannot find lib definition for 'es2015.promis'. Did you mean 'es2015.promise'?"]
    );
}

#[test]
fn should_mark_lib_files_as_declaration_files_given_default_lib_when_loading_program() {
    // Arrange
    let roots = vec![root("let value = 1;")];

    // Act
    let program = Program::load(roots, &options("es5"));

    // Assert
    assert!(program.files()[0].parsed().is_declaration_file());
}
