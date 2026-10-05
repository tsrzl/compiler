# TypeScript 7 compiler work plan

This plan turns the full-compiler goal into testable coverage gates and work that
can proceed in parallel without changing compiler behavior before its tests are
defined.

## Current checkpoint

The pinned compatibility input is 12,641 TypeScript source fixtures across the
compiler, conformance, project, and transpile suites. Compiler and conformance
fixtures expand to option configurations and 44,689 TypeScript-Go reference
artifacts. The inventory and baseline classifications are documented in
[`typescript-7-test-universe.md`](typescript-7-test-universe.md).

The Rust suite currently has 573 tests: 323 pass and 250 deliberately fail;
none are ignored. The compiler/oracle map links 258 behaviors to upstream cases
or areas. A separate map records 16 extension contracts. Corpus links touch 100
of 110 source-suite/area groups; 10 project groups have no mapped Rust behavior
test yet. A group with one linked example is sampled, not covered. The latest
full test run has all 250 failures mapped by
`scripts/validate-red-test-map.ts`. Keep adding focused red tests to complete
the behavior map before implementation resumes behind the shared contracts.

## Work packages

| Package | Scope | Test artifact | Depends on |
| --- | --- | --- | --- |
| Corpus and oracle accounting | Match each fixture, directive-selected configuration, and diagnostic/emit/declaration artifact to explicit coverage or a documented oracle boundary. The 62 formerly unmatched cases are classified by a pinned-runner audit. | Case-to-test coverage map and reviewed oracle-status list. | Pinned TypeScript and TypeScript-Go revisions. |
| Scanner, parser, and syntax | Lexical forms, grammar productions, recovery, JSX, decorators, source locations, and syntax-only diagnostics. | Focused scanner/parser/syntax tests. | Corpus and oracle accounting. |
| Binding and type checking | Name resolution, type relations, inference, generics, overloads, control flow, contextual typing, and diagnostics. | Focused checker and declaration tests. | Corpus and oracle accounting; stable AST/source-span contract. |
| Emit and declarations | JavaScript output, target transforms, module formats, source maps, declaration output, and emit options. | Byte-for-byte pinned outputs where available. | Corpus and oracle accounting; stable AST and checker query contract. |
| Projects, modules, and CLI | Configuration parsing, file discovery, module resolution, package exports, project references, diagnostics, output paths, and transpile behavior. | Project fixtures plus focused CLI and resolver tests. | Corpus and oracle accounting; stable compiler-core entry points. |
| Analyzer and generator hooks | Immutable analyzer views, owned diagnostics, generator inputs/outputs, pipeline order, and generated-source checking. | Project-specific lifecycle tests; upstream TypeScript has no matching extension-hook suite. | Stable compiler phase/view contracts. |

Each test author owns a disjoint integration-test file while the coverage map is
under construction. A single integrator owns the shared corpus map, fixture
classification, and repository-wide validation. Tests must use one observable
behavior, `should_...` names, Arrange/Act/Assert sections, and the pinned oracle.

## Parallel coverage lanes

Run these as separate test-design and audit tracks. Each lane owns its named
test file and source-area subset; the integrator alone edits the shared map and
decides cross-area or oracle classifications.

