use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_report_incompatible_overload_implementation_given_function_signature_when_checking_types()
{
    // TypeScript 7.0.2 case: conformance/functions/functionOverloadCompatibilityWithVoid01.ts
    // Oracle expectation: the number-returning overload is incompatible with its void implementation (TS2394).
    // Arrange
    let source = SourceFile::from_path(
        Path::new("functionOverloadCompatibilityWithVoid01.ts"),
        "function f(x: string): number; function f(x: string): void { return; }",
    )
    .expect("the TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2394),
        "an overload signature must be compatible with its implementation"
    );
}

#[test]
fn should_parse_function_overload_signature_given_following_implementation_when_building_syntax_tree()
 {
    // TypeScript 7.0.2 case: conformance/functions/functionOverloadCompatibilityWithVoid02.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("functionOverloadCompatibilityWithVoid02.ts"),
        "function f(x: string): void; function f(x: string): number { return 0; }",
    )
    .expect("the TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

#[test]
fn should_accept_void_overload_given_value_returning_implementation_when_checking_types() {
    // TypeScript 7.0.2 case: conformance/functions/functionOverloadCompatibilityWithVoid02.ts.
    // Oracle expectation: a void overload may use an implementation that returns a value.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("functionOverloadCompatibilityWithVoid02.ts"),
        "function f(x: string): void; function f(x: string): number { return 0; }",
    )
    .expect("the TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2015);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}
