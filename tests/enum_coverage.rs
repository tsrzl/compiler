use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned TypeScript input: conformance/constEnums/constEnum1.ts.
#[test]
fn should_parse_const_enum_declaration_given_const_modifier_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "const enum Answer { FortyTwo = 42 }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
    let declaration = syntax_tree.program().statements()[0]
        .as_enum_declaration()
        .expect("the statement should be an enum declaration");
    assert!(declaration.is_const());
}

#[test]
fn should_report_reserved_enum_name_given_keyword_enum_declaration_when_checking_types() {
    // Pinned TypeScript 7.0.2 case: conformance/enums/enumErrors.ts reports TS2431 for `enum any`.
    // Arrange
    let source = SourceFile::from_path(Path::new("enumErrors.ts"), "enum any {}")
        .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2431),
        "an enum cannot use the predefined type name `any`: {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_fold_string_enum_initializer_given_constant_concatenation_when_emitting_javascript() {
    // Pinned TypeScript 7.0.2 case: conformance/enums/enumConstantMemberWithString.ts.
    // TS-Go folds the constant member initializer `"ab" + "cd"` to `"abcd"`.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("string-enum.ts"),
        "enum Value { Name = \"ab\" + \"cd\" }",
    )
    .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().is_empty(),
        "{:?}",
        result.diagnostics()
    );
    let javascript = result.emitted_files()[0].text();
    assert!(
        javascript.contains("\"abcd\""),
        "a constant string enum initializer should be folded: {javascript}"
    );
}

#[test]
fn should_omit_reverse_mapping_given_computed_string_enum_member_when_emitting_javascript() {
    // Pinned TypeScript 7.0.2 case: conformance/enums/enumConstantMemberWithString.ts.
    // String-valued enum members emit forward properties without numeric reverse mappings.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("string-enum.ts"),
        "enum Value { Name = \"ab\" + \"cd\" }",
    )
    .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().is_empty(),
        "{:?}",
        result.diagnostics()
    );
    let javascript = result.emitted_files()[0].text();
    assert!(
        !javascript.contains("Value[Value[\"Name\"]"),
        "a string-valued enum member should not get a reverse mapping: {javascript}"
    );
}
