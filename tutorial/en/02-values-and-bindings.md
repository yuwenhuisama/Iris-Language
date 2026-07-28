# Values and Bindings

This chapter teaches the basic data you can write directly in Iris source and the local binding forms that hold it: `let`, `mut`, and `const`. You'll see how annotations become static and runtime contracts, why `nil` is a real object, and how truthiness works through `to_bool` instead of hard-coded condition rules.

```iris
let name: String = "Iris"
mut count: Integer
count = 1

mut value: String | Integer = "ready"
value = 42
```

This snippet is reused from `IRIS-V1-CONTROL-EX001`.

## Bindings choose the shape of a local name

The binding forms are deliberately small. `let` declares an immutable local and must have an initializer. `mut` declares a mutable local. A `mut` binding can defer initialization only when it has a written type. `const` uses the same grammar production as `let` and `mut`, but it names an immutable declaration-level binding rather than a mutable local cell.

```iris
let language = "Iris"
let answer: Integer = 42
mut state: Symbol = :ready
const Version: Integer = 1
```

## Annotations fix the local contract

If you omit a type annotation on a local binding, the initializer fixes the local type. Later assignments to a `mut` binding must still fit that fixed type. If you want a wider cell, write the wider type up front.

```iris
mut exact = "ready"
mut flexible: String | Integer = "ready"
flexible = 42
```

## Literal forms create objects

The common scalar literals are source forms for objects. Integer literals are arbitrary precision at the language level. Unsuffixed floating literals are `Float64`; suffixes choose `Float32` or `Float64`. Strings are immutable text values. `m"..."` creates a fresh `MutableString` identity. Symbols begin with `:` and represent immutable interned names.

```iris
let whole = 1_000
let ratio = 0.5f64
let title = "report"
let buffer = m"draft"
let tag = :ready
```

## Nil and collections are ordinary values

`nil`, `true`, and `false` are not special non-objects. They are singleton objects with specified runtime behavior. `nil` has Type `Nil`. A type such as `String` does not include `nil` unless you say so with `String?` or `String | Nil`.

```iris
let missing: String? = nil
let present: String | Nil = "name"
```

Collections also have literal forms, but their detailed behavior belongs later in the spec. At this stage, read them as object-producing expressions: arrays use `[...]`, hash literals use `%{ ... }`, and ranges use `..=` or `..<`.

```iris
let items = [1, 2, 3]
let table = %{ :name: "Iris", :version: 1 }
let closed = 1 ..= 3
let half_open = 1 ..< 3
```

## Truthiness goes through to_bool

Conditions use the dynamic `to_bool() -> Bool` protocol. Root `Object` is truthy by default. `nil` is falsy. `false` is falsy and `true` is truthy because Bool returns itself. The logical operators `&&` and `||` return one of their operands, not the converted Bool, and they evaluate the right side only when needed.

```iris
let chosen = if config.ready? { "ready" } else { nil }
let cached = value || compute_default()
let both = label && label.length()
```

This snippet is reused from `IRIS-V1-CONTROL-EX007`.

That truthiness rule is dynamic, but it is not a type predicate by itself. A truth test doesn't automatically turn a `String?` into a `String`. Use an explicit nil comparison, `is`, `as`, `as?`, typed `catch`, or match type pattern when you need flow narrowing.

```iris
let name: String? = load_name()

if name != nil {
  let strong: String = name
}
```

This snippet is adapted from `IRIS-V1-TYPES-EX003`.

## Static promise, dynamic freedom

A binding annotation is both a static promise and a runtime boundary guard. It doesn't freeze the object's behavior, choose an overload, or copy the value. It promises that anything stored in that cell satisfies the written type when the boundary is crossed.

Prefer `let` until you need reassignment. Reach for `mut` when the cell changes over time. Use `const` for immutable named declarations that should participate in the qualified declaration namespace.

## Read the spec

This chapter simplifies these normative clauses:

- [`IRIS-V1-GRAMMAR-C024`](../../spec/iris-v1/02-lexical-grammar.md): integer literal forms.
- [`IRIS-V1-GRAMMAR-C029`](../../spec/iris-v1/02-lexical-grammar.md): float suffixes.
- [`IRIS-V1-GRAMMAR-C037`](../../spec/iris-v1/02-lexical-grammar.md): `MutableString` literal creation.
- [`IRIS-V1-GRAMMAR-C040`](../../spec/iris-v1/02-lexical-grammar.md): array, hash, tuple, and range literal syntax.
- [`IRIS-V1-CONTROL-C003`](../../spec/iris-v1/04-bindings-callables-control-flow.md): `let` and `mut` binding declarations.
- [`IRIS-V1-CONTROL-C004`](../../spec/iris-v1/04-bindings-callables-control-flow.md): definite assignment and deferred `mut` rules.
- [`IRIS-V1-CONTROL-C005`](../../spec/iris-v1/04-bindings-callables-control-flow.md): binding annotations as fixed local contracts.
- [`IRIS-V1-CONTROL-C039`](../../spec/iris-v1/04-bindings-callables-control-flow.md): truthiness through `to_bool`.
- [`IRIS-V1-TYPES-C011`](../../spec/iris-v1/05-types-contracts-generics.md): `Nil` and nilability.
- [`IRIS-V1-TYPES-C012`](../../spec/iris-v1/05-types-contracts-generics.md): `T?` as `T | Nil`.
