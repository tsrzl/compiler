#!/usr/bin/env node
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { basename, extname, join, relative, sep } from "node:path";
import { parseArgs } from "node:util";
import {
  TYPESCRIPT_REVISION, TYPESCRIPT_GO_REVISION,
  assertPinnedRepository, readTsv, writeTsv, reportError,
} from "./typescript-corpus.ts";
import type { TsvRow } from "./typescript-corpus.ts";

const SUITES = ["compiler", "conformance", "projects", "transpile"] as const;
const REFERENCE_SUFFIXES = [
  ".errors.txt", ".js", ".js.map", ".sourcemap.txt", ".symbols", ".trace.json", ".types",
];
const FIELDS = [
  "suite", "area", "source_case", "extension", "directives", "configuration_count",
  "configurations", "reference_artifacts", "artifact_kinds", "reference_files", "reference_status",
];
type Directive = readonly [name: string, value: string];
type ReviewedOutcomes = Map<string, Map<string, string>>;

function caseFiles(directory: string): string[] {
  const files: string[] = [];
  const entries = readdirSync(directory, { withFileTypes: true })
    .sort((left, right) => left.name < right.name ? -1 : left.name > right.name ? 1 : 0);
  for (const entry of entries) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) files.push(...caseFiles(path));
    else if (entry.isFile() && [".ts", ".tsx"].includes(extname(entry.name))) files.push(path);
  }
  return files;
}

function caseDirectives(path: string): Directive[] {
  const bytes = readFileSync(path);
  let encoding = "utf-8";
  if (bytes[0] === 0xff && bytes[1] === 0xfe) encoding = "utf-16le";
  else if (bytes[0] === 0xfe && bytes[1] === 0xff) encoding = "utf-16be";
  const text = new TextDecoder(encoding, { fatal: true }).decode(bytes);
  const directives: Directive[] = [];
  for (const line of text.split(/\r\n|\n|\r/)) {
    const match = /^\s*\/\/\s*@([\w-]+)\s*:\s*(.*?)\s*$/.exec(line);
    if (match) directives.push([match[1].toLowerCase(), match[2]]);
  }
  return directives;
}

