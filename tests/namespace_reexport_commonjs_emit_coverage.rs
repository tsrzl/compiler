use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_emit_namespace_reexport_given_commonjs_module_when_compiling_sources() {
    // Arrange
    // Pinned fixture: conformance/externalModules/typeOnly/exportNamespace2.ts.
    let module = SourceFile::from_path(Path::new("module.ts"), "export const value = 1;")
        .expect("a TypeScript path has a supported source kind");
    let barrel =
        SourceFile::from_path(Path::new("barrel.ts"), "export * as api from \"./module\";")
            .expect("a TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2015).with_module(ModuleKind::CommonJs);

    // Act
    let result = Compiler::with_options(options).compile_sources([module, barrel]);

    // Assert
    assert_eq!(result.diagnostics(), []);
    let barrel_output = result
        .emitted_files()
        .iter()
        .find(|file| file.path() == Path::new("barrel.js"))
        .expect("the namespace re-export source emits barrel.js");
    assert!(
        barrel_output
            .text()
            .contains("exports.api = __importStar(require(\"./module\"));"),
        "CommonJS output should publish the imported module namespace: {}",
        barrel_output.text()
    );
}
