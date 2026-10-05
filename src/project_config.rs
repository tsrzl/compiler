//! JSONC project-file loading for the command-line adapter.

use std::collections::{BTreeMap, HashSet};

use std::fs;
use std::path::{Path, PathBuf};

use crate::jsonc::{JsonParser, JsonValue};

#[derive(Debug, Default)]
pub(crate) struct ProjectConfig {
    pub(crate) root_files: Vec<PathBuf>,
    pub(crate) target: Option<String>,
    pub(crate) module: Option<String>,
    pub(crate) out_dir: Option<PathBuf>,
    pub(crate) declaration: Option<bool>,
    pub(crate) emit_declaration_only: Option<bool>,
    pub(crate) strict_null_checks: Option<bool>,
    root_selection_overridden: bool,
}

pub(crate) fn load(project_path: &Path) -> Result<ProjectConfig, String> {
    load_config(project_path, &mut HashSet::new())
}

fn load_config(
    project_path: &Path,
    visited: &mut HashSet<PathBuf>,
) -> Result<ProjectConfig, String> {
    let config_path = if project_path.is_dir() {
        project_path.join("tsconfig.json")
    } else {
        project_path.to_path_buf()
    };
    let identity = fs::canonicalize(&config_path).unwrap_or_else(|_| config_path.clone());
    if !visited.insert(identity.clone()) {
        return Err(format!(
            "{}: circular project configuration extension",
            config_path.display()
        ));
    }
    let result = load_config_file(&config_path, visited);
    visited.remove(&identity);
    result
}

