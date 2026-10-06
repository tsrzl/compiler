use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_narrow_nullable_string_given_assertion_function_call_when_checking_types() {
    // Arrange
    // Pinned fixture: conformance/controlFlow/assertionTypePredicates1.ts.
    let source = SourceFile::from_path(
        Path::new("assertion.ts"),
        r"function assert(value: unknown): asserts value {}
function getLength(value: string | undefined) {
    assert(value);
    return value.length;
}",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(
        result.diagnostics(),
        [],
        "the assertion call should narrow the nullable string before property access"
    );
}
