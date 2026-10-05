use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

struct TempProject {
    root: PathBuf,
}

impl TempProject {
    fn new(name: &str, source: &str, compiler_options: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "tsrzl-option-coverage-{}-{name}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("the temporary project directory can be created");
        fs::write(root.join("input.ts"), source).expect("the TypeScript input can be written");
        let configuration = format!(
            "{{\"compilerOptions\":{{\"target\":\"ES2015\",{compiler_options}}},\"files\":[\"input.ts\"]}}"
        );
        fs::write(root.join("tsconfig.json"), configuration)
            .expect("the project configuration can be written");

        Self { root }
    }

    fn compile(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tsrzl"))
            .arg("--project")
            .arg(&self.root)
            .output()
            .expect("the compiler CLI can be started")
    }

    fn compile_with_no_implicit_any_flag(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tsrzl"))
            .arg("--noImplicitAny")
            .arg(self.root.join("input.ts"))
            .output()
            .expect("the compiler CLI can be started")
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn assert_diagnostic(output: &Output, code: u16) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let expected = format!("TS{code}:");
    assert!(
        stderr.contains(&expected),
        "expected {expected}, got: {stderr}"
    );
}

#[test]
fn should_report_implicit_any_given_unannotated_parameter_when_compiling_project() {
    // Upstream: compiler/noImplicitAnyFunctions.ts
    // Arrange
    let project = TempProject::new(
        "no-implicit-any",
        "function f3(x) {}",
        r#""noImplicitAny": true"#,
    );

    // Act
    let output = project.compile();

    // Assert
    assert_diagnostic(&output, 7006);
}

#[test]
fn should_report_implicit_any_given_unannotated_parameter_when_using_cli_option() {
    // Upstream: compiler/noImplicitAnyFunctions.ts
    // Arrange
    let project = TempProject::new(
        "no-implicit-any-cli",
        "function f3(x) {}",
        "\"noImplicitAny\": false",
    );

    // Act
    let output = project.compile_with_no_implicit_any_flag();

    // Assert
    assert_diagnostic(&output, 7006);
}

#[test]
fn should_report_unused_local_given_unread_binding_when_compiling_project() {
    // Upstream: compiler/noUnusedLocals_writeOnly.ts
    // Arrange
    let project = TempProject::new(
        "no-unused-locals",
        "function run(): void { const unused = 1; } run();",
        r#""noUnusedLocals": true"#,
    );

    // Act
    let output = project.compile();

    // Assert
    assert_diagnostic(&output, 6133);
}

#[test]
fn should_report_unused_parameter_given_unread_parameter_when_compiling_project() {
    // Upstream: compiler/unusedSingleParameterInFunctionDeclaration.ts
    // Arrange
    let project = TempProject::new(
        "no-unused-parameters",
        "function greet(name: string): void {}",
        r#""noUnusedParameters": true"#,
    );

    // Act
    let output = project.compile();

    // Assert
    assert_diagnostic(&output, 6133);
}

#[test]
fn should_require_override_modifier_given_overriding_member_when_compiling_project() {
    // Upstream: conformance/override/override1.ts
    // Arrange
    let project = TempProject::new(
        "no-implicit-override",
        "class Base { save(): void {} }\nclass Derived extends Base { save(): void {} }",
        r#""noImplicitOverride": true"#,
    );

    // Act
    let output = project.compile();

    // Assert
    assert_diagnostic(&output, 4114);
}

#[test]
fn should_report_switch_fallthrough_given_nonterminating_case_when_compiling_project() {
    // Upstream: compiler/fallFromLastCase2.ts
    // Arrange
    let project = TempProject::new(
        "no-fallthrough",
        "function choose(value: number): number { switch (value) { case 0: value = 1; case 1: return 1; default: return 0; } }",
        r#""noFallthroughCasesInSwitch": true"#,
    );

    // Act
    let output = project.compile();

    // Assert
    assert_diagnostic(&output, 7029);
}

#[test]
fn should_report_explicit_undefined_given_exact_optional_property_when_compiling_project() {
    // Upstream: compiler/deleteExpressionMustBeOptional_exactOptionalPropertyTypes.ts
    // Arrange
    let project = TempProject::new(
        "exact-optional-property-types",
        "interface Options { name?: string; }\nconst options: Options = { name: undefined };",
        r#""strictNullChecks": true, "exactOptionalPropertyTypes": true"#,
    );

    // Act
    let output = project.compile();

    // Assert
    assert_diagnostic(&output, 2375);
}

#[test]
fn should_report_possible_undefined_given_array_index_when_compiling_project() {
    // Upstream: conformance/pedantic/noUncheckedIndexedAccess.ts
    // Arrange
    let project = TempProject::new(
        "no-unchecked-indexed-access",
        "const values: string[] = [];\nconst value: string = values[0];",
        r#""strictNullChecks": true, "noUncheckedIndexedAccess": true"#,
    );

    // Act
    let output = project.compile();

    // Assert
    assert_diagnostic(&output, 2322);
}

#[test]
fn should_report_unknown_catch_binding_given_unannotated_catch_when_compiling_project() {
    // Upstream: compiler/useUnknownInCatchVariables01.ts
    // Arrange
    let project = TempProject::new(
        "unknown-catch-variable",
        "function inspect(): void { try {} catch (error) { const text: string = error; } }",
        r#""useUnknownInCatchVariables": true"#,
    );

    // Act
    let output = project.compile();

    // Assert
    assert_diagnostic(&output, 2322);
}

#[test]
fn should_report_missing_return_given_partial_return_path_when_compiling_project() {
    // Upstream: compiler/noImplicitReturnsWithoutReturnExpression.ts
    // Arrange
    let project = TempProject::new(
        "no-implicit-returns",
        "function getCount(flag: boolean): number { if (flag) { return 1; } }",
        r#""strictNullChecks": false, "noImplicitReturns": true"#,
    );

    // Act
    let output = project.compile();

    // Assert
    assert_diagnostic(&output, 7030);
}
