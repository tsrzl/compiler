use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_invalid_interface_extension_given_union_base_when_checking_types() {
    // TypeScript 7.0.2 case: conformance/interfaces/interfaceDeclarations/interfaceExtendsObjectIntersectionErrors.ts
    // Oracle expectation: the I30 extension of U reports TS2312.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("interfaceExtendsObjectIntersectionErrors.ts"),
        "type U = { a: number } | { b: string }; interface I30 extends U { x: string }",
    )
    .expect("the TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2312),
        "an interface cannot extend a union type"
    );
}
