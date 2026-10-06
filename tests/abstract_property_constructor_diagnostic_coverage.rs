use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_ts2715_given_abstract_property_access_in_constructor_when_checking_types() {
    // Pinned fixture: compiler/abstractPropertyInConstructor.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("abstract-property.ts"),
        "abstract class Base {\n  abstract value: string;\n  constructor() { this.value; }\n}",
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
            .any(|diagnostic| diagnostic.code() == 2715),
        "accessing an abstract property from its constructor should report TS2715, got {:?}",
        result.diagnostics()
    );
}
