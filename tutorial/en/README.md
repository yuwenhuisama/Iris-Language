# Iris Tutorial

This tutorial is a practical introduction to Iris v1 for working programmers. It teaches the language model through concrete programs, verified execution on the Iris register machine, and direct citations into the frozen specification.

The Iris language syntax is stable: the v1 specification is frozen, and owner-approved errata through v1.33 shape the current forms of callable Types, stored properties, `shared` class state, `typeof`, generators, and decorators. These chapters show current spelling throughout.

The implementation under [`crates/`](../../crates/) provides an `iris` binary that includes both an ahead-of-time bytecode register machine (`--vm`) and a tree-walking reference engine. While the compiler and register machine cover core language constructs today, Iris is not yet a complete production toolchain.

**Quickstart and Prerequisites**

Building Iris requires a stable Rust toolchain (see [`rust-toolchain.toml`](../../rust-toolchain.toml)). Clone the repository and build the CLI binary from the workspace root:

```bash
git clone https://github.com/yuwenhuisama/Iris-Language.git
cd Iris-Language
cargo build -p iris-cli
```

On Unix systems this produces `./target/debug/iris`. On Windows environments it creates `.\target\debug\iris.exe`.

Before running either `hello.iris` command below, create that file using the program in [chapter 01](01-getting-started.md). The `-e` commands need no script file.

```bash
./target/debug/iris --vm hello.iris    # execute script on the register machine
./target/debug/iris hello.iris         # execute script on the reference engine (default)
./target/debug/iris --vm -e 'print(1 + 2)' # execute expression on the VM
./target/debug/iris -e 'print(1 + 2)'  # execute expression on the reference engine
./target/debug/iris                    # interactive reference REPL session, :quit to leave
```

**Understanding Example Metadata and Validation**

Every code example in this tutorial is paired with an exact verification contract immediately preceding its code fence:

- `vm`: Executed directly on `./target/debug/iris --vm` with expected exit code 0 and exact stdout.
- `reference`: Executed on `./target/debug/iris` with expected exit code 0 and exact stdout.
- `expected-error`: Runs with a specified engine (`vm` or `reference`), expecting a nonzero exit code and matching stderr diagnostic substring.
- `spec-only`: Illustrates normative syntax or semantics specified in the language standard, accompanied by visible explanatory prose.

Authored `print(...)` functions and collection helper methods demonstrated throughout these lessons are runtime conveniences for verified teaching; they do not constitute a frozen standard library.

The defining principle of Iris is stated in `IRIS-V1-IDENTITY-C008`: static promises bound dynamic behavior. Every runtime value is an object and dispatch happens at runtime, while types, Contracts, generic constraints, Class identity, package identity, and metaprogramming policies remain static promises that dynamic behavior has to respect.

## Chapters

The tutorial is organized into three progressive stages:

**Start**

- [01. What Iris Is](01-getting-started.md): The core mental model, running scripts with `--vm`, and the dynamic-static contract.
- [02. Values and Bindings](02-values-and-bindings.md): Local bindings (`let`, `mut`), shared storage, literal forms, and truthiness.

**Core**

- [03. Control Flow](03-control-flow.md): Value-producing `if` expressions, `while` loops with `break`, `for` iteration, and pattern matching.
- [04. Functions, Closures, Blocks](04-callables-and-closures.md): Named Methods, anonymous Closures, trailing block channels, and invocation via `.call`.
- [05. Classes and Objects](05-classes-and-objects.md): Classes, instances, stored properties, `@ivar` storage, inheritance, and identity versus equality.
- [06. Modules and Contracts](06-modules-and-contracts.md): Mixin composition, method lookup order, and Contract-qualified dispatch.

**Advanced**

- [07. Gradual Types](07-types-and-generics.md): Type annotations, union types, nilability, invariance, and generic constraints.
- [08. Errors and Resources](08-errors-and-resources.md): Exception handling with `try`/`catch`, deterministic cleanup, and tasks.
- [09. Dynamic Meets Static](09-dynamic-and-static.md): Reopening classes, atomic revision transactions, and reflection policies.
- [10. Where To Next](10-where-to-next.md): Navigating the 14 specification artifacts, C ABI host embedding, and conformance vectors.

## How to read this tutorial

Read chapters 01 through 06 first to master the core object model and composition mechanics. Chapters 07 through 09 explain how gradual typing, error handling, and metaprogramming operate under bounded dynamism. Chapter 10 connects your practical knowledge directly to the formal specification artifacts, conformance suite, and embedding interfaces.

Every runnable example in chapters 01 through 05 includes verified expected output and specifies its execution engine via metadata labels (`vm` or `reference`). Each chapter concludes with a "Read the spec" section listing the normative clauses that define language behavior. If this tutorial sounds simpler than the spec, trust the spec.
