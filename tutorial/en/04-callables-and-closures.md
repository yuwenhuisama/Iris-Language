# Functions, Closures, Blocks

This chapter explains how callable code is structured and invoked in Iris. A named `fun` declaration defines a Method on a class or module rather than an unattached global function. Referencing a method creates a BoundMethod. A closure literal creates an anonymous callable object that captures lexical bindings by reference. In all cases, callable values are invoked using `.call(...)`.

## Callable Types name their kind

In Iris, callable types explicitly state their specific runtime category:

| Type | Represents |
| --- | --- |
| `Closure<(P) -> R>` | An anonymous closure object |
| `BoundMethod<(P) -> R>` | A method bound to a receiver object |
| `Block<(P) -> R>` | A union alias (`BoundMethod<S> | Closure<S>`) for block arguments |

Type arguments for callables are invariant, consistent with all other generic types in Iris.

## Named fun declarations create Methods

Named functions are declared with the `fun` keyword. Ordinary `fun` inside a Class defines an instance Method; `class fun` installs on the Class object. Inside a Module declaration, `module fun` installs on the Module object, while ordinary `fun` supplies a mixin Method to a composed host (`IRIS-V1-CONTROL-C074`). This is distinct from a top-level `fun` in an executable Module body, which installs on that Module's `main` receiver (`IRIS-V1-CONTROL-C012`).

<!-- iris-example: {"id":"04-fun-declaration","mode":"vm","stdout":"36\n"} -->
```iris
module Math {
  public module fun square(n: Integer) -> Integer {
    n * n
  }
}
print(Math.square(6))
```

Expected output:

```text
36
```

Because named functions are methods belonging to a module or class, they participate in regular dynamic method dispatch.

## Parameters and blocks have explicit channels

Parameters in Iris follow clear syntactic channels:

- Required positional: `name: Type`
- Optional positional: `name: Type = default`
- Rest positional: `*items: Type`
- Trailing block channel: `&block: Block<(P) -> R>`

The block channel provides a dedicated route for passing execution blocks into methods.

**Reference-only example.** Save this program as `blocks.iris` and run `./target/debug/iris blocks.iris`. The current VM raises `NameError` for this typed block example; the reference engine executes it. The trailing closure supplies `&block`, not an extra positional argument, and the method invokes it through `.call(n)`.

<!-- iris-example: {"id":"04-trailing-block","mode":"reference","stdout":"6\n"} -->
```iris
module Helpers {
  public module fun apply(n: Integer, &block: Block<(Integer) -> Integer>) -> Integer {
    block.call(n)
  }
}
print(Helpers.apply(3) { |x: Integer| -> Integer; x * 2 })
```

Expected output:

```text
6
```

## Closure literals are anonymous callables

Closure literals use curly braces and pipe delimiters: `{ |params| -> ReturnType; body }`.

<!-- iris-example: {"id":"04-closure-literal","mode":"vm","stdout":"21\n"} -->
```iris
let mult = { |x: Integer| -> Integer; x * 3 }
print(mult.call(7))
```

Expected output:

```text
21
```

A closure is a full runtime object that can be stored in variables, passed to other methods, and invoked on demand.

## Invocation is always call

Callable values—whether closures, bound methods, or block arguments—are always invoked using the explicit message `.call(...)`.

Bare call syntax like `mult(7)` is not valid for callable variables; parentheses without an explicit selector are reserved for direct method sends on the receiver.

<!-- iris-example: {"id":"04-invocation-call","mode":"vm","stdout":"10\n"} -->
```iris
class Multiplier {
  public fun factor() -> Integer { 10 }
}
let m = Multiplier.new()
let bound = m.factor
print(bound.call())
```

Expected output:

```text
10
```

Extracting `m.factor` produces a `BoundMethod` object that retains `m` as its target receiver.

## Closures capture lexical cells

Closures capture enclosing variables by reference rather than by copying their values. Modifications made inside the closure update the original variable cell, and outside changes are visible inside the closure.

