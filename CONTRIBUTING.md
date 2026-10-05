# Contributing

## Test workflow

For each behavior change, follow red → green → refactor:

1. Write one focused test for one externally observable behavior.
2. Run it and confirm the expected failure.
3. Implement the smallest change that makes it pass.
4. Refactor while keeping the behavior test green.
5. Run `cntryl-tools validate-tests` from the repository root.

Name tests `should_<result>_given_<condition>_when_<action>`, using concrete behavior and inputs.
Split independent behaviors into separate tests. For tests longer than five lines, mark the
`// Arrange`, `// Act`, and `// Assert` sections.

## Repository scripts

Write scripts in TypeScript using Node built-in modules and erasable syntax.
Run them directly with Node.js 24.12 or newer; no npm dependencies are allowed.
See [`scripts/README.md`](scripts/README.md) for inventory, oracle-audit, and
coverage-report commands. Validate generated data against the pinned inputs.

## Benchmarks

Use `cntryl-stress` for performance workloads. Each benchmark target should be explicitly declared
in `Cargo.toml` with `harness = false`, define workloads with `#[stress]`, and end with
`cntryl_stress::stress_main!()`. Add workload benchmarks alongside the compiler operation they
measure; do not add synthetic placeholder workloads.
