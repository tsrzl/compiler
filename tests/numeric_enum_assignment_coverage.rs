use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_accept_numeric_literal_given_numeric_enum_annotation_when_checking_types() {
    // TypeScript 7.0.2 case: conformance/types/primitives/enum/validEnumAssignments.ts.
    // With target ES2015, the reference accepts assigning a numeric literal to a numeric enum.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("validEnumAssignments.ts"),
        "enum Level { Low = 1 } const value: Level = 1;",
    )
    .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert_eq!(
        result.diagnostics(),
        [],
        "a numeric literal is assignable to a numeric enum type"
    );
}
