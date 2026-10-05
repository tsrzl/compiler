use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

// Pinned TypeScript 7.0.2 case: conformance/classes/classStaticBlock/classStaticBlock1.ts.
// TS-Go preserves a static initialization block when targeting ES2022.
#[test]
fn should_emit_static_initialization_block_given_es2022_target_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("main.ts"),
        "class Registry { static ready = false; static { Registry.ready = true; } }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2022));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert!(
        result.emitted_files()[0]
            .text()
            .contains("static { Registry.ready = true; }"),
        "the static initialization block is preserved"
    );
}
