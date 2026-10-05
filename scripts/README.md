# Repository scripts

Run the TypeScript scripts directly with Node.js 24.12 or newer. They use only
Node built-in modules, with no package installation. The local `package.json`
selects ES modules. Scripts use erasable TypeScript syntax supported by
[Node's native type stripping](https://nodejs.org/api/typescript.html); running
them validates behavior but does not perform TypeScript type checking.

## Corpus inventory

From the repository root, regenerate all source cases and matching oracle
artifacts from the exact pinned revisions:

```sh
node scripts/generate-typescript-case-inventory.ts \
  --typescript-root /path/to/TypeScript \
  --typescript-go-root /path/to/typescript-go
```

The inventory includes compiler, conformance, projects, and transpile fixtures.
Both checkouts must have clean tracked files and no untracked files; the Go
submodule mount and ignored runner outputs are excluded from this check.
Reviewed missing-reference outcomes are read from
`docs/typescript-7-reference-audit.tsv`. The generator rejects different source
revisions, duplicate fixture basenames, unknown audited cases, and inconsistent
audit outcomes. TSV fields preserve quoted tabs, newlines, and double quotes.

## Missing-reference audit

The pinned TypeScript-Go compiler runner requires the provided corpus at its
`_submodules/TypeScript` location. The script checks that the runner's case
directory resolves to the provided corpus and does not create or change mounts.
Go must be available at the version required by the oracle checkout's `go.mod`.

```sh
node scripts/audit-typescript-reference-gaps.ts \
  --typescript-root /path/to/TypeScript \
  --typescript-go-root /path/to/typescript-go \
  --run-oracle \
  --runner-results /tmp/tsrzl-reference-results.jsonl
```

The selected cases and known configurations come from the inventory and the
canonical previous audit, independently of the destination file. Use
`--previous-audit PATH` to select another reviewed audit explicitly.
The script requires successful package/case completion, an explicit recognized
reason for every skip, and all diagnostic, output, source-map and structural
checks for every successful case. It rejects incomplete selections before
writing an audit, including a run missing one known configuration of a present
source case. To inspect an existing trusted runner log, omit `--run-oracle`.
An oracle skip or empty output classification does not establish Rust coverage.

## Coverage progress

```sh
node scripts/report-typescript-coverage.ts \
  --typescript-go-root /path/to/typescript-go
```

This validates the behavior map against actual Rust test functions and pinned
corpus paths and artifacts, validates the extension-contract and compiler
option maps, and writes
`docs/typescript-7-coverage-progress.tsv`. Its source-area counts distinguish
focused samples from areas with no linked tests. Linking one sample to an area
does not classify all fixtures or prove that area's behavior is implemented.
Go source and CLI baseline links must resolve inside the clean pinned oracle
checkout and exist there.

To verify that every intentionally red Rust behavior test has an entry in one
of the compiler/oracle, project-option, project-audit, or extension maps, capture
a full Cargo test run and validate its failures:

```sh
cargo test --no-fail-fast > /tmp/tsrzl-cargo-tests.log 2>&1
node scripts/validate-red-test-map.ts --test-results /tmp/tsrzl-cargo-tests.log
```

The validator reports any failing behavior test that is absent from all maps.
The Cargo run may exit nonzero while mapped red-to-green cases remain.

## Project and transpile audit

```sh
node scripts/validate-project-transpile-audit.ts \
  --typescript-root /path/to/TypeScript
```

This read-only validator compares the audit with both the inventory and pinned
source tree. It checks every project/transpile fixture, all project runner JSON
links, explicit missing-input gaps, and mapped Rust test names and source
references. It reports counts and exits nonzero for invalid data. Pending rows
remain pending; validation does not make their compiler behavior pass.

## Per-fixture behavior map

```sh
node scripts/generate-typescript-behavior-backlog.ts
```

This writes one row per inventoried source fixture to
`docs/typescript-7-rust-behavior-backlog.tsv`, carrying its directives, option
configurations, reference artifacts, oracle status, and Rust test links.
Project/transpile rows also contain the runner configuration and oracle
expectation. A fixture referenced by path is distinguished from one that only
shares an area sample; neither link proves all of its configurations or
artifacts are covered.
