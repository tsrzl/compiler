use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;

// Pinned project case: projects/privacyCheck-SimpleReference/test.ts.
// The original case uses bare imports; this relative-import reduction isolates
// declaration emit. TS-Go 7.0.2 preserves both the alias and qualified type.
#[test]
fn should_preserve_import_equals_alias_given_exported_imported_type_when_emitting_declarations() {
    // Arrange
    let consumer = SourceFile::from_path(
        Path::new("projects/privacyCheck-SimpleReference/use.ts"),
        "import lib = require('./lib');\nexport const value: lib.Item = new lib.Item();\n",
    )
    .expect("the importing source path has a supported source kind");
    let dependency = SourceFile::from_path(
        Path::new("projects/privacyCheck-SimpleReference/lib.ts"),
        "export class Item {}\n",
    )
    .expect("the dependency source path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2015)
        .with_module(ModuleKind::CommonJs)
        .with_declaration(true)
        .with_emit_declaration_only(true);

    // Act
    let result = Compiler::with_options(options).compile_sources([consumer, dependency]);

    // Assert
    let declaration = result
        .emitted_files()
        .iter()
        .find(|file| file.path().file_name().and_then(|name| name.to_str()) == Some("use.d.ts"))
        .map(|file| file.text())
        .unwrap_or_default();
    assert_eq!(
        declaration,
        "import lib = require('./lib');\nexport declare const value: lib.Item;\n"
    );
}
