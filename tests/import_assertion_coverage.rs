use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_preserve_dynamic_import_attributes_given_es_module_output_when_emitting_javascript() {
    // Upstream: conformance/importAssertion/importAssertion1.ts (module=esnext).
    // Oracle: TypeScript-Go preserves the second import() argument and its `with` object.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("dynamic-import.ts"),
        "const pending = import('./0', { with: { type: \"json\" } });",
    )
    .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(
        CompilerOptions::new(ScriptTarget::Latest).with_module(ModuleKind::EsNext),
    );

    // Act
    let result = compiler.compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert!(
        result.emitted_files()[0]
            .text()
            .contains("import('./0', { with: { type: \"json\" } })")
    );
}
