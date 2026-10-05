use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_emit_null_type_annotation_given_default_null_expression_when_emitting_declarations() {
    // Upstream: conformance/declarationEmit/exportDefaultExpressionComments.ts.
    // Its target=ES2015, module=CommonJS, declaration=true baseline emits `declare const _default: null`.
    // Arrange
    let source = SourceFile::from_path(Path::new("input.ts"), "export default null;")
        .expect("a TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2015)
        .with_module(ModuleKind::CommonJs)
        .with_declaration(true)
        .with_strict_null_checks(true);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    assert!(result.diagnostics().is_empty());
    let declaration = result
        .emitted_files()
        .iter()
        .find(|file| file.path().file_name().and_then(|name| name.to_str()) == Some("input.d.ts"))
        .expect("declaration output is emitted")
        .text();
    assert!(
        declaration.contains("declare const _default: null;"),
        "the declaration should annotate the null literal type, got: {declaration}"
    );
}
