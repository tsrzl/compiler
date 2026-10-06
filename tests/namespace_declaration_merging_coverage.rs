use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

// Pinned TypeScript fixture: conformance/internalModules/DeclarationMerging/TwoInternalModulesWithTheSameNameAndSameCommonRoot.ts.
#[test]
fn should_emit_members_from_merged_namespaces_given_multiple_source_files_when_compiling() {
    // Arrange
    let first = SourceFile::from_path(
        Path::new("part1.ts"),
        "namespace Box { export const left = 1; }\n",
    )
    .expect("the first namespace source path has a supported source kind");
    let second = SourceFile::from_path(
        Path::new("part2.ts"),
        "namespace Box { export const right = 2; }\n",
    )
    .expect("the second namespace source path has a supported source kind");
    let consumer = SourceFile::from_path(
        Path::new("consumer.ts"),
        "const total: number = Box.left + Box.right;\n",
    )
    .expect("the consumer source path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2015);

    // Act
    let result = Compiler::with_options(options).compile_sources([first, second, consumer]);

    // Assert
    let first_output = result
        .emitted_files()
        .iter()
        .find(|file| file.path().file_name().and_then(|name| name.to_str()) == Some("part1.js"))
        .expect("the first namespace source emits JavaScript")
        .text();
    let second_output = result
        .emitted_files()
        .iter()
        .find(|file| file.path().file_name().and_then(|name| name.to_str()) == Some("part2.js"))
        .expect("the second namespace source emits JavaScript")
        .text();
    assert!(first_output.contains("Box.left = 1;"), "{first_output:?}");
    assert!(
        second_output.contains("Box.right = 2;"),
        "{second_output:?}"
    );
}
