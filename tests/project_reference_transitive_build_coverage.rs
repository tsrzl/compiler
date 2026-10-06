use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

struct TemporaryProject {
    path: PathBuf,
}

impl TemporaryProject {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "tsrzl-project-reference-transitive-build-{name}-{}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("the temporary project directory can be created");
        Self { path }
    }

    fn write(&self, relative_path: &str, contents: &str) {
        let path = self.path.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("the project directory can be created");
        }
        fs::write(path, contents).expect("the project configuration or source can be written");
    }

    fn mark_build_info_from_previous_compiler(&self, relative_path: &str) {
        let path = self.path.join(relative_path);
        let mut contents =
            fs::read_to_string(&path).expect("the generated build information can be read");
        let version_key = contents
            .find("\"version\"")
            .expect("the build information has a compiler version");
        let value_start = contents[version_key + "\"version\"".len()..]
            .find('"')
            .map(|offset| version_key + "\"version\"".len() + offset + 1)
            .expect("the build information compiler version is a string");
        let value_end = contents[value_start..]
            .find('"')
            .map(|offset| value_start + offset)
            .expect("the build information compiler version is terminated");
        contents.replace_range(value_start..value_end, "FakeTsPreviousVersion");
        fs::write(path, contents).expect("the stale build information can be written");
    }

    fn build_application(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tsrzl"))
            .current_dir(&self.path)
            .arg("--build")
            .arg(self.path.join("app/tsconfig.json"))
            .output()
            .expect("the compiler CLI can be started")
    }

    fn incremental_build_application(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tsrzl"))
            .current_dir(&self.path)
            .arg("--build")
            .arg("--incremental")
            .arg("--verbose")
            .arg(self.path.join("app/tsconfig.json"))
            .output()
            .expect("the compiler CLI can be started")
    }

    fn build_application_stopping_on_errors(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tsrzl"))
            .current_dir(&self.path)
            .arg("--build")
            .arg("--stopBuildOnErrors")
            .arg("--verbose")
            .arg(self.path.join("app/tsconfig.json"))
            .output()
            .expect("the compiler CLI can be started")
    }

    fn dry_build_application(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tsrzl"))
            .current_dir(&self.path)
            .arg("--build")
            .arg("--dry")
            .arg(self.path.join("app/tsconfig.json"))
            .output()
            .expect("the compiler CLI can be started")
    }

    fn verbose_dry_build_application(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tsrzl"))
            .current_dir(&self.path)
            .arg("--build")
            .arg("--dry")
            .arg("--verbose")
            .arg(self.path.join("app/tsconfig.json"))
            .output()
            .expect("the compiler CLI can be started")
    }

    fn clean_application(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tsrzl"))
            .current_dir(&self.path)
            .arg("--build")
            .arg("--clean")
            .arg(self.path.join("app/tsconfig.json"))
            .output()
            .expect("the compiler CLI can be started")
    }

    fn force_dry_build_application(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tsrzl"))
            .current_dir(&self.path)
            .arg("--build")
            .arg("--dry")
            .arg("--force")
            .arg(self.path.join("app/tsconfig.json"))
            .output()
            .expect("the compiler CLI can be started")
    }
}

impl Drop for TemporaryProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

// Pinned TypeScript-Go test: internal/project/projectreferencesprogram_test.go.
#[test]
fn should_build_transitive_project_references_given_three_level_graph_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new("transitive");
    project.write(
        "core/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","target":"es2015","outDir":"dist"},"include":["index.ts"]}"#,
    );
    project.write("core/index.ts", "export const coreValue: number = 42;\n");
    project.write(
        "middle/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","target":"es2015","outDir":"dist"},"references":[{"path":"../core"}],"include":["index.ts"]}"#,
    );
    project.write(
        "middle/index.ts",
        "import { coreValue } from \"../core\";\nexport const middleValue: number = coreValue;\n",
    );
    project.write(
        "app/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"module":"commonjs","target":"es2015","outDir":"dist"},"references":[{"path":"../middle"}],"include":["main.ts"]}"#,
    );
    project.write(
        "app/main.ts",
        "import { middleValue } from \"../middle\";\nexport const result: number = middleValue;\n",
    );

    // Act
    let output = project.build_application();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(output.status.success(), "{diagnostics}");
    for output in [
        "core/dist/index.js",
        "middle/dist/index.js",
        "app/dist/main.js",
    ] {
        assert!(
            project.path.join(output).is_file(),
            "expected transitive build output {output}"
        );
    }
}

