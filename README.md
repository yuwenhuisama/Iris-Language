# Iris Programming Language

**A modern object-oriented scripting language with dynamic behavior bounded by static promises.**

[Website](https://yuwenhuisama.github.io/Iris-Language/) · [Tutorial](tutorial/en/README.md) · [Specification](spec/iris-v1/README.md) · [中文规范](spec/iris-v1/zh-cn/README.md)

> **Status: frozen v1 specification with owner-approved errata; the implementation runs but is not a toolchain release.**
> The specification is complete and frozen. The Rust reference implementation executes Iris programs through a tree-walking evaluator and a partial bytecode backend, and ships an `iris` binary with a script runner and REPL. Many language constructs are still unimplemented. Iris is not ready for production use, and no compatibility is promised until a toolchain release exists.

---

## What Iris Is

Iris is a dynamic, object-oriented scripting language designed for host embedding and native extension.

The defining idea is stated in `IRIS-V1-IDENTITY-C008`: **static promises bound dynamic behavior.** Declared superclasses, declared Contracts, visible member names and signatures, typed properties, generic constraints, and package identity are preserved across dynamic mutation, hot package upgrade, reflection, native binding, and optimization.

### Core principles

- **Everything is an object.** Every runtime value is an object and `Object` is the top type.
- **Behavior is message sending.** Operators, methods, named infix calls, property protocols, and Contract-qualified sends are all message or callable protocols.
- **Dispatch is runtime-dynamic.** Static annotations may validate and narrow a send, but they never silently select a different selector or overload. Iris v1 has **no** overload dispatch by static type.
- **Mutation is transactional.** Dynamic changes to Classes, Modules, Contracts, and packages go through validated candidate transactions with atomic publication or rollback. Invalid candidates never partially mutate live state.
- **Composition over multiple inheritance.** Single Class inheritance plus Modules and Contracts.
- **Built for embedding.** A stable C ABI boundary, opaque handles, and explicit signatures, with no raw managed pointer escape.

### Dynamic behavior, static promises

| Layer | May change dynamically | Remains promised |
| --- | --- | --- |
| Objects and Methods | Compatible method bodies, non-contract members, Module composition | Value objecthood, selector identity, declared Class and Contract promises |
| Types and Contracts | Runtime-checked views, `Dynamic` sends within bounds | No overload, invariant generics, Contract slot identity |
| Metaprogramming | Candidate revisions, open transactions, decorators | Static spine, MetaCapabilities, ReflectionPolicy, atomic publish |
| Packages | Same-major compatible hot upgrade | Package ID, API major identity, stable nominal type identity |
| Native and FFI | Extension implementation behind metadata | Stable C ABI, opaque handles, runtime-thread affinity |

## Specification

The v1 specification is **frozen**: its semantics are fixed and its clauses are stable identifiers. Defects found during implementation are recorded in [`docs/spec-defects-v1.md`](docs/spec-defects-v1.md) rather than silently patched.

Where implementation exposed a genuine gap, the owner approved errata clauses that supersede specific clauses **in place**. An errata clause always names what it supersedes, so no clause is silently deleted and every chapter's history stays auditable. Chapters carrying errata say so in their status line.

Every normative paragraph carries a clause ID such as `IRIS-V1-IDENTITY-C008`, so implementations, tests, and conformance vectors cite the exact requirement they satisfy.

| Chapter | Topic |
| --- | --- |
| [01](spec/iris-v1/01-language-identity.md) | Language identity, compatibility promise, deferrals |
| [02](spec/iris-v1/02-lexical-grammar.md) | Lexical grammar, tokens, precedence |
| [03](spec/iris-v1/03-runtime-object-model.md) | Runtime object model |
| [04](spec/iris-v1/04-bindings-callables-control-flow.md) | Bindings, callables, control flow |
| [05](spec/iris-v1/05-types-contracts-generics.md) | Types, Contracts, generics |
| [06](spec/iris-v1/06-collections-text-regex.md) | Collections, text, binary, regex, stable hashing |
| [07](spec/iris-v1/07-async-resources-diagnostics.md) | Async, resources, diagnostics |
| [08](spec/iris-v1/08-modules-metaprogramming.md) | Modules and metaprogramming |
| [09](spec/iris-v1/09-native-host-ffi.md) | Native host and FFI |
| [10](spec/iris-v1/10-serialization-standard-library.md) | Serialization and standard library boundary |
| [11](spec/iris-v1/11-migration-divergence.md) | Migration and divergence ledger |
| [12](spec/iris-v1/12-conformance.md) | Conformance framework |

A full Simplified Chinese translation is available under [`spec/iris-v1/zh-cn/`](spec/iris-v1/zh-cn/README.md) as a reference translation; the English chapters are authoritative.

## Tutorial

The specification is precise but not a starting point. The [tutorial](tutorial/en/README.md) is a guided introduction for working programmers: eleven chapters from the object model and bindings through control flow, callables, Classes, Modules and Contracts, gradual types, errors and resources, and bounded metaprogramming, ending with a map back into the spec.

Every chapter closes with the clause IDs it simplifies, so you can move from prose to normative text at any point. It is available in [English](tutorial/en/README.md) and [简体中文](tutorial/zh-cn/README.md).

## Reference Implementation

The v1 implementation is written in Rust and lives under [`crates/`](crates/). It now runs Iris programs: a tree-walking evaluator over a real object model, a garbage-collected heap, a partial bytecode backend that is differentially compared against the evaluator, a C ABI boundary, and an `iris` binary with a script runner and a REPL. It is not a toolchain release — there is no package manager, no standard-library distribution, and no stability promise.

| Crate | Responsibility |
| --- | --- |
| `iris-lexer` | Source decoding, contextual tokenization, literal conversion, diagnostics |
| `iris-syntax` | Syntax tree and canonical parse-shape rendering |
| `iris-parser` | Recursive-descent declarations, Pratt expressions, static analysis |
| `iris-runtime` | Object model: Classes, revisions, MRO, dispatch, heap, GC tracing, stable hashing |
| `iris-eval` | Tree-walking evaluator, backend abstraction, differential observations |
| `iris-vm` | Register-based bytecode backend and verifier; deliberately partial |
| `iris-cli` | The `iris` command: script runner and REPL |
| `iris-conformance` | Conformance vector runner and milestone report |
| `iris-abi` | The stable `extern "C"` boundary, opaque handles, and unwinding barrier |

Notable properties already implemented and tested:

- Correctly rounded `Float32` and `Float64` literal conversion with exact round-ties-to-even, independent of host locale and rounding mode, with no double rounding on the hexadecimal path (`IRIS-V1-GRAMMAR-C030`).
- Arbitrary-precision `Integer` literals with exact round-trip.
- Contextual tokenization resolving the frozen conflicts between range operators and Contract views, generic closers and right shift, and regex literals and division.
- The full 17-row operator precedence table, including right-associative `**` and rejected non-associative chains.
- Logical Class identity across revisions, Module composition and MRO ordering, Contract views and qualified dispatch, and transactional publish-or-rollback.
- A moving collector that traces the whole object graph, with identity hashes stored rather than derived from addresses.

The bytecode backend is intentionally partial: it compiles what it fully understands and declines everything else, because a backend that approximates would make a differential row agree for the wrong reason.

### Run it

Requires a stable Rust toolchain; see [`rust-toolchain.toml`](rust-toolchain.toml).

```bash
cargo build -p iris-cli
./target/debug/iris script.iris        # run a script
./target/debug/iris -e 'print(1 + 2)'  # run source from the command line
./target/debug/iris                    # interactive session, :quit to leave
```

The language is further along than the runner: many spec constructs still report `unsupported construct`. Start from the [tutorial](tutorial/en/README.md) for what the language means, and treat the binary as a partial implementation of it.

### Build and test

```bash
cargo test --workspace
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

### Conformance

Vectors are derived from the frozen specification tables and committed under [`conformance/iris-v1/`](conformance/iris-v1/). Run any chapter suite:

```bash
cargo run -p iris-conformance -- --chapter RUNTIME
```

Current result at this commit, across all thirteen chapters:

| Chapter | Passed | Failed | Other |
| --- | --- | --- | --- |
| RUNTIME | 194 | 0 | |
| CONTROL | 180 | 3 | |
| COLLECTIONS | 132 | 0 | 1 differential |
| TYPES | 103 | 1 | |
| META | 85 | 6 | 1 needs-subsystem |
| ASYNC | 56 | 0 | |
| FFI | 45 | 0 | |
| GRAMMAR | 40 | 0 | 1 deferred, 5 authored-expect, 9 unrunnable-source |
| CONFORMANCE | 40 | 0 | |
| LIBRARY | 29 | 0 | |
| IDENTITY | 20 | 0 | 1 needs-subsystem |
| TRACE | 8 | 0 | |
| MIGRATION | 6 | 0 | |
| **Total** | **938** | **10** | |

The ten failures are real and tracked, not suppressed. The runner reports non-executable vectors in their own buckets and never counts them as passing:

- **authored-expect** — the frozen spec row names no stable diagnostic code, so the expectation is authored locally and is not treated as coverage.
- **needs-subsystem** — the vector requires a subsystem this implementation does not have yet.
- **no-fixture / unrunnable_source** — the frozen spec row supplies prose instead of an executable fixture or expectation. See the per-chapter classification files under [`conformance/iris-v1/`](conformance/iris-v1/).
- **differential** — requires cross-backend comparison that is not yet available for that row.

## Repository Layout

```text
spec/iris-v1/          Frozen v1 specification, English and Simplified Chinese
tutorial/              Guided tutorial, English and Simplified Chinese
crates/                Rust reference implementation
conformance/iris-v1/   Conformance vector corpus, schema, and classification
docs/                  Specification defect ledger, IR design, milestone status
legacy/                Archived prior C++ implementation, frozen and unmaintained
```

## Contributing

The specification is frozen; it is not edited to accommodate implementation convenience. If the implementation and the spec disagree, the discrepancy is recorded in [`docs/spec-defects-v1.md`](docs/spec-defects-v1.md) with both readings and the chosen behavior.

Conformance vectors are evidence. They are never weakened, reclassified, or special-cased to make a result look better.

## License

See [LICENSE](LICENSE).
