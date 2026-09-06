# Where To Next

This tutorial provides the conceptual on-ramp for Iris. The formal specification is the definitive authority. This concluding chapter maps the 14 specification artifacts, outlines targeted reading paths, and details the core contracts governing host embedding, the foreign function interface (FFI), and test conformance.

We can bring together modules, contracts, gradual types, and runtime objects in an executable capstone program:

<!-- iris-example: {"id":"10-capstone-conformance","mode":"vm","stdout":"Record::active\n2026-v1\ntrue\n"} -->
```iris
contract Describable {
  fun describe() -> String
}

module Timestamped {
  public fun stamp() -> String {
    "2026-v1"
  }
}

class Record for Describable mixin Timestamped {
  public impl fun describe() -> String {
    "Record::active"
  }
}

let rec = Record.new()
let desc = rec as Describable

print(desc..describe())
print(rec.stamp())
print(rec is Record)
```

Expected terminal output:

```text
Record::active
2026-v1
true
```

## The 14 spec artifacts

The Iris v1 specification consists of 14 stable documents. Each chapter carries an immutable status indicating frozen semantics and approved errata:

| Order | Spec artifact | Read it for |
| --- | --- | --- |
| 1 | [README.md](../../spec/iris-v1/README.md) | Inventory, editorial conventions, clause identification rules, and dependency graph. |
| 2 | [01-language-identity.md](../../spec/iris-v1/01-language-identity.md) | Dynamic/static division, implementation independence, package identity, and deferred features. |
| 3 | [02-lexical-grammar.md](../../spec/iris-v1/02-lexical-grammar.md) | Contextual tokens, literal precision rules, operator precedence, associativity, and grammar. |
| 4 | [03-runtime-object-model.md](../../spec/iris-v1/03-runtime-object-model.md) | Object semantics, identity, method dispatch, classes, modules, MRO linearization, and revisions. |
| 5 | [04-bindings-callables-control-flow.md](../../spec/iris-v1/04-bindings-callables-control-flow.md) | Lexical bindings, scopes, callables, closures, iteration loops, pattern matching, and `ExceptionContext`. |
| 6 | [05-types-contracts-generics.md](../../spec/iris-v1/05-types-contracts-generics.md) | Gradual type checking, unions, nilability, casts, contracts, reified invariant generics, and `Never`. |
| 7 | [06-collections-text-regex.md](../../spec/iris-v1/06-collections-text-regex.md) | Arrays, tuples, hashes, ranges, string encoding, symbols, byte buffers, regex, and stable hashing. |
| 8 | [07-async-resources-diagnostics.md](../../spec/iris-v1/07-async-resources-diagnostics.md) | Single-threaded `Task`, async scheduling, `Closeable` resources, diagnostics, and lifecycle tracking. |
| 9 | [08-modules-metaprogramming.md](../../spec/iris-v1/08-modules-metaprogramming.md) | Package namespacing, open transactions, member visibility, decorators, `meta deny`, and `ReflectionPolicy`. |
| 10 | [09-native-host-ffi.md](../../spec/iris-v1/09-native-host-ffi.md) | C Host ABI, opaque handles, native payloads, thread affinity, and script-level FFI. |
| 11 | [10-serialization-standard-library.md](../../spec/iris-v1/10-serialization-standard-library.md) | JSON serialization, IrisValue encoding, core package boundaries, and standard library limits. |
| 12 | [11-migration-divergence.md](../../spec/iris-v1/11-migration-divergence.md) | Recorded architectural migrations and divergence from earlier legacy designs. |
| 13 | [12-conformance.md](../../spec/iris-v1/12-conformance.md) | Test vector schemas, validation categories, applicability rules, and freeze gates. |
| 14 | [traceability-matrix.md](../../spec/iris-v1/traceability-matrix.md) | Bidirectional mapping from design decisions to normative clauses and tests. |

## Suggested reading paths

Different domains benefit from focused reading paths through the specification:

- **Language Semantics**: Read `README.md`, `01-language-identity.md`, `02-lexical-grammar.md`, `03-runtime-object-model.md`, `04-bindings-callables-control-flow.md`, `05-types-contracts-generics.md`, `07-async-resources-diagnostics.md`, and `08-modules-metaprogramming.md`.
- **Application Engineering**: Focus on `03-runtime-object-model.md`, `04-bindings-callables-control-flow.md`, `05-types-contracts-generics.md`, `06-collections-text-regex.md`, and `07-async-resources-diagnostics.md`.
- **Tooling and Engine Development**: Begin with `README.md`, `01-language-identity.md`, `02-lexical-grammar.md`, `12-conformance.md`, and `traceability-matrix.md`.
- **Native Extensions and Embedding**: Read `01-language-identity.md`, `05-types-contracts-generics.md`, `07-async-resources-diagnostics.md`, `08-modules-metaprogramming.md`, and `09-native-host-ffi.md`.

