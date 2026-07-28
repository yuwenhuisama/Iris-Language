# Iris v1 Language Identity

Status: Iris v1 draft, frozen semantics.

IRIS-V1-IDENTITY-C001: This chapter defines the Iris v1 identity, compatibility promise, normative vocabulary, implementation boundary, version identities, and v1 deferrals. It MUST be read after [README.md](README.md) and before the grammar, runtime, type, package, library, migration, conformance, and traceability chapters.

IRIS-V1-IDENTITY-C002: This chapter MUST NOT define parser grammar, runtime dispatch algorithms, object layout, library byte formats, FFI call details, conformance-vector schemas, or migration recipes. Those details belong to later chapters and MUST cross-link back to this chapter when they rely on identity, compatibility, versioning, or deferral rules.

## Modern Successor Identity

IRIS-V1-IDENTITY-C003: Iris v1 is the modern successor to Legacy Iris. It MUST preserve the defining language identity recorded by frozen decisions, but it MUST NOT promise legacy source restoration, legacy syntax restoration, legacy parser behavior, legacy extension ABI behavior, or legacy implementation quirks. Every intentional incompatibility with Legacy Iris MUST be documented in [11-migration-divergence.md](11-migration-divergence.md).

IRIS-V1-IDENTITY-C004: Iris v1 MUST NOT be treated as an unrelated new language. A conforming specification, implementation, migration document, or conformance suite MUST preserve the core identity that every value is an object, operators are messages, dispatch is runtime-dynamic, composition uses single Class inheritance plus Modules and Contracts, lexical Closure and trailing-block programming remain core, Classes and Methods have runtime identity, constrained runtime mutation exists, any object may be raised as an exception, and the language is designed for host embedding and native extension support.

IRIS-V1-IDENTITY-C005: Iris v1 MUST NOT imitate Rust, Kotlin, TypeScript, Ruby, C#, or any host language surface when doing so would contradict a frozen Iris decision. Host implementation languages MAY influence implementation strategy only under the implementation-independence rules in this chapter and the native boundary rules in [09-native-host-ffi.md](09-native-host-ffi.md).

IRIS-V1-IDENTITY-N001: Informative note: The review archaeology describes Iris as a fully object-oriented message language with Module, Contract, block, dynamic Class mutation, and native-host affinity. This chapter makes that line normative through the frozen decisions rather than through the old implementation.

## Object, Message, And Static Promise Principles

IRIS-V1-IDENTITY-C006: Iris v1 uses objecthood as its root semantic model. Every runtime value MUST be an object and `Object` MUST be the top type as specified by the type chapter. Numeric values, singleton values, Class objects, Module objects, Contract objects, Method objects, Closure objects, Type objects, package objects, native handles, async Tasks, and diagnostic values MUST either have the identity behavior assigned by their later chapter or be explicitly classified as identity-less value objects there.

IRIS-V1-IDENTITY-C007: Iris v1 uses message sending as its behavioral model. Operators, ordinary Methods, named infix calls, property protocols, Contract-qualified sends, metaprogramming operations, async entry points, and FFI library-bound functions MUST be specified as message or callable protocols unless a later chapter explicitly defines a primitive bypass such as `same?`.

IRIS-V1-IDENTITY-C008: Iris v1 uses static promises to bound dynamic behavior. Static facts such as declared superclass, declared Contracts, visible member names and signatures, typed properties, generic constraints, native layout obligations, package identity, and API major identity MUST be preserved across dynamic mutation, open transactions, package upgrade, reflection, native binding, and optimization.

IRIS-V1-IDENTITY-C009: Iris v1 MUST keep ordinary message identity independent of static type choice. Static annotations, inferred types, expected return types, union branches, generic arguments, or Contract declarations MAY validate and narrow a send, but they MUST NOT silently choose a different ordinary selector or overload. Explicit Contract-qualified dispatch is the only v1 source mechanism that selects a Contract slot by Contract identity.

IRIS-V1-IDENTITY-C010: Iris v1 MUST NOT provide overload dispatch by static type, generic argument, union branch, expected result, declaration order, or implementation order. Same-name obligations that cannot share one compatible implementation MUST use the explicit qualified Contract mechanism specified in the type and runtime chapters.

