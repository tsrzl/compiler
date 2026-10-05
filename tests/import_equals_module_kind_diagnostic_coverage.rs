use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;

// Pinned project case: projects/VisibilityOfCrosssModuleTypeUsage/commands.ts.
// TS-Go 7.0.2 reports TS1202 for this import assignment with --module es2015.
#[test]
fn should_report_import_assignment_given_ecmascript_module_when_compiling_sources() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("commands.ts"),
        "import server = require(\"server\");",
    )
    .expect("a TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Latest).with_module(ModuleKind::Es2015);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 1202),
        "expected TS1202 for an import assignment in an ECMAScript module, got {:?}",
        result.diagnostics()
    );
}