fn load_config_file(
    config_path: &Path,
    visited: &mut HashSet<PathBuf>,
) -> Result<ProjectConfig, String> {
    let source = fs::read_to_string(config_path)
        .map_err(|error| format!("could not read {}: {error}", config_path.display()))?;
    let value = JsonParser::parse(&source)
        .map_err(|error| format!("{}: {error}", config_path.display()))?;
    let JsonValue::Object(configuration) = value else {
        return Err(format!(
            "{}: project configuration must be a JSON object",
            config_path.display()
        ));
    };
    let project_directory = config_path
        .parent()
        .filter(|directory| !directory.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let inherited = load_extended_configs(&configuration, project_directory, config_path, visited)?;
    let options = parse_compiler_options(&configuration, config_path, project_directory)?;
    let root_files = select_root_files(
        &configuration,
        project_directory,
        config_path,
        inherited.as_ref(),
        options.out_dir.as_deref(),
    )?;
    let own = ProjectConfig {
        root_files,
        target: options.target,
        module: options.module,
        out_dir: options.out_dir,
        declaration: options.declaration,
        emit_declaration_only: options.emit_declaration_only,
        strict_null_checks: options.strict_null_checks,
        root_selection_overridden: configuration.contains_key("files")
            || configuration.contains_key("include")
            || configuration.contains_key("exclude"),
    };
    Ok(match inherited {
        Some(base) => merge_project_configs(base, own),
        None => own,
    })
}

struct CompilerOptionOverrides {
    target: Option<String>,
    module: Option<String>,
    out_dir: Option<PathBuf>,
    declaration: Option<bool>,
    emit_declaration_only: Option<bool>,
    strict_null_checks: Option<bool>,
}

fn parse_compiler_options(
    configuration: &BTreeMap<String, JsonValue>,
    config_path: &Path,
    project_directory: &Path,
) -> Result<CompilerOptionOverrides, String> {
    let compiler_options = match configuration.get("compilerOptions") {
        Some(JsonValue::Object(options)) => Some(options),
        Some(_) => {
            return Err(format!(
                "{}: 'compilerOptions' must be an object",
                config_path.display()
            ));
        }
        None => None,
    };
    let string_option = |name| {
        compiler_options
            .map(|options| string_option(options, name, config_path))
            .transpose()
            .map(Option::flatten)
    };
    let boolean_option = |name| {
        compiler_options
            .map(|options| boolean_option(options, name, config_path))
            .transpose()
            .map(Option::flatten)
    };
    let out_dir = string_option("outDir")?.map(|directory| project_directory.join(directory));
    Ok(CompilerOptionOverrides {
        target: string_option("target")?,
        module: string_option("module")?,
        out_dir,
        declaration: boolean_option("declaration")?,
        emit_declaration_only: boolean_option("emitDeclarationOnly")?,
        strict_null_checks: boolean_option("strictNullChecks")?,
    })
}

fn select_root_files(
    configuration: &BTreeMap<String, JsonValue>,
    project_directory: &Path,
    config_path: &Path,
    inherited: Option<&ProjectConfig>,
    out_dir: Option<&Path>,
) -> Result<Vec<PathBuf>, String> {
    match configuration.get("files") {
        Some(JsonValue::Array(files)) => files
            .iter()
            .enumerate()
            .map(|(index, file)| {
                let JsonValue::String(file) = file else {
                    return Err(format!(
                        "{}: 'files' entry {} must be a string",
                        config_path.display(),
                        index
                    ));
                };
                Ok(project_directory.join(file))
            })
            .collect(),
        Some(_) => Err(format!(
            "{}: 'files' must be an array of strings",
            config_path.display()
        )),
        None if configuration.contains_key("include") || configuration.contains_key("exclude") => {
            let include = string_array_option(configuration, "include", config_path)?;
            let exclude = string_array_option(configuration, "exclude", config_path)?;
            discover_root_files(
                project_directory,
                include.as_deref(),
                exclude.as_deref(),
                out_dir,
            )
        }
        None => match inherited {
            Some(configuration) if configuration.root_selection_overridden => {
                Ok(configuration.root_files.clone())
            }
            _ => discover_root_files(project_directory, None, None, out_dir),
        },
    }
}

fn load_extended_configs(
    configuration: &BTreeMap<String, JsonValue>,
    project_directory: &Path,
    config_path: &Path,
    visited: &mut HashSet<PathBuf>,
) -> Result<Option<ProjectConfig>, String> {
    let mut inherited = None;
    for base_path in extended_config_paths(configuration, project_directory, config_path)? {
        let base = load_config(&base_path, visited)?;
        inherited = Some(match inherited {
            Some(current) => merge_project_configs(current, base),
            None => base,
        });
    }
    Ok(inherited)
}

fn extended_config_paths(
    configuration: &BTreeMap<String, JsonValue>,
    project_directory: &Path,
    config_path: &Path,
) -> Result<Vec<PathBuf>, String> {
    let Some(value) = configuration.get("extends") else {
        return Ok(Vec::new());
    };
    let specifiers = match value {
        JsonValue::String(specifier) => vec![specifier.as_str()],
        JsonValue::Array(specifiers) => specifiers
            .iter()
            .enumerate()
            .map(|(index, value)| match value {
                JsonValue::String(specifier) => Ok(specifier.as_str()),
                _ => Err(format!(
                    "{}: 'extends' entry {index} must be a string",
                    config_path.display()
                )),
            })
            .collect::<Result<Vec<_>, _>>()?,
        _ => {
            return Err(format!(
                "{}: 'extends' must be a string or an array of strings",
                config_path.display()
            ));
        }
    };
    Ok(specifiers
        .into_iter()
        .map(|specifier| resolve_extended_config(project_directory, specifier))
        .collect())
}

fn resolve_extended_config(project_directory: &Path, specifier: &str) -> PathBuf {
    let specifier_path = Path::new(specifier);
    let candidate = if specifier_path.is_absolute() {
        specifier_path.to_path_buf()
    } else {
        project_directory.join(specifier_path)
    };
    if candidate.is_dir() {
        return candidate.join("tsconfig.json");
    }
    if candidate.is_file() {
        return candidate;
    }
    if let Some(package_config) = resolve_package_extended_config(project_directory, specifier) {
        return package_config;
    }
    if candidate.extension().is_none() {
        let json_candidate = candidate.with_extension("json");
        if json_candidate.is_file() {
            return json_candidate;
        }
    }
    candidate
}

fn resolve_package_extended_config(project_directory: &Path, specifier: &str) -> Option<PathBuf> {
    let (package_name, subpath) = package_specifier_parts(specifier)?;
    for directory in project_directory.ancestors() {
        let package_root = directory.join("node_modules").join(package_name);
        if !package_root.is_dir() {
            continue;
        }
        let manifest = fs::read_to_string(package_root.join("package.json")).ok()?;
        let JsonValue::Object(manifest) = JsonParser::parse(&manifest).ok()? else {
            return None;
        };
        let JsonValue::Object(exports) = manifest.get("exports")? else {
            return None;
        };
        let target = resolve_export_subpath(exports, &subpath)?;
        let relative_target = target.strip_prefix("./")?;
        let relative_path = Path::new(relative_target);
        if relative_path.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        }) {
            return None;
        }
        let target_path = package_root.join(relative_path);
        return target_path.is_file().then_some(target_path);
    }
    None
}