<!-- iris-example: {"id":"04-lexical-capture","mode":"vm","stdout":"15\n15\n"} -->
```iris
mut total = 10
let add = { |amount: Integer| -> Integer; total = total + amount; total }
print(add.call(5))
print(total)
```

Expected output:

```text
15
15
```

## Return stays inside the current callable

In Iris, `return` is strictly local to its immediately enclosing callable body.

Executing `return` inside a closure terminates only that closure invocation, returning a value to the caller of `.call(...)`. It does not jump out of the outer method frame.

<!-- iris-example: {"id":"04-return-local","mode":"vm","stdout":"0\n8\n"} -->
```iris
module Checker {
  public module fun test_val(n: Integer) -> Integer {
    let check = { |x: Integer| -> Integer; if x < 0 { return 0 }; x }
    check.call(n)
  }
}
print(Checker.test_val(-5))
print(Checker.test_val(8))
```

Expected output:

```text
0
8
```

## Static promise, dynamic freedom

Callable signatures establish static boundaries for parameter counts and types. However, because callables are objects, they can be dynamically swapped or passed as arguments while preserving interface contracts.

**Hands-on Exercise**

In `blocks.iris`, change the trailing closure to add `4` instead of multiplying by `2`, keeping its `Integer` parameter and return annotations. Run `./target/debug/iris blocks.iris` and confirm output `7`.

## Read the spec

This chapter simplifies the following normative clauses:

- [`IRIS-V1-CONTROL-C014`](../../spec/iris-v1/04-bindings-callables-control-flow.md): callable runtime kinds (Method, BoundMethod, Closure).
- [`IRIS-V1-CONTROL-C015`](../../spec/iris-v1/04-bindings-callables-control-flow.md): named Method syntax and ownership.
- [`IRIS-V1-CONTROL-C016`](../../spec/iris-v1/04-bindings-callables-control-flow.md): Closure literal syntax.
- [`IRIS-V1-CONTROL-C017`](../../spec/iris-v1/04-bindings-callables-control-flow.md): omitted return annotation defaults.
- [`IRIS-V1-CONTROL-C019`](../../spec/iris-v1/04-bindings-callables-control-flow.md): normal return and final expression values.
- [`IRIS-V1-CONTROL-C020`](../../spec/iris-v1/04-bindings-callables-control-flow.md): callable-local return semantics.
- [`IRIS-V1-CONTROL-C022`](../../spec/iris-v1/04-bindings-callables-control-flow.md): parameter declaration order.
- [`IRIS-V1-CONTROL-C026`](../../spec/iris-v1/04-bindings-callables-control-flow.md): call argument binding.
- [`IRIS-V1-CONTROL-C028`](../../spec/iris-v1/04-bindings-callables-control-flow.md): Closure capture by reference.
- [`IRIS-V1-CONTROL-C030`](../../spec/iris-v1/04-bindings-callables-control-flow.md): trailing Closure block channel.
- [`IRIS-V1-CONTROL-C074`](../../spec/iris-v1/04-bindings-callables-control-flow.md): `module fun` installation.
- [`IRIS-V1-CONTROL-C075`](../../spec/iris-v1/04-bindings-callables-control-flow.md): omitted return annotation contract is `Dynamic<Object>`.
- [`IRIS-V1-CONTROL-C076`](../../spec/iris-v1/04-bindings-callables-control-flow.md): `call` as the sole invocation spelling.
- [`IRIS-V1-TYPES-C094`](../../spec/iris-v1/05-types-contracts-generics.md): `Closure<S>` and `BoundMethod<S>` callable Types.
- [`IRIS-V1-TYPES-C095`](../../spec/iris-v1/05-types-contracts-generics.md): the `Block<S>` alias.
- [`IRIS-V1-TYPES-C096`](../../spec/iris-v1/05-types-contracts-generics.md): callable Type arguments are invariant.
