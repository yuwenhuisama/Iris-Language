# Iris v1 Specification Index

Status: Iris v1.11, frozen semantics with owner-approved errata.

IRIS-V1-TRACE-C013: This directory is the only home for formal Iris v1 specification artifacts. The approved semantic source is `spec/drafts/iris-language-specification.md`. Historical files under `legacy/Document/`, legacy scripts, old generated parser output, and existing implementation code are evidence only. They are not normative unless a frozen decision explicitly adopts a behavior.

IRIS-V1-TRACE-C001: The language semantics are closed for v1. Writers MUST preserve the frozen decisions, including later revisions that supersede earlier wording. Apparent gaps MUST be recorded as `DEFERRED V1` or escalated before writing. Writers MUST NOT add, remove, reinterpret, or silently complete a language feature. This clause is qualified by the revision procedure in IRIS-V1-TRACE-C019 through IRIS-V1-TRACE-C022: a gap closed through that procedure is not a silent completion.

## Revision Procedure

IRIS-V1-TRACE-C019: Implementation experience MAY reveal that a frozen chapter states a semantic requirement without supplying the concrete source syntax needed to satisfy it, or omits a rule an implementation cannot avoid deciding. Such a finding is an errata candidate, not a licence to invent language features. A writer MUST NOT act on an errata candidate without recorded owner approval.

IRIS-V1-TRACE-C020: An approved errata revision MUST increment the specification revision to `v1.1` and later `v1.n`, MUST preserve every published clause, example, and vector ID under IRIS-V1-TRACE-C007, and MUST add new material under the next available clause numbers. Renumbering or deleting a published ID remains prohibited. Both the English chapters and the Simplified Chinese translation under `zh-cn/` MUST be updated in the same revision so the two remain consistent.

IRIS-V1-TRACE-C021: An errata revision MAY close a stated-but-unspecified gap, supply missing grammar productions for behavior another clause already requires, or record an inference rule an implementation must otherwise guess. It MUST NOT reinterpret a decided semantic, weaken a frozen guarantee, or remove a requirement. Where an errata revision widens a previously closed inventory, such as the reserved keyword set fixed by D-509, the revising clause MUST state the new total explicitly and supersede the superseded count in place.

IRIS-V1-TRACE-C022: Every errata revision MUST be verified against the committed conformance corpus before publication. The revising writer MUST report any vector whose result changes, and MUST NOT adjust, retag, or delete a vector to conceal a behavior change introduced by the revision. A vector that legitimately fails under revised semantics is evidence to be reported, not a defect to be hidden.

## Artifact Inventory

IRIS-V1-TRACE-C014: The Iris v1 specification set has exactly these 14 product artifacts, in this order:

| Order | Artifact | Purpose |
| --- | --- | --- |
| 1 | `README.md` | Normative document index, editorial contract, terminology, status, and reading order. |
| 2 | `01-language-identity.md` | Goals, compatibility identity, normative conventions, versioning, and v1 deferrals. |
| 3 | `02-lexical-grammar.md` | Source encoding, tokens, literals, keywords, precedence, associativity, and consolidated grammar. |
| 4 | `03-runtime-object-model.md` | Values, identity, equality/hash, Class/Module/Contract, MRO, revisions, Methods, properties, and built-ins. |
| 5 | `04-bindings-callables-control-flow.md` | Bindings, scopes, callable forms, parameters, Closure capture, calls, assignments, conditionals, loops, match, and exceptions. |
| 6 | `05-types-contracts-generics.md` | Gradual contracts, Dynamic, union and intersection, nilability, casts, inference, generics, Contract views, and Never. |
| 7 | `06-collections-text-regex.md` | Tuple, Array, Hash, Range, iteration, String, MutableString, Symbol, Bytes, ByteArray, Regex, and hashing boundaries. |
| 8 | `07-async-resources-diagnostics.md` | Task/Awaitable, single-thread scheduler, Closeable/using, ExceptionContext, stack/cause/suppressed, and diagnostic streams. |
| 9 | `08-modules-metaprogramming.md` | Packages, modules, import/export, executable declaration bodies, open transactions, revisions, decorators, MetaCapabilities, and ReflectionPolicy. |
| 10 | `09-native-host-ffi.md` | Stable C Host/extension ABI boundary, handles, GC/native payload, async completion, FFI script API, and Rust/C++ integration rules. |
| 11 | `10-serialization-standard-library.md` | JSON/IrisValue scope, Encoding API, core versus standard packages, and deferred library areas. |
| 12 | `11-migration-divergence.md` | Every intentional legacy divergence and migration example. |
| 13 | `12-conformance.md` | Example/vector ID schema, positive/negative/diagnostic/differential coverage, and freeze criteria. |
| 14 | `traceability-matrix.md` | Every frozen D-ID mapped to exact normative clause(s), examples/vectors, divergence entry, or explicit deferred status. |

IRIS-V1-TRACE-C002: No other formal product artifact belongs in this directory for Iris v1. Supporting scripts or generated evidence, if needed by later tasks, MUST live outside this 14-file product inventory unless the plan is updated.

## Editorial Contract

