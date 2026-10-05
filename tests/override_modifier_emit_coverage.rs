use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

// Pinned TypeScript 7.0.2 case: conformance/override/override1.ts.
// TS-Go erases `override` from the emitted JavaScript method.
#[test]
fn should_erase_override_modifier_given_overriding_method_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("main.ts"),
        "class Base { area() { return 0; } }\nclass Shape extends Base { override area() { return 1; } }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    let javascript = result.emitted_files()[0].text();
    assert!(
        javascript.contains("area() { return 1; }"),
        "{javascript:?}"
    );
    assert!(!javascript.contains("override"), "{javascript:?}");
}
