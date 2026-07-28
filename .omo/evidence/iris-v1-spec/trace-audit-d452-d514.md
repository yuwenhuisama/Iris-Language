# Trace Audit D-452 Through D-514

Date: 2026-07-27

Scope: Audit `spec/iris-v1/traceability-matrix.md` rows `D-452` through `D-514` only. No specification files were edited.

Rule applied: each D-ID appears once. A broad suite is accepted only when its source or expected observable names the row obligation directly. Otherwise this file gives a precise vector spec or a precise replacement using an existing direct vector. `D-472` remains deferred. `D-491` and `D-498` must be normative because their cited FFI clauses are normative requirements.

## Counts

| Count key | Value |
| --- | ---: |
| Total audited rows | 63 |
| D-ID range | `D-452` through `D-514` |
| Unique D-IDs in table below | 63 |
| Accepted direct coverage rows | 28 |
| Deferred rows | 1 |
| Rows needing direct existing-vector replacement or status update | 21 |
| Rows needing a precise new or revised vector spec | 13 |
| Rows where broad suite coverage is rejected | 32 |
| Required normative status corrections | 2 |
| Spec files edited by this task | 0 |

## Machine Applicable Audit Table

| D-ID | Audit result | Exact clauses | Accepted direct coverage or precise vector spec | Downstream action |
| --- | --- | --- | --- | --- |
| `D-452` | vector spec needed | `IRIS-V1-TYPES-C003`, `IRIS-V1-TYPES-C007` | Spec: compiler plus reflection vector with `fun f(value) { value }`, expected parameter and return metadata are `Dynamic<Object>`, runtime accepts any Object after Dynamic bound check, inferred body facts are not exported. | Add or revise `IRIS-V1-TYPES-V017` to name omitted Method parameter and return defaults directly. |
| `D-453` | accepted direct | `IRIS-V1-TYPES-C014`, `IRIS-V1-TYPES-C017` | `IRIS-V1-TYPES-V017`, source names `Dynamic<T>` and expected observations include runtime guards and static Types. | Keep row. |
| `D-454` | accepted direct | `IRIS-V1-TYPES-C017`, `IRIS-V1-TYPES-C035` | `IRIS-V1-TYPES-V017`, source names `is`, `as`, `as?` and expected observations include casts and runtime guards. | Keep row. |
| `D-455` | vector spec needed | `IRIS-V1-TYPES-C015`, `IRIS-V1-TYPES-C017` | Spec: compile `type Name<T> = Array<T>` and use it in reflection and assignment, expected canonical target Type identity; compile recursive direct and indirect aliases, expected static errors. | Add or revise `IRIS-V1-TYPES-V017` to name transparent generic non-recursive Type aliases. |
| `D-456` | vector spec needed | `IRIS-V1-TYPES-C074`, `IRIS-V1-TYPES-C017` | Spec: compare repeated `Box<String>` Type reflections and Class object reflection, expected same interned Type identity for equal normalized Type expressions and Type object distinct from Class object. | Add or revise `IRIS-V1-TYPES-V017` to name Type interning and Class distinction. |
| `D-457` | vector spec needed | `IRIS-V1-TYPES-C042`, `IRIS-V1-TYPES-C053` | Spec: Contract with instance Method, Class-object Method, property, and generic Method requirements is accepted; Contract with Method body, stored state, initializer, private requirement, or executable statement is rejected. | Add or revise `IRIS-V1-TYPES-V017` or map to a direct Contract vector. |
| `D-458` | accepted direct | `IRIS-V1-TYPES-C080`, `IRIS-V1-TYPES-C010` | `IRIS-V1-TYPES-V017`, source names `Never` and expected observations include Never flow. | Keep row. |
| `D-459` | vector spec needed | `IRIS-V1-COLLECTIONS-C023`, `IRIS-V1-COLLECTIONS-C003` | Spec: Array literal and empty Array cases check explicit closed `Array<T>`, invariance, bounded read and scalar write, mutation, equality, hash rejection, and no read growth. | Add or revise `IRIS-V1-COLLECTIONS-V040`; keep `IRIS-V1-MIG-013`. |
| `D-460` | vector spec needed | `IRIS-V1-COLLECTIONS-C003`, `IRIS-V1-COLLECTIONS-C040` | Spec: Array Range slice returns independent mutable snapshot; mutating snapshot does not affect source; Array content equality succeeds; Array public `hash` raises `InvalidKeyError`. | Add or revise `IRIS-V1-COLLECTIONS-V040` to name slices and unhashability directly. |
| `D-461` | vector spec needed | `IRIS-V1-COLLECTIONS-C027`, `IRIS-V1-COLLECTIONS-C003` | Spec: parse `%{ 1 + 2: value, key_binding: other }`, expected expression keys; parse legacy `{ key => value }`, expected migration diagnostic; empty `%{}` requires expected `Hash<K,V>` context. | Keep `IRIS-V1-GRAMMAR-V011` only if expanded as above; keep `IRIS-V1-MIG-019`. |
| `D-462` | vector spec needed | `IRIS-V1-COLLECTIONS-C029`, `IRIS-V1-COLLECTIONS-C040` | Spec: `hash[key]`, `hash[key] = value`, `fetch`, `delete`, missing lookup, missing fetch, missing delete, and Contract failures produce the exact result and error rules. | Add or revise `IRIS-V1-COLLECTIONS-V040` to name lookup, update, fetch, and delete directly. |
| `D-463` | accepted direct | `IRIS-V1-COLLECTIONS-C006`, `IRIS-V1-COLLECTIONS-C007` | `IRIS-V1-GRAMMAR-V011` names `a ..= b` and `a ..< b`; `IRIS-V1-COLLECTIONS-V040` names Range. | Keep row and `IRIS-V1-MIG-011`. |
| `D-464` | accepted direct | `IRIS-V1-COLLECTIONS-C038`, `IRIS-V1-COLLECTIONS-C003` | `IRIS-V1-COLLECTIONS-V040`, source names Range stepping program and expected observations name Range behavior. | Keep row. |
| `D-465` | accepted direct | `IRIS-V1-COLLECTIONS-C003`, `IRIS-V1-COLLECTIONS-C021` | `IRIS-V1-COLLECTIONS-V040`, source names Tuple and expected observations include identity, equality, hashability, and iteration. | Keep row. |
| `D-466` | accepted direct | `IRIS-V1-COLLECTIONS-C003`, `IRIS-V1-COLLECTIONS-C011` | `IRIS-V1-COLLECTIONS-V040`, source names Iterator and Iteration; migration row `IRIS-V1-MIG-016` covers the historical keyword. | Keep row. |
| `D-467` | accepted direct | `IRIS-V1-COLLECTIONS-C003`, `IRIS-V1-COLLECTIONS-C040` | `IRIS-V1-COLLECTIONS-V040`, source names Tuple, Array, Hash, Range, Iterator, Iteration, fail-fast traversal, and expected observations name mutability, iteration, and fail-fast policy. | Keep row. |
| `D-468` | accepted direct | `IRIS-V1-CONTROL-C068`, `IRIS-V1-CONTROL-C070` | `IRIS-V1-CONTROL-V041`, source names attempted `defer { cleanup }` syntax and expected observable names `defer` rejection. | Keep row and `IRIS-V1-MIG-026`. |
| `D-469` | accepted direct | `IRIS-V1-CONTROL-C068`, `IRIS-V1-CONTROL-C070` | `IRIS-V1-CONTROL-V041` names `try`, typed `catch`, catch-all, `finally`, re-raise, and `raise value from cause`; `IRIS-V1-ASYNC-V027` directly covers cleanup interaction. | Keep row. |
| `D-470` | direct replacement needed | `IRIS-V1-ASYNC-C057`, `IRIS-V1-ASYNC-C032` | Current `IRIS-V1-ASYNC-V026` is async callable coverage, not `using` cleanup. Direct coverage is `IRIS-V1-ASYNC-V027`, source names `using`, Closeable, async cleanup, iterator cleanup, and cleanup failure. | Replace coverage with `IRIS-V1-ASYNC-V027`. |
| `D-471` | vector spec needed | `IRIS-V1-ASYNC-C031`, `IRIS-V1-ASYNC-C057` | Spec: standard Closeable resource `close()` called twice after success returns `nil` with no duplicate side effect; first close failure does not permit duplicate released-resource side effects; ordinary `using` observes idempotence. | Add or revise `IRIS-V1-ASYNC-V027` to name idempotent standard Closeable close directly. |
| `D-472` | accepted deferred | `IRIS-V1-IDENTITY-C033`, `IRIS-V1-ASYNC-C017` | Metadata-only coverage is correct: cancellation is explicitly `DEFERRED V1` and has no executable v1 behavior. | Keep `DEFERRED V1` status and metadata-only coverage. |
| `D-473` | accepted direct | `IRIS-V1-CONTROL-C065`, `IRIS-V1-CONTROL-C070` | `IRIS-V1-CONTROL-V039`, `IRIS-V1-CONTROL-V041`, and `IRIS-V1-ASYNC-V027` name ExceptionContext, causes, suppression, cleanup, and diagnostics. | Keep row. |
| `D-474` | direct replacement needed | `IRIS-V1-META-C011`, `IRIS-V1-META-C113` | Direct coverage is `IRIS-V1-META-V001` for two explicit Module blocks and `IRIS-V1-META-V002` for ordinary statement outside any Module block. `IRIS-V1-META-V046` is broad support only. | Replace or augment coverage with `IRIS-V1-META-V001`, `IRIS-V1-META-V002`, `IRIS-V1-META-V046`. |
| `D-475` | direct replacement needed | `IRIS-V1-META-C012`, `IRIS-V1-META-C113` | Direct coverage is `IRIS-V1-META-V003` for duplicate Module origins; add an open-module positive if needed for explicit opens. | Replace or augment coverage with `IRIS-V1-META-V003` plus a positive open-module vector spec. |
| `D-476` | direct replacement needed | `IRIS-V1-META-C013`, `IRIS-V1-META-C113` | Direct rejection coverage is `IRIS-V1-META-V004` for wildcard import and runtime-string import. Precise positive spec should include `import pkg::Module as Alias` and `from pkg::Module import Name, Other as Alias`. | Replace or augment coverage with `IRIS-V1-META-V004` and a positive alias/selective import vector spec. |
| `D-477` | direct replacement needed | `IRIS-V1-META-C113`, `IRIS-V1-META-C016` | Direct coverage is `IRIS-V1-META-V005` for explicit re-export and `IRIS-V1-META-V010` for static extension only re-exported transitively. | Replace or augment coverage with `IRIS-V1-META-V005`, `IRIS-V1-META-V010`. |
| `D-478` | direct replacement needed | `IRIS-V1-META-C113`, `IRIS-V1-META-C017` | Direct failure coverage is `IRIS-V1-META-V006` for initialization cycle. Positive DAG order coverage is named by `IRIS-V1-META-V001` and `IRIS-V1-META-V046`. | Replace or augment coverage with `IRIS-V1-META-V001`, `IRIS-V1-META-V006`, `IRIS-V1-META-V046`. |
| `D-479` | accepted direct | `IRIS-V1-META-C010`, `IRIS-V1-META-C003` | `IRIS-V1-META-V046`, source names package manifest and lockfile; expected observable names package identity and dependency observations. | Keep row. |
| `D-480` | accepted direct | `IRIS-V1-META-C008`, `IRIS-V1-META-C009`, `IRIS-V1-META-C010` | `IRIS-V1-META-V043`, `IRIS-V1-META-V044`, `IRIS-V1-META-V045`, and `IRIS-V1-META-V046` directly cover permission requests, grants, scoped inspect, mutate denial, and self-authorization rejection. | Keep row. |
| `D-481` | direct replacement needed | `IRIS-V1-META-C020`, `IRIS-V1-META-C113` | Direct static import rejection is `IRIS-V1-META-V004`; direct dynamic package loading is in `IRIS-V1-META-V046` source. | Replace or augment coverage with `IRIS-V1-META-V004`, `IRIS-V1-META-V046`. |
| `D-482` | direct replacement needed | `IRIS-V1-META-C096`, `IRIS-V1-META-C113` | Current `IRIS-V1-META-V046` is package coverage. Direct reflection coverage is `IRIS-V1-META-V032` for permission-filtered immutable view and `IRIS-V1-META-V033` for no mutable table. | Replace coverage with `IRIS-V1-META-V032`, `IRIS-V1-META-V033`, `IRIS-V1-META-V048`. |
| `D-483` | direct replacement needed | `IRIS-V1-META-C097`, `IRIS-V1-META-C081` | Precise spec: reflection API smoke vector checks required members for Class, Module, Contract, Method, Type, and Revision views; mutation authority still follows `IRIS-V1-META-C081`. | Replace `IRIS-V1-META-V046` with `IRIS-V1-META-V048` plus the reflection API smoke vector spec. |
| `D-484` | direct replacement needed | `IRIS-V1-META-C099`, `IRIS-V1-META-C113` | Direct coverage is `IRIS-V1-META-V034` for `respond_to?` not invoking `method_missing` and `IRIS-V1-META-V035` for Contract namespace separation. | Replace coverage with `IRIS-V1-META-V034`, `IRIS-V1-META-V035`, `IRIS-V1-META-V048`. |
| `D-485` | direct replacement needed | `IRIS-V1-META-C081`, `IRIS-V1-META-C113` | Direct transaction and capability coverage is `IRIS-V1-META-V012` through `IRIS-V1-META-V017`, `IRIS-V1-META-V024` through `IRIS-V1-META-V028`, plus `IRIS-V1-META-V047` and `IRIS-V1-META-V048`. | Replace `IRIS-V1-META-V046` with the direct transaction and capability vectors. |
| `D-486` | direct replacement needed | `IRIS-V1-META-C113`, `IRIS-V1-META-C081` | Direct coverage is `IRIS-V1-META-V008`, conditional body defines member through meta API and member is dynamic-only reflection metadata; `IRIS-V1-META-V046` covers static import boundary only. | Replace or augment coverage with `IRIS-V1-META-V008`, `IRIS-V1-META-V046`. |
| `D-487` | direct replacement needed | `IRIS-V1-ASYNC-C057`, `IRIS-V1-ASYNC-C019` | Current `IRIS-V1-ASYNC-V027` and `IRIS-V1-ASYNC-V028` are cleanup and revision-event vectors. Direct async callable and scheduler coverage is `IRIS-V1-ASYNC-V026`. | Replace coverage with `IRIS-V1-ASYNC-V026`. |
| `D-488` | direct replacement needed | `IRIS-V1-ASYNC-C010`, `IRIS-V1-ASYNC-C057` | Direct coverage is `IRIS-V1-ASYNC-V026`, source names async functions and Closures returning Task and forbidden blocking wait; expected observable names Task type. | Replace coverage with `IRIS-V1-ASYNC-V026`. |
| `D-489` | direct replacement needed | `IRIS-V1-ASYNC-C057`, `IRIS-V1-ASYNC-C019` | Direct coverage is `IRIS-V1-ASYNC-V026`, expected observable names scheduler, await, failure replay, and unobserved diagnostics. | Replace coverage with `IRIS-V1-ASYNC-V026`. |
| `D-490` | direct replacement needed | `IRIS-V1-ASYNC-C057`, `IRIS-V1-ASYNC-C062` | Direct coverage needs all boundary vectors: `IRIS-V1-ASYNC-V026` for scheduling, `IRIS-V1-ASYNC-V027` for resources, `IRIS-V1-ASYNC-V028` for meta and revision event scheduling. | Add `IRIS-V1-ASYNC-V026`; keep `IRIS-V1-ASYNC-V027`, `IRIS-V1-ASYNC-V028`. |
| `D-491` | status update needed | `IRIS-V1-FFI-C006`, `IRIS-V1-FFI-C003` | `IRIS-V1-FFI-V021` directly names Host/extension ABI, table negotiation, safe wrapper calls, and wrapper boundaries. Clauses are normative, not implementation freedom. | Change status from `informative implementation freedom` to `normative`. |
| `D-492` | accepted direct | `IRIS-V1-FFI-C032`, `IRIS-V1-FFI-C006` | `IRIS-V1-FFI-V021`, source names opaque handle rooting and expected observable names handle identity. | Keep row. |
| `D-493` | accepted direct | `IRIS-V1-FFI-C006`, `IRIS-V1-FFI-C012` | `IRIS-V1-FFI-V021`, source names wrong runtime and worker-thread access; expected observable names thread affinity. | Keep row. |
| `D-494` | accepted direct | `IRIS-V1-FFI-C054`, `IRIS-V1-FFI-C021` | `IRIS-V1-FFI-V021`, source names native error status and expected observable names error propagation. | Keep row. |
| `D-495` | direct replacement needed | `IRIS-V1-FFI-C054`, `IRIS-V1-FFI-C026` | Direct metadata binding coverage is `IRIS-V1-FFI-V009` for digest mismatch, `IRIS-V1-FFI-V010` for runtime-only registration, plus `IRIS-V1-FFI-V021` for suite-level ABI checks. | Add `IRIS-V1-FFI-V009`, `IRIS-V1-FFI-V010`; keep `IRIS-V1-FFI-V021`. |
| `D-496` | direct replacement needed | `IRIS-V1-FFI-C030`, `IRIS-V1-FFI-C032` | Direct native payload and Closeable resource coverage is `IRIS-V1-FFI-V011`, `IRIS-V1-FFI-V012`, and `IRIS-V1-FFI-V013`; `IRIS-V1-FFI-V021` is broad support. | Add `IRIS-V1-FFI-V011`, `IRIS-V1-FFI-V012`, `IRIS-V1-FFI-V013`; keep `IRIS-V1-FFI-V021`. |
| `D-497` | direct replacement needed | `IRIS-V1-FFI-C037`, `IRIS-V1-FFI-C016` | Direct native async token coverage is `IRIS-V1-FFI-V005` and `IRIS-V1-FFI-V006`; `IRIS-V1-FFI-V021` is broad support. | Add `IRIS-V1-FFI-V005`, `IRIS-V1-FFI-V006`; keep `IRIS-V1-FFI-V021`. |
| `D-498` | status update needed | `IRIS-V1-FFI-C054`, `IRIS-V1-FFI-C006` | `IRIS-V1-FFI-V021` directly names table negotiation. Clauses `IRIS-V1-FFI-C038` through `IRIS-V1-FFI-C042` are normative requirements, not implementation freedom. | Change status from `informative implementation freedom` to `normative`; optionally add `IRIS-V1-FFI-V014`, `IRIS-V1-FFI-V015`. |
| `D-499` | accepted direct | `IRIS-V1-FFI-C006`, `IRIS-V1-FFI-C054` | `IRIS-V1-FFI-V022`, source names script `FFI.open`, permission denial, signature-less call, and C-compatible wrapper corpus. | Keep row. |
| `D-500` | accepted direct | `IRIS-V1-FFI-C050`, `IRIS-V1-FFI-C043` | `IRIS-V1-FFI-V022`, source names Library identity and expected observable names FFI library object. | Keep row. |
| `D-501` | accepted direct | `IRIS-V1-FFI-C050`, `IRIS-V1-FFI-C054` | `IRIS-V1-FFI-V022`, source names sidecar signatures and signature-less call; expected observable names binding, signature, dynamic-only Method, and unsafe-call rejection. | Keep row. |
| `D-502` | accepted direct | `IRIS-V1-LIBRARY-C008`, `IRIS-V1-LIBRARY-C028` | `IRIS-V1-LIBRARY-V015`, source names Serializable value, raw object, unsupported live handle; expected observable names no automatic raw object dump. | Keep row. |
| `D-503` | accepted direct | `IRIS-V1-LIBRARY-C007`, `IRIS-V1-LIBRARY-C008`, `IRIS-V1-LIBRARY-C009`, `IRIS-V1-CONFORMANCE-C044` | `IRIS-V1-LIBRARY-V015`, `IRIS-V1-LIBRARY-V005`, `IRIS-V1-LIBRARY-V006`, `IRIS-V1-LIBRARY-V007` directly cover format version, invalid magic/version, excessive declared length, and round-trip values. | Keep row. |
| `D-504` | accepted direct | `IRIS-V1-LIBRARY-C028`, `IRIS-V1-LIBRARY-C032` | `IRIS-V1-LIBRARY-V017`, source names core versus standard package inventory and expected observable names core membership and deferred library areas. | Keep row. |
| `D-505` | accepted direct | `IRIS-V1-GRAMMAR-C015`, `IRIS-V1-GRAMMAR-C054` | `IRIS-V1-GRAMMAR-V011`, `IRIS-V1-COLLECTIONS-V042`, and `IRIS-V1-LIBRARY-V017` directly name Regex literals, match operators, core Regex, and package split. | Keep row. |
| `D-506` | accepted direct | `IRIS-V1-GRAMMAR-C054`, `IRIS-V1-GRAMMAR-C015` | `IRIS-V1-GRAMMAR-V011`, `IRIS-V1-COLLECTIONS-V042`, and `IRIS-V1-LIBRARY-V017` directly name Regex flags, Unicode properties, unsupported backreference, duplicate flags, safe subset, and package split. | Keep row. |
| `D-507` | accepted direct | `IRIS-V1-GRAMMAR-C040`, `IRIS-V1-GRAMMAR-C041`, `IRIS-V1-CONFORMANCE-C045` | `IRIS-V1-GRAMMAR-V003` and `IRIS-V1-GRAMMAR-V009` directly name exponent and full precedence cases; `IRIS-V1-CONFORMANCE-C045` requires complete precedence coverage. | Keep row. |
| `D-508` | accepted direct | `IRIS-V1-GRAMMAR-C041`, `IRIS-V1-CONFORMANCE-C045` | `IRIS-V1-GRAMMAR-V003`, `IRIS-V1-GRAMMAR-V007`, and `IRIS-V1-GRAMMAR-V009` directly name associativity and non-chainable diagnostics. | Keep row. |
| `D-509` | accepted direct | `IRIS-V1-GRAMMAR-C013`, `IRIS-V1-GRAMMAR-C014`, `IRIS-V1-MIGRATION-C009` | `IRIS-V1-GRAMMAR-V001`, `IRIS-V1-GRAMMAR-V010`, and `IRIS-V1-MIGRATION-EX024` directly name exact keyword inventory and historical non-keyword handling. | Keep row and migration links. |
| `D-510` | accepted direct | `IRIS-V1-GRAMMAR-C017`, `IRIS-V1-GRAMMAR-C018`, `IRIS-V1-GRAMMAR-C019`, `IRIS-V1-GRAMMAR-C020`, `IRIS-V1-GRAMMAR-C021`, `IRIS-V1-GRAMMAR-C022`, `IRIS-V1-GRAMMAR-C023` | `IRIS-V1-GRAMMAR-V004` and `IRIS-V1-GRAMMAR-V011` directly name every listed longest-match conflict. | Keep row. |
| `D-511` | vector spec needed | `IRIS-V1-META-C086`, `IRIS-V1-META-C085` | Spec: decorators on Class, Module, Contract, Method, and property preserve declaration kind, nominal identity, package identity, and static spine; decorator returning a different declaration category is rejected. | Add or revise `IRIS-V1-META-V048`; current word `decorators` is too broad by itself. |
| `D-512` | vector spec needed | `IRIS-V1-META-C087`, `IRIS-V1-META-C113` | Spec: directly imported declarative decorator static plan is deterministic and compiler-visible; plan reading time or random is rejected; runtime phase applies inside candidate transaction and rolls back on failure. | Add or revise `IRIS-V1-META-V030`, `IRIS-V1-META-V031`, and `IRIS-V1-META-V048` to name static plan plus runtime transaction. |
| `D-513` | vector spec needed | `IRIS-V1-META-C113`, `IRIS-V1-META-C081` | Spec: decorator generated Method without required MetaCapability aborts; decorator cannot bypass package permission, ReflectionPolicy, `override`, `impl`, Contract compatibility, or static-spine checks. | Add or revise `IRIS-V1-META-V029` and `IRIS-V1-META-V048` to name no privilege escalation. |
| `D-514` | vector spec needed | `IRIS-V1-META-C092`, `IRIS-V1-META-C113` | Spec: generic Decorator Contract receives immutable declaration metadata and controlled transform context; runtime transform is synchronous and non-awaiting; replay on hot upgrade or rollback preserves source order; reflection exposes ordered decorator identity and filtered diff metadata. | Add or revise `IRIS-V1-META-V048` to name Decorator Contracts, failure, replay, and reflection. |

## Special Case Handling

The Host ABI and ABI versioning status rows in the main audit table require `normative` status, not `informative implementation freedom`, because their cited FFI clauses state required behavior.

The cancellation row in the main audit table stays `DEFERRED V1` with metadata-only coverage. It must not gain an executable cancellation vector for v1.

## Broad Coverage Rejections

The rows marked `vector spec needed` or `direct replacement needed` in the main audit table use a broad suite today but need a direct existing vector replacement or a precise vector spec before a writer applies final traceability edits.

## Validation Notes

Manual checks performed:

| Check | Result |
| --- | --- |
| Range count `514 - 452 + 1` | 63 |
| Main audit table rows with D-ID cells | 63 |
| Duplicate D-IDs in audit table | 0 |
| Missing D-IDs from range | 0 |
| `D-472` deferred handling present | yes |
| `D-491` normative correction present | yes |
| `D-498` normative correction present | yes |
| Product spec files edited | no |
