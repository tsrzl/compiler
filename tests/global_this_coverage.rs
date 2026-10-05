use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_accept_global_this_property_access_given_unknown_member_when_checking_types() {
    // Upstream: conformance/es2019/globalThisUnknown.ts, `globalThis.hi`.
    // Arrange
    let source = SourceFile::from_path(Path::new("global.ts"), "globalThis.hi;")
        .expect("a TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2015).with_strict_null_checks(false);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_accept_global_this_element_access_given_unknown_member_when_checking_types() {
    // Upstream: conformance/es2019/globalThisUnknown.ts, `globalThis['hi']`.
    // Arrange
    let source = SourceFile::from_path(Path::new("global.ts"), "globalThis['hi'];")
        .expect("a TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2015).with_strict_null_checks(false);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}
