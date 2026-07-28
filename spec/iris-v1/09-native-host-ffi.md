# Iris v1 Native Host And FFI

Status: Iris v1 draft, frozen semantics.

IRIS-V1-FFI-C001: This chapter defines the stable Host ABI, native extension boundary, runtime-rooted handle model, native error model, native metadata binding, native payload ownership, async completion bridge, ABI version negotiation, script FFI surface, and Rust/C++ integration rules for Iris v1. It MUST be read after [README.md](README.md), [01-language-identity.md](01-language-identity.md), [03-runtime-object-model.md](03-runtime-object-model.md), [05-types-contracts-generics.md](05-types-contracts-generics.md), [07-async-resources-diagnostics.md](07-async-resources-diagnostics.md), and [08-modules-metaprogramming.md](08-modules-metaprogramming.md).

IRIS-V1-FFI-C002: This chapter MUST NOT define parser productions, implementation function names, internal object layouts, Rust crate APIs, C++ class APIs, GC algorithms, OS loader internals, cryptography APIs, general networking APIs, or any script binary path outside the standard `FFI` subsystem. It defines observable boundary contracts only.

## Stable Boundary Roles

IRIS-V1-FFI-C003: The stable Iris v1 Host/extension binary ABI is a C ABI. Rust, C++, Zig, C#, Swift, or other language bindings MAY wrap that ABI, but they MUST NOT become the binary compatibility identity. A conforming implementation MUST NOT require consumers to link against a Rust ABI, C++ ABI, internal object ABI, vtable layout, exception ABI, allocator ABI, or managed object layout for Iris v1 compatibility.

IRIS-V1-FFI-C004: The Host ABI covers runtime lifecycle, package and native artifact loading, value handles, calls, callbacks, metadata registration, extension entry, diagnostics, and async posting. Native extensions that integrate with the runtime MUST enter through this Host ABI or a wrapper that is mechanically bound to it. Ordinary Iris scripts MUST NOT call Host ABI tables, extension tables, raw loader APIs, or runtime handles directly.

IRIS-V1-FFI-C005: The script-originated external binary integration path is exactly one path: the standard `FFI` subsystem. `FFI.open` creates an `FFI::Library`, and all script-originated calls into external dynamic libraries occur through bound Methods on that Library. No alternate script API may expose arbitrary native symbols, Host ABI functions, extension tables, raw runtime handles, or loader internals.

IRIS-V1-FFI-C006: The following boundary role table is normative:

| Boundary role            | Stable binary contract                  | Script access                                          | Value representation                                                 | Error channel                                                   | Thread rule                                                           |
| ------------------------ | --------------------------------------- | ------------------------------------------------------ | -------------------------------------------------------------------- | --------------------------------------------------------------- | --------------------------------------------------------------------- |
| Host embedding Iris      | C Host ABI function tables              | No direct script access                                | Opaque runtime-rooted handles                                        | Status plus result or ExceptionContext handles                  | Runtime thread for heap-touching calls, post queue from other threads |
| Native extension package | C extension ABI negotiated with runtime | Visible only through package metadata and Iris Methods | Opaque handles and validated payload descriptors                     | Status plus ExceptionContext handles                            | Runtime thread for handle use, post queue for workers                 |
| Rust wrapper             | Convenience wrapper over C ABI          | No direct script access                                | RAII wrappers around opaque handles                                  | Converts statuses to wrapper errors without crossing ABI panics | Checks runtime ownership and thread affinity before calls             |
| C++ wrapper              | Convenience wrapper over C ABI          | No direct script access                                | RAII wrappers around opaque handles                                  | Catches or forbids C++ exceptions at ABI edge                   | Checks runtime ownership and thread affinity before calls             |
| Script`FFI`            | Standard Iris API                       | The only script binary path                            | `FFI::Library` objects and bound Method metadata, not Host handles | Ordinary Iris exceptions and ExceptionContext                   | Library Methods run on runtime thread, external work posts back       |

IRIS-V1-FFI-N001: Historical note: Legacy headers and the pointer extension expose C++ objects, raw `void*` payload access, direct class registration, and ad hoc extension entry points. They are archaeology only. Iris v1 keeps the extension goal but replaces that model with C tables, opaque handles, metadata verification, and runtime-owned payload descriptors.

## Runtime-Rooted Handles

IRIS-V1-FFI-C007: Every Iris value, object, Class, Module, Contract, Method, Closure, Type, Task, ExceptionContext, metadata object, or native wrapper value that crosses the Host ABI MUST cross as an opaque handle owned by one IrisRuntime. A raw managed object pointer, raw Class pointer, raw Method pointer, raw GC address, object layout address, vtable address, or interior pointer MUST NOT cross the ABI.

IRIS-V1-FFI-C008: An opaque value handle strongly roots its target while it is live. Releasing the handle removes that root. Scoped or local handles are ordinary rooted value handles whose lifetime is bounded by a handle frame; closing the frame bulk-releases every live handle in that frame. A conforming implementation MAY move, compact, pin, intern, or reallocate managed values internally as long as live handles continue to denote the same Iris identity or identity-less value semantics required by earlier chapters.

