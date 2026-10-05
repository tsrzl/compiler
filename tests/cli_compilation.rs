use std::fs;
use std::process::Command;

#[test]
fn should_write_javascript_given_typescript_file_when_running_compiler_cli() {
    // Arrange
    let directory = std::env::temp_dir().join(format!("tsrzl-cli-{}", std::process::id()));
    let input_path = directory.join("input.ts");
    let output_directory = directory.join("out");
    fs::create_dir_all(&directory).expect("the test directory can be created");
    fs::write(&input_path, "const answer: number = 42;")
        .expect("the TypeScript input can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--out-dir")
        .arg(&output_directory)
        .arg(&input_path)
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    assert_eq!(
        fs::read_to_string(output_directory.join("input.js"))
            .expect("the JavaScript output can be read"),
        "const answer = 42;\n"
    );
    fs::remove_dir_all(directory).expect("the test directory can be removed");
}

#[test]
fn should_accept_es2018_target_given_typescript_file_when_running_compiler_cli() {
    // Arrange
    let directory = std::env::temp_dir().join(format!("tsrzl-es2018-{}", std::process::id()));
    let input_path = directory.join("input.ts");
    fs::create_dir_all(&directory).expect("the test directory can be created");
    fs::write(&input_path, "const answer: number = 42;")
        .expect("the TypeScript input can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--target")
        .arg("es2018")
        .arg("--out-dir")
        .arg(directory.join("out"))
        .arg(&input_path)
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    assert_eq!(
        fs::read_to_string(directory.join("out/input.js"))
            .expect("the JavaScript output can be read"),
        "const answer = 42;\n"
    );
    fs::remove_dir_all(directory).expect("the test directory can be removed");
}

#[test]
fn should_emit_commonjs_given_commonjs_module_option_when_running_compiler_cli() {
    // Arrange
    let directory = std::env::temp_dir().join(format!("tsrzl-commonjs-{}", std::process::id()));
    let input_path = directory.join("input.ts");
    let output_directory = directory.join("out");
    fs::create_dir_all(&directory).expect("the test directory can be created");
    fs::write(&input_path, "export const answer: number = 42;")
        .expect("the TypeScript input can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--target")
        .arg("es2020")
        .arg("--module")
        .arg("commonjs")
        .arg("--out-dir")
        .arg(&output_directory)
        .arg(&input_path)
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    assert_eq!(
        fs::read_to_string(output_directory.join("input.js"))
            .expect("the JavaScript output can be read"),
        "\"use strict\";\nObject.defineProperty(exports, \"__esModule\", { value: true });\nexports.answer = void 0;\nexports.answer = 42;\n"
    );
    fs::remove_dir_all(directory).expect("the test directory can be removed");
}

#[test]
fn should_write_source_map_given_source_map_option_when_running_compiler_cli() {
    // Arrange
    let directory = std::env::temp_dir().join(format!("tsrzl-source-map-{}", std::process::id()));
    let input_path = directory.join("input.ts");
    let output_directory = directory.join("out");
    fs::create_dir_all(&directory).expect("the test directory can be created");
    fs::write(&input_path, "const answer: number = 42;")
        .expect("the TypeScript input can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--sourceMap")
        .arg("--outDir")
        .arg(&output_directory)
        .arg(&input_path)
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    let javascript = fs::read_to_string(output_directory.join("input.js"))
        .expect("the JavaScript output can be read");
    let source_map = fs::read_to_string(output_directory.join("input.js.map"))
        .expect("the source map can be read");
    assert!(javascript.contains("//# sourceMappingURL=input.js.map"));
    assert!(source_map.contains("\"sources\":[\"../input.ts\"]"));
    fs::remove_dir_all(directory).expect("the test directory can be removed");
}

