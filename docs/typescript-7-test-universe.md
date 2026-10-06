# TypeScript 7 compatibility test universe

## Pinned oracle and source corpus

The compatibility baseline is `microsoft/typescript-go` tag `typescript/v7.0.2`
at commit `2bd066d87f5bafd315be9f40889d0a60b9e58e0b`. Its TypeScript test
submodule is pinned at `4d4f005c8541e0255a9d8791205fdce326e462bc`.

The pinned TypeScript checkout contains these compiler-relevant source fixture
inventories. TypeScript-Go's compiler test runner enumerates both `.ts` and
`.tsx` files for the compiler and conformance suites:

| Suite | `.ts` files | `.tsx` files | Total source cases |
| --- | ---: | ---: | ---: |
| Compiler regression cases | 6,419 | 118 | 6,537 |
| Language conformance cases | 5,695 | 212 | 5,907 |
| Project cases | 175 | 0 | 175 |
| Transpile cases | 22 | 0 | 22 |

That is 12,444 compiler and conformance source cases, plus 197 project and
transpile source files. The conformance directory table below counts `.ts`
fixtures; its 5,695 `.ts` cases have another 212 `.tsx` fixtures.

The conformance source cases break down by upstream directory as follows:

| TypeScript conformance area | Cases |
| --- | ---: |
| Root cases | 2 |
| Symbols | 8 |
| additionalChecks | 1 |
| ambient | 22 |
| async | 185 |
| asyncGenerators | 3 |
| classes | 466 |
| constEnums | 9 |
| controlFlow | 56 |
| declarationEmit | 23 |
| decorators | 88 |
| directives | 5 |
| dynamicImport | 71 |
| emitter | 13 |
| enums | 14 |
| es2016 | 1 |
| es2017 | 12 |
| es2018 | 4 |
| es2019 | 13 |
| es2020 | 15 |
| es2021 | 12 |
| es2022 | 7 |
| es2023 | 2 |
| es2024 | 3 |
| es2025 | 4 |
| es5 | 1 |
| es6 | 1,045 |
| es7 | 45 |
| esDecorators | 110 |
| esnext | 2 |
| expressions | 376 |
| externalModules | 227 |
| functions | 18 |
| generators | 15 |
| importAssertion | 5 |
| importAttributes | 11 |
| importDefer | 17 |
| interfaces | 66 |
| internalModules | 76 |
| jsdoc | 341 |
| jsx | 4 |
| moduleResolution | 51 |
| node | 94 |
| nonjsExtensions | 5 |
| override | 31 |
| parser | 819 |
| pedantic | 2 |
| references | 15 |
| salsa | 191 |
| scanner | 35 |
| statements | 203 |
| types | 842 |
| typings | 9 |

These counts are the input inventory, before the TypeScript-Go runner's skip
rules and per-option variants. The regression and conformance suites are the
primary language and compiler behavior universe. Project and transpile cases
cover separate orchestration and emit paths. Fourslash cases exercise editor
and language-service behavior and are outside the compiler target in
`AGENTS.md`.

The pinned TypeScript-Go checkout has 49,316 reference-baseline artifacts
across all of its suites. Its submodule compiler and conformance folders hold
21,874 and 22,815 reference files respectively. These are output artifacts,
not test counts, and include diagnostics, JavaScript, type and symbol dumps,
source maps, and diff files. The Go runner also expands test-file option
directives into configurations, so one source case can have several expected
outputs. The total of 49,316 also includes editor, configuration, watch, and
other results.

`docs/typescript-7-case-inventory.tsv` indexes all 12,641 source cases. Each
row records the suite, upstream area, fixture path, `// @` directives,
available configuration variants, and matching TypeScript-Go reference files.
The inventory currently maps 6,097 compiler cases to 21,874 reference files
and 5,732 conformance cases to 22,815 reference files. It also records 440
compiler and 175 conformance fixtures with no matching reference artifact.
Within those, 45 fixtures are on the runner's explicit skip list and 559 are
classified as unsupported options under its rules. This includes 51 cases whose
skip reason was confirmed by executing the pinned runner after it read their
configuration files. Eleven other source cases passed in 13 configurations with
intentionally empty reference outputs. The 62 formerly unmatched source cases
are now recorded in [`typescript-7-reference-audit.tsv`](typescript-7-reference-audit.tsv)
as 64 configuration outcomes: 51 skips and 13 passes. None remain unclassified
in this missing-reference set. These are oracle statuses; Rust behavior coverage
for those cases is still pending. The 175 project and 22 transpile source files
are indexed without a TypeScript-Go submodule reference suite.

Regenerate the inventory from the exact pinned checkouts with:

```sh
node scripts/generate-typescript-case-inventory.ts \
  --typescript-root /path/to/TypeScript \
  --typescript-go-root /path/to/typescript-go
```

All repository scripts are dependency-free TypeScript executed directly by
Node.js 24.12 or newer. See [`../scripts/README.md`](../scripts/README.md) for
reproducing the pinned runner audit and the complete source-area coverage report.

The pinned Go runner's generated submodule baselines cover compiler and
conformance cases, but not the TypeScript `projects` and `transpile` case
directories. Those areas need dedicated project and transpile expectations in
the Rust compatibility suite rather than being counted as already covered by
the Go baselines.

[`typescript-7-project-transpile-audit.tsv`](typescript-7-project-transpile-audit.tsv)
classifies every project/transpile source fixture and links all 316 project
runner JSON cases. One runner JSON references an absent `InvalidRootFile`
directory; that missing-fixture gap has its own row. Six focused behaviors have
direct pinned-CLI oracle checks and red Rust tests. The other fixture rows
record pending tests and oracle checks explicitly.

## Rust test contract

- Add an executable Rust regression test before changing compiler behavior.
- Each test proves one observable behavior, names the result, condition, and
  action with `should_...`, and uses Arrange / Act / Assert sections.
- Derive expected diagnostics, emitted JavaScript, declarations, and CLI or
  project behavior from the pinned oracle. Keep the expected output in the
  test or a focused golden fixture.
- Keep compiler-language cases in the existing integration-test modules by
  behavior area. Add project, configuration, module-resolution, and CLI cases
  in their respective suites. Analyzer and generator lifecycle contracts need
  focused project-specific tests because the TypeScript corpus has no matching
  extension-hook contract.
- Treat upstream case files as the coverage source, not as a requirement to
  create one giant test that loops over unrelated behaviors. Convert cases
  into focused Rust behavior tests and record which upstream fixtures they
  represent.

## Test-first completion gate

The focused red tests are initial examples, not the complete red suite.
Compiler feature implementation remains paused while the compatibility tests
are mapped. The map must account for every compiler and conformance fixture,
the option configurations selected by its directives, and each applicable
oracle artifact. A source case that has separate observable diagnostic, emit,
declaration, or resolution behavior should produce separate focused Rust tests
for those behaviors.

The test map also needs explicit Rust expectations for the 175 project source
files, 22 transpile cases, CLI/configuration behavior, and the analyzer and
generator contracts. The last two have no direct upstream language-corpus
equivalent. The case inventory is a source and oracle-output map, not yet the
full set of Rust behavior tests. Resume implementation only after the coverage
map identifies the failing behavior tests and records any intentionally
unsupported oracle cases.

## Rust behavior coverage status

The Rust suite currently has 798 behavior tests: 346 pass, 452 deliberately
fail, and none are ignored. Coverage remains test-first: these red tests record
compiler, project, and extension behavior that still needs implementation. The
304 tests that preceded this test-first pass are grouped as follows:

| Rust integration suite | Tests before this test-first pass |
| --- | ---: |
| Analyzer hooks | 2 |
| CLI compilation | 22 |
| Declaration emit | 29 |
| Generator hooks | 2 |
| JavaScript emit | 89 |
| Module resolution | 20 |
| Multiple-source compilation | 4 |
| Parser diagnostics | 8 |
| Source files | 1 |
| Source-text locations | 1 |
| Syntax declarations | 1 |
| Syntax tree | 8 |
| Type checking | 116 |

There is now a dedicated scanner behavior suite. The current tests are a
focused subset, not a port of the upstream fixture universe. The initial forty-
one red tests cover Unicode escapes in identifiers and keywords, invalid numeric
separators, line breaks in strings, regular expression literals, private
identifiers, decorators, JSX, project-reference builds, `satisfies` checking
and erasure, `as const`, generic function calls, overload resolution, indexed
access and tuple types, type narrowing, template interpolation name checking,
optional property and call access, async and generator functions, array and
object destructuring, namespaces, dynamic imports, conditional and mapped types,
template literal types, constrained generic inference, conditional `infer`,
`keyof`, readonly mapped properties, discriminated unions, intersection types,
unknown narrowing, import attributes, source maps, and parent-directory
`node_modules` resolution. The additional eighteen red tests cover interface
heritage and generic instantiation, truthiness narrowing and overload failure,
generic and readonly declaration output, path mappings, package entry and
`@types` resolution, Node16 exports, `--noEmit`, output-directory layout,
template substitution emit, deferred namespace imports, JSX preserve mode,
standard decorator lowering, optional-chain target lowering, and static import
attribute emit. A separate green parser test records existing support for
deferred imports.
Eight additional binding tests cover ambient declarations/modules, computed
symbol properties, export assignments, type references, JSDoc checking and
declarations, and interface merging. Six project/transpile tests cover removed
options, declaration directories, root-directory errors, inline source maps,
declaration maps, and cross-file declaration inference. All fourteen fail for
the expected missing behavior. Twelve extension-lifecycle tests add nine green
contracts and three red path/output-path contracts; their separate map is
[`typescript-7-extension-contracts.tsv`](typescript-7-extension-contracts.tsv).
The immutable semantic views and shared identity APIs still needed for full
extension support are described in [`compiler-contract-gaps.md`](compiler-contract-gaps.md).
Twenty-three compiler-option tests cover fifteen options, including both the
project configuration and CLI forms of `noImplicitAny`, plus strict-null
diagnostics for property and nullish-coalescing reads. Each remains red for
its mapped diagnostic. Other cases require pinned diagnostics for
implicit `any`, unused locals and parameters, missing override modifiers,
switch fallthrough, exact optional properties, unchecked indexed access,
unknown catch bindings, and incomplete return paths. Their configurations and
oracle artifacts are recorded in
[`typescript-7-option-coverage.tsv`](typescript-7-option-coverage.tsv).
Additional red tests now cover triple-slash reference diagnostics, `const enum`
and string index-signature parsing, async generator and tagged-template parsing,
interface extensions of union types, overload implementation compatibility,
derived-to-base assignment, versioned package type resolution, `noEmitOnError`,
syntax error recovery, the CLI `--project`/source-file conflict, required
return-value diagnostics, type-only star re-exports, local type queries, and
import-equals aliases in declaration output.
The TypeScript 7 oracle reports TS1360 for a string checked against `number`
with `satisfies`; the current parser treats the new syntax as unresolved names
or parse errors.

At this checkpoint, the compiler/oracle map has 483 behavior entries, touching
all 110 source-suite/area groups in the inventory. Each mapped test samples its
group rather than exhaustively covering it. The largest remaining backlogs
include compiler regressions, JSDoc, external modules, statements,
Salsa/incremental behavior, and project/transpile configurations. See
[`typescript-7-work-plan.md`](typescript-7-work-plan.md) for parallel work
ownership and exit gates.
The red-test map validator confirms that all 452 deliberate Rust failures have
links to the compiler/oracle, option, project, or extension maps.

[`typescript-7-rust-behavior-backlog.tsv`](typescript-7-rust-behavior-backlog.tsv)
adds one row for every source fixture. It currently records 353 fixtures
referenced by exact path, 504 with only an area sample, and 11,784 with no
mapped behavior-test reference. Each row also carries its recorded option
directives/configurations, reference-artifact names and kinds, and oracle
status. Project/transpile rows also carry the runner configuration and oracle
expectation. Exact path links identify the sample's upstream source; they do
not mean the Rust
test executed that fixture or covered every output/configuration.

