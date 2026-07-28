# Functions, Closures, Blocks

This chapter explains Iris callables. A named `fun` declaration creates a Method, not a separate function object. Reading a Method from a receiver creates a BoundMethod. A Closure literal creates an anonymous callable with lexical capture. Trailing blocks are Closure literals passed through a dedicated `&block` channel.

```iris
class Counter {
  fun initialize() -> Nil { @value = 0 }
  fun add(delta: Integer) -> Integer { @value += delta }
}

let counter = Counter.new()
let bound: (Integer) -> Integer = counter.add
let closure: (Integer) -> Integer = { |delta: Integer| -> Integer; counter.add(delta) }
```

This snippet is adapted from `IRIS-V1-CONTROL-EX003`.

## Named fun declarations create Methods

A Method declaration starts with `fun`, optionally preceded by modifiers such as visibility, `override`, `impl`, `async`, or `class` in the grammar positions where they are allowed. Placement determines ownership. Inside a Class, `fun` creates an instance Method. `class fun` creates a singleton Method on the Class object. Inside a Module body, top-level `fun` attaches to that Module's `main` receiver.

```iris
class Greeter {
  fun initialize(name: String) -> Nil { @name = name }
  fun label() -> String { @name }
  fun rename(name: String) -> Nil { @name = name }
}
```

## Parameters and blocks have explicit channels

Parameter syntax is explicit. Required positional parameters are `name: Type`. Optional positional parameters add `= default`. Rest positional parameters use `*items: Type`. Keyword-only parameters start with `key`. Keyword rest uses `**options: Type`. The block channel uses `&block: (P) -> R` and can be optional with `= nil`.

```iris
fun render(
  title: String,
  count: Integer = 1,
  *items: String,
  key path: String,
  &block: (String) -> Nil = nil
) -> Nil {
  if block != nil { block(title) }
}
```

Calls use parentheses for ordinary arguments. Keyword arguments are spelled `name: value`. A trailing Closure after the call is not a final positional argument; it is supplied through the target's `&block` parameter.

```iris
render("report", path: "out.txt") { |line: String| -> Nil
  line.to_string()
}
```

## Closure literals are anonymous callables

Closure literals use `{ |parameters| -> ReturnType body }`. A single-line body starts after a semicolon. A multiline body starts after the header terminator. Empty parameters use `||`. A Closure return annotation may be omitted only when an expected callable type supplies one closed function type, so tutorial examples write it explicitly.

```iris
let identity: (String) -> String = { |value: String| -> String; value }
let answer: () -> Integer = { || -> Integer; 42 }
```

## Closures capture lexical cells

Closures capture lexical binding cells by reference. If the captured binding is mutable, writes through the Closure and writes through the outer scope operate on the same cell. A Closure created in receiver-owning code also captures the current receiver relation, so raw `@name` keeps referring to that receiver.

```iris
mut total: Integer = 0
let add: (Integer) -> Integer = { |value: Integer| -> Integer
  total += value
  return total
}
```

This snippet is adapted from `IRIS-V1-CONTROL-EX005`.

## Return stays inside the current callable

`return` is local to the current callable. In a Method, it exits that Method frame. In a Closure, it exits that Closure invocation only. A Closure can't use `break` or `continue` to jump across its call boundary into an outer loop.

```iris
fun choose(value: Integer) -> Integer {
  let normalize: (Integer) -> Integer = { |item: Integer| -> Integer
    if item < 0 { return 0 }
    item
  }
  normalize(value)
}
```

## Static promise, dynamic freedom

Callable types are written `(P1, P2) -> R` after binding. That type describes the call shape and result promise. The callable value may be a BoundMethod or a Closure, but argument checks, return checks, block shape, keyword names, and arity still follow the declared contract.

This model sets up [Classes and Objects](05-classes-and-objects.md): Classes install Methods, instances produce BoundMethods, and Closures let behavior travel while keeping lexical captures explicit.

## Read the spec

This chapter simplifies these normative clauses:

- [`IRIS-V1-CONTROL-C014`](../../spec/iris-v1/04-bindings-callables-control-flow.md): Method, BoundMethod, and Closure are the callable runtime kinds.
- [`IRIS-V1-CONTROL-C015`](../../spec/iris-v1/04-bindings-callables-control-flow.md): named Method syntax and ownership.
- [`IRIS-V1-CONTROL-C016`](../../spec/iris-v1/04-bindings-callables-control-flow.md): Closure syntax.
- [`IRIS-V1-CONTROL-C017`](../../spec/iris-v1/04-bindings-callables-control-flow.md): omitted Method and Closure return annotation rules.
- [`IRIS-V1-CONTROL-C019`](../../spec/iris-v1/04-bindings-callables-control-flow.md): normal return and final expression values.
- [`IRIS-V1-CONTROL-C020`](../../spec/iris-v1/04-bindings-callables-control-flow.md): callable-local return and invalid loop transfer across Closures.
- [`IRIS-V1-CONTROL-C022`](../../spec/iris-v1/04-bindings-callables-control-flow.md): parameter declaration order.
- [`IRIS-V1-CONTROL-C026`](../../spec/iris-v1/04-bindings-callables-control-flow.md): call argument binding.
- [`IRIS-V1-CONTROL-C028`](../../spec/iris-v1/04-bindings-callables-control-flow.md): Closure capture by reference.
- [`IRIS-V1-CONTROL-C030`](../../spec/iris-v1/04-bindings-callables-control-flow.md): trailing Closure block channel.
- [`IRIS-V1-TYPES-C036`](../../spec/iris-v1/05-types-contracts-generics.md): callable Types.
