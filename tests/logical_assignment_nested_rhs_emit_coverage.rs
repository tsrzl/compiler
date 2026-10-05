use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_lower_nullish_assignment_with_nullish_rhs_given_es2015_target_when_emitting_javascript() {
    // Pinned TypeScript-Go fixture: conformance/esnext/logicalAssignment/logicalAssignment11.ts.
    // The target=es2015 .js baseline lowers `e ??= x ?? "x"` to a nullish check and assignment.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("logicalAssignment11.ts"),
        "let x: string | undefined;\nlet e: string | undefined;\ne ??= x ?? \"x\";",
    )
    .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);
    let javascript = result
        .emitted_files()
        .first()
        .expect("JavaScript is emitted for the source")
        .text();

    // Assert
    assert!(
        javascript.contains(
            "e !== null && e !== void 0 ? e : (e = x !== null && x !== void 0 ? x : \"x\")"
        ),
        "ES2015 output should lower the nested nullish assignment like the pinned baseline: {javascript}"
    );
}
