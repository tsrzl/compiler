use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_narrow_discriminated_union_given_switch_case_when_checking_types() {
    // Arrange
    // Pinned fixture: conformance/controlFlow/exhaustiveSwitchStatements1.ts.
    let source = SourceFile::from_path(
        Path::new("shape.ts"),
        r#"interface Square { kind: "square"; size: number; }
interface Circle { kind: "circle"; radius: number; }
type Shape = Square | Circle;
function area(shape: Shape): number {
    switch (shape.kind) {
        case "square": return shape.size;
        case "circle": return shape.radius;
    }
}"#,
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(
        result.diagnostics(),
        [],
        "each switch case should narrow Shape to its discriminant variant"
    );
}