#[test]
fn should_compile_relative_typescript_import_given_entry_file_when_running_compiler_cli() {
    // Arrange
    let directory = std::env::temp_dir().join(format!("tsrzl-import-{}", std::process::id()));
    let entry_path = directory.join("main.ts");
    let imported_path = directory.join("polyfill.ts");
    let output_directory = directory.join("out");
    fs::create_dir_all(&directory).expect("the test directory can be created");
    fs::write(&entry_path, "import './polyfill';")
        .expect("the importing TypeScript source can be written");
    fs::write(&imported_path, "const ready: boolean = true;")
        .expect("the imported TypeScript source can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--out-dir")
        .arg(&output_directory)
        .arg(&entry_path)
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    assert_eq!(
        fs::read_to_string(output_directory.join("main.js"))
            .expect("the entry JavaScript output can be read"),
        "import './polyfill';\n"
    );
    assert_eq!(
        fs::read_to_string(output_directory.join("polyfill.js"))
            .expect("the imported JavaScript output can be read"),
        "const ready = true;\n"
    );
    fs::remove_dir_all(directory).expect("the test directory can be removed");
}

#[test]
fn should_compile_export_all_dependency_given_entry_file_when_running_compiler_cli() {
    // Arrange
    let directory = std::env::temp_dir().join(format!("tsrzl-export-all-{}", std::process::id()));
    let entry_path = directory.join("main.ts");
    let exported_path = directory.join("answer.ts");
    let output_directory = directory.join("out");
    fs::create_dir_all(&directory).expect("the test directory can be created");
    fs::write(&entry_path, "export * from './answer';")
        .expect("the entry TypeScript source can be written");
    fs::write(&exported_path, "export const answer: number = 42;")
        .expect("the re-exported TypeScript source can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--out-dir")
        .arg(&output_directory)
        .arg(&entry_path)
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    assert_eq!(
        fs::read_to_string(output_directory.join("main.js"))
            .expect("the entry JavaScript output can be read"),
        "export * from './answer';\n"
    );
    assert_eq!(
        fs::read_to_string(output_directory.join("answer.js"))
            .expect("the re-exported JavaScript output can be read"),
        "export const answer = 42;\n"
    );
    fs::remove_dir_all(directory).expect("the test directory can be removed");
}

#[test]
fn should_compile_named_reexport_dependency_given_entry_file_when_running_compiler_cli() {
    // Arrange
    let directory =
        std::env::temp_dir().join(format!("tsrzl-named-reexport-{}", std::process::id()));
    let entry_path = directory.join("main.ts");
    let exported_path = directory.join("answer.ts");
    let output_directory = directory.join("out");
    fs::create_dir_all(&directory).expect("the test directory can be created");
    fs::write(&entry_path, "export { answer as renamed } from './answer';")
        .expect("the entry TypeScript source can be written");
    fs::write(&exported_path, "export const answer: number = 42;")
        .expect("the re-exported TypeScript source can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--out-dir")
        .arg(&output_directory)
        .arg(&entry_path)
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    assert_eq!(
        fs::read_to_string(output_directory.join("main.js"))
            .expect("the entry JavaScript output can be read"),
        "export { answer as renamed } from './answer';\n"
    );
    assert_eq!(
        fs::read_to_string(output_directory.join("answer.js"))
            .expect("the re-exported JavaScript output can be read"),
        "export const answer = 42;\n"
    );
    fs::remove_dir_all(directory).expect("the test directory can be removed");
}

#[test]
fn should_compile_files_from_jsonc_project_given_project_path_when_running_compiler_cli() {
    // Arrange
    let directory = std::env::temp_dir().join(format!("tsrzl-project-{}", std::process::id()));
    let config_path = directory.join("tsconfig.json");
    let input_path = directory.join("main.ts");
    fs::create_dir_all(&directory).expect("the project directory can be created");
    fs::write(
        &config_path,
        "{\n  // Explicit project roots.\n  \"files\": [\"main.ts\",],\n}\n",
    )
    .expect("the project configuration can be written");
    fs::write(&input_path, "const answer: number = 42;")
        .expect("the project source can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--project")
        .arg(&directory)
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    assert_eq!(
        fs::read_to_string(directory.join("main.js"))
            .expect("the project JavaScript output can be read"),
        "const answer = 42;\n"
    );
    fs::remove_dir_all(directory).expect("the project directory can be removed");
}

