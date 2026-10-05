use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_incompatible_union_property_given_object_literal_when_checking_types() {
    // TypeScript 7.0.2 case: conformance/types/union/contextualTypeWithUnionTypeObjectLiteral.ts.
    // Oracle expectation: a string | number property cannot satisfy either union member (TS2322).
    // Arrange
    let source = SourceFile::from_path(
        Path::new("contextualTypeWithUnionTypeObjectLiteral.ts"),
        "declare let strOrNumber: string | number; const value: { prop: string } | { prop: number } = { prop: strOrNumber };",
    )
    .expect("the TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2322),
        "a union target must reject an object property that is not assignable to any union member"
    );
}
