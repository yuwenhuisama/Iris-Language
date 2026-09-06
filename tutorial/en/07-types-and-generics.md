# Gradual Types

This chapter explains gradual typing in Iris as a system of enforceable promises rather than a rigid static checker. You can write untyped dynamic code, or add precise type annotations that Iris validates at compile time where possible and guards at runtime boundaries. We will explore `Object`, nilable types, type tests, casts, `typeof`, first-class type values, reified invariant generics, and the empty type `Never`.

<!-- iris-example: {"id":"07-gradual-types","mode":"vm","stdout":"30\n"} -->
```iris
module Adder {
  public module fun add(a: Integer, b: Integer) -> Integer {
    a + b
  }
}

let total: Integer = Adder.add(10, 20)
print(total)
```

Expected terminal output:

```text
30
```

`Adder.add` carries written contracts on its parameters and return value. Per `IRIS-V1-TYPES-C003`, omitted method parameter and return annotations default to `Dynamic<Object>`, accepting any value at the boundary while dynamic message dispatch proceeds inside.

Local variable bindings follow a distinct rule under `IRIS-V1-CONTROL-C005`: when a binding omits an explicit type annotation, the initializer's precise static type becomes the binding's fixed local static type. An unannotated binding initialized with `"hello"` is typed as `String`, not `Dynamic<Object>`. Programs that require a broader or dynamic binding cell must write that type contract explicitly, such as `mut x: Dynamic<Object> = source` or `mut x: String | Integer = "ready"`.

## Optional annotations still mean something

Type annotations in Iris are optional, but once written, they establish unbreakable boundary guards. Binding annotations, parameter types, return signatures, property declarations, and generic arguments share a common enforcement rule: statically provable mismatches are diagnosed immediately, while dynamic crossings are checked at the runtime boundary.

<!-- iris-example: {"id":"07-union-bindings","mode":"vm","stdout":"ready\n42\n"} -->
```iris
mut value: String | Integer = "ready"
print(value)

value = 42
print(value)
```

Expected terminal output:

```text
ready
42
```

The union type annotation ensures that `value` holds only a `String` or an `Integer`. Assigning another type triggers a boundary violation. The binding does not widen dynamically after assignment.

## Object, Nil, and nilable types

`Object` is the top type in Iris. Every runtime value is an instance of `Object`, including `nil`. However, this does not mean every type admits `nil`. Concrete types like `String` strictly exclude `nil` unless declared as `String?` or `String | Nil`.

<!-- iris-example: {"id":"07-nilable-types","mode":"vm","stdout":"value is present\n"} -->
```iris
let text: String? = "value is present"

if text != nil {
  let unwrapped: String = text as String
  print(unwrapped)
}
```

Expected terminal output:

```text
value is present
```

The syntax `T?` is syntactic sugar for `T | Nil`, and both normalize to the same type identity. Checking against `nil` allows code to narrow the type safely.

## Tests and casts

Iris provides three fundamental operators for inspecting and casting types: `is`, `as`, and `as?`. The operator `is` performs a runtime query and returns a `Bool`. The operator `as` verifies the type, returning the value or raising a runtime exception if the check fails. The operator `as?` checks safely, returning `nil` on failure.

<!-- iris-example: {"id":"07-tests-and-casts","mode":"vm","stdout":"hello\n"} -->
```iris
let value: Object = "hello"

if value is String {
  let text = value as String
  print(text)
}
```

Expected terminal output:

```text
hello
```

These operations inspect the existing object identity directly. They do not perform implicit numeric conversions, copy collection elements, or alter method lookup tables.

When an explicit cast fails, Iris raises an immediate runtime error.

<!-- iris-example: {"id":"07-cast-failure","mode":"expected-error","engine":"vm","exit":1,"stderr":"Type","stdout":""} -->
```iris
let raw: Object = "text"
let num = raw as Integer
```

## Typeof copies a static type

The `typeof(expression)` operator is a static type constructor rather than a runtime query (`IRIS-V1-TYPES-C093`). It yields the normalized static type of the operand expression at that exact program point, reflecting any active flow-sensitive narrowing. The expression operand is type-checked but never evaluated at runtime.

<!-- iris-example: {"id":"07-typeof-expression","mode":"vm","stdout":"25\n"} -->
```iris
let base: Integer = 10
let copy: typeof(base) = 25
print(copy)
```

Expected terminal output:

```text
25
```

When the static type cannot be determined from surrounding declarations (such as from an omitted method return), `typeof(expression)` evaluates to `Dynamic<Object>`.

## Types are values

Type objects in Iris are first-class, immutable, identity-bearing runtime values. The expression `(T).type` reifies a static type expression into an executable value.

<!-- iris-example: {"id":"07-types-as-values","mode":"vm","stdout":"nominal\ntrue\n"} -->
```iris
let int_type = Integer.type
print(int_type.kind())
print(int_type.subtype?(Object.type))
```

Expected terminal output:

```text
nominal
true
```

Closed generic classes and union combinations reify in the same manner. Parentheses are required when forming expressions like `(String | Nil).type` to distinguish union syntax from bitwise operations.

## Generics are reified and invariant

Generic classes, modules, contracts, and callables retain full type argument metadata at runtime. Generic classes can be instantiated and operated with reified type parameters:

<!-- iris-example: {"id":"07-generic-box","mode":"vm","stdout":"42\n"} -->
```iris
class Box<T> {
  property value: T = 0
  public fun set_val(v: T) -> Nil {
    @value = v
    nil
  }
  public fun get_val() -> T {
    @value
  }
}

let b = Box<Integer>.new()
b.set_val(42)
print(b.get_val())
```

