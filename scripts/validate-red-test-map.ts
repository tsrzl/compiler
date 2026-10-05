#!/usr/bin/env node
import { readFileSync } from "node:fs";
import { parseArgs } from "node:util";
import { readTsv, reportError } from "./typescript-corpus.ts";

function mappedNamesFromMarkdown(path: string): string[] {
  const text = readFileSync(path, "utf8");
  return [...text.matchAll(/^\| `(should_[^`]+)` \|/gm)].map((match) => match[1]);
}

function main(): void {
  const { values } = parseArgs({ options: {
    "test-results": { type: "string" },
    "test-universe": { type: "string", default: "docs/typescript-7-test-universe.md" },
    "extension-contracts": { type: "string", default: "docs/typescript-7-extension-contracts.tsv" },
    "option-coverage": { type: "string", default: "docs/typescript-7-option-coverage.tsv" },
    "project-audit": { type: "string", default: "docs/typescript-7-project-transpile-audit.tsv" },
    help: { type: "boolean", short: "h" },
  } });
  if (values.help) {
    console.log("Usage: node scripts/validate-red-test-map.ts --test-results PATH [--test-universe PATH] [--extension-contracts PATH] [--option-coverage PATH] [--project-audit PATH]");
    return;
  }
  if (!values["test-results"]) throw new Error("--test-results is required");

  const results = readFileSync(values["test-results"], "utf8");
  const startedCounts = [...results.matchAll(/^running (\d+) tests?$/gm)]
    .map((match) => Number(match[1]));
  const suiteResults = [...results.matchAll(
    /test result: (?:FAILED|ok)\. (\d+) passed; (\d+) failed; (\d+) ignored/g,
  )].map((match) => ({ passed: Number(match[1]), failed: Number(match[2]), ignored: Number(match[3]) }));
  if (startedCounts.length === 0 || startedCounts.length !== suiteResults.length) {
    throw new Error(`Incomplete Rust test results in ${values["test-results"]}`);
  }
  const expectedTestCount = startedCounts.reduce((total, count) => total + count, 0);
  const reportedTestCount = suiteResults.reduce(
    (total, suite) => total + suite.passed + suite.failed + suite.ignored,
    0,
  );
  if (expectedTestCount !== reportedTestCount) {
    throw new Error(
      `Incomplete Rust test log: started ${expectedTestCount} tests but received results for ${reportedTestCount}`,
    );
  }
  const failedNames = [...results.matchAll(/^test (should_[A-Za-z0-9_]+) \.\.\. FAILED$/gm)]
    .map((match) => match[1]);
  const duplicateFailures = failedNames.filter((name, index) => failedNames.indexOf(name) !== index);
  if (duplicateFailures.length > 0) {
    throw new Error(`Duplicate failed behavior test names: ${[...new Set(duplicateFailures)].sort().join(", ")}`);
  }

  const mappedNames = new Set([
    ...mappedNamesFromMarkdown(values["test-universe"]),
    ...readTsv(values["extension-contracts"]).map((row) => row.test_name),
    ...readTsv(values["option-coverage"]).map((row) => row.test_name),
    ...readTsv(values["project-audit"]).flatMap((row) => row.rust_test_mapping.split(";").filter(Boolean)),
  ]);
  const unmappedFailures = failedNames.filter((name) => !mappedNames.has(name)).sort();
  const report = {
    status: unmappedFailures.length === 0 ? "valid" : "invalid",
    executedBehaviorTests: reportedTestCount,
    failedBehaviorTests: failedNames.length,
    mappedFailures: failedNames.length - unmappedFailures.length,
    unmappedFailures,
  };
  console.log(JSON.stringify(report, null, 2));
  if (unmappedFailures.length > 0) process.exitCode = 1;
}

try { main(); } catch (error) { reportError(error); }
