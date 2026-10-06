use tsrzl::ast::{Ast, FlowFlags, NodeFlags, NodeId, SyntaxKind};
use tsrzl::bind::{BoundFile, FlowId, FlowPayload, bind_source_file};
use tsrzl::parser::{
    ExternalModuleIndicatorOptions, ParseOptions, ParsedSourceFile, ScriptKind, parse_source_file,
};

fn parse(text: &str) -> ParsedSourceFile {
    parse_source_file(&ParseOptions::new("test.ts", ScriptKind::Ts), text)
}

fn bind(parsed: &ParsedSourceFile) -> BoundFile {
    bind_source_file(parsed, ExternalModuleIndicatorOptions::default())
}

fn all(ast: &Ast, id: NodeId, kind: SyntaxKind, found: &mut Vec<NodeId>) {
    if ast.node(id).kind() == kind {
        found.push(id);
    }
    for child in ast.children(id) {
        all(ast, child, kind, found);
    }
}

fn nodes(parsed: &ParsedSourceFile, kind: SyntaxKind) -> Vec<NodeId> {
    let mut found = Vec::new();
    all(parsed.ast(), parsed.ast().root(), kind, &mut found);
    found
}

/// Returns the identifier with `name` at `occurrence` in source order.
fn identifier(parsed: &ParsedSourceFile, name: &str, occurrence: usize) -> NodeId {
    nodes(parsed, SyntaxKind::Identifier)
        .into_iter()
        .filter(|&id| parsed.ast().identifier_text(id) == Some(name))
        .nth(occurrence)
        .expect("the identifier occurs in the source")
}

fn flow_of(bound: &BoundFile, node: NodeId) -> FlowId {
    bound.flow_node_of(node).expect("the node has a flow node")
}

#[test]
fn should_flag_statement_unreachable_given_statement_after_return_when_binding() {
    // Arrange
    let parsed = parse("function run() { return; run(); }");

    // Act
    let bound = bind(&parsed);

    // Assert
    let statement = nodes(&parsed, SyntaxKind::ExpressionStatement)[0];
    assert!(bound.node_flags(statement).contains(NodeFlags::UNREACHABLE));
}

#[test]
fn should_flag_statement_unreachable_given_statement_after_throw_when_binding() {
    // Arrange
    let parsed = parse("throw 1;\nlet after = 2;");

    // Act
    let bound = bind(&parsed);

    // Assert
    let statement = nodes(&parsed, SyntaxKind::VariableStatement)[0];
    assert!(bound.node_flags(statement).contains(NodeFlags::UNREACHABLE));
}

#[test]
fn should_not_flag_uninitialized_var_unreachable_given_var_after_return_when_binding() {
    // Arrange
    let parsed = parse("function run() { return; var hoisted; }");

    // Act
    let bound = bind(&parsed);

    // Assert
    let statement = nodes(&parsed, SyntaxKind::VariableStatement)[0];
    assert!(!bound.node_flags(statement).contains(NodeFlags::UNREACHABLE));
}

#[test]
fn should_flag_implicit_return_given_reachable_function_end_when_binding() {
    // Arrange
    let parsed = parse("function run(flag: boolean) { if (flag) { return 1; } }");

    // Act
    let bound = bind(&parsed);

    // Assert
    let function = nodes(&parsed, SyntaxKind::FunctionDeclaration)[0];
    assert!(
        bound
            .node_flags(function)
            .contains(NodeFlags::HAS_IMPLICIT_RETURN | NodeFlags::HAS_EXPLICIT_RETURN)
    );
}

#[test]
fn should_not_flag_implicit_return_given_return_on_every_path_when_binding() {
    // Arrange
    let parsed =
        parse("function run(flag: boolean) { if (flag) { return 1; } else { return 2; } }");

    // Act
    let bound = bind(&parsed);

    // Assert
    let function = nodes(&parsed, SyntaxKind::FunctionDeclaration)[0];
    assert!(
        !bound
            .node_flags(function)
            .contains(NodeFlags::HAS_IMPLICIT_RETURN)
    );
}

