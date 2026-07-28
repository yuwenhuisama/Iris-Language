# Iris Programming Language

**A modern object-oriented scripting language with dynamic behavior bounded by static promises.**

[Website](https://yuwenhuisama.github.io/Iris-Language/) · [Specification](spec/iris-v1/README.md) · [中文规范](spec/iris-v1/zh-cn/README.md)

> **Status: frozen v1 draft specification, not a toolchain release.**
> The language specification is complete and frozen. The reference implementation is an in-progress frontend slice. Iris is not ready for production use, and no compatibility is promised until a toolchain release exists.

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

## Reference Implementation

The v1 implementation is written in Rust and lives under [`crates/`](crates/). Milestone 1 delivers a **frontend slice only** — lexing, parsing, literal conversion, and a conformance runner. There is no bytecode VM, object model, or standard library yet.

| Crate | Responsibility |
| --- | --- |
| `iris-lexer` | Source text handling, contextual tokenization, literal conversion |
| `iris-syntax` | Syntax tree and canonical parse-shape rendering |
| `iris-parser` | Recursive-descent declarations and Pratt expression parsing |
| `iris-eval` | Minimal literal evaluation |
| `iris-conformance` | Conformance vector runner and milestone report |
| `iris-abi` | Reserved for the C ABI boundary; intentionally empty in v1 |

Notable properties already implemented and tested:

- Correctly rounded `Float32` and `Float64` literal conversion with exact round-ties-to-even, independent of host locale and rounding mode, with no double rounding on the hexadecimal path (`IRIS-V1-GRAMMAR-C030`).
- Arbitrary-precision `Integer` literals with exact round-trip.
- Contextual tokenization resolving the frozen conflicts between range operators and Contract views, generic closers and right shift, and regex literals and division.
- The full 17-row operator precedence table, including right-associative `**` and rejected non-associative chains.

### Build and test

Requires a stable Rust toolchain; see [`rust-toolchain.toml`](rust-toolchain.toml).

```bash
cargo test --workspace
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

### Conformance

Vectors are derived from the frozen specification tables and committed under [`conformance/iris-v1/`](conformance/iris-v1/). Run the GRAMMAR chapter suite:

```bash
cargo run -p iris-conformance -- --chapter GRAMMAR
```

Current milestone 1 result across the 41 committed GRAMMAR vectors:

```
passed: 26, failed: 0, deferred: 1, authored_expect: 5, unrunnable_source: 9
```

The runner reports non-executable vectors in their own buckets and never counts them as passing:

- **authored-expect** — the frozen spec row names no stable diagnostic code, so the expectation is authored locally and is not treated as coverage.
- **deferred** — requires differential interpreter/JIT comparison that does not exist yet.
- **unrunnable_source** — the frozen spec row supplies prose instead of an executable fixture or expectation. See [`conformance/iris-v1/GRAMMAR-classification.md`](conformance/iris-v1/GRAMMAR-classification.md).

## Repository Layout

```text
spec/iris-v1/          Frozen v1 specification, English and Simplified Chinese
crates/                Rust reference implementation
conformance/iris-v1/   Conformance vector corpus, schema, and classification
docs/                  Specification defect ledger
legacy/                Archived prior C++ implementation, frozen and unmaintained
```

## Contributing

The specification is frozen; it is not edited to accommodate implementation convenience. If the implementation and the spec disagree, the discrepancy is recorded in [`docs/spec-defects-v1.md`](docs/spec-defects-v1.md) with both readings and the chosen behavior.

Conformance vectors are evidence. They are never weakened, reclassified, or special-cased to make a result look better.

## License

See [LICENSE](LICENSE).
