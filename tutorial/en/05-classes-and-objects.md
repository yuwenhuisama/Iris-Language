# Classes and Objects

This chapter details the object model of Iris v1. You will create Classes, attach methods, define stored properties, inspect instance storage, understand inheritance, and distinguish reference identity from structural equality.

## Stored properties declare typed slots

The `property` shorthand defines a typed property slot and generates corresponding accessors. When custom behavior is needed, `property fun` defines explicit property accessors.

<!-- iris-example: {"id":"05-stored-properties","mode":"vm","stdout":"100\n"} -->
```iris
class Account {
  public property balance: Integer = 100
}
let a = Account.new()
print(a.balance)
```

Expected output:

```text
100
```

Property reads dispatch through the property getter without requiring explicit parentheses.

## Classes are objects with a new message

Classes in Iris are first-class runtime objects. You instantiate a class by sending the `new` message to the Class object. Iris does not use constructor overloading or constructor names matching the class.

<!-- iris-example: {"id":"05-classes-new","mode":"vm","stdout":"42\n"} -->
```iris
class Counter {
  public fun value() -> Integer { 42 }
}
let c = Counter.new()
print(c.value())
```

Expected output:

```text
42
```

Calling `Counter.new()` allocates a new instance and dispatches initialization before returning the instance.

## Raw instance variables live on the current receiver

Raw instance variables begin with `@` (such as `@value`). They represent internal slot storage associated with the current receiver instance. Outside code cannot reach into private ivar state directly, preserving encapsulation.

<!-- iris-example: {"id":"05-instance-state","mode":"vm","stdout":"0\n7\n"} -->
```iris
class Entity {
  public fun initialize() -> Nil {
    @val = 0
    nil
  }
  public fun set_val(n: Integer) -> Integer {
    @val = n
  }
  public fun get_val() -> Integer {
    @val
  }
}
let e = Entity.new()
print(e.get_val())
e.set_val(7)
print(e.get_val())
```

Expected output:

```text
0
7
```

In contrast, class-level shared state must be explicitly declared with `shared mut @@name` or `shared let @@name`.

## Self and super make receiver roles explicit

The `self` keyword references the active receiver instance within method bodies.

Classes support single inheritance via `extends`. Methods in subclasses can override inherited behavior and participate in method lookup order.

<!-- iris-example: {"id":"05-inheritance-super","mode":"vm","stdout":"base derived\ntrue\n"} -->
```iris
class Base {
  public fun name() -> String { "base" }
}
class Derived extends Base {
  public override fun name() -> String { super() + " derived" }
}
let d = Derived.new()
print(d.name())
print(d is Base)
```

Expected output:

```text
base derived
true
```

Calling `super(args...)` invokes the next implementation in the receiver's method resolution order (MRO).

## Equality is separate from identity

Iris maintains a clear distinction between value equality and identity:

- Equality (`==`) dispatches to the comparison protocol defined on the object's class.
- Identity (`same?`) checks whether two identity-bearing values denote the same observable object identity, not whether memory addresses match. If either operand is identity-less, it raises `IdentityError` (`IRIS-V1-RUNTIME-C029`).

<!-- iris-example: {"id":"05-equality-identity","mode":"vm","stdout":"true\nfalse\n"} -->
```iris
let first = Object.new()
let second = first
let third = Object.new()
print(first same? second)
print(first same? third)
```

Expected output:

```text
true
false
```

`first same? second` evaluates to `true` because both identifiers point to the same instance. `first same? third` evaluates to `false` because they are separate allocations.

## Open classes keep logical identity

Iris supports reopening existing classes using `open class`. This updates the active class revision while strictly maintaining the logical identity and static guarantees already established.

<!-- iris-example: {"id":"05-open-class","mode":"vm","stdout":"updated\n"} -->
```iris
class Service {
  public fun status() -> String { "initial" }
}
open class Service {
  public override fun status() -> String { "updated" }
}
let s = Service.new()
print(s.status())
```

Expected output:

```text
updated
```

Reopening a class publishes a new active revision atomically, allowing dynamic updates without invalidating existing identity references.

## Static promise, dynamic freedom

A class defines a static spine: its superclass, property contracts, and member signatures form an immutable promise. Dynamic operations such as class reopening or method replacement must conform to this static spine before changes are published.

**Hands-on Exercise**

Save the `Account` example as `account.iris`. After its first print, add `a.balance = 125` and `print(a.balance)`. Run `./target/debug/iris --vm account.iris` and confirm output `100` followed by `125`: the generated setter changes the stored slot while preserving its `Integer` contract.

## Read the spec

This chapter simplifies the following normative clauses:

- [`IRIS-V1-RUNTIME-C003`](../../spec/iris-v1/03-runtime-object-model.md): runtime objecthood.
- [`IRIS-V1-RUNTIME-C009`](../../spec/iris-v1/03-runtime-object-model.md): logical classes and active revisions.
- [`IRIS-V1-RUNTIME-C010`](../../spec/iris-v1/03-runtime-object-model.md): reopening preserves logical Class identity.
- [`IRIS-V1-RUNTIME-C023`](../../spec/iris-v1/03-runtime-object-model.md): ordinary message identity.
- [`IRIS-V1-RUNTIME-C029`](../../spec/iris-v1/03-runtime-object-model.md): primitive identity comparison with `same?`.
- [`IRIS-V1-RUNTIME-C038`](../../spec/iris-v1/03-runtime-object-model.md): Method binding produces BoundMethod.
- [`IRIS-V1-RUNTIME-C040`](../../spec/iris-v1/03-runtime-object-model.md): distinct BoundMethod identities.
- [`IRIS-V1-RUNTIME-C046`](../../spec/iris-v1/03-runtime-object-model.md): single class inheritance and MRO lookup.
- [`IRIS-V1-RUNTIME-C054`](../../spec/iris-v1/03-runtime-object-model.md): object construction via `new`.
- [`IRIS-V1-RUNTIME-C055`](../../spec/iris-v1/03-runtime-object-model.md): absence of named constructor overloads.
- [`IRIS-V1-RUNTIME-C066`](../../spec/iris-v1/03-runtime-object-model.md): receiver instance storage with `@ivar`.
- [`IRIS-V1-RUNTIME-C081`](../../spec/iris-v1/03-runtime-object-model.md): explicit `super(args...)` invocation.
- [`IRIS-V1-RUNTIME-C086`](../../spec/iris-v1/03-runtime-object-model.md): default object equality semantics.
- [`IRIS-V1-RUNTIME-C161`](../../spec/iris-v1/03-runtime-object-model.md): single backing slot for stored properties.
- [`IRIS-V1-RUNTIME-C162`](../../spec/iris-v1/03-runtime-object-model.md): shared class storage.
- [`IRIS-V1-RUNTIME-C164`](../../spec/iris-v1/03-runtime-object-model.md): revision migration conventions.
- [`IRIS-V1-GRAMMAR-C058`](../../spec/iris-v1/02-lexical-grammar.md): property shorthand and accessor definitions.
- [`IRIS-V1-GRAMMAR-C064`](../../spec/iris-v1/02-lexical-grammar.md): property scope modifiers.
