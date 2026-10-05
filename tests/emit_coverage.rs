use std::fs;
use std::path::Path;
use std::process::Command;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_lower_exponentiation_operator_given_es2015_target_when_emitting_javascript() {
    // Pinned TypeScript case: conformance/es7/exponentiationOperator/emitExponentiationOperator1.ts.
    // Its target=es2015 baseline lowers `2 ** 3` to `Math.pow(2, 3)`.
    // Arrange
    let source = SourceFile::from_path(Path::new("answer.ts"), "const answer: number = 2 ** 3;")
        .expect("the TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2015);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    assert!(
        result.diagnostics().is_empty(),
        "exponentiation is valid syntax: {:?}",
        result.diagnostics()
    );
    assert!(
        result.emitted_files()[0].text().contains("Math.pow(2, 3)"),
        "ES2015 output should lower exponentiation to Math.pow: {}",
        result.emitted_files()[0].text()
    );
}

#[test]
fn should_lower_object_spread_given_es2015_target_when_emitting_javascript() {
    // Pinned TypeScript case: conformance/types/spread/objectSpread.ts (target ES2015).
    // Direct TypeScript-Go output uses Object.assign for object spread.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("spread.ts"),
        "const left = { value: 1 }; const merged = { ...left, extra: 2 };",
    )
    .expect("the TypeScript path has a supported source kind");
    let options = CompilerOptions::new(ScriptTarget::Es2015);

    // Act
    let result = Compiler::with_options(options).compile(source);

    // Assert
    assert!(
        result.diagnostics().is_empty(),
        "object spread is valid syntax: {:?}",
        result.diagnostics()
    );
    assert!(
        result.emitted_files()[0].text().contains("Object.assign"),
        "ES2015 output should lower object spread: {}",
        result.emitted_files()[0].text()
    );
}

#[test]
fn should_map_computed_class_members_given_source_map_option_when_emitting_javascript() {
    // Arrange
    let directory = std::env::temp_dir().join(format!("tsrzl-computed-map-{}", std::process::id()));
    let input_path = directory.join("computedPropertyNamesSourceMap1_ES6.ts");
    let output_directory = directory.join("out");
    fs::create_dir_all(&directory).expect("the test directory can be created");
    fs::write(
        &input_path,
        "class C {\n    [\"hello\"]() {\n        debugger;\n\t}\n\tget [\"goodbye\"]() {\n\t\treturn 0;\n\t}\n}\n",
    )
    .expect("the TypeScript input can be written");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--target")
        .arg("es2015")
        .arg("--sourceMap")
        .arg("--outDir")
        .arg(&output_directory)
        .arg(&input_path)
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    // The mapping segments match the pinned TypeScript-Go ES6 baseline for
    // computedPropertyNamesSourceMap1_ES6.js.map.
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    let source_map =
        fs::read_to_string(output_directory.join("computedPropertyNamesSourceMap1_ES6.js.map"))
            .expect("the source map can be read");
    assert!(source_map.contains(
        "\"mappings\":\";AAAA,MAAM,CAAC;IACH,CAAC,OAAO,CAAC;QACL,SAAS;IAChB,CAAC;IACD,IAAI,CAAC,SAAS,CAAC;QACd,OAAO,CAAC,CAAC;IACV,CAAC;CACD\""
    ));
    fs::remove_dir_all(directory).expect("the test directory can be removed");
}
