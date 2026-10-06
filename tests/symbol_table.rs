use tsrzl::ast::SymbolFlags;
use tsrzl::symbols::{
    INTERNAL_SYMBOL_NAME_CALL, SymbolArena, SymbolTable, escape_internal_symbol_name,
    escape_symbol_name,
};

#[test]
fn should_return_inserted_symbol_given_name_when_looking_up_symbol_table() {
    // Arrange
    let mut arena = SymbolArena::default();
    let symbol = arena.create(SymbolFlags::FUNCTION, "run");
    let mut table = SymbolTable::default();
    table.insert("run", symbol);

    // Act
    let actual = table.get("run");

    // Assert
    assert_eq!(actual, Some(symbol));
}

#[test]
fn should_iterate_in_insertion_order_given_several_names_when_reading_symbol_table() {
    // Arrange
    let mut arena = SymbolArena::default();
    let mut table = SymbolTable::default();
    for name in ["zeta", "alpha", "mid"] {
        let symbol = arena.create(SymbolFlags::BLOCK_SCOPED_VARIABLE, name);
        table.insert(name, symbol);
    }

    // Act
    let names: Vec<&str> = table.iter().map(|(name, _)| name).collect();

    // Assert
    assert_eq!(names, ["zeta", "alpha", "mid"]);
}

#[test]
fn should_report_external_module_given_quoted_module_symbol_when_reading_symbol() {
    // Arrange
    let mut arena = SymbolArena::default();
    let module = arena.create(SymbolFlags::VALUE_MODULE, "\"./feature\"");

    // Act
    let actual = arena.symbol(module).is_external_module();

    // Assert
    assert!(actual);
}

#[test]
fn should_combine_export_symbol_flags_given_local_with_export_symbol_when_reading_symbol() {
    // Arrange
    let mut arena = SymbolArena::default();
    let export = arena.create(SymbolFlags::FUNCTION, "run");
    let local = arena.create(SymbolFlags::EXPORT_VALUE, "run");
    arena.symbol_mut(local).export_symbol = Some(export);

    // Act
    let actual = arena.combined_local_and_export_symbol_flags(local);

    // Assert
    assert_eq!(actual, SymbolFlags::EXPORT_VALUE | SymbolFlags::FUNCTION);
}

#[test]
fn should_escape_internal_prefix_given_internal_name_when_escaping_symbol_name() {
    // Arrange
    let name = INTERNAL_SYMBOL_NAME_CALL;

    // Act
    let actual = escape_internal_symbol_name(name);

    // Assert
    assert_eq!(actual, "__call");
}

#[test]
fn should_add_underscore_given_user_name_with_double_underscore_when_escaping_symbol_name() {
    // Arrange
    let name = "__proto";

    // Act
    let actual = escape_symbol_name(name);

    // Assert
    assert_eq!(actual, "___proto");
}