| Lane | Test and audit scope | Owner boundary | Current state / next gap |
| --- | --- | --- | --- |
| Front end and syntax | Scanner, parser productions and recovery, JSX, decorators, import attributes/defer, source locations. | `scanner.rs`, `syntax_tree.rs`, `parser_diagnostics.rs`, or a new front-end test file; one upstream area per task. | Red tests include string index signature, ES2018 async generator, tagged-template parsing, syntax recovery, valid `\\u{...}` identifier escapes, `import.meta`, `new.target`, first-line hashbangs, TS1161 for an unterminated regex at EOF, AST representation of `using` and `await using` declarations, TS1206 for decorators on variables, legacy experimental class-decorator parsing, arbitrary string-literal export names, namespace declaration parsing, TS2414 for the reserved class name `any`, tuple type syntax, static class initialization blocks, abstract class declarations, and abstract method signatures. Template literal and indexed access types also have focused parser tests. The conformance lane added auto-accessor lowering for an ES2022 target. Continue grammar and diagnostic variants. |
| Type system | Binding, assignability, inference, generics, overloads, control flow, declarations. | `type_checking.rs`, `advanced_type_system.rs`, `declaration_emit.rs`; partition by type-system area. | Union interface heritage (TS2312), incompatible overload implementation (TS2394), missing constructor implementation (TS2390), missing method implementation (TS2391), mismatched overload implementation name (TS2389), optional parameter read (TS18048), derived-to-base assignability, optional-chain `undefined` assignability, unassigned-variable reads (TS2454), strict property initialization (TS2564), string-index property mismatch (TS2411), duplicate block-scoped declarations (TS2451), invalid boolean enum initializers (TS18033), invalid BigInt constructor arguments (TS2345), invalid `keyof` key assignment (TS2322), and conditional `infer` syntax are mapped. Map uncovered type/control-flow fixtures and directive-selected configurations. |
| Emit | Target lowering, module formats, JSX/decorator transforms, source maps, declarations and emit options. | `javascript_emit.rs`, `declaration_emit.rs`, or a new emit file; assign distinct output behaviors. | Pinned computed-class source-map mappings, type-predicate declaration emission, ES2015 exponentiation lowering, object-spread lowering, ES2015 nullish-assignment lowering, const-enum member inlining, erasure of the TypeScript `override` modifier, and preserving static initialization blocks for ES2022 are mapped. Target transforms, declaration output, and option matrices remain thin. |
| Projects and modules | Config parsing, file discovery, package resolution, project references, output layout, CLI and transpile. | `module_resolution.rs`, `cli_compilation.rs`, `project_feature_coverage.rs`, `project_transpile_coverage.rs`, `compiler_options_coverage.rs`; split by resolver, project, and CLI corpus. | All 175 project and 22 transpile fixtures are classified, and all 316 project runner JSONs are linked. One runner case has missing inputs. Existing project/transpile behavior tests, seventeen option tests across thirteen options, TS5042 project/source-file validation, `typesVersions`, `noEmitOnError`, project `noEmit`, `allowJs` default discovery, JSON module resolution, scoped package `types` resolution, and `rootDirs` cross-root resolution are mapped red; most fixtures/configurations still need focused tests and direct oracle checks. `allowUnreachableCode=false` now has a focused TS7027 test. The `projects/non-relative` group now has a focused TypeScript 7 `paths`-mapping resolution test. Default `outDir` exclusion from implicit root discovery is confirmed green against the pinned CLI. `declarationDir` alone is covered, while its combination with `outDir` now has a focused red test for preserving the nested declaration path. `DeclareExportAdded` now covers emitted triple-slash preservation. `reference-1` now has a transitive triple-slash resolution test, reducing the source-area groups without samples from 23 to 22. `declarations_ImportedInPrivate` now has a pinned-output test showing exported namespaces are missing from declaration output. The `decoratorMetadata` project fixture is now linked to the existing TS5108 test for its removed ES5 target; TS-Go 7.0.2 confirmed that diagnostic on the exact project config. `NestedLocalModule-SimpleCase` now verifies that TypeScript 7 parses nested import-equals syntax before reporting TS1147 semantically. The `outputdir_simple` discrepancy shares that unresolved reference-discovery behavior, so it is tracked with the existing test instead of duplicating it as an output-layout test. `MultipleLevels/B/B.ts` now exercises nested import-equals dependencies. Imported-function declaration coverage now distinguishes imported return-type preservation from the existing imported-class case; CascadingImports has a focused equivalent relative-import declaration test. The original `CircularReferencing` runner case still fails before cycle traversal because of import-equals and ambient-module gaps; a reduced relative ES-module cycle now has a red TS7023 test. `CircularReferencing-2` also has a red CommonJS emit test for eliding its unused cycle-closing import. Apostrophe path handling succeeds in both compilers; the upstream fixture’s remaining failures concern qualified class extends and reference discovery. The `declarations_MultipleTimesImport` group now has a red declaration test requiring repeated references to the same module to produce one import. `declarations_IndirectImport` now has a red equivalent-source declaration test for preserving transitive imported types; the original fixture’s remaining syntax/resolution gap is import-equals support. `declarations_MultipleTimesMultipleImport` now has a separate red declaration test requiring the inferred origin module to replace its intermediate import. The `declarations_GlobalImport` candidate is masked by unresolved bare-module and import-equals handling, `declarations_ExportNamespace` by unsupported namespace syntax, and `RelativePaths` by import-equals parsing plus project-runner-specific root resolution; no isolated CLI behavior test is established yet. `moduleMergeOrder` now has parser-level namespace coverage; direct TS-Go 7.0.2 `--noEmit` runs accept the original source pair in both runner input orders, while the project’s merged semantic/output ordering remains untested. |
| Extension contracts | Analyzer ordering/views/diagnostics and generator inputs/outputs, deduplication, generated-source checking, and error behavior. | `analyzer_hooks.rs`, `generator_hooks.rs`, and `extension_lifecycle.rs`; one file per author. | Sixteen recorded contracts include twelve lifecycle tests: nine green and three red path/output-path cases. Semantic views, source-aware diagnostics, generated-file identity, and full ownership APIs remain contract work. |
| Oracle and coverage accounting | Configuration variants, expected artifact kinds, runner skips, unsupported options, and missing references. | Inventory, audit and progress report; no production or test-file edits. | Integrator-owned. The missing-reference audit resolved all 62 cases: 51 resolved-option skips and 11 cases with empty outputs in 13 configurations. Keep all 45 explicit skips, 559 unsupported-option source cases and 11 empty-output cases distinct from Rust coverage. |

