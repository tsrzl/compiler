use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

// Pinned TypeScript 7.0.2 case: conformance/types/keyof/keyofAndIndexedAccess.ts.
// TS-Go narrows `keyof Person` to the declared property-name union.
#[test]
fn should_reject_unknown_key_given_keyof_type_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("main.ts"),
        "interface Person { name: string; } const key: keyof Person = \"missing\";",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(
        result.diagnostics().len(),
        1,
        "unexpected diagnostics: {:?}",
        result.diagnostics()
    );
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type '\"missing\"' is not assignable to type '\"name\"'."
    );
}