IRIS-V1-IDENTITY-C011: Dynamic mutation in Iris v1 MUST be constrained by candidate transactions, validation against the static spine, MetaCapabilities, package and ReflectionPolicy permissions, and atomic publication. Invalid candidates MUST NOT partially mutate live Class, Module, Contract, package, or metadata state.

IRIS-V1-IDENTITY-C012: The static spine of a Class, Module, Contract, generic definition, package, or native binding MUST remain a durable promise for compiled code, typed bindings, reflection metadata, Host/native binding, and conformance. Dynamic surfaces MAY evolve only where later chapters define compatible evolution rules.

## Dynamic Behavior, Static Promises

IRIS-V1-IDENTITY-C013: The Iris v1 dynamic/static model is the following compact contract:

| Layer | What may change dynamically | What remains promised | Owning chapters |
| --- | --- | --- | --- |
| Objects and Methods | Compatible Method bodies, non-contract members, property accessors, Module composition, class-object state, metadata allowed by policy | Value objecthood, selector identity, Method identity rules, declared Class or Contract promises, identity classification | [03-runtime-object-model.md](03-runtime-object-model.md), [04-bindings-callables-control-flow.md](04-bindings-callables-control-flow.md) |
| Types and Contracts | Runtime checked views, Dynamic sends inside bounds, compatible implementations, reified generic materialization | No overload, invariant generics, Contract slot identity, declared requirements, `Dynamic<T>` boundary checks, `Never` flow meaning | [05-types-contracts-generics.md](05-types-contracts-generics.md) |
| Metaprogramming | Candidate Class or Module revisions, open transactions, decorators, reflection-visible dynamic members | Static spine, MetaCapabilities, ReflectionPolicy, synchronous non-escaping transaction boundaries, atomic publish or rollback | [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| Packages | Same-major compatible hot upgrade through explicit transaction | Package ID, API major identity, one active implementation per API major, stable nominal Type identity, old-revision retention while referenced | [08-modules-metaprogramming.md](08-modules-metaprogramming.md), [11-migration-divergence.md](11-migration-divergence.md) |
| Native and FFI | Extension implementation behind metadata, script-bound FFI library Methods, async completion through runtime posts | Stable C ABI boundary, opaque handles, runtime-thread affinity, explicit signatures, no raw managed pointer escape | [09-native-host-ffi.md](09-native-host-ffi.md) |
| Core and standard packages | Separately versioned official packages outside core ABI | Core runtime package set, stable format boundaries, explicit Contract-driven serialization, no hidden crypto promise from internal BLAKE3 | [10-serialization-standard-library.md](10-serialization-standard-library.md) |

IRIS-V1-IDENTITY-C014: The preceding model table is normative. Later chapters MUST refine the listed layers without contradicting the separation between dynamic evolution and static promises.

IRIS-V1-IDENTITY-EX001: Informative example: A package upgrade can replace a compatible Method body and publish a new active Class revision. A static caller that depended on the old Method signature still has the same static promise, while a later send uses the newly active revision if the upgrade commits.

## Normative Vocabulary And Editorial Rules

IRIS-V1-IDENTITY-C015: The RFC terms `MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, and `MAY` have the meanings assigned by [README.md](README.md). A paragraph in this chapter that contains a normative requirement MUST begin with an `IRIS-V1-IDENTITY-Cnnn` clause ID.

IRIS-V1-IDENTITY-C016: Informative notes, examples, implementation notes, and historical notes in this chapter MUST be labeled with the prefixes defined by [README.md](README.md). Informative text MUST NOT create a requirement, even when it explains a frozen decision.

IRIS-V1-IDENTITY-C017: Official Iris v1 terminology MUST use `Contract`, `MutableString`, `ReflectionPolicy`, logical Class, active revision, Method, BoundMethod, Closure, FFI, Host ABI, and `DEFERRED V1` as defined by [README.md](README.md). Historical terminology MAY appear only in clearly labeled historical or migration text.

IRIS-V1-IDENTITY-C018: Later chapters MUST use exact cross-links to this chapter for compatibility promise, implementation independence, semantic versioning identity, package identity, core/standard boundary, and v1 deferrals rather than restating those rules loosely.

## Implementation Independence

IRIS-V1-IDENTITY-C019: Iris v1 semantics MUST be independent of interpreter, JIT, AOT compiler, runtime representation, host compiler, operating system, pointer width, allocation strategy, inlining, hidden numeric representation, container bucket strategy, GC movement, or native wrapper language. A conforming implementation MAY optimize only when observable Iris behavior remains equivalent to the frozen semantics.

IRIS-V1-IDENTITY-C020: Optimized code MAY use primitives, specialized representations, inline caches, unboxing, specialized machine code, eliminated guards, or hoisted checks only when it preserves the selected Method, receiver Class or active revision, operand representation assumptions, lookup hierarchy version, dynamic dispatch hooks, static spine promises, and every other dependency required by later chapters. A dependency change MUST invalidate, fall back, or deoptimize before an affected later send observes stale behavior.

IRIS-V1-IDENTITY-C021: Public stable hashes, Type identities, package identities, conformance results, diagnostic categories, and serialized format compatibility MUST NOT depend on process address, host `size_t`, filesystem path, source display name, machine-local build data, object layout, raw pointer identity, runtime allocation order, JIT choice, or interpreter/JIT mode unless a later chapter explicitly marks a value as runtime-local.

IRIS-V1-IDENTITY-C022: A conforming implementation MUST expose ordinary Iris failures as ordinary catchable Iris exceptions or structured diagnostics where the later chapters require them. It MUST NOT expose frozen-language failures as Rust panics, C++ exceptions crossing the C ABI, assertions, process aborts, JIT crashes, undefined behavior, or host-only error strings.

IRIS-V1-IDENTITY-C023: The Host ABI compatibility contract for v1 is C, not Rust, C++, or an internal object ABI. Rust and C++ wrappers MAY provide safer or more ergonomic APIs, but they MUST NOT become the binary compatibility identity of Iris v1.

## Semantic Version And Package Identity

IRIS-V1-IDENTITY-C024: The Iris language major version is part of the semantic identity of public stable hash algorithms, named nominal Type identities, package compatibility, conformance vectors, standard format compatibility, and migration ledgers. A change that breaks a frozen language-major semantic contract MUST require a future Iris language major version.

IRIS-V1-IDENTITY-C025: Within one Iris language major version, public stable hash mappings for values that later chapters mark as specification-stable MUST remain unchanged across conforming implementations, platforms, processes, interpreter/JIT modes, and minor or patch revisions. A later language major MAY adopt a new mapping only with explicit versioned legacy entry points where frozen decisions require migration or reproduction.

IRIS-V1-IDENTITY-C026: Publishable packages MUST declare a globally unique reverse-domain-style `package_id` and an `api_major`. Named nominal Type identity MUST include package ID, package API major, fully qualified name, Iris language major, type kind, generic arity, and closed argument identities where applicable. It MUST exclude filesystem path, display name, source hash, and machine/build data.

IRIS-V1-IDENTITY-C027: Package minor and patch releases sharing `(package_id, api_major)` MUST preserve nominal Type identity. Breaking superclass, Contract, member, generic, native layout, or compatible static API promises MUST require a new package API major.

IRIS-V1-IDENTITY-C028: A runtime MUST activate at most one package implementation revision for each `(package_id, api_major)`. Same-major package implementations MUST NOT coexist through load context, load order, or hidden identity splitting. Coexisting incompatible implementations require different API majors and therefore distinct nominal identities.

IRIS-V1-IDENTITY-C029: Same-major package hot upgrade MUST be explicit and transactional. It MUST preserve compatible stable identities, validate all affected runtime and native obligations as a candidate, publish atomically, and leave the old package fully active on failure.

IRIS-V1-IDENTITY-C030: Manifestless local scripts have runtime-local package identity. Their named Types MAY be stable within one runtime, but they MUST NOT claim cross-process stable public Type hashes, publishable serialization contracts, or package API compatibility.

## Core And Standard Boundary

IRIS-V1-IDENTITY-C031: Iris v1 core and stable runtime packages include core object, Type, Contract, reflection, collections, text, bytes, Regex, Async Task and event loop, IO, File, Path, Encoding, Unicode, JSON, IrisValue, FFI, Package, diagnostics, and testing. [10-serialization-standard-library.md](10-serialization-standard-library.md) MUST refine this boundary without turning separately versioned standard packages into language-core ABI.

IRIS-V1-IDENTITY-C032: HTTP, general networking protocols, cryptography APIs, databases, GUI APIs, advanced PCRE-style Regex engines, and similar facilities MUST be separately versioned official standard packages, not Iris v1 language-core ABI. Internal BLAKE3 use for frozen hashes and integrity MUST NOT imply a public core cryptography suite.

## V1 Deferrals And Non-Goals

IRIS-V1-IDENTITY-C033: The following table is normative. Every row is an explicit `DEFERRED V1`, `OUT OF SCOPE`, or `PROHIBITED` item for Iris v1 and MUST NOT be implied by other normative prose.

| Item | Status | V1 rule | Related chapters |
| --- | --- | --- | --- |
| Legacy source compatibility as a restoration target | `PROHIBITED` | Iris v1 is a modern successor, not a legacy-source compatibility promise. | [11-migration-divergence.md](11-migration-divergence.md) |
| Rust runtime, parser, VM, GC, JIT, extension SDK, or workspace crates | `OUT OF SCOPE` | This specification wave defines language and boundary contracts only. | [12-conformance.md](12-conformance.md) |
| Treating old generated parser files, old PDF text, legacy scripts, or current implementation behavior as normative by default | `PROHIBITED` | Historical artifacts are evidence only unless a frozen decision adopts them. | [README.md](README.md), [11-migration-divergence.md](11-migration-divergence.md) |
| Backend-specific language semantics between interpreter, JIT, or native lowering | `PROHIBITED` | Observable language behavior must match across conforming backends. | [12-conformance.md](12-conformance.md) |
| Native no-GIL shared-memory Iris threads | `DEFERRED V1` | V1 has one IrisRuntime cooperative scheduler and shared heap. Future semantics are unspecified. | [07-async-resources-diagnostics.md](07-async-resources-diagnostics.md) |
| Task, thread, or async cancellation semantics | `DEFERRED V1` | V1 has no cancellation type, cancellation points, masking, asynchronous interruption, or `CancellationError` Contract. | [07-async-resources-diagnostics.md](07-async-resources-diagnostics.md) |
| Non-generic `Task`, async void, or fire-and-forget async signatures | `PROHIBITED` | Async calls return `Task<T>` and no-result async uses `Task<Nil>`. | [07-async-resources-diagnostics.md](07-async-resources-diagnostics.md) |
| Implicit blocking wait in Iris code | `PROHIBITED` | Host APIs may drive the event loop, but Iris code has no hidden blocking await substitute. | [07-async-resources-diagnostics.md](07-async-resources-diagnostics.md) |
| `defer` statement, keyword, or cleanup mechanism | `PROHIBITED` | D-468 keeps cleanup on explicit `try/finally`, iterator close, explicit `close`, and standard helpers; no v1 cleanup syntax is reserved here. | [07-async-resources-diagnostics.md](07-async-resources-diagnostics.md), [02-lexical-grammar.md](02-lexical-grammar.md) |
| Await, scheduler yield, thread transfer, or escaping transaction capability inside open/revision transactions | `PROHIBITED` | Open/revision transactions are synchronous, thread-confined, non-suspending, and non-escaping. | [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| Automatic meta-transaction retry, rebase, merge, or hidden block re-execution | `DEFERRED V1` | Users may retry explicitly and own external effects. | [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| Candidate-instance preview during open transactions | `DEFERRED V1` | Candidate state is visible through transaction-aware metadata/reflection only. | [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| Immediate Ruby-style meta hooks such as `included`, `method_added`, `inherited`, or equivalents | `PROHIBITED` | D-322 uses after-commit revision events instead of callbacks during candidate construction, validation, or commit. | [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| Automatic construction/revision reconciliation | `DEFERRED V1` | There is no `new_current`, `new_checked`, constructor retry, revision lock, automatic post-construction reinitialization, or instance state migration. | [03-runtime-object-model.md](03-runtime-object-model.md) |
| Automatic live-instance enumeration or implicit `migrate_revision` calls | `DEFERRED V1` | `migrate_revision` is an ordinary convention invoked only by application code. | [03-runtime-object-model.md](03-runtime-object-model.md) |
| Hidden external-effect rollback for opens, upgrades, decorators, migrations, initializers, native calls, or IO | `PROHIBITED` | Runtime rollback covers only runtime-owned candidate state unless a later chapter defines explicit cleanup. | [07-async-resources-diagnostics.md](07-async-resources-diagnostics.md), [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| Overload sets or implicit type-directed dispatch | `PROHIBITED` | Iris v1 uses one ordinary selector identity plus explicit Contract-qualified slots. | [05-types-contracts-generics.md](05-types-contracts-generics.md) |
| Declaration-site variance, use-site projection, or generic `Actual as Exposed` syntax | `DEFERRED V1` | Generic Class and Contract instantiations are invariant. | [05-types-contracts-generics.md](05-types-contracts-generics.md) |
| Generic specialization as source semantics, SFINAE, arbitrary compile-time execution, non-type parameters, dependent types, higher-kinded types, variadic type parameters, conditional types, mapped types, or type-level metaprogramming | `DEFERRED V1` | Internal JIT specialization is allowed only when unobservable. | [05-types-contracts-generics.md](05-types-contracts-generics.md) |
| Default generic type arguments | `DEFERRED V1` | V1 applications use fixed full arity, with `_` only where later chapters permit local inference. | [05-types-contracts-generics.md](05-types-contracts-generics.md) |
| Recursive Type aliases | `DEFERRED V1` | V1 aliases are transparent, generic, and non-recursive. | [05-types-contracts-generics.md](05-types-contracts-generics.md) |
| Raw generic instance types or bare generic Class construction | `PROHIBITED` | Bare generic Class names denote definition metadata, not instance types. | [05-types-contracts-generics.md](05-types-contracts-generics.md) |
| Opening closed generic Classes or Modules independently | `PROHIBITED` | Opens target the unapplied generic definition and propagate transactionally. | [05-types-contracts-generics.md](05-types-contracts-generics.md), [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| Proc/lambda split, nonlocal-return Closure, or `LocalJumpError` model | `PROHIBITED` | D-421 keeps v1 Closure return local to the Closure and does not split Closure into Proc/lambda families. | [04-bindings-callables-control-flow.md](04-bindings-callables-control-flow.md) |
| Contract Method bodies, default implementations, stored state, initializers, raw ivars, or private requirements | `PROHIBITED` | Contracts are pure static promises with qualified slot identities. | [05-types-contracts-generics.md](05-types-contracts-generics.md) |
| Opening Contracts | `PROHIBITED` | `open contract` is not a v1 operation. | [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| Raw mutable reflection tables or eval-string primary mutation | `PROHIBITED` | Reflection exposes permission-filtered immutable views and mutation uses transactions. | [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| Compiler AST as general v1 reflection API | `DEFERRED V1` | Reflection has a fixed minimal typed core API. | [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| Historical first-class `ReflectionCapability` token model | `PROHIBITED` | V1 uses `ReflectionPolicy` and `MetaCapabilities`. | [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| Finer first-class delegated reflection tokens | `DEFERRED V1` | D-329 keeps v1 authorization in Host-configured `ReflectionPolicy` inspect/mutate scopes; delegated token systems are later work distinct from the prohibited historical `ReflectionCapability` token model. | [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| Transitive propagation of static open extensions | `PROHIBITED` | Static extension visibility requires direct import of the exporting module. | [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| Wildcard imports or runtime-string language imports | `PROHIBITED` | Imports are static, explicit, aliasable/selective, and never wildcard. | [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| Partially initialized or lazy cyclic Modules | `PROHIBITED` | Module initialization is an acyclic deterministic DAG. | [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| Script access to Host ABI, extension function tables, raw runtime handles, loader internals, or arbitrary native symbols outside FFI | `PROHIBITED` | Script-originated external binary calls use the standard FFI subsystem only. | [09-native-host-ffi.md](09-native-host-ffi.md) |
| Stable Rust, C++, internal object, pointer, layout, or vtable ABI | `PROHIBITED` | C is the stable Host/extension ABI. | [09-native-host-ffi.md](09-native-host-ffi.md) |
| Signature-less or raw unsafe FFI invocation | `PROHIBITED` | FFI calls require explicit sidecar or programmatic signatures. | [09-native-host-ffi.md](09-native-host-ffi.md) |
| Automatic reflective object or ivar serialization | `PROHIBITED` | Serialization is explicit and Contract-driven. | [10-serialization-standard-library.md](10-serialization-standard-library.md) |
| IrisValue byte/tag micro-schema inside the language identity chapter | `DEFERRED V1 to format specification` | This chapter fixes format responsibility only. | [10-serialization-standard-library.md](10-serialization-standard-library.md) |
| Public core cryptography suite from internal BLAKE3 use | `PROHIBITED` | Cryptography APIs are separate official standard packages. | [10-serialization-standard-library.md](10-serialization-standard-library.md) |
| HTTP, general networking protocols, databases, GUI, and similar facilities in language-core ABI | `DEFERRED V1 to standard packages` | These areas ship as separately versioned official standard packages. | [10-serialization-standard-library.md](10-serialization-standard-library.md) |

IRIS-V1-IDENTITY-C034: If a later chapter needs a feature listed in the v1 deferral table, it MUST either preserve the listed status, narrow the later chapter to avoid the feature, or update the approved decision source before claiming v1 conformance. A later chapter MUST NOT silently convert a deferral into normative semantics.

## Traceability Coverage For This Chapter

IRIS-V1-IDENTITY-C035: The future traceability matrix MUST map this chapter to D-001, D-002, D-073, D-074, D-075, D-076, D-077, D-078, D-079, D-080, D-081, D-082, D-083, D-084, D-085, D-086, D-087, D-174, D-175, D-176, D-177, D-178, D-179, D-180, D-181, D-242, D-243, D-244, D-245, D-246, D-247, D-248, D-249, D-250, D-322, D-329, D-421, D-452, D-453, D-454, D-455, D-456, D-457, D-458, D-468, D-472, D-487, D-488, D-489, D-490, and D-504. It MAY also link adjacent decisions when later chapters refine details that this identity chapter only frames.

IRIS-V1-IDENTITY-N002: Informative note: D-073 through D-087 define the stable public numeric hash identity. This chapter captures the versioning and implementation-independence promise only. The canonical byte grammar and hashing vectors belong to [06-collections-text-regex.md](06-collections-text-regex.md) and [12-conformance.md](12-conformance.md).


## Identity Coverage Vectors

IRIS-V1-IDENTITY-C036: The following vectors are normative traceability vectors with concrete audit inputs. They cover identity and compatibility decisions without creating implementation code.

| Vector ID | Category | Applicability | Source/Input | Expected observable | Decisions |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-IDENTITY-V001` | diagnostic | documentation validator required; interpreter not applicable; JIT not applicable; native not applicable | Validate the fixed Iris v1 artifact set and semantic source declaration. | Exactly 14 product artifacts exist under `spec/iris-v1`; `.omo/drafts/iris-language-specification.md` is the approved semantic source; no implementation or archaeology file is normative. | `D-000`, `D-001` |
| `IRIS-V1-IDENTITY-V002` | diagnostic | documentation validator required; interpreter optional; JIT optional; native optional | Validate a source corpus using object values, dynamic operator dispatch, closures, raising arbitrary objects, and host/native boundary declarations. | Corpus is accepted only when those identity surfaces are present and not replaced by source-compatible Legacy Iris restoration claims. | `D-002` |

## Identity Coverage Cases

These rows provide concrete documentation and source inputs for the identity decisions that this chapter owns.

| Vector ID | Category | Applicability | Source/Input | Expected observable | Decisions |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-IDENTITY-V012` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | Documentation fixture: `spec/iris-v1/README.md` plus `01-language-identity.md` declares Iris v1 a modern successor and `11-migration-divergence.md` records each legacy incompatibility. | Documentation validation succeeds only when the compatibility declaration is present and no clause promises legacy source, parser, ABI, or implementation restoration. | `D-001` |
| `IRIS-V1-IDENTITY-V013` | positive | compiler required; interpreter required; JIT required; native not applicable | Iris source: `class Probe { fun +(other: Object) { 7 } } let p = Probe.new(); let result = p + Object.new(); let block = { raise p }; try { block() } catch value, context { [result, value, context.value] }`; a separate native fixture declares a C Host ABI extension entry. | Interpreter and JIT return tuple `[7, p, p]`; `p` is raised and caught unchanged; the source accepts Class, Method, Closure, operator-message, and arbitrary-object exception surfaces. The native declaration is accepted without requiring a native call. | `D-002` |
| `IRIS-V1-IDENTITY-V014` | positive | interpreter required; JIT required; native not applicable | Iris source fixture creates two `Widget` instances, commits a compatible `open class Widget` revision, invokes `first.migrate_revision()` explicitly, and reads the revision marker from both instances. | The explicitly migrated `first` reports the new marker; `second` retains its prior marker until application code calls its ordinary `migrate_revision` Method. No implicit enumeration or migration occurs at commit. | `D-264` |