fn package_specifier_parts(specifier: &str) -> Option<(&str, String)> {
    let parts = specifier.split('/').collect::<Vec<_>>();
    let package_parts = if parts.first()?.starts_with('@') {
        2
    } else {
        1
    };
    if parts.len() < package_parts || parts[..package_parts].iter().any(|part| part.is_empty()) {
        return None;
    }
    let package_name = &specifier[..parts[..package_parts].join("/").len()];
    let subpath = if parts.len() == package_parts {
        ".".to_owned()
    } else {
        format!("./{}", parts[package_parts..].join("/"))
    };
    Some((package_name, subpath))
}

fn resolve_export_subpath(exports: &BTreeMap<String, JsonValue>, subpath: &str) -> Option<String> {
    if let Some(JsonValue::String(target)) = exports.get(subpath) {
        return Some(target.clone());
    }
    let mut patterns = exports
        .iter()
        .filter_map(|(pattern, target)| {
            let (prefix, suffix) = pattern.split_once('*')?;
            let captured = subpath.strip_prefix(prefix)?.strip_suffix(suffix)?;
            if captured.is_empty() {
                return None;
            }
            let JsonValue::String(target) = target else {
                return None;
            };
            Some((prefix.len(), suffix.len(), captured, target))
        })
        .collect::<Vec<_>>();
    patterns.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| right.1.cmp(&left.1)));
    let (_, _, captured, target) = patterns.into_iter().next()?;
    Some(target.replace('*', captured))
}

fn merge_project_configs(base: ProjectConfig, derived: ProjectConfig) -> ProjectConfig {
    let root_files = if derived.root_selection_overridden {
        derived.root_files
    } else if base.root_selection_overridden {
        base.root_files
    } else {
        derived.root_files
    };
    ProjectConfig {
        root_files,
        target: derived.target.or(base.target),
        module: derived.module.or(base.module),
        out_dir: derived.out_dir.or(base.out_dir),
        declaration: derived.declaration.or(base.declaration),
        emit_declaration_only: derived.emit_declaration_only.or(base.emit_declaration_only),
        strict_null_checks: derived.strict_null_checks.or(base.strict_null_checks),
        root_selection_overridden: derived.root_selection_overridden
            || base.root_selection_overridden,
    }
}

