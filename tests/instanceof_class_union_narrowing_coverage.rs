use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_narrow_class_union_given_instanceof_guard_when_checking_types() {
    // Arrange
    // Pinned fixture: conformance/controlFlow/controlFlowInstanceofExtendsFunction.ts.
    let source = SourceFile::from_path(
        Path::new("instanceof.ts"),
        "class X { why(): void {} }\n\
         function use(value: X | number) {\n\
             if (value instanceof X) {\n\
                 value.why();\n\
             }\n\
         }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(
        result.diagnostics(),
        [],
        "the instanceof guard should narrow the union to class X"
    );
}
