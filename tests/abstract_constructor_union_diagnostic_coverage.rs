use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_ts2511_given_union_with_abstract_constructor_when_checking_types() {
    // Pinned fixture: compiler/abstractClassUnionInstantiation.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("abstract-constructor-union.ts"),
        "abstract class Abstract {}\nclass Concrete {}\ndeclare const ctor: typeof Abstract | typeof Concrete;\nnew ctor();",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().iter().any(|diagnostic| {
            diagnostic.code() == 2511 && diagnostic.message().contains("abstract class")
        }),
        "instantiating a constructor union containing an abstract class should report TS2511, got {:?}",
        result.diagnostics()
    );
}
