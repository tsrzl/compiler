use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_missing_jsdoc_implemented_member_given_javascript_class_when_compiling_sources() {
    // Arrange
    // Pinned TypeScript 7.0.2 case: conformance/jsdoc/jsdocImplements_interface.ts.
    // TS-Go reports TS2420 when the JSDoc-annotated class omits an interface method.
    let interface = SourceFile::from_path(Path::new("defs.d.ts"), "interface A { name: string; }")
        .expect("a declaration path has a supported source kind");
    let implementation = SourceFile::from_path(
        Path::new("a.js"),
        "// @ts-check\n/** @implements {A} */\nclass B3 {}",
    )
    .expect("a JavaScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([interface, implementation]);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2420),
        "expected TS2420 for the missing JSDoc-implemented member, got {:?}",
        result.diagnostics()
    );
}
