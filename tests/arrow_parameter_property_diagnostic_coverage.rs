use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_ts2369_given_parameter_property_in_arrow_function_when_checking_types() {
    // Pinned fixture: compiler/ArrowFunctionExpression1.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("arrow-parameter-property.ts"),
        "var value = (public name: string) => {};",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2369),
        "the arrow parameter property should report TS2369, got {:?}",
        result.diagnostics()
    );
}