The coverage target is a complete *classification* of the corpus and its
selected configurations, plus focused tests for each distinct observable
behavior. It is not one Rust test per source file: related fixtures can map to
one behavior test only when the observable contract is the same, while a single
fixture with distinct diagnostics, JavaScript, declaration, or resolution
outputs may need several tests.

## Unsampled project groups

All ten groups without a linked Rust test are in the project suite. Several
cannot yet produce a trustworthy focused red from the direct TS-Go CLI because
the project runner has different resolution context or the inputs fail earlier
on import-equals syntax. Keep those groups visible and add tests only when the
expected behavior can be established independently.

| Group | Current evidence | Next action |
| --- | --- | --- |
| `NestedLocalModule-WithRecursiveTypecheck` | TS-Go CLI reports TS1147 and TS2307; TSRZL stops earlier on namespace/import-equals syntax. | Recover the project-runner baseline before testing recursive checking. |
| `RelativePaths` | Runner selects `app.ts` with no options or reference baseline; TS-Go 7 removed Classic resolution, so direct CLI cannot establish which `b.ts` the bare imports select. | Recover a TS7-compatible runner oracle for its distinguishing resolver behavior. |
| `VisibilityOfCrosssModuleTypeUsage` | TS-Go reports TS1202, missing Node `fs` types, and TS2307 before the cross-module visibility behavior; TSRZL stops earlier on import-equals parsing. | Recover a runner baseline that reaches the intended check. |
| `ext-int-ext` | CLI reports TS1202/TS2307 before the mixed namespace/import behavior. | Recover runner options or keep as a known combined gap. |
| `privacyCheck-ImportInParent` | TS-Go CLI reports TS1147/TS2307 before privacy checks. | Recover the project-runner baseline. |
| `privacyCheck-IndirectReference` | TS-Go CLI stops at TS2307 for a bare import before declaration privacy; TSRZL would stop earlier on import-equals parsing. | Recover the project-runner resolution context. |
| `privacyCheck-InsideModule` | No TS-Go project baseline is present; the fixture uses nested namespaces, import-equals, and bare modules, which mask the privacy behavior. | Obtain the pinned runner oracle or keep this as an explicit gap. |
| `privacyCheck-SimpleReference` | TS-Go CLI reports TS2307 for bare module specifiers before privacy behavior. | Recover the project-runner resolution context. |
| `relative-nested` | TS-Go accepts the import chain; TSRZL stops at import-equals parsing. | Test after import-equals support or use a runner-equivalent oracle. |
| `relative-nested-ref` | TS-Go defaults report TS1202/TS7010; CommonJS adaptation reaches a known ambient-module diagnostic. | Avoid duplicating the existing ambient-reference test. |

