# Agent Instructions

## Project Direction

Implement full TypeScript 7 compiler support in Rust, using the official
`microsoft/typescript-go` `typescript/v7.0.2` release as the initial compatibility baseline.
Include source parsing, type checking, JavaScript and declaration emit, project/configuration
handling, module resolution, project references, and compiler CLI behavior.

Provide first-class analyzer and generator hooks as separate pipeline stages. Analyzers consume
read-only compiler views and report owned diagnostics. Generators consume immutable input views
and return owned generated sources; they must not mutate an existing compilation. Keep extension
logic out of scanner, parser, binder, checker, and emitter implementations.

## Test-Driven Changes

- Use red → green → refactor for each behavior change: add one focused failing test, confirm it
  fails for the expected reason, implement the smallest change that makes it pass, then refactor
  with the test still passing.
- Each test should prove one observable behavior. Split tests that assert multiple independent
  behaviors.
- Use behavior names starting with `should_` and state the result, condition, and action, for
  example `should_emit_javascript_given_a_typed_variable_when_compiling`.
- For tests longer than five lines, use explicit `// Arrange`, `// Act`, and `// Assert` sections.
- Run `cntryl-tools validate-tests` from the repository root for test changes. Treat incomplete
  validator analysis as a failed check and review any reported findings.
- Use the pinned TypeScript 7 conformance cases as the compatibility oracle. Add focused Rust
  regression tests for each behavior before changing implementation code.

## Benchmarks

- Use `cntryl-stress` for benchmark workloads; it is configured as a dev dependency.
- Add explicit Cargo bench targets with `harness = false` when a real compiler workload exists.
- Benchmark meaningful compiler operations and use `#[stress]` plus
  `cntryl_stress::stress_main!()` in each target.

## Dependencies

- Implement compiler behavior with the Rust standard library; do not add Cargo dependencies.
- `cntryl-stress` is the only allowed Cargo dependency and must remain a dev dependency used by
  benchmark targets.
- `cntryl-tools` is an external development CLI used to validate tests, not a Cargo dependency.

## Repository Scripts

- Write repository scripts in TypeScript and run them directly with Node.js 24.12 or newer.
- Use erasable TypeScript syntax and Node built-in modules; do not add npm dependencies.
- Keep scripts in `scripts/` and use explicit `.ts` extensions in imports.
- Validate script changes against the pinned corpus and oracle data before replacing generated artifacts.

## Rust Design

- Keep dependencies pointing inward: CLI and project orchestration depend on compiler-core APIs;
  compiler-core must not depend on the CLI or extension implementations.
- Prefer immutable compiler inputs, explicit IDs/spans, and per-compilation ownership over global
  mutable state or long-lived references between compiler phases.
- Keep production modules cohesive and small. Use fallible APIs for user input and reserve panics
  for violated internal invariants.
