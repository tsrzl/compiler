use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_emit_shorthand_module_declaration_given_declaration_option_when_emitting() {
    // Pinned TypeScript 7.0.2 case: conformance/ambient/ambientShorthand_declarationEmit.ts.
    // TS-Go preserves the shorthand ambient module in the generated declaration file.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("ambientShorthand_declarationEmit.ts"),
        "declare module \"foo\";",
    )
    .expect("the TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2015).with_declaration(true);
    let compiler = Compiler::with_options(options);

    // Act
    let result = compiler.compile(source);

    // Assert
    let declaration = result
        .emitted_files()
        .iter()
        .find(|file| {
            file.path().file_name().and_then(|name| name.to_str())
                == Some("ambientShorthand_declarationEmit.d.ts")
        })
        .expect("the declaration output is emitted");
    assert_eq!(declaration.text(), "declare module \"foo\";\n");
}