// Pinned TypeScript-Go test: internal/execute/tsctests/tscbuild_test.go.
#[test]
fn should_build_non_composite_project_given_project_build_mode_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new("non-composite-build");
    project.write(
        "app/tsconfig.json",
        r#"{"compilerOptions":{"module":"commonjs","outDir":"dist"},"include":["index.ts"]}"#,
    );
    project.write("app/index.ts", "export const answer: number = 42;\n");

    // Act
    let output = project.build_application();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(output.status.success(), "{diagnostics}");
    assert!(
        project.path.join("app/dist/index.js").is_file(),
        "the non-composite project should emit JavaScript"
    );
}

// Pinned TypeScript-Go test: internal/project/projectreferencesprogram_test.go.
#[test]
fn should_rebuild_dependent_declarations_given_dependency_type_changes_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new("dependent-rebuild");
    project.write(
        "core/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","target":"es2015","outDir":"dist"},"include":["index.ts"]}"#,
    );
    project.write("core/index.ts", "export const coreValue: number = 42;\n");
    project.write(
        "middle/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","target":"es2015","outDir":"dist"},"references":[{"path":"../core"}],"include":["index.ts"]}"#,
    );
    project.write(
        "middle/index.ts",
        "import { coreValue } from \"../core\";\nexport const middleValue = coreValue;\n",
    );
    project.write(
        "app/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","target":"es2015","outDir":"dist"},"references":[{"path":"../middle"}],"include":["main.ts"]}"#,
    );
    project.write(
        "app/main.ts",
        "import { middleValue } from \"../middle\";\nexport const result = middleValue;\n",
    );
    let initial_build = project.build_application();
    assert!(
        initial_build.status.success(),
        "{}{}",
        String::from_utf8_lossy(&initial_build.stdout),
        String::from_utf8_lossy(&initial_build.stderr)
    );
    assert_eq!(
        fs::read_to_string(project.path.join("app/dist/main.d.ts"))
            .expect("the initial application declaration can be read"),
        "export declare const result: number;\n"
    );
    project.write(
        "core/index.ts",
        "export const coreValue: string = \"changed\";\n",
    );

    // Act
    let output = project.build_application();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(output.status.success(), "{diagnostics}");
    assert_eq!(
        fs::read_to_string(project.path.join("app/dist/main.d.ts"))
            .expect("the rebuilt application declaration can be read"),
        "export declare const result: string;\n"
    );
}

// Pinned TypeScript-Go test: internal/execute/tsctests/tscbuild_test.go.
#[test]
fn should_skip_project_outputs_given_dry_build_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new("dry-build");
    project.write(
        "core/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","outDir":"dist"},"include":["index.ts"]}"#,
    );
    project.write("core/index.ts", "export const value = 1;\n");
    project.write(
        "app/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","outDir":"dist"},"references":[{"path":"../core"}],"include":["index.ts"]}"#,
    );
    project.write(
        "app/index.ts",
        "import { value } from \"../core\";\nexport const result = value;\n",
    );

    // Act
    let output = project.dry_build_application();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(output.status.success(), "{diagnostics}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout)
            .matches("A non-dry build would build project")
            .count(),
        2,
        "{diagnostics}"
    );
    for output_directory in ["core/dist", "app/dist"] {
        assert!(
            !project.path.join(output_directory).exists(),
            "dry build unexpectedly wrote {output_directory}"
        );
    }
}

// Pinned TypeScript-Go test: internal/execute/tsctests/tscbuild_test.go.
#[test]
fn should_remove_outputs_given_clean_build_of_referenced_projects_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new("clean-build");
    project.write(
        "core/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","outDir":"dist"},"include":["index.ts"]}"#,
    );
    project.write("core/index.ts", "export const value = 1;\n");
    project.write(
        "app/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","outDir":"dist"},"references":[{"path":"../core"}],"include":["index.ts"]}"#,
    );
    project.write(
        "app/index.ts",
        "import { value } from \"../core\";\nexport const result = value;\n",
    );
    let initial_build = project.build_application();
    assert!(
        initial_build.status.success(),
        "{}{}",
        String::from_utf8_lossy(&initial_build.stdout),
        String::from_utf8_lossy(&initial_build.stderr)
    );
    let outputs = [
        "core/dist/index.js",
        "core/dist/index.d.ts",
        "core/dist/tsconfig.tsbuildinfo",
        "app/dist/index.js",
        "app/dist/index.d.ts",
        "app/dist/tsconfig.tsbuildinfo",
    ];
    for output in outputs {
        assert!(
            project.path.join(output).is_file(),
            "expected initial build output {output}"
        );
    }

    // Act
    let output = project.clean_application();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(output.status.success(), "{diagnostics}");
    for output in outputs {
        assert!(
            !project.path.join(output).exists(),
            "clean build left output {output}"
        );
    }
}

