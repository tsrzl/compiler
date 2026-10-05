use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_accept_zero_argument_atomics_pause_call_given_esnext_library_when_checking_types() {
    // Upstream: conformance/esnext/esnextSharedMemory.ts (`Atomics.pause();`).
    // TS-Go 7.0.2 with --target es2015 --lib esnext assigns `void` and declares
    // `Atomics.pause: (n?: number) => void` in esnextSharedMemory.types.
    // Arrange
    let source = SourceFile::from_path(Path::new("esnextSharedMemory.ts"), "Atomics.pause();")
        .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.code() != 2304),
        "Atomics should be available from the ESNext library, got {:?}",
        result.diagnostics()
    );
}
