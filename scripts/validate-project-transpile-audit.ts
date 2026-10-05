#!/usr/bin/env node
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { extname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { parseArgs } from "node:util";
import {
  assertPinnedRepository,
  readTsv,
  TYPESCRIPT_REVISION,
} from "./typescript-corpus.ts";

type RunnerCase = {
  projectRoot: string;
  project?: string;
  inputFiles?: string[];
  scenario?: string;
};

type AuditCounts = {
  projectFixtures: number;
  transpileFixtures: number;
  projectRunnerJson: number;
  mappedTests: number;
  mappedFixtureRows: number;
  missingRootGapRows: number;
};

type Options = {
  typescriptRoot: string;
  inventoryPath: string;
  auditPath: string;
  testRoot: string;
  help: boolean;
};

const SOURCE_EXTENSIONS = new Set([".ts", ".tsx"]);
const PROJECT_FIXTURE_SUITES = new Set(["projects", "transpile"]);
const AUDIT_SUITES = new Set([...PROJECT_FIXTURE_SUITES, "project-runner-gap"]);

function parseOptions(): Options {
  const { values } = parseArgs({
    options: {
      "typescript-root": { type: "string" },
      inventory: { type: "string", default: "docs/typescript-7-case-inventory.tsv" },
      audit: { type: "string", default: "docs/typescript-7-project-transpile-audit.tsv" },
      "test-root": { type: "string", default: "tests" },
      help: { type: "boolean", short: "h" },
    },
  });
  if (!values.help && !values["typescript-root"]) {
    throw new Error("--typescript-root is required");
  }
  return {
    typescriptRoot: values["typescript-root"] ?? "",
    inventoryPath: values.inventory,
    auditPath: values.audit,
    testRoot: values["test-root"],
    help: values.help ?? false,
  };
}

function printUsage(): void {
  console.log(
    "Usage: node scripts/validate-project-transpile-audit.ts --typescript-root PATH [--inventory PATH] [--audit PATH] [--test-root PATH] [--help]",
  );
  console.log("  --typescript-root PATH  Pinned TypeScript checkout (required)");
  console.log("  --inventory PATH        Case inventory TSV (default: docs/typescript-7-case-inventory.tsv)");
  console.log("  --audit PATH            Project/transpile audit TSV (default: docs/typescript-7-project-transpile-audit.tsv)");
  console.log("  --test-root PATH        Rust tests directory (default: tests)");
}

function walkFiles(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    return entry.isDirectory() ? walkFiles(path) : [path];
  }).sort();
}

function relativePosix(root: string, path: string): string {
  return relative(root, path).split(sep).join("/");
}

function isWithin(root: string, path: string): boolean {
  const fromRoot = relative(root, path);
  return fromRoot === "" || (!fromRoot.startsWith(`..${sep}`) && fromRoot !== ".." && !isAbsolute(fromRoot));
}

function addError(errors: string[], message: string): void {
  errors.push(message);
}

function compareSets(
  label: string,
  expected: ReadonlySet<string>,
  actual: ReadonlySet<string>,
  errors: string[],
): void {
  const missing = [...expected].filter((entry) => !actual.has(entry)).sort();
  const extra = [...actual].filter((entry) => !expected.has(entry)).sort();
  if (missing.length || extra.length) {
    addError(errors, `${label} mismatch: missing=[${missing.join(", ")}], extra=[${extra.join(", ")}]`);
  }
}

function duplicateValues(values: readonly string[]): string[] {
  const seen = new Set<string>();
  const duplicates = new Set<string>();
  for (const value of values) {
    if (seen.has(value)) duplicates.add(value);
    seen.add(value);
  }
  return [...duplicates].sort();
}

function isRunnerCase(value: unknown): value is RunnerCase {
  if (typeof value !== "object" || value === null || Array.isArray(value)) return false;
  const candidate = value as Record<string, unknown>;
  return typeof candidate.projectRoot === "string"
    && (candidate.project === undefined || typeof candidate.project === "string")
    && (candidate.scenario === undefined || typeof candidate.scenario === "string")
    && (candidate.inputFiles === undefined
      || (Array.isArray(candidate.inputFiles) && candidate.inputFiles.every((file) => typeof file === "string")));
}

