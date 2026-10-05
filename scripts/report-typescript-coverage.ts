#!/usr/bin/env node
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { isAbsolute, join, relative, resolve, sep } from "node:path";
import { parseArgs } from "node:util";
import { assertPinnedRepository, TYPESCRIPT_GO_REVISION, readTsv, writeTsv, reportError } from "./typescript-corpus.ts";
import type { TsvRow } from "./typescript-corpus.ts";

interface AreaProgress {
  suite: string;
  area: string;
  sourceCases: number;
  recordedConfigurations: number;
  artifacts: number;
  statuses: Map<string, number>;
  tests: Set<string>;
  caseLinks: Set<string>;
  areaLinks: Set<string>;
}

function behaviorTests(directory: string): Map<string, string> {
  const names = new Map<string, string>();
  for (const filename of readdirSync(directory).filter((name) => name.endsWith(".rs"))) {
    const path = join(directory, filename);
    const text = readFileSync(path, "utf8");
    for (const match of text.matchAll(/^\s*#\[test\]\s*(?:\/\/[^\n]*\n\s*)*fn\s+(\w+)\s*\(/gm)) {
      if (names.has(match[1])) throw new Error(`Duplicate Rust behavior-test name: ${match[1]}`);
      names.set(match[1], path);
    }
  }
  return names;
}

function main(): void {
  const { values } = parseArgs({ options: {
    inventory: { type: "string", default: "docs/typescript-7-case-inventory.tsv" },
    "test-universe": { type: "string", default: "docs/typescript-7-test-universe.md" },
    "test-root": { type: "string", default: "tests" },
    "extension-contracts": { type: "string", default: "docs/typescript-7-extension-contracts.tsv" },
    "option-coverage": { type: "string", default: "docs/typescript-7-option-coverage.tsv" },
    "typescript-go-root": { type: "string" },
    output: { type: "string", default: "docs/typescript-7-coverage-progress.tsv" },
    help: { type: "boolean", short: "h" },
  } });
  if (values.help) {
    console.log("Usage: node scripts/report-typescript-coverage.ts --typescript-go-root PATH [--inventory PATH] [--test-universe PATH] [--test-root PATH] [--extension-contracts PATH] [--option-coverage PATH] [--output PATH]");
    return;
  }
  const inventory = readTsv(values.inventory);
  const goRoot = values["typescript-go-root"];
  if (!goRoot) throw new Error("--typescript-go-root is required to validate oracle references");
  assertPinnedRepository(goRoot, TYPESCRIPT_GO_REVISION, "TypeScript-Go oracle");
  const sourceCases = new Map<string, TsvRow>();
  const groups = new Map<string, AreaProgress>();
  for (const row of inventory) {
    if (sourceCases.has(row.source_case)) throw new Error(`Duplicate corpus fixture: ${row.source_case}`);
    sourceCases.set(row.source_case, row);
    const key = `${row.suite}\t${row.area}`;
    const progress: AreaProgress = groups.get(key) ?? {
      suite: row.suite, area: row.area, sourceCases: 0, recordedConfigurations: 0,
      artifacts: 0, statuses: new Map(), tests: new Set(), caseLinks: new Set(), areaLinks: new Set(),
    };
    const configurations = Number(row.configuration_count);
    const artifacts = Number(row.reference_artifacts);
    if (![configurations, artifacts].every((value) => Number.isInteger(value) && value >= 0)) {
      throw new Error(`Invalid inventory counts: ${row.source_case}`);
    }
    progress.sourceCases += 1;
    progress.recordedConfigurations += configurations;
    progress.artifacts += artifacts;
    progress.statuses.set(row.reference_status, (progress.statuses.get(row.reference_status) ?? 0) + 1);
    groups.set(key, progress);
  }

  const rustTests = behaviorTests(values["test-root"]);
  const mappedTests = new Set<string>();
  const text = readFileSync(values["test-universe"], "utf8");
  for (const match of text.matchAll(/^\| `(should_[^`]+)` \| (.*?) \|/gm)) {
    const name = match[1];
    if (!rustTests.has(name)) throw new Error(`Mapped Rust test does not exist: ${name}`);
    if (mappedTests.has(name)) throw new Error(`Rust behavior test is mapped twice: ${name}`);
    mappedTests.add(name);
    const references = [...match[2].matchAll(/`([^`]+)`/g)].map((item) => item[1]);
    if (references.length === 0) throw new Error(`Rust behavior test has no source/oracle reference: ${name}`);
    for (const reference of references) {
      if (reference.startsWith("typescript-go/") || reference.startsWith("tsc/")) {
        const root = reference.startsWith("typescript-go/")
          ? resolve(goRoot) : resolve(goRoot, "testdata/baselines/reference");
        const path = reference.startsWith("typescript-go/")
          ? resolve(root, reference.slice("typescript-go/".length)) : resolve(root, reference);
        const withinRoot = relative(root, path);
        if (isAbsolute(withinRoot) || withinRoot === ".." || withinRoot.startsWith(`..${sep}`) || !existsSync(path)) {
          throw new Error(`Unknown oracle reference for ${name}: ${reference}`);
        }
        continue;
      }
      const exact = sourceCases.get(reference);
      const matches = exact ? [exact] : inventory.filter((row) => row.source_case.startsWith(`${reference}/`));
      if (matches.length === 0) throw new Error(`Unknown corpus reference for ${name}: ${reference}`);
      for (const key of new Set(matches.map((row) => `${row.suite}\t${row.area}`))) {
        const progress = groups.get(key)!;
        progress.tests.add(name);
        if (exact) progress.caseLinks.add(reference);
        else progress.areaLinks.add(reference);
      }
    }
  }

  let extensionContracts = 0;
  if (existsSync(values["extension-contracts"])) {
    const extensionNames = new Set<string>();
    for (const row of readTsv(values["extension-contracts"])) {
      const testPath = rustTests.get(row.test_name);
      if (!testPath || resolve(testPath) !== resolve(row.location)) {
        throw new Error(`Extension contract does not match a Rust test: ${row.test_name}`);
      }
      if (extensionNames.has(row.test_name)) throw new Error(`Duplicate extension contract: ${row.test_name}`);
      extensionNames.add(row.test_name);
    }
    extensionContracts = extensionNames.size;
  }

  const optionNames = new Set<string>();
  for (const row of readTsv(values["option-coverage"])) {
    if (optionNames.has(row.test_name)) throw new Error(`Duplicate compiler-option test: ${row.test_name}`);
    optionNames.add(row.test_name);
    if (!rustTests.has(row.test_name)) throw new Error(`Compiler-option test does not exist: ${row.test_name}`);
    const fixture = sourceCases.get(row.upstream_case);
    if (!fixture || !["compiler", "conformance"].includes(fixture.suite)) {
      throw new Error(`Unknown compiler-option source case: ${row.upstream_case}`);
    }
    if (!fixture.reference_files.split(";").includes(row.expected_artifact)) {
      throw new Error(`Expected artifact is not indexed for ${row.test_name}: ${row.expected_artifact}`);
    }
    if (!row.oracle_configuration.trim() || !row.expected_behavior.trim()) {
      throw new Error(`Compiler-option mapping is incomplete: ${row.test_name}`);
    }
  }

  const output: TsvRow[] = [...groups.keys()].sort().map((key) => {
    const progress = groups.get(key)!;
    return {
      suite: progress.suite, area: progress.area,
      source_cases: String(progress.sourceCases), recorded_configurations: String(progress.recordedConfigurations),
      reference_artifacts: String(progress.artifacts),
      reference_status_counts: JSON.stringify(Object.fromEntries([...progress.statuses].sort())),
      linked_behavior_tests: String(progress.tests.size), exact_fixture_links: String(progress.caseLinks.size),
      area_reference_links: String(progress.areaLinks.size),
      rust_tests: [...progress.tests].sort().join(";"), sample_fixture_paths: [...progress.caseLinks].sort().join(";"),
      sample_area_paths: [...progress.areaLinks].sort().join(";"),
      mapping_status: progress.tests.size > 0 ? "focused-samples-only" : "pending-focused-tests",
    };
  });
  writeTsv(values.output, [
    "suite", "area", "source_cases", "recorded_configurations", "reference_artifacts",
    "reference_status_counts", "linked_behavior_tests", "exact_fixture_links", "area_reference_links",
    "rust_tests", "sample_fixture_paths", "sample_area_paths", "mapping_status",
  ], output);
  const sampled = output.filter((row) => row.mapping_status === "focused-samples-only").length;
  console.log(JSON.stringify({
    sourceCases: sourceCases.size, groups: groups.size, groupsWithSamples: sampled,
    groupsWithoutSamples: groups.size - sampled, rustTests: rustTests.size,
    mappedCorpusOrOracleBehaviors: mappedTests.size, extensionContracts, output: values.output,
    optionCoverageTests: optionNames.size,
  }, null, 2));
}

try { main(); } catch (error) { reportError(error); }
