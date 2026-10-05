use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_accept_get_canonical_locales_given_es2016_target_when_checking_types() {
    // Upstream: conformance/es2016/es2016IntlAPIs.ts; its types artifact returns string[].
    // Oracle: Intl.getCanonicalLocales accepts a string and returns an assignable string array.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("intl.ts"),
        "const locales: string[] = Intl.getCanonicalLocales('EN-US');",
    )
    .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2016));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}