function staticSkips(root: string): Set<string> {
  const path = join(root, "internal", "testrunner", "compiler_runner.go");
  const match = /var skippedTests = \[\]string\{([\s\S]*?)\n\}/.exec(readFileSync(path, "utf8"));
  if (!match) throw new Error(`Cannot find the pinned test runner skip list in ${path}`);
  return new Set([...match[1].matchAll(/"([^"\n]+)"/g)].map((item) => item[1]));
}

function unsupportedOption(directives: readonly Directive[]): string | undefined {
  const settings = new Map(directives.filter(([name]) => name !== "filename"));
  const values = (name: string): Set<string> =>
    new Set((settings.get(name) ?? "").split(",").map((value) => value.trim().toLowerCase()));
  // A fixed priority makes multi-option exclusions reproducible across runs.
  for (const value of ["amd", "umd", "system"]) {
    if (values("module").has(value)) return `unsupported-module-${value}`;
  }
  for (const value of ["node10", "classic"]) {
    if (values("moduleresolution").has(value)) return `unsupported-module-resolution-${value}`;
  }
  if (values("esmoduleinterop").has("false")) return "esmoduleinterop-false";
  if (values("allowsyntheticdefaultimports").has("false")) return "allow-synthetic-default-imports-false";
  if (settings.get("baseurl")?.trim()) return "unsupported-baseurl";
  if (settings.get("outfile")?.trim()) return "unsupported-outfile";
  if (values("target").has("es5")) return "unsupported-target-es5";
  if (values("alwaysstrict").has("false")) return "always-strict-false";
  return undefined;
}

function parseReferenceArtifact(stem: string, filename: string): [string, string] | undefined {
  if (!filename.startsWith(stem)) return undefined;
  let remainder = filename.slice(stem.length);
  let configuration = "default";
  if (remainder.startsWith("(")) {
    const closing = remainder.indexOf(")");
    if (closing < 0) return undefined;
    configuration = remainder.slice(1, closing);
    remainder = remainder.slice(closing + 1);
  }
  return REFERENCE_SUFFIXES.some((suffix) => remainder === suffix || remainder === `${suffix}.diff`)
    ? [configuration, remainder] : undefined;
}

function referenceArtifacts(stem: string, names: readonly string[]): string[] {
  let low = 0;
  let high = names.length;
  while (low < high) {
    const middle = Math.floor((low + high) / 2);
    if (names[middle] < stem) low = middle + 1;
    else high = middle;
  }
  const artifacts: string[] = [];
  for (let index = low; index < names.length && names[index].startsWith(stem); index += 1) {
    if (parseReferenceArtifact(stem, names[index])) artifacts.push(names[index]);
  }
  return artifacts;
}

function reviewedReferenceOutcomes(path: string): ReviewedOutcomes {
  const byCase: ReviewedOutcomes = new Map();
  if (!existsSync(path)) return byCase;
  for (const row of readTsv(path)) {
    if (row.typescript_revision !== TYPESCRIPT_REVISION || row.typescript_go_revision !== TYPESCRIPT_GO_REVISION) {
      throw new Error(`Reference audit has a different oracle revision: ${path}`);
    }
    const status = row.reference_status;
    if (status !== "verified-empty-reference-output" && !status?.startsWith("unsupported-by-tsgo:")) {
      throw new Error(`Invalid reference audit status: ${status}`);
    }
    if (row.oracle_status !== (status === "verified-empty-reference-output" ? "pass" : "skip")) {
      throw new Error(`Reference audit outcome disagrees with its status: ${row.source_case}`);
    }
    const configurations = byCase.get(row.source_case) ?? new Map<string, string>();
    if (configurations.has(row.configuration)) throw new Error(`Duplicate audited configuration: ${row.source_case}`);
    configurations.set(row.configuration, status);
    byCase.set(row.source_case, configurations);
  }
  return byCase;
}

function inventoryRows(typescriptRoot: string, goRoot: string, audit: ReviewedOutcomes): TsvRow[] {
  const rows: TsvRow[] = [];
  const caseRoot = join(typescriptRoot, "tests", "cases");
  const baselineRoot = join(goRoot, "testdata", "baselines", "reference", "submodule");
  const skipped = staticSkips(goRoot);
  for (const suite of SUITES) {
    const suiteRoot = join(caseRoot, suite);
    const files = caseFiles(suiteRoot);
    const stems = files.map((path) => basename(path, extname(path)));
    const hasBaselines = suite === "compiler" || suite === "conformance";
    if (hasBaselines && new Set(stems).size !== stems.length) {
      throw new Error(`${suite} has duplicate case basenames`);
    }
    const names = hasBaselines
      ? readdirSync(join(baselineRoot, suite), { withFileTypes: true })
          .filter((entry) => entry.isFile()).map((entry) => entry.name).sort()
      : [];
    for (const path of files) {
      const sourceCase = relative(caseRoot, path).split(sep).join("/");
      const parts = relative(suiteRoot, path).split(sep);
      const directives = caseDirectives(path);
      const artifacts = referenceArtifacts(basename(path, extname(path)), names);
      const configurations = new Set<string>();
      const kinds = new Set<string>();
      for (const artifact of artifacts) {
        const parsed = parseReferenceArtifact(basename(path, extname(path)), artifact);
        if (parsed) {
          configurations.add(parsed[0]);
          kinds.add(parsed[1]);
        }
      }
      const reviewed = audit.get(sourceCase);
      let status: string;
      if (reviewed) {
        if (artifacts.length > 0) throw new Error(`Audited missing-reference case now has artifacts: ${sourceCase}`);
        for (const configuration of reviewed.keys()) configurations.add(configuration);
        const statuses = new Set(reviewed.values());
        status = statuses.size === 1 ? [...statuses][0] : "reviewed-mixed-oracle-configurations";
      } else if (!hasBaselines) status = "no-submodule-reference-suite";
      else if (artifacts.length > 0) status = "reference-artifacts-present";
      else if (skipped.has(basename(path))) status = "explicit-tsgo-test-skip";
      else {
        const reason = unsupportedOption(directives);
        status = reason ? `unsupported-by-tsgo:${reason}` : "no-reference-artifact-found";
      }
      rows.push({
        suite, area: parts.length > 1 ? parts[0] : "(root)", source_case: sourceCase,
        extension: extname(path), directives: directives.map(([name, value]) => `${name}=${value}`).join(" | "),
        configuration_count: String(configurations.size), configurations: [...configurations].sort().join(";"),
        reference_artifacts: String(artifacts.length), artifact_kinds: [...kinds].sort().join(";"),
        reference_files: artifacts.join(";"), reference_status: status,
      });
    }
  }
  const indexed = new Set(rows.map((row) => row.source_case));
  for (const sourceCase of audit.keys()) {
    if (!indexed.has(sourceCase)) throw new Error(`Reference audit contains an unknown source case: ${sourceCase}`);
  }
  return rows;
}

function main(): void {
  const { values } = parseArgs({ options: {
    "typescript-root": { type: "string" }, "typescript-go-root": { type: "string" },
    "reference-audit": { type: "string", default: "docs/typescript-7-reference-audit.tsv" },
    output: { type: "string", default: "docs/typescript-7-case-inventory.tsv" },
    help: { type: "boolean", short: "h" },
  } });
  if (values.help) {
    console.log("Usage: node scripts/generate-typescript-case-inventory.ts --typescript-root PATH --typescript-go-root PATH [--reference-audit PATH] [--output PATH]");
    return;
  }
  const tsRoot = values["typescript-root"];
  const goRoot = values["typescript-go-root"];
  if (!tsRoot || !goRoot) throw new Error("--typescript-root and --typescript-go-root are required");
  assertPinnedRepository(tsRoot, TYPESCRIPT_REVISION, "TypeScript corpus");
  assertPinnedRepository(goRoot, TYPESCRIPT_GO_REVISION, "TypeScript-Go oracle");
  const rows = inventoryRows(tsRoot, goRoot, reviewedReferenceOutcomes(values["reference-audit"]));
  writeTsv(values.output, FIELDS, rows);
  console.log(`Indexed ${rows.length} source cases into ${values.output}`);
}

try { main(); } catch (error) { reportError(error); }
