# Architecture and compatibility target

## Compatibility baseline

The initial oracle is the official TypeScript 7.0.2 source release at
[`typescript/v7.0.2`](https://github.com/microsoft/typescript-go/tree/typescript/v7.0.2).
TypeScript 7.0 is the native Go port and follows TypeScript 6.0 compiler behavior with the 7.0
defaults and removals. TSRZL targets compiler behavior and command-line builds; its analyzer and
generator APIs are TSRZL extensions because TypeScript 7.0 does not publish a stable compiler API.

“Full compiler support” means conformance across the TypeScript and JavaScript syntax accepted by
the baseline, diagnostics and type checking, JavaScript/declaration/source-map output, command-line
and `tsconfig.json` options, module and package resolution, project references, incremental builds,
and watch builds. Compatibility claims require evidence from the upstream conformance corpus, not
only hand-written smoke cases.

## Pipeline boundaries

```text
CLI / API
  -> project and configuration loading
  -> source loading and module resolution
  -> scanner -> parser -> syntax trees
  -> binder -> symbols and scopes
  -> type checker -> semantic model and diagnostics
  -> generators (immutable input view, owned virtual source outputs)
  -> generated source parse -> bind -> check
  -> analyzers (read-only view of authored and generated sources)
  -> transforms -> JavaScript / declarations / maps
  -> deterministic output writer
```

The compiler core owns parsing, binding, checking, transforms, and emission. Project orchestration
owns configuration, files, references, build state, and output paths. CLI argument parsing is an
adapter over the compiler API. Analyzer and generator implementations are registered by the caller
and cannot mutate core compiler state.

## Ownership rules

- Source text is immutable and shared by reference-counted ownership where sharing is useful.
- Syntax and semantic relationships use stable IDs and explicit spans rather than borrowed
  pointers between phases.
- Diagnostics and generated files own their message/text and identify their source independently
  of parser or checker lifetimes.
- Each compilation owns its phase state. Caches and incremental state have explicit owners and
  invalidation inputs; there is no process-global mutable compiler state.
- An analyzer receives an immutable compilation view and returns diagnostics. A generator receives
  an immutable input view and returns owned source files and diagnostics. Generated sources enter a
  new compilation pass instead of modifying the input snapshot.

## Validation

Every behavior change starts with one focused failing test, then the smallest green implementation,
then refactoring. Differential conformance cases compare TSRZL with the pinned TypeScript 7.0.2
compiler for diagnostics and emitted artifacts. Rust behavior tests use `should_...` names and are
checked with `cntryl-tools validate-tests`.
