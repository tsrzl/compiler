use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_ts2427_given_interface_named_string_when_checking_types() {
    // Pinned fixture: compiler/InterfaceDeclaration8.ts.
    // Arrange
    let source = SourceFile::from_path(Path::new("reserved-interface.ts"), "interface string {}")
        .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().iter().any(|diagnostic| {
            diagnostic.code() == 2427
                && diagnostic
                    .message()
                    .contains("Interface name cannot be 'string'")
        }),
        "an interface named with the predefined type string should report TS2427, got {:?}",
        result.diagnostics()
    );
}
