# Iris Tutorial

This tutorial is a guided introduction to Iris v1 for working programmers. It explains the language model, shows grammar-checked examples, and points to the frozen specification when you need exact rules.

The syntax described here is stable: the v1 specification is frozen, and the owner-approved errata through v1.33 that shaped the current spelling of callable Types, stored properties, `shared` class state, `typeof`, generators, and decorators are folded into these chapters. Where a chapter shows a form the errata changed, it shows the current spelling only.

The tooling is not stable. There is an in-progress reference implementation in [`crates/`](../../crates/) with an `iris` binary that runs a script file or an interactive session, but it covers a subset of the language, and there is no package manager or toolchain release. Treat every code block here as a teaching artifact first; some run today, many do not.

```bash
cargo build -p iris-cli
./target/debug/iris hello.iris     # run a script
./target/debug/iris -e 'print(1 + 2)'
./target/debug/iris               # interactive session, :quit to leave
```

The main idea to carry through the whole tutorial is simple: Iris is dynamic, but not unbounded. Every value is an object and dispatch happens at runtime, while types, Contracts, generic constraints, Class identity, package identity, and metaprogramming policies remain static promises that dynamic behavior has to respect.

## Chapters

| Chapter | Topic | What it covers |
| --- | --- | --- |
| [01. What Iris Is](01-getting-started.md) | Language identity | Every value as an object, operators as messages, and the honest implementation status. |
| [02. Values and Bindings](02-values-and-bindings.md) | Values and names | `let`, `mut`, `const`, `shared`, annotations, literals, `nil`, and truthiness. |
| [03. Control Flow](03-control-flow.md) | Branches and loops | `if` as an expression, `while`, `for`, `match`, `break`, `continue`, and `yield` generators. |
| [04. Functions, Closures, Blocks](04-callables-and-closures.md) | Callables | `fun`, `module fun`, parameters, `Closure<S>` and `Block<S>` types, `.call`, and captures. |
| [05. Classes and Objects](05-classes-and-objects.md) | Object model | Classes, `self`, `@ivar`, stored properties, inheritance, construction, identity, and equality. |
| [06. Modules and Contracts](06-modules-and-contracts.md) | Composition | `module` mixins, MRO order, `contract`, `for`, `impl`, Contract views, and `..` dispatch. |
| [07. Gradual Types](07-types-and-generics.md) | Type promises | Optional annotations, `Object`, unions, `?`, casts, `typeof`, generics, invariance, and `Never`. |
| [08. Errors and Resources](08-errors-and-resources.md) | Failure and cleanup | Raising any object, `try` and `catch`, `ExceptionContext`, `using`, `Closeable`, `Task`, and `await`. |
| [09. Dynamic Meets Static](09-dynamic-and-static.md) | Bounded mutation | `open class`, active revisions, atomic publish or rollback, `meta deny`, decorators, and `Reflection::*`. |
| [10. Where To Next](10-where-to-next.md) | Spec handoff | A map of the 14 spec artifacts, a suggested reading path, FFI, host embedding, and conformance. |

## How to read this tutorial

Read chapters 01 through 05 first if you want the base language. Chapters 06 through 09 explain why Iris feels different from many object-oriented languages: composition, Contracts, gradual typing, and metaprogramming all interact through the same dynamic and static split. Chapter 10 hands you to the specification.

Every chapter ends with a "Read the spec" section. Those clause IDs are the source of truth. If this tutorial sounds simpler than the spec, trust the spec.
