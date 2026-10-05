use std::fs;
use std::path::Path;
use std::process::Command;

use tsrzl::compiler::{Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_erase_ambient_function_given_typed_call_when_emitting_javascript() {
    // Upstream: conformance/ambient/ambientDeclarations.ts
    // Arrange
    let source = SourceFile::from_path(
        Path::new("ambient.ts"),
        "declare function parseCount(input: string): number;\nconst count: number = parseCount(\"42\");",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);
    let javascript = result
        .emitted_files()
        .iter()
        .find(|file| {
            file.path()
                .file_name()
                .is_some_and(|name| name == "ambient.js")
        })
        .expect("JavaScript output is emitted");

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert!(!javascript.text().contains("declare function parseCount"));
    assert!(
        javascript
            .text()
            .contains("const count = parseCount(\"42\");")
    );
}

#[test]
fn should_resolve_ambient_module_given_named_import_when_compiling_sources() {
    // Upstream: conformance/ambient/ambientDeclarationsExternal.ts
    // Arrange
    let declarations = SourceFile::from_path(
        Path::new("decls.d.ts"),
        "declare module \"equ\" { export const value: number; }",
    )
    .expect("a declaration path has a supported source kind");
    let consumer = SourceFile::from_path(
        Path::new("consumer.ts"),
        "import { value } from \"equ\"; const answer: number = value;",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(
        CompilerOptions::new(ScriptTarget::Es2015).with_module(ModuleKind::CommonJs),
    );

    // Act
    let result = compiler.compile_sources([declarations, consumer]);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_preserve_symbol_computed_property_given_symbol_key_when_emitting_javascript() {
    // Upstream: conformance/Symbols/ES5SymbolProperty1.ts (ES2015 configuration)
    // Arrange
    let source = SourceFile::from_path(
        Path::new("symbol.ts"),
        "interface SymbolConstructor { foo: string; }\nvar Symbol: SymbolConstructor;\nvar obj = { [Symbol.foo]: 0 };\nobj[Symbol.foo];",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);
    let javascript = result.emitted_files()[0].text();

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert!(javascript.contains("[Symbol.foo]: 0"));
}

#[test]
fn should_emit_export_assignment_given_commonjs_module_when_emitting_javascript() {
    // Upstream: conformance/externalModules/exportAssignTypes.ts
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "const answer = 42; export = answer;",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(
        CompilerOptions::new(ScriptTarget::Es2015).with_module(ModuleKind::CommonJs),
    );

    // Act
    let result = compiler.compile(source);
    let javascript = result.emitted_files()[0].text();

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert!(javascript.contains("module.exports = answer;"));
}

#[test]
fn should_load_type_reference_given_type_roots_configuration_when_running_compiler_cli() {
    // Upstream: conformance/references/library-reference-1.ts
    // Arrange
    let directory = std::env::temp_dir().join(format!(
        "tsrzl-binding-type-reference-{}",
        std::process::id()
    ));
    let output_directory = directory.join("out");
    fs::create_dir_all(directory.join("types/widgets"))
        .expect("the type package directory can be created");
    fs::write(
        directory.join("types/widgets/index.d.ts"),
        "declare var widget: { enabled: boolean };",
    )
    .expect("the type declaration can be written");
    fs::write(
        directory.join("main.ts"),
        "/// <reference types=\"widgets\" />\nconst enabled: boolean = widget.enabled;",
    )
    .expect("the TypeScript input can be written");
    fs::write(
        directory.join("tsconfig.json"),
        "{\"compilerOptions\":{\"target\":\"ES2015\",\"typeRoots\":[\"./types\"],\"outDir\":\"./out\"},\"files\":[\"main.ts\"]}",
    )
    .expect("the project configuration can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--project")
        .arg(directory.join("tsconfig.json"))
        .output()
        .expect("the compiler CLI can be started");
    let emitted = fs::read_to_string(output_directory.join("main.js")).ok();
    fs::remove_dir_all(&directory).expect("the test directory can be removed");

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    assert!(
        emitted
            .as_deref()
            .is_some_and(|javascript| javascript.contains("const enabled = widget.enabled;"))
    );
}

#[test]
fn should_report_jsdoc_argument_mismatch_given_number_parameter_when_checking_javascript() {
    // Upstream: conformance/jsdoc/checkJsdocParamTag1.ts (allowJs and checkJs configuration)
    // Arrange
    let source = SourceFile::from_path(
        Path::new("checked.js"),
        "// @ts-check\n/** @param {number} value */\nfunction use(value) {}\nuse(\"bad\");",
    )
    .expect("a JavaScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2345),
        "expected the incompatible argument diagnostic, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_emit_jsdoc_parameter_type_given_annotated_javascript_function_when_emitting_declarations()
{
    // Upstream: conformance/jsdoc/declarations/jsDeclarationsFunctionJSDoc.ts
    // Arrange
    let source = SourceFile::from_path(
        Path::new("greet.js"),
        "/** @param {string} name */\nexport function greet(name) { return \"Hello \" + name; }",
    )
    .expect("a JavaScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2015)
        .with_declaration(true)
        .with_emit_declaration_only(true);

    // Act
    let result = Compiler::with_options(options).compile(source);
    let declaration = result
        .emitted_files()
        .iter()
        .find(|file| {
            file.path()
                .file_name()
                .is_some_and(|name| name == "greet.d.ts")
        })
        .expect("declaration output is emitted");

    // Assert
    assert!(
        declaration
            .text()
            .contains("export declare function greet(name: string): string;")
    );
}

#[test]
fn should_merge_interface_members_given_duplicate_declarations_when_checking_types() {
    // Upstream: conformance/interfaces/declarationMerging/mergeTwoInterfaces.ts
    // Arrange
    let source = SourceFile::from_path(
        Path::new("merge.ts"),
        "interface Options { left: string; }\ninterface Options { right: number; }\nconst options: Options = { left: \"ready\", right: 1 };",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}
