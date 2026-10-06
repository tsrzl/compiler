use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_ts1248_given_const_modifier_on_class_field_when_checking_types() {
    // Pinned fixture: compiler/ClassDeclarationWithInvalidConstOnPropertyDeclaration.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("const-class-field.ts"),
        "class AtomicNumbers {\n  static const H = 1;\n}",
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
            .any(|diagnostic| diagnostic.code() == 1248),
        "a class field with the const modifier should report TS1248, got {:?}",
        result.diagnostics()
    );
}
