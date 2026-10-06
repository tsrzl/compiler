use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_lower_instance_field_given_es2015_target_when_emitting_javascript() {
    // Pinned fixture: conformance/classes/propertyMemberDeclarations/instanceMemberInitialization.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("instance-field.ts"),
        "class Example { value = 1; }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    let javascript = result.emitted_files()[0].text();
    assert!(javascript.contains("constructor()"), "got {javascript:?}");
    assert!(javascript.contains("this.value = 1;"), "got {javascript:?}");
    assert!(!javascript.contains("value = 1;"), "got {javascript:?}");
}

#[test]
fn should_lower_static_field_given_es2015_target_when_emitting_javascript() {
    // Pinned fixture: conformance/classes/propertyMemberDeclarations/staticMemberInitialization.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("static-field.ts"),
        "class Example { static value = 1; }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    let javascript = result.emitted_files()[0].text();
    assert!(
        !javascript.contains("static value = 1;"),
        "got {javascript:?}"
    );
    assert!(
        javascript.contains("Example.value = 1;"),
        "got {javascript:?}"
    );
}
