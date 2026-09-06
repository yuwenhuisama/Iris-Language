# Values and Bindings

This chapter covers the basic data literals you can write in Iris source and the binding forms that store them: `let`, `mut`, `const`, and `shared`. You will learn how annotations establish fixed local contracts, how collections and `nil` behave as ordinary objects, and how truthiness evaluates through the `to_bool` protocol.

## Bindings choose the shape of a local name

Iris provides dedicated binding keywords:

- `let` introduces an immutable local variable. It requires an immediate initializer expression.
- `mut` introduces a mutable local variable whose value can be reassigned.
- `const` declares an immutable constant value at declaration level.

<!-- iris-example: {"id":"02-bindings","mode":"vm","stdout":"Iris\n2\n"} -->
```iris
let language = "Iris"
mut count = 1
count = count + 1
print(language)
print(count)
```

Expected output:

```text
Iris
2
```

Assigning to an immutable `let` binding is rejected by the compiler.

## Shared and global storage are declared, never conjured

Local variables live on the current execution stack frame. When multiple instances or methods need shared state, storage must be declared explicitly before use.

Within a Class or Module, `shared mut @@name` or `shared let @@name` declares storage anchored to that type in the class hierarchy. Similarly, `global let $name` or `global mut $name` declares package-level variables. Accessing or assigning to an undeclared `@@name` or `$name` produces a `MISSING_DECLARED_STORAGE` compile-time error.

<!-- iris-example: {"id":"02-shared","mode":"vm","stdout":"1\n2\n"} -->
```iris
class Counter {
  shared mut @@count: Integer = 0
  public fun bump() -> Integer {
    @@count = @@count + 1
  }
}
let c = Counter.new()
print(c.bump())
print(c.bump())
```

Expected output:

```text
1
2
```

## Annotations fix the local contract

When you write a type annotation on a binding, that type fixes the contract for all future assignments to the slot.

<!-- iris-example: {"id":"02-annotations","mode":"vm","stdout":"15\n"} -->
```iris
mut count: Integer = 10
count = count + 5
print(count)
```

Expected output:

```text
15
```

If a binding omits a type annotation, its type is inferred from the initial expression and remains fixed for that binding cell.

## Literal forms create objects

Every literal form produces an object:

- **Integers**: Arbitrary precision integer values (e.g. `1000`).
- **Floats**: `Float64` by default; suffixes like `f32` and `f64` explicitly specify width (e.g. `0.5f64`).
- **Strings**: Immutable text values enclosed in double quotes (e.g. `"report"`).
- **Symbols**: Interned immutable identifiers starting with a colon (e.g. `:ready`).

<!-- iris-example: {"id":"02-literals","mode":"vm","stdout":"1000\n0.5\nreport\nready\n"} -->
```iris
let count = 1000
let ratio = 0.5f64
let title = "report"
let tag = :ready
print(count)
print(ratio)
print(title)
print(tag)
```

Expected output:

```text
1000
0.5
report
ready
```

## Nil and collections are ordinary values

`nil`, `true`, and `false` are singleton objects, not primitive sentinels. `nil` is an instance of `Nil`.

Collections also produce standard objects:

- Arrays use bracket notation `[...]`.
- Maps use hash table notation `%{ ... }`.

<!-- iris-example: {"id":"02-nil-collections","mode":"vm","stdout":"2\nIris\n"} -->
```iris
let items = [1, 2]
let table = %{ :lang: "Iris" }
print(items.length())
print(table[:lang])
```

Expected output:

```text
2
Iris
```

## Truthiness goes through to_bool

Conditionals in Iris evaluate truthiness through the `to_bool() -> Bool` protocol.

- `Object` is truthy by default.
- `nil` is falsy.
- `false` is falsy, while `true` is truthy.
- Logical operators `||` and `&&` short-circuit and return the operand value itself rather than coercing to a boolean.

<!-- iris-example: {"id":"02-truthiness","mode":"vm","stdout":"active\nyes\n"} -->
```iris
let primary = nil
let fallback = "active"
let chosen = primary || fallback
print(chosen)
print(if "text" { "yes" } else { "no" })
```

Expected output:

```text
active
yes
```

Because `primary` is `nil` (falsy), the `||` operator returns the evaluated right-hand operand `"active"`.

## Static promise, dynamic freedom

Binding annotations act as boundary guarantees. They do not alter object representation or dispatch mechanics, but they ensure that invalid values cannot cross into typed locations.

**Hands-on Exercise**

Create a script `bindings_test.iris` that establishes an immutable binding to a mutable map `let config = %{ :port: 8080, :host: "localhost" }` (remember that `let` prevents reassigning the variable name but does not freeze map contents), declares a mutable variable `mut status: String = "starting"`, reassigns `status` to `"running"`, and prints both the `:port` from the map and `status`. Run with `./target/debug/iris --vm bindings_test.iris` to confirm output `8080` and `running`.

## Read the spec

This chapter simplifies the following normative clauses:

- [`IRIS-V1-GRAMMAR-C024`](../../spec/iris-v1/02-lexical-grammar.md): integer literal forms.
- [`IRIS-V1-GRAMMAR-C029`](../../spec/iris-v1/02-lexical-grammar.md): float literal formats and suffixes.
- [`IRIS-V1-GRAMMAR-C037`](../../spec/iris-v1/02-lexical-grammar.md): `MutableString` literal creation.
- [`IRIS-V1-GRAMMAR-C040`](../../spec/iris-v1/02-lexical-grammar.md): array, hash, tuple, and range literal syntax.
- [`IRIS-V1-CONTROL-C003`](../../spec/iris-v1/04-bindings-callables-control-flow.md): `let` and `mut` binding declarations.
- [`IRIS-V1-CONTROL-C004`](../../spec/iris-v1/04-bindings-callables-control-flow.md): definite assignment rules.
- [`IRIS-V1-CONTROL-C005`](../../spec/iris-v1/04-bindings-callables-control-flow.md): binding annotations as fixed local contracts.
- [`IRIS-V1-CONTROL-C009`](../../spec/iris-v1/04-bindings-callables-control-flow.md): undeclared shared or global storage failure.
- [`IRIS-V1-GRAMMAR-C059`](../../spec/iris-v1/02-lexical-grammar.md): `shared let` and `shared mut` declarations.
- [`IRIS-V1-RUNTIME-C162`](../../spec/iris-v1/03-runtime-object-model.md): shared cell creation and immutability.
- [`IRIS-V1-CONTROL-C039`](../../spec/iris-v1/04-bindings-callables-control-flow.md): truthiness evaluation through `to_bool`.
- [`IRIS-V1-TYPES-C011`](../../spec/iris-v1/05-types-contracts-generics.md): `Nil` and nilability.
- [`IRIS-V1-TYPES-C012`](../../spec/iris-v1/05-types-contracts-generics.md): `T?` as syntactic sugar for `T | Nil`.