IRIS-V1-FFI-C009: Handles belong to exactly one runtime. A Host, extension, wrapper, or FFI binding MUST NOT pass a handle to another runtime, compare handle numeric values as stable identity, persist handle bytes, derive hashes from handle numbers, or assume reuse never occurs after release. Runtime destruction invalidates every handle for that runtime and makes later handle use fail with a closed or invalid-runtime status.

IRIS-V1-FFI-C010: Every ABI value handle is a rooted handle. Its release mode may be explicit release or scoped frame release, but either mode MUST keep the target strongly rooted until the release occurs. Temporary call-frame references that are not handles MAY exist only inside one ABI call implementation; they MUST NOT be stored, returned, placed in ABI records, treated as value handles, or escape the call. A wrapper MAY enforce handle lifetime with language scopes, but the C ABI contract remains the authority.

IRIS-V1-FFI-C011: The following handle table is normative:

| Handle category               | Rooting effect                                                                      | May cross C ABI | May be used off runtime thread                                  | Release rule                                                                          | Failure on misuse                        |
| ----------------------------- | ----------------------------------------------------------------------------------- | --------------- | --------------------------------------------------------------- | ------------------------------------------------------------------------------------- | ---------------------------------------- |
| Runtime handle                | Owns runtime identity, not an Iris value root                                       | Yes             | Only for thread-safe lifecycle or post operations named as such | Runtime lifecycle rules                                                               | Closed or invalid-runtime status         |
| Explicit-release value handle | Strongly roots one Iris value or metadata object until explicit release             | Yes             | No                                                              | Explicit release                                                                      | Invalid-handle or thread-affinity status |
| Scoped value handle           | Strongly roots one Iris value or metadata object until its handle frame is released | Yes             | No                                                              | Scoped frame release, with optional earlier explicit release if the table declares it | Invalid-handle or thread-affinity status |
| Completion token              | Authorizes one Task completion post                                                 | Yes             | Post only from worker threads                                   | Consumed exactly once or closed on runtime shutdown                                   | Duplicate-completion or closed status    |

## Thread Affinity And Post Queue

IRIS-V1-FFI-C012: All ABI calls that touch Iris handles, the managed heap, Class or Module metadata, Type metadata, Method dispatch, ReflectionPolicy, MetaCapabilities, the scheduler, Task state, or ExceptionContext graphs MUST run on the owning runtime event-loop thread. Calling such an API from another thread MUST return a thread-affinity status and MUST NOT race the managed heap.

IRIS-V1-FFI-C013: External threads MAY perform external work that does not inspect, dereference, compare, retain, release, or otherwise use Iris handles. External threads MAY post copied or externally owned data, statuses, and completion tokens to the owning runtime through the thread-safe post queue. The runtime thread converts posted data to Iris values, observes handles, and completes Tasks.

IRIS-V1-FFI-C014: The post queue is the only v1 cross-thread entry into one runtime from external worker threads. It MUST preserve FIFO ordering for posts accepted from one producer in submission order. Iris v1 does not specify ordering between independent external producers before their posts are accepted by one runtime queue.

IRIS-V1-FFI-C015: Future native no-GIL shared-memory Iris threads require a later ABI major version. Iris v1 Host ABI implementations MUST NOT expose worker-thread handle use, worker-thread heap access, or shared managed-memory mutation as a compatible extension of this ABI.

IRIS-V1-FFI-C016: The following thread matrix is normative:

| Operation                                               | Runtime thread                                     | External worker thread                                                    | Required failure                               |
| ------------------------------------------------------- | -------------------------------------------------- | ------------------------------------------------------------------------- | ---------------------------------------------- |
| Create or release a value handle                        | Allowed                                            | Prohibited unless function table declares a thread-safe release primitive | Thread-affinity status                         |
| Read, write, or call through a handle                   | Allowed                                            | Prohibited                                                                | Thread-affinity status                         |
| Build Iris values from posted native data               | Allowed                                            | Prohibited                                                                | Thread-affinity status                         |
| Perform blocking OS or device work without Iris handles | Allowed if Host permits blocking                   | Allowed                                                                   | Not applicable                                 |
| Post copied completion data                             | Allowed                                            | Allowed through post queue only                                           | Closed status after shutdown                   |
| Complete a Task directly                                | Allowed only through runtime completion processing | Prohibited                                                                | Thread-affinity or duplicate-completion status |

## Status, Results, And ExceptionContext

IRIS-V1-FFI-C017: Every C ABI operation that can fail MUST return a status code and place ordinary results, raised values, or `ExceptionContext` objects in explicit out handles or result records. A status alone is not enough when the operation raised or captured an Iris exception. A string-only error channel is insufficient for conformance.

IRIS-V1-FFI-C018: Native code MAY create ordinary Iris errors, raise Iris values, or return failed Task completions only through ABI operations that create or propagate an `ExceptionContext`. The raised Iris value remains an Iris object or value, while stack, cause, suppressed cleanup failures, async links, and native bridge records belong to the `ExceptionContext` as defined by [04-bindings-callables-control-flow.md](04-bindings-callables-control-flow.md) and [07-async-resources-diagnostics.md](07-async-resources-diagnostics.md).

