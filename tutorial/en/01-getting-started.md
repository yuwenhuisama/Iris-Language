# What Iris Is

This chapter gives you the mental model for Iris v1: every runtime value is an object, behavior is message sending, and dynamic behavior is bounded by static promises. Iris v1 is currently a frozen language specification only. There is no compiler, interpreter, REPL, playground, package manager, or standard library you can use to execute Iris programs. Treat every code block here as grammar-checked teaching material, not as something to run.

```iris
class Counter {
  fun initialize() -> Nil { @value = 0 }
  fun add(delta: Integer) -> Integer { @value += delta }
}

let counter = Counter.new()
let bound: (Integer) -> Integer = counter.add
```

This snippet is adapted from `IRIS-V1-CONTROL-EX003`.

## Every value is an object

Iris is object-oriented at the root. `Counter` is a Class object. `Counter.new()` sends the `new` message to that Class object. `counter.add` reads an instance Method and produces a BoundMethod, which can be stored in a callable binding. There isn't a separate runtime category called Function for named `fun` declarations.

That is the first difference from many languages. Iris syntax may look familiar, but the semantics are message based. A method call, a property read, a named infix call, and most operators are all sends to a receiver.

```iris
let total = 1 + 2
let shifted = total << 1
let equal = total == shifted
```

## Operators are message sends

The `+`, `<<`, and `==` forms are not global functions. They are selected through the receiver's runtime Class and current active revision, subject to the rules in the runtime chapter. Iris also has a few core control forms that are not overloadable messages, including `!`, `&&`, `||`, `&&=`, and `||=`.

## Identity is explicit

Identity is explicit. The primitive `same?` tests whether two identity-bearing operands are the same object. It bypasses ordinary Method lookup, `==`, comparison methods, and user replacement. Not every value has observable identity, so the spec distinguishes identity-bearing objects, such as Class objects, Closure objects, and mutable containers, from identity-less immutable values, such as Integer and String values.

```iris
let first = Object.new()
let second = first
let identity = first same? second
```

Use `==` when you mean the ordinary equality protocol. Use `same?` only when you mean object identity and the operands are identity-bearing.

## Static promises bound dynamic behavior

Iris is also designed around a static and dynamic split. Static annotations, declared superclasses, visible member signatures, declared Contracts, typed properties, and generic constraints are promises. Runtime dispatch remains dynamic, but those promises cannot be silently weakened by dynamic mutation, package upgrade, reflection, native binding, or optimization.

```iris
fun add(a: Integer, b: Integer) -> Integer {
  a + b
}

fun dynamic_add(a, b) {
  a + b
}
```

This snippet is adapted from `IRIS-V1-TYPES-EX001`.

The first Method has a written contract on parameters and return value. The second omits those annotations, so its public Method signature is dynamic within the `Object` bound. In both cases, `a + b` is still a dynamic message send. A static type never chooses a hidden overload.

## Static promise, dynamic freedom

Static facts constrain what must remain true. Dynamic behavior decides which current Method body receives a message at runtime. Iris doesn't use static overload resolution, expected return types, generic arguments, or union branches to pick a different ordinary selector. If you need a Contract-specific slot, later chapters use explicit Contract-qualified dispatch with `..`.

## Where the next chapters go

This tutorial follows that split throughout. [Values and Bindings](02-values-and-bindings.md) starts with local names and literal values. [Control Flow](03-control-flow.md) shows that branches, loops, and matching are value-producing forms. [Functions, Closures, Blocks](04-callables-and-closures.md) explains the callable model. [Classes and Objects](05-classes-and-objects.md) returns to object identity, instance state, inheritance, and construction.

## Read the spec

This chapter simplifies these normative clauses:

- [`IRIS-V1-TRACE-C011`](../../spec/iris-v1/README.md): the specification set does not implement Iris.
- [`IRIS-V1-IDENTITY-C006`](../../spec/iris-v1/01-language-identity.md): every runtime value is an object.
- [`IRIS-V1-IDENTITY-C007`](../../spec/iris-v1/01-language-identity.md): behavior is message sending.
- [`IRIS-V1-IDENTITY-C008`](../../spec/iris-v1/01-language-identity.md): static promises bound dynamic behavior.
- [`IRIS-V1-IDENTITY-C009`](../../spec/iris-v1/01-language-identity.md): static type choice doesn't change ordinary selector identity.
- [`IRIS-V1-IDENTITY-C013`](../../spec/iris-v1/01-language-identity.md): the compact dynamic and static model.
- [`IRIS-V1-RUNTIME-C003`](../../spec/iris-v1/03-runtime-object-model.md): runtime objecthood.
- [`IRIS-V1-RUNTIME-C027`](../../spec/iris-v1/03-runtime-object-model.md): overloadable symbolic operators are ordinary Method sends.
- [`IRIS-V1-RUNTIME-C029`](../../spec/iris-v1/03-runtime-object-model.md): `same?` is primitive identity comparison.