#[test]
fn should_use_project_module_option_given_commonjs_configuration_when_running_compiler_cli() {
    // Arrange
    let directory =
        std::env::temp_dir().join(format!("tsrzl-project-module-{}", std::process::id()));
    let config_path = directory.join("tsconfig.json");
    let input_path = directory.join("main.ts");
    fs::create_dir_all(&directory).expect("the project directory can be created");
    fs::write(
        &config_path,
        "{\"files\":[\"main.ts\"],\"compilerOptions\":{\"module\":\"CommonJS\"}}",
    )
    .expect("the project configuration can be written");
    fs::write(&input_path, "export const answer: number = 42;")
        .expect("the project source can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--project")
        .arg(&config_path)
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    assert_eq!(
        fs::read_to_string(directory.join("main.js"))
            .expect("the project JavaScript output can be read"),
        "\"use strict\";\nObject.defineProperty(exports, \"__esModule\", { value: true });\nexports.answer = void 0;\nexports.answer = 42;\n"
    );
    fs::remove_dir_all(directory).expect("the project directory can be removed");
}

#[test]
fn should_use_project_output_directory_given_out_dir_configuration_when_running_compiler_cli() {
    // Arrange
    let directory =
        std::env::temp_dir().join(format!("tsrzl-project-outdir-{}", std::process::id()));
    let config_path = directory.join("tsconfig.json");
    let input_path = directory.join("main.ts");
    fs::create_dir_all(&directory).expect("the project directory can be created");
    fs::write(
        &config_path,
        "{\"files\":[\"main.ts\"],\"compilerOptions\":{\"outDir\":\"dist\"}}",
    )
    .expect("the project configuration can be written");
    fs::write(&input_path, "const answer: number = 42;")
        .expect("the project source can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--project")
        .arg(&config_path)
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    assert_eq!(
        fs::read_to_string(directory.join("dist/main.js"))
            .expect("the configured JavaScript output can be read"),
        "const answer = 42;\n"
    );
    fs::remove_dir_all(directory).expect("the project directory can be removed");
}

#[test]
fn should_emit_only_project_declarations_given_emit_declaration_only_configuration_when_running_compiler_cli()
 {
    // Arrange
    let directory =
        std::env::temp_dir().join(format!("tsrzl-project-declarations-{}", std::process::id()));
    let config_path = directory.join("tsconfig.json");
    let input_path = directory.join("main.ts");
    fs::create_dir_all(&directory).expect("the project directory can be created");
    fs::write(
        &config_path,
        "{\"files\":[\"main.ts\"],\"compilerOptions\":{\"declaration\":true,\"emitDeclarationOnly\":true}}",
    )
    .expect("the project configuration can be written");
    fs::write(&input_path, "export const answer: number = 42;")
        .expect("the project source can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--project")
        .arg(&config_path)
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    assert_eq!(
        fs::read_to_string(directory.join("main.d.ts"))
            .expect("the project declaration output can be read"),
        "export declare const answer: number;\n"
    );
    assert!(!directory.join("main.js").exists());
    fs::remove_dir_all(directory).expect("the project directory can be removed");
}

#[test]
fn should_emit_declaration_file_given_declaration_option_when_running_compiler_cli() {
    // Arrange
    let directory = std::env::temp_dir().join(format!("tsrzl-declaration-{}", std::process::id()));
    let input_path = directory.join("input.ts");
    let output_directory = directory.join("out");
    fs::create_dir_all(&directory).expect("the test directory can be created");
    fs::write(&input_path, "export const answer: number = 42;")
        .expect("the TypeScript input can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--declaration")
        .arg("--out-dir")
        .arg(&output_directory)
        .arg(&input_path)
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    assert_eq!(
        fs::read_to_string(output_directory.join("input.d.ts"))
            .expect("the declaration output can be read"),
        "export declare const answer: number;\n"
    );
    fs::remove_dir_all(directory).expect("the test directory can be removed");
}