// Pinned TypeScript-Go test: internal/execute/tsctests/tscbuild_test.go.
#[test]
fn should_allow_repeated_project_clean_given_existing_clean_state_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new("repeated-clean-build");
    project.write(
        "core/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","outDir":"dist"},"include":["index.ts"]}"#,
    );
    project.write("core/index.ts", "export const value = 1;\n");
    project.write(
        "app/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","outDir":"dist"},"references":[{"path":"../core"}],"include":["index.ts"]}"#,
    );
    project.write(
        "app/index.ts",
        "import { value } from \"../core\";\nexport const result = value;\n",
    );
    let initial_build = project.build_application();
    assert!(
        initial_build.status.success(),
        "{}{}",
        String::from_utf8_lossy(&initial_build.stdout),
        String::from_utf8_lossy(&initial_build.stderr)
    );
    let first_clean = project.clean_application();
    assert!(
        first_clean.status.success(),
        "{}{}",
        String::from_utf8_lossy(&first_clean.stdout),
        String::from_utf8_lossy(&first_clean.stderr)
    );

    // Act
    let output = project.clean_application();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(output.status.success(), "{diagnostics}");
    for output_directory in ["core/dist", "app/dist"] {
        assert!(
            !project.path.join(output_directory).exists(),
            "repeated clean unexpectedly recreated {output_directory}"
        );
    }
}

// Pinned TypeScript-Go test: internal/execute/tsctests/tscbuild_test.go.
#[test]
fn should_rebuild_all_referenced_projects_given_force_option_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new("force-build");
    project.write(
        "core/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","outDir":"dist"},"include":["index.ts"]}"#,
    );
    project.write("core/index.ts", "export const coreValue = 1;\n");
    project.write(
        "middle/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","outDir":"dist"},"references":[{"path":"../core"}],"include":["index.ts"]}"#,
    );
    project.write(
        "middle/index.ts",
        "import { coreValue } from \"../core\";\nexport const middleValue = coreValue;\n",
    );
    project.write(
        "app/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","outDir":"dist"},"references":[{"path":"../middle"}],"include":["main.ts"]}"#,
    );
    project.write(
        "app/main.ts",
        "import { middleValue } from \"../middle\";\nexport const result = middleValue;\n",
    );
    let initial_build = project.build_application();
    assert!(
        initial_build.status.success(),
        "{}{}",
        String::from_utf8_lossy(&initial_build.stdout),
        String::from_utf8_lossy(&initial_build.stderr)
    );

    // Act
    let output = project.force_dry_build_application();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(output.status.success(), "{diagnostics}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout)
            .matches("A non-dry build would build project")
            .count(),
        3,
        "{diagnostics}"
    );
}

// Pinned TypeScript-Go test: internal/execute/tsctests/tscbuild_test.go.
#[test]
fn should_skip_up_to_date_projects_given_dry_build_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new("up-to-date-dry-build");
    project.write(
        "core/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","outDir":"dist"},"include":["index.ts"]}"#,
    );
    project.write("core/index.ts", "export const coreValue = 1;\n");
    project.write(
        "middle/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","outDir":"dist"},"references":[{"path":"../core"}],"include":["index.ts"]}"#,
    );
    project.write(
        "middle/index.ts",
        "import { coreValue } from \"../core\";\nexport const middleValue = coreValue;\n",
    );
    project.write(
        "app/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","outDir":"dist"},"references":[{"path":"../middle"}],"include":["main.ts"]}"#,
    );
    project.write(
        "app/main.ts",
        "import { middleValue } from \"../middle\";\nexport const result = middleValue;\n",
    );
    let initial_build = project.build_application();
    assert!(
        initial_build.status.success(),
        "{}{}",
        String::from_utf8_lossy(&initial_build.stdout),
        String::from_utf8_lossy(&initial_build.stderr)
    );

    // Act
    let output = project.dry_build_application();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(output.status.success(), "{diagnostics}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout)
            .matches(" is up to date")
            .count(),
        3,
        "{diagnostics}"
    );
}

