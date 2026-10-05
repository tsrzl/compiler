#!/usr/bin/env node
/** Record pinned Go test-runner evidence for cases without reference artifacts. */

import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { parseArgs } from "node:util";
import type { TsvRow } from "./typescript-corpus.ts";
import {
  assertPinnedRepository,
  readTsv,
  reportError,
  TYPESCRIPT_GO_REVISION,
  TYPESCRIPT_REVISION,
  writeTsv,
} from "./typescript-corpus.ts";

const FIELDS = [
  "source_case",
  "configuration",
  "oracle_status",
  "reference_status",
  "reason",
  "checks",
  "oracle_test",
  "typescript_revision",
  "typescript_go_revision",
] as const;

const REQUIRED_CHECKS = [
  "error",
  "output",
  "sourcemap",
  "sourcemap_record",
  "union_ordering",
  "source_file_parent_pointers",
] as const;
const DEFAULT_INVENTORY = "docs/typescript-7-case-inventory.tsv";
const DEFAULT_OUTPUT = "docs/typescript-7-reference-audit.tsv";

type GoEvent = {
  Action?: string;
  Package?: string;
  Test?: string;
  Output?: string;
};

function skipStatus(reason: string): string {
  const patterns: Array<[RegExp, string]> = [
    [/^unsupported module kind (AMD|UMD|System)$/, "unsupported-module-$1"],
    [/^unsupported module resolution kind (1|2)$/, "unsupported-module-resolution-$1"],
    [/^unsupported baseUrl .+$/, "unsupported-baseurl"],
    [/^unsupported outFile .+$/, "unsupported-outfile"],
    [/^unsupported target ES5$/, "unsupported-target-es5"],
    [/^esModuleInterop=false is unsupported$/, "esmoduleinterop-false"],
    [/^allowSyntheticDefaultImports=false is unsupported$/, "allow-synthetic-default-imports-false"],
    [/^alwaysStrict=false is unsupported$/, "always-strict-false"],
  ];
  for (const [pattern, template] of patterns) {
    const match = reason.match(pattern);
    if (!match) continue;
    let status = template.includes("$1")
      ? template.replace("$1", match[1].toLowerCase())
      : template;
    if (status.endsWith("-1")) status = status.slice(0, -2) + "-classic";
    if (status.endsWith("-2")) status = status.slice(0, -2) + "-node10";
    return `unsupported-by-tsgo:${status}`;
  }
  throw new Error(`Unclassified pinned-runner skip: ${reason}`);
}

function readInventory(path: string): {
  candidates: Map<string, TsvRow>;
  byName: Map<string, string>;
} {
  const rows = readTsv(path);
  const candidates = new Map<string, TsvRow>();
  for (const row of rows) {
    if (
      (row.suite === "compiler" || row.suite === "conformance") &&
      Number.parseInt(row.reference_artifacts, 10) === 0
    ) {
      candidates.set(row.source_case, row);
    }
  }

  const byName = new Map<string, string>();
  for (const sourceCase of candidates.keys()) {
    const name = sourceCase.split(/[\\/]/).at(-1) ?? "";
    if (byName.has(name)) throw new Error(`Ambiguous compiler fixture name: ${name}`);
    byName.set(name, sourceCase);
  }
  return { candidates, byName };
}

function expectedSelection(inventoryPath: string, previousAuditPath: string): Map<string, Set<string>> {
  const { candidates } = readInventory(inventoryPath);
  const expected = new Map<string, Set<string>>();
  for (const [sourceCase, row] of candidates) {
    const configurations = row.configurations ? row.configurations.split(";") : [];
    if (Number(row.configuration_count) !== configurations.length || new Set(configurations).size !== configurations.length) {
      throw new Error(`Invalid inventory configuration list: ${sourceCase}`);
    }
    if (configurations.length > 0 || row.reference_status === "no-reference-artifact-found") {
      expected.set(sourceCase, new Set(configurations));
    }
  }
  if (existsSync(previousAuditPath)) {
    const priorPairs = new Set<string>();
    for (const row of readTsv(previousAuditPath)) {
      if (!candidates.has(row.source_case) || !row.configuration) {
        throw new Error(`Invalid prior audited configuration: ${row.source_case}`);
      }
      if (row.typescript_revision !== TYPESCRIPT_REVISION || row.typescript_go_revision !== TYPESCRIPT_GO_REVISION) {
        throw new Error(`Prior audit has a different oracle revision: ${previousAuditPath}`);
      }
      const key = `${row.source_case}\t${row.configuration}`;
      if (priorPairs.has(key)) throw new Error(`Duplicate prior audited configuration: ${key}`);
      priorPairs.add(key);
      const configurations = expected.get(row.source_case) ?? new Set<string>();
      configurations.add(row.configuration);
      expected.set(row.source_case, configurations);
    }
  }
  if (expected.size === 0) throw new Error("No compiler cases were selected for the pinned runner");
  return expected;
}