#[test]
fn should_preserve_inferred_const_literal_given_declaration_option_when_running_compiler_cli() {
    // Arrange
    let directory =
        std::env::temp_dir().join(format!("tsrzl-const-declaration-{}", std::process::id()));
    let input_path = directory.join("input.ts");
    let output_directory = directory.join("out");
    fs::create_dir_all(&directory).expect("the test directory can be created");
    fs::write(&input_path, "export const answer = 42;")
        .expect("the TypeScript input can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--declaration")
        .arg("--out-dir")
        .arg(&output_directory)
        .arg(&input_path)
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    assert_eq!(
        fs::read_to_string(output_directory.join("input.d.ts"))
            .expect("the declaration output can be read"),
        "export declare const answer = 42;\n"
    );
    fs::remove_dir_all(directory).expect("the test directory can be removed");
}

#[test]
fn should_preserve_default_literal_given_declaration_option_when_running_compiler_cli() {
    // Arrange
    let directory =
        std::env::temp_dir().join(format!("tsrzl-default-declaration-{}", std::process::id()));
    let input_path = directory.join("input.ts");
    let output_directory = directory.join("out");
    fs::create_dir_all(&directory).expect("the test directory can be created");
    fs::write(&input_path, "export default 42;").expect("the TypeScript input can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--declaration")
        .arg("--out-dir")
        .arg(&output_directory)
        .arg(&input_path)
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    assert_eq!(
        fs::read_to_string(output_directory.join("input.d.ts"))
            .expect("the declaration output can be read"),
        "declare const _default = 42;\nexport default _default;\n"
    );
    fs::remove_dir_all(directory).expect("the test directory can be removed");
}

#[test]
fn should_emit_only_declarations_given_emit_declaration_only_option_when_running_compiler_cli() {
    // Arrange
    let directory =
        std::env::temp_dir().join(format!("tsrzl-declarations-only-{}", std::process::id()));
    let input_path = directory.join("input.ts");
    let output_directory = directory.join("out");
    fs::create_dir_all(&directory).expect("the test directory can be created");
    fs::write(&input_path, "export const answer: number = 42;")
        .expect("the TypeScript input can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--declaration")
        .arg("--emitDeclarationOnly")
        .arg("--out-dir")
        .arg(&output_directory)
        .arg(&input_path)
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    assert_eq!(
        fs::read_to_string(output_directory.join("input.d.ts"))
            .expect("the declaration output can be read"),
        "export declare const answer: number;\n"
    );
    assert!(!output_directory.join("input.js").exists());
    fs::remove_dir_all(directory).expect("the test directory can be removed");
}

#[test]
fn should_reject_emit_declaration_only_without_declaration_when_running_compiler_cli() {
    // Arrange
    let directory = std::env::temp_dir().join(format!(
        "tsrzl-invalid-declarations-only-{}",
        std::process::id()
    ));
    let input_path = directory.join("input.ts");
    fs::create_dir_all(&directory).expect("the test directory can be created");
    fs::write(&input_path, "export const answer: number = 42;")
        .expect("the TypeScript input can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--emitDeclarationOnly")
        .arg(&input_path)
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    assert_eq!(process.status.code(), Some(2));
    assert_eq!(
        String::from_utf8_lossy(&process.stderr),
        "error TS5069: Option 'emitDeclarationOnly' cannot be specified without specifying option 'declaration' or option 'composite'.\n"
    );
    fs::remove_dir_all(directory).expect("the test directory can be removed");
}

#[test]
fn should_reject_removed_project_target_given_es5_configuration_when_running_compiler_cli() {
    // Arrange
    let directory =
        std::env::temp_dir().join(format!("tsrzl-project-target-{}", std::process::id()));
    let config_path = directory.join("tsconfig.json");
    let input_path = directory.join("main.ts");
    fs::create_dir_all(&directory).expect("the project directory can be created");
    fs::write(
        &config_path,
        "{\"files\":[\"main.ts\"],\"compilerOptions\":{\"target\":\"ES5\"}}",
    )
    .expect("the project configuration can be written");
    fs::write(&input_path, "const answer: number = 42;")
        .expect("the project source can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--project")
        .arg(&config_path)
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    assert!(!process.status.success());
    assert_eq!(
        String::from_utf8_lossy(&process.stderr),
        "error TS5108: Option 'target=ES5' has been removed. Please remove it from your configuration.\n"
    );
    fs::remove_dir_all(directory).expect("the project directory can be removed");
}

