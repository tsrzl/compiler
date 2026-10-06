#!/usr/bin/env node
// Generates src/ast/flags.rs and src/check/flags.rs from the pinned TypeScript-Go flag constants.
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { parseArgs } from "node:util";
import { assertPinnedRepository, TYPESCRIPT_GO_REVISION, reportError } from "./typescript-corpus.ts";

type Module = "ast" | "check";

interface FlagSet {
  goType: string;
  module: Module;
  file: string;
  doc: string;
}

const PACKAGE_DIRECTORIES: Record<Module, string> = { ast: "internal/ast", check: "internal/checker" };
const MACRO_IMPORTS: Record<Module, string> = { ast: "super::flags_type::flags_type", check: "crate::ast::flags_type" };

const FLAG_SETS: FlagSet[] = [
  { goType: "TokenFlags", module: "ast", file: "tokenflags.go", doc: "Flags describing how a token was written." },
  { goType: "NodeFlags", module: "ast", file: "nodeflags.go", doc: "Flags describing a node's syntax and parse context." },
  { goType: "ModifierFlags", module: "ast", file: "modifierflags.go", doc: "Flags summarizing a declaration's modifiers." },
  { goType: "SymbolFlags", module: "ast", file: "symbolflags.go", doc: "Flags classifying the declarations merged into a symbol." },
  { goType: "FlowFlags", module: "ast", file: "flow.go", doc: "Flags classifying a control flow graph node." },
  { goType: "CheckFlags", module: "ast", file: "checkflags.go", doc: "Flags describing a symbol the checker created." },
  { goType: "TypeFlags", module: "check", file: "types.go", doc: "Flags classifying a type; their numeric order also orders union constituents." },
  { goType: "ObjectFlags", module: "check", file: "types.go", doc: "Flags classifying object, union, and intersection types; some bits depend on the type kind." },
];

// Evaluates a Go constant expression with Go operator precedence: unary `^`, then
// `<<` and `&`/`&^` (multiplicative), then `|` and `-` (additive).
function evaluate(expression: string, lookup: (name: string) => number | undefined, goType: string): number {
  const tokens = expression.match(/\w+|<<|&\^|[|&^()-]/g) ?? [];
  let index = 0;
  const peek = (): string | undefined => tokens[index];
  const take = (expected?: string): string => {
    const token = tokens[index++];
    if (token === undefined || (expected !== undefined && token !== expected)) {
      throw new Error(`Unexpected token in ${goType} expression: ${expression}`);
    }
    return token;
  };
  const unary = (): number => {
    const token = take();
    if (token === "^") return ~unary() >>> 0;
    if (token === "(") {
      const value = additive();
      take(")");
      return value;
    }
    if (/^\d+$/.test(token)) return Number(token);
    const value = lookup(token.replace(new RegExp(`^${goType}`), ""));
    if (value === undefined) throw new Error(`Unknown ${goType} operand: ${token}`);
    return value;
  };
  const multiplicative = (): number => {
    let value = unary();
    for (let operator = peek(); operator === "<<" || operator === "&" || operator === "&^"; operator = peek()) {
      take();
      const right = unary();
      if (operator === "<<") value = (value << right) >>> 0;
      else if (operator === "&") value = (value & right) >>> 0;
      else value = (value & ~right) >>> 0;
    }
    return value;
  };
  const additive = (): number => {
    let value = multiplicative();
    for (let operator = peek(); operator === "|" || operator === "-"; operator = peek()) {
      take();
      const right = multiplicative();
      value = operator === "|" ? (value | right) >>> 0 : (value - right) >>> 0;
    }
    return value;
  };
  const value = additive();
  if (index !== tokens.length) throw new Error(`Trailing tokens in ${goType} expression: ${expression}`);
  return value;
}

function parseFlags(source: string, goType: string): [string, number][] {
  const declaration = new RegExp(`^${goType}(\\w+)(?:\\s+${goType})?\\s*=\\s*(.+)$`);
  const typeDeclaration = `type ${goType} `;
  const blockStart = source.indexOf("const (", source.indexOf(typeDeclaration));
  const block = source.slice(blockStart, source.indexOf("\n)", blockStart));
  // Collect every declaration first: Go constants may refer to constants declared later.
  const expressions = new Map<string, string>();
  for (const rawLine of block.split("\n")) {
    const match = declaration.exec(rawLine.replace(/\/\/.*$/, "").trim());
    if (match !== null) expressions.set(match[1], match[2]);
  }
  if (expressions.size === 0) throw new Error(`No ${goType} constants found`);
  const values = new Map<string, number>();
  const resolve = (name: string): number | undefined => {
    const known = values.get(name);
    if (known !== undefined) return known;
    const expression = expressions.get(name);
    if (expression === undefined) return undefined;
    const value = evaluate(expression, resolve, goType);
    values.set(name, value);
    return value;
  };
  return [...expressions.keys()].map((name): [string, number] => [name, resolve(name) as number]);
}

function constantName(name: string): string {
  return name
    .replace(/JSDoc/g, "Jsdoc")
    .replace(/([A-Z]+)([A-Z][a-z])/g, "$1_$2")
    .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
    .toUpperCase();
}

function render(module: Module, sets: [FlagSet, [string, number][]][]): string {
  const blocks = sets.map(([set, flags]) => {
    const constants = flags
      .map(([name, value]) => {
        const hex = value.toString(16).padStart(8, "0");
        return `        ${constantName(name)} = 0x${hex.slice(0, 4)}_${hex.slice(4)};`;
      })
      .join("\n");
    return `flags_type! {\n    /// ${set.doc}\n    ${set.goType} {\n${constants}\n    }\n}`;
  });
  return `// Code generated by scripts/generate-ast-flags.ts from TypeScript-Go ${TYPESCRIPT_GO_REVISION}. DO NOT EDIT.

use ${MACRO_IMPORTS[module]};

${blocks.join("\n\n")}
`;
}

function main(): void {
  const { values } = parseArgs({ options: {
    "typescript-go-root": { type: "string" },
    "ast-output": { type: "string", default: "src/ast/flags.rs" },
    "check-output": { type: "string", default: "src/check/flags.rs" },
  } });
  const root = values["typescript-go-root"];
  if (root === undefined) throw new Error("--typescript-go-root is required");
  assertPinnedRepository(root, TYPESCRIPT_GO_REVISION, "TypeScript-Go");
  const sets = FLAG_SETS.map((set): [FlagSet, [string, number][]] => [
    set,
    parseFlags(readFileSync(join(root, PACKAGE_DIRECTORIES[set.module], set.file), "utf8"), set.goType),
  ]);
  const outputs: Record<Module, string> = { ast: values["ast-output"], check: values["check-output"] };
  for (const module of ["ast", "check"] as const) {
    writeFileSync(outputs[module], render(module, sets.filter(([set]) => set.module === module)));
  }
  console.log(JSON.stringify(Object.fromEntries(sets.map(([set, flags]) => [set.goType, flags.length]))));
}

try {
  main();
} catch (error) {
  reportError(error);
}