IRIS-V1-TRACE-C003: Normative text defines requirements on conforming Iris v1 implementations, tools, specifications, or programs. Normative clauses MUST use RFC 2119-style terms when a requirement is intended: `MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, and `MAY`. These terms carry their ordinary specification meaning inside normative clauses.

IRIS-V1-TRACE-C004: Informative text explains rationale, gives examples, describes archaeology, or notes implementation freedom. Informative text MUST be explicitly labeled with one of these prefixes:

| Label | Use |
| --- | --- |
| `Informative note:` | Explanatory prose that does not create a requirement. |
| `Informative example:` | Non-executable or illustrative example prose. |
| `Implementation note:` | Permitted implementation strategy or warning, not a semantic rule. |
| `Historical note:` | Legacy behavior or source archaeology, not a v1 rule. |

IRIS-V1-TRACE-C005: Every normative behavior, syntax rule, diagnostic rule, semantic boundary, or deferral MUST have a stable clause ID. Every executable example and every conformance vector MUST have a stable ID. A paragraph without one of these IDs MUST NOT contain normative requirements.

IRIS-V1-TRACE-C006: Each chapter writer MUST keep current terminology canonical. Historical stale-term context: `Interface`, `StringBuilder`, first-class `ReflectionCapability` token, and permanent instance-bound ClassRevision are non-canonical historical names. They MUST appear only in clearly marked historical or migration text, or in canonical-replacement glossary rows that explicitly identify the historical term being replaced.

## Stable IDs

IRIS-V1-TRACE-C007: Clause, example, and vector IDs are permanent once published. Do not renumber existing IDs to make room for new material. Add a suffixed sub-ID or the next available sequence number instead.

| ID kind | Syntax | Example | Meaning |
| --- | --- | --- | --- |
| Normative clause | `IRIS-V1-<chapter>-C<nnn>` | `IRIS-V1-RUNTIME-C001` | A required rule or explicit deferral. |
| Informative note | `IRIS-V1-<chapter>-N<nnn>` | `IRIS-V1-GRAMMAR-N001` | Non-normative explanatory note. |
| Example | `IRIS-V1-<chapter>-EX<nnn>` | `IRIS-V1-TYPES-EX001` | Source example tied to one or more clauses. |
| Conformance vector | `IRIS-V1-<chapter>-V<nnn>` | `IRIS-V1-FFI-V001` | Machine-readable positive, negative, diagnostic, or differential test case. |
| Migration row | `IRIS-V1-MIG-<nnn>` | `IRIS-V1-MIG-001` | Legacy behavior disposition in the divergence ledger. |
| Traceability row | `D-<nnn>` | `D-509` | Frozen decision identifier from the approved draft. |

IRIS-V1-TRACE-C015: Chapter codes are fixed as follows:

| Artifact | Chapter code |
| --- | --- |
| `01-language-identity.md` | `IDENTITY` |
| `02-lexical-grammar.md` | `GRAMMAR` |
| `03-runtime-object-model.md` | `RUNTIME` |
| `04-bindings-callables-control-flow.md` | `CONTROL` |
| `05-types-contracts-generics.md` | `TYPES` |
| `06-collections-text-regex.md` | `COLLECTIONS` |
| `07-async-resources-diagnostics.md` | `ASYNC` |
| `08-modules-metaprogramming.md` | `META` |
| `09-native-host-ffi.md` | `FFI` |
| `10-serialization-standard-library.md` | `LIBRARY` |
| `11-migration-divergence.md` | `MIGRATION` |
| `12-conformance.md` | `CONFORMANCE` |
| `traceability-matrix.md` | `TRACE` |

IRIS-V1-TRACE-C008: Heading anchors SHOULD be plain GitHub Markdown anchors generated from heading text. Clause IDs MUST appear in the heading or at the start of the clause paragraph so automated checks can find them without parsing prose.

## Canonical Terminology

IRIS-V1-TRACE-C016: Use these terms consistently across the Iris v1 specification:

| Term | Definition for this specification |
| --- | --- |
| Iris v1 | The first formal revived Iris language specification, a modern successor to legacy Iris with frozen v1 semantics. |
| Legacy Iris | The historical language, documents, scripts, and implementation used as archaeology and migration evidence, not as automatic authority. |
| Contract | The canonical Iris term for an explicit interface/protocol-style static and dynamic obligation surface. Canonical replacement context: use `Contract`, not historical `Interface`, for official v1 terminology. |
| MutableString | The canonical mutable text type. Canonical replacement context: use `MutableString`, not historical `StringBuilder`, for official v1 terminology. |
| ReflectionPolicy | The runtime and Host-controlled policy that grants or denies reflection inspection and mutation by caller package, target, operation, and scope. Canonical replacement context: it is not a historical first-class `ReflectionCapability` token. |
| logical Class | The stable Class identity visible to Iris programs across allowed dynamic mutation. A logical Class may have changing active revisions. |
| active revision | The current committed revision of a logical Class, Module, or related runtime declaration surface that dispatch and reflection observe at a given point. |
| Method | A callable member installed on a Class, Module, Contract view, or related receiver surface. Operators are Method sends where the chapter rules say so. |
| BoundMethod | A Method coupled to a receiver or binding context for invocation. Its exact identity and capture rules belong in the runtime and callable chapters. |
| Closure | A lexical callable value with captured environment as specified by the callable and control-flow chapter. |
| FFI | The standard script-level external binary integration subsystem. For v1, it is the only script-originated path for external binary calls and supports stable C ABI bindings through declared signatures. |
| Host ABI | The stable C boundary used by hosts and native extensions to embed or extend Iris. Rust wrappers are convenience APIs, not the binary compatibility contract. |
| DEFERRED V1 | A named non-goal for the v1 semantic contract. Deferred items are governed by IRIS-V1-TRACE-C005 and MUST NOT be implied by normative prose. |

## Reading And Dependency Order

IRIS-V1-TRACE-C009: Readers and writers SHOULD use this dependency order:

| Step | Read | Reason |
| --- | --- | --- |
| 1 | `README.md` | Establishes inventory, IDs, terminology, and scope. |
| 2 | `01-language-identity.md` | Defines language identity, compatibility posture, normative vocabulary, versioning, and deferrals. |
| 3 | `02-lexical-grammar.md` | Defines parser-visible forms that later semantic chapters reference. |
| 4 | `03-runtime-object-model.md` | Defines values, identity, dispatch, revisions, object model, and built-in runtime foundations. |
| 5 | `04-bindings-callables-control-flow.md` | Depends on grammar and runtime model for names, callable values, control transfer, and exceptions. |
| 6 | `05-types-contracts-generics.md` | Depends on identity, runtime, and callable foundations for Contracts and gradual types. |
| 7 | `06-collections-text-regex.md` | Depends on grammar, runtime, control flow, and type contracts for literals, iteration, text, Regex, and hashing. |
| 8 | `07-async-resources-diagnostics.md` | Depends on callable/control semantics and runtime object model for Task, Awaitable, cleanup, and diagnostics. |
| 9 | `08-modules-metaprogramming.md` | Depends on Class, Module, Contract, type, and revision rules for mutation, decorators, and reflection. |
| 10 | `09-native-host-ffi.md` | Depends on metadata, Contract, runtime, resource, and reflection rules for native and script binary boundaries. |
| 11 | `10-serialization-standard-library.md` | Depends on collections, text, Regex, FFI, and Contract boundaries for library scope. |
| 12 | `11-migration-divergence.md` | Depends on all normative chapters so every legacy difference can point to a replacement rule. |
| 13 | `12-conformance.md` | Depends on normative chapters to define executable examples, vectors, diagnostics, and freeze gates. |
| 14 | `traceability-matrix.md` | Depends on the completed set so every frozen decision maps to exact clauses, examples, vectors, divergence rows, or deferrals. |

## Frozen Semantics Status

IRIS-V1-TRACE-C017: The approved draft records `status: approved` and states that every semantic and design question in the current v1 scope is frozen or explicitly deferred. D-000 fixes this output directory. D-001 fixes the compatibility identity as a modern successor, not a source-compatible restoration. D-002 fixes the core language identity at the level of every-value objecthood, dynamic message dispatch, closures, constrained runtime mutation, exception raising, embedding, and native extension support.

IRIS-V1-TRACE-C010: D-509 closes the v1 reserved keyword inventory. The v1.1 errata revision widens that inventory once, adding exactly `typeof`, under the procedure in IRIS-V1-TRACE-C019 through IRIS-V1-TRACE-C022; the authoritative count and table live in `IRIS-V1-GRAMMAR-C013`, which supersedes the earlier total in place. D-510 fixes contextual longest-match token handling for the listed conflicts. D-511 through D-514 fix decorator shape, deterministic planning, MetaCapabilities limits, failure behavior, replay, and reflection. Later chapters MUST preserve these decisions and link them to exact clauses rather than restating them loosely in prose.

## Implementation And Non-Goals

IRIS-V1-TRACE-C011: This specification set does not implement Iris. Work in this directory MUST NOT modify compiler, parser, runtime, VM, GC, JIT, CLI, extension SDK, FFI bindings, tests, generated parser sources, `legacy/Document/`, or legacy script corpora.

IRIS-V1-TRACE-C018: The following are explicit non-goals for this documentation wave:

| Non-goal | Status |
| --- | --- |
| Rust runtime, parser, VM, GC, JIT, extension SDK, or workspace crates | `DEFERRED V1 implementation` |
| Moving, rewriting, or relabeling archaeology under `legacy/Document/` | `OUT OF SCOPE` |
| Treating the old PDF, generated parser files, or legacy implementation as normative by default | `OUT OF SCOPE` |
| Backend-specific behavior that changes semantics between interpreter and JIT | `PROHIBITED` |
| New language features or reinterpretation of frozen decisions | `PROHIBITED except through the IRIS-V1-TRACE-C019 errata procedure` |
| Extra product artifacts beyond the exact 14-file inventory | `PROHIBITED unless the plan changes` |

IRIS-V1-TRACE-C012: Implementation freedom may be documented only when it preserves the observable behavior required by the frozen v1 semantics.