function readRunnerJson(path: string, caseRoot: string, errors: string[]): RunnerCase | undefined {
  const absolutePath = resolve(caseRoot, path);
  if (!isWithin(caseRoot, absolutePath)) {
    addError(errors, `project runner JSON path escapes tests/cases: ${path}`);
    return undefined;
  }
  if (!existsSync(absolutePath)) {
    addError(errors, `project runner JSON does not exist: ${path}`);
    return undefined;
  }
  try {
    const parsed: unknown = JSON.parse(readFileSync(absolutePath, "utf8"));
    if (!isRunnerCase(parsed)) {
      addError(errors, `project runner JSON has an invalid project-root/input shape: ${path}`);
      return undefined;
    }
    return parsed;
  } catch (error) {
    addError(errors, `invalid project runner JSON ${path}: ${String(error)}`);
    return undefined;
  }
}

function findRustTestBody(source: string, name: string): {
  body: string;
  pinnedComment: string;
} | undefined {
  const declaration = new RegExp(`\\bfn\\s+${name}\\s*\\(`).exec(source);
  if (!declaration || declaration.index === undefined) return undefined;
  const functionIndex = declaration.index;
  const attributeIndex = source.lastIndexOf("#[test]", functionIndex);
  if (attributeIndex < 0) return undefined;
  const attributeToFunction = source.slice(attributeIndex + "#[test]".length, functionIndex);
  if (!/^\s*$/.test(attributeToFunction)) return undefined;
  const commentIndex = source.lastIndexOf("// Pinned", attributeIndex);
  if (commentIndex < 0) return undefined;
  const commentLineStart = source.lastIndexOf("\n", commentIndex) + 1;
  const commentLineEnd = source.indexOf("\n", commentIndex);
  const pinnedLine = source.slice(commentLineStart, commentLineEnd < 0 ? undefined : commentLineEnd).trimStart();
  if (!pinnedLine.startsWith("// Pinned")) return undefined;
  const pinnedComment = source.slice(commentIndex, attributeIndex);
  const commentLines = pinnedComment.split(/\r?\n/).map((line) => line.trim()).filter(Boolean);
  if (commentLines.some((line) => !line.startsWith("//"))) return undefined;
  const nextPinnedComment = source.indexOf("\n// Pinned", functionIndex);
  const bodyEnd = nextPinnedComment < 0 ? source.length : nextPinnedComment;
  return { body: source.slice(functionIndex, bodyEnd), pinnedComment };
}

function fixtureIsCoveredByMappedTest(
  fixturePath: string,
  testBody: string,
  pinnedComment: string,
): boolean {
  if (pinnedComment.includes(fixturePath)) return true;
  if (!fixturePath.startsWith("projects/")) return false;
  const fixtureParts = fixturePath.split("/");
  const projectAreaPrefix = `projects/${fixtureParts[1]}/`;
  if (!pinnedComment.includes(projectAreaPrefix)) return false;
  const relativeToProjectArea = fixtureParts.slice(2).join("/");
  return relativeToProjectArea.length > 0 && testBody.includes(relativeToProjectArea);
}