// Pinned TypeScript-Go test: internal/execute/tsctests/tscbuild_test.go.
#[test]
fn should_schedule_only_changed_leaf_project_given_source_change_when_running_dry_build_cli() {
    // Arrange
    let project = TemporaryProject::new("changed-leaf-dry-build");
    project.write(
        "core/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","outDir":"dist"},"include":["index.ts"]}"#,
    );
    project.write("core/index.ts", "export const coreValue = 1;\n");
    project.write(
        "middle/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","outDir":"dist"},"references":[{"path":"../core"}],"include":["index.ts"]}"#,
    );
    project.write(
        "middle/index.ts",
        "import { coreValue } from \"../core\";\nexport const middleValue = coreValue;\n",
    );
    project.write(
        "app/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","outDir":"dist"},"references":[{"path":"../middle"}],"include":["main.ts"]}"#,
    );
    project.write(
        "app/main.ts",
        "import { middleValue } from \"../middle\";\nexport const result = middleValue;\n",
    );
    let initial_build = project.build_application();
    assert!(
        initial_build.status.success(),
        "{}{}",
        String::from_utf8_lossy(&initial_build.stdout),
        String::from_utf8_lossy(&initial_build.stderr)
    );
    project.write(
        "app/main.ts",
        "import { middleValue } from \"../middle\";\nexport const result = middleValue + \"!\";\n",
    );

    // Act
    let output = project.verbose_dry_build_application();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(output.status.success(), "{diagnostics}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout)
            .matches(" is up to date")
            .count(),
        2,
        "{diagnostics}"
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout)
            .matches("A non-dry build would build project")
            .count(),
        1,
        "{diagnostics}"
    );
}

// Pinned TypeScript-Go test: internal/execute/tsctests/tscbuild_test.go.
#[test]
fn should_schedule_changed_project_given_tsconfig_change_when_running_dry_build_cli() {
    // Arrange
    let project = TemporaryProject::new("changed-config-dry-build");
    project.write(
        "core/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","outDir":"dist"},"include":["index.ts"]}"#,
    );
    project.write("core/index.ts", "export const coreValue = 1;\n");
    project.write(
        "middle/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","outDir":"dist"},"references":[{"path":"../core"}],"include":["index.ts"]}"#,
    );
    project.write(
        "middle/index.ts",
        "import { coreValue } from \"../core\";\nexport const middleValue = coreValue;\n",
    );
    project.write(
        "app/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","target":"es2015","outDir":"dist"},"references":[{"path":"../middle"}],"include":["main.ts"]}"#,
    );
    project.write(
        "app/main.ts",
        "import { middleValue } from \"../middle\";\nexport const result = middleValue;\n",
    );
    let initial_build = project.build_application();
    assert!(
        initial_build.status.success(),
        "{}{}",
        String::from_utf8_lossy(&initial_build.stdout),
        String::from_utf8_lossy(&initial_build.stderr)
    );
    project.write(
        "app/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","target":"es2020","outDir":"dist"},"references":[{"path":"../middle"}],"include":["main.ts"]}"#,
    );

    // Act
    let output = project.verbose_dry_build_application();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(output.status.success(), "{diagnostics}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout)
            .matches(" is up to date")
            .count(),
        2,
        "{diagnostics}"
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout)
            .matches("A non-dry build would build project")
            .count(),
        1,
        "{diagnostics}"
    );
}

// Pinned TypeScript-Go test: internal/execute/tsctests/tscbuild_test.go.
#[test]
fn should_rebuild_projects_given_stale_build_info_version_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new("stale-build-info");
    project.write(
        "core/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","outDir":"dist"},"include":["index.ts"]}"#,
    );
    project.write("core/index.ts", "export const coreValue = 1;\n");
    project.write(
        "middle/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","outDir":"dist"},"references":[{"path":"../core"}],"include":["index.ts"]}"#,
    );
    project.write(
        "middle/index.ts",
        "import { coreValue } from \"../core\";\nexport const middleValue = coreValue;\n",
    );
    project.write(
        "app/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","outDir":"dist"},"references":[{"path":"../middle"}],"include":["main.ts"]}"#,
    );
    project.write(
        "app/main.ts",
        "import { middleValue } from \"../middle\";\nexport const result = middleValue;\n",
    );
    let initial_build = project.build_application();
    assert!(
        initial_build.status.success(),
        "{}{}",
        String::from_utf8_lossy(&initial_build.stdout),
        String::from_utf8_lossy(&initial_build.stderr)
    );
    for project_name in ["core", "middle", "app"] {
        project.mark_build_info_from_previous_compiler(&format!(
            "{project_name}/dist/tsconfig.tsbuildinfo"
        ));
    }

    // Act
    let output = project.verbose_dry_build_application();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(output.status.success(), "{diagnostics}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout)
            .matches("differs with current version")
            .count(),
        3,
        "{diagnostics}"
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout)
            .matches("A non-dry build would build project")
            .count(),
        3,
        "{diagnostics}"
    );
}

