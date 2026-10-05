use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_lower_using_declaration_given_es2022_target_when_emitting_javascript() {
    // Upstream: conformance/statements/VariableStatements/usingDeclarations/usingDeclarations.1.ts.
    // Its target=ES2022, module=ESNext, lib=ESNext output injects disposal helpers and a finally block.
    // Arrange
    let source = SourceFile::from_path(Path::new("using.ts"), "using resource = null; export {};")
        .expect("a TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2022).with_module(ModuleKind::EsNext);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    let javascript = result.emitted_files()[0].text();
    assert!(
        javascript.contains("__addDisposableResource"),
        "ES2022 using output should register disposable resources, got: {javascript}"
    );
    assert!(javascript.contains("__disposeResources"));
    assert!(javascript.contains("finally"));
}
