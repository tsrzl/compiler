use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_parse_for_await_of_given_async_function_body_when_building_syntax_tree() {
    // Oracle: TS-Go 7.0.2 parser.forAwait.es2018.ts, accepted subfile
    // inAsyncFunctionWithDeclIsOk.ts; target ES2018 and lib ESNext produce no errors.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("inAsyncFunctionWithDeclIsOk.ts"),
        "async function f7() {\n    let y: any;\n    for await (const x of y) {\n    }\n}",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
