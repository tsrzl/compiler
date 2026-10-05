use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_accept_regexp_escape_given_es2025_regexp_library_when_checking_types() {
    // Upstream: conformance/es2025/regExpEscape.ts (lib=es2024,es2025.regexp, strict=true).
    // Oracle: RegExp.escape accepts a string and returns a string; the source has no diagnostics.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("regexp-escape.ts"),
        "const regExp = new RegExp(RegExp.escape(\"foo.bar\")); regExp.test(\"foo.bar\");",
    )
    .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Latest));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}
