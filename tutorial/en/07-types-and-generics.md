# Gradual Types

This chapter explains Iris's type layer as a set of promises, not a separate execution model. You can leave code unannotated and get dynamic boundaries, or you can write annotations and make Iris enforce them statically where it can and at runtime where it must. You'll also see `Object`, unions, nilable `?`, type tests and casts, generics, invariance, and `Never`.

```iris
fun add(a: Integer, b: Integer) -> Integer {
  a + b
}

fun dynamic_add(a, b) {
  a + b
}

let total: Integer = add(1, 2)
let loose = dynamic_add("a", 3)
```

This example is reused from `IRIS-V1-TYPES-EX001`. `add` has written Contracts on its parameters and result. `dynamic_add` omits them, so those positions have `Dynamic<Object>` as their static and runtime Contract.

## Optional annotations still mean something

Iris is gradual. You don't have to annotate every local or Method, but once you write an annotation, it is both a static Contract and a runtime boundary guard. A binding annotation, parameter annotation, return type, property type, generic argument, and Contract requirement all have the same basic shape: if a violation is provable, it is diagnosed before execution; if it isn't provable, the boundary checks at runtime.

```iris
let name: String = "Iris"
mut count: Integer
count = 1

mut value: String | Integer = "ready"
value = 42
```

This snippet is reused from the binding examples. The union type says `value` can hold either `String` or `Integer`. It doesn't say the binding widens after assignment. The declared type is fixed for that binding.

## Object, Nil, and nilable types

`Object` is the top type. Every Iris value is an `Object`, including `nil`. That doesn't make every type nilable. A concrete type such as `String` excludes `nil` unless you write `String?` or `String | Nil`.

```iris
let name: String? = load_name()

if name != nil {
  let strong: String = name
}

let value: String | MutableString = load_text()
value.length()
```

This is reused from `IRIS-V1-TYPES-EX003`. `T?` is exact sugar for `T | Nil`, and Iris normalizes the two to the same type identity. A nil check can narrow the true path. Union member access works only when every branch has a safe common member and call shape.

## Tests and casts

Iris has three core type operations. `is` asks a question and returns `Bool`. `as` checks or proves a type and raises on failure. `as?` checks safely and returns `nil` on failure.

```iris
let object: Object = load_value()

if object is String {
  object.length()
}

let maybe_user: User? = object as? User
let printable = object as Printable
printable..print()
```

This example is reused from `IRIS-V1-TYPES-EX004`. These operations preserve the same value. They don't convert numbers, clone collections, change generic arguments, or choose a different ordinary Method. Contract casts create a checked Contract view, and qualified Contract calls still require `..`.

## Generics are reified and invariant

Generic Classes, Contracts, Modules, and Methods carry runtime-preserved arguments and metadata. A generic declaration can use a `where` clause to constrain type parameters.

```iris
contract Comparable<T> {
  fun compare(other: T) -> Integer
}

class SortedBox<T> where T: Comparable<T> & NonNil {
  property value: T
}

let names: SortedBox<String> = SortedBox<String>.new()
let objects: SortedBox<Object> = names  // rejected because generic Classes are invariant
```

This example is reused from `IRIS-V1-TYPES-EX008`. Invariance is the key rule: `SortedBox<String>` is not a subtype of `SortedBox<Object>` just because `String` is an `Object`. Casts don't cross invariant generic arguments either. If you want a converted container, you write conversion code.

Bare generic Class names are metadata, not instance types. For `class Box<T>`, `Box` names the generic definition object. Instance annotations and ordinary construction need a closed type such as `Box<String>` or a construction-site placeholder such as `Box<_>.new(value)`.

```iris
fun make<T>() -> T where T: Object {
  load_value() as T
}

let user: User = make()
let box = Box<_>.new(user)
```

This is reused from `IRIS-V1-TYPES-EX009`. Inference is local and bounded. Iris can use actual arguments and an immediate expected result type. It doesn't inspect Method bodies, later uses, or whole-program state.

## Never marks paths with no value

`Never` is the bottom type. It has no normal runtime values and is assignable to every type. Raising, bare re-raise, statically unreachable paths, and calls declared `-> Never` produce `Never`.

```iris
fun fail(message: String) -> Never {
  raise message
}

let value: String = if ready? {
  "ready"
} else {
  fail("not ready")
}
```

This example is reused from `IRIS-V1-TYPES-EX011`. The `else` branch doesn't widen the `if` result, because it doesn't produce a normal value.

## Static promise, dynamic freedom

Dynamic freedom: unannotated positions accept dynamic sends within their `Dynamic<Object>` boundary, `is` and `as?` can narrow based on runtime values, and generic materialization happens when closed arguments are needed.

Static promise: written annotations, Contract views, generic constraints, invariant arguments, nilability, and `Never` flow remain enforceable boundaries. None of them create overload dispatch or implicit conversion.

## Read the spec

For exact rules, read [05-types-contracts-generics.md](../../spec/iris-v1/05-types-contracts-generics.md):

| Clause | Topic |
| --- | --- |
| `IRIS-V1-TYPES-C003` through `IRIS-V1-TYPES-C007` | Gradual annotation boundaries and `Dynamic<Object>` defaults. |
| `IRIS-V1-TYPES-C008` through `IRIS-V1-TYPES-C017` | Type constructors, `Object`, `Nil`, `T?`, `Dynamic<T>`, aliases, and Type objects. |
| `IRIS-V1-TYPES-C018` through `IRIS-V1-TYPES-C027` | Union, intersection, nilability, `NonNil`, and same-name obligation rules. |
| `IRIS-V1-TYPES-C028` through `IRIS-V1-TYPES-C035` | `is`, `as`, `as?`, Contract views, and narrowing. |
| `IRIS-V1-TYPES-C054` through `IRIS-V1-TYPES-C074` | Generic declarations, constraints, inference, materialization, and invariance. |
| `IRIS-V1-TYPES-C080` through `IRIS-V1-TYPES-C083` | `Never` and unreachable flow. |
