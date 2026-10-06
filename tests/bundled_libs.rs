use tsrzl::bundled::{lib_names, lib_text};

#[test]
fn should_list_libs_in_name_order_given_bundled_libs_when_reading_lib_names() {
    // Arrange
    let names = lib_names();

    // Act
    let sorted = names.windows(2).all(|pair| pair[0] < pair[1]);

    // Assert
    assert!(sorted && names.len() == 108);
}

#[test]
fn should_return_declarations_given_es5_lib_name_when_reading_lib_text() {
    // Arrange
    let name = "lib.es5.d.ts";

    // Act
    let text = lib_text(name);

    // Assert
    assert!(text.is_some_and(|text| text.contains("interface Array<T>")));
}

#[test]
fn should_return_none_given_unknown_lib_name_when_reading_lib_text() {
    // Arrange
    let name = "lib.unknown.d.ts";

    // Act
    let text = lib_text(name);

    // Assert
    assert_eq!(text, None);
}

#[test]
fn should_map_option_to_file_given_lib_option_name_when_reading_lib_file() {
    // Arrange
    let option = "es2015.promise";

    // Act
    let file = tsrzl::bundled::lib_file_for_option(option);

    // Assert
    assert_eq!(file, Some("lib.es2015.promise.d.ts"));
}

#[test]
fn should_use_es6_lib_given_es2015_target_when_reading_default_lib() {
    // Arrange
    let target = "es2015";

    // Act
    let file = tsrzl::bundled::default_lib_file_name(target);

    // Assert
    assert_eq!(file, "lib.es6.d.ts");
}

#[test]
fn should_use_plain_lib_given_es5_target_when_reading_default_lib() {
    // Arrange
    let target = "es5";

    // Act
    let file = tsrzl::bundled::default_lib_file_name(target);

    // Assert
    assert_eq!(file, "lib.d.ts");
}

#[test]
fn should_accept_file_name_given_lib_file_name_when_resolving_lib_name() {
    // Arrange
    let name = "LIB.ES2015.PROMISE.D.TS";

    // Act
    let file = tsrzl::bundled::lib_file_name(name);

    // Assert
    assert_eq!(file, Some("lib.es2015.promise.d.ts"));
}

#[test]
fn should_sort_default_lib_first_given_lib_priorities_when_ordering_libs() {
    // Arrange
    let names = ["lib.dom.d.ts", "lib.d.ts", "lib.es5.d.ts"];

    // Act
    let mut sorted = names;
    sorted.sort_by_key(|name| tsrzl::bundled::lib_priority(name));

    // Assert
    assert_eq!(sorted, ["lib.d.ts", "lib.es5.d.ts", "lib.dom.d.ts"]);
}

#[test]
fn should_parse_without_diagnostics_given_every_bundled_lib_when_parsing() {
    // Arrange
    let names = lib_names();

    // Act
    let failing: Vec<_> = names
        .into_iter()
        .filter(|name| {
            let options = tsrzl::parser::ParseOptions::new(*name, tsrzl::parser::ScriptKind::Ts);
            let text = lib_text(name).expect("listed libs have text");
            !tsrzl::parser::parse_source_file(&options, text)
                .diagnostics()
                .is_empty()
        })
        .collect();

    // Assert
    assert_eq!(failing, Vec::<&str>::new());
}

#[test]
fn should_bind_without_diagnostics_given_every_bundled_lib_when_binding() {
    // Arrange
    let names = lib_names();

    // Act
    let failing: Vec<_> = names
        .into_iter()
        .filter(|name| {
            let options = tsrzl::parser::ParseOptions::new(*name, tsrzl::parser::ScriptKind::Ts);
            let text = lib_text(name).expect("listed libs have text");
            let parsed = tsrzl::parser::parse_source_file(&options, text);
            let bound = tsrzl::bind::bind_source_file(
                &parsed,
                tsrzl::parser::ExternalModuleIndicatorOptions::default(),
            );
            !bound.diagnostics().is_empty()
        })
        .collect();

    // Assert
    assert_eq!(failing, Vec::<&str>::new());
}