#[test]
fn should_narrow_with_true_condition_given_identifier_in_if_branch_when_binding() {
    // Arrange
    let parsed = parse("declare const value: string | undefined;\nif (value) { value; }");

    // Act
    let bound = bind(&parsed);

    // Assert
    let inner = flow_of(&bound, identifier(&parsed, "value", 2));
    let node = bound.flow().node(inner);
    assert_eq!(
        (
            node.flags().contains(FlowFlags::TRUE_CONDITION),
            node.payload()
        ),
        (true, FlowPayload::Node(identifier(&parsed, "value", 1)))
    );
}

#[test]
fn should_skip_condition_node_given_non_narrowing_condition_when_binding() {
    // Arrange
    let parsed =
        parse("declare function check(): boolean;\nlet value = 1;\nif (check()) { value; }");

    // Act
    let bound = bind(&parsed);

    // Assert
    let inner = flow_of(&bound, identifier(&parsed, "value", 1));
    assert!(
        !bound
            .flow()
            .node(inner)
            .flags()
            .intersects(FlowFlags::CONDITION)
    );
}

#[test]
fn should_record_assignment_given_reassigned_variable_when_binding() {
    // Arrange
    let parsed = parse("let value = 1;\nvalue = 2;\nvalue;");

    // Act
    let bound = bind(&parsed);

    // Assert
    let read = flow_of(&bound, identifier(&parsed, "value", 2));
    let node = bound.flow().node(read);
    assert_eq!(
        (node.flags().contains(FlowFlags::ASSIGNMENT), node.payload()),
        (true, FlowPayload::Node(identifier(&parsed, "value", 1)))
    );
}

#[test]
fn should_record_assignment_given_initialized_declaration_when_binding() {
    // Arrange
    let parsed = parse("let value = 1;\nvalue;");

    // Act
    let bound = bind(&parsed);

    // Assert
    let read = flow_of(&bound, identifier(&parsed, "value", 1));
    let declaration = nodes(&parsed, SyntaxKind::VariableDeclaration)[0];
    assert_eq!(
        bound.flow().node(read).payload(),
        FlowPayload::Node(declaration)
    );
}

#[test]
fn should_reach_loop_label_given_identifier_in_while_body_when_binding() {
    // Arrange
    let parsed = parse("let value = 1;\nwhile (value) { value = 0; }");

    // Act
    let bound = bind(&parsed);

    // Assert
    let condition = flow_of(&bound, identifier(&parsed, "value", 1));
    assert!(
        bound
            .flow()
            .node(condition)
            .flags()
            .contains(FlowFlags::LOOP_LABEL)
    );
}

#[test]
fn should_join_both_branches_given_statement_after_if_else_when_binding() {
    // Arrange
    let parsed = parse("let value = 1;\nif (value) { value = 2; } else { value = 3; }\nvalue;");

    // Act
    let bound = bind(&parsed);

    // Assert
    let after = flow_of(&bound, identifier(&parsed, "value", 4));
    assert_eq!(bound.flow().antecedents(after).count(), 2);
}

#[test]
fn should_follow_right_operand_with_true_condition_given_logical_and_when_binding() {
    // Arrange
    let parsed = parse("declare const value: string | undefined;\nvalue && value.length;");

    // Act
    let bound = bind(&parsed);

    // Assert
    let right = flow_of(&bound, identifier(&parsed, "value", 2));
    assert!(
        bound
            .flow()
            .node(right)
            .flags()
            .contains(FlowFlags::TRUE_CONDITION)
    );
}

#[test]
fn should_flag_unused_label_unreachable_given_unreferenced_label_when_binding() {
    // Arrange
    let parsed = parse("outer: for (;;) { break; }");

    // Act
    let bound = bind(&parsed);

    // Assert
    let label = identifier(&parsed, "outer", 0);
    assert!(bound.node_flags(label).contains(NodeFlags::UNREACHABLE));
}

#[test]
fn should_not_flag_used_label_given_labeled_break_when_binding() {
    // Arrange
    let parsed = parse("outer: for (;;) { break outer; }");

    // Act
    let bound = bind(&parsed);

    // Assert
    let label = identifier(&parsed, "outer", 0);
    assert!(!bound.node_flags(label).contains(NodeFlags::UNREACHABLE));
}

#[test]
fn should_reach_code_after_switch_given_switch_without_default_when_binding() {
    // Arrange
    let parsed =
        parse("function run(value: number) { switch (value) { case 1: return 1; } value; }");

    // Act
    let bound = bind(&parsed);

    // Assert
    let after = nodes(&parsed, SyntaxKind::ExpressionStatement)[0];
    assert!(!bound.node_flags(after).contains(NodeFlags::UNREACHABLE));
}