IRIS-V1-FFI-C019: Rust panics, C++ exceptions, SEH exceptions, host language unwinding, long jumps, or foreign exceptions MUST NOT cross the C ABI boundary. Wrappers MUST catch or prevent recoverable language-level unwinding before it reaches C. If memory corruption, stack corruption, or another unrecoverable condition prevents safe translation, the implementation MAY abort rather than return undefined state.

IRIS-V1-FFI-C020: Iris unwinding MUST NOT long jump across native frames. A native callback or Host call that raises returns through the ABI with a status and `ExceptionContext` handle. Wrapper APIs may present this as a language-native error object, but the C boundary still observes status plus handles.

IRIS-V1-FFI-C021: The following error table is normative:

| Event                                       | C ABI result                                                 | Iris visibility                                             | Required preserved data                                        |
| ------------------------------------------- | ------------------------------------------------------------ | ----------------------------------------------------------- | -------------------------------------------------------------- |
| Successful call returns value               | Success status plus result handle or primitive result record | Caller observes returned Iris value                         | Runtime ownership and Type Contract                            |
| Iris Method raises                          | Failure status plus ExceptionContext handle                  | Ordinary catch or failed Task observes same context         | Raised value, cause, suppressed contexts, stack records        |
| Native callback raises Iris value           | Failure status plus ExceptionContext handle                  | Ordinary Iris propagation                                   | Native bridge frame plus raised value                          |
| Thread-affinity violation                   | Thread-affinity status, no heap mutation                     | Host or wrapper error, not an Iris heap race                | Runtime identity and attempted operation category              |
| Rust panic or C++ exception at wrapper edge | Translated native error status where safe                    | Ordinary Iris error only if wrapper creates one through ABI | Panic or exception text may be diagnostic, not sole Iris error |
| Runtime already closed                      | Closed status                                                | No new Iris value unless runtime can safely report one      | Runtime identity and shutdown state                            |

## Native Metadata Binding

IRIS-V1-FFI-C022: Native packages MUST ship Iris-readable compile-time metadata for each exported Iris Class, Module, Contract, Method, property, generic declaration, native layout obligation, MetaCapabilities policy, package identity, permissions, required ABI version, and native artifact digest. The compiler imports this metadata as static API only after validating that it is complete and consistent with earlier chapters.

IRIS-V1-FFI-C023: Runtime native load MUST verify that the loaded native artifact matches the metadata selected by package resolution and lockfiles. Verification MUST include package identity, API major, Iris language major, ABI major and minor requirements, declared feature bits, artifact digest, native layout descriptors, Type and Contract signatures, Method signatures, MetaCapabilities, and ReflectionPolicy-relevant package identity.

IRIS-V1-FFI-C024: Runtime-only native registration MAY create dynamic-only members only through the same open transaction, MetaCapabilities, ReflectionPolicy, static-spine, Contract, and package rules as Iris metaprogramming. Runtime-only native registration MUST NOT expand an already compiled static API, bypass import rules, bypass `impl` or `override`, or make the compiler believe a member existed at compile time.

IRIS-V1-FFI-C025: Native Method binding MUST preserve the callable signature recorded in metadata. Argument and return values cross the native boundary through handles or declared primitive interop records, and every written Contract remains both a static promise and a runtime boundary guard under [05-types-contracts-generics.md](05-types-contracts-generics.md).

IRIS-V1-FFI-C026: The following metadata binding table is normative:

| Metadata item                                  | Compile-time role                                         | Runtime verification                                              | Failure rule                                        |
| ---------------------------------------------- | --------------------------------------------------------- | ----------------------------------------------------------------- | --------------------------------------------------- |
| Package identity and API major                 | Establishes nominal Type and package identity             | Must match manifest, lock, artifact, and runtime load request     | Abort load before executable native code is exposed |
| Class, Module, Contract, and Type metadata     | Supplies static API and reflection facts                  | Must match artifact registration tables and static spine          | Reject load or candidate publication                |
| Method and property signatures                 | Supplies call,`impl`, `override`, and Contract checks | Must match native callable descriptors                            | Reject binding before calls can occur               |
| Native layout and payload descriptors          | Supplies shape and GC obligations                         | Must validate size, alignment, trace, drop, and ownership records | Reject publication or package load                  |
| MetaCapabilities and ReflectionPolicy identity | Supplies authority and visibility checks                  | Must match manifest, package, and Host grants                     | Deny operation or abort candidate                   |
| Artifact digest and ABI requirements           | Supplies reproducible binding target                      | Must match loaded binary and negotiated table version             | Reject load                                         |

## Native Payloads, GC, And Closeable Resources

IRIS-V1-FFI-C027: Native-backed Classes that store native payloads MUST register runtime-owned payload descriptors. A descriptor states size, alignment, construction policy, destruction policy, optional trace obligations, optional final memory cleanup, native layout compatibility, and whether the payload represents an external resource. The runtime controls allocation and lifetime of the payload storage attached to Iris objects.

