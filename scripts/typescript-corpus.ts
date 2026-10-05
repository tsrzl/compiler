import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";

export const TYPESCRIPT_REVISION = "4d4f005c8541e0255a9d8791205fdce326e462bc";
export const TYPESCRIPT_GO_REVISION = "2bd066d87f5bafd315be9f40889d0a60b9e58e0b";
export type TsvRow = Record<string, string>;

export function assertPinnedRepository(root: string, expected: string, label: string): void {
  const actual = execFileSync("git", ["-C", root, "rev-parse", "HEAD"], {
    encoding: "utf8",
  }).trim();
  if (actual !== expected) {
    throw new Error(`${label} revision is ${actual}; expected ${expected}`);
  }
  const changes = execFileSync("git", [
    "-C", root, "status", "--porcelain", "--untracked-files=all", "--ignore-submodules=all",
  ], { encoding: "utf8" }).trim();
  if (changes) {
    throw new Error(`${label} checkout has modified or untracked files; use a clean pinned checkout`);
  }
}

function parseTsv(text: string): string[][] {
  const rows: string[][] = [];
  let row: string[] = [];
  let field = "";
  let quoted = false;
  let closedQuote = false;
  for (let index = 0; index < text.length; index += 1) {
    const char = text[index];
    if (quoted) {
      if (char === '"') {
        if (text[index + 1] === '"') {
          field += '"';
          index += 1;
        } else {
          quoted = false;
          closedQuote = true;
        }
      } else {
        field += char;
      }
      continue;
    }
    if (char === "\t" || char === "\n" || char === "\r") {
      row.push(field);
      field = "";
      closedQuote = false;
      if (char !== "\t") {
        rows.push(row);
        row = [];
        if (char === "\r" && text[index + 1] === "\n") index += 1;
      }
    } else if (closedQuote) {
      throw new Error("Unexpected data after a quoted TSV field");
    } else if (char === '"' && field.length === 0) {
      quoted = true;
    } else {
      field += char;
    }
  }
  if (quoted) throw new Error("Unterminated quoted TSV field");
  if (field.length > 0 || row.length > 0 || closedQuote) {
    row.push(field);
    rows.push(row);
  }
  return rows;
}

export function readTsv(path: string): TsvRow[] {
  const rows = parseTsv(readFileSync(path, "utf8").replace(/^\uFEFF/, ""));
  const fields = rows.shift();
  if (!fields || new Set(fields).size !== fields.length || fields.some((field) => !field)) {
    throw new Error(`Invalid TSV header: ${path}`);
  }
  return rows.map((values, index) => {
    if (values.length !== fields.length) {
      throw new Error(`TSV row ${index + 2} in ${path} has ${values.length} fields; expected ${fields.length}`);
    }
    return Object.fromEntries(fields.map((field, fieldIndex) => [field, values[fieldIndex]]));
  });
}

export function writeTsv(path: string, fields: readonly string[], rows: readonly TsvRow[]): void {
  const encode = (value: string): string =>
    /["\t\r\n]/.test(value) ? `"${value.replaceAll('"', '""')}"` : value;
  const lines = [fields.map(encode).join("\t")];
  for (const row of rows) {
    lines.push(fields.map((field) => {
      if (typeof row[field] !== "string") throw new Error(`Missing TSV field: ${field}`);
      return encode(row[field]);
    }).join("\t"));
  }
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, `${lines.join("\n")}\n`, "utf8");
}

export function reportError(error: unknown): void {
  console.error(`error: ${error instanceof Error ? error.message : String(error)}`);
  process.exitCode = 1;
}
