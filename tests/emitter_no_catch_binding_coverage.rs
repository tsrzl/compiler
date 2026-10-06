use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_preserve_omitted_catch_binding_given_es2019_target_when_emitting_javascript() {
    // TypeScript 7.0.2 case: conformance/emitter/es2019/noCatchBinding/emitter.noCatchBinding.es2019.ts.
    // Its ES2019 JavaScript baseline preserves a catch clause without a binding.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("emitter.noCatchBinding.es2019.ts"),
        "function f() { try {} catch {} }",
    )
    .expect("a TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2019);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert!(
        result.emitted_files()[0].text().contains("catch {"),
        "ES2019 emit should preserve an omitted catch binding: {}",
        result.emitted_files()[0].text()
    );
}