fn string_array_option(
    options: &BTreeMap<String, JsonValue>,
    name: &str,
    config_path: &Path,
) -> Result<Option<Vec<String>>, String> {
    match options.get(name) {
        Some(JsonValue::Array(values)) => values
            .iter()
            .enumerate()
            .map(|(index, value)| {
                let JsonValue::String(value) = value else {
                    return Err(format!(
                        "{}: '{name}' entry {index} must be a string",
                        config_path.display()
                    ));
                };
                Ok(value.clone())
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Some),
        Some(_) => Err(format!(
            "{}: '{name}' must be an array of strings",
            config_path.display()
        )),
        None => Ok(None),
    }
}

fn string_option(
    options: &BTreeMap<String, JsonValue>,
    name: &str,
    config_path: &Path,
) -> Result<Option<String>, String> {
    match options.get(name) {
        Some(JsonValue::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(format!(
            "{}: compiler option '{name}' must be a string",
            config_path.display()
        )),
        None => Ok(None),
    }
}

fn boolean_option(
    options: &BTreeMap<String, JsonValue>,
    name: &str,
    config_path: &Path,
) -> Result<Option<bool>, String> {
    match options.get(name) {
        Some(JsonValue::Boolean(value)) => Ok(Some(*value)),
        Some(_) => Err(format!(
            "{}: compiler option '{name}' must be a boolean",
            config_path.display()
        )),
        None => Ok(None),
    }
}

fn discover_root_files(
    directory: &Path,
    include_patterns: Option<&[String]>,
    exclude_patterns: Option<&[String]>,
    output_directory: Option<&Path>,
) -> Result<Vec<PathBuf>, String> {
    let mut pending = vec![directory.to_path_buf()];
    let mut files = Vec::new();
    while let Some(current) = pending.pop() {
        let entries = fs::read_dir(&current)
            .map_err(|error| format!("could not read {}: {error}", current.display()))?;
        for entry in entries {
            let entry = entry.map_err(|error| {
                format!("could not read an entry in {}: {error}", current.display())
            })?;
            let file_type = entry.file_type().map_err(|error| {
                format!("could not inspect {}: {error}", entry.path().display())
            })?;
            if file_type.is_dir() {
                if !is_default_excluded_directory(&entry.file_name()) {
                    pending.push(entry.path());
                }
            } else if file_type.is_file() && is_default_project_source(&entry.path()) {
                let path = entry.path();
                if output_directory.is_some_and(|output| path.starts_with(output)) {
                    continue;
                }
                let relative_path = path.strip_prefix(directory).map_err(|error| {
                    format!(
                        "could not make {} relative to the project: {error}",
                        path.display()
                    )
                })?;
                let relative_path = path_as_pattern(relative_path);
                let included = include_patterns.is_none_or(|patterns| {
                    patterns
                        .iter()
                        .any(|pattern| matches_pattern_or_directory(pattern, &relative_path))
                });
                let excluded = exclude_patterns.is_some_and(|patterns| {
                    patterns
                        .iter()
                        .any(|pattern| matches_pattern_or_directory(pattern, &relative_path))
                });
                if included && !excluded {
                    files.push(path);
                }
            }
        }
    }
    files.sort();
    Ok(files)
}

fn path_as_pattern(path: &Path) -> String {
    path.components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect::<Vec<_>>()
        .join("/")
}

fn matches_pattern_or_directory(pattern: &str, path: &str) -> bool {
    let pattern = normalize_pattern(pattern);
    if pattern.contains(['*', '?']) {
        return glob_matches(&pattern, path);
    }
    path == pattern
        || path
            .strip_prefix(&pattern)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn normalize_pattern(pattern: &str) -> String {
    let mut pattern = pattern.replace('\\', "/");
    while let Some(stripped) = pattern.strip_prefix("./") {
        pattern = stripped.to_owned();
    }
    pattern.trim_end_matches('/').to_owned()
}

fn glob_matches(pattern: &str, path: &str) -> bool {
    let pattern = pattern.split('/').collect::<Vec<_>>();
    let path = path.split('/').collect::<Vec<_>>();
    let mut memo = vec![None; (pattern.len() + 1) * (path.len() + 1)];
    glob_segments_match(&pattern, &path, 0, 0, &mut memo)
}

fn glob_segments_match(
    pattern: &[&str],
    path: &[&str],
    pattern_index: usize,
    path_index: usize,
    memo: &mut [Option<bool>],
) -> bool {
    let path_width = path.len() + 1;
    let memo_index = pattern_index * path_width + path_index;
    if let Some(result) = memo[memo_index] {
        return result;
    }
    let result = if pattern_index == pattern.len() {
        path_index == path.len()
    } else if pattern[pattern_index] == "**" {
        glob_segments_match(pattern, path, pattern_index + 1, path_index, memo)
            || (path_index < path.len()
                && glob_segments_match(pattern, path, pattern_index, path_index + 1, memo))
    } else {
        path_index < path.len()
            && glob_segment_matches(pattern[pattern_index], path[path_index])
            && glob_segments_match(pattern, path, pattern_index + 1, path_index + 1, memo)
    };
    memo[memo_index] = Some(result);
    result
}

fn glob_segment_matches(pattern: &str, value: &str) -> bool {
    let pattern = pattern.chars().collect::<Vec<_>>();
    let value = value.chars().collect::<Vec<_>>();
    let mut previous = vec![false; value.len() + 1];
    previous[0] = true;
    for character in pattern {
        let mut current = vec![false; value.len() + 1];
        if character == '*' {
            current[0] = previous[0];
            for index in 1..=value.len() {
                current[index] = previous[index] || current[index - 1];
            }
        } else {
            for index in 1..=value.len() {
                current[index] =
                    previous[index - 1] && (character == '?' || character == value[index - 1]);
            }
        }
        previous = current;
    }
    previous[value.len()]
}

fn is_default_excluded_directory(name: &std::ffi::OsStr) -> bool {
    matches!(
        name.to_str(),
        Some("node_modules" | "bower_components" | "jspm_packages")
    )
}

fn is_default_project_source(path: &Path) -> bool {
    matches!(
        path.extension().and_then(std::ffi::OsStr::to_str),
        Some("ts" | "tsx" | "mts" | "cts")
    )
}
