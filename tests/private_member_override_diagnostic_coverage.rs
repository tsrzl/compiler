use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_reject_private_member_redeclaration_given_derived_class_override_when_checking_types() {
    // Arrange
    // Pinned fixture: compiler/inheritanceGrandParentPrivateMemberCollision.ts.
    // Its target=es2015, strict=false baseline reports TS2415.
    let source = SourceFile::from_path(
        Path::new("inheritanceGrandParentPrivateMemberCollision.ts"),
        "class A { private myMethod() {} }\nclass B extends A {}\nclass C extends B { private myMethod() {} }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2415),
        "expected TS2415 for redeclaring an inherited private member, got {:?}",
        result.diagnostics()
    );
}
