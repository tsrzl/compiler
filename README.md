# TSRZL Compiler

An idiomatic Rust implementation of the TypeScript 7 compiler, with compiler-platform extension
points for analyzers and generators. The initial compatibility baseline is the official
`typescript/v7.0.2` release.

## Layout

- `src/` — compiler library and command-line entry point.
- `tests/` — integration tests for externally visible compiler behavior.
- `benches/` — performance workloads using `cntryl-stress`.

The current compiler is an early, partial implementation. It includes immutable source text,
source-file classification, a scanner and parser for a small TypeScript subset, initial type checks,
JavaScript and declaration emit for supported syntax, relative module resolution for supported
imports and named or export-all re-exports, and analyzer and generator hooks. The CLI accepts source
paths or JSONC project files with `files`, `include`, `exclude`, relative `extends`, and a subset of
compiler options. It does not yet provide full TypeScript compatibility. Work continues behavior by
behavior against the pinned TypeScript release. See [AGENTS.md](AGENTS.md) and
[CONTRIBUTING.md](CONTRIBUTING.md) for project conventions.

See [the architecture and compatibility target](docs/architecture.md) for the compiler pipeline
and compatibility definition.