#[test]
fn should_limit_project_sources_given_include_pattern_when_running_compiler_cli() {
    // Arrange
    let directory =
        std::env::temp_dir().join(format!("tsrzl-project-include-{}", std::process::id()));
    let config_path = directory.join("tsconfig.json");
    let included_path = directory.join("src/main.ts");
    let excluded_path = directory.join("outside.ts");
    fs::create_dir_all(directory.join("src")).expect("the source directory can be created");
    fs::write(&config_path, "{\"include\":[\"src/**/*.ts\"]}")
        .expect("the project configuration can be written");
    fs::write(&included_path, "const included: number = 42;")
        .expect("the included project source can be written");
    fs::write(&excluded_path, "const excluded: number = 42;")
        .expect("the outside project source can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--project")
        .arg(&config_path)
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    assert!(directory.join("src/main.js").exists());
    assert!(!directory.join("outside.js").exists());
    fs::remove_dir_all(directory).expect("the project directory can be removed");
}

#[test]
fn should_exclude_project_sources_given_exclude_pattern_when_running_compiler_cli() {
    // Arrange
    let directory =
        std::env::temp_dir().join(format!("tsrzl-project-exclude-{}", std::process::id()));
    let config_path = directory.join("tsconfig.json");
    let included_path = directory.join("main.ts");
    let excluded_path = directory.join("generated/internal.ts");
    fs::create_dir_all(directory.join("generated"))
        .expect("the generated source directory can be created");
    fs::write(
        &config_path,
        "{\"include\":[\"**/*.ts\"],\"exclude\":[\"generated/**\"]}",
    )
    .expect("the project configuration can be written");
    fs::write(&included_path, "const answer: number = 42;")
        .expect("the included project source can be written");
    fs::write(&excluded_path, "const generated: number = 42;")
        .expect("the excluded project source can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--project")
        .arg(&config_path)
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    assert!(directory.join("main.js").exists());
    assert!(!directory.join("generated/internal.js").exists());
    fs::remove_dir_all(directory).expect("the project directory can be removed");
}

