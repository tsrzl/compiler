use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

// Pinned TypeScript project fixture: projects/PrologueEmit/__extends.ts (project/prologueEmit.json).
// TS-Go 7.0.2 at --target es2015 emits `m.child = child;` inside the namespace IIFE.
// The project JSON requests `outFile`; the CLI also rejects ES5, so this uses the supported target.
#[test]
fn should_emit_namespace_child_assignment_given_exported_class_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("__extends.ts"),
        "namespace m {\n    export class base {}\n    export class child extends base {}\n}",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result
            .emitted_files()
            .first()
            .expect("JavaScript is emitted for the source file")
            .text()
            .contains("m.child = child;"),
        "an exported child class should be assigned to its namespace, got {}",
        result.emitted_files()[0].text()
    );
}
