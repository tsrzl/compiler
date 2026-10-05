use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_missing_side_effect_import_given_unresolved_module_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(Path::new("main.ts"), "import './missing';")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2882);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Cannot find module or type declarations for side-effect import of './missing'."
    );
}

#[test]
fn should_report_missing_named_import_module_given_unresolved_module_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(Path::new("main.ts"), "import { answer } from './missing';")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2307);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Cannot find module './missing' or its corresponding type declarations."
    );
}

#[test]
fn should_resolve_package_from_parent_node_modules_given_nested_importer_when_compiling_sources() {
    // Arrange
    let importing_source = SourceFile::from_path(
        Path::new("project/src/app.ts"),
        "import { answer } from \"answer-package\"; const result: number = answer;",
    )
    .expect("a TypeScript path has a supported source kind");
    let package_source = SourceFile::from_path(
        Path::new("project/node_modules/answer-package/index.ts"),
        "export const answer: number = 42;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([importing_source, package_source]);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_check_imported_value_type_given_named_import_when_compiling_sources() {
    // Arrange
    let importing_source = SourceFile::from_path(
        Path::new("main.ts"),
        "import { answer as value } from './answer'; const label: string = value;",
    )
    .expect("a TypeScript path has a supported source kind");
    let imported_source =
        SourceFile::from_path(Path::new("answer.ts"), "export const answer: number = 42;")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([importing_source, imported_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'number' is not assignable to type 'string'."
    );
}

#[test]
fn should_resolve_imported_class_type_given_type_only_import_when_compiling_sources() {
    // Arrange
    let importing_source = SourceFile::from_path(
        Path::new("main.ts"),
        "import type { Greeter } from './greeter'; function accept(value: Greeter): void {}",
    )
    .expect("a TypeScript path has a supported source kind");
    let exported_source = SourceFile::from_path(Path::new("greeter.ts"), "export class Greeter {}")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([importing_source, exported_source]);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_report_missing_named_export_given_imported_module_when_compiling_sources() {
    // Arrange
    let importing_source =
        SourceFile::from_path(Path::new("main.ts"), "import { missing } from './answer';")
            .expect("a TypeScript path has a supported source kind");
    let imported_source =
        SourceFile::from_path(Path::new("answer.ts"), "export const answer = 42;")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([importing_source, imported_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2305);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Module '\"./answer\"' has no exported member 'missing'."
    );
}

#[test]
fn should_bind_missing_imported_name_given_missing_named_export_when_compiling_sources() {
    // Arrange
    let importing_source = SourceFile::from_path(
        Path::new("main.ts"),
        "import { missing } from './answer'; const copy = missing;",
    )
    .expect("a TypeScript path has a supported source kind");
    let imported_source =
        SourceFile::from_path(Path::new("answer.ts"), "export const answer = 42;")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([importing_source, imported_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2305);
}

#[test]
fn should_check_default_import_type_given_default_export_when_compiling_sources() {
    // Arrange
    let importing_source = SourceFile::from_path(
        Path::new("main.ts"),
        "import answer from './answer'; const label: string = answer;",
    )
    .expect("a TypeScript path has a supported source kind");
    let imported_source = SourceFile::from_path(Path::new("answer.ts"), "export default 42;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([importing_source, imported_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'number' is not assignable to type 'string'."
    );
}

#[test]
fn should_report_missing_default_export_given_default_import_when_compiling_sources() {
    // Arrange
    let importing_source =
        SourceFile::from_path(Path::new("main.ts"), "import answer from './answer';")
            .expect("a TypeScript path has a supported source kind");
    let imported_source =
        SourceFile::from_path(Path::new("answer.ts"), "export const answer = 42;")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([importing_source, imported_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 1192);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Module '\"./answer\"' has no default export."
    );
}

#[test]
fn should_check_namespace_member_type_given_namespace_import_when_compiling_sources() {
    // Arrange
    let importing_source = SourceFile::from_path(
        Path::new("main.ts"),
        "import * as answerModule from './answer'; const label: string = answerModule.answer;",
    )
    .expect("a TypeScript path has a supported source kind");
    let imported_source =
        SourceFile::from_path(Path::new("answer.ts"), "export const answer: number = 42;")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([importing_source, imported_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'number' is not assignable to type 'string'."
    );
}

#[test]
fn should_resolve_reexported_value_type_given_export_all_when_compiling_sources() {
    // Arrange
    let importing_source = SourceFile::from_path(
        Path::new("main.ts"),
        "import { answer } from './barrel'; const label: string = answer;",
    )
    .expect("a TypeScript path has a supported source kind");
    let barrel_source = SourceFile::from_path(Path::new("barrel.ts"), "export * from './answer';")
        .expect("a TypeScript path has a supported source kind");
    let exported_source =
        SourceFile::from_path(Path::new("answer.ts"), "export const answer: number = 42;")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let result =
        Compiler::new().compile_sources([importing_source, barrel_source, exported_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'number' is not assignable to type 'string'."
    );
}

#[test]
fn should_resolve_named_reexported_value_type_given_imported_alias_when_compiling_sources() {
    // Arrange
    let importing_source = SourceFile::from_path(
        Path::new("main.ts"),
        "import { renamed } from './barrel'; const label: string = renamed;",
    )
    .expect("a TypeScript path has a supported source kind");
    let barrel_source = SourceFile::from_path(
        Path::new("barrel.ts"),
        "export { answer as renamed } from './answer';",
    )
    .expect("a TypeScript path has a supported source kind");
    let exported_source =
        SourceFile::from_path(Path::new("answer.ts"), "export const answer: number = 42;")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let result =
        Compiler::new().compile_sources([importing_source, barrel_source, exported_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'number' is not assignable to type 'string'."
    );
}

#[test]
fn should_resolve_named_reexported_type_given_imported_alias_when_compiling_sources() {
    // Arrange
    let importing_source = SourceFile::from_path(
        Path::new("main.ts"),
        "import type { Renamed } from './barrel'; const value: Renamed = 42;",
    )
    .expect("a TypeScript path has a supported source kind");
    let barrel_source = SourceFile::from_path(
        Path::new("barrel.ts"),
        "export { Answer as Renamed } from './answer';",
    )
    .expect("a TypeScript path has a supported source kind");
    let exported_source =
        SourceFile::from_path(Path::new("answer.ts"), "export type Answer = string;")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let result =
        Compiler::new().compile_sources([importing_source, barrel_source, exported_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'number' is not assignable to type 'string'."
    );
}

#[test]
fn should_resolve_type_only_named_reexport_given_imported_alias_when_compiling_sources() {
    // Arrange
    let importing_source = SourceFile::from_path(
        Path::new("main.ts"),
        "import type { Renamed } from './barrel'; const value: Renamed = 42;",
    )
    .expect("a TypeScript path has a supported source kind");
    let barrel_source = SourceFile::from_path(
        Path::new("barrel.ts"),
        "export type { Answer as Renamed } from './answer';",
    )
    .expect("a TypeScript path has a supported source kind");
    let exported_source =
        SourceFile::from_path(Path::new("answer.ts"), "export type Answer = string;")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let result =
        Compiler::new().compile_sources([importing_source, barrel_source, exported_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'number' is not assignable to type 'string'."
    );
}

#[test]
fn should_resolve_type_only_local_export_given_imported_alias_when_compiling_sources() {
    // Arrange
    let importing_source = SourceFile::from_path(
        Path::new("main.ts"),
        "import type { PublicAnswer } from './answer'; const value: PublicAnswer = 42;",
    )
    .expect("a TypeScript path has a supported source kind");
    let exported_source = SourceFile::from_path(
        Path::new("answer.ts"),
        "type Answer = string; export type { Answer as PublicAnswer };",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([importing_source, exported_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'number' is not assignable to type 'string'."
    );
}

#[test]
fn should_report_missing_reexported_member_given_named_reexport_when_compiling_sources() {
    // Arrange
    let barrel_source = SourceFile::from_path(
        Path::new("barrel.ts"),
        "export { missing as renamed } from './answer';",
    )
    .expect("a TypeScript path has a supported source kind");
    let exported_source =
        SourceFile::from_path(Path::new("answer.ts"), "export const answer = 42;")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([barrel_source, exported_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2305);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Module '\"./answer\"' has no exported member 'missing'."
    );
}

#[test]
fn should_report_missing_module_given_export_all_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(Path::new("main.ts"), "export * from './missing';")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2307);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Cannot find module './missing' or its corresponding type declarations."
    );
}

#[test]
fn should_resolve_dynamic_import_given_relative_module_when_compiling_sources() {
    // Arrange
    let entry = SourceFile::from_path(
        Path::new("main.ts"),
        "export const pending = import('./lazy');",
    )
    .expect("a TypeScript path has a supported source kind");
    let dependency =
        SourceFile::from_path(Path::new("lazy.ts"), "export const answer: number = 42;")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([entry, dependency]);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert!(
        result.emitted_files()[0]
            .text()
            .contains("import('./lazy')")
    );
}

#[test]
fn should_resolve_imported_type_in_its_module_scope_given_name_collision_when_compiling_sources() {
    // Arrange
    let importing_source = SourceFile::from_path(
        Path::new("main.ts"),
        "import type { Answer } from './answer'; const value: Answer = 42;",
    )
    .expect("a TypeScript path has a supported source kind");
    let imported_source =
        SourceFile::from_path(Path::new("answer.ts"), "export type Answer = string;")
            .expect("a TypeScript path has a supported source kind");
    let unrelated_source = SourceFile::from_path(
        Path::new("unrelated.ts"),
        "type Answer = number; const value: Answer = 42;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result =
        Compiler::new().compile_sources([importing_source, imported_source, unrelated_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'number' is not assignable to type 'string'."
    );
}

#[test]
fn should_resolve_reexported_type_in_its_module_scope_given_name_collision_when_compiling_sources()
{
    // Arrange
    let importing_source = SourceFile::from_path(
        Path::new("main.ts"),
        "import type { Answer } from './barrel'; const value: Answer = 42;",
    )
    .expect("a TypeScript path has a supported source kind");
    let barrel_source = SourceFile::from_path(Path::new("barrel.ts"), "export * from './answer';")
        .expect("a TypeScript path has a supported source kind");
    let imported_source =
        SourceFile::from_path(Path::new("answer.ts"), "export type Answer = string;")
            .expect("a TypeScript path has a supported source kind");
    let unrelated_source =
        SourceFile::from_path(Path::new("unrelated.ts"), "type Answer = number;")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([
        importing_source,
        barrel_source,
        imported_source,
        unrelated_source,
    ]);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'number' is not assignable to type 'string'."
    );
}

#[test]
fn should_resolve_imported_interface_in_its_module_scope_given_name_collision_when_compiling_sources()
 {
    // Arrange
    let importing_source = SourceFile::from_path(
        Path::new("main.ts"),
        "import type { User } from './user'; const user: User = { name: 42 };",
    )
    .expect("a TypeScript path has a supported source kind");
    let imported_source = SourceFile::from_path(
        Path::new("user.ts"),
        "export interface User { name: string; }",
    )
    .expect("a TypeScript path has a supported source kind");
    let unrelated_source = SourceFile::from_path(
        Path::new("unrelated.ts"),
        "interface User { name: number; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result =
        Compiler::new().compile_sources([importing_source, imported_source, unrelated_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'number' is not assignable to type 'string' for property 'name'."
    );
}

#[test]
fn should_bind_imported_type_alias_given_local_import_name_when_compiling_sources() {
    // Arrange
    let importing_source = SourceFile::from_path(
        Path::new("main.ts"),
        "import type { Answer as Value } from './answer'; const value: Value = 42;",
    )
    .expect("a TypeScript path has a supported source kind");
    let imported_source =
        SourceFile::from_path(Path::new("answer.ts"), "export type Answer = string;")
            .expect("a TypeScript path has a supported source kind");
    let global_source = SourceFile::from_path(Path::new("globals.ts"), "type Value = number;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result =
        Compiler::new().compile_sources([importing_source, imported_source, global_source]);

    // Assert
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), 2322);
    assert_eq!(
        result.diagnostics()[0].message(),
        "Type 'number' is not assignable to type 'string'."
    );
}