#[test]
fn should_flag_code_after_exhaustive_switch_given_default_return_when_binding() {
    // Arrange
    let parsed =
        parse("function run(value: number) { switch (value) { default: return 1; } value; }");

    // Act
    let bound = bind(&parsed);

    // Assert
    let after = nodes(&parsed, SyntaxKind::ExpressionStatement)[0];
    assert!(bound.node_flags(after).contains(NodeFlags::UNREACHABLE));
}

#[test]
fn should_flag_code_after_try_finally_given_return_in_try_when_binding() {
    // Arrange
    let parsed = parse("function run() { try { return 1; } finally { run; } run(); }");

    // Act
    let bound = bind(&parsed);

    // Assert
    let after = nodes(&parsed, SyntaxKind::ExpressionStatement)[1];
    assert!(bound.node_flags(after).contains(NodeFlags::UNREACHABLE));
}

#[test]
fn should_keep_code_after_iife_reachable_given_return_inside_iife_when_binding() {
    // Arrange
    let parsed = parse("(function () { return 1; })();\nlet after = 2;");

    // Act
    let bound = bind(&parsed);

    // Assert
    let statement = nodes(&parsed, SyntaxKind::VariableStatement)[0];
    assert!(!bound.node_flags(statement).contains(NodeFlags::UNREACHABLE));
}

#[test]
fn should_record_array_mutation_before_call_given_push_call_statement_when_binding() {
    // Arrange
    let parsed = parse("const items = [];\nitems.push(1);\nitems;");

    // Act
    let bound = bind(&parsed);

    // Assert
    let read = flow_of(&bound, identifier(&parsed, "items", 2));
    let mutation = bound
        .flow()
        .node(read)
        .antecedent()
        .expect("the call node follows the mutation");
    assert!(
        bound
            .flow()
            .node(mutation)
            .flags()
            .contains(FlowFlags::ARRAY_MUTATION)
    );
}

#[test]
fn should_record_call_given_dotted_assertion_call_statement_when_binding() {
    // Arrange
    let parsed = parse(
        "declare const assert: { ok(value: unknown): asserts value };\nlet value: unknown;\nassert.ok(value);\nvalue;",
    );

    // Act
    let bound = bind(&parsed);

    // Assert
    let read = flow_of(&bound, identifier(&parsed, "value", 4));
    assert!(bound.flow().node(read).flags().contains(FlowFlags::CALL));
}

#[test]
fn should_record_end_flow_given_source_file_when_binding() {
    // Arrange
    let parsed = parse("let value = 1;");

    // Act
    let bound = bind(&parsed);

    // Assert
    let end = bound.end_flow_node_of(parsed.ast().root());
    assert!(end.is_some_and(|end| {
        bound
            .flow()
            .node(end)
            .flags()
            .contains(FlowFlags::ASSIGNMENT)
    }));
}

#[test]
fn should_record_switch_clause_given_narrowing_switch_when_binding() {
    // Arrange
    let parsed = parse("declare const kind: \"a\" | \"b\";\nswitch (kind) { case \"a\": kind; }");

    // Act
    let bound = bind(&parsed);

    // Assert
    let inner = flow_of(&bound, identifier(&parsed, "kind", 2));
    assert!(matches!(
        bound.flow().node(inner).payload(),
        FlowPayload::SwitchClause {
            clause_start: 0,
            clause_end: 1,
            ..
        }
    ));
}

#[test]
fn should_flag_contains_this_given_this_in_method_when_binding() {
    // Arrange
    let parsed = parse("class Counter { count = 0; next() { return this.count; } }");

    // Act
    let bound = bind(&parsed);

    // Assert
    let method = nodes(&parsed, SyntaxKind::MethodDeclaration)[0];
    assert!(bound.node_flags(method).contains(NodeFlags::CONTAINS_THIS));
}

#[test]
fn should_flag_async_functions_given_async_function_when_binding() {
    // Arrange
    let parsed = parse("async function load() {}");

    // Act
    let bound = bind(&parsed);

    // Assert
    assert!(
        bound
            .node_flags(parsed.ast().root())
            .contains(NodeFlags::HAS_ASYNC_FUNCTIONS)
    );
}
