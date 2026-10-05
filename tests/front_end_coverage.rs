use std::fs;
use std::path::Path;
use std::process::Command;

use tsrzl::compiler::{Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_emit_division_in_template_substitution_given_template_expression_when_emitting_javascript()
 {
    // Upstream: conformance/es6/templates/templateStringWithEmbeddedDivision.ts
    // Arrange
    let source = SourceFile::from_path(
        Path::new("template.ts"),
        "const value = `abc${ 1 / 1 }def`;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert!(
        result.emitted_files()[0]
            .text()
            .contains("`abc${1 / 1}def`")
    );
}

#[test]
fn should_parse_deferred_namespace_import_given_module_import_when_building_syntax_tree() {
    // Upstream: conformance/importDefer/importDeferNamespace.ts
    // Arrange
    let source = SourceFile::from_path(
        Path::new("feature.ts"),
        "import defer * as feature from \"./feature.js\"; feature.start();",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

#[test]
fn should_preserve_jsx_fragment_given_preserve_mode_when_running_compiler_cli() {
    // Upstream: conformance/jsx/tsxFragmentPreserveEmit.tsx
    // Arrange
    let directory = std::env::temp_dir().join(format!("tsrzl-jsx-preserve-{}", std::process::id()));
    let input_path = directory.join("view.tsx");
    let output_directory = directory.join("out");
    fs::create_dir_all(&directory).expect("the test directory can be created");
    fs::write(
        &input_path,
        "declare namespace JSX { interface Element {} interface IntrinsicElements { span: any; } }\nconst view = <><span>hi</span></>;",
    )
    .expect("the TSX input can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--jsx")
        .arg("preserve")
        .arg("--out-dir")
        .arg(&output_directory)
        .arg(&input_path)
        .output()
        .expect("the compiler CLI can be started");
    let emitted = fs::read_to_string(output_directory.join("view.jsx")).ok();
    fs::remove_dir_all(&directory).expect("the test directory can be removed");

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    assert_eq!(
        emitted.as_deref(),
        Some("const view = <><span>hi</span></>;\n")
    );
}

#[test]
fn should_apply_standard_class_decorator_given_decorated_class_when_emitting_javascript() {
    // Upstream: conformance/esDecorators/classDeclaration/esDecorators-classDeclaration-simpleTransformation.ts
    // Arrange
    let source = SourceFile::from_path(
        Path::new("decorated.ts"),
        "const sealed = (value: any, context: any) => value; @sealed class Example {}",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2022));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(result.emitted_files()[0].text().contains("__esDecorate("));
}

#[test]
fn should_lower_optional_property_access_given_es2015_target_when_emitting_javascript() {
    // Upstream: conformance/expressions/optionalChaining/optionalChainingInference.ts
    // Arrange
    let source = SourceFile::from_path(
        Path::new("person.ts"),
        "const person = { name: \"Ada\" }; const name = person?.name;",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.emitted_files()[0]
            .text()
            .contains("person === null || person === void 0 ? void 0 : person.name")
    );
}

#[test]
fn should_preserve_static_import_attributes_given_es_module_import_when_emitting_javascript() {
    // Upstream: conformance/importAttributes/importAttributes1.ts (module=esnext)
    // Arrange
    let dependency = SourceFile::from_path(
        Path::new("fixture/settings.ts"),
        "export default { answer: 42 };",
    )
    .expect("a TypeScript path has a supported source kind");
    let entry = SourceFile::from_path(
        Path::new("fixture/b.ts"),
        "import settings from \"./settings\" with { type: \"json\" }; export const result = settings.answer;",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(
        CompilerOptions::new(ScriptTarget::Es2015).with_module(ModuleKind::EsNext),
    );

    // Act
    let result = compiler.compile_sources([dependency, entry]);
    let entry_javascript = result
        .emitted_files()
        .iter()
        .find(|file| file.path().file_name().is_some_and(|name| name == "b.js"))
        .expect("the entry module JavaScript is emitted");

    // Assert
    assert!(
        entry_javascript
            .text()
            .contains("from \"./settings\" with { type: \"json\" }")
    );
}
