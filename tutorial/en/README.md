# Iris Tutorial

This tutorial is a guided introduction to Iris v1 for working programmers. It explains the language model, shows grammar-checked examples, and points to the frozen specification when you need exact rules. Iris v1 has no usable implementation today. There is no compiler, interpreter, REPL, playground, package manager, or toolchain release for these chapters, so the examples are teaching artifacts, not commands to run.

The main idea to carry through the whole tutorial is simple: Iris is dynamic, but not unbounded. Every value is an object and dispatch happens at runtime, while types, Contracts, generic constraints, Class identity, package identity, and metaprogramming policies remain static promises that dynamic behavior has to respect.

## Chapters

| Chapter | Topic | What it covers |
| --- | --- | --- |
| [01. What Iris Is](01-getting-started.md) | Language identity | Every value as an object, operators as messages, and the honest implementation status. |
| [02. Values and Bindings](02-values-and-bindings.md) | Values and names | `let`, `mut`, `const`, annotations, literals, `nil`, and truthiness. |
| [03. Control Flow](03-control-flow.md) | Branches and loops | `if`, `while`, `for`, `match`, `break`, and `continue`. |
| [04. Functions, Closures, Blocks](04-callables-and-closures.md) | Callables | `fun`, parameters, return types, Closure literals, trailing blocks, and captures. |
| [05. Classes and Objects](05-classes-and-objects.md) | Object model | Classes, `self`, `@ivar`, inheritance, construction, identity, and equality. |
| [06. Modules and Contracts](06-modules-and-contracts.md) | Composition | `module` mixins, MRO order, `contract`, `for`, `impl`, Contract views, and `..` dispatch. |
| [07. Gradual Types](07-types-and-generics.md) | Type promises | Optional annotations, `Object`, unions, `?`, casts, generics, invariance, and `Never`. |
| [08. Errors and Resources](08-errors-and-resources.md) | Failure and cleanup | Raising any object, `try` and `catch`, `ExceptionContext`, `using`, `Closeable`, `Task`, and `await`. |
| [09. Dynamic Meets Static](09-dynamic-and-static.md) | Bounded mutation | `open class`, active revisions, atomic publish or rollback, `meta deny`, decorators, and reflection policy. |
| [10. Where To Next](10-where-to-next.md) | Spec handoff | A map of the 14 spec artifacts, a suggested reading path, FFI, host embedding, and conformance. |

## How to read this tutorial

Read chapters 01 through 05 first if you want the base language. Chapters 06 through 09 explain why Iris feels different from many object-oriented languages: composition, Contracts, gradual typing, and metaprogramming all interact through the same dynamic and static split. Chapter 10 hands you to the specification.

Every chapter ends with a "Read the spec" section. Those clause IDs are the source of truth. If this tutorial sounds simpler than the spec, trust the spec.
