use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_reject_number_assignment_given_destructured_constrained_generic_result_when_checking_types()
 {
    // TypeScript 7.0.2: conformance/inferFromBindingPattern.ts types baseline says x2 is string.
    // The same inferred type rejects assigning x2 to number with TS2322.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("inferFromBindingPattern.ts"),
        "declare function f2<T extends string>(): [T]; let [x2] = f2(); const numberValue: number = x2;",
    )
    .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2322),
        "destructuring should retain the constrained generic return type string"
    );
}
