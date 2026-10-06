use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_parse_jsx_string_attribute_given_tsx_source_when_building_syntax_tree() {
    // Pinned TypeScript 7.0.2 case: conformance/jsx/jsxReactTestSuite.tsx.
    // Arrange
    let source = SourceFile::from_path(Path::new("view.tsx"), "const view = <div id=\"main\" />;")
        .expect("a TSX path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

#[test]
fn should_parse_jsx_expression_attribute_given_tsx_source_when_building_syntax_tree() {
    // Pinned TypeScript 7.0.2 case: conformance/jsx/jsxReactTestSuite.tsx.
    // Arrange
    let source =
        SourceFile::from_path(Path::new("view.tsx"), "const view = <div title={value} />;")
            .expect("a TSX path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

#[test]
fn should_parse_jsx_boolean_attribute_given_tsx_source_when_building_syntax_tree() {
    // Pinned TypeScript 7.0.2 case: conformance/jsx/jsxReactTestSuite.tsx.
    // Arrange
    let source = SourceFile::from_path(Path::new("view.tsx"), "const view = <input disabled />;")
        .expect("a TSX path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

#[test]
fn should_parse_jsx_spread_attribute_given_tsx_source_when_building_syntax_tree() {
    // Pinned TypeScript 7.0.2 case: conformance/jsx/jsxReactTestSuite.tsx.
    // Arrange
    let source = SourceFile::from_path(Path::new("view.tsx"), "const view = <div {...props} />;")
        .expect("a TSX path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

#[test]
fn should_parse_jsx_text_child_given_tsx_source_when_building_syntax_tree() {
    // Pinned TypeScript 7.0.2 case: conformance/jsx/jsxReactTestSuite.tsx.
    // Arrange
    let source = SourceFile::from_path(Path::new("view.tsx"), "const view = <div>hello</div>;")
        .expect("a TSX path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

#[test]
fn should_parse_nested_jsx_element_given_tsx_source_when_building_syntax_tree() {
    // Pinned TypeScript 7.0.2 case: conformance/jsx/jsxReactTestSuite.tsx.
    // Arrange
    let source = SourceFile::from_path(Path::new("view.tsx"), "const view = <div><span /></div>;")
        .expect("a TSX path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

#[test]
fn should_parse_jsx_expression_child_given_tsx_source_when_building_syntax_tree() {
    // Pinned TypeScript 7.0.2 case: conformance/jsx/jsxReactTestSuite.tsx.
    // Arrange
    let source = SourceFile::from_path(Path::new("view.tsx"), "const view = <div>{value}</div>;")
        .expect("a TSX path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

#[test]
fn should_parse_jsx_fragment_given_tsx_source_when_building_syntax_tree() {
    // Pinned TypeScript 7.0.2 case: conformance/jsx/tsxFragmentPreserveEmit.tsx.
    // Arrange
    let source = SourceFile::from_path(Path::new("view.tsx"), "const view = <>text</>;")
        .expect("a TSX path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

#[test]
fn should_report_ts17002_given_mismatched_jsx_closing_tag_when_building_syntax_tree() {
    // Pinned TypeScript 7.0.2 case: conformance/jsx/jsxInvalidEsprimaTestSuite.tsx.
    // Arrange
    let source = SourceFile::from_path(Path::new("view.tsx"), "const view = <a></b>;")
        .expect("a TSX path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert!(
        syntax_tree.diagnostics().iter().any(|diagnostic| {
            diagnostic.code() == 17002
                && diagnostic
                    .message()
                    .contains("Expected corresponding JSX closing tag for 'a'")
        }),
        "a mismatched JSX closing tag should report TS17002, got {:?}",
        syntax_tree.diagnostics()
    );
}
