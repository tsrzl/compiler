use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::{BinaryOperator, Expression, SyntaxTree};

#[test]
fn should_parse_type_annotated_variable_given_typescript_source_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(Path::new("answer.ts"), "const answer: number = 42;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics().len(), 0);
    let statement = syntax_tree
        .program()
        .statements()
        .first()
        .expect("the source contains one declaration");
    let variable = statement
        .as_variable_declaration()
        .expect("the statement is a variable declaration");
    assert_eq!(
        variable
            .type_annotation()
            .map(tsrzl::syntax::TypeReference::name),
        Some("number")
    );
}

#[test]
fn should_parse_private_identifier_given_class_field_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(Path::new("person.ts"), "class Person { #name: string; }")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

#[test]
fn should_parse_standard_decorator_given_class_declaration_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("decorated.ts"),
        "const sealed = (value: any, context: any) => value; @sealed class Example {}",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

#[test]
fn should_parse_intrinsic_jsx_element_given_tsx_source_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(Path::new("view.tsx"), "const view = <div />;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

#[test]
fn should_parse_import_attributes_given_json_import_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("settings.ts"),
        "import settings from \"./settings.json\" with { type: \"json\" };",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

#[test]
fn should_parse_deferred_import_given_default_binding_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("deferred.ts"),
        "import defer from \"./feature.js\";",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

#[test]
fn should_parse_each_variable_given_comma_separated_declarations_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("values.ts"),
        "const first: number = 1, second: number = 2;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics().len(), 0);
    assert_eq!(syntax_tree.program().statements().len(), 2);
}

#[test]
fn should_parse_multiplication_precedence_given_mixed_expression_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(Path::new("total.ts"), "const total: number = 1 + 2 * 3;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    let initializer = syntax_tree.program().statements()[0]
        .as_variable_declaration()
        .and_then(|declaration| declaration.initializer())
        .expect("the variable has an initializer");
    assert_eq!(syntax_tree.diagnostics().len(), 0);
    assert!(matches!(
        initializer,
        Expression::BinaryExpression {
            operator: BinaryOperator::Add,
            right,
            ..
        } if matches!(
            right.as_ref(),
            Expression::BinaryExpression {
                operator: BinaryOperator::Multiply,
                ..
            }
        )
    ));
}

#[test]
fn should_parse_division_precedence_given_mixed_expression_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("quotient.ts"),
        "const quotient: number = 8 / 2 + 1;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    let initializer = syntax_tree.program().statements()[0]
        .as_variable_declaration()
        .and_then(|declaration| declaration.initializer())
        .expect("the variable has an initializer");
    assert_eq!(syntax_tree.diagnostics().len(), 0);
    assert!(matches!(
        initializer,
        Expression::BinaryExpression {
            operator: BinaryOperator::Add,
            left,
            ..
        } if matches!(
            left.as_ref(),
            Expression::BinaryExpression {
                operator: BinaryOperator::Divide,
                ..
            }
        )
    ));
}

#[test]
fn should_retain_parenthesized_expression_given_grouped_initializer_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(Path::new("total.ts"), "const total: number = (1 + 2) * 3;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    let initializer = syntax_tree.program().statements()[0]
        .as_variable_declaration()
        .and_then(|declaration| declaration.initializer())
        .expect("the variable has an initializer");
    assert_eq!(syntax_tree.diagnostics().len(), 0);
    assert!(matches!(
        initializer,
        Expression::BinaryExpression {
            operator: BinaryOperator::Multiply,
            left,
            ..
        } if matches!(
            left.as_ref(),
            Expression::ParenthesizedExpression { expression, .. }
                if matches!(
                    expression.as_ref(),
                    Expression::BinaryExpression {
                        operator: BinaryOperator::Add,
                        ..
                    }
                )
        )
    ));
}

#[test]
fn should_parse_function_call_given_call_initializer_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("answer.ts"),
        "const answer: number = compute(40, 2);",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    let initializer = syntax_tree.program().statements()[0]
        .as_variable_declaration()
        .and_then(|declaration| declaration.initializer())
        .expect("the variable has an initializer");
    assert_eq!(syntax_tree.diagnostics().len(), 0);
    assert!(matches!(
        initializer,
        Expression::CallExpression {
            callee,
            arguments,
            ..
        } if arguments.len() == 2
            && matches!(
                callee.as_ref(),
                Expression::Identifier { name, .. } if name == "compute"
            )
    ));
}

#[test]
fn should_parse_unary_negation_given_negative_initializer_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(Path::new("offset.ts"), "const offset: number = -1;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    let initializer = syntax_tree.program().statements()[0]
        .as_variable_declaration()
        .and_then(|declaration| declaration.initializer())
        .expect("the variable has an initializer");
    assert_eq!(syntax_tree.diagnostics().len(), 0);
    assert!(matches!(
        initializer,
        Expression::UnaryExpression {
            operator: tsrzl::syntax::UnaryOperator::Negate,
            operand,
            ..
        } if matches!(operand.as_ref(), Expression::NumberLiteral { value, .. } if value == "1")
    ));
}

#[test]
fn should_parse_property_access_given_member_initializer_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(Path::new("person.ts"), "const name: string = person.name;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    let initializer = syntax_tree.program().statements()[0]
        .as_variable_declaration()
        .and_then(|declaration| declaration.initializer())
        .expect("the variable has an initializer");
    assert_eq!(syntax_tree.diagnostics().len(), 0);
    assert!(matches!(
        initializer,
        Expression::PropertyAccessExpression { receiver, name, .. }
            if name == "name"
                && matches!(
                    receiver.as_ref(),
                    Expression::Identifier { name, .. } if name == "person"
                )
    ));
}
