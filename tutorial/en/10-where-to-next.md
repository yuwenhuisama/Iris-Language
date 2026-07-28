# Where To Next

This tutorial gives you the on-ramp. The specification is the authority. This chapter maps the 14 Iris v1 spec artifacts, suggests a reading order for different goals, and points to the chapters that matter for FFI, host embedding, and conformance.

```iris
let library = FFI.open("mathlib", declarations: "mathlib.ffi")
let value = library.hypot(3.0f64, 4.0f64)
library.close()
```

This snippet is reused from `IRIS-V1-FFI-EX001`. It shows the shape of script FFI: open a library through the standard `FFI` subsystem, call only declared signatures, then close the resource. It is not something to run today because Iris v1 has no implementation.

## The 14 spec artifacts

The spec directory has a fixed inventory. The file names matter because clause IDs and traceability refer to them.

| Order | Spec artifact | Read it for |
| --- | --- | --- |
| 1 | [README.md](../../spec/iris-v1/README.md) | Inventory, editorial rules, clause ID rules, terminology, and dependency order. |
| 2 | [01-language-identity.md](../../spec/iris-v1/01-language-identity.md) | The dynamic/static contract, implementation independence, package identity, and v1 deferrals. |
| 3 | [02-lexical-grammar.md](../../spec/iris-v1/02-lexical-grammar.md) | Tokens, literals, precedence, associativity, reserved keywords, and grammar. |
| 4 | [03-runtime-object-model.md](../../spec/iris-v1/03-runtime-object-model.md) | Objects, identity, dispatch, Classes, Modules, MRO, revisions, properties, truthiness, and built-ins. |
| 5 | [04-bindings-callables-control-flow.md](../../spec/iris-v1/04-bindings-callables-control-flow.md) | Bindings, scopes, functions, Closures, calls, loops, match, raise, catch, finally, and `ExceptionContext`. |
| 6 | [05-types-contracts-generics.md](../../spec/iris-v1/05-types-contracts-generics.md) | Gradual typing, `Dynamic`, unions, intersections, nilability, casts, Contracts, generics, and `Never`. |
| 7 | [06-collections-text-regex.md](../../spec/iris-v1/06-collections-text-regex.md) | Tuple, Array, Hash, Range, iteration, String, MutableString, Symbol, Bytes, Regex, and stable hashing. |
| 8 | [07-async-resources-diagnostics.md](../../spec/iris-v1/07-async-resources-diagnostics.md) | `Task`, `Awaitable`, scheduler behavior, `using`, Closeable cleanup, diagnostics, and revision events. |
| 9 | [08-modules-metaprogramming.md](../../spec/iris-v1/08-modules-metaprogramming.md) | Packages, imports, exports, open transactions, static and dynamic members, decorators, MetaCapabilities, and ReflectionPolicy. |
| 10 | [09-native-host-ffi.md](../../spec/iris-v1/09-native-host-ffi.md) | Stable C Host ABI, opaque handles, native metadata, native payloads, async completion, script FFI, and wrapper guidance. |
| 11 | [10-serialization-standard-library.md](../../spec/iris-v1/10-serialization-standard-library.md) | JSON and IrisValue scope, Encoding API, core package boundary, and standard-library deferrals. |
| 12 | [11-migration-divergence.md](../../spec/iris-v1/11-migration-divergence.md) | Every intentional divergence and migration ledger row. |
| 13 | [12-conformance.md](../../spec/iris-v1/12-conformance.md) | Vector schema, categories, applicability, expected observations, coverage rules, and freeze gates. |
| 14 | [traceability-matrix.md](../../spec/iris-v1/traceability-matrix.md) | Mapping from frozen decisions to clauses, examples, vectors, and deferrals. |

## Suggested reading paths

If you want language semantics, read README, identity, grammar, runtime, control, types, async, and metaprogramming. That path explains how programs mean what they mean.

If you want to understand ordinary programming style, read runtime, control, types, collections, async, and then metaprogramming. You can skim the grammar until you need exact precedence or literal rules.

If you want to work on tooling, read README, identity, grammar, conformance, and traceability first. Tooling work needs exact IDs, artifact boundaries, vector categories, and freeze gates before it needs prose examples.

If you want native integration, read identity, types, async, metaprogramming, and FFI in that order. The native chapter depends on the same Contract, ReflectionPolicy, MetaCapabilities, and Task rules as source code.

## FFI and host embedding

Iris v1 has two native-adjacent surfaces, and they are deliberately separate.

The Host ABI is the stable C boundary for embedding or extending an Iris runtime. It uses opaque runtime-rooted handles, status plus result records, and runtime-thread affinity for heap-touching calls. Rust and C++ wrappers may exist, but they are convenience layers over the C ABI, not the stable binary identity.

Script FFI is the Iris source-level path for external binary calls. It goes through `FFI.open`, verified signatures, and `FFI::Library` objects. Scripts don't call Host ABI tables, extension tables, raw loader APIs, or arbitrary symbols directly.

Native async work returns `Task<T>` and completes through a runtime-owned completion token posted back to the runtime thread. Worker threads don't read Iris handles or touch the managed heap.

## Conformance

The conformance chapter doesn't ship a runner. It defines the future record shape and coverage rules. A conformance vector is a stable JSON record with an ID, source clauses, input, applicability, expected observations, and tags.

Important points for readers:

| Rule | Meaning |
| --- | --- |
| Vector IDs are stable | `IRIS-V1-<chapter>-V<nnn>` is the identity, not the human-readable name. |
| Categories are fixed | Vectors are `positive`, `negative`, `diagnostic`, or `differential`. |
| Expected observations are semantic | Tools compare Iris values, types, diagnostics, statuses, artifacts, and side effects, not heap addresses or debug strings. |
| Backend differences are not allowed | Interpreter, JIT, native, and host paths must agree where they apply. |
| Deferred features aren't behavior | A deferred item can have a rejection or diagnostic vector, not a vector pretending the feature exists. |

## Static promise, dynamic freedom

Dynamic freedom: the specification allows implementations to choose interpreter, JIT, AOT, allocation, GC, caches, and host wrappers, as long as observable Iris behavior is unchanged.

Static promise: the spec inventory, clause IDs, package identity, Type identity, conformance schema, Host ABI role, and backend-independent semantics are stable enough for future tools and implementations to target.

## Read the spec

For exact rules, start with [README.md](../../spec/iris-v1/README.md), then follow these clauses:

| Clause | Topic |
| --- | --- |
| `IRIS-V1-TRACE-C014` | The fixed 14-artifact inventory. |
| `IRIS-V1-TRACE-C009` | The spec dependency reading order. |
| `IRIS-V1-TRACE-C016` | Canonical terminology. |
| `IRIS-V1-IDENTITY-C019` through `IRIS-V1-IDENTITY-C023` | Implementation independence and C Host ABI identity. |
| `IRIS-V1-FFI-C003` through `IRIS-V1-FFI-C006` | Stable boundary roles. |
| `IRIS-V1-FFI-C043` through `IRIS-V1-FFI-C050` | Script FFI library surface. |
| `IRIS-V1-CONFORMANCE-C003` through `IRIS-V1-CONFORMANCE-C018` | Conformance terms, IDs, categories, and schema shape. |
| `IRIS-V1-CONFORMANCE-C046` through `IRIS-V1-CONFORMANCE-C049` | Required vector classes by chapter. |