## Sequencing and parallel work

1. **Finish coverage in parallel.** Run the six lanes above concurrently on
   non-overlapping corpus areas. Each author records the exact fixture,
   directives/configuration, oracle artifact, expected observation, and current
   red reason. The integrator reviews and reconciles map updates serially.
2. **Freeze shared compiler contracts.** Before implementation is delegated,
   settle source-file identity and spans, AST node ownership, symbol/type IDs,
   diagnostic ownership, compiler-option representation, module-graph and emit
   result ownership, and immutable analyzer/generator views. Keep changes to
   those shared contracts with one owner.
3. **Implement behind stable module boundaries.** The scanner/parser can work
   against the frozen AST contract. Project/configuration and module-graph work
   can proceed against the frozen core API. Binder/checker work owns semantic
   IDs and queries; emit and declaration work consumes those queries and owns
   output formatting. Analyzer/generator work consumes immutable views and
   returns owned outputs through the compiler pipeline. Shared AST, diagnostics,
   options, and orchestration changes stay with the integration owner.
4. **Keep dependent integration serial.** The critical path is parse/bind and
   module graph → semantic checking → JavaScript/declaration emit → project and
   extension-pipeline qualification. Parallelize only work that respects those
   inputs and outputs; integrate and qualify each vertical slice before
   downstream slices rely on it.
5. **Use red → green → refactor per behavior.** Confirm the new test fails for
   the intended missing behavior, implement the smallest change, then run its
   suite, `cntryl-tools validate-tests`, formatting, and relevant compatibility
   cases. Keep production dependencies at zero except the allowed
   `cntryl-stress` benchmark dev dependency.

Tests can be authored concurrently now because their files and corpus areas are
separate. Compiler implementation can be safely parallelized after the AST,
type-query, diagnostic, and pipeline contracts are stable; before that point,
independent feature work would collide in the parser, AST, and compiler driver.

[`compiler-contract-gaps.md`](compiler-contract-gaps.md) records the current API
evidence and proposed shared contracts. Regenerate
[`typescript-7-coverage-progress.tsv`](typescript-7-coverage-progress.tsv) with
`node scripts/report-typescript-coverage.ts --typescript-go-root /path/to/typescript-go`
to validate test links and inspect
all source-area work queues. The scripts are TypeScript and use only Node
built-in modules.
Run `node scripts/validate-project-transpile-audit.ts --typescript-root /path/to/TypeScript`
to check the separate project/transpile audit against all pinned fixtures and
runner JSONs.
Run `node scripts/generate-typescript-behavior-backlog.ts` to regenerate the
12,641-row fixture map with exact path references, area samples, and unmapped
fixtures kept distinct.

## Exit gate for starting compiler implementation

- Every compiler and conformance fixture has an explicit map status for each
  selected option configuration and applicable oracle output kind; map statuses
  link to focused Rust tests rather than implying one test per fixture.
- The 175 project cases, 22 transpile cases, CLI/configuration behavior, and
  analyzer/generator contracts have explicit tests or documented boundaries.
- All unclassified reference gaps and TypeScript-Go option exclusions have
  been reviewed and assigned a deliberate compatibility policy.
- Every intentionally failing Rust behavior test is present in a compiler,
  option, project, or extension map; validate this from a full
  `cargo test --no-fail-fast` log with `scripts/validate-red-test-map.ts`.
- `cntryl-tools validate-tests` reports complete analysis and no naming,
  structure, or multi-behavior violations.
