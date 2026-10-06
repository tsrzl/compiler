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

// Pinned TypeScript fixture: conformance/internalModules/DeclarationMerging/ClassAndModuleWithSameNameAndCommonRoot.ts.
#[test]
fn should_emit_class_namespace_member_given_exported_namespace_value_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("widget.ts"),
        "class Widget {}\nnamespace Widget { export const label = \"widget\"; }\nconst value: string = Widget.label;\n",
    )
    .expect("the class and namespace source path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2015);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    let javascript = result
        .emitted_files()
        .iter()
        .find(|file| file.path().file_name().and_then(|name| name.to_str()) == Some("widget.js"))
        .expect("the class and namespace source emits JavaScript")
        .text();
    assert!(
        javascript.contains("Widget.label = \"widget\";"),
        "the namespace member should augment the class value; got {javascript:?}"
    );
}

// Pinned TypeScript input: compiler/augmentedTypesModules.ts (namespace-before-class case).
#[test]
fn should_report_namespace_before_class_given_instantiated_namespace_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("merge-order.ts"),
        "namespace Widget { export const label = \"widget\"; }\nclass Widget {}\n",
    )
    .expect("the namespace and class source path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2434),
        "the instantiated namespace before its merged class should report TS2434; got {:?}",
        result.diagnostics()
    );
}

// Pinned TypeScript fixture: conformance/internalModules/DeclarationMerging/ClassAndModuleWithSameNameAndCommonRootES6.ts.
#[test]
fn should_report_cross_file_namespace_merge_given_class_in_another_source_file_when_checking_types()
{
    // Arrange
    let class = SourceFile::from_path(Path::new("class.ts"), "class Widget {}\n")
        .expect("the class source path has a supported source kind");
    let namespace = SourceFile::from_path(
        Path::new("namespace.ts"),
        "namespace Widget { export const label = \"widget\"; }\n",
    )
    .expect("the namespace source path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([class, namespace]);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2433),
        "a class and its merging namespace in separate files should report TS2433; got {:?}",
        result.diagnostics()
    );
}
