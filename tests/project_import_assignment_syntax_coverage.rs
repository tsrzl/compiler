use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned project cases: projects/outputdir_module_subfolder/test.ts, projects/outputdir_module_simple/test.ts, and projects/declarations_GlobalImport/useModule.ts; TS-Go parses import-equals before semantic diagnostics.
#[test]
fn should_parse_import_assignment_given_require_import_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(Path::new("test.ts"), "import m1 = require(\"ref/m1\");")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