IRIS-V1-FFI-C028: Native payload trace logic MUST report only managed handles or roots through the ABI mechanism supplied for tracing. It MUST NOT dereference moved managed objects, retain raw managed pointers, create new Iris values, call arbitrary Iris Methods, raise Iris exceptions, block on external IO, or depend on worker-thread heap access.

IRIS-V1-FFI-C029: Native payload drop or final memory cleanup runs in a GC-safe runtime context. It MUST NOT raise into Iris, invoke ordinary Iris callbacks, allocate managed objects except where the ABI explicitly says a cleanup context permits it, or perform user-visible deterministic resource release. Failures in final memory cleanup are runtime diagnostics, not catchable language results.

IRIS-V1-FFI-C030: Files, sockets, GPU objects, database connections, OS handles, and similar external resources MUST use explicit idempotent `Closeable` behavior for deterministic release. GC timing is not a resource-management promise. A native-backed Iris object may have both a payload descriptor for memory safety and a `Closeable` Method for deterministic external release.

IRIS-V1-FFI-C031: Extension-owned raw object-pointer conventions are prohibited. A native extension MAY keep its own non-managed pointers inside runtime-owned payload storage or external resource records, but those pointers are not Iris object pointers and MUST NOT be exposed to scripts as raw addresses.

IRIS-V1-FFI-C032: The following native ownership table is normative:

| Native state                            | Owner                                         | Deterministic release                             | GC or runtime cleanup                    | May raise into Iris                                              |
| --------------------------------------- | --------------------------------------------- | ------------------------------------------------- | ---------------------------------------- | ---------------------------------------------------------------- |
| Runtime-rooted handle                   | IrisRuntime handle table                      | Explicit release or frame release                 | Runtime destruction invalidates          | Release failure reports status only                              |
| Native payload storage                  | IrisRuntime payload allocator                 | Object API may close external resource separately | Descriptor drop or final cleanup         | No                                                               |
| External file or socket                 | Native resource wrapper plus Iris object      | Idempotent`close()` through `Closeable`       | Final cleanup may release as last resort | `close()` may raise ordinary Iris error, final cleanup may not |
| Managed object reference from payload   | IrisRuntime handle or trace-reported root     | Handle release or descriptor update               | Trace keeps reachable targets alive      | No from trace or final cleanup                                   |
| Raw native pointer for external library | `FFI::Library` or native payload descriptor | `FFI::Library.close()` or resource `close()`  | Library or payload final cleanup         | Close Method may raise, final cleanup may not                    |

## Native Async Completion

IRIS-V1-FFI-C033: A native async Method that returns `Task<T>` MUST obtain a runtime-owned completion token or completion source tied to that Task. The token is opaque, belongs to one runtime, and authorizes at most one completion. The returned Task follows the identity, completion, await, and unobserved-failure rules in [07-async-resources-diagnostics.md](07-async-resources-diagnostics.md).

IRIS-V1-FFI-C034: Worker threads performing native async work MUST use only the thread-safe post queue with copied or externally owned data, status, and completion token. They MUST NOT read Iris handles, create Iris values, resolve Type Contracts, complete Tasks directly, inspect ExceptionContext graphs, or touch the managed heap.

IRIS-V1-FFI-C035: The runtime thread receives a posted completion, validates the token, converts posted data into Iris values under the declared `T` Contract, creates or propagates an `ExceptionContext` for failure, and completes the Task exactly once. A duplicate completion attempt MUST fail with duplicate-completion status and MUST NOT alter the already completed Task.

IRIS-V1-FFI-C036: Posting after runtime shutdown MUST return closed status. If shutdown closes before a native async operation posts completion, the runtime MUST either complete or diagnose according to its shutdown policy without allowing a worker thread to use Iris handles after close.

IRIS-V1-FFI-C037: The following async bridge table is normative:

| Phase                     | Native side authority                              | Runtime thread duty                      | Failure rule                                             |
| ------------------------- | -------------------------------------------------- | ---------------------------------------- | -------------------------------------------------------- |
| Async Method call starts  | Receives declared arguments on runtime thread      | Creates Task and completion token        | Setup error returns status and ExceptionContext          |
| Worker runs external work | Uses no Iris handles, owns or copies external data | None until post accepted                 | Worker handle use is prohibited                          |
| Worker posts success      | Sends copied result data and token                 | Converts to Iris value and checks`T`   | Conversion failure completes Task as failed              |
| Worker posts failure      | Sends error descriptor and token                   | Creates ExceptionContext and fails Task  | Missing context data becomes native bridge error context |
| Duplicate post            | No authority after first consumed token            | Leaves Task unchanged                    | Duplicate-completion status                              |
| Runtime closes            | Further posts return closed status                 | Invalidates tokens under shutdown policy | No heap access from workers                              |

## Function-Table Versioning

