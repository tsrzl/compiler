use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::source_text::Utf16Offset;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_point_at_unexpected_binding_token_given_missing_variable_name_when_parsing() {
    // Arrange
    let source = SourceFile::from_path(Path::new("broken.ts"), "const : number = 1;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(
        syntax_tree.diagnostics()[0].span().start(),
        Utf16Offset::new(6)
    );
}

#[test]
fn should_report_unterminated_string_literal_given_missing_closing_quote_when_parsing() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("broken.ts"),
        "const message: string = 'unfinished;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert!(
        syntax_tree
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 1002)
    );
}

#[test]
fn should_report_required_parameter_after_optional_given_function_declaration_when_parsing() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("broken.ts"),
        "function answer(optional?: number, required: number): number { return required; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics().len(), 1);
    assert_eq!(syntax_tree.diagnostics()[0].code(), 1016);
    assert_eq!(
        syntax_tree.diagnostics()[0].message(),
        "A required parameter cannot follow an optional parameter."
    );
    assert_eq!(
        syntax_tree.diagnostics()[0].span().start(),
        Utf16Offset::new(35)
    );
    assert_eq!(syntax_tree.diagnostics()[0].span().length(), 8);
}

#[test]
fn should_reject_readonly_modifier_given_class_method_when_parsing() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("broken.ts"),
        "class Greeter { readonly greet(): void {} }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics().len(), 1);
    assert_eq!(syntax_tree.diagnostics()[0].code(), 1024);
    assert_eq!(
        syntax_tree.diagnostics()[0].message(),
        "'readonly' modifier can only appear on a property declaration or index signature."
    );
}

#[test]
fn should_report_unparenthesized_nullish_disjunction_mixture_given_expression_when_parsing() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("broken.ts"),
        "function mixed(first: string | undefined, second: boolean, third: boolean) { return first ?? second || third; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics().len(), 1);
    assert_eq!(syntax_tree.diagnostics()[0].code(), 5076);
    assert_eq!(
        syntax_tree.diagnostics()[0].message(),
        "'??' and '||' operations cannot be mixed without parentheses."
    );
}

#[test]
fn should_report_unparenthesized_nullish_conjunction_mixture_given_expression_when_parsing() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("broken.ts"),
        "function mixed(first: string | undefined, second: boolean) { return first ?? second && true; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics().len(), 1);
    assert_eq!(syntax_tree.diagnostics()[0].code(), 5076);
    assert_eq!(
        syntax_tree.diagnostics()[0].message(),
        "'??' and '&&' operations cannot be mixed without parentheses."
    );
}

#[test]
fn should_report_break_outside_loop_given_function_body_when_parsing() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("broken.ts"),
        "function invalid(): void { break; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics().len(), 1);
    assert_eq!(syntax_tree.diagnostics()[0].code(), 1107);
    assert_eq!(
        syntax_tree.diagnostics()[0].message(),
        "Jump target cannot cross function boundary."
    );
}

#[test]
fn should_report_continue_outside_loop_given_function_body_when_parsing() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("broken.ts"),
        "function invalid(): void { continue; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics().len(), 1);
    assert_eq!(syntax_tree.diagnostics()[0].code(), 1107);
    assert_eq!(
        syntax_tree.diagnostics()[0].message(),
        "Jump target cannot cross function boundary."
    );
}
