use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_emit_type_predicate_given_exported_function_when_emitting_declarations() {
    // Pinned TypeScript case: conformance/declarationEmit/typePredicates/declarationEmitIdentifierPredicates01.ts.
    // TS-Go 7.0.2 emits: export declare function f(x: any): x is number;
    // The body is reduced to a boolean literal to isolate the return predicate syntax.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("predicate.ts"),
        "export function f(x: any): x is number { return true; }",
    )
    .expect("the TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2015)
        .with_module(ModuleKind::CommonJs)
        .with_declaration(true);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    assert!(
        result.diagnostics().is_empty(),
        "the valid type predicate should compile: {:?}",
        result.diagnostics()
    );
    let declaration = result
        .emitted_files()
        .iter()
        .find(|file| {
            file.path().file_name().and_then(|name| name.to_str()) == Some("predicate.d.ts")
        })
        .expect("declaration output is emitted")
        .text();
    assert_eq!(
        declaration,
        "export declare function f(x: any): x is number;\n"
    );
}

#[test]
fn should_preserve_folded_string_enum_value_given_constant_initializer_when_emitting_declarations()
{
    // Pinned TypeScript 7.0.2 case: conformance/enums/enumConstantMemberWithStringEmitDeclaration.ts.
    // TS-Go folds the member value and preserves it in the exported declaration.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("string-enum.ts"),
        "export enum Value { Name = \"ab\" + \"cd\" }",
    )
    .expect("the TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2015).with_declaration(true);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    assert!(
        result.diagnostics().is_empty(),
        "the string enum declaration is valid: {:?}",
        result.diagnostics()
    );
    let declaration = result
        .emitted_files()
        .iter()
        .find(|file| {
            file.path().file_name().and_then(|name| name.to_str()) == Some("string-enum.d.ts")
        })
        .expect("the string enum declaration is emitted");
    assert_eq!(
        declaration.text(),
        "export declare enum Value {\n    Name = \"abcd\"\n}\n"
    );
}