IRIS-V1-FFI-C038: C ABI evolution MUST use negotiated, versioned function tables and size-tagged records. Runtime, Host, extension, and wrapper participants declare required ABI major, minimum minor, supported feature bits, record sizes, and table sizes before any extension code receives authority to create or observe Iris values.

IRIS-V1-FFI-C039: ABI major mismatch MUST reject load or attachment. Compatible minor versions MAY append fields, append function table entries, add feature bits, or add optional records. A participant MUST NOT reinterpret an older record as a newer larger layout unless the supplied size covers the field being read and the required feature bit is present.

IRIS-V1-FFI-C040: Symbol IDs, Type IDs, handle IDs, table slots, feature-bit numbers, and native binding IDs are runtime-local unless a clause in this specification explicitly marks the value as stable across runtimes. They MUST NOT be persisted or used as public Type hashes, package identity, or cross-run ABI identity.

IRIS-V1-FFI-C041: A Rust wrapper crate, C++ wrapper library, or other binding MAY pin one or more supported C ABI versions and provide RAII handles, ownership checks, callback guards, panic or exception barriers, and typed conversion helpers. Such wrappers MUST report their underlying C ABI version and MUST fail closed when the negotiated C table cannot satisfy their safety assumptions.

IRIS-V1-FFI-C042: The following versioning table is normative:

| Change                                                  | ABI compatibility                                                    | Required mechanism                                       |
| ------------------------------------------------------- | -------------------------------------------------------------------- | -------------------------------------------------------- |
| Add optional helper function                            | Minor-compatible                                                     | Append table entry, increase table size, set feature bit |
| Add required semantic rule that old code cannot satisfy | Major-breaking                                                       | New ABI major                                            |
| Add field to record tail                                | Minor-compatible                                                     | Size-tagged record, read only when size covers field     |
| Change meaning of existing field or status              | Major-breaking                                                       | New ABI major                                            |
| Add status code with old fallback meaning               | Minor-compatible only if old participants can handle generic failure | Feature bit or documented fallback                       |
| Promise Rust or C++ object layout as binary contract    | Prohibited                                                           | Not allowed in Iris v1                                   |

## Script FFI Library Surface

IRIS-V1-FFI-C043: `FFI` is the standard namespace and service Class for script-originated external binary calls. `FFI.open(path, declarations: ...) -> FFI::Library` loads one Host-authorized dynamic library and returns an identity-bearing `FFI::Library` object. The Host or package manifest MUST grant the exact conceptual permission scopes `ffi.load` and `ffi.call` before loading or invocation succeeds. These scopes are manifest permission requests and Host grants under [08-modules-metaprogramming.md](08-modules-metaprogramming.md); an Iris package, dependency, native artifact, or FFI sidecar MUST NOT self-authorize them.

IRIS-V1-FFI-C044: `FFI::Library` owns and pins the native library lifetime, its bound symbol descriptors, bound callbacks, native pointers represented by FFI values, close state, and permission scope. It MUST implement idempotent `Closeable`. Multiple `FFI::Library` objects are independent even when they refer to the same filesystem artifact unless the Host explicitly shares loader state without changing observable Library identity.

IRIS-V1-FFI-C045: Every script-callable native symbol MUST have an explicit verified signature before invocation. Signatures may come from a sidecar declaration file passed to `FFI.open` or from a programmatic `library.bind(:symbol, signature, options)` operation. Sidecar and programmatic signatures have equivalent validation, permission checks, and reflection metadata. Unbound symbols MUST NOT be invoked.

IRIS-V1-FFI-C046: V1 exposes no signature-less raw unsafe call, address-call primitive, arbitrary `dlsym` object, Host ABI function pointer call, or unchecked callback trampoline to scripts. An implementation MUST NOT provide a conforming-mode escape hatch that invokes a native symbol without the signature metadata required by this chapter.

IRIS-V1-FFI-C047: An FFI signature MUST declare at least symbol name, C calling convention, parameter count, parameter C types, result C type, integer widths and signedness, floating widths, pointer nullability, pointer ownership and lifetime, text encoding, buffer length relations, callback metadata, error and result convention, thread behavior, close or release obligations, and whether the function may block. Missing required declaration data MUST reject binding before any call occurs.

IRIS-V1-FFI-C048: Bound FFI symbols become dynamic-only Methods on the `FFI::Library` object. They do not alter package static API, do not create Host ABI access, do not expose extension function tables, and do not bypass Type Contracts. Reflection MAY show their verified signatures subject to ReflectionPolicy and FFI permissions.

IRIS-V1-FFI-C049: V1 FFI supports stable C ABI calls only. Rust, C++, or other implementation-language libraries that want script FFI access MUST export C-compatible wrapper symbols and C-compatible data representations declared by the signature metadata. The Rust wrapper API for Host embedding is not script FFI, and script FFI MUST NOT promise Rust ABI compatibility.

IRIS-V1-FFI-C050: The following FFI binding table is normative:

