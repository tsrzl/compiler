use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_inline_const_enum_member_given_property_access_when_emitting_javascript() {
    // TypeScript 7.0.2 case: conformance/constEnums/constEnumPropertyAccess1.ts.
    // With target ES2015, a constant enum member access emits its numeric value.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("const-enum.ts"),
        "const enum Answer { FortyTwo = 42 } const answer = Answer.FortyTwo;",
    )
    .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert!(
        result.emitted_files()[0]
            .text()
            .contains("const answer = 42 /* Answer.FortyTwo */;")
    );
}