## FFI and host embedding

Iris strictly separates runtime embedding from script-level foreign function calling.

The Host ABI provides a stable C interface for host processes embedding an Iris engine. It uses opaque, runtime-rooted handles and enforces runtime-thread affinity for heap-allocated objects. Rust and C++ wrapper APIs serve as convenient language bindings over this C boundary rather than replacing it.

**Specification-only (not executed):** The script-level `FFI` subsystem specifies calls to external shared libraries. This illustration requires `mathlib` and its `mathlib.ffi` declaration file, neither supplied here; it does not demonstrate current CLI FFI support:

<!-- iris-example: {"id":"10-ffi-specification","mode":"spec-only","reason":"Specification example illustrating script-level FFI library loading and invocation syntax"} -->
```iris
let library = FFI.open("mathlib", declarations: "mathlib.ffi")
let value = library.hypot(3.0f64, 4.0f64)
library.close()
```

Script code never handles raw memory addresses or unmanaged pointers directly. Native asynchronous operations post completion events back to the primary Iris runtime thread.

## Conformance

The Iris specification defines conformance requirements and vector formats; it is separate from the repository's executable `iris-conformance` runner. JSON vectors live under [`conformance/iris-v1/`](../../conformance/iris-v1/). The runner reports implementation evidence, including passing cases, active gaps, and cases requiring unfinished subsystems or diagnostic expectations. A run is not a claim of complete conformance.

Run grammar vectors using Cargo:

```bash
cargo run -p iris-conformance -- --chapter GRAMMAR
```

The specification sets the following conformance rules; they are requirements, not a claim that every row currently passes:

| Rule | Meaning |
| --- | --- |
| Stable Vector Identifiers | Vectors use immutable IDs in the format `IRIS-V1-<chapter>-V<nnn>`. |
| Categorized Observations | Tests are grouped into `positive`, `negative`, `diagnostic`, and `differential` categories. |
| Semantic Value Comparison | Assertions verify language values, exceptions, and side effects rather than internal memory addresses. |
| Cross-Backend Parity | Applicable implementations must agree on semantic output. The repository has a VM and reference evaluator; this does not imply an available native compilation engine. |
| Deferred Features Marked | Deferred capabilities carry explicit rejection vectors rather than simulated implementations. |

## Static promise, dynamic freedom

Iris delivers an open architecture grounded in reliable guarantees.

**Dynamic freedom**
- Implementers can develop alternative runtimes, register VMs, JIT compilers, or AOT engines.
- Scripts can use dynamic message dispatch and transactional metaprogramming.
- Host environments can bind native extensions dynamically through safe handle abstractions.

**Static promises**
- The 14 specification artifacts and clause identifiers remain permanent references.
- Test vectors state semantic expectations; runner results show how an implementation measures against them.
- The C Host ABI preserves ABI stability across toolchain releases.
- Invariant type semantics and contract obligations remain guaranteed across all execution environments.

**Hands-on Exercise**

Inspect the conformance vectors in `conformance/iris-v1/vectors/runtime/`. Pick a simple vector, read its source clauses and input assertions, and run `cargo run -p iris-conformance -- --chapter RUNTIME`. Compare the report with the vector's expectations and note any gaps or unsupported cases rather than assuming all rows pass.

## Read the spec

To begin exploring the normative specification, start with [README.md](../../spec/iris-v1/README.md) and review the following foundational clauses:

| Clause | Topic |
| --- | --- |
| `IRIS-V1-TRACE-C014` | The fixed 14-artifact specification inventory. |
| `IRIS-V1-TRACE-C009` | Specification dependency order and navigation. |
| `IRIS-V1-TRACE-C016` | Normative terminology and definitions. |
| `IRIS-V1-IDENTITY-C019` through `IRIS-V1-IDENTITY-C023` | Implementation independence and C Host ABI identity. |
| `IRIS-V1-FFI-C003` through `IRIS-V1-FFI-C006` | Boundary separation between host embedding and script FFI. |
| `IRIS-V1-FFI-C043` through `IRIS-V1-FFI-C050` | Script FFI library and declaration contracts. |
| `IRIS-V1-CONFORMANCE-C003` through `IRIS-V1-CONFORMANCE-C018` | Vector schemas, categories, and test validation rules. |
| `IRIS-V1-CONFORMANCE-C046` through `IRIS-V1-CONFORMANCE-C049` | Mandatory vector coverage requirements by chapter. |
