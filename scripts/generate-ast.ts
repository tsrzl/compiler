#!/usr/bin/env node
// Generates src/ast/nodes.rs from the pinned TypeScript-Go AST schema (`_scripts/ast.json`), using
// TypeScript-Go's own schema resolver so member inheritance, optionality, and child order match.
import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { parseArgs } from "node:util";
import { assertPinnedRepository, TYPESCRIPT_GO_REVISION, reportError } from "./typescript-corpus.ts";

// Structural views of the TypeScript-Go schema API used by this generator.
interface SchemaType {
  kind: string;
  name?: string;
  listKind?: string;
  elementType?: SchemaType;
  baseKind(): string;
}

interface SchemaMember {
  name: string;
  type: SchemaType;
  listKind?: string;
  optional: boolean;
  noFactory: boolean;
  isKindParam(): boolean;
  isChild(): boolean;
}

interface SchemaNode {
  name: string;
  members: SchemaMember[];
  allKinds(): { name: string }[];
}

type KindGuard =
  | { aliasName: string; guardName: string; type: "range"; first: string; last: string }
  | { aliasName: string; guardName: string; type: "enumerated"; members: string[] };

interface SchemaApi {
  nodes(): SchemaNode[];
  kindGuards(): KindGuard[];
  resolveKindMarkerValue(name: string): string;
}

interface Field {
  name: string;
  rustType: string;
  child?: "node" | "list" | "modifiers" | "raw";
  optional: boolean;
}

interface NodeShape {
  name: string;
  kinds: string[];
  fields: Field[];
}

// Members whose values belong to other compiler phases rather than to syntax.
const EXCLUDED_MEMBERS = new Set(["SyntheticExpression.Type"]);

function snakeCase(name: string): string {
  const snake = name
    .replace(/JSDoc/g, "Jsdoc")
    .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
    .replace(/([A-Z])([A-Z][a-z])/g, "$1_$2")
    .toLowerCase();
  return snake === "type" ? "type_node" : snake;
}

function primitiveType(member: SchemaMember, owner: string): string | undefined {
  const name = member.type.name;
  switch (name) {
    case "string":
      return "Box<str>";
    case "bool":
      return "bool";
    case "int":
      return "i32";
    case "TokenFlags":
    case "NodeFlags":
    case "ModifierFlags":
      return name;
    default:
      throw new Error(`Unsupported primitive ${name} on ${owner}.${member.name}`);
  }
}

function fieldFor(member: SchemaMember, owner: string): Field | undefined {
  if (member.noFactory || member.isKindParam() || EXCLUDED_MEMBERS.has(`${owner}.${member.name}`)) return undefined;
  const name = snakeCase(member.name);
  const optional = member.optional;
  const wrap = (rustType: string) => (optional ? `Option<${rustType}>` : rustType);
  const element = member.type.elementType;
  if (member.type.kind === "list" && element?.kind === "primitive" && element.name === "string") {
    return { name, rustType: wrap("Box<[Box<str>]>"), optional };
  }
  const baseKind = member.type.baseKind();
  if (baseKind === "node") return { name, rustType: wrap("NodeId"), child: "node", optional };
  if (baseKind === "list") {
    if (member.listKind === "ModifierList") return { name, rustType: wrap("ModifierList"), child: "modifiers", optional };
    if (member.listKind === "NodeList") return { name, rustType: wrap("NodeList"), child: "list", optional };
    return { name, rustType: wrap("Box<[NodeId]>"), child: "raw", optional };
  }
  if (baseKind === "kind") return { name, rustType: wrap("SyntaxKind"), optional };
  if (baseKind === "primitive") return { name, rustType: wrap(primitiveType(member, owner)!), optional };
  throw new Error(`Unsupported member ${owner}.${member.name} with base kind ${baseKind}`);
}

function shapes(api: SchemaApi): NodeShape[] {
  return api.nodes().map((node) => {
    const fields = node.members.flatMap((member) => fieldFor(member, node.name) ?? []);
    const names = new Set<string>();
    for (const field of fields) {
      if (names.has(field.name)) throw new Error(`Duplicate field ${node.name}.${field.name}`);
      names.add(field.name);
    }
    const kinds = [...new Set(node.allKinds().map((kind) => kind.name))];
    return { name: node.name, kinds, fields };
  });
}

function renderStruct(shape: NodeShape): string {
  const fields = shape.fields.map((field) => `    pub ${field.name}: ${field.rustType},`).join("\n");
  return `/// The data of a \`${shape.name}\` node.\n#[derive(Debug, Clone, PartialEq)]\npub struct ${shape.name} {\n${fields}\n}`;
}

function renderVisit(field: Field): string {
  const method = { node: "visit_node", list: "visit_list", modifiers: "visit_modifiers", raw: "visit_raw" }[field.child!];
  if (field.optional) {
    const argument = field.child === "node" ? `*${field.name}` : field.name;
    return `if let Some(${field.name}) = &data.${field.name}\n                    && visitor.${method}(${argument})\n                {\n                    return true;\n                }`;
  }
  const argument = field.child === "node" ? `data.${field.name}` : `&data.${field.name}`;
  return `if visitor.${method}(${argument}) {\n                    return true;\n                }`;
}

