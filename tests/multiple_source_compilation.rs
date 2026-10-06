use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_emit_sources_in_input_order_given_multiple_typescript_files_when_compiling_sources() {
    // Arrange
    let sources = [
        SourceFile::from_path(Path::new("first.ts"), "const first: number = 1;")
            .expect("a TypeScript path has a supported source kind"),
        SourceFile::from_path(Path::new("second.ts"), "const second: string = 'two';")
            .expect("a TypeScript path has a supported source kind"),
    ];

    // Act
    let result = Compiler::new().compile_sources(sources);

    // Assert
    let emitted_text = result
        .emitted_files()
        .iter()
        .map(tsrzl::compiler::EmittedFile::text)
        .collect::<Vec<_>>();
    assert_eq!(
        emitted_text,
        ["const first = 1;\n", "const second = 'two';\n"]
    );
}

#[test]
fn should_resolve_global_type_given_type_alias_in_another_script_when_compiling_sources() {
    // Arrange
    let declaration_source = SourceFile::from_path(Path::new("types.ts"), "type Label = string;")
        .expect("a TypeScript path has a supported source kind");
    let using_source = SourceFile::from_path(Path::new("main.ts"), "const label: Label = 'ready';")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([declaration_source, using_source]);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_resolve_global_interface_given_interface_in_another_script_when_compiling_sources() {
    // Arrange
    let declaration_source =
        SourceFile::from_path(Path::new("types.ts"), "interface User { name: string; }")
            .expect("a TypeScript path has a supported source kind");
    let using_source =
        SourceFile::from_path(Path::new("main.ts"), "const user: User = { name: 'Ada' };")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([declaration_source, using_source]);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_resolve_imported_enum_member_given_enum_type_when_compiling_sources() {
    // Arrange
    let enum_source = SourceFile::from_path(
        Path::new("direction.ts"),
        "export enum Direction { Up, Down }",
    )
    .expect("a TypeScript path has a supported source kind");
    let using_source = SourceFile::from_path(
        Path::new("main.ts"),
        "import { Direction } from './direction'; const direction: Direction = Direction.Up;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([enum_source, using_source]);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_assign_file_ids_in_input_order_given_multiple_sources_when_compiling_sources() {
    // Arrange
    let sources = [
        SourceFile::from_path(Path::new("first.ts"), "const first = 1;")
            .expect("a TypeScript path has a supported source kind"),
        SourceFile::from_path(Path::new("second.ts"), "const second = 2;")
            .expect("a TypeScript path has a supported source kind"),
    ];

    // Act
    let result = Compiler::new().compile_sources(sources);

    // Assert
    let file_ids = result
        .syntax_trees()
        .iter()
        .map(|tree| tree.file_id().index())
        .collect::<Vec<_>>();
    assert_eq!(file_ids, [0, 1]);
}

#[test]
fn should_attribute_diagnostic_to_second_file_given_error_in_second_source_when_compiling_sources()
{
    // Arrange
    let sources = [
        SourceFile::from_path(Path::new("first.ts"), "const first = 1;")
            .expect("a TypeScript path has a supported source kind"),
        SourceFile::from_path(Path::new("second.ts"), "const second: number = 'two';")
            .expect("a TypeScript path has a supported source kind"),
    ];

    // Act
    let result = Compiler::new().compile_sources(sources);

    // Assert
    let files = result
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.file().map(tsrzl::source_file::FileId::index))
        .collect::<Vec<_>>();
    assert_eq!(files, [Some(1)]);
}

#[test]
fn should_order_diagnostics_by_input_file_given_earlier_offset_in_later_file_when_compiling_sources()
 {
    // Arrange
    let sources = [
        SourceFile::from_path(
            Path::new("first.ts"),
            "const padding = 0; const a: number = 'a';",
        )
        .expect("a TypeScript path has a supported source kind"),
        SourceFile::from_path(Path::new("second.ts"), "const b: number = 'b';")
            .expect("a TypeScript path has a supported source kind"),
    ];

    // Act
    let result = Compiler::new().compile_sources(sources);

    // Assert
    let files = result
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.file().map(tsrzl::source_file::FileId::index))
        .collect::<Vec<_>>();
    assert_eq!(files, [Some(0), Some(1)]);
}
