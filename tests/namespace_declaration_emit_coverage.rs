use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;

// Pinned TypeScript-Go fixture: compiler/declarationEmitNamespaceMergedWithInterfaceNestedFunction.ts.
#[test]
fn should_emit_namespace_function_assignment_given_merged_interface_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("namespace.ts"),
        "interface Bar {}\nnamespace Bar { export function biz() { return 0; } }\n",
    )
    .expect("the namespace merge source path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2015).with_module(ModuleKind::CommonJs);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    let javascript = result
        .emitted_files()
        .iter()
        .find(|file| file.path().file_name().and_then(|name| name.to_str()) == Some("namespace.js"))
        .expect("the namespace merge source emits JavaScript")
        .text();
    assert!(
        javascript.contains("Bar.biz = biz;"),
        "the namespace IIFE should publish its exported function; got {javascript:?}"
    );
}

#[test]
fn should_emit_namespace_function_declaration_given_merged_interface_when_emitting_declarations() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("namespace.ts"),
        "interface Bar {}\nnamespace Bar { export function biz() { return 0; } }\n",
    )
    .expect("the namespace merge source path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2015)
        .with_module(ModuleKind::CommonJs)
        .with_declaration(true)
        .with_emit_declaration_only(true);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    let declaration = result
        .emitted_files()
        .iter()
        .find(|file| {
            file.path().file_name().and_then(|name| name.to_str()) == Some("namespace.d.ts")
        })
        .expect("the namespace merge source emits declarations")
        .text();
    assert!(
        declaration.contains("declare namespace Bar {\n    function biz(): number;\n}"),
        "the declaration should expose the namespace function; got {declaration:?}"
    );
}

// Pinned TypeScript-Go fixture: conformance/externalModules/typeOnly/nestedNamespace.ts.
#[test]
fn should_publish_exported_class_from_namespace_given_commonjs_module_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("a.ts"),
        "export namespace types {\n    export class A {}\n}\n",
    )
    .expect("the exported namespace source path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2015).with_module(ModuleKind::CommonJs);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    let javascript = result
        .emitted_files()
        .iter()
        .find(|file| file.path().file_name().and_then(|name| name.to_str()) == Some("a.js"))
        .expect("the exported namespace source emits JavaScript")
        .text();
    assert!(
        javascript.contains("types.A = A;"),
        "the namespace should publish its exported class; got {javascript:?}"
    );
}

// Pinned TypeScript-Go fixture: conformance/internalModules/moduleBody/moduleWithStatementsOfEveryKind.ts.
#[test]
fn should_keep_unexported_namespace_value_local_given_namespace_member_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("private-member.ts"),
        "namespace A { const hidden = 1; }\n",
    )
    .expect("the namespace source path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    let javascript = result.emitted_files()[0].text();
    assert!(
        javascript.contains("const hidden = 1;"),
        "the namespace IIFE should retain its private value locally; got {javascript:?}"
    );
}

#[test]
fn should_emit_nested_namespace_given_exported_namespace_member_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("nested.ts"),
        "namespace Y { export namespace Module { export class A {} } }\n",
    )
    .expect("the nested namespace source path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    let javascript = result.emitted_files()[0].text();
    assert!(
        javascript.contains("Y.Module") && javascript.contains("Module.A = A;"),
        "the nested namespace should publish its exported class through its parent; got {javascript:?}"
    );
}

// Pinned TypeScript fixture: conformance/internalModules/DeclarationMerging/ClassAndModuleWithSameNameAndCommonRootES6.ts.
#[test]
fn should_emit_dotted_namespace_member_given_qualified_namespace_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("class.ts"),
        "namespace X.Y { export class Point {} }\n",
    )
    .expect("the dotted namespace source path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    let javascript = result.emitted_files()[0].text();
    assert!(
        javascript.contains("X.Y = {}") && javascript.contains("Y.Point = Point;"),
        "the dotted namespace should create the qualified object and export its class; got {javascript:?}"
    );
}

#[test]
fn should_emit_enum_member_from_namespace_given_exported_enum_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("enum.ts"),
        "namespace Y { export enum Color { Blue, Red } }\n",
    )
    .expect("the namespace enum source path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    let javascript = result.emitted_files()[0].text();
    assert!(
        javascript.contains("Color = Y.Color || (Y.Color = {}));")
            && javascript.contains("Color[Color[\"Blue\"] = 0] = \"Blue\";"),
        "the namespace should publish its exported enum; got {javascript:?}"
    );
}

// Pinned TypeScript fixture: conformance/externalModules/typeOnly/nestedNamespace.ts.
#[test]
fn should_export_namespace_binding_given_external_module_namespace_when_emitting_commonjs() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("a.ts"),
        "export namespace types { export class A {} }\n",
    )
    .expect("the external namespace source path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2015).with_module(ModuleKind::CommonJs);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    let javascript = result.emitted_files()[0].text();
    assert!(
        javascript.contains("exports.types = types = {}"),
        "the exported namespace should initialize its CommonJS export; got {javascript:?}"
    );
}

// Pinned TypeScript fixture: conformance/externalModules/typeOnly/nestedNamespace.ts.
#[test]
fn should_emit_exported_class_given_namespace_member_when_emitting_declarations() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("a.ts"),
        "export namespace types { export class A {} }\n",
    )
    .expect("the external namespace source path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2015)
        .with_module(ModuleKind::CommonJs)
        .with_declaration(true);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    let declaration = result
        .emitted_files()
        .iter()
        .find(|file| {
            file.path()
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".d.ts"))
        })
        .expect("the exported namespace emits a declaration file")
        .text();
    assert!(
        declaration.contains("class A"),
        "the namespace declaration should retain its exported class; got {declaration:?}"
    );
}