Expected terminal output:

```text
42
```

A generic declaration can also enforce constraints using a `where` clause.

**Specification-only (not executed):** This sketch illustrates compound `where` constraints and an invalid invariant assignment, not a complete runnable program: it omits conformance to the local `Comparable<T>` contract and initialization of `value`. Generic classes are strictly invariant (`IRIS-V1-TYPES-C068`), so even with those prerequisites satisfied, `SortedBox<String>` is not assignable to `SortedBox<Object>`.

<!-- iris-example: {"id":"07-generic-invariance","mode":"spec-only","reason":"Specification example demonstrating invariant generic class parameterization and rejection of covariance"} -->
```iris
contract Comparable<T> {
  fun compare(other: T) -> Integer
}

class SortedBox<T> where T: Comparable<T> & NonNil {
  property value: T
}

let names: SortedBox<String> = SortedBox<String>.new()
let objects: SortedBox<Object> = names
```

Iris enforces strict generic invariance. `SortedBox<String>` is not a subtype of `SortedBox<Object>`, even though `String` is a subtype of `Object`. Type casts cannot bridge invariant generic boundaries.

Callable types follow identical invariance guarantees. `Closure<(Integer) -> Object>` cannot be substituted for `Closure<(Integer) -> Symbol>`, ensuring call signatures remain dependable.

Generic type arguments can also be supplied explicitly during method calls when local inference requires disambiguation:

**Specification-only (not executed):** This syntax illustration omits definitions of `choose` and `value`. It shows explicit method call type arguments and the wildcard placeholder `_` allowed at call sites (`IRIS-V1-GRAMMAR-C067`):

<!-- iris-example: {"id":"07-explicit-type-arguments","mode":"spec-only","reason":"Specification example illustrating explicit type argument bracket syntax and wildcards"} -->
```iris
let chosen = choose<String, Integer>(value)
let partial = choose<String, _>(value)
```

## Never marks paths with no value

`Never` is the bottom type in Iris. It denotes execution paths that cannot produce a normal runtime value. Expressions that throw exceptions, infinite loops, and functions declared with `-> Never` evaluate to the `Never` type.

**Specification-only (not executed):** This control-flow illustration omits the application-defined condition `ready?`. It shows how a function returning `Never` (`IRIS-V1-TYPES-C080`) participates in control flow without widening the static return type of an `if` expression:

<!-- iris-example: {"id":"07-never-paths","mode":"spec-only","reason":"Specification example demonstrating bottom type Never in branching and early exit handling"} -->
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

Because `Never` represents an impossible value, it is assignable to all other types without widening union results.

## Static promise, dynamic freedom

Iris gradual typing unifies safety and expressiveness.

**Dynamic freedom**
- Unannotated method parameters default to `Dynamic<Object>`; unannotated local bindings retain their inferred initializer type.
- Runtime type tests `is` and conditional casts `as?` allow code to adapt cleanly to incoming data.
- Generic arguments are materialized when needed at execution time.

**Static promises**
- Explicit annotations create hard runtime boundaries that cannot be bypassed.
- Generics remain strictly invariant across all parameterizations.
- Nilability rules prevent accidental `nil` propagation into non-nilable slots.
- Type annotations never silently alter method resolution or trigger static overloading.

**Hands-on Exercise**

Declare a function `safe_length(item: Object) -> Integer` that checks whether `item` is a `String`. If true, cast it to `String` and return its length. Otherwise, return `0`. Verify the function with both a string and an integer argument using `./target/debug/iris --vm test_type.iris`.

Expected terminal output:

```text
5
0
```

## Read the spec

For detailed normative rules and formal typing judgements, consult [05-types-contracts-generics.md](../../spec/iris-v1/05-types-contracts-generics.md):

| Clause | Topic |
| --- | --- |
| `IRIS-V1-TYPES-C003` through `IRIS-V1-TYPES-C007` | Gradual annotation boundaries and `Dynamic<Object>` defaults. |
| `IRIS-V1-TYPES-C008` through `IRIS-V1-TYPES-C017` | Type constructors, `Object`, `Nil`, `T?`, `Dynamic<T>`, aliases, and Type objects. |
| `IRIS-V1-TYPES-C018` through `IRIS-V1-TYPES-C027` | Union, intersection, nilability, `NonNil`, and same-name obligation rules. |
| `IRIS-V1-TYPES-C028` through `IRIS-V1-TYPES-C035` | `is`, `as`, `as?`, contract views, and narrowing. |
| `IRIS-V1-TYPES-C054` through `IRIS-V1-TYPES-C074` | Generic declarations, constraints, inference, materialization, and invariance. |
| `IRIS-V1-TYPES-C080` through `IRIS-V1-TYPES-C083` | `Never` and unreachable flow. |
| `IRIS-V1-TYPES-C093` | `typeof(expression)` as a static type copy. |
| `IRIS-V1-TYPES-C094` through `IRIS-V1-TYPES-C096` | `Closure<S>`, `BoundMethod<S>`, `Block<S>`, and callable invariance. |
| `IRIS-V1-TYPES-C097` | `TypeContractError` from a failed closed generic materialization. |
| `IRIS-V1-GRAMMAR-C063`, `C065`, `C066`, `C067` | Closed generic names, `(T).type`, and explicit call type arguments. |
