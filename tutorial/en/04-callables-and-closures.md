# Functions, Closures, Blocks

This chapter explains Iris callables. A named `fun` declaration creates a Method, not a separate function object. Reading a Method from a receiver creates a BoundMethod. A Closure literal creates an anonymous callable with lexical capture. Trailing blocks are Closure literals passed through a dedicated `&block` channel. Every callable value is invoked with `.call(...)`.

```iris
class Counter {
  property value: Integer = 0
  fun add(delta: Integer) -> Integer { @value += delta }
}

let counter = Counter.new()
let bound: BoundMethod<(Integer) -> Integer> = counter.add
let closure: Closure<(Integer) -> Integer> = { |delta: Integer| -> Integer; counter.add(delta) }

bound.call(1)
closure.call(2)
```

This snippet is adapted from `IRIS-V1-CONTROL-EX003`.

## Callable Types name their kind

A callable Type is never a bare signature. The signature `(P1, P2) -> R` is a component, and the Type wraps it in the kind of callable it describes:

| Type | Holds |
| --- | --- |
| `Closure<(P) -> R>` | a Closure value |
| `BoundMethod<(P) -> R>` | a BoundMethod value |
| `Block<(P) -> R>` | either of the two, for the block channel |

`Block<S>` is a language-core Type alias declared as `type Block<S> = BoundMethod<S> | Closure<S>`. It lives in the `Kernel` Module, which is composed into `Object`, so it is visible everywhere without an import.

Callable Type arguments are invariant, exactly like every other generic argument. `Closure<(Integer) -> Object>` is not assignable to `Closure<(Integer) -> Symbol>`, and neither is the reverse. Compatibility is checked at the call site against the invoked callable's declared signature, not through variance between callable Types.

## Named fun declarations create Methods

A Method declaration starts with `fun`, optionally preceded by modifiers such as visibility, `override`, `impl`, `async`, or `class` and `module` in the grammar positions where they are allowed. Placement determines ownership. Inside a Class, `fun` creates an instance Method. `class fun` creates a singleton Method on the Class object. Inside a Module declaration, `module fun` installs a Method on the Module object itself, while an unmodified `fun` is an instance Method supplied to whatever composes the Module.

```iris
module Config {
  module fun default_path() -> String { "iris.toml" }
  fun describe() -> String { "config" }
}

Config.default_path()
```

`class` and `module` are mutually exclusive on one declaration.

## Parameters and blocks have explicit channels

Parameter syntax is explicit. Required positional parameters are `name: Type`. Optional positional parameters add `= default`. Rest positional parameters use `*items: Type`. Keyword-only parameters start with `key`. Keyword rest uses `**options: Type`. The block channel uses `&block: Block<(P) -> R>` and can be optional with `= nil`.

```iris
fun render(
  title: String,
  count: Integer = 1,
  *items: String,
  key path: String,
  &block: Block<(String) -> Nil> = nil
) -> Nil {
  if block != nil { block.call(title) }
}
```

An omitted optional block binds `nil`, so `block != nil` is the presence test before `block.call(...)`.

Calls use parentheses for ordinary arguments. Keyword arguments are spelled `name: value`. A trailing Closure after the call is not a final positional argument; it is supplied through the target's `&block` parameter. Passing a Closure through both `&saved` and a trailing block in one call is an `ArgumentError`.

```iris
render("report", path: "out.txt") { |line: String| -> Nil
  line.to_string()
}
```

## Closure literals are anonymous callables

Closure literals use `{ |parameters| -> ReturnType body }`. A single-line body starts after a semicolon. A multiline body starts after the header terminator. Empty parameters use `||`. A Closure return annotation may be omitted only when an expected callable type supplies one closed function type, so tutorial examples write it explicitly.

```iris
let identity: Closure<(String) -> String> = { |value: String| -> String; value }
let answer: Closure<() -> Integer> = { || -> Integer; 42 }

answer.call()
```

## Invocation is always call

`call` is the sole invocation spelling for Closure, BoundMethod, and block values. You cannot apply an argument list directly to a callable value: `closure(1)` is not an invocation. Only a Method send, such as `f(1)` or `obj.m(1)`, uses bare call syntax, and ordinary sends always require the parentheses — `f 1` is rejected as `PARSE_CALL_REQUIRES_PARENTHESES`.

```iris
fun apply(value: Integer, &block: Block<(Integer) -> Integer>) -> Integer {
  block.call(value)
}

let doubled = apply(21) { |value: Integer| -> Integer; value * 2 }
```

Because `call` is an ordinary selector on an identity-bearing callable object, it dispatches like any other message and binds arguments under the ordinary parameter rules.

## Closures capture lexical cells

Closures capture lexical binding cells by reference. If the captured binding is mutable, writes through the Closure and writes through the outer scope operate on the same cell. A Closure created in receiver-owning code also captures the current receiver relation, so raw `@name` keeps referring to that receiver.

```iris
mut total: Integer = 0
let add: Closure<(Integer) -> Integer> = { |value: Integer| -> Integer
  total += value
  return total
}
```

This snippet is adapted from `IRIS-V1-CONTROL-EX005`.

A Method declaration is not a Closure. A Method that reads a name that only exists as a local in the surrounding body does not capture it; the declaration is accepted and the read raises `NameError` when the Method runs.

## Return stays inside the current callable

`return` is local to the current callable. In a Method, it exits that Method frame. In a Closure, it exits that Closure invocation only. A Closure can't use `break` or `continue` to jump across its call boundary into an outer loop; that is `CONTROL_TARGET_CROSSES_CLOSURE`.

```iris
fun choose(value: Integer) -> Integer {
  let normalize: Closure<(Integer) -> Integer> = { |item: Integer| -> Integer
    if item < 0 { return 0 }
    item
  }
  normalize.call(value)
}
```

When a Method omits `-> ReturnType`, its declared return Contract is `Dynamic<Object>`. The implementation may use the body's final expression for local diagnostics, but it never publishes an inferred narrower return type.

## Static promise, dynamic freedom

A callable Type describes the call shape, the result promise, and now the callable kind. The value behind it may be a BoundMethod or a Closure only where the Type admits that kind, and argument checks, return checks, block shape, keyword names, and arity still follow the declared contract.

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
- [`IRIS-V1-CONTROL-C074`](../../spec/iris-v1/04-bindings-callables-control-flow.md): `module fun` installation.
- [`IRIS-V1-CONTROL-C075`](../../spec/iris-v1/04-bindings-callables-control-flow.md): omitted return annotation is `Dynamic<Object>`.
- [`IRIS-V1-CONTROL-C076`](../../spec/iris-v1/04-bindings-callables-control-flow.md): `call` as the sole invocation spelling.
- [`IRIS-V1-TYPES-C094`](../../spec/iris-v1/05-types-contracts-generics.md): `Closure<S>` and `BoundMethod<S>` callable Types.
- [`IRIS-V1-TYPES-C095`](../../spec/iris-v1/05-types-contracts-generics.md): the `Block<S>` alias and the block channel.
- [`IRIS-V1-TYPES-C096`](../../spec/iris-v1/05-types-contracts-generics.md): callable Type arguments are invariant.