function usedFlagTypes(nodeShapes: NodeShape[]): string[] {
  const used = new Set<string>();
  for (const shape of nodeShapes) {
    for (const field of shape.fields) {
      for (const flagType of ["ModifierFlags", "NodeFlags", "TokenFlags"]) {
        if (field.rustType.includes(flagType)) used.add(flagType);
      }
    }
  }
  return [...used].sort();
}

function render(nodeShapes: NodeShape[]): string {
  const structs = nodeShapes.filter((shape) => shape.fields.length > 0).map(renderStruct).join("\n\n");
  const variants = nodeShapes
    .map((shape) => (shape.fields.length > 0 ? `    ${shape.name}(${shape.name}),` : `    ${shape.name},`))
    .join("\n");
  const acceptArms = nodeShapes
    .map((shape) => {
      const pattern = shape.fields.length > 0 ? `Self::${shape.name}(_)` : `Self::${shape.name}`;
      return `            ${pattern} => matches!(\n                kind,\n                ${shape.kinds.map((kind) => `SyntaxKind::${kind}`).join("\n                    | ")}\n            ),`;
    })
    .join("\n");
  const childArms = nodeShapes
    .filter((shape) => shape.fields.some((field) => field.child !== undefined))
    .map((shape) => {
      const visits = shape.fields.filter((field) => field.child !== undefined).map(renderVisit).join("\n                ");
      return `            Self::${shape.name}(data) => {\n                ${visits}\n                false\n            }`;
    })
    .join("\n");
  const accessors = nodeShapes
    .filter((shape) => shape.fields.length > 0)
    .map((shape) => `    /// Returns the \`${shape.name}\` data, if this node is one.\n    #[must_use]\n    pub const fn as_${snakeCase(shape.name)}(&self) -> Option<&${shape.name}> {\n        match self {\n            Self::${shape.name}(data) => Some(data),\n            _ => None,\n        }\n    }`)
    .join("\n\n");
  return `// Code generated by scripts/generate-ast.ts from TypeScript-Go ${TYPESCRIPT_GO_REVISION}. DO NOT EDIT.
#![allow(missing_docs, clippy::too_many_lines)]

use super::arena::{ModifierList, NodeId, NodeList};
use super::visitor::ChildVisitor;
use super::{${["SyntaxKind", ...usedFlagTypes(nodeShapes)].sort().join(", ")}};

${structs}

/// The kind-specific data of a node.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeData {
${variants}
}

impl NodeData {
    /// Returns whether this data shape is valid for a node of \`kind\`.
    #[must_use]
    pub const fn accepts_kind(&self, kind: SyntaxKind) -> bool {
        match self {
${acceptArms}
        }
    }

    /// Visits child nodes in TypeScript-Go \`ForEachChild\` order, stopping when the visitor
    /// returns \`true\`.
    pub fn visit_children(&self, visitor: &mut impl ChildVisitor) -> bool {
        match self {
${childArms}
            _ => false,
        }
    }

${accessors}
}
`;
}

function constantName(name: string): string {
  return name.replace(/([a-z0-9])([A-Z])/g, "$1_$2").toUpperCase();
}

function renderKindGuards(api: SchemaApi): string {
  const guards = api.kindGuards();
  const guardNames = new Map(guards.map((guard) => [guard.aliasName, snakeCase(guard.guardName)]));
  const methods = guards.map((guard) => {
    const name = snakeCase(guard.guardName);
    let body: string;
    if (guard.type === "range") {
      body = `(Self::${constantName(guard.first)}..=Self::${constantName(guard.last)}).contains(&self)`;
    } else {
      const aliases = guard.members.filter((member) => guardNames.has(member)).map((member) => `self.${guardNames.get(member)}()`);
      const kinds = guard.members.filter((member) => !guardNames.has(member)).map((member) => `Self::${member}`);
      const terms = [...(kinds.length > 0 ? [`matches!(self, ${kinds.join(" | ")})`] : []), ...aliases];
      body = terms.join(" || ");
    }
    return `    /// Returns whether the kind belongs to TypeScript-Go's \`${guard.aliasName}\`.\n    #[must_use]\n    pub fn ${name}(self) -> bool {\n        ${body}\n    }`;
  });
  return `// Code generated by scripts/generate-ast.ts from TypeScript-Go ${TYPESCRIPT_GO_REVISION}. DO NOT EDIT.

use super::SyntaxKind;

impl SyntaxKind {
${methods.join("\n\n")}
}
`;
}

async function main(): Promise<void> {
  const { values } = parseArgs({ options: {
    "typescript-go-root": { type: "string" },
    output: { type: "string", default: "src/ast/nodes.rs" },
    "kind-guards-output": { type: "string", default: "src/ast/kind_guards.rs" },
  } });
  const root = values["typescript-go-root"];
  if (root === undefined) throw new Error("--typescript-go-root is required");
  assertPinnedRepository(root, TYPESCRIPT_GO_REVISION, "TypeScript-Go");
  const schema = (await import(pathToFileURL(join(root, "_scripts/schema.ts")).href)) as { api: SchemaApi };
  const nodeShapes = shapes(schema.api);
  writeFileSync(values.output, render(nodeShapes));
  writeFileSync(values["kind-guards-output"], renderKindGuards(schema.api));
  execFileSync("rustfmt", ["--edition", "2024", values.output, values["kind-guards-output"]]);
  console.log(JSON.stringify({ nodes: nodeShapes.length, output: values.output }));
}

main().catch(reportError);
