#!/usr/bin/env node
import { readFileSync, readdirSync } from "node:fs";
import { parseArgs } from "node:util";
import { join } from "node:path";
import { readTsv, writeTsv, reportError } from "./typescript-corpus.ts";
import type { TsvRow } from "./typescript-corpus.ts";

type SourceLinks = {
  exact: Set<string>;
  area: Set<string>;
  tests: Set<string>;
};

function testsInDirectory(directory: string): Set<string> {
  const names = new Set<string>();
  for (const filename of readdirSync(directory).filter((entry) => entry.endsWith(".rs"))) {
    const source = readFileSync(join(directory, filename), "utf8");
    for (const match of source.matchAll(/^\s*#\[test\]\s*(?:\/\/[^\n]*\n\s*)*fn\s+(should_\w+)\s*\(/gm)) {
      if (names.has(match[1])) throw new Error(`Duplicate Rust behavior test: ${match[1]}`);
      names.add(match[1]);
    }
  }
  return names;
}

function main(): void {
  const { values } = parseArgs({
    options: {
      inventory: { type: "string", default: "docs/typescript-7-case-inventory.tsv" },
      "test-universe": { type: "string", default: "docs/typescript-7-test-universe.md" },
      "project-audit": { type: "string", default: "docs/typescript-7-project-transpile-audit.tsv" },
      "test-root": { type: "string", default: "tests" },
      output: { type: "string", default: "docs/typescript-7-rust-behavior-backlog.tsv" },
      help: { type: "boolean", short: "h" },
    },
  });
  if (values.help) {
    console.log("Usage: node scripts/generate-typescript-behavior-backlog.ts [--inventory PATH] [--test-universe PATH] [--project-audit PATH] [--test-root PATH] [--output PATH]");
    return;
  }

  const inventory = readTsv(values.inventory);
  const tests = testsInDirectory(values["test-root"]);
  const links = new Map<string, SourceLinks>();
  for (const row of inventory) {
    if (links.has(row.source_case)) throw new Error(`Duplicate corpus fixture: ${row.source_case}`);
    links.set(row.source_case, { exact: new Set(), area: new Set(), tests: new Set() });
  }

  const mapText = readFileSync(values["test-universe"], "utf8");
  for (const match of mapText.matchAll(/^\| `(should_[^`]+)` \| (.*?) \|/gm)) {
    const [, testName, referenceColumn] = match;
    if (!tests.has(testName)) throw new Error(`Mapped test does not exist: ${testName}`);
    const references = [...referenceColumn.matchAll(/`([^`]+)`/g)].map((reference) => reference[1]);
    if (references.length === 0) throw new Error(`Mapped test has no source/oracle reference: ${testName}`);
    for (const reference of references) {
      if (reference.startsWith("typescript-go/") || reference.startsWith("tsc/")) continue;
      const exact = links.get(reference);
      if (exact) {
        exact.exact.add(testName);
        exact.tests.add(testName);
        continue;
      }
      const matches = inventory.filter((row) => row.source_case.startsWith(`${reference}/`));
      if (matches.length === 0) throw new Error(`Unknown source reference: ${reference}`);
      for (const row of matches) {
        const source = links.get(row.source_case)!;
        source.area.add(testName);
        source.tests.add(testName);
      }
    }
  }

  const projectAudit = readTsv(values["project-audit"]);
  const projectSources = new Set(inventory
    .filter((fixture) => ["projects", "transpile"].includes(fixture.suite))
    .map((fixture) => fixture.source_case));
  const projectFixtures = new Map<string, TsvRow>();
  for (const row of projectAudit) {
    if (["projects", "transpile"].includes(row.suite)) {
      if (!projectSources.has(row.fixture_path)) throw new Error(`Project audit has an unknown fixture: ${row.fixture_path}`);
      if (projectFixtures.has(row.fixture_path)) throw new Error(`Duplicate project/transpile fixture audit: ${row.fixture_path}`);
      projectFixtures.set(row.fixture_path, row);
    }
    if (!row.rust_test_mapping) continue;
    const source = links.get(row.fixture_path);
    if (!source) throw new Error(`Project audit maps an unknown source fixture: ${row.fixture_path}`);
    for (const testName of row.rust_test_mapping.split(";").filter(Boolean)) {
      if (!tests.has(testName)) {
        throw new Error(`Project fixture maps to a missing Rust test: ${testName}`);
      }
      source.exact.add(testName);
      source.tests.add(testName);
    }
  }
  const missingProjectRows = [...projectSources].filter((sourceCase) => !projectFixtures.has(sourceCase));
  if (missingProjectRows.length > 0 || projectFixtures.size !== projectSources.size) {
    throw new Error(`Incomplete project/transpile audit: ${JSON.stringify(missingProjectRows.sort())}`);
  }
  if (!projectAudit.some((row) => row.suite === "project-runner-gap")) {
    throw new Error("Project audit is missing its project-runner gap classification");
  }

  const rows: TsvRow[] = inventory.map((fixture) => {
    const source = links.get(fixture.source_case)!;
    const exactTests = [...source.exact].sort();
    const areaTests = [...source.area].filter((test) => !source.exact.has(test)).sort();
    return {
      suite: fixture.suite,
      area: fixture.area,
      source_case: fixture.source_case,
      configuration_count: fixture.configuration_count,
      configurations: fixture.configurations,
      directives: fixture.directives,
      reference_artifacts: fixture.reference_artifacts,
      artifact_kinds: fixture.artifact_kinds,
      reference_files: fixture.reference_files,
      oracle_status: fixture.reference_status,
      project_scenario: projectFixtures.get(fixture.source_case)?.project_scenario ?? "",
      project_configuration: projectFixtures.get(fixture.source_case)?.relevant_configuration ?? "",
      upstream_case_json: projectFixtures.get(fixture.source_case)?.upstream_case_json ?? "",
      project_fixture_role: projectFixtures.get(fixture.source_case)?.fixture_role ?? "",
      project_oracle_expectation: projectFixtures.get(fixture.source_case)?.oracle_expectation ?? "",
      project_coverage_status: projectFixtures.get(fixture.source_case)?.coverage_status ?? "",
      rust_behavior_map_status: exactTests.length > 0
        ? "references_exact_case"
        : areaTests.length > 0 ? "area_sample_only" : "no_mapped_test_reference",
      exact_fixture_tests: exactTests.join(";"),
      area_sample_tests: areaTests.join(";"),
      mapped_behavior_test_count: String(source.tests.size),
    };
  });
  writeTsv(values.output, [
    "suite", "area", "source_case", "configuration_count", "configurations", "directives",
    "reference_artifacts", "artifact_kinds", "reference_files", "oracle_status",
    "project_scenario", "project_configuration", "upstream_case_json", "project_fixture_role",
    "project_oracle_expectation", "project_coverage_status", "rust_behavior_map_status",
    "exact_fixture_tests", "area_sample_tests", "mapped_behavior_test_count",
  ], rows);

  const counts = Object.fromEntries(
    ["references_exact_case", "area_sample_only", "no_mapped_test_reference"].map((status) => [
      status,
      rows.filter((row) => row.rust_behavior_map_status === status).length,
    ]),
  );
  console.log(JSON.stringify({ sourceCases: rows.length, statusCounts: counts, output: values.output }, null, 2));
}

try { main(); } catch (error) { reportError(error); }