| New Rust behavior test | Upstream TypeScript case area | Expected TypeScript 7 behavior |
| --- | --- | --- |
| `should_parse_const_enum_declaration_given_const_modifier_when_building_syntax_tree` | `conformance/constEnums/constEnum1.ts` | Accept a `const enum` declaration without syntax diagnostics. |
| `should_erase_satisfies_operator_given_numeric_initializer_when_emitting_javascript` | `conformance/expressions/typeSatisfaction` | Erase the operator and emit the numeric expression. |
| `should_report_incompatible_satisfies_type_given_string_expression_when_checking_types` | `conformance/expressions/typeSatisfaction/typeSatisfaction_errorLocations1.ts` | Report TS1360 for `string` against `number`. |
| `should_accept_const_assertion_given_string_literal_when_checking_types` | `conformance/expressions/typeSatisfaction/typeSatisfaction_asConstArrays.ts` | Accept `as const`. |
| `should_check_generic_function_call_given_identity_function_when_checking_types` | `conformance/types/typeRelationships/typeInference` | Infer the generic call result and accept its `string` assignment. |
| `should_report_unknown_identifier_given_template_interpolation_when_checking_types` | `conformance/es6/templates` | Report TS2304 for an unknown interpolation name. |
| `should_preserve_optional_property_access_given_optional_chain_when_emitting_javascript` | `conformance/expressions/optionalChaining/optionalChainingInference.ts` | Accept and preserve optional property access for the default target. |
| `should_report_possibly_undefined_given_optional_parameter_read_when_checking_types` | `compiler/optionalParamArgsTest.ts` | Report TS18048 for an optional parameter read without narrowing. |
| `should_accept_unicode_escape_given_identifier_when_compiling` | `conformance/scanner/ecmascript5/scannerS7.6_A4.2_T1.ts` | Decode Unicode escapes in identifiers. |
| `should_build_referenced_project_given_project_reference_when_running_compiler_cli` | `typescript-go/internal/project/projectreferencesprogram_test.go` | Build a referenced library before the application project. |
| `should_build_transitive_project_references_given_three_level_graph_when_running_compiler_cli` | `typescript-go/internal/project/projectreferencesprogram_test.go` | Build core and middle dependencies before the app and emit JavaScript for all three projects. |
| `should_build_non_composite_project_given_project_build_mode_when_running_compiler_cli` | `typescript-go/internal/execute/tsctests/tscbuild_test.go` | Build a standalone project in build mode when it is not composite and has no references. |
| `should_schedule_only_changed_leaf_project_given_source_change_when_running_dry_build_cli` | `typescript-go/internal/execute/tsctests/tscbuild_test.go` | Schedule only the changed app project while reporting its core and middle dependencies as up to date. |
| `should_schedule_changed_project_given_tsconfig_change_when_running_dry_build_cli` | `typescript-go/internal/execute/tsctests/tscbuild_test.go` | Schedule an app project for rebuild after its target option changes while its dependencies remain current. |
| `should_rebuild_projects_given_stale_build_info_version_when_running_compiler_cli` | `typescript-go/internal/execute/tsctests/tscbuild_test.go` | Rebuild all referenced projects when their build-info compiler version differs from the current version. |
| `should_schedule_project_given_extended_tsconfig_change_when_running_dry_build_cli` | `typescript-go/internal/execute/tsctests/tscbuild_test.go` | Rebuild a project when a compiler option changes in its extended config while dependencies remain up to date. |
| `should_rebuild_incremental_project_given_corrupt_build_info_when_running_compiler_cli` | `typescript-go/internal/execute/tsctests/tscbuild_test.go` | Recover from malformed build information, emit the source, and rewrite valid versioned metadata. |
| `should_skip_dependents_given_upstream_error_when_stop_build_on_errors_is_enabled` | `typescript-go/internal/execute/tsctests/tscbuild_test.go` | Build the failing upstream project and skip the middle and app projects when stop-build-on-errors is enabled. |
| `should_build_dependents_given_upstream_error_when_stop_build_on_errors_is_disabled` | `typescript-go/internal/execute/tsctests/tscbuild_test.go` | Continue building the middle and app projects by default despite a core diagnostic. |
| `should_report_project_reference_cycle_given_cyclic_solution_build_when_running_compiler_cli` | `typescript-go/internal/execute/tsctests/tscbuild_test.go` | Report TS6202 and stop a solution build when project references form an unmarked cycle. |
| `should_build_explicitly_circular_project_reference_given_solution_build_when_running_compiler_cli` | `typescript-go/internal/execute/tsctests/tscbuild_test.go` | Build both projects when a cyclic project reference is explicitly marked `circular`. |
| `should_rebuild_dependent_declarations_given_dependency_type_changes_when_running_compiler_cli` | `typescript-go/internal/project/projectreferencesprogram_test.go` | Rebuild the dependent projects so an app declaration reflects a changed core export type. |
| `should_skip_project_outputs_given_dry_build_when_running_compiler_cli` | `typescript-go/internal/execute/tsctests/tscbuild_test.go` | List pending project builds without writing any project outputs. |
| `should_remove_outputs_given_clean_build_of_referenced_projects_when_running_compiler_cli` | `typescript-go/internal/execute/tsctests/tscbuild_test.go` | Remove JavaScript, declaration, and build-info outputs from every referenced project. |
| `should_allow_repeated_project_clean_given_existing_clean_state_when_running_compiler_cli` | `typescript-go/internal/execute/tsctests/tscbuild_test.go` | Allow a second clean build to succeed after the first clean removed project outputs. |
| `should_rebuild_all_referenced_projects_given_force_option_when_running_compiler_cli` | `typescript-go/internal/execute/tsctests/tscbuild_test.go` | Rebuild every project in a reference graph even when all projects are up to date. |
| `should_skip_up_to_date_projects_given_dry_build_when_running_compiler_cli` | `typescript-go/internal/execute/tsctests/tscbuild_test.go` | Report all projects as up to date during a dry build after a successful build. |
| `should_report_unbuilt_project_reference_given_composite_import_when_running_cli` | `typescript-go/internal/checker/checker.go` | Report TS6305 when an imported composite project reference has no emitted declaration output. |
| `should_emit_async_function_given_async_function_declaration_when_compiling` | `conformance/async/es6/functionDeclarations` | Preserve async function syntax and emit its body. |
| `should_parse_async_arrow_function_given_parenthesized_parameters_when_building_syntax_tree` | `conformance/async/es2017/asyncArrowFunction/asyncArrowFunction1_es2017.ts` | Parse an async arrow with a parenthesized parameter list. |
| `should_parse_async_arrow_function_given_single_parameter_when_building_syntax_tree` | `conformance/async/es2017/asyncArrowFunction/asyncUnParenthesizedArrowFunction_es2017.ts` | Parse an async arrow with one unparenthesized parameter. |
| `should_parse_await_expression_given_async_arrow_body_when_building_syntax_tree` | `conformance/async/es2017/asyncArrowFunction/asyncArrowFunctionCapturesThis_es2017.ts` | Parse an await expression inside an async arrow body. |
| `should_emit_async_arrow_function_given_es2017_target_when_compiling` | `conformance/async/es2017/asyncArrowFunction/asyncArrowFunction1_es2017.ts` | Preserve the async modifier and erase the arrow return type for an ES2017 target. |
| `should_emit_generator_function_given_generator_declaration_when_compiling` | `conformance/generators` | Preserve generator syntax and emit its body. |
| `should_parse_generator_declaration_given_asterisk_modifier_when_building_syntax_tree` | `conformance/generators/generatorImplicitAny.ts` | Parse a generator function declaration without syntax diagnostics. |
| `should_report_implicit_any_yield_given_unannotated_generator_when_checking_types` | `conformance/generators/generatorImplicitAny.ts` | Report TS7057 when an unannotated generator's `yield` result is used. |
| `should_accept_contextually_typed_yield_given_annotated_variable_when_checking_types` | `conformance/generators/generatorImplicitAny.ts` | Contextually type a `yield` result from its annotated variable. |
| `should_preserve_object_destructuring_given_object_initializer_when_emitting_javascript` | `conformance/es6/destructuring` | Preserve object binding patterns. |
| `should_emit_namespace_given_exported_namespace_value_when_compiling` | `conformance/internalModules` | Emit the exported value assignment inside a namespace IIFE. |
| `should_emit_members_from_merged_namespaces_given_multiple_source_files_when_compiling` | `conformance/internalModules/DeclarationMerging/TwoInternalModulesWithTheSameNameAndSameCommonRoot.ts` | Emit exported members from separate declarations of the same namespace across source files. |
| `should_emit_class_namespace_member_given_exported_namespace_value_when_compiling` | `conformance/internalModules/DeclarationMerging/ClassAndModuleWithSameNameAndCommonRoot.ts` | Emit a namespace value that augments a same-named class. |
| `should_report_namespace_before_class_given_instantiated_namespace_when_checking_types` | `compiler/augmentedTypesModules.ts` | Report TS2434 when a runtime namespace declaration precedes its merged class. |
| `should_report_cross_file_namespace_merge_given_class_in_another_source_file_when_checking_types` | `conformance/internalModules/DeclarationMerging/ClassAndModuleWithSameNameAndCommonRootES6.ts` | Report TS2433 when a class and its merging namespace occur in separate source files. |
| `should_emit_function_namespace_member_given_exported_namespace_value_when_compiling` | `conformance/internalModules/DeclarationMerging/FunctionAndModuleWithSameNameAndCommonRoot.ts` | Emit a namespace value that augments a same-named function. |
| `should_emit_enum_namespace_member_given_exported_namespace_value_when_compiling` | `conformance/internalModules/DeclarationMerging/EnumAndModuleWithSameNameAndCommonRoot.ts` | Emit a namespace value that augments a same-named enum. |
| `should_emit_namespace_function_assignment_given_merged_interface_when_emitting_javascript` | `compiler/declarationEmitNamespaceMergedWithInterfaceNestedFunction.ts` | Emit the assignment that publishes an exported function from a namespace merged with an interface. |
| `should_emit_namespace_function_declaration_given_merged_interface_when_emitting_declarations` | `compiler/declarationEmitNamespaceMergedWithInterfaceNestedFunction.ts` | Emit the exported namespace function signature in declaration output. |
| `should_export_namespace_binding_given_external_module_namespace_when_emitting_commonjs` | `conformance/externalModules/typeOnly/nestedNamespace.ts` | Initialize an exported namespace through its CommonJS export binding. |
| `should_emit_exported_class_given_namespace_member_when_emitting_declarations` | `conformance/externalModules/typeOnly/nestedNamespace.ts` | Preserve an exported namespace class in declaration output. |
| `should_publish_exported_class_from_namespace_given_commonjs_module_when_emitting_javascript` | `conformance/externalModules/typeOnly/nestedNamespace.ts` | Emit an exported class assignment inside a namespace exported from a CommonJS module. |
| `should_keep_unexported_namespace_value_local_given_namespace_member_when_emitting_javascript` | `conformance/internalModules/moduleBody/moduleWithStatementsOfEveryKind.ts` | Keep a private namespace variable inside the emitted namespace closure. |
| `should_emit_nested_namespace_given_exported_namespace_member_when_emitting_javascript` | `conformance/internalModules/moduleBody/moduleWithStatementsOfEveryKind.ts` | Emit a nested namespace through its exported parent member. |
| `should_emit_dotted_namespace_member_given_qualified_namespace_when_emitting_javascript` | `conformance/internalModules/DeclarationMerging/ClassAndModuleWithSameNameAndCommonRootES6.ts` | Emit the nested namespace object and exported class for a dotted namespace declaration. |
| `should_emit_enum_member_from_namespace_given_exported_enum_when_emitting_javascript` | `conformance/internalModules/moduleBody/moduleWithStatementsOfEveryKind.ts` | Emit an exported enum through its containing namespace. |
| `should_parse_dotted_namespace_declaration_given_qualified_name_when_building_syntax_tree` | `conformance/internalModules/DeclarationMerging/ClassAndModuleWithSameNameAndCommonRootES6.ts` | Parse a namespace whose declaration name contains multiple dotted segments. |
| `should_accept_conditional_type_given_generic_type_parameter_when_checking_types` | `conformance/types/conditional/conditionalTypes1.ts` | Accept and represent a conditional type. |
| `should_accept_mapped_type_given_keyof_type_parameter_when_checking_types` | `conformance/types/mapped/mappedTypeModifiers.ts` | Accept a mapped type over `keyof`. |
| `should_accept_template_literal_type_given_string_type_interpolation_when_checking_types` | `conformance/types/literal/templateLiteralTypes1.ts` | Accept a template literal type. |
| `should_resolve_dynamic_import_given_relative_module_when_compiling_sources` | `conformance/dynamicImport/importCallExpression1ES2020.ts` | Resolve and emit a relative dynamic import. |
| `should_parse_dynamic_import_call_given_string_specifier_when_building_syntax_tree` | `conformance/dynamicImport/importCallExpression1ES2020.ts` | Parse a dynamic import call with a string module specifier. |
| `should_reject_escaped_keyword_given_unicode_escape_when_scanning_typescript` | `conformance/scanner/ecmascript5/scannerUnicodeEscapeInKeyword1.ts` | Report TS1260 for an escaped keyword. |
| `should_report_consecutive_numeric_separators_given_numeric_literal_when_scanning_typescript` | `conformance/parser/ecmascript2021/numericSeparators/parser.numericSeparators.decimal.ts` | Report TS6189 for consecutive separators. |
| `should_report_unterminated_string_given_unescaped_line_terminator_when_scanning_typescript` | `conformance/scanner/ecmascript5/scannerStringLiterals.ts` | Report TS1002 for a raw line terminator in a string. |
| `should_parse_regular_expression_literal_given_variable_initializer_when_building_syntax_tree` | `conformance/parser/ecmascript5/RegularExpressions/parserRegularExpression1.ts` | Parse a regular expression literal as an expression. |
| `should_parse_private_identifier_given_class_field_when_building_syntax_tree` | `conformance/classes/members/privateNames/privateNameField.ts` | Parse a private class field. |
| `should_parse_standard_decorator_given_class_declaration_when_building_syntax_tree` | `conformance/esDecorators/classDeclaration/accessors/esDecorators-classDeclaration-accessors-staticPrivate.ts` | Parse a standard class decorator. |
| `should_parse_legacy_class_decorator_given_decorated_class_when_building_syntax_tree` | `conformance/decorators/class/constructableDecoratorOnClass01.ts` | Parse legacy experimental class-decorator syntax. |
| `should_parse_namespace_declaration_given_namespace_block_when_building_syntax_tree` | `projects/moduleMergeOrder/a.ts` | Parse a namespace declaration and its namespace block. |
| `should_report_missing_constructor_implementation_given_overload_only_when_checking_types` | `compiler/ClassDeclaration10.ts` | Report TS2390 for a constructor overload without an implementation. |
| `should_report_missing_method_implementation_given_overload_only_when_checking_types` | `compiler/ClassDeclaration10.ts` | Report TS2391 for a method overload without an implementation. |
| `should_report_mismatched_overload_implementation_given_different_method_name_when_checking_types` | `compiler/ClassDeclaration13.ts` | Report TS2389 when a method implementation does not match its overload name. |
| `should_report_ts2389_given_mismatched_string_method_overload_when_checking_types` | `compiler/ClassDeclaration22.ts` | Report TS2389 when the implementation name does not match a string-literal overload name. |
| `should_report_ts2369_given_parameter_property_in_arrow_function_when_checking_types` | `compiler/ArrowFunctionExpression1.ts` | Report TS2369 when an arrow-function parameter uses a parameter-property modifier. |
| `should_reject_reserved_type_keyword_given_class_name_when_checking_types` | `compiler/ClassDeclaration24.ts` | Report TS2414 when a class is named with the reserved type keyword `any`. |
| `should_report_ts1248_given_const_modifier_on_class_field_when_checking_types` | `compiler/ClassDeclarationWithInvalidConstOnPropertyDeclaration.ts` | Report TS1248 when a class field uses the `const` modifier. |
| `should_parse_intrinsic_jsx_element_given_tsx_source_when_building_syntax_tree` | `conformance/jsx/tsxElementResolution.tsx` | Parse an intrinsic JSX element in a TSX source. |
| `should_accept_indexed_access_type_given_interface_property_when_checking_types` | `conformance/types/keyof/keyofAndIndexedAccess.ts` | Resolve an indexed access to its property type. |
| `should_resolve_overload_given_matching_string_argument_when_checking_types` | `conformance/expressions/functionCalls/overloadResolution.ts` | Select the string overload for a string argument. |
| `should_narrow_union_given_typeof_guard_when_checking_types` | `conformance/controlFlow/controlFlowTypeofObject.ts` | Narrow a string-or-number parameter in a `typeof` guard. |
| `should_emit_typeof_expression_given_identifier_operand_when_compiling` | `conformance/expressions/unaryOperators/typeofOperator/typeofOperatorWithStringType.ts` | Preserve the runtime `typeof` operator and its operand. |
| `should_accept_tuple_type_given_matching_array_literal_when_checking_types` | `conformance/types/tuple/tupleElementTypes1.ts` | Accept a tuple with matching element types. |
| `should_preserve_regular_expression_literal_given_variable_initializer_when_emitting_javascript` | `conformance/parser/ecmascript5/RegularExpressions/parserRegularExpression1.ts` | Preserve a regular expression literal in JavaScript. |
| `should_emit_private_identifier_given_typed_class_field_when_emitting_javascript` | `conformance/classes/members/privateNames/privateNameField.ts` | Erase the field type while preserving the private field. |
| `should_preserve_array_destructuring_given_array_initializer_when_emitting_javascript` | `conformance/es6/destructuring` | Preserve an array binding pattern. |
| `should_preserve_optional_call_given_optional_function_expression_when_emitting_javascript` | `conformance/expressions/optionalChaining/callChain/callChain.ts` | Preserve optional call syntax. |
| `should_accept_generic_constraint_given_matching_argument_when_checking_types` | `conformance/types/typeRelationships/typeInference` | Preserve a string literal through a constrained generic call. |
| `should_infer_conditional_type_member_given_array_type_when_checking_types` | `conformance/types/conditional` | Infer an array element type through `infer`. |
| `should_accept_keyof_type_given_interface_property_when_checking_types` | `conformance/types/keyof/keyofAndIndexedAccess.ts` | Accept an interface property name in its `keyof` type. |
| `should_reject_mapped_readonly_assignment_given_readonly_property_when_checking_types` | `conformance/types/mapped/mappedTypeModifiers.ts` | Report TS2540 for assignment through a readonly mapped property. |
| `should_narrow_discriminated_union_given_literal_property_guard_when_checking_types` | `conformance/types/union/discriminatedUnionTypes1.ts` | Narrow a union to the matching literal-tagged object type. |
| `should_narrow_discriminated_union_given_switch_case_when_checking_types` | `conformance/controlFlow/exhaustiveSwitchStatements1.ts` | Narrow the union to the member matching each switch case and accept its member-specific property access. |
| `should_parse_bigint_literal_union_given_type_alias_when_building_syntax_tree` | `compiler/bigintPropertyName.ts` | Parse BigInt literal types in a type alias without syntax diagnostics. |
| `should_report_ts1539_given_bigint_literal_object_property_when_checking_types` | `compiler/bigintPropertyName.ts` | Report TS1539 when a BigInt literal is used as an object property name. |
| `should_report_ts2790_given_delete_on_required_property_when_checking_types` | `compiler/deleteExpressionMustBeOptional_exactOptionalPropertyTypes.ts` | Report TS2790 when deleting a required property with strict null checking enabled. |
| `should_report_ts6234_given_call_to_get_accessor_when_checking_types` | `compiler/accessorAccidentalCallDiagnostic.ts` | Report TS6234 with an accessor-specific diagnostic when calling a getter. |
| `should_report_ts1183_given_getter_body_in_object_type_when_checking_types` | `compiler/accessorBodyInTypeContext.ts` | Report TS1183 when a getter implementation appears in an object type. |
| `should_report_ts1183_given_setter_body_in_object_type_when_checking_types` | `compiler/accessorBodyInTypeContext.ts` | Report TS1183 when a setter implementation appears in an object type. |
| `should_report_ts1183_given_getter_body_in_interface_when_checking_types` | `compiler/accessorBodyInTypeContext.ts` | Report TS1183 when a getter implementation appears in an interface. |
| `should_report_ts1183_given_setter_body_in_interface_when_checking_types` | `compiler/accessorBodyInTypeContext.ts` | Report TS1183 when a setter implementation appears in an interface. |
| `should_report_ts2511_given_union_with_abstract_constructor_when_checking_types` | `compiler/abstractClassUnionInstantiation.ts` | Report TS2511 when instantiating a constructor union that includes an abstract class. |
| `should_report_ts2427_given_interface_named_string_when_checking_types` | `compiler/InterfaceDeclaration8.ts` | Report TS2427 when an interface uses the predefined type name `string`. |
| `should_report_ts2369_given_constructor_signature_parameter_property_when_checking_types` | `compiler/ParameterList13.ts` | Report TS2369 when a construct signature parameter uses a parameter-property modifier. |
| `should_report_ts2369_given_parameter_property_in_setter_when_checking_types` | `compiler/MemberAccessorDeclaration15.ts` | Report TS2369 when a setter parameter uses a parameter-property modifier. |
| `should_report_ts7013_given_construct_signature_without_return_type_when_checking_types` | `compiler/ParameterList13.ts` | Report TS7013 when a construct signature omits its return type. |
| `should_report_ts7006_given_untyped_construct_signature_parameter_when_checking_types` | `compiler/ParameterList13.ts` | Report TS7006 for an untyped parameter in a construct signature. |
| `should_report_ts2391_given_function_declaration_without_implementation_when_checking_types` | `compiler/FunctionDeclaration3.ts` | Report TS2391 when a top-level function declaration has no implementation. |
| `should_report_ts7010_given_function_declaration_without_return_annotation_when_checking_types` | `compiler/FunctionDeclaration3.ts` | Report TS7010 when a function declaration lacks a return annotation. |
| `should_report_ts7010_given_numeric_named_method_without_return_annotation_when_checking_types` | `compiler/ClassDeclaration21.ts` | Report TS7010 when a numeric-named method overload lacks a return annotation. |
| `should_report_ts2389_given_numeric_method_implementation_name_mismatch_when_checking_types` | `compiler/ClassDeclaration21.ts` | Report TS2389 when a numeric-named method implementation does not match its overload name. |
| `should_report_ts2391_given_method_overload_followed_by_constructor_when_checking_types` | `compiler/ClassDeclaration14.ts` | Report TS2391 when a constructor declaration follows a method overload without an implementation. |
| `should_report_ts7010_given_class_method_overload_without_return_annotation_when_checking_types` | `compiler/ClassDeclaration14.ts` | Report TS7010 when a class method overload lacks a return annotation. |
| `should_report_ts2390_given_constructor_overload_after_method_overload_when_checking_types` | `compiler/ClassDeclaration14.ts` | Report TS2390 when a constructor overload has no implementation. |
| `should_report_ts1440_given_variable_declaration_in_class_member_when_building_syntax_tree` | `compiler/ClassDeclaration26.ts` | Report TS1440 for a variable declaration in a class-member position. |
| `should_report_ts1068_given_var_constructor_member_when_building_syntax_tree` | `compiler/ClassDeclaration26.ts` | Report TS1068 when `var` precedes a constructor member. |
| `should_parse_boolean_literal_union_given_type_alias_when_building_syntax_tree` | `conformance/types/literal/booleanLiteralTypes1.ts` | Parse a type alias whose members are boolean literal types. |
| `should_parse_union_of_object_types_given_type_alias_when_building_syntax_tree` | `conformance/controlFlow/exhaustiveSwitchStatements1.ts` | Parse a union of object type literals in a type alias without syntax diagnostics. |
| `should_parse_string_literal_property_type_given_interface_when_building_syntax_tree` | `conformance/controlFlow/exhaustiveSwitchStatements1.ts` | Parse a string literal type on an interface property without syntax diagnostics. |
| `should_parse_string_literal_union_given_type_alias_when_building_syntax_tree` | `conformance/types/stringLiteral/stringLiteralTypesInUnionTypes01.ts` | Parse a type alias whose members are string literal types. |
| `should_parse_numeric_literal_union_given_type_alias_when_building_syntax_tree` | `conformance/types/literal/numericLiteralTypes1.ts` | Parse a type alias whose members are numeric literal types. |
| `should_parse_negative_numeric_literal_given_type_alias_when_building_syntax_tree` | `conformance/types/literal/numericLiteralTypes1.ts` | Parse a negative numeric literal member in a type alias. |
| `should_narrow_union_given_in_operator_true_branch_when_checking_types` | `conformance/expressions/typeGuards/typeGuardOfFromPropNameInUnionType.ts` | Narrow a class union to the member containing the tested property in the `in` true branch. |
| `should_parse_in_operator_given_return_expression_when_building_syntax_tree` | `conformance/expressions/typeGuards/typeGuardOfFromPropNameInUnionType.ts` | Parse the full `"a" in value` expression without syntax diagnostics. |
| `should_parse_instanceof_operator_given_return_expression_when_building_syntax_tree` | `conformance/controlFlow/controlFlowInstanceOfGuardPrimitives.ts` | Retain the full `value instanceof X` expression in the return-expression span. |
| `should_narrow_class_union_given_instanceof_guard_when_checking_types` | `conformance/controlFlow/controlFlowInstanceofExtendsFunction.ts` | Narrow an `X | number` value to `X` inside an `instanceof X` branch. |
| `should_accept_intersection_type_given_matching_object_properties_when_checking_types` | `conformance/types/intersection/intersectionTypeMembers.ts` | Accept a value with the properties required by both intersection members. |
| `should_narrow_unknown_given_typeof_string_guard_when_checking_types` | `conformance/types/unknown/unknownControlFlow.ts` | Narrow `unknown` to `string` in a `typeof` guard. |
| `should_accept_derived_member_access_given_true_branch_of_user_type_guard_when_checking_types` | `conformance/expressions/typeGuards/typeGuardFunction.ts` | Accept access to a derived member after a user-defined type predicate narrows the base value. |
| `should_parse_type_predicate_return_type_given_function_declaration_when_building_syntax_tree` | `conformance/expressions/typeGuards/typeGuardFunction.ts` | Parse and retain the complete `parameter is Type` return annotation on a function declaration. |
| `should_narrow_nullable_string_given_assertion_function_call_when_checking_types` | `conformance/controlFlow/assertionTypePredicates1.ts` | Narrow a nullable string after a call to a function declared with `asserts value`. |
| `should_narrow_unknown_given_assertion_type_predicate_call_when_checking_types` | `conformance/controlFlow/assertionTypePredicates1.ts` | Narrow `unknown` to `string` after a call declared with `asserts value is string`. |
| `should_parse_import_attributes_given_json_import_when_building_syntax_tree` | `conformance/importAttributes/importAttributes1.ts` | Parse an import declaration with a JSON import attribute. |
| `should_parse_side_effect_import_given_json_attribute_when_building_syntax_tree` | `conformance/importAttributes/importAttributes1.ts` | Parse a side-effect import with a JSON import attribute. |
| `should_parse_reexport_given_json_import_attribute_when_building_syntax_tree` | `conformance/importAttributes/importAttributes2.ts` | Parse a re-export declaration with a JSON import attribute. |
| `should_parse_deferred_import_given_default_binding_when_building_syntax_tree` | `conformance/importDefer/importBindingDefer.ts` | Parse a deferred import declaration with its default binding. |
| `should_write_source_map_given_source_map_option_when_running_compiler_cli` | `conformance/es6/computedProperties/computedPropertyNamesSourceMap1_ES6.ts` | Write JavaScript source maps and link the emitted JavaScript to its map. |
| `should_map_computed_class_members_given_source_map_option_when_emitting_javascript` | `conformance/es6/computedProperties/computedPropertyNamesSourceMap1_ES6.ts` | Emit the pinned source-map segment mappings for computed class methods and accessors. |
| `should_accept_code_point_escape_given_identifier_when_scanning_typescript` | `conformance/scanner/ecmascript5/scannerUnicodeEscapeInKeyword2.ts` | Accept the valid `\\u{0061}` code-point escape in an identifier with an ESNext target. |
| `should_report_undefined_from_optional_chain_given_non_nullable_return_when_checking_types` | `conformance/controlFlow/controlFlowOptionalChain.ts` | Include `undefined` in the optional-chain result and reject its return as non-nullable. |
| `should_resolve_json_import_given_resolve_json_module_project_option_when_running_compiler_cli` | `compiler/isolatedModules_resolveJsonModule.ts` | Resolve and emit an imported JSON module when `resolveJsonModule` is enabled. |
| `should_report_unassigned_variable_read_given_declaration_without_initializer_when_checking_types` | `conformance/types/stringLiteral/stringLiteralMatchedInSwitch01.ts` | Report TS2454 when reading a local variable before it has been assigned. |
| `should_parse_import_meta_expression_given_module_source_when_building_syntax_tree` | `conformance/es2019/importMeta/importMeta.ts` | Parse `import.meta` in a valid ES module source without syntax diagnostics. |
| `should_accept_global_this_property_access_given_unknown_member_when_checking_types` | `conformance/es2019/globalThisUnknown.ts` | Accept an unknown `globalThis` property when implicit-any checking is disabled. |
| `should_accept_global_this_element_access_given_unknown_member_when_checking_types` | `conformance/es2019/globalThisUnknown.ts` | Accept an unknown `globalThis` element access when implicit-any checking is disabled. |
| `should_parse_import_type_given_module_specifier_when_building_syntax_tree` | `conformance/types/import/importTypeAmbient.ts` | Parse an import type with a module specifier and qualified exported type. |
| `should_resolve_imported_interface_given_import_type_reference_when_compiling_sources` | `conformance/types/import/importTypeLocal.ts` | Resolve an imported interface through a relative import type in a sibling source. |
| `should_parse_typeof_import_given_module_specifier_when_building_syntax_tree` | `conformance/types/import/importTypeAmbient.ts` | Parse `typeof import("foo")` as an imported module type query. |
| `should_parse_qualified_typeof_import_given_module_specifier_when_building_syntax_tree` | `conformance/types/import/importTypeAmbient.ts` | Parse `typeof import("foo2").Bar` as a qualified imported module type query. |
| `should_resolve_scoped_package_types_given_package_json_types_field_when_running_compiler_cli` | `compiler/moduleResolution_packageJson_scopedPackage.ts` | Follow a scoped package's `types` field and resolve the imported declaration. |
| `should_report_uninitialized_property_given_strict_project_option_when_compiling_project` | `conformance/classes/propertyMemberDeclarations/strictPropertyInitialization.ts` | Report TS2564 for an uninitialized required class property under strict mode. |
| `should_emit_type_predicate_given_exported_function_when_emitting_declarations` | `conformance/declarationEmit/typePredicates/declarationEmitIdentifierPredicates01.ts` | Emit the exported function's `x is number` type predicate in its declaration. |
| `should_report_indexed_property_type_mismatch_given_string_index_signature_when_checking_types` | `conformance/types/objectTypeLiteral/indexSignatures/stringIndexerConstrainsPropertyDeclarations.ts` | Report TS2411 when a named property violates its string index signature. |
| `should_report_numeric_property_mismatch_given_numeric_index_signature_when_checking_types` | `conformance/types/objectTypeLiteral/indexSignatures/numericIndexerConstrainsPropertyDeclarations.ts` | Report TS2411 when a numeric property violates its numeric index signature. |
| `should_read_numeric_index_signature_value_given_number_key_when_checking_types` | `conformance/types/objectTypeLiteral/indexSignatures/numericIndexingResults.ts` | Infer the numeric index signature's value type when a number key is read. |
| `should_parse_new_target_meta_property_given_function_body_when_building_syntax_tree` | `conformance/es6/newTarget/newTarget.es6.ts` | Parse the `new.target` meta-property within a function body. |
| `should_suppress_javascript_output_given_no_emit_project_configuration_when_running_compiler_cli` | `compiler/compilerOptionsOutDirAndNoEmit.ts` | Honor project `noEmit` and leave the JavaScript output absent. |
| `should_ignore_first_line_hashbang_given_typescript_source_when_parsing` | `compiler/shebang.ts` | Accept and preserve a first-line hashbang in a TypeScript source file. |
| `should_lower_exponentiation_operator_given_es2015_target_when_emitting_javascript` | `conformance/es7/exponentiationOperator/emitExponentiationOperator1.ts` | Lower exponentiation to `Math.pow` when targeting ES2015. |
| `should_lower_right_associative_exponentiation_given_es2015_target_when_emitting_javascript` | `conformance/es7/exponentiationOperator/emitExponentiationOperator1.ts` | Lower `2 ** 3 ** 2` to nested `Math.pow(2, Math.pow(3, 2))` calls. |
| `should_lower_exponentiation_assignment_given_es2015_target_when_emitting_javascript` | `conformance/es7/exponentiationOperator/emitCompoundExponentiationOperator1.ts` | Lower `value **= 3` to `value = Math.pow(value, 3)`. |
| `should_evaluate_exponentiation_assignment_receiver_once_given_side_effectful_access_when_emitting_javascript` | `conformance/es7/exponentiationOperator/emitCompoundExponentiationAssignmentWithPropertyAccessingOnLHS1.ts` | Evaluate a side-effectful property receiver once when lowering `**=`. |
| `should_include_javascript_project_source_given_allow_js_option_when_running_compiler_cli` | `compiler/allowJsClassThisTypeCrash.ts` | Include `.js` root files in default project discovery with `allowJs`. |
| `should_report_duplicate_block_scoped_bindings_given_same_scope_when_checking_types` | `compiler/letDeclarations-scopes-duplicates.ts` | Report TS2451 for same-scope duplicate `let` declarations. |
| `should_lower_object_spread_given_es2015_target_when_emitting_javascript` | `conformance/types/spread/objectSpread.ts` | Lower object spread to `Object.assign` when targeting ES2015. |
| `should_accept_numeric_literal_given_numeric_enum_annotation_when_checking_types` | `conformance/types/primitives/enum/validEnumAssignments.ts` | Accept numeric literal assignment to a numeric enum. |
| `should_report_unterminated_regex_given_regex_at_end_of_file_when_scanning` | `compiler/unterminatedRegexAtEndOfSource1.ts` | Report TS1161 for a regular-expression literal that reaches end of file. |
| `should_resolve_cross_root_imports_given_root_dirs_project_option_when_running_compiler_cli` | `compiler/pathMappingBasedModuleResolution6_node.ts` | Resolve a relative import through configured `rootDirs`. |
| `should_resolve_non_relative_project_import_given_paths_pattern_when_running_compiler_cli` | `projects/non-relative/consume.ts` | Resolve a non-relative import through a TypeScript 7 `paths` mapping. |
| `should_resolve_inherited_path_pattern_given_project_import_when_running_compiler_cli` | `projects/non-relative/consume.ts` | Resolve an inherited `paths` pattern relative to its declaring config instead of a conflicting root-level candidate. |
| `should_report_boolean_enum_initializer_given_computed_member_when_checking_types` | `conformance/enums/enumErrors.ts` | Report TS18033 for a boolean computed enum member initializer. |
| `should_report_boxed_number_given_computed_enum_member_when_checking_types` | `conformance/enums/enumErrors.ts` | Report TS18033 when a boxed `Number` initializes a computed enum member. |
| `should_fold_string_enum_initializer_given_constant_concatenation_when_emitting_javascript` | `conformance/enums/enumConstantMemberWithString.ts` | Fold a constant string concatenation in a string-valued enum member. |
| `should_omit_reverse_mapping_given_computed_string_enum_member_when_emitting_javascript` | `conformance/enums/enumConstantMemberWithString.ts` | Emit a computed string-valued enum member without a numeric reverse mapping. |
| `should_preserve_folded_string_enum_value_given_constant_initializer_when_emitting_declarations` | `conformance/enums/enumConstantMemberWithStringEmitDeclaration.ts` | Preserve the folded string constant in the emitted enum declaration. |
| `should_report_reserved_enum_name_given_keyword_enum_declaration_when_checking_types` | `conformance/enums/enumErrors.ts` | Report TS2431 when an enum uses the predefined type name `any`. |
| `should_parse_await_using_declaration_given_null_initializer_when_building_tree` | `conformance/statements/VariableStatements/usingDeclarations/awaitUsingDeclarations.1.ts` | Parse `await using` as a variable declaration under ESNext module syntax. |
| `should_report_invalid_variable_decorator_given_decorated_variable_when_compiling` | `conformance/decorators/invalid/decoratorOnVar.ts` | Report TS1206 when a decorator is applied to a variable declaration. |
| `should_lower_nullish_assignment_given_es2015_target_when_emitting_javascript` | `conformance/es2021/logicalAssignment/logicalAssignment1.ts` | Lower `??=` to an ES2015 nullish check and assignment. |
| `should_lower_conjunction_assignment_given_es2015_target_when_emitting_javascript` | `conformance/es2021/logicalAssignment/logicalAssignment1.ts` | Lower `&&=` to a short-circuit check and assignment for ES2015. |
| `should_lower_disjunction_assignment_given_es2015_target_when_emitting_javascript` | `conformance/es2021/logicalAssignment/logicalAssignment1.ts` | Lower `||=` to a short-circuit check and assignment for ES2015. |
| `should_preserve_conjunction_assignment_given_es2021_target_when_emitting_javascript` | `conformance/es2021/logicalAssignment/logicalAssignment1.ts` | Preserve `&&=` syntax for an ES2021 target. |
| `should_preserve_disjunction_assignment_given_es2021_target_when_emitting_javascript` | `conformance/es2021/logicalAssignment/logicalAssignment1.ts` | Preserve `||=` syntax for an ES2021 target. |
| `should_preserve_nullish_assignment_given_es2021_target_when_emitting_javascript` | `conformance/es2021/logicalAssignment/logicalAssignment1.ts` | Preserve `??=` syntax for an ES2021 target. |
| `should_report_unused_expect_error_given_valid_following_statement_when_compiling` | `conformance/directives/ts-expect-error.ts` | Report TS2578 for an unused `@ts-expect-error` directive. |
| `should_suppress_type_error_given_ignore_directive_on_preceding_line_when_compiling` | `conformance/directives/ts-ignore.ts` | Suppress the following line's type diagnostic with `@ts-ignore`. |
| `should_parse_using_declaration_given_null_initializer_when_building_tree` | `conformance/statements/VariableStatements/usingDeclarations/usingDeclarations.1.ts` | Parse synchronous `using` as a variable declaration. |
| `should_lower_using_declaration_given_es2022_target_when_emitting_javascript` | `conformance/statements/VariableStatements/usingDeclarations/usingDeclarations.1.ts` | Emit disposal helpers and a `finally` block for a synchronous `using` declaration targeting ES2022. |
| `should_resolve_referenced_ambient_module_given_named_import_when_running_compiler_cli` | `projects/relative-global-ref/consume.ts` | Load a referenced ambient module and resolve its named import in a project. |
| `should_report_implicit_any_this_given_unannotated_function_declaration_when_compiling_project` | `compiler/thisInFunctionCall.ts` | Report TS2683 for an unannotated function declaration's `this` value with `noImplicitThis`. |
| `should_report_implicit_any_given_array_extends_project_configuration_when_running_compiler_cli` | `compiler/configFileExtendsAsList.ts` | Apply inherited `noImplicitAny` from an array-valued `extends` chain and report TS7006. |
| `should_apply_later_array_extended_option_given_conflicting_values_when_running_compiler_cli` | `compiler/configFileExtendsAsList.ts` | Apply `noImplicitAny=true` from the later array-extended config when the earlier config sets it to false. |
| `should_report_implicit_any_given_no_implicit_any_in_parent_config_when_running_compiler_cli` | `compiler/configFileExtendsAsList.ts` | Report TS7006 when a single extended config enables `noImplicitAny`. |
| `should_report_implicit_any_return_given_circular_imported_calls_when_compiling_project` | `projects/CircularReferencing/consume.ts`, `projects/CircularReferencing/decl.ts` | Report TS7023 for implicitly typed functions with mutually recursive return inference. |
| `should_compile_relative_import_equals_dependency_given_project_root_when_running_compiler_cli` | `projects/relative-global/consume.ts`, `projects/relative-global/decl.ts` | Parse and resolve a relative import-equals dependency when compiling the project. |
| `should_resolve_nested_relative_import_equals_given_project_root_when_running_compiler_cli` | `projects/relative-nested/app.ts`, `projects/relative-nested/main/consume.ts`, `projects/relative-nested/decl.ts` | Resolve the root's nested relative import-equals dependency chain when compiling a project. |
| `should_elide_unused_cycle_import_given_commonjs_project_when_emitting_javascript` | `projects/CircularReferencing-2/a.ts`, `projects/CircularReferencing-2/b.ts`, `projects/CircularReferencing-2/c.ts` | Omit an unused import that closes a project dependency cycle from CommonJS output. |
| `should_emit_one_import_given_repeated_module_imports_when_emitting_declarations` | `projects/declarations_MultipleTimesImport/useModule.ts`, `projects/declarations_MultipleTimesImport/m4.ts` | Emit one declaration import for repeated references to the same module. |
| `should_preserve_transitive_imported_type_given_declaration_emit_when_compiling_project` | `projects/declarations_IndirectImport/useModule.ts`, `projects/declarations_IndirectImport/m5.ts`, `projects/declarations_IndirectImport/m4.ts` | Preserve the transitive module origin of an exported inferred declaration type. |
| `should_elide_intermediate_module_import_given_exported_inferred_type_when_emitting_declarations` | `projects/declarations_MultipleTimesMultipleImport/useModule.ts`, `projects/declarations_MultipleTimesMultipleImport/m5.ts`, `projects/declarations_MultipleTimesMultipleImport/m4.ts` | Emit the final exported type through its origin module and omit the intermediate import. |
| `should_suggest_node_types_given_missing_module_global_and_empty_types_option_when_compiling_project` | `conformance/typings/typingsSuggestion1.ts` | Report TS2591 with Node type-install guidance when `types` is empty and `module` is unavailable. |
| `should_parse_string_literal_export_name_given_export_alias_when_building_tree` | `conformance/es2022/arbitraryModuleNamespaceIdentifiers/arbitraryModuleNamespaceIdentifiers_syntax.ts` | Accept and retain a string-literal public name in an export specifier. |
| `should_parse_string_literal_namespace_export_given_star_reexport_when_building_tree` | `conformance/es2022/arbitraryModuleNamespaceIdentifiers/arbitraryModuleNamespaceIdentifiers_syntax.ts` | Accept a string-literal namespace name on an export-star declaration. |
| `should_suppress_type_errors_given_nocheck_directive_when_compiling_file` | `conformance/directives/ts-expect-error-nocheck.ts` | Suppress file diagnostics with `@ts-nocheck`. |
| `should_report_bigint_exponentiation_given_es2015_target_when_checking_types` | `compiler/bigIntWithTargetLessThanES2016.ts` | Report TS2791 for BigInt exponentiation below the ES2016 target. |
| `should_report_missing_object_values_given_es5_library_when_compiling_project` | `conformance/es2017/useObjectValuesAndEntries2.ts` | Report TS2550 when `Object.values` is unavailable in the selected ES5 library. |
| `should_report_missing_object_entries_given_es5_library_when_compiling_project` | `conformance/es2017/useObjectValuesAndEntries2.ts` | Report TS2550 when `Object.entries` is unavailable in the selected ES5 library. |
| `should_accept_object_values_given_es2017_object_library_when_compiling_project` | `conformance/es2017/useObjectValuesAndEntries1.ts` | Accept `Object.values` when ES2017.Object is selected with an ES2015 target. |
| `should_accept_object_entries_given_es2017_object_library_when_compiling_project` | `conformance/es2017/useObjectValuesAndEntries1.ts` | Accept `Object.entries` when ES2017.Object is selected with an ES2015 target. |
| `should_inline_const_enum_member_given_property_access_when_emitting_javascript` | `conformance/constEnums/constEnumPropertyAccess1.ts` | Substitute the constant enum member value during JavaScript emit. |
| `should_report_invalid_character_given_nul_characters_when_scanning_typescript` | `conformance/scanner/ecmascript5/scannerUnexpectedNullCharacter1.ts` | Report TS1127 for each NUL character in source text. |
| `should_assign_structurally_compatible_named_interfaces_given_variable_initializer_when_checking_types` | `conformance/types/typeRelationships/assignmentCompatibility/assignmentCompatWithObjectMembers.ts` | Accept assignment between named interfaces with matching members. |
| `should_resolve_package_extended_config_through_exports_wildcard_when_running_compiler_cli` | `compiler/tsconfigExtendsPackageJsonExportsWildcard.ts` | Resolve an `extends` package path through a wildcard package export and apply its inherited `strictNullChecks` option. |
| `should_report_forward_parameter_reference_given_default_initializer_when_checking_types` | `compiler/capturedParametersInInitializers1.ts` | Report TS2373 when a default parameter initializer references a later parameter. |
| `should_parse_for_await_of_given_async_function_body_when_building_syntax_tree` | `conformance/parser/ecmascript2018/forAwait/parser.forAwait.es2018.ts` | Parse a `for await...of` loop in an async function under ES2018. |
| `should_resolve_global_type_from_type_package_given_types_project_option_when_running_compiler_cli` | `conformance/references/library-reference-13.ts` | Load global declarations from a configured type package and type root. |
| `should_preserve_omitted_catch_binding_given_es2019_target_when_emitting_javascript` | `conformance/emitter/es2019/noCatchBinding/emitter.noCatchBinding.es2019.ts` | Preserve a catch clause without a binding when targeting ES2019. |
| `should_report_nested_javascript_type_mismatch_given_max_node_module_js_depth_three_when_compiling_project` | `projects/NodeModulesSearch/maxDepthIncreased` | Check nested JavaScript modules at the configured `maxNodeModuleJsDepth` and report TS2322. |
| `should_report_jsdoc_array_type_mismatch_given_javascript_assignment_when_checking_types` | `conformance/salsa/checkSpecialPropertyAssignments.ts` | Report TS2322 when a JSDoc `string[]` value is assigned to a `number[]` variable. |
| `should_reject_symbol_given_bigint_constructor_argument_when_checking_types` | `conformance/es2020/constructBigint.ts` | Report TS2345 when a symbol is passed to `BigInt`. |
| `should_accept_string_given_bigint_constructor_argument_when_checking_types` | `conformance/es2020/constructBigint.ts` | Accept a string argument to the ESNext `BigInt` constructor. |
| `should_resolve_package_from_parent_node_modules_given_nested_importer_when_compiling_sources` | `conformance/node/nodeModulesPackageExports.ts` | Resolve a package from an ancestor `node_modules` directory. |
| `should_instantiate_generic_interface_given_structural_assignment_when_checking_types` | `conformance/types/namedTypes/genericInstantiationEquivalentToObjectLiteral.ts` | Accept generic interface instantiation and structural assignment. |
| `should_inherit_interface_members_given_extends_clause_when_checking_types` | `conformance/interfaces/interfaceDeclarations/interfaceExtendsObjectIntersection.ts` | Include inherited interface members in the derived type. |
| `should_narrow_nullable_string_given_truthy_guard_when_checking_types` | `conformance/controlFlow/controlFlowTruthiness.ts` | Narrow `string \| undefined` to `string` in the truthy branch. |
| `should_report_no_matching_overload_given_boolean_argument_when_checking_types` | `conformance/expressions/functionCalls/overloadResolution.ts` | Report TS2769 when no overload accepts the argument. |
| `should_preserve_generic_parameters_given_exported_function_when_emitting_declarations` | `conformance/types/typeRelationships/typeInference/genericFunctionParameters.ts` | Emit the generic parameter and signature in the declaration. |
| `should_preserve_readonly_interface_property_given_exported_interface_when_emitting_declarations` | `conformance/controlFlow/typeGuardsAsAssertions.ts` | Preserve `readonly` on the emitted interface property. |
| `should_resolve_path_mapped_import_given_paths_project_option_when_running_compiler_cli` | `compiler/resolutionCandidateFromPackageJsonField1.ts` | Resolve a path-mapped import from project configuration. |
| `should_resolve_package_main_given_non_index_package_entry_when_running_compiler_cli` | `conformance/moduleResolution/packageJsonMain.ts` | Resolve a package through its `main` entry. |
| `should_resolve_at_types_package_given_bare_import_when_compiling_sources` | `conformance/moduleResolution/nodeModulesAtTypesPriority.ts` | Resolve package types from `node_modules/@types`. |
| `should_resolve_exported_package_subpath_given_node16_project_when_running_compiler_cli` | `conformance/node/nodeModulesPackageExports.ts` | Resolve a package subpath selected by the Node16 `exports` map. |
| `should_resolve_package_import_map_given_hash_specifier_when_running_compiler_cli` | `conformance/node/nodeModulesPackageImports.ts` | Resolve a `#` package import through `package.json` and type-check the mapped source. |
| `should_resolve_package_import_wildcard_given_nested_hash_specifier_when_running_compiler_cli` | `conformance/node/nodeModulesPackageImportsRootWildcard.ts` | Substitute a wildcard in `package.json` imports and resolve a nested `.js` specifier to TypeScript. |
| `should_suppress_javascript_output_given_no_emit_option_when_running_compiler_cli` | `typescript-go/internal/project` and `tsc/noEmit/when-project-has-strict-true.js` | Accept `--noEmit` and produce no JavaScript file. |
| `should_preserve_source_subdirectories_given_out_dir_option_when_running_compiler_cli` | `compiler/commonSourceDirectory.ts` | Preserve source subdirectories below the configured output directory. |
| `should_emit_division_in_template_substitution_given_template_expression_when_emitting_javascript` | `conformance/es6/templates/templateStringWithEmbeddedDivision.ts` | Preserve the division expression within a template substitution. |
| `should_parse_deferred_namespace_import_given_module_import_when_building_syntax_tree` | `conformance/importDefer/importDeferNamespace.ts` | Parse `import defer * as` namespace binding syntax. |
| `should_preserve_jsx_fragment_given_preserve_mode_when_running_compiler_cli` | `conformance/jsx/tsxFragmentPreserveEmit.tsx` | Preserve a JSX fragment under the `preserve` JSX mode. |
| `should_apply_standard_class_decorator_given_decorated_class_when_emitting_javascript` | `conformance/esDecorators/classDeclaration/esDecorators-classDeclaration-simpleTransformation.ts` | Emit the standard decorator application helper call. |
| `should_lower_optional_property_access_given_es2015_target_when_emitting_javascript` | `conformance/expressions/optionalChaining/optionalChainingInference.ts` | Lower optional property access for an ES2015 target. |
| `should_lower_auto_accessor_given_es2022_target_when_emitting_javascript` | `conformance/esDecorators/classDeclaration/esDecorators-classDeclaration-commentPreservation.ts` | Lower an auto-accessor to private backing storage and getter/setter methods under ES2022. |
| `should_preserve_static_import_attributes_given_es_module_import_when_emitting_javascript` | `conformance/importAttributes/importAttributes1.ts` | Preserve static import attributes in ES module output. |
| `should_preserve_dynamic_import_attributes_given_es_module_output_when_emitting_javascript` | `conformance/importAssertion/importAssertion1.ts` | Preserve the second argument to `import()` and its `with` attributes in ES module output. |
| `should_report_invalid_hex_escape_given_untagged_template_when_compiling` | `conformance/es2018/invalidTaggedTemplateEscapeSequences.ts` | Report TS1125 for an invalid hexadecimal escape in an untagged template. |
| `should_accept_regexp_escape_given_es2025_regexp_library_when_checking_types` | `conformance/es2025/regExpEscape.ts` | Resolve `RegExp.escape` as a string-to-string function from the ES2025 RegExp library. |
| `should_report_invalid_unicode_escape_given_non_hex_digit_when_scanning_string_literal` | `conformance/scanner/ecmascript5/scannerS7.8.4_A7.1_T4.ts` | Report TS1125 for a non-hexadecimal digit in a string Unicode escape. |
| `should_reject_private_member_as_public_structural_assignment_given_class_instance_when_checking_types` | `conformance/types/typeRelationships/assignmentCompatibility/assignmentCompatWithObjectMembersAccessibility.ts` | Report TS2322 when a private class member does not satisfy a public structural property. |
| `should_reject_protected_property_access_given_external_instance_when_checking_types` | `conformance/classes/members/accessibility/classPropertyAsProtected.ts` | Report TS2445 when code outside the class reads a protected property. |
| `should_reject_protected_member_access_through_base_receiver_given_derived_class_method_when_checking_types` | `conformance/classes/members/accessibility/protectedInstanceMemberAccessibility.ts` | Report TS2446 when a derived class reads a protected member through a base-class-typed receiver. |
| `should_erase_non_null_assertion_given_nullable_variable_when_emitting_javascript` | `compiler/narrowingWithNonNullExpression.ts` | Erase a postfix non-null assertion while preserving its operand in JavaScript output. |
| `should_parse_non_null_assertion_given_variable_expression_when_building_syntax_tree` | `compiler/narrowingWithNonNullExpression.ts` | Parse `value!` as a full expression without syntax diagnostics. |
| `should_accept_get_canonical_locales_given_es2016_target_when_checking_types` | `conformance/es2016/es2016IntlAPIs.ts` | Resolve `Intl.getCanonicalLocales` and accept its `string[]` return under the ES2016 target. |
| `should_report_missing_hex_digit_given_hex_prefix_without_digits_when_scanning_typescript` | `conformance/scanner/ecmascript5/scannerS7.8.3_A6.1_T1.ts` | Report TS1125 for a hexadecimal prefix with no digits. |
| `should_reject_number_assignment_given_destructured_constrained_generic_result_when_checking_types` | `conformance/inferFromBindingPattern.ts` | Report TS2322 when a constrained generic tuple element inferred as `string` is assigned to `number`. |
| `should_resolve_arbitrary_native_extension_declaration_given_allow_arbitrary_extensions_when_running_node18_compiler_cli` | `conformance/nonjsExtensions/declarationFilesForNodeNativeModules.ts` | Resolve `./dir/native.node` through `dir/native.d.node.ts` and emit Node 18 module output with `allowArbitraryExtensions`. |
| `should_report_missing_month_argument_given_year_only_date_utc_call_when_checking_types` | `conformance/es5/es5DateAPIs.ts` | Report TS2554 when `Date.UTC` receives only the year argument under the ES5 library. |
| `should_reject_number_assignment_given_resizable_flag_when_checking_types` | `conformance/es2024/resizableArrayBuffer.ts` | Report TS2322 when the `ArrayBuffer.resizable` boolean is assigned to `number`. |
| `should_lower_nullish_assignment_with_nullish_rhs_given_es2015_target_when_emitting_javascript` | `conformance/esnext/logicalAssignment/logicalAssignment11.ts` | Lower nested nullish assignment to a nullish check and assignment under ES2015. |
| `should_reject_boolean_assignment_given_es2023_resolved_use_grouping_result_when_checking_types` | `conformance/es2023/intlNumberFormatES2023.ts` | Report TS2322 because `resolvedOptions().useGrouping` can be a string under the ES2023 Intl library. |
| `should_accept_zero_argument_atomics_pause_call_given_esnext_library_when_checking_types` | `conformance/esnext/esnextSharedMemory.ts` | Resolve `Atomics.pause` from the ESNext library with an optional numeric argument. |
| `should_report_incompatible_union_property_given_object_literal_when_checking_types` | `conformance/types/union/contextualTypeWithUnionTypeObjectLiteral.ts` | Report TS2322 when a fresh object literal property has a union type incompatible with every target union member. |
| `should_preserve_imported_function_return_type_given_exported_call_result_when_emitting_declarations` | `projects/declarations_ImportedUseInFunction/useModule.ts` | Preserve a named return type from an imported function call in the declaration output. |
| `should_preserve_cascaded_type_import_given_exported_class_property_when_emitting_declarations` | `projects/declarations_CascadingImports/useModule.ts` | Preserve an imported type binding in an intermediate declaration file across a module chain. |
| `should_exclude_output_directory_from_implicit_roots_given_out_dir_option_when_running_compiler_cli` | `projects/projectOption/DefaultExcludeNodeModulesAndOutDir/a.ts` | Exclude the configured output directory from implicit project root discovery. |
| `should_report_missing_bare_import_assignment_given_no_module_project_input_when_running_compiler_cli` | `projects/NoModule/decl.ts` | Report TS2307 for an unresolved bare import-equals module in the NoModule project case. |
| `should_emit_namespace_child_assignment_given_exported_class_when_emitting_javascript` | `projects/PrologueEmit/__extends.ts` | Emit the exported `m.child` class assignment inside a namespace IIFE. |
| `should_emit_module_identifier_as_commonjs_require_given_sibling_module_when_running_compiler_cli` | `projects/ModuleIdentifier/consume.ts`, `projects/baseline/emit.ts` | Emit a relative import-equals module identifier as a CommonJS `require`. |
| `should_emit_declaration_for_referenced_source_given_triple_slash_path_when_compiling_root` | `projects/ReferenceResolution/src/ts/foo/foo.ts` | Include the triple-slash referenced source in declaration output. |
| `should_preserve_triple_slash_reference_given_declaration_path_when_emitting_javascript` | `projects/DeclareExportAdded/consumer.ts` | Preserve the triple-slash reference directive in emitted JavaScript. |
| `should_preserve_imported_class_type_given_exported_inferred_value_when_emitting_declarations` | `projects/declarations_SimpleImport/useModule.ts` | Preserve the imported class type in an exported inferred declaration. |
| `should_preserve_import_equals_alias_given_exported_imported_type_when_emitting_declarations` | `projects/privacyCheck-SimpleReference/test.ts` | Preserve the import-equals alias and its qualified imported class type in declaration output. |
| `should_resolve_nested_import_equals_dependencies_given_multilevel_project_when_running_compiler_cli` | `projects/MultipleLevels/B/B.ts` | Resolve nested relative import-equals dependencies when compiling the project module. |
| `should_report_removed_classic_module_resolution_given_project_option_when_running_compiler_cli` | `projects/RelativePaths/app.ts` | Report TS5108 when the project selects removed `moduleResolution=Classic`. |
| `should_erase_ambient_function_given_typed_call_when_emitting_javascript` | `conformance/ambient/ambientDeclarations.ts` | Erase the ambient function declaration and preserve its call. |
| `should_report_initializer_given_ambient_variable_declaration_when_compiling_sources` | `conformance/ambient/ambientErrors.ts` | Report TS1039 for an initializer on an ambient variable. |
| `should_report_function_body_given_ambient_function_declaration_when_compiling_sources` | `conformance/ambient/ambientErrors.ts` | Report TS1183 when an ambient function declares an implementation body. |
| `should_report_parameter_initializer_given_ambient_function_signature_when_compiling_sources` | `conformance/ambient/ambientErrors.ts` | Report TS2371 for a default parameter in an ambient function signature. |
| `should_report_nonconstant_initializer_given_ambient_enum_member_when_compiling_sources` | `conformance/ambient/ambientErrors.ts` | Report TS1066 for a computed initializer in an ambient enum declaration. |
| `should_accept_constant_expressions_given_ambient_enum_members_when_compiling_sources` | `conformance/ambient/ambientEnumDeclaration1.ts` | Accept arithmetic expressions and prior member references in ambient enum constants. |
| `should_assign_incrementing_values_given_ambient_const_enum_members_when_compiling_sources` | `conformance/ambient/ambientEnumDeclaration2.ts` | Assign increasing literal values to uninitialized ambient const enum members. |
| `should_report_nested_module_given_ambient_module_inside_namespace_when_compiling_sources` | `conformance/ambient/ambientErrors.ts` | Report TS2435 when an ambient module is nested inside a namespace. |
| `should_report_relative_name_given_ambient_module_declaration_when_compiling_sources` | `conformance/ambient/ambientErrors.ts` | Report TS2436 when an ambient module declaration uses a relative module name. |
| `should_report_mixed_ambient_exports_given_export_assignment_when_compiling_sources` | `conformance/ambient/ambientErrors.ts` | Report TS2309 when an ambient module combines `export =` with named exports. |
| `should_accept_exported_ambient_variable_given_namespace_member_when_compiling_sources` | `conformance/ambient/ambientInsideNonAmbient.ts` | Accept an exported ambient variable declaration inside a namespace. |
| `should_reject_export_modifier_given_ambient_module_augmentation_when_compiling_sources` | `conformance/ambient/ambientExternalModuleInsideNonAmbientExternalModule.ts` | Report TS2668 when an ambient module augmentation inside an external module has an `export` modifier. |
| `should_reject_export_modifier_given_nested_ambient_module_when_compiling_sources` | `conformance/ambient/ambientExternalModuleInsideNonAmbient.ts` | Report TS2668 when a nested ambient module declaration has an `export` modifier. |
| `should_report_missing_module_given_ambient_augmentation_when_compiling_sources` | `conformance/ambient/ambientExternalModuleInsideNonAmbientExternalModule.ts` | Report TS2664 when an ambient module augmentation targets a missing module. |
| `should_allow_arbitrary_member_given_shorthand_module_import_when_compiling_sources` | `conformance/ambient/ambientShorthand.ts` | Treat an import from a shorthand ambient module as `any`. |
| `should_merge_duplicate_shorthand_modules_given_import_when_compiling_sources` | `conformance/ambient/ambientShorthand_duplicate.ts` | Merge repeated shorthand module declarations and resolve an import from the module. |
| `should_merge_members_given_repeated_ambient_module_declarations_when_compiling_sources` | `conformance/ambient/ambientExternalModuleMerging.ts` | Combine exports from repeated ambient module declarations for imported member access. |
| `should_merge_shorthand_module_with_named_module_declaration_when_compiling_sources` | `conformance/ambient/ambientShorthand_merging.ts` | Combine a shorthand ambient module with a declaration that adds named exports. |
| `should_emit_shorthand_module_declaration_given_declaration_option_when_emitting` | `conformance/ambient/ambientShorthand_declarationEmit.ts` | Preserve a shorthand ambient module declaration in emitted `.d.ts` output. |
| `should_resolve_named_reexport_given_shorthand_module_when_compiling_sources` | `conformance/ambient/ambientShorthand_reExport.ts` | Resolve a named re-export from a shorthand ambient module. |
| `should_resolve_star_reexport_given_shorthand_module_when_compiling_sources` | `conformance/ambient/ambientShorthand_reExport.ts` | Resolve a star re-export from a shorthand ambient module. |
| `should_report_reserved_namespace_name_given_declare_namespace_when_compiling_sources` | `conformance/ambient/ambientModuleDeclarationWithReservedIdentifierInDottedPath2.ts` | Report TS2819 for a namespace named with the reserved identifier `debugger`. |
| `should_accept_reserved_identifier_given_dotted_namespace_segment_when_compiling_sources` | `conformance/ambient/ambientModuleDeclarationWithReservedIdentifierInDottedPath.ts` | Accept `debugger` as an intermediate identifier in a dotted namespace path. |
| `should_resolve_ambient_module_given_named_import_when_compiling_sources` | `conformance/ambient/ambientDeclarationsExternal.ts` | Resolve the named import supplied by an ambient module. |
| `should_resolve_wildcard_ambient_module_given_suffix_import_when_compiling_sources` | `conformance/ambient/ambientDeclarationsPatterns.ts` | Resolve an import ending in `!text` through a wildcard ambient module declaration. |
| `should_merge_concrete_module_with_wildcard_ambient_declaration_when_compiling_sources` | `conformance/ambient/ambientDeclarationsPatterns_merging1.ts` | Merge a concrete ambient module's export with the export provided by a matching wildcard declaration. |
| `should_report_missing_wildcard_export_given_standalone_ambient_declaration_when_compiling_sources` | `conformance/ambient/ambientDeclarationsPatterns_merging1.ts` | Report TS2305 when a standalone concrete ambient module declaration in a separate file does not inherit the wildcard module's export. |
| `should_report_missing_export_given_augmentation_for_another_wildcard_module_when_compiling_sources` | `conformance/ambient/ambientDeclarationsPatterns_merging2.ts` | Report TS2305 when a named import targets a wildcard-matched module outside the augmented module's specifier. |
| `should_reject_augmentation_member_given_distinct_wildcard_module_specifier_when_checking_types` | `conformance/ambient/ambientDeclarationsPatterns_merging3.ts` | Keep a concrete module augmentation scoped to its specifier and report TS2339 for another wildcard-matched module. |
| `should_report_multiple_asterisks_given_wildcard_ambient_module_when_compiling_sources` | `conformance/ambient/ambientDeclarationsPatterns_tooManyAsterisks.ts` | Report TS5061 when an ambient module pattern contains more than one asterisk. |
| `should_preserve_symbol_computed_property_given_symbol_key_when_emitting_javascript` | `conformance/Symbols/ES5SymbolProperty1.ts` | Preserve the computed symbol property key. |
| `should_emit_export_assignment_given_commonjs_module_when_emitting_javascript` | `conformance/externalModules/exportAssignTypes.ts` | Emit `module.exports` for an export assignment. |
| `should_preserve_export_assignment_type_given_relative_import_equals_when_compiling_sources` | `conformance/externalModules/exportAssignTypes.ts` | Preserve a string export-assignment type through a relative CommonJS import-equals. |
| `should_preserve_numeric_export_type_given_relative_import_equals_when_compiling_sources` | `conformance/externalModules/exportAssignTypes.ts` | Preserve an exported number type through a relative CommonJS import-equals. |
| `should_preserve_boolean_export_type_given_relative_import_equals_when_compiling_sources` | `conformance/externalModules/exportAssignTypes.ts` | Preserve an exported boolean type through a relative CommonJS import-equals. |
| `should_preserve_array_element_type_given_relative_import_equals_when_compiling_sources` | `conformance/externalModules/exportAssignTypes.ts` | Preserve an exported array element type through a relative CommonJS import-equals. |
| `should_preserve_object_property_type_given_relative_import_equals_when_compiling_sources` | `conformance/externalModules/exportAssignTypes.ts` | Preserve an exported object's numeric property type through a relative CommonJS import-equals. |
| `should_preserve_any_value_type_given_relative_import_equals_when_compiling_sources` | `conformance/externalModules/exportAssignTypes.ts` | Accept an unannotated any export through a relative CommonJS import-equals. |
| `should_preserve_function_return_type_given_relative_import_equals_when_compiling_sources` | `conformance/externalModules/exportAssignTypes.ts` | Preserve an exported function's numeric return type through a relative CommonJS import-equals. |
| `should_infer_generic_call_result_given_relative_import_equals_when_compiling_sources` | `conformance/externalModules/exportAssignTypes.ts` | Infer the numeric result type of calling an exported generic function through a relative CommonJS import-equals. |
| `should_report_string_to_number_mismatch_given_relative_import_equals_when_compiling_sources` | `conformance/externalModules/exportAssignTypes.ts` | Report TS2322 when a string export-assignment value is assigned to number through a relative import-equals. |
| `should_load_type_reference_given_type_roots_configuration_when_running_compiler_cli` | `conformance/references/library-reference-1.ts` | Resolve a triple-slash type reference through configured type roots. |
| `should_report_jsdoc_argument_mismatch_given_number_parameter_when_checking_javascript` | `conformance/jsdoc/checkJsdocParamTag1.ts` | Report TS2345 for a mismatched JSDoc-typed argument. |
| `should_report_missing_string_member_given_jsdoc_callback_parameter_when_checking_javascript` | `conformance/jsdoc/callbackTag1.ts` | Contextually type a JSDoc callback parameter as string and report TS2551 for toFixed. |
| `should_report_extra_property_given_jsdoc_satisfies_tag_when_checking_javascript` | `conformance/jsdoc/checkJsdocSatisfiesTag9.ts` | Report TS2353 for an extra object property checked by a JSDoc `@satisfies` tag. |
| `should_report_no_matching_constructor_given_jsdoc_overloads_when_checking_javascript` | `conformance/jsdoc/overloadTag2.ts` | Report TS2769 when a JavaScript constructor call matches none of its JSDoc overloads. |
| `should_report_missing_jsdoc_implemented_member_given_javascript_class_when_compiling_sources` | `conformance/jsdoc/jsdocImplements_interface.ts` | Report TS2420 when a JSDoc-annotated JavaScript class omits an interface member. |
| `should_report_missing_member_given_jsdoc_this_type_when_checking_javascript` | `conformance/jsdoc/thisTag3.ts` | Report TS2339 when a function accesses a member absent from its JSDoc `@this` type. |
| `should_report_template_constraint_mismatch_given_jsdoc_typedef_when_checking_javascript` | `conformance/jsdoc/checkJsdocTypeTag4.ts` | Report TS2344 when a JSDoc typedef supplies a type argument outside its `@template` constraint. |
| `should_report_imported_jsdoc_type_mismatch_given_adjacent_declaration_when_compiling_sources` | `conformance/jsdoc/importTag19.ts` | Resolve a JSDoc `@import` type and report TS2322 for an incompatible property. |
| `should_report_missing_boolean_member_given_jsdoc_type_predicate_when_checking_javascript` | `conformance/jsdoc/returnTagTypeGuard.ts` | Narrow to boolean from a JSDoc type predicate, then report TS2339 for a missing boolean member. |
| `should_report_jsdoc_template_return_mismatch_given_number_argument_when_checking_javascript` | `conformance/jsdoc/jsdocTemplateTag.ts` | Infer a JSDoc template return as number and report TS2322 when assigning it to string. |
| `should_report_jsdoc_typedef_property_mismatch_given_annotated_object_when_checking_javascript` | `conformance/jsdoc/typedefTagNested.ts` | Report TS2322 when a JavaScript object property does not match its JSDoc `@typedef` type. |
| `should_report_required_parameter_after_optional_jsdoc_parameter_given_checked_javascript_when_checking_types` | `conformance/jsdoc/checkJsdocOptionalParamOrder.ts` | Report TS1016 when a required parameter follows an optional JSDoc parameter in checked JavaScript. |
| `should_report_return_type_mismatch_given_jsdoc_returns_tag_when_checking_types` | `conformance/jsdoc/checkJsdocReturnTag2.ts` | Report TS2322 when a JavaScript function returns a number against `@returns {string}`. |
| `should_emit_jsdoc_parameter_type_given_annotated_javascript_function_when_emitting_declarations` | `conformance/jsdoc/declarations/jsDeclarationsFunctionJSDoc.ts` | Preserve the JSDoc parameter type in declaration output. |
| `should_report_comma_operator_in_jsx_expression_given_tsx_source_when_building_syntax_tree` | `conformance/jsx/jsxParsingError1.tsx` | Report TS18007 for a comma operator in a JSX expression container. |
| `should_merge_interface_members_given_duplicate_declarations_when_checking_types` | `conformance/interfaces/declarationMerging/mergeTwoInterfaces.ts` | Combine the members of merged interface declarations. |
| `should_reject_legacy_out_file_given_allow_js_project_when_running_compiler_cli` | `projects/jsFileCompilation/DifferentNamesNotSpecifiedWithAllowJs/a.ts` | Report TS5102 for the removed `outFile` option. |
| `should_write_declarations_to_declaration_dir_given_declaration_dir_project_option_when_running_compiler_cli` | `projects/declarationDir` | Write declaration files under the configured declaration directory. |
| `should_route_declarations_to_declaration_dir_given_out_dir_when_running_compiler_cli` | `projects/declarationDir/subfolder/b.ts` | Keep declarations under `declarationDir` when `outDir` is also configured. |
| `should_report_source_outside_root_dir_given_project_root_dir_option_when_running_compiler_cli` | `projects/rootDirectory/FolderA/FolderB/fileB.ts` | Report TS6059 for a source outside the configured root directory. |
| `should_emit_inline_source_map_given_inline_source_map_option_when_compiling` | `transpile/jsWithInlineSourceMapBasic.ts` | Include an inline source-map data URL in emitted JavaScript. |
| `should_emit_declaration_map_given_declaration_map_option_when_compiling` | `transpile/declarationBasicSyntax.ts` | Write a declaration map alongside the declaration output. |
| `should_infer_cross_file_declaration_type_given_transitive_factory_call_when_compiling_sources` | `transpile/declarationCrossFileInferences.ts` | Infer `import("./defines.js").A` in declaration output. |
| `should_report_implicit_any_given_unannotated_parameter_when_compiling_project` | `compiler/noImplicitAnyFunctions.ts` | Report TS7006 with `noImplicitAny`. |
| `should_report_implicit_any_given_unannotated_parameter_when_using_cli_option` | `compiler/noImplicitAnyFunctions.ts` | Report TS7006 with the `--noImplicitAny` CLI option. |
| `should_report_unused_local_given_unread_binding_when_compiling_project` | `compiler/noUnusedLocals_writeOnly.ts` | Report TS6133 with `noUnusedLocals`. |
| `should_report_unused_parameter_given_unread_parameter_when_compiling_project` | `compiler/unusedSingleParameterInFunctionDeclaration.ts` | Report TS6133 with `noUnusedParameters`. |
| `should_require_override_modifier_given_overriding_member_when_compiling_project` | `conformance/override/override1.ts` | Report TS4114 with `noImplicitOverride`. |
| `should_report_switch_fallthrough_given_nonterminating_case_when_compiling_project` | `compiler/fallFromLastCase2.ts` | Report TS7029 with `noFallthroughCasesInSwitch`. |
| `should_report_explicit_undefined_given_exact_optional_property_when_compiling_project` | `compiler/deleteExpressionMustBeOptional_exactOptionalPropertyTypes.ts` | Report TS2375 for explicitly undefined optional property with `exactOptionalPropertyTypes`. |
| `should_report_possible_undefined_given_array_index_when_compiling_project` | `conformance/pedantic/noUncheckedIndexedAccess.ts` | Report TS2322 for a possibly undefined indexed value with `noUncheckedIndexedAccess`. |
| `should_report_possible_null_given_nullable_object_property_read_when_checking_types` | `conformance/types/nonPrimitive/nonPrimitiveStrictNull.ts` | Report TS18047 when a nullable object value is read without narrowing. |
| `should_report_nullish_rhs_possibly_null_given_nullable_left_operand_when_checking_types` | `conformance/expressions/nullishCoalescingOperator/nullishCoalescingOperator4.ts` | Report TS18049 when a possibly nullish value is dereferenced on the right side of `??`. |
| `should_report_unknown_type_property_access_given_unchecked_value_when_checking_types` | `conformance/types/unknown/unknownType1.ts` | Report TS18046 for property access on a value of type `unknown`. |
| `should_emit_null_type_annotation_given_default_null_expression_when_emitting_declarations` | `conformance/declarationEmit/exportDefaultExpressionComments.ts` | Emit an explicit `null` type annotation for an exported null default under strict null checking. |
| `should_require_type_only_import_given_verbatim_module_syntax_when_compiling_project` | `compiler/isolatedModulesShadowGlobalTypeNotValue.ts` | Report TS1484 when a regular import is used only as a type under `verbatimModuleSyntax`. |
| `should_report_unknown_catch_binding_given_unannotated_catch_when_compiling_project` | `compiler/useUnknownInCatchVariables01.ts` | Report TS2322 for an unknown catch binding with `useUnknownInCatchVariables`. |
| `should_report_missing_return_given_partial_return_path_when_compiling_project` | `compiler/noImplicitReturnsWithoutReturnExpression.ts` | Report TS7030 with `noImplicitReturns`. |
| `should_report_source_reference_to_itself_given_triple_slash_directive_when_running_compiler_cli` | `projects/InvalidReferences/main.ts` | Report TS1006 for a source file that references itself. |
| `should_report_missing_source_references_given_unresolved_triple_slash_paths_when_running_compiler_cli` | `projects/InvalidReferences/main.ts` | Report TS6053 for unresolved triple-slash source paths. |
| `should_parse_string_index_signature_given_interface_member_when_building_syntax_tree` | `conformance/additionalChecks/noPropertyAccessFromIndexSignature1.ts` | Accept a string index signature in an interface. |
| `should_parse_numeric_index_signature_given_interface_member_when_building_syntax_tree` | `conformance/types/objectTypeLiteral/indexSignatures/numericIndexingResults.ts` | Accept a numeric index signature in an interface. |
| `should_parse_numeric_index_signature_in_object_type_given_numeric_key_when_building_syntax_tree` | `conformance/types/objectTypeLiteral/indexSignatures/numericIndexingResults.ts` | Accept a numeric index signature in an object type literal. |
| `should_parse_numeric_index_signature_given_class_member_when_building_syntax_tree` | `conformance/types/objectTypeLiteral/indexSignatures/numericIndexingResults.ts` | Accept a numeric index signature in a class. |
| `should_report_index_signature_property_access_given_option_when_compiling_project` | `conformance/additionalChecks/noPropertyAccessFromIndexSignature1.ts` | Report TS4111 when dot access reads a key supplied only by an index signature under `noPropertyAccessFromIndexSignature`. |
| `should_allow_bracket_access_given_index_signature_property_and_option_when_compiling_project` | `conformance/additionalChecks/noPropertyAccessFromIndexSignature1.ts` | Allow bracket access to an index-signature-only key when `noPropertyAccessFromIndexSignature` is enabled. |
| `should_allow_declared_property_dot_access_given_index_signature_and_option_when_compiling_project` | `conformance/additionalChecks/noPropertyAccessFromIndexSignature1.ts` | Allow dot access to an explicitly declared property even when the type also has an index signature. |
| `should_parse_async_generator_declaration_given_es2018_function_syntax_when_building_tree` | `conformance/asyncGenerators/asyncGeneratorParameterEvaluation.ts` | Parse an adjacent `async function*` declaration in the ES2018 configuration. |
| `should_reject_project_option_mixed_with_source_files_given_project_and_file_arguments_when_running_compiler_cli` | `tsc/ignoreConfig/mixing-project-and-files.js` | Report TS5042 when `--project` and source files are passed together. |
| `should_report_invalid_interface_extension_given_union_base_when_checking_types` | `conformance/interfaces/interfaceDeclarations/interfaceExtendsObjectIntersectionErrors.ts` | Report TS2312 when an interface extends a union with non-static members. |
| `should_report_incompatible_overload_implementation_given_function_signature_when_checking_types` | `conformance/functions/functionOverloadCompatibilityWithVoid01.ts` | Report TS2394 when an overload signature is incompatible with its implementation. |
| `should_parse_function_overload_signature_given_following_implementation_when_building_syntax_tree` | `conformance/functions/functionOverloadCompatibilityWithVoid02.ts` | Parse an overload signature followed by its implementation. |
| `should_accept_void_overload_given_value_returning_implementation_when_checking_types` | `conformance/functions/functionOverloadCompatibilityWithVoid02.ts` | Accept a value-returning implementation for a `void` overload. |
| `should_parse_tagged_template_as_single_statement_given_tag_expression_when_building_tree` | `conformance/es6/templates/taggedTemplateStringsWithTagsTypedAsAny.ts` | Parse a tagged template expression as one expression statement. |
| `should_preserve_labeled_break_given_labeled_while_loop_when_emitting_javascript` | `conformance/statements/labeledStatements` | Preserve the labeled loop and its labeled `break` in JavaScript output. |
| `should_resolve_versioned_package_types_given_types_versions_mapping_when_running_compiler_cli` | `conformance/moduleResolution/typesVersions.multiFile.ts` | Resolve declarations selected by a package `typesVersions` mapping. |
| `should_resolve_browser_export_condition_given_custom_conditions_when_running_compiler_cli` | `conformance/moduleResolution/customConditions.ts` | Resolve the `browser` package-export condition when it is enabled in `customConditions`. |
| `should_report_invalid_allow_importing_ts_extensions_given_emit_when_running_cli` | `typescript-go/internal/compiler/program.go`, `conformance/moduleResolution/allowImportingTsExtensions.ts` | Report TS5096 when `allowImportingTsExtensions` is enabled while JavaScript emission remains enabled. |
| `should_report_ts_extension_import_given_option_disabled_when_running_cli` | `typescript-go/internal/checker/checker.go`, `conformance/moduleResolution/allowImportingTsExtensions.ts` | Report TS5097 for a `.ts` import when `allowImportingTsExtensions` is disabled. |
| `should_allow_unresolved_side_effect_import_given_option_disabled_when_compiling_project` | `compiler/sideEffectImports1.ts` | Accept and emit a missing side-effect import when `noUncheckedSideEffectImports` is false. |
| `should_allow_unresolved_relative_side_effect_import_given_option_disabled_when_compiling_project` | `compiler/sideEffectImports1.ts` | Accept and emit an unresolved extensionless relative side-effect import when the option is false. |
| `should_allow_unresolved_javascript_side_effect_import_given_option_disabled_when_compiling_project` | `compiler/sideEffectImports1.ts` | Accept and emit an unresolved relative `.js` side-effect import when the option is false. |
| `should_rewrite_relative_typescript_import_given_rewrite_option_when_emitting_javascript` | `conformance/externalModules/rewriteRelativeImportExtensions/emit.ts` | Rewrite a relative `.ts` import to `.js` in emitted JavaScript. |
| `should_rewrite_mts_import_to_mjs_given_rewrite_option_when_emitting_javascript` | `conformance/externalModules/rewriteRelativeImportExtensions/emit.ts` | Rewrite a relative `.mts` import to `.mjs` in emitted JavaScript. |
| `should_rewrite_cts_import_to_cjs_given_rewrite_option_when_emitting_javascript` | `conformance/externalModules/rewriteRelativeImportExtensions/emit.ts` | Rewrite a relative `.cts` import to `.cjs` in emitted JavaScript. |
| `should_rewrite_tsx_import_to_jsx_given_rewrite_option_when_emitting_javascript` | `conformance/externalModules/rewriteRelativeImportExtensions/emit.ts` | Rewrite a relative `.tsx` import to `.jsx` in emitted JavaScript. |
| `should_rewrite_side_effect_typescript_import_given_rewrite_option_when_emitting_javascript` | `conformance/externalModules/rewriteRelativeImportExtensions/emit.ts` | Rewrite a relative `.ts` side-effect import to `.js` in emitted JavaScript. |
| `should_rewrite_typescript_star_reexport_given_rewrite_option_when_emitting_javascript` | `conformance/externalModules/rewriteRelativeImportExtensions/emit.ts` | Rewrite a relative `.ts` star re-export to `.js` in emitted JavaScript. |
| `should_retain_following_class_given_unexpected_top_level_brace_when_parsing` | `conformance/parser/ecmascript5/ErrorRecovery/SourceUnits/parserErrorRecovery_SourceUnit1.ts` | Recover from an unexpected top-level brace and retain the following class declaration. |
| `should_suppress_javascript_output_given_type_error_and_no_emit_on_error_project_option_when_running_compiler_cli` | `compiler/noEmitOnError.ts` | Suppress JavaScript output when a project has a type error and `noEmitOnError`. |
| `should_assign_derived_instance_to_base_return_type_given_class_inheritance_when_checking_types` | `conformance/types/typeRelationships/assignmentCompatibility/unionTypesAssignability.ts` | Accept a derived class instance where its base class is expected. |
| `should_report_required_return_value_given_non_void_function_without_return_when_checking_types` | `compiler/missingReturnStatement.ts` | Report TS2355 when a non-void function has no return value. |
| `should_insert_return_terminator_given_line_break_before_expression_when_emitting_javascript` | `conformance/statements/returnStatements/returnStatementNoAsiAfterTransform.ts` | Preserve automatic semicolon insertion after `return` when a line terminator precedes the following expression. |
| `should_report_call_on_non_callable_value_given_number_expression_when_checking_types` | `compiler/callOnInstance.ts` | Report TS2349 when a numeric value is called as a function. |
| `should_report_missing_return_path_given_non_void_function_with_partial_return_when_checking_types` | `compiler/exhaustiveSwitchImplicitReturn.ts` | Report TS2366 when a non-void function has a path that falls through without returning. |
| `should_reject_private_member_redeclaration_given_derived_class_override_when_checking_types` | `compiler/inheritanceGrandParentPrivateMemberCollision.ts` | Report TS2415 when a derived class redeclares an inherited private member. |
| `should_report_static_member_suggestion_given_unqualified_static_member_reference_when_checking_types` | `compiler/accessInstanceMemberFromStaticMethod01.ts` | Report TS2662 with a suggestion when an unqualified name refers to a class static member. |
| `should_report_static_member_access_suggestion_given_instance_property_read_when_checking_types` | `compiler/classStaticPropertyAccess.ts` | Report TS2576 with a suggestion to access a static member through its class after an instance property read. |
| `should_report_unreachable_statement_given_allow_unreachable_code_false_when_compiling_project` | `compiler/reachabilityChecks11.ts` | Report TS7027 for a statement after an unconditional return when `allowUnreachableCode` is false. |
| `should_parse_ambient_namespace_given_namespace_declaration_when_building_syntax_tree` | `projects/declareVariableCollision/decl.d.ts` | Parse an ambient `declare namespace` in a project declaration file. |
| `should_parse_type_only_star_reexport_given_export_declaration_when_building_syntax_tree` | `conformance/externalModules/typeOnly/exportNamespace4.ts` | Parse `export type * from './a'` without syntax diagnostics. |
| `should_parse_namespace_reexport_given_identifier_alias_when_building_syntax_tree` | `conformance/externalModules/typeOnly/exportNamespace2.ts` | Parse `export * as api from './module'` as a namespace re-export. |
| `should_emit_namespace_reexport_given_commonjs_module_when_compiling_sources` | `conformance/externalModules/typeOnly/exportNamespace2.ts` | Lower a namespace re-export to a CommonJS `__importStar(require(...))` assignment. |
| `should_parse_parameter_decorator_given_class_method_parameter_when_building_syntax_tree` | `conformance/decorators/class/method/parameter/decoratorOnClassMethodParameter1.ts` | Parse a legacy decorator on a class method parameter. |
| `should_emit_literal_return_type_given_local_type_query_when_emitting_declarations` | `transpile/declarationNotInScopeTypes.ts` | Emit `export declare function two(): "";` for a local `typeof` query. |
| `should_preserve_value_type_query_given_type_alias_when_emitting_declarations` | `conformance/types/specifyingTypes/typeQueries/circularTypeofWithVarOrFunc.ts` | Preserve `typeof value` in a type alias declaration. |
| `should_accept_class_type_query_given_class_name_when_checking_types` | `conformance/types/specifyingTypes/typeQueries/typeQueryOnClass.ts` | Accept a query for the static side of a class. |
| `should_accept_instance_type_query_given_instance_name_when_checking_types` | `conformance/types/specifyingTypes/typeQueries/typeQueryOnClass.ts` | Accept a query for the instance type of a class value. |
| `should_accept_qualified_type_query_given_class_member_when_checking_types` | `conformance/types/specifyingTypes/typeQueries/typeQueryWithReservedWords.ts` | Accept a qualified query for a class prototype method. |
| `should_parse_this_member_type_query_given_class_property_when_building_syntax_tree` | `conformance/types/specifyingTypes/typeQueries/typeofThis.ts` | Parse `typeof this.member` in a class property annotation. |
| `should_report_circular_type_query_given_self_referential_variable_annotation_when_checking_types` | `conformance/types/specifyingTypes/typeQueries/recursiveTypesWithTypeof.ts` | Report TS2502 for a circular variable type query. |
| `should_accept_enum_member_type_query_given_enum_member_when_checking_types` | `conformance/types/specifyingTypes/typeQueries/typeofANonExportedType.ts` | Accept a query for an enum member value. |
| `should_report_type_only_query_given_type_alias_when_checking_types` | `conformance/types/specifyingTypes/typeQueries/typeofTypeParameter.ts` | Report TS2693 when a type-only symbol is used as a value query. |
| `should_reject_numeric_type_query_target_given_numeric_operand_when_building_syntax_tree` | `conformance/types/specifyingTypes/typeQueries/invalidTypeOfTarget.ts` | Report TS1003 when a numeric literal is used as a type-query target. |
| `should_lower_instance_field_given_es2015_target_when_emitting_javascript` | `conformance/classes/propertyMemberDeclarations/instanceMemberInitialization.ts` | Emit an instance field initializer inside the generated constructor for ES2015. |
| `should_lower_typed_array_instance_field_given_es2015_target_when_emitting_javascript` | `compiler/2dArrays.ts` | Lower an array-typed instance field into a constructor assignment for ES2015. |
| `should_lower_static_field_given_es2015_target_when_emitting_javascript` | `conformance/classes/propertyMemberDeclarations/staticMemberInitialization.ts` | Emit a static field initializer after the class for ES2015. |
| `should_parse_array_binding_pattern_given_for_of_statement_when_building_syntax_tree` | `conformance/es6/for-ofStatements/for-of38.ts` | Parse an array binding pattern as a for-of initializer. |
| `should_preserve_array_binding_pattern_given_for_of_statement_when_emitting_javascript` | `conformance/es6/for-ofStatements/for-of38.ts` | Preserve the array binding pattern in ES2015 JavaScript output. |
| `should_parse_function_type_alias_given_typed_parameter_when_building_syntax_tree` | `conformance/types/specifyingTypes/typeLiterals/functionLiteral.ts` | Parse a function type alias with a typed parameter and return type without syntax diagnostics. |
| `should_parse_call_signature_type_literal_given_typed_parameter_when_building_syntax_tree` | `conformance/types/specifyingTypes/typeLiterals/functionLiteral.ts` | Parse an object type literal containing a typed call signature without syntax diagnostics. |
| `should_parse_construct_signature_given_typed_parameter_when_building_syntax_tree` | `conformance/types/specifyingTypes/typeLiterals/functionLiteral.ts` | Parse a construct signature with a typed parameter and return type without syntax diagnostics. |
| `should_parse_generic_call_signature_given_type_parameter_when_building_syntax_tree` | `conformance/types/specifyingTypes/typeLiterals/functionLiteral.ts` | Parse a generic call signature in an object type literal without syntax diagnostics. |
| `should_parse_constrained_infer_given_conditional_type_when_building_syntax_tree` | `conformance/types/conditional/inferTypesWithExtends1.ts` | Parse a constrained `infer U extends ...` type parameter in a conditional type. |
| `should_parse_assertion_signature_given_function_declaration_when_building_syntax_tree` | `conformance/controlFlow/assertionTypePredicates1.ts` | Parse an assertion signature on a function declaration. |
| `should_parse_method_signature_in_object_type_given_typed_parameter_when_building_syntax_tree` | `conformance/types/objectTypeLiteral/methodSignatures/functionLiterals.ts` | Parse a named method signature in an object type literal without syntax diagnostics. |
| `should_parse_optional_property_in_object_type_given_question_mark_when_building_syntax_tree` | `conformance/types/objectTypeLiteral/methodSignatures/objectTypesWithOptionalProperties.ts` | Parse an optional property in an object type literal without syntax diagnostics. |
| `should_parse_string_index_signature_in_object_type_given_string_key_when_building_syntax_tree` | `conformance/types/objectTypeLiteral/indexSignatures/stringIndexingResults.ts` | Parse a string index signature in an object type literal without syntax diagnostics. |
| `should_parse_readonly_property_in_object_type_given_readonly_modifier_when_building_syntax_tree` | `conformance/controlFlow/controlFlowAliasing.ts` | Parse a readonly property in an object type literal without syntax diagnostics. |
| `should_parse_ambient_module_given_quoted_module_declaration_when_building_syntax_tree` | `projects/NestedDeclare/consume.ts` | Parse a quoted ambient module declaration without syntax diagnostics. |
| `should_emit_map_root_source_mapping_url_given_out_dir_and_source_map_when_running_compiler_cli` | `projects/outputdir_simple/test.ts` | Emit the configured relative `mapRoot` URL in JavaScript when source maps and `outDir` are enabled. |
| `should_emit_javascript_given_transitive_import_equals_project_when_running_compiler_cli` | `projects/privacyCheck-IndirectReference/test.ts` | Emit the test module's `require("externalModule")` for a project with a transitive import-equals dependency. |
| `should_parse_global_namespace_export_given_ambient_declaration_when_building_syntax_tree` | `projects/declarations_ExportNamespace/decl.d.ts` | Accept `export as namespace` in an ambient declaration file. |
| `should_resolve_ambient_global_namespace_type_given_export_as_namespace_when_emitting_declarations` | `projects/declarations_ExportNamespace/decl.d.ts`, `projects/declarations_ExportNamespace/useModule.ts` | Resolve the ambient global namespace type and preserve it in declaration output. |
| `should_report_removed_out_file_option_given_project_configuration_when_running_compiler_cli` | `projects/outputdir_singleFile/test.ts` | Report TS5102 when a project config uses TypeScript 7's removed `outFile` option. |
| `should_report_removed_base_url_given_project_configuration_when_running_compiler_cli` | `typescript-go/internal/compiler/program.go` | Report TS5102 and suggest `paths` when a project config uses TypeScript 7's removed `baseUrl` option. |
| `should_report_removed_node10_resolution_given_project_config_when_running_cli` | `typescript-go/internal/compiler/program.go` | Report TS5108 when a project config selects removed `moduleResolution=node10`. |
| `should_report_removed_amd_module_given_project_config_when_running_cli` | `typescript-go/internal/compiler/program.go` | Report TS5108 when a project config selects removed `module=AMD`. |
| `should_report_removed_system_module_given_project_config_when_running_cli` | `typescript-go/internal/compiler/program.go` | Report TS5108 when a project config selects removed `module=System`. |
| `should_report_removed_umd_module_given_project_config_when_running_cli` | `typescript-go/internal/compiler/program.go` | Report TS5108 when a project config selects removed `module=UMD`. |
| `should_report_removed_always_strict_false_given_project_config_when_running_cli` | `typescript-go/internal/compiler/program.go` | Report TS5108 when a project config sets removed `alwaysStrict=false`. |
| `should_report_removed_es_module_interop_false_given_project_config_when_running_cli` | `typescript-go/internal/compiler/program.go` | Report TS5108 when a project config sets removed `esModuleInterop=false`. |
| `should_report_removed_synthetic_default_imports_false_given_project_config_when_running_cli` | `typescript-go/internal/compiler/program.go` | Report TS5108 when a project config sets removed `allowSyntheticDefaultImports=false`. |
| `should_report_removed_downlevel_iteration_given_project_config_when_running_cli` | `typescript-go/internal/compiler/program.go` | Report TS5102 when a project config sets removed `downlevelIteration`. |
| `should_reject_removed_project_target_given_es5_configuration_when_running_compiler_cli` | `projects/decoratorMetadata/emitDecoratorMetadataCommonJSIsolatedModule/main.ts` | Report TS5108 for the TypeScript 7 project configuration's removed ES5 target. |
| `should_parse_import_equals_given_namespace_declaration_when_building_syntax_tree` | `projects/NestedLocalModule-SimpleCase/test1.ts` | Parse import-equals syntax nested inside a namespace without syntax diagnostics; TypeScript 7 later reports TS1147 semantically. |
| `should_report_invalid_module_reference_given_import_equals_inside_namespace_when_compiling` | `projects/NestedLocalModule-WithRecursiveTypecheck/test1.ts` | Report TS1147 for an import-equals module reference inside a namespace. |
| `should_report_ts1147_given_import_equals_inside_exported_namespace_when_running_compiler_cli` | `projects/privacyCheck-ImportInParent/test.ts` | Report TS1147 for import-equals inside an exported namespace. |
| `should_report_ts1147_given_import_equals_inside_global_namespace_when_running_compiler_cli` | `projects/privacyCheck-InsideModule/testGlo.ts` | Report TS1147 for import-equals inside a global namespace. |
| `should_report_ts1147_given_namespace_import_equals_when_running_compiler_cli` | `projects/ext-int-ext/internal2.ts` | Report TS1147 for namespace-local import-equals in the mixed external/internal project. |
| `should_report_import_assignment_given_ecmascript_module_when_compiling_sources` | `projects/VisibilityOfCrosssModuleTypeUsage/commands.ts` | Report TS1202 for an import assignment when the output module kind is ECMAScript. |
| `should_report_implicit_any_return_given_unannotated_ambient_function_when_compiling` | `projects/relative-nested-ref/decl.d.ts` | Report TS7010 for an ambient function with no return annotation. |
| `should_parse_template_literal_type_given_string_substitution_when_building_syntax_tree` | `conformance/types/literal/templateLiteralTypes8.ts` | Parse a template literal type containing a string substitution without syntax diagnostics. |
| `should_parse_indexed_access_type_given_property_key_when_building_syntax_tree` | `conformance/types/keyof/keyofAndIndexedAccess.ts` | Parse an indexed access type using a string literal property key without syntax diagnostics. |
| `should_reject_unknown_key_given_keyof_type_when_checking_types` | `conformance/types/keyof/keyofAndIndexedAccess.ts` | Report TS2322 when a string is not a member of a `keyof` property-name union. |
| `should_parse_tuple_type_given_two_element_types_when_building_syntax_tree` | `conformance/types/tuple/tupleElementTypes1.ts` | Parse a fixed-length tuple type with two element types without syntax diagnostics. |
| `should_parse_infer_type_given_array_conditional_type_when_building_syntax_tree` | `conformance/types/conditional/inferTypes1.ts` | Parse `infer` in an array conditional type without syntax diagnostics. |
| `should_parse_static_initialization_block_given_class_member_when_building_syntax_tree` | `conformance/classes/classStaticBlock/classStaticBlock1.ts` | Parse a static initialization block within a class declaration. |
| `should_emit_static_initialization_block_given_es2022_target_when_emitting_javascript` | `conformance/classes/classStaticBlock/classStaticBlock1.ts` | Preserve a static initialization block in ES2022 JavaScript output. |
| `should_parse_abstract_class_given_abstract_modifier_when_building_syntax_tree` | `conformance/classes/classDeclarations/classAbstractKeyword/classAbstractSingleLineDecl.ts` | Parse the `abstract` modifier on a class declaration without syntax diagnostics. |
| `should_parse_abstract_method_signature_given_abstract_class_when_building_syntax_tree` | `conformance/classes/classDeclarations/classAbstractKeyword/classAbstractGeneric.ts` | Parse an abstract method signature into a class method node. |
| `should_report_ts2715_given_abstract_property_access_in_constructor_when_checking_types` | `compiler/abstractPropertyInConstructor.ts` | Report TS2715 when a class constructor accesses one of its abstract properties. |
| `should_erase_override_modifier_given_overriding_method_when_emitting_javascript` | `conformance/override/override1.ts` | Erase the `override` modifier while emitting the overriding method as JavaScript. |
| `should_emit_source_root_in_source_map_given_out_dir_and_source_map_when_running_compiler_cli` | `projects/outputdir_simple/test.ts`, `projects/outputdir_subfolder/test.ts` | Write the normalized `sourceRoot` path into external source-map JSON. |
| `should_preserve_referenced_source_subdirectory_given_out_dir_project_option_when_running_compiler_cli` | `projects/outputdir_subfolder/test.ts`, `projects/outputdir_subfolder/ref/m1.ts`, `projects/outputdir_mixed_subfolder/ref/m1.ts` | Emit a triple-slash referenced file below its matching `outDir` subdirectory. |
| `should_resolve_static_triple_slash_reference_given_global_type_when_compiling_project` | `projects/reference-path-static/test.ts`, `projects/reference-path-static/lib.ts` | Resolve a `static` triple-slash path so the referenced global type is available. |
| `should_parse_import_assignment_given_require_import_when_building_syntax_tree` | `projects/outputdir_module_subfolder/test.ts`, `projects/outputdir_module_simple/test.ts`, `projects/declarations_GlobalImport/useModule.ts` | Parse import-equals syntax; TypeScript 7 reports separate module-kind or module-resolution diagnostics. |
| `should_parse_qualified_class_heritage_given_dotted_base_name_when_building_syntax_tree` | `projects/Quote'InName/m'ain.ts` | Parse a class extending the qualified name `test.ClassA` without syntax diagnostics. |
| `should_parse_qualified_implements_clause_given_dotted_interface_name_when_building_syntax_tree` | `conformance/externalModules/typeOnly/implementsClause.ts` | Parse `implements types.Component` without syntax diagnostics. |
| `should_parse_generic_class_heritage_given_type_argument_when_building_syntax_tree` | `conformance/parser/ecmascript5/Generics/parserGenericsInTypeContexts2.ts` | Parse a type argument on a class heritage type without syntax diagnostics. |
| `should_parse_qualified_interface_heritage_given_dotted_base_name_when_building_syntax_tree` | `conformance/externalModules/typeOnly/extendsClause.ts` | Parse an interface extending the qualified name `types.C` without syntax diagnostics. |
| `should_parse_multiple_interface_heritage_types_given_comma_separated_bases_when_building_syntax_tree` | `conformance/interfaces/interfaceDeclarations/interfaceWithMultipleBaseTypes.ts` | Parse an interface extending two comma-separated base interfaces without syntax diagnostics. |
| `should_parse_generic_function_declaration_given_type_parameter_when_building_syntax_tree` | `conformance/types/typeParameters/typeParameterLists/typeParameterUsedAsConstraint.ts` | Parse a generic function declaration with a type parameter without syntax diagnostics. |
| `should_parse_type_parameter_constraint_given_generic_function_when_building_syntax_tree` | `conformance/types/typeParameters/typeParameterLists/typeParameterUsedAsConstraint.ts` | Parse an `extends` constraint on a generic function type parameter without syntax diagnostics. |
| `should_parse_const_type_parameter_given_generic_function_when_building_syntax_tree` | `conformance/types/typeParameters/typeParameterLists/typeParameterConstModifiers.ts` | Parse a `const` modifier on a generic function type parameter without syntax diagnostics. |
| `should_parse_default_type_parameter_given_generic_function_when_building_syntax_tree` | `compiler/genericDefaults.ts` | Parse a default type argument on a generic function type parameter without syntax diagnostics. |
| `should_parse_default_type_parameter_given_generic_type_alias_when_building_syntax_tree` | `compiler/genericDefaults.ts` | Parse a default type argument on a generic type alias parameter without syntax diagnostics. |
| `should_parse_generic_class_declaration_given_type_parameter_when_building_syntax_tree` | `conformance/types/typeParameters/typeParameterLists/typeParameterUsedAsConstraint.ts` | Parse a generic class declaration without syntax diagnostics. |
| `should_parse_generic_interface_declaration_given_type_parameter_when_building_syntax_tree` | `conformance/types/typeParameters/typeParameterLists/typeParameterUsedAsConstraint.ts` | Parse a generic interface declaration without syntax diagnostics. |
| `should_parse_generic_type_alias_given_type_parameter_when_building_syntax_tree` | `conformance/async/es6/asyncAliasReturnType_es6.ts` | Parse a generic type alias without syntax diagnostics. |
| `should_parse_generic_arrow_function_given_type_parameter_when_building_syntax_tree` | `conformance/types/typeParameters/typeParameterLists/typeParametersAvailableInNestedScope.ts` | Parse a generic arrow function assigned to a variable without syntax diagnostics. |
| `should_parse_intersection_type_given_two_named_types_when_building_syntax_tree` | `conformance/types/intersection/operatorsAndIntersectionTypes.ts` | Parse an intersection type between two named types without syntax diagnostics. |
| `should_parse_conditional_type_given_extends_check_when_building_syntax_tree` | `conformance/types/conditional/conditionalTypes1.ts` | Parse a conditional type with an `extends` check and true/false branches without syntax diagnostics. |
| `should_parse_namespaced_jsx_tag_given_namespace_and_tag_when_building_syntax_tree` | `conformance/jsx/tsxNamespacedTagName1.tsx` | Parse a JSX tag whose name contains a namespace separator without syntax diagnostics. |
| `should_parse_covariant_type_parameter_given_out_variance_annotation_when_building_syntax_tree` | `conformance/types/typeParameters/typeParameterLists/varianceAnnotations.ts` | Parse an `out` variance annotation on a generic type parameter without syntax diagnostics. |
| `should_resolve_transitive_triple_slash_reference_given_nested_reference_when_compiling_project` | `projects/reference-1/main.ts`, `projects/reference-1/lib/classB.ts`, `projects/reference-1/lib/classA.ts` | Resolve a triple-slash reference reached through another referenced file. |
| `should_preserve_exported_namespace_given_namespace_declaration_when_emitting_declarations` | `projects/declarations_ImportedInPrivate/useModule.ts` | Preserve an exported namespace and its exported member in declaration output. |
| `should_report_common_source_directory_outside_project_given_out_dir_without_root_dir_when_compiling_project` | `projects/outputdir_module_multifolder/test.ts`, `projects/outputdir_module_multifolder_ref/m2.ts` | Report TS5011 when output root inference would escape the project root. |
| `should_resolve_sibling_type_given_triple_slash_reference_when_compiling_project` | `projects/outputdir_multifolder/test.ts`, `projects/outputdir_multifolder_ref/m2.ts` | Resolve a global type from a sibling project directory through a triple-slash reference. |

Next, convert the upstream compiler and project fixture inventory into a
reviewable feature matrix with representative source paths, expected
diagnostics, JavaScript, declaration output, and option variations. Add the
focused red Rust behavior tests from that matrix before implementing features;
keep a link from each Rust behavior test to the upstream fixture(s) it covers.