| FFI surface                                   | Required declaration               | Script result                                                | Ownership and close rule                                                  | Failure rule                                                               |
| --------------------------------------------- | ---------------------------------- | ------------------------------------------------------------ | ------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| `FFI.open(path, declarations: sidecar)`     | Sidecar signatures and permissions | New`FFI::Library`                                          | Library pins native artifact until close                                  | Missing grant, invalid sidecar, or load failure raises ordinary Iris error |
| `library.bind(:symbol, signature, options)` | Programmatic complete signature    | Dynamic-only Method on that Library                          | Bound descriptor tied to Library lifetime                                 | Incomplete signature or denied symbol raises ordinary Iris error           |
| Call bound Method                             | Existing verified signature        | Iris value converted from C result                           | Signature ownership rules decide copying, borrowing, or Closeable wrapper | C error convention maps to Iris exception or result by signature           |
| Pass callback to C                            | Callback signature and lifetime    | C calls re-enter only through runtime-approved callback path | Runtime-rooted callback handle released by declared rule                  | Callback after release or close returns declared native failure            |
| `library.close()`                           | None beyond Library state          | `nil` on successful idempotent close                       | Releases bound descriptors and native loader reference                    | Close failure raises ordinary Iris error when deterministic close runs     |

## Safe Rust And C++ Extern C Guidance

IRIS-V1-FFI-C051: Rust integration intended for stable distribution SHOULD expose `extern "C"` functions and consume the negotiated C tables. Safe Rust wrappers SHOULD own handle release with RAII, encode runtime ownership in wrapper values, prevent handle use after release, block `Send` or cross-thread use for heap-touching handles, catch panics at wrapper boundaries where recovery is safe, and translate statuses into typed wrapper errors without hiding `ExceptionContext` handles.

IRIS-V1-FFI-C052: C++ integration intended for stable distribution SHOULD expose `extern "C"` functions and consume the negotiated C tables. C++ wrappers SHOULD own handle release with RAII, avoid throwing across C callbacks, translate exceptions before the C edge where recovery is safe, avoid exposing C++ object layout as ABI, and treat Iris handles as opaque runtime-owned capabilities.

IRIS-V1-FFI-C053: A safe wrapper MUST NOT promise that its language-native object identity, destructor timing, thread model, allocator, panic or exception type, generic type, or vtable layout is an Iris stable ABI. Its safety claims are local to the wrapper version and the C ABI version it negotiated.

## Examples And Conformance Vectors

IRIS-V1-FFI-EX001: Informative example, script FFI binds only declared C signatures:

```iris
let library = FFI.open("mathlib", declarations: "mathlib.ffi")
let value = library.hypot(3.0f64, 4.0f64)
library.close()
```

IRIS-V1-FFI-EX002: Informative example, native async bridge shape:

```iris
async fun load_image(path: String) -> Bytes {
  return await NativeImages.read(path)
}
```

IRIS-V1-FFI-C054: The following vector table is normative. The conformance chapter MUST preserve these vector IDs or map them to machine-readable records with the same observable outcomes:

| Vector ID            | Kind     | Scenario                                                                               | Expected result                                                                          |
| -------------------- | -------- | -------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| `IRIS-V1-FFI-V001` | Failure  | Native extension attempts to pass a raw managed object pointer across C ABI            | ABI validation rejects or runtime returns invalid-boundary status                        |
| `IRIS-V1-FFI-V002` | Positive | Strong handle survives GC movement or compaction                                       | Handle still denotes same Iris identity or value semantics                               |
| `IRIS-V1-FFI-V003` | Failure  | Handle from runtime A used with runtime B                                              | Invalid-runtime status, no heap mutation                                                 |
| `IRIS-V1-FFI-V004` | Failure  | Worker thread reads or calls through Iris handle                                       | Thread-affinity status                                                                   |
| `IRIS-V1-FFI-V005` | Positive | Worker thread posts copied completion data and token                                   | Runtime thread completes Task with checked value                                         |
| `IRIS-V1-FFI-V006` | Failure  | Native async operation posts the same completion token twice                           | First completion stands, duplicate-completion status for second post                     |
| `IRIS-V1-FFI-V007` | Positive | Native Method raises Iris value                                                        | Caller receives status plus ExceptionContext, catch observes ordinary Iris exception     |
| `IRIS-V1-FFI-V008` | Failure  | C++ exception or Rust panic crosses C ABI boundary                                     | Wrapper or runtime rejects boundary crossing, no undefined Iris state                    |
| `IRIS-V1-FFI-V009` | Failure  | Native artifact metadata digest mismatches loaded binary                               | Package or extension load aborts before binding                                          |
| `IRIS-V1-FFI-V010` | Failure  | Runtime-only native registration tries to expand compiled static API                   | Member remains dynamic-only or registration is rejected                                  |
| `IRIS-V1-FFI-V011` | Positive | Native payload traces managed handle roots only                                        | Referenced managed values remain alive, no arbitrary Iris call from trace                |
| `IRIS-V1-FFI-V012` | Positive | Closeable native resource closes twice                                                 | Both calls complete according to idempotent close contract                               |
| `IRIS-V1-FFI-V013` | Failure  | Final payload cleanup attempts to raise into Iris                                      | Runtime reports diagnostic or rejects descriptor, no Iris propagation from final cleanup |
| `IRIS-V1-FFI-V014` | Failure  | ABI major mismatch during extension negotiation                                        | Load rejected                                                                            |
| `IRIS-V1-FFI-V015` | Positive | ABI minor-compatible table appends optional field with size tag                        | Old participant ignores field, new participant reads only when size and feature allow    |
| `IRIS-V1-FFI-V016` | Positive | `FFI.open` with valid sidecar and grants                                             | Returns identity-bearing`FFI::Library`                                                 |
| `IRIS-V1-FFI-V017` | Failure  | Script calls unbound native symbol                                                     | Ordinary Iris error, no native call                                                      |
| `IRIS-V1-FFI-V018` | Failure  | Script requests signature-less unsafe call                                             | Static or runtime diagnostic, no native call                                             |
| `IRIS-V1-FFI-V019` | Positive | `library.bind` declares complete C signature                                         | Bound symbol appears as dynamic-only Library Method                                      |
| `IRIS-V1-FFI-V020` | Failure  | FFI declaration omits pointer ownership, nullability, or error convention where needed | Binding rejected before call                                                             |


