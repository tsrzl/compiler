use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_nullish_rhs_possibly_null_given_nullable_left_operand_when_checking_types() {
    // Arrange
    // Pinned fixture: conformance/expressions/nullishCoalescingOperator/nullishCoalescingOperator4.ts.
    // Its target=es2015, strict=true baseline reports TS18049.
    let source = SourceFile::from_path(
        Path::new("nullishCoalescingOperator4.ts"),
        "function use(a1: string | undefined | null) { const aa1 = a1 ?? a1.toLowerCase(); }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(
        CompilerOptions::new(ScriptTarget::Es2015).with_strict_null_checks(true),
    );

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 18049),
        "expected TS18049 for a nullish-coalescing right operand that may be null or undefined, got {:?}",
        result.diagnostics()
    );
}
