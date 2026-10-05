use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_retain_following_class_given_unexpected_top_level_brace_when_parsing() {
    // Upstream: conformance/parser/ecmascript5/ErrorRecovery/SourceUnits/parserErrorRecovery_SourceUnit1.ts
    // The pinned baseline reports TS1128 for the extra brace and still emits class D.
    // Arrange
    let source = SourceFile::from_path(Path::new("recovery.ts"), "class C {\n}\n}\nclass D {\n}")
        .expect("the TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);
    let class_names: Vec<_> = syntax_tree
        .program()
        .statements()
        .iter()
        .filter_map(|statement| statement.as_class_declaration())
        .map(|declaration| declaration.name())
        .collect();

    // Assert
    assert_eq!(class_names, ["C", "D"]);
}
