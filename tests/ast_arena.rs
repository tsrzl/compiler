use tsrzl::ast::{
    AstBuilder, BinaryExpression, Identifier, NodeData, NodeFlags, NodeId, SyntaxKind,
};

fn identifier(builder: &mut AstBuilder, text: &str, pos: u32) -> NodeId {
    let end = pos + u32::try_from(text.len()).expect("test identifiers are short");
    builder.add_node(
        SyntaxKind::Identifier,
        pos,
        end,
        NodeFlags::NONE,
        NodeData::Identifier(Identifier { text: text.into() }),
    )
}

fn binary_expression(builder: &mut AstBuilder) -> (NodeId, [NodeId; 3]) {
    let left = identifier(builder, "a", 0);
    let operator = builder.add_node(
        SyntaxKind::PlusToken,
        1,
        3,
        NodeFlags::NONE,
        NodeData::Token,
    );
    let right = identifier(builder, "b", 4);
    let data = NodeData::BinaryExpression(BinaryExpression {
        modifiers: None,
        left,
        type_node: None,
        operator_token: operator,
        right,
    });
    let binary = builder.add_node(SyntaxKind::BinaryExpression, 0, 5, NodeFlags::NONE, data);
    (binary, [left, operator, right])
}

#[test]
fn should_visit_children_in_source_order_given_binary_expression_when_walking_ast() {
    // Arrange
    let mut builder = AstBuilder::new();
    let (binary, expected) = binary_expression(&mut builder);
    let ast = builder.finish(binary);

    // Act
    let actual = ast.children(binary);

    // Assert
    assert_eq!(actual, expected);
}

#[test]
fn should_link_child_to_parent_given_finished_binary_expression_when_reading_parent() {
    // Arrange
    let mut builder = AstBuilder::new();
    let (binary, [left, _, _]) = binary_expression(&mut builder);
    let ast = builder.finish(binary);

    // Act
    let actual = ast.node(left).parent();

    // Assert
    assert_eq!(actual, Some(binary));
}
