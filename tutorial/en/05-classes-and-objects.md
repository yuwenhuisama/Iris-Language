# Classes and Objects

This chapter puts the object model into code. You'll define Classes, attach Methods with `fun`, use raw instance variables with `@name`, refer to the current receiver with `self`, construct instances with `Type.new()`, and distinguish identity from equality. The chapter ends by pointing toward Modules and Contracts, which are the next composition tools after single Class inheritance.

```iris
class Counter {
  fun initialize() -> Nil { @value = 0 }
  fun add(delta: Integer) -> Integer { @value += delta }
  fun value() -> Integer { @value }
}

let counter = Counter.new()
```

This snippet is adapted from `IRIS-V1-CONTROL-EX003`.

## Classes are objects with a new message

A Class declaration creates a logical Class object. Instances are made by sending `new` to the Class object. Iris v1 doesn't have Class-named constructor syntax or constructor overloads. Construction allocates an instance, runs stored property initialization where present, calls the final dynamic `initialize(...)`, and returns the instance if initialization succeeds.

```iris
class Point {
  fun initialize(x: Integer, y: Integer) -> Nil {
    @x = x
    @y = y
  }
}

let point = Point.new(1, 2)
```

## Raw instance variables live on the current receiver

Raw instance variables use `@name`. They name storage on the current receiver. Source code can't write `other.@name`; outside code should use Methods or properties. A raw ivar read that finds no slot returns `nil` under the raw ivar rules. First assignment can create receiver state only when the receiver's policy allows instance state expansion.

```iris
class Named {
  fun initialize(name: String) -> Nil { @name = name }
  fun name() -> String { @name }
  fun rename(name: String) -> Nil { @name = name }
}
```

## Self and super make receiver roles explicit

`self` names the current receiver. It is useful when you want to pass the receiver as an object or make the receiver role explicit. Unqualified Method calls inside receiver code can also resolve to privileged sends to the current receiver.

```iris
class Box {
  fun initialize(value: Object) -> Nil { @value = value }
  fun value() -> Object { @value }
  fun copy_value_to(other: Box) -> Object { other.replace(value()) }
  fun replace(value: Object) -> Object { @value = value }
  fun receiver() -> Box { self }
}
```

Inheritance uses `extends`. A Class has a single runtime superclass. `super(args...)` calls the same selector after the current Method's lexical owner in the receiver's current method lookup order. There is no bare `super` and no implicit argument forwarding.

```iris
class NamedCounter extends Counter {
  fun initialize(name: String) -> Nil {
    super()
    @name = name
  }
}
```

## Equality is separate from identity

Equality and identity are different. `==` is an ordinary comparison Method slot. Root equality for identity-bearing objects first checks whether both references denote the same object, then follows the comparison protocol. `same?` is the primitive identity test. It accepts only identity-bearing operands and raises `IdentityError` for identity-less values.

```iris
let first = Object.new()
let second = first
let third = Object.new()

let same_reference = first same? second
let equal_by_protocol = first == third
```

Methods and BoundMethods also have identity. Every time code reads `obj.method`, Iris creates a fresh BoundMethod object that captures the receiver relation and exact Method identity selected at binding time. Later ordinary sends use the current active Class revision, but a saved BoundMethod keeps its captured Method identity and revalidates receiver membership when invoked.

```iris
class Counter {
  fun value() -> Integer { @value }
}

let counter = Counter.new()
let before = counter.value
let again = counter.value
```

This snippet is adapted from a runtime example in `spec-snippets.json` sourced from `03-runtime-object-model.md`.

## Open classes keep logical identity

Classes can be reopened with `open class`, subject to the static spine and validation rules. Reopening publishes a new active revision of the same logical Class identity. It is dynamic behavior, but it can't remove or weaken the static facts that existing typed code was promised.

```iris
let klass = Counter

open class Counter {
  override fun value() -> Integer { 2 }
}

let still_same_class = klass same? Counter
```

## Static promise, dynamic freedom

A Class is dynamic because its active revision can change through an authorized open operation. It is statically promised because its declared superclass, declared Contracts, visible member names and signatures, property contracts, generic arity, and other static spine facts must remain compatible before a new revision can publish.

Single inheritance is only the first layer of composition. Chapter 06, handled separately, introduces Modules for reusable behavior and Contracts for named obligation surfaces. Contracts are also where explicit `..` qualified dispatch becomes necessary when ordinary selector identity is not enough.

## Read the spec

This chapter simplifies these normative clauses:

- [`IRIS-V1-RUNTIME-C003`](../../spec/iris-v1/03-runtime-object-model.md): every runtime value is an object.
- [`IRIS-V1-RUNTIME-C009`](../../spec/iris-v1/03-runtime-object-model.md): logical Classes and active revisions.
- [`IRIS-V1-RUNTIME-C010`](../../spec/iris-v1/03-runtime-object-model.md): reopening keeps logical Class identity.
- [`IRIS-V1-RUNTIME-C023`](../../spec/iris-v1/03-runtime-object-model.md): ordinary message identity.
- [`IRIS-V1-RUNTIME-C029`](../../spec/iris-v1/03-runtime-object-model.md): primitive `same?`.
- [`IRIS-V1-RUNTIME-C038`](../../spec/iris-v1/03-runtime-object-model.md): Method binding creates a BoundMethod.
- [`IRIS-V1-RUNTIME-C040`](../../spec/iris-v1/03-runtime-object-model.md): each Method read creates a distinct BoundMethod identity.
- [`IRIS-V1-RUNTIME-C046`](../../spec/iris-v1/03-runtime-object-model.md): single Class inheritance plus Module composition lookup.
- [`IRIS-V1-RUNTIME-C054`](../../spec/iris-v1/03-runtime-object-model.md): standard construction through `new`.
- [`IRIS-V1-RUNTIME-C055`](../../spec/iris-v1/03-runtime-object-model.md): no Class-named constructor syntax or constructor overloads.
- [`IRIS-V1-RUNTIME-C066`](../../spec/iris-v1/03-runtime-object-model.md): raw `@x` storage on the current receiver.
- [`IRIS-V1-RUNTIME-C081`](../../spec/iris-v1/03-runtime-object-model.md): explicit `super(args...)`.
- [`IRIS-V1-RUNTIME-C086`](../../spec/iris-v1/03-runtime-object-model.md): default equality for identity-bearing objects.