function validate(options: Options): { counts: AuditCounts; errors: string[] } {
  const typescriptRoot = resolve(options.typescriptRoot);
  assertPinnedRepository(typescriptRoot, TYPESCRIPT_REVISION, "TypeScript corpus");
  const caseRoot = join(typescriptRoot, "tests", "cases");
  const inventory = readTsv(resolve(options.inventoryPath));
  const audit = readTsv(resolve(options.auditPath));
  const rustTestSources = walkFiles(resolve(options.testRoot))
    .filter((path) => extname(path) === ".rs")
    .map((path) => ({ path, source: readFileSync(path, "utf8") }));
  if (rustTestSources.length === 0) throw new Error(`No Rust integration test files exist under: ${options.testRoot}`);
  const errors: string[] = [];

  for (const row of audit) {
    if (!AUDIT_SUITES.has(row.suite)) addError(errors, `unexpected audit suite: ${row.suite}`);
  }

  const inventorySources = new Map<string, Set<string>>();
  for (const suite of PROJECT_FIXTURE_SUITES) {
    inventorySources.set(
      suite,
      new Set(inventory.filter((row) => row.suite === suite).map((row) => row.source_case)),
    );
  }

  for (const suite of PROJECT_FIXTURE_SUITES) {
    const rows = audit.filter((row) => row.suite === suite);
    const paths = rows.map((row) => row.fixture_path);
    const duplicates = duplicateValues(paths);
    if (duplicates.length) addError(errors, `${suite} has duplicate source rows: ${duplicates.join(", ")}`);
    compareSets(`${suite} inventory/audit fixture set`, inventorySources.get(suite) ?? new Set(), new Set(paths), errors);

    const fixtureRoot = join(caseRoot, suite);
    if (!existsSync(fixtureRoot)) {
      addError(errors, `pinned fixture suite directory does not exist: ${fixtureRoot}`);
      continue;
    }
    const pinnedFixtures = new Set(
      walkFiles(fixtureRoot)
        .filter((path) => SOURCE_EXTENSIONS.has(extname(path)))
        .map((path) => relativePosix(caseRoot, path)),
    );
    compareSets(`${suite} pinned/inventory fixture set`, pinnedFixtures, inventorySources.get(suite) ?? new Set(), errors);
    for (const path of paths) {
      const absolutePath = resolve(caseRoot, path);
      if (!isWithin(caseRoot, absolutePath) || !existsSync(absolutePath)) {
        addError(errors, `${suite} audit fixture path is absent or outside tests/cases: ${path}`);
      }
    }
  }

  const projectJsonRoot = join(caseRoot, "project");
  const projectJsonPaths = walkFiles(projectJsonRoot)
    .filter((path) => extname(path) === ".json")
    .map((path) => relativePosix(caseRoot, path));
  const projectJsonSet = new Set(projectJsonPaths);
  const linkedProjectJsons = new Set<string>();
  const projectRows = audit.filter((row) => row.suite === "projects");
  for (const row of projectRows) {
    if (!row.upstream_case_json) {
      addError(errors, `project fixture has no runner JSON mapping: ${row.fixture_path}`);
      continue;
    }
    const rowPaths = row.upstream_case_json.split(";").filter(Boolean);
    const duplicates = duplicateValues(rowPaths);
    if (duplicates.length) addError(errors, `project fixture repeats runner JSON paths: ${row.fixture_path}: ${duplicates.join(", ")}`);
    for (const path of rowPaths) {
      linkedProjectJsons.add(path);
      readRunnerJson(path, caseRoot, errors);
    }
  }
  const projectGapRows = audit.filter((entry) => entry.suite === "project-runner-gap");
  if (projectGapRows.length === 0) {
    addError(errors, "audit must include an explicit project runner row for a missing source root");
  }
  const gapJsonPaths = projectGapRows.flatMap((row) => row.upstream_case_json.split(";").filter(Boolean));
  const duplicateGapJsonPaths = duplicateValues(gapJsonPaths);
  if (duplicateGapJsonPaths.length) {
    addError(errors, `project runner gap JSON paths must be unique: ${duplicateGapJsonPaths.join(", ")}`);
  }
  for (const row of projectGapRows) {
    if (!row.upstream_case_json) {
      addError(errors, `project runner gap has no source JSON: ${row.fixture_path}`);
      continue;
    }
    const paths = row.upstream_case_json.split(";").filter(Boolean);
    if (paths.length !== 1) {
      addError(errors, `project runner gap must identify one runner JSON: ${row.fixture_path}`);
      continue;
    }
    const [path] = paths;
    linkedProjectJsons.add(path);
    const json = readRunnerJson(path, caseRoot, errors);
    if (!json) continue;
    if (typeof json.projectRoot !== "string" || !Array.isArray(json.inputFiles)) {
      addError(errors, `project runner gap JSON lacks projectRoot/inputFiles: ${path}`);
      continue;
    }
    const projectRootPath = resolve(typescriptRoot, json.projectRoot);
    if (existsSync(projectRootPath)) addError(errors, `project runner gap root unexpectedly exists: ${json.projectRoot}`);
    for (const input of json.inputFiles) {
      const inputPath = resolve(projectRootPath, input);
      if (existsSync(inputPath)) addError(errors, `project runner gap input unexpectedly exists: ${json.projectRoot}/${input}`);
      if (!row.fixture_path.includes(input)) addError(errors, `project runner gap row omits declared missing input ${input}: ${row.fixture_path}`);
    }
    if (!row.fixture_path.startsWith("missing:") || !/missing|absent/i.test(row.oracle_expectation)) {
      addError(errors, `project runner gap does not document a missing fixture/oracle gap: ${row.fixture_path}`);
    }
  }

  const transpileRows = audit.filter((row) => row.suite === "transpile");
  for (const row of transpileRows) {
    if (row.upstream_case_json !== "") {
      addError(errors, `transpile fixture must leave upstream_case_json empty: ${row.fixture_path}`);
    }
  }
  compareSets("project runner JSON link union", projectJsonSet, linkedProjectJsons, errors);

  const mappedRows = audit.filter((row) => row.rust_test_mapping !== "");
  const mappedNames = new Set(mappedRows.flatMap((row) => row.rust_test_mapping.split(";").filter(Boolean)));
  const rustTestLocations = new Map<string, string>();
  for (const { path, source } of rustTestSources) {
    for (const match of source.matchAll(/^\s*fn\s+(should_[A-Za-z0-9_]+)\s*\(/gm)) {
      if (rustTestLocations.has(match[1])) addError(errors, `duplicate Rust behavior test name: ${match[1]}`);
      else rustTestLocations.set(match[1], path);
    }
  }
  const fixtureRowsByTest = new Map<string, string[]>();
  for (const row of mappedRows) {
    if (!/oracle-verified/i.test(row.coverage_status)) {
      addError(errors, `mapped fixture lacks oracle-verified status: ${row.fixture_path}`);
    }
    const names = row.rust_test_mapping.split(";").filter(Boolean);
    if (duplicateValues(names).length > 0) {
      addError(errors, `project fixture repeats a Rust test mapping: ${row.fixture_path}`);
    }
    for (const testName of names) {
      if (!/^should_[A-Za-z0-9_]+$/.test(testName)) {
        addError(errors, `invalid Rust behavior test mapping name: ${testName}`);
        continue;
      }
      if (!rustTestLocations.has(testName)) {
        addError(errors, `mapped Rust test function does not exist: ${testName}`);
        continue;
      }
      const mappedFixtures = fixtureRowsByTest.get(testName) ?? [];
      mappedFixtures.push(row.fixture_path);
      fixtureRowsByTest.set(testName, mappedFixtures);
    }
  }
  for (const [testName, fixtures] of fixtureRowsByTest) {
    const source = readFileSync(rustTestLocations.get(testName)!, "utf8");
    const test = findRustTestBody(source, testName);
    if (!test) {
      addError(errors, `mapped Rust test is missing its #[test] item or pinned-case comment: ${testName}`);
      continue;
    }
    if (!test.pinnedComment.includes("// Pinned")) {
      addError(errors, `mapped Rust test needs a pinned-case comment before #[test]: ${testName}`);
    }
    for (const fixture of fixtures) {
      if (!fixtureIsCoveredByMappedTest(fixture, test.body, test.pinnedComment)) {
        addError(errors, `mapping includes a fixture not named by test comment/setup: ${testName} -> ${fixture}`);
      }
    }
  }
  for (const row of audit.filter((entry) => PROJECT_FIXTURE_SUITES.has(entry.suite) && !entry.rust_test_mapping)) {
    if (/oracle-verified/i.test(row.coverage_status)) {
      addError(errors, `unmapped fixture is marked oracle-verified: ${row.fixture_path}`);
    }
  }

  const counts: AuditCounts = {
    projectFixtures: projectRows.length,
    transpileFixtures: transpileRows.length,
    projectRunnerJson: projectJsonPaths.length,
    mappedTests: mappedNames.size,
    mappedFixtureRows: mappedRows.length,
    missingRootGapRows: audit.filter((row) => row.suite === "project-runner-gap").length,
  };
  return { counts, errors };
}

try {
  const options = parseOptions();
  if (options.help) {
    printUsage();
  } else {
    const { counts, errors } = validate(options);
    console.log(JSON.stringify({ status: errors.length ? "invalid" : "valid", counts, errors }, null, 2));
    if (errors.length) process.exitCode = 1;
  }
} catch (error) {
  console.error(JSON.stringify({ status: "invalid", counts: null, errors: [String(error)] }, null, 2));
  process.exitCode = 1;
}
