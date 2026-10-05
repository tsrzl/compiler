use std::path::Path;

use tsrzl::source_file::{ScriptKind, SourceFile};

#[test]
fn should_classify_declaration_file_given_d_ts_path_when_creating_source_file() {
    // Arrange
    let path = Path::new("src/model.d.ts");

    // Act
    let source_file = SourceFile::from_path(path, "export interface Model {}")
        .expect("a .d.ts file has a supported source kind");

    // Assert
    assert_eq!(source_file.script_kind(), ScriptKind::TypeScriptDeclaration);
}