#[test]
fn should_discover_default_project_sources_given_config_without_files_when_running_compiler_cli() {
    // Arrange
    let directory =
        std::env::temp_dir().join(format!("tsrzl-project-default-{}", std::process::id()));
    fs::create_dir_all(&directory).expect("the project directory can be created");
    fs::write(directory.join("tsconfig.json"), "{}")
        .expect("the project configuration can be written");
    fs::write(directory.join("main.ts"), "const answer: number = 42;")
        .expect("the default project source can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .current_dir(&directory)
        .arg("--project")
        .arg("tsconfig.json")
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    assert_eq!(
        fs::read_to_string(directory.join("main.js"))
            .expect("the default project output can be read"),
        "const answer = 42;\n"
    );
    fs::remove_dir_all(directory).expect("the project directory can be removed");
}

#[test]
fn should_inherit_compiler_options_given_extended_project_configuration_when_running_compiler_cli()
{
    // Arrange
    let directory =
        std::env::temp_dir().join(format!("tsrzl-project-extends-{}", std::process::id()));
    let base_config_path = directory.join("base.json");
    let config_path = directory.join("tsconfig.json");
    let input_path = directory.join("main.ts");
    fs::create_dir_all(&directory).expect("the project directory can be created");
    fs::write(
        &base_config_path,
        "{\"compilerOptions\":{\"module\":\"CommonJS\"}}",
    )
    .expect("the base project configuration can be written");
    fs::write(
        &config_path,
        "{\"extends\":\"./base\",\"files\":[\"main.ts\"]}",
    )
    .expect("the derived project configuration can be written");
    fs::write(&input_path, "export const answer: number = 42;")
        .expect("the project source can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--project")
        .arg(&config_path)
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    assert_eq!(
        fs::read_to_string(directory.join("main.js"))
            .expect("the project JavaScript output can be read"),
        "\"use strict\";\nObject.defineProperty(exports, \"__esModule\", { value: true });\nexports.answer = void 0;\nexports.answer = 42;\n"
    );
    fs::remove_dir_all(directory).expect("the project directory can be removed");
}

#[test]
fn should_allow_null_assignment_given_disabled_strict_null_checks_when_running_compiler_cli() {
    // Arrange
    let directory = std::env::temp_dir().join(format!("tsrzl-null-checks-{}", std::process::id()));
    let input_path = directory.join("input.ts");
    let output_directory = directory.join("out");
    fs::create_dir_all(&directory).expect("the test directory can be created");
    fs::write(&input_path, "const value: number = null;")
        .expect("the TypeScript input can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--strictNullChecks")
        .arg("false")
        .arg("--out-dir")
        .arg(&output_directory)
        .arg(&input_path)
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    assert_eq!(
        fs::read_to_string(output_directory.join("input.js"))
            .expect("the JavaScript output can be read"),
        "const value = null;\n"
    );
    fs::remove_dir_all(directory).expect("the test directory can be removed");
}

#[test]
fn should_allow_null_given_disabled_strict_null_checks_in_tsconfig_when_running_cli() {
    // Arrange
    let directory = std::env::temp_dir().join(format!("tsrzl-null-project-{}", std::process::id()));
    let config_path = directory.join("tsconfig.json");
    let input_path = directory.join("input.ts");
    fs::create_dir_all(&directory).expect("the project directory can be created");
    fs::write(
        &config_path,
        r#"{"compilerOptions":{"strictNullChecks":false},"files":["input.ts"]}"#,
    )
    .expect("the project configuration can be written");
    fs::write(&input_path, "const value: number = null;")
        .expect("the TypeScript input can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--project")
        .arg(&config_path)
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    assert_eq!(
        fs::read_to_string(directory.join("input.js")).expect("the JavaScript output can be read"),
        "const value = null;\n"
    );
    fs::remove_dir_all(directory).expect("the project directory can be removed");
}

#[test]
fn should_build_referenced_project_given_project_reference_when_running_compiler_cli() {
    // Arrange
    let directory =
        std::env::temp_dir().join(format!("tsrzl-project-reference-{}", std::process::id()));
    let library_directory = directory.join("library");
    let application_directory = directory.join("application");
    fs::create_dir_all(&library_directory).expect("the library project directory can be created");
    fs::create_dir_all(&application_directory)
        .expect("the application project directory can be created");
    fs::write(
        library_directory.join("tsconfig.json"),
        r#"{"compilerOptions":{"composite":true,"declaration":true,"outDir":"dist"},"include":["index.ts"]}"#,
    )
    .expect("the library project configuration can be written");
    fs::write(
        library_directory.join("index.ts"),
        "export const answer: number = 42;",
    )
    .expect("the library source can be written");
    fs::write(
        application_directory.join("tsconfig.json"),
        r#"{"compilerOptions":{"composite":true,"outDir":"dist"},"references":[{"path":"../library"}],"include":["main.ts"]}"#,
    )
    .expect("the application project configuration can be written");
    fs::write(
        application_directory.join("main.ts"),
        "import { answer } from '../library/index'; export const result = answer;",
    )
    .expect("the application source can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--build")
        .arg(application_directory.join("tsconfig.json"))
        .output()
        .expect("the compiler CLI can be started");
    let status_succeeded = process.status.success();
    let standard_error = String::from_utf8_lossy(&process.stderr).into_owned();
    let library_javascript_exists = library_directory.join("dist/index.js").is_file();
    let application_javascript_exists = application_directory.join("dist/main.js").is_file();
    fs::remove_dir_all(&directory).expect("the test directory can be removed");

    // Assert
    assert!(status_succeeded, "{standard_error}");
    assert!(library_javascript_exists);
    assert!(application_javascript_exists);
}
