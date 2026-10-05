use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_reserved_namespace_name_given_declare_namespace_when_compiling_sources() {
    // Pinned TypeScript 7.0.2 case: conformance/ambient/ambientModuleDeclarationWithReservedIdentifierInDottedPath2.ts.
    // TS-Go reports TS2819 because a namespace cannot be named debugger.
    // Arrange
    let source = SourceFile::from_path(Path::new("ambient.ts"), "declare namespace debugger {}")
        .expect("the ambient namespace path has a supported source kind");
    let compiler = Compiler::with_options(
        CompilerOptions::new(ScriptTarget::Es2015).with_module(ModuleKind::CommonJs),
    );

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2819),
        "expected TS2819 for the reserved namespace name, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_accept_reserved_identifier_given_dotted_namespace_segment_when_compiling_sources() {
    // Pinned TypeScript 7.0.2 case: conformance/ambient/ambientModuleDeclarationWithReservedIdentifierInDottedPath.ts.
    // TS-Go accepts debugger as an intermediate name in a dotted namespace path.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("ambient.ts"),
        "declare namespace chrome.debugger { var tabId: number; }\nconst tabId: number = chrome.debugger.tabId;",
    )
    .expect("the ambient namespace path has a supported source kind");
    let compiler = Compiler::with_options(
        CompilerOptions::new(ScriptTarget::Es2015).with_module(ModuleKind::CommonJs),
    );

    // Act
    let result = compiler.compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}
