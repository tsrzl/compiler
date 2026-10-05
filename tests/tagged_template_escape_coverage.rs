use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_invalid_hex_escape_given_untagged_template_when_compiling() {
    // Upstream: conformance/es2018/invalidTaggedTemplateEscapeSequences.ts (target=es2015).
    // Oracle: TypeScript-Go reports TS1125 for an invalid hex escape in an untagged template.
    // Arrange
    let source = SourceFile::from_path(Path::new("template.ts"), "const value = `\\xG`;")
        .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 1125),
        "expected TS1125 for the invalid template escape, got {:?}",
        result.diagnostics()
    );
}
