# Compiler contract gaps

This document turns the ownership and phase boundaries in
[`architecture.md`](architecture.md) into implementation contracts that can be shared across
compiler work. It records the shape of the current Rust APIs and proposes interfaces; it does not
make TypeScript conformance or pass/fail claims. The compatibility target remains full TypeScript
7.0.2 compiler behavior, including syntax, checking, emit, configuration, resolution, project
references, incremental builds, and watch builds, as stated in
[`architecture.md:11-15`](architecture.md#L11).

## Contracts needed before parallel compiler work

| Contract | Current shape and evidence | Consumers and proposed contract |
| --- | --- | --- |
| Source identity | [`SourceFile`](../src/source_file.rs#L24) owns a path, text, and script kind, but has no compilation identity. The binder keys scopes by `PathBuf` ([`binder.rs:13-39`](../src/binder.rs#L13)); in-compilation module resolution compares normalized paths ([`module_resolver.rs:277-289`](../src/module_resolver.rs#L277)). Normalization is lexical ([`module_resolver.rs:322-339`](../src/module_resolver.rs#L322)). | Scanner, parser, binder, resolver, diagnostics, emit, and extensions need to refer to the same input independently of path spelling. Allocate a `FileId` per compilation input and keep path as file metadata. Record authored/generated origin and input order separately. Paths remain resolution/output names, not semantic identity. |
| Source locations and syntax identity | [`TextSpan`](../src/syntax/mod.rs#L20) contains only a file-local UTF-16 range; `Diagnostic` contains only code, message, and that range ([`syntax/mod.rs:47-83`](../src/syntax/mod.rs#L47)). [`SyntaxTree`](../src/syntax/mod.rs#L85) has no file ID or revision. The scanner tracks byte and UTF-16 offsets ([`scanner.rs:83-97`](../src/syntax/scanner.rs#L83)), while [`SourceText`](../src/source_text.rs#L50) exposes UTF-16 line lookup but no checked conversion to UTF-8 byte offsets ([`source_text.rs:83-98`](../src/source_text.rs#L83)). AST identity and ranges are uneven: a function has neither a full span nor ID ([`ast.rs:432-441`](../src/syntax/ast.rs#L432)); type aliases, interfaces, and property signatures have no span or ID ([`ast.rs:1016-1084`](../src/syntax/ast.rs#L1016)); variables expose only a name span ([`ast.rs:1086-1125`](../src/syntax/ast.rs#L1086)). | Parser, checker, emit, analyzers, navigation, and future code fixes need consistent node references and source locations. Use a file-local `TextRange` and a source-aware `SourceSpan { file: FileId, range: TextRange }`; assign `NodeId` values unique within a syntax tree and expose `NodeRef { file: FileId, node: NodeId }`. Keep TypeScript-facing offsets in UTF-16 units and provide checked UTF-16/UTF-8 conversions at `SourceText` boundaries. IDs need only be stable for a compilation snapshot now. |
| Semantic identity and query views | The binder's private [`SymbolTable`](../src/binder.rs#L13) uses path/name maps and string spellings; `TypeTarget` is a module path plus a string ([`binder.rs:30-39`](../src/binder.rs#L30)). Annotation resolution returns `Vec<String>` ([`binder.rs:203-219`](../src/binder.rs#L203)). The type system currently provides string-based helpers ([`type_system.rs:1-25`](../src/type_system.rs#L1)); the checker constructs per-program `HashMap<String, Vec<String>>` environments ([`type_checker.rs:36-67`](../src/type_checker.rs#L36), [`type_checker.rs:197-228`](../src/type_checker.rs#L197)). | Checker, declaration emit, analyzers, and generators need shared semantic results without exposing checker mutation or relying on type spellings as identity. Make a per-compilation `SemanticModel` own private symbol/type arenas and indexes. Expose a read-only `SemanticModelView` with queries such as `symbol_at(NodeRef)`, `type_of(NodeRef)`, `symbol(SymbolId) -> SymbolView`, and `type(TypeId) -> TypeView`. Use `SymbolId` and `TypeId` as snapshot-scoped handles; keep interning, scope construction, and relation solving internal. |
| File-attributed diagnostics | Scanner/parser/checker/resolver and extensions all return the same pathless [`Diagnostic`](../src/syntax/mod.rs#L47). Module-resolution diagnostics are constructed from an import span without file identity ([`module_resolver.rs:76-101`](../src/module_resolver.rs#L76)); unsupported generated paths likewise receive a zero-offset span without identifying the path ([`compiler.rs:253-265`](../src/compiler.rs#L253)). Compilation-wide diagnostics are sorted by span start only ([`compiler.rs:275-284`](../src/compiler.rs#L275)). | CLI formatting, IDE clients, analyzers, references, and multi-file conformance need unambiguous source locations where a source file exists, plus stable ordering. Give each diagnostic a `DiagnosticCode`, category, owned message, `primary: Option<SourceSpan>`, origin/phase, and optional related locations. Source-attributed diagnostics must carry `Some(SourceSpan)`; compiler-option and project diagnostics can omit a primary source span. Sort with a total key that includes input/generated file order when present, source offset, and a documented phase/tie-break order. Preserve TypeScript diagnostic codes where they are compatibility behavior; reserve extension-specific code policy for a deliberate decision rather than assigning values ad hoc. |
| Resolved options and module graph | [`CompilerOptions`](../src/compiler.rs#L75) currently stores target, module, declaration, emit-only, and strict-null-checks. [`ProjectConfig`](../src/project_config.rs#L10) carries only root files and a small subset of options. The CLI translates these directly before calling `compile_sources` ([`main.rs:148-214`](../src/main.rs#L148)). The in-memory resolver checks trees ([`module_resolver.rs:8-20`](../src/module_resolver.rs#L8)) while `resolve_from_disk` is a separate path-based helper ([`module_resolver.rs:249-255`](../src/module_resolver.rs#L249)). | Compiler core, project loading, module resolution, diagnostics, and emit need one normalized input contract. Add an immutable `CompilationRequest` containing source inputs, unresolved options, and project/host inputs; normalize it once into `ResolvedCompilerOptions`. Resolve imports into a per-compilation `ModuleGraph` keyed by `ModuleId`/`FileId`, with explicit resolution outcomes and diagnostics. Keep filesystem/configuration acquisition in the project adapter and pass results inward. |
| Generated and emitted file identity | [`GeneratedSource`](../src/generator.rs#L45) owns only path and text. The compiler parses each generated path and appends its tree without a path-collision check ([`compiler.rs:246-273`](../src/compiler.rs#L246)); emit derives `.js` and declaration paths from source paths ([`compiler.rs:375-428`](../src/compiler.rs#L375)). Neither `CompilationResult` nor emitted files retain source/generated provenance ([`compiler.rs:326-373`](../src/compiler.rs#L326)). | Generator integration, module resolution, emit, output writers, and project builds need to distinguish source identity from output destination. Give each admitted generated input a compiler-assigned `FileId` and `GeneratedBy { extension_id, output_index }` provenance. Validate normalized input and output paths before emit; define one collision policy that reports the conflict and deterministically excludes or resolves the colliding file. Preserve output ordering by compilation input order and extension registration/output order. Diagnostic number and message for collisions remain a compatibility decision. |
| Immutable extension snapshots | [`GenerationContext`](../src/generator.rs#L14) and [`AnalysisContext`](../src/analyzer.rs#L11) borrow immutable syntax-tree and diagnostic slices; generators return owned sources/diagnostics ([`generator.rs:45-117`](../src/generator.rs#L45)), and analyzers return owned diagnostics ([`analyzer.rs:5-9`](../src/analyzer.rs#L5)). They do not expose resolved options, a module graph, or semantic queries. The compiler creates one common generator context before generation, then parses/checks generated trees and constructs the analyzer context ([`compiler.rs:239-280`](../src/compiler.rs#L239)). | Generators need the same immutable authored snapshot, including options and useful semantic queries, and must return owned outputs without mutating that snapshot. Analyzers need one immutable final snapshot containing authored plus admitted generated sources and final semantic results. Define separate `GeneratorInputView` and `AnalysisView` over a common `CompilationSnapshot`; all registered generators share the former, and analyzers run after generated files have passed normal compiler phases. Keep extension code outside scanner, parser, binder, checker, and emit implementations. |
| Phase result boundaries | [`Compiler::compile_sources`](../src/compiler.rs#L224) currently performs parse, bind/check, generator execution, generated-source parsing, another bind/check, module checking, analyzer execution, sorting, and emit in one orchestration method ([`compiler.rs:224-300`](../src/compiler.rs#L224)). | Teams implementing phases need stable inputs/outputs rather than a shared edit hotspot. Introduce owned phase results such as `ParsedCompilation`, `CheckedCompilation` (trees plus `SemanticModel` and diagnostics), `GeneratedInputs`, `AnalyzedCompilation`, and `EmitResult`. Keep the compiler/project orchestrator responsible for sequencing; each phase consumes immutable prior results and owns its output. The names are proposals, not existing APIs. |

## Proposed shared Rust shapes

The following sketches describe contracts, not a request to freeze these exact public names:

```rust
pub struct FileId(u32);
pub struct NodeId(u32);
pub struct SymbolId(u32);
pub struct TypeId(u32);
pub struct ModuleId(u32);

pub struct TextRange {
    pub start: Utf16Offset,
    pub length: usize,
}

pub struct SourceSpan {
    pub file: FileId,
    pub range: TextRange,
}

pub struct NodeRef {
    pub file: FileId,
    pub node: NodeId,
}

pub struct CompilationRequest {
    pub sources: Vec<SourceInput>,
    pub options: CompilerOptions,
    pub project: Option<ProjectInput>,
}

pub struct CompilationSnapshot<'a> {
    pub sources: &'a [SyntaxTree],
    pub options: &'a ResolvedCompilerOptions,
    pub modules: &'a ModuleGraph,
    pub semantics: SemanticModelView<'a>,
    pub diagnostics: &'a [Diagnostic],
}
```

The semantic model should expose immutable, fallible queries over IDs. The parser and binder may
own mutable construction state during their phase, but later phases should only receive the
completed model or a view. Diagnostic constructors should require source identity for source-backed
locations; project/configuration diagnostics should be allowed to omit a primary span rather than
use a fake file-local zero span.

For generated inputs, use an owned result that carries the generator identity and output ordering
metadata, then let the compiler assign the final `FileId` after path validation. A source path is
not enough to distinguish a failed candidate, a collision, or an admitted virtual file. Emitted
files should retain the originating `FileId` and an output kind so project writers can detect
destination collisions before writing.

## Scope and sequencing

### Establish now

1. Agree on `FileId`, `NodeId`, `SourceSpan`, and checked source-position conversions. These unblock
   diagnostics, parser/checker parallelism, semantic queries, and generated-file ownership.
2. Agree on `SymbolId`/`TypeId` and the immutable semantic query surface before expanding checker,
   declaration emit, or semantic extension APIs. The implementation may grow the type algebra over
   time while preserving those handles and queries.
3. Normalize compiler options and define the request/module-graph boundary. Resolver, project
   loading, CLI, checker, and emit must consume the same resolved options and resolution results.
4. Define deterministic diagnostic ordering, generated-source provenance, input/output collision
   behavior, and immutable generator/analyzer snapshots. These are shared pipeline contracts and
   should be decided before parallel hook or emit work.
5. Split orchestration at owned phase results so scanner/parser, binder/checker, resolution, hooks,
   and emit can evolve without all modifying `compile_sources`.

### Add when the corresponding compiler work begins

Full TypeScript 7 support remains the goal. It requires broadening the syntax tree beyond its
current statement and declaration shapes; for example, the present `Statement` variants are listed
in [`ast.rs:19-56`](../src/syntax/ast.rs#L19), and current alias/interface forms are a single type
annotation or a list of property signatures ([`ast.rs:1016-1084`](../src/syntax/ast.rs#L1016)). The
ID and semantic contracts above should support later generics, overload sets, control-flow and
contextual types, declaration merging, JavaScript checking, transforms, source maps, and other
TypeScript syntax without using names or path strings as semantic identity.

Project references, build/watch mode, and incremental compilation add revision management,
dependency invalidation, and reusable caches. Keep IDs snapshot-scoped for the initial compiler;
only introduce cross-revision identity guarantees when the incremental owner and invalidation rules
are defined. Build/project orchestration should own project graph state and caches, while each
single compilation continues to own its syntax and semantic phase state. Configuration and module
resolution must eventually cover the full TypeScript 7 option and package-resolution surface; the
current small option structs are not the target boundary.