function escapeRegex(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function runOracle(
  typescriptRoot: string,
  typescriptGoRoot: string,
  runnerResultsPath: string,
  inventoryPath: string,
  previousAuditPath: string,
): void {
  const corpusCasesPath = realpathSync(join(typescriptRoot, "tests", "cases"));
  const mountedCasesPath = realpathSync(join(typescriptGoRoot, "_submodules", "TypeScript", "tests", "cases"));
  if (corpusCasesPath !== mountedCasesPath) {
    throw new Error(
      `TypeScript-Go test corpus resolves to ${mountedCasesPath}; expected ${corpusCasesPath}`,
    );
  }

  const sourceCases = [...expectedSelection(inventoryPath, previousAuditPath).keys()].sort();
  const basenames = sourceCases.map((sourceCase) => {
    const basename = sourceCase.split(/[\\/]/).at(-1);
    if (!basename) throw new Error(`Invalid source case path: ${sourceCase}`);
    return escapeRegex(basename);
  });
  const runPattern = `^TestSubmodule$/^(${basenames.join("|")})(_|$)`;
  const result = spawnSync(
    "go",
    ["test", "./internal/testrunner", "-run", runPattern, "-count=1", "-json"],
    { cwd: typescriptGoRoot, encoding: "utf8", maxBuffer: 32 * 1024 * 1024 },
  );
  if (result.error) throw result.error;

  mkdirSync(dirname(runnerResultsPath), { recursive: true });
  writeFileSync(runnerResultsPath, result.stdout ?? "", "utf8");
  if (result.status !== 0) {
    const detail = (result.stderr ?? "").trim();
    throw new Error(
      `Pinned Go runner exited with status ${String(result.status)}${detail ? `: ${detail}` : ""}`,
    );
  }
}

function auditRows(resultsPath: string, inventoryPath: string, previousAuditPath: string): TsvRow[] {
  const { candidates, byName } = readInventory(inventoryPath);
  const events = readFileSync(resultsPath, "utf8")
    .split(/\r?\n/)
    .filter((line) => line.trim().length > 0 && !line.startsWith("go: downloading "))
    .map((line) => JSON.parse(line) as GoEvent)
    .filter((event) => event.Package === "github.com/microsoft/typescript-go/internal/testrunner");

  if (!events.some((event) => event.Action === "pass" && !event.Test)) {
    throw new Error("The pinned compiler runner did not complete successfully");
  }
  if (!events.some((event) => event.Test === "TestSubmodule" && event.Action === "pass")) {
    throw new Error("TestSubmodule did not run and pass");
  }
  if (events.some((event) => event.Action === "fail")) {
    throw new Error("The pinned compiler runner contains failing evidence");
  }

  const tests = new Map<string, GoEvent[]>();
  for (const event of events) {
    const name = event.Test ?? "";
    if (!name.startsWith("TestSubmodule/")) continue;
    const group = tests.get(name) ?? [];
    group.push(event);
    tests.set(name, group);
  }

  const rows: TsvRow[] = [];
  const seenCases = new Set<string>();
  const sortedTests = [...tests.entries()].sort(([left], [right]) =>
    left < right ? -1 : left > right ? 1 : 0,
  );
  for (const [testName, testEvents] of sortedTests) {
    const parts = testName.split("/");
    if (parts.length !== 2) continue;

    const matchingNames = [...byName.keys()].filter(
      (filename) => parts[1] === filename || parts[1].startsWith(`${filename}_`),
    );
    if (matchingNames.length !== 1) {
      throw new Error(`Runner selected an unexpected case: ${testName}`);
    }
    const filename = matchingNames[0];
    const sourceCase = byName.get(filename);
    if (!sourceCase) throw new Error(`Missing inventory entry for compiler fixture: ${filename}`);
    const candidate = candidates.get(sourceCase);
    if (!candidate) throw new Error(`Missing candidate row for compiler fixture: ${sourceCase}`);

    const suffix = parts[1].slice(filename.length).replace(/^_/, "") || "default";
    const actions = testEvents.map((event) => event.Action ?? "");
    if (actions.filter((action) => action === "run").length !== 1 || !["pass", "skip"].includes(actions.at(-1) ?? "")) {
      throw new Error(`Incomplete or failing runner evidence: ${testName}`);
    }
    const outcome = actions.at(-1) ?? "";
    const checks: string[] = [];
    let reason: string;
    let status: string;
    if (outcome === "skip") {
      const output = testEvents.map((event) => event.Output ?? "").join("");
      const match = output.match(/compiler_runner\.go:\d+: (.*)\n/);
      if (!match) throw new Error(`Missing skip reason: ${testName}`);
      reason = match[1].trim();
      status = skipStatus(reason);
    } else {
      for (const check of REQUIRED_CHECKS) {
        const child = tests.get(`${testName}/${check}`) ?? [];
        if (child.filter((event) => event.Action === "run").length !== 1 || child.at(-1)?.Action !== "pass") {
          throw new Error(`Missing successful ${check} check: ${testName}`);
        }
        checks.push(check);
      }
      status = "verified-empty-reference-output";
      reason = "pinned runner accepted absent diagnostic, emit, and source-map baselines";
      const directives = candidate.directives.split(" | ");
      for (const option of ["noemit=true", "notypesandsymbols=true"]) {
        if (directives.includes(option)) reason += `; ${option}`;
      }
    }

    seenCases.add(sourceCase);
    rows.push({
      source_case: sourceCase,
      configuration: suffix,
      oracle_status: outcome,
      reference_status: status,
      reason,
      checks: checks.sort().join(";"),
      oracle_test: testName,
      typescript_revision: TYPESCRIPT_REVISION,
      typescript_go_revision: TYPESCRIPT_GO_REVISION,
    });
  }

  const expected = expectedSelection(inventoryPath, previousAuditPath);
  const expectedCases = new Set(expected.keys());

  const missing = [...expectedCases].filter((sourceCase) => !seenCases.has(sourceCase)).sort();
  const unexpected = [...seenCases].filter((sourceCase) => !expectedCases.has(sourceCase)).sort();
  if (expectedCases.size === 0 || missing.length > 0 || unexpected.length > 0) {
    throw new Error(`Incomplete selection: missing=${JSON.stringify(missing)}, unexpected=${JSON.stringify(unexpected)}`);
  }
  const observedPairs = new Set(rows.map((row) => `${row.source_case}\t${row.configuration}`));
  const missingConfigurations = [...expected].flatMap(([sourceCase, configurations]) =>
    [...configurations].map((configuration) => `${sourceCase}\t${configuration}`)
      .filter((key) => !observedPairs.has(key)),
  );
  if (missingConfigurations.length > 0) {
    throw new Error(`Incomplete configuration selection: ${JSON.stringify(missingConfigurations.sort())}`);
  }
  const unexpectedConfigurations = rows.filter((row) => {
    const configurations = expected.get(row.source_case)!;
    return configurations.size > 0 && !configurations.has(row.configuration);
  });
  if (unexpectedConfigurations.length > 0) {
    throw new Error(`Unexpected configuration selection: ${JSON.stringify(unexpectedConfigurations.map((row) => `${row.source_case}\t${row.configuration}`))}`);
  }
  return rows;
}

function requiredValue(value: string | undefined, name: string): string {
  if (value === undefined) throw new Error(`Missing required option: --${name}`);
  return value;
}

function main(): void {
  const { values } = parseArgs({
    options: {
      "typescript-root": { type: "string" },
      "typescript-go-root": { type: "string" },
      "runner-results": { type: "string" },
      inventory: { type: "string", default: DEFAULT_INVENTORY },
      output: { type: "string", default: DEFAULT_OUTPUT },
      "previous-audit": { type: "string", default: DEFAULT_OUTPUT },
      "run-oracle": { type: "boolean", default: false },
      help: { type: "boolean", short: "h" },
    },
    strict: true,
    allowPositionals: false,
  });
  if (values.help) {
    console.log("Usage: node scripts/audit-typescript-reference-gaps.ts --typescript-root PATH --typescript-go-root PATH --runner-results PATH [--run-oracle] [--inventory PATH] [--previous-audit PATH] [--output PATH]");
    return;
  }
  const typescriptRoot = requiredValue(values["typescript-root"], "typescript-root");
  const typescriptGoRoot = requiredValue(values["typescript-go-root"], "typescript-go-root");
  const runnerResults = requiredValue(values["runner-results"], "runner-results");
  const inventory = values.inventory ?? DEFAULT_INVENTORY;
  const output = values.output ?? DEFAULT_OUTPUT;
  const previousAudit = values["previous-audit"] ?? DEFAULT_OUTPUT;

  assertPinnedRepository(typescriptRoot, TYPESCRIPT_REVISION, "TypeScript corpus");
  assertPinnedRepository(typescriptGoRoot, TYPESCRIPT_GO_REVISION, "TypeScript-Go oracle");
  if (values["run-oracle"]) {
    runOracle(typescriptRoot, typescriptGoRoot, runnerResults, inventory, previousAudit);
  }
  const rows = auditRows(runnerResults, inventory, previousAudit);
  writeTsv(output, FIELDS, rows);
  const caseCount = new Set(rows.map((row) => row.source_case)).size;
  console.log(`Audited ${rows.length} configurations across ${caseCount} cases into ${output}`);
}

try {
  main();
} catch (error) {
  reportError(error);
}
