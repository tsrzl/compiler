use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_invalid_variable_decorator_given_decorated_variable_when_compiling() {
    // TypeScript 7.0.2 case: conformance/decorators/invalid/decoratorOnVar.ts.
    // With target ES2015, the oracle reports TS1206 for a decorator on a variable.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("decoratorOnVar.ts"),
        "declare function dec<T>(target: T): T; @dec var value: number;",
    )
    .expect("the TypeScript path has a supported source kind");

    // Act
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));
    let result = compiler.compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 1206),
        "decorators cannot be applied to variable declarations"
    );
}
