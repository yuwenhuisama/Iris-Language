# Iris Programming Language

**A modern object-oriented scripting language with dynamic behavior bounded by static promises.**

[Website](https://yuwenhuisama.github.io/Iris-Language/) · [Tutorial (EN)](tutorial/en/README.md) · [教程 (中文)](tutorial/zh-cn/README.md) · [Specification](spec/iris-v1/README.md) · [中文规范](spec/iris-v1/zh-cn/README.md)

> **Status: frozen v1 specification with owner-approved errata; runnable Rust engine, not a released toolchain.**
> The language specification is frozen. The repository contains a working Rust runtime that includes a register bytecode virtual machine alongside a reference tree-walking evaluator, plus a C ABI boundary. It is an active development implementation rather than a packaged release: there is no package manager or general distribution yet.

---

## Quickstart

Requires a stable Rust toolchain (see [`rust-toolchain.toml`](rust-toolchain.toml)).

Build the CLI binary with Cargo:

```bash
cargo build -p iris-cli
```

Run code directly on the register bytecode VM:

```bash
./target/debug/iris --vm -e 'print("Hello, Iris!")'
```

Terminal output:

```text
Hello, Iris!
```

Create a script file named `hello.iris`:

<!-- iris-example: {"id":"readme-hello","mode":"vm","stdout":"Hello, Iris!\n"} -->
```iris
print("Hello, Iris!")
```

Run the script on the VM:

```bash
./target/debug/iris --vm hello.iris
```

Terminal output:

```text
Hello, Iris!
```

Running without `--vm` uses the reference evaluator. An interactive session without arguments also starts the reference REPL (`:quit` to exit):

```bash
./target/debug/iris hello.iris        # run via reference evaluator
./target/debug/iris -e 'print(1 + 2)'  # run source via reference evaluator
./target/debug/iris                    # start interactive REPL session
```

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

A full Simplified Chinese translation is available under [`spec/iris-v1/zh-cn/`](spec/iris-v1/zh-cn/README.md) as a reference translation. The English chapters are authoritative.

## Tutorial

The specification is precise, but the tutorial is where most programmers start. The guided tutorial spans ten chapters plus an index in both English and Simplified Chinese:

- **English**: [tutorial/en/README.md](tutorial/en/README.md)
- **Simplified Chinese (简体中文)**: [tutorial/zh-cn/README.md](tutorial/zh-cn/README.md)

| Chapter | Topic | What it covers |
| --- | --- | --- |
| [01. What Iris Is](tutorial/en/01-getting-started.md) | Getting started | Building the CLI, running scripts on the VM, language model, honest runtime status |
| [02. Values and Bindings](tutorial/en/02-values-and-bindings.md) | Values and names | `let`, `mut`, `const`, `shared`, annotations, literals, `nil`, truthiness |
| [03. Control Flow](tutorial/en/03-control-flow.md) | Branches and loops | `if` expressions, `while`, `for`, `match`, `break`, `continue`, `yield` |
| [04. Functions, Closures, Blocks](tutorial/en/04-callables-and-closures.md) | Callables | `fun`, `module fun`, parameters, `Closure<S>`, `Block<S>`, `.call`, captures |
| [05. Classes and Objects](tutorial/en/05-classes-and-objects.md) | Object model | Classes, `self`, `@ivar`, stored properties, inheritance, construction |
| [06. Modules and Contracts](tutorial/en/06-modules-and-contracts.md) | Composition | `module` mixins, MRO order, `contract`, `for`, `impl`, views, `..` dispatch |
| [07. Gradual Types](tutorial/en/07-types-and-generics.md) | Type promises | Optional annotations, `Object`, unions, `?`, casts, `typeof`, generics |
| [08. Errors and Resources](tutorial/en/08-errors-and-resources.md) | Failure and cleanup | Raising objects, `try`/`catch`, `ExceptionContext`, `using`, `Closeable`, `Task` |
| [09. Dynamic Meets Static](tutorial/en/09-dynamic-and-static.md) | Bounded mutation | `open class`, revisions, atomic publish/rollback, `meta deny`, reflection |
| [10. Where To Next](tutorial/en/10-where-to-next.md) | Spec handoff | Map of spec artifacts, reading path, FFI, host embedding, conformance |

Each chapter links back directly to the relevant frozen specification clauses.

Executable tutorial code snippets are verified through automated tooling, while spec-only and forward-looking snippets are explicitly excluded from execution runs. You can run the tutorial code snippet checker and its test suite:

```bash
node tools/check-tutorial.mjs
node --test tools/check-tutorial.test.mjs
```

## Runtime Architecture

The implementation lives in Rust crates under [`crates/`](crates/):

| Crate | Responsibility |
| --- | --- |
| `iris-lexer` | Source decoding, contextual tokenization, literal conversion, diagnostics |
| `iris-syntax` | Syntax tree and canonical parse-shape rendering |
| `iris-parser` | Recursive-descent declarations, Pratt expressions, static analysis |
| `iris-runtime` | Object model: Classes, revisions, MRO, dispatch, heap, GC tracing, stable hashing |
| `iris-eval` | Tree-walking evaluator, backend abstraction, differential observations |
| `iris-vm` | Register-based bytecode VM and verifier, running independently from the evaluator |
| `iris-cli` | The `iris` executable: VM runner, reference runner, and interactive REPL |
| `iris-conformance` | Conformance vector runner and milestone report |
| `iris-abi` | The stable `extern "C"` boundary, opaque handles, and unwinding barrier |

Notable properties implemented and tested:

- Correctly rounded `Float32` and `Float64` literal conversions with exact round-ties-to-even, independent of host locale or rounding mode (`IRIS-V1-GRAMMAR-C030`).
- Arbitrary-precision `Integer` literals with exact round-trip.
- Contextual tokenization resolving conflicts between range operators and Contract views, generic closers and right shift, and regex literals and division.
- The complete 17-row operator precedence table, including right-associative `**` and rejected non-associative chains.
- Logical Class identity across revisions, Module composition and MRO ordering, Contract views with qualified dispatch, and transactional publish-or-rollback.
- A moving collector that traces the whole object graph, storing identity hashes rather than deriving them from addresses.
- A register-based bytecode VM with three-address instructions and an ahead-of-time bytecode verifier that checks operand ranges, jump targets, and definite assignment before execution.

The bytecode VM declines unsupported language constructs explicitly rather than guessing or silently falling back to the reference evaluator.

## Building and Verification

Requires a stable Rust toolchain (see [`rust-toolchain.toml`](rust-toolchain.toml)).

```bash
# Build the workspace
cargo build

# Run unit and integration tests
cargo test --workspace

# Linting and formatting
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

### Conformance Suite

Specification conformance vectors are kept under [`conformance/iris-v1/`](conformance/iris-v1/). You can execute conformance vectors by chapter:

```bash
cargo run -p iris-conformance -- --chapter RUNTIME
```

Results track passing vectors, active gaps, and vectors requiring unfinished subsystems or author-specified diagnostic expectations. Conformance vectors are evidence: they are never modified or reclassified merely to inflate passing counts.

## Repository Layout

```text
spec/iris-v1/          Frozen v1 specification, English and Simplified Chinese
tutorial/              Guided tutorial, English and Simplified Chinese
crates/                Rust runtime implementation (VM, evaluator, runtime, CLI)
conformance/iris-v1/   Conformance vector corpus, schema, and classification
docs/                  Specification defect ledger, IR design, milestone status
legacy/                Archived prior C++ implementation, frozen and unmaintained
```

## Contributing

The specification is frozen. Discrepancies between the reference implementation and specification are documented in [`docs/spec-defects-v1.md`](docs/spec-defects-v1.md) with both readings and rationale.

## License

See [LICENSE](LICENSE).