## FFI Coverage Vectors

IRIS-V1-FFI-C058: The following vectors are normative traceability vectors with concrete Host, native, and FFI boundary observations.

| Vector ID | Category | Applicability | Source/Input | Expected observable | Decisions |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-FFI-V059` | negative | interpreter not applicable; JIT not applicable; native required; reason: package identity is checked while native metadata is loaded. | `metadata/manifestless_native.json` omits package identity and ReflectionPolicy; `fixtures/ffi/manifestless_native.c` exports a C extension entry without trusted Host identity. | Native-load status is `metadata-mismatch`; diagnostic is `ffi.native-package-identity-missing` with severity `error`, phase `native-load`, and clause `IRIS-V1-FFI-C022`; reflection metadata is not published. | `D-334` |
| `IRIS-V1-FFI-V060` | positive | interpreter not applicable; JIT not applicable; native required; reason: the fixture attaches a C extension table before Iris source execution. | `fixtures/ffi/host_abi_v1.c`: `iris_extension_attach(requested_major=1, minimum_minor=0)` is compiled as C and called through the Host table; `metadata/host_abi_v1.json` declares `abi_major: 1`, `abi_minor: 0`, and `c_abi: true`. | Status is `success`; the extension receives only the negotiated C table, and the table reports `abi_major=1`, `abi_minor=0`. No Rust, C++, object-layout, vtable, or allocator entry is present. | `D-491` |
| `IRIS-V1-FFI-V061` | positive | interpreter not applicable; JIT not applicable; native required; reason: the fixture observes Host handles and GC through the C ABI. | `fixtures/ffi/rooted_handle.c`: create integer `41`, retain it as `IrisHandle h`, force a collecting allocation cycle, read `h`, release `h`, then attempt a second read; `metadata/rooted_handle.json` declares `handle_kind: rooted_value`. | Before release, status is `success`, value is integer `41`, and type is `Integer`; no managed address is exposed. After release, status is `invalid-handle`, value is absent, and no heap mutation occurs. | `D-492` |
| `IRIS-V1-FFI-V062` | negative | interpreter not applicable; JIT not applicable; native required; reason: the fixture invokes a Host handle operation from an external worker. | `fixtures/ffi/thread_affinity.c`: a worker calls `iris_handle_get_int(h)` and then posts copied integer `7` with its completion token; `metadata/thread_affinity.json` declares `worker_uses_handles: false` for the accepted post path. | The worker handle read returns status `thread-affinity`, value is absent, and side effect is `heap_mutation=false`; the copied-data post returns `success` and is consumed on the runtime thread. | `D-493` |
| `IRIS-V1-FFI-V063` | negative | interpreter not applicable; JIT not applicable; native required; reason: the fixture raises through a native callback boundary. | `fixtures/ffi/native_error.c`: callback `raise_marker` creates raised value symbol `:ffi_marker` through the Host ABI; `metadata/native_error.json` declares `error_convention: status_plus_exception_context`. | Status is `failure`; result handle is absent; ExceptionContext handle is present with value `:ffi_marker`, type `ExceptionContext`, and a native bridge frame. No string-only error result and no foreign unwind occurs. | `D-494` |
| `IRIS-V1-FFI-V064` | negative | interpreter not applicable; JIT not applicable; native required; reason: metadata validation and native load occur before executable Iris code. | `metadata/static_api.json` declares package `fixture.static_api`, Method `answer() -> Integer`, ABI `1.0`, and digest `sha256:00`; `fixtures/ffi/static_api.c` has a different artifact digest `sha256:11`. | Package-load status is `metadata-mismatch`; diagnostic is `ffi.native-artifact-digest-mismatch` with severity `error`, phase `native-load`, and clause `IRIS-V1-FFI-C023`; no Method is published and no native code is called. | `D-495` |
| `IRIS-V1-FFI-V065` | positive | interpreter required; JIT not applicable; native required; reason: the script exercises a native-backed Closeable object through the native boundary. | `fixtures/ffi/payload_resource.c` declares a runtime-owned payload descriptor `{size: 8, alignment: 8, trace: none, drop: no_raise}` and a native `close` counter; script source is `let r = NativeFixture.resource(); r.close(); r.close()`. | Both calls return value `nil` of type `Nil`; the native close counter is exactly `1`; final cleanup emits no ExceptionContext and no ordinary Iris error. | `D-496` |
| `IRIS-V1-FFI-V066` | negative | interpreter required; JIT not applicable; native required; reason: the runtime thread creates and observes the Task while the fixture posts native completion. | `fixtures/ffi/async_once.c` posts copied integer `9` twice with one completion token; script source is `let t = NativeFixture.once(); await t`. | `await t` returns integer `9` of type `Integer`; the first post status is `success`; the second post status is `duplicate-completion`; Task value and completion count remain `9` and `1`. | `D-497` |
| `IRIS-V1-FFI-V067` | negative | interpreter not applicable; JIT not applicable; native required; reason: ABI table negotiation precedes source execution. | `fixtures/ffi/negotiate_v2.c` requests ABI major `2`, minimum minor `0`; `metadata/negotiate_v2.json` declares a size-tagged request record of size `24`; the runtime fixture exposes ABI `1.3`. | Attach status is `abi-major-mismatch`; diagnostic is `ffi.abi-major-mismatch` with severity `error`, phase `native-load`, and clause `IRIS-V1-FFI-C039`; no table is published and no extension entry runs. | `D-498` |
| `IRIS-V1-FFI-V068` | negative | interpreter required; JIT not applicable; native required; reason: the script request reaches the native loader but does not execute a bound Method. | Script source is `HostABI.load("fixtures/ffi/libfixture")`; package metadata requests `ffi.load` and `ffi.call`, but exposes no script-visible Host ABI capability. | Static validation rejects the source with diagnostic `ffi.host-abi-script-access` at phase `static`; no library is loaded, no native symbol is called, and no `FFI::Library` value is created. | `D-499` |
| `IRIS-V1-FFI-V069` | positive | interpreter required; JIT not applicable; native required; reason: `FFI.open` invokes the authorized native loader. | Script source is `let a = FFI.open("fixtures/ffi/libfixture", declarations: "fixtures/ffi/libfixture.ffi"); let b = FFI.open("fixtures/ffi/libfixture", declarations: "fixtures/ffi/libfixture.ffi"); [a == b, a.class_name]`; manifest grants `ffi.load` and `ffi.call`. | Value is array `[false, "FFI::Library"]`; element types are `Bool` and `String`; both opens return status `success`, and the two Library objects have distinct object identities. | `D-500` |
| `IRIS-V1-FFI-V070` | negative | interpreter required; JIT not applicable; native required; reason: binding validation precedes native invocation. | Sidecar `fixtures/ffi/libfixture.ffi` declares `int32 fixture_add(int32, int32)`; script source is `let l = FFI.open("fixtures/ffi/libfixture", declarations: "fixtures/ffi/libfixture.ffi"); l.call(:fixture_hidden)`. | Runtime exception has kind `FFI::UnboundSymbolError`, phase `runtime`, and message policy `not compared`; diagnostic is absent; the native call counter remains `0`, and no signature-less or raw-address call is available. | `D-501` |

## Traceability Notes

IRIS-V1-FFI-C055: This chapter owns the native and script FFI boundary decisions D-491 through D-501. It preserves the identity chapter rule that C is the only stable Host ABI, the async chapter rule that external completion posts into the runtime scheduler, the metaprogramming chapter rule that native metadata and runtime-only registration obey package, transaction, MetaCapabilities, and ReflectionPolicy checks, and the type chapter rule that native signatures are enforced Contracts.

IRIS-V1-FFI-C056: Chapter-owned decision IDs are `D-491`, `D-492`, `D-493`, `D-494`, `D-495`, `D-496`, `D-497`, `D-498`, `D-499`, `D-500`, and `D-501`.

IRIS-V1-FFI-C057: Referenced non-owned decision IDs include `D-001`, `D-002`, `D-077`, `D-172`, `D-173`, `D-174`, `D-175`, `D-176`, `D-177`, `D-178`, `D-179`, `D-180`, `D-181`, `D-207`, `D-208`, `D-209`, `D-210`, `D-211`, `D-212`, `D-213`, `D-214`, `D-215`, `D-216`, `D-217`, `D-218`, `D-219`, `D-220`, `D-242`, `D-243`, `D-244`, `D-245`, `D-246`, `D-247`, `D-248`, `D-249`, `D-250`, `D-322`, `D-323`, `D-324`, `D-325`, `D-326`, `D-327`, `D-469`, `D-470`, `D-471`, `D-472`, `D-473`, `D-474`, `D-475`, `D-476`, `D-477`, `D-478`, `D-479`, `D-480`, `D-481`, `D-482`, `D-483`, `D-484`, `D-485`, `D-486`, `D-487`, `D-488`, `D-489`, and `D-490`.
