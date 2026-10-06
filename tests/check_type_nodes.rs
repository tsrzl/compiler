use tsrzl::ast::{Ast, NodeId, SyntaxKind};
use tsrzl::check::{Checker, CheckerOptions, LiteralValue, NodeRef, TypeData, TypeFlags, TypeId};
use tsrzl::parser::{ParseOptions, ScriptKind, parse_source_file};
use tsrzl::program::ProgramFile;

fn file(text: &str) -> ProgramFile {
    ProgramFile::new(parse_source_file(
        &ParseOptions::new("a.ts", ScriptKind::Ts),
        text,
    ))
}

fn find(ast: &Ast, id: NodeId, kind: SyntaxKind) -> Option<NodeId> {
    if ast.node(id).kind() == kind {
        return Some(id);
    }
    ast.children(id)
        .into_iter()
        .find_map(|child| find(ast, child, kind))
}

/// Returns the type annotation of the first variable declaration in file 0.
fn annotation(files: &[ProgramFile]) -> NodeRef {
    let ast = files[0].parsed().ast();
    let declaration =
        find(ast, ast.root(), SyntaxKind::VariableDeclaration).expect("a declaration");
    let node = ast
        .node(declaration)
        .data()
        .type_node()
        .expect("the declaration is annotated");
    NodeRef { file: 0, node }
}

fn annotated_type(text: &str) -> (Checker<'static>, TypeId) {
    let files: &'static [ProgramFile] = Box::leak(Box::new([file(text)]));
    let mut checker = Checker::new(files, CheckerOptions::default());
    let ty = checker.type_from_type_node(annotation(files));
    (checker, ty)
}

#[test]
fn should_return_string_given_string_keyword_when_resolving_type_node() {
    // Arrange
    let text = "let value: string;";

    // Act
    let (checker, ty) = annotated_type(text);

    // Assert
    assert_eq!(ty, checker.types().intrinsics().string);
}

#[test]
fn should_return_boolean_union_given_boolean_keyword_when_resolving_type_node() {
    // Arrange
    let text = "let value: boolean;";

    // Act
    let (checker, ty) = annotated_type(text);

    // Assert
    assert_eq!(ty, checker.types().intrinsics().boolean);
}

#[test]
fn should_return_null_given_null_literal_type_when_resolving_type_node() {
    // Arrange
    let text = "let value: null;";

    // Act
    let (checker, ty) = annotated_type(text);

    // Assert
    assert_eq!(ty, checker.types().intrinsics().null);
}

#[test]
fn should_return_regular_string_literal_given_string_literal_type_when_resolving_type_node() {
    // Arrange
    let text = "let value: \"on\";";

    // Act
    let (checker, ty) = annotated_type(text);

    // Assert
    assert!(matches!(
        checker.types().get(ty).data(),
        TypeData::Literal { value: LiteralValue::String(value), regular_type, .. }
            if &**value == "on" && *regular_type == ty
    ));
}

#[test]
fn should_return_negative_number_literal_given_negated_numeric_type_when_resolving_type_node() {
    // Arrange
    let text = "let value: -1;";

    // Act
    let (checker, ty) = annotated_type(text);

    // Assert
    assert!(matches!(
        checker.types().get(ty).data(),
        TypeData::Literal { value: LiteralValue::Number(value), .. } if value.to_bits() == (-1.0_f64).to_bits()
    ));
}

#[test]
fn should_return_regular_true_given_true_literal_type_when_resolving_type_node() {
    // Arrange
    let text = "let value: true;";

    // Act
    let (checker, ty) = annotated_type(text);

    // Assert
    assert_eq!(ty, checker.types().intrinsics().regular_true);
}

#[test]
fn should_union_constituents_given_union_type_node_when_resolving_type_node() {
    // Arrange
    let text = "let value: number | string;";

    // Act
    let (checker, ty) = annotated_type(text);

    // Assert
    let intrinsics = checker.types().intrinsics();
    assert_eq!(
        checker.types().union_constituents(ty),
        [intrinsics.string, intrinsics.number]
    );
}

#[test]
fn should_see_through_parentheses_given_parenthesized_type_when_resolving_type_node() {
    // Arrange
    let text = "let value: (number);";

    // Act
    let (checker, ty) = annotated_type(text);

    // Assert
    assert_eq!(ty, checker.types().intrinsics().number);
}

#[test]
fn should_return_boolean_given_type_predicate_when_resolving_type_node() {
    // Arrange
    let files: &'static [ProgramFile] = Box::leak(Box::new([file(
        "function isText(value: unknown): value is string { return true; }",
    )]));
    let mut checker = Checker::new(files, CheckerOptions::default());
    let ast = files[0].parsed().ast();
    let predicate = find(ast, ast.root(), SyntaxKind::TypePredicate).expect("a predicate");

    // Act
    let ty = checker.type_from_type_node(NodeRef {
        file: 0,
        node: predicate,
    });

    // Assert
    assert_eq!(
        checker.types().get(ty).flags(),
        TypeFlags::UNION | TypeFlags::BOOLEAN
    );
}
