use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_lower_auto_accessor_given_es2022_target_when_emitting_javascript() {
    // Upstream: conformance/esDecorators/classDeclaration/esDecorators-classDeclaration-commentPreservation.ts.
    // TS-Go 7.0.2 lowers `accessor z = 1` to a private backing field and accessors.
    // Arrange
    let source =
        SourceFile::from_path(Path::new("auto-accessor.ts"), "class C { accessor z = 1; }")
            .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2022));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    let javascript = result.emitted_files()[0].text();
    assert!(
        javascript.contains("#z_accessor_storage = 1;"),
        "ES2022 auto-accessor should lower to backing storage, got: {javascript}"
    );
    assert!(javascript.contains("get z()"));
    assert!(javascript.contains("set z(value)"));
}
