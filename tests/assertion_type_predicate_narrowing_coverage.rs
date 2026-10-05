use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_narrow_unknown_given_assertion_type_predicate_call_when_checking_types() {
    // Arrange
    // Pinned fixture: conformance/controlFlow/assertionTypePredicates1.ts.
    let source = SourceFile::from_path(
        Path::new("assertion.ts"),
        r#"function assertString(value: unknown): asserts value is string {}
function getLength(value: unknown) {
    assertString(value);
    return value.length;
}"#,
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(
        result.diagnostics(),
        [],
        "the assertion type predicate should narrow unknown before property access"
    );
}