// Pinned TypeScript-Go test: internal/execute/tsctests/tscbuild_test.go.
#[test]
fn should_schedule_project_given_extended_tsconfig_change_when_running_dry_build_cli() {
    // Arrange
    let project = TemporaryProject::new("changed-extended-config-dry-build");
    project.write(
        "core/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","outDir":"dist"},"include":["index.ts"]}"#,
    );
    project.write("core/index.ts", "export const coreValue = 1;\n");
    project.write(
        "middle/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","outDir":"dist"},"references":[{"path":"../core"}],"include":["index.ts"]}"#,
    );
    project.write(
        "middle/index.ts",
        "import { coreValue } from \"../core\";\nexport const middleValue = coreValue;\n",
    );
    project.write(
        "app/tsconfig.base.json",
        r#"{"compilerOptions":{"target":"es2015"}}"#,
    );
    project.write(
        "app/tsconfig.json",
        r#"{"extends":"./tsconfig.base.json","compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","outDir":"dist"},"references":[{"path":"../middle"}],"include":["main.ts"]}"#,
    );
    project.write(
        "app/main.ts",
        "import { middleValue } from \"../middle\";\nexport const result = middleValue;\n",
    );
    let initial_build = project.build_application();
    assert!(
        initial_build.status.success(),
        "{}{}",
        String::from_utf8_lossy(&initial_build.stdout),
        String::from_utf8_lossy(&initial_build.stderr)
    );
    project.write(
        "app/tsconfig.base.json",
        r#"{"compilerOptions":{"target":"es2020"}}"#,
    );

    // Act
    let output = project.verbose_dry_build_application();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(output.status.success(), "{diagnostics}");
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("app/tsconfig.base.json"),
        "{diagnostics}"
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout)
            .matches(" is up to date")
            .count(),
        2,
        "{diagnostics}"
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout)
            .matches("A non-dry build would build project")
            .count(),
        1,
        "{diagnostics}"
    );
}

// Pinned TypeScript-Go test: internal/execute/tsctests/tscbuild_test.go.
#[test]
fn should_rebuild_incremental_project_given_corrupt_build_info_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new("corrupt-build-info");
    project.write("app/tsconfig.json", "{}\n");
    project.write("app/main.ts", "export const answer = 42;\n");
    project.write("app/tsconfig.tsbuildinfo", "Some random string");

    // Act
    let output = project.incremental_build_application();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(output.status.success(), "{diagnostics}");
    assert!(
        project.path.join("app/main.js").is_file(),
        "the source file should be emitted after corrupt build-info recovery"
    );
    let build_info = fs::read_to_string(project.path.join("app/tsconfig.tsbuildinfo"))
        .expect("the build information should be rewritten");
    assert_ne!(build_info, "Some random string");
    assert!(build_info.contains("\"version\":\"7.0.2\""));
}

// Pinned TypeScript-Go test: internal/execute/tsctests/tscbuild_test.go.
#[test]
fn should_skip_dependents_given_upstream_error_when_stop_build_on_errors_is_enabled() {
    // Arrange
    let project = TemporaryProject::new("stop-downstream-builds-on-error");
    project.write(
        "core/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","outDir":"dist"},"include":["index.ts"]}"#,
    );
    project.write(
        "core/index.ts",
        "export function multiply(a: number, b: number) { return a * b; }\nmultiply();\n",
    );
    project.write(
        "middle/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","outDir":"dist"},"references":[{"path":"../core"}],"include":["index.ts"]}"#,
    );
    project.write(
        "middle/index.ts",
        "import { multiply } from \"../core\";\nexport const value = multiply(2, 3);\n",
    );
    project.write(
        "app/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"module":"commonjs","outDir":"dist"},"references":[{"path":"../middle"}],"include":["main.ts"]}"#,
    );
    project.write(
        "app/main.ts",
        "import { value } from \"../middle\";\nexport const result = value;\n",
    );

    // Act
    let output = project.build_application_stopping_on_errors();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(!output.status.success(), "{diagnostics}");
    assert!(diagnostics.contains("TS2554:"), "{diagnostics}");
    assert_eq!(diagnostics.matches("Skipping build of project").count(), 2);
    assert!(project.path.join("core/dist/index.js").is_file());
    assert!(!project.path.join("middle/dist/index.js").exists());
    assert!(!project.path.join("app/dist/main.js").exists());
}
