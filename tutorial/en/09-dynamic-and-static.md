# Dynamic Meets Static

This chapter is the payoff for the earlier chapters. Iris lets programs reopen Classes and Modules, publish new active revisions, add dynamic members, apply decorators, and inspect metadata. Those powers are bounded. A candidate change must validate against the static spine, MetaCapabilities, package and reflection policy, Contract obligations, and transaction rules before it can become visible.

```iris
let klass = Counter
let first = Counter.new()

open class Counter {
  override fun value() -> Integer { 2 }
}

klass same? Counter        // true, reopen kept the logical Class identity
first.value()             // later send uses the current active revision
```

This snippet is reused from the runtime examples. Reopening a Class doesn't create a second Class object. It keeps the logical Class identity and publishes a new active revision after validation.

## Open transactions

`open class Name { ... }` and `open module Name { ... }` use the same transaction model as programmatic `Class#open` and `Module#open`. The body builds a candidate. Code outside the transaction keeps seeing the published active revision until commit.

```iris
Tool.open() { |target: Class| -> Nil
  target.define_method(:status) { |self: Tool| -> Symbol; :ready }
}
```

This is reused from `IRIS-V1-META-EX003`. Programmatic opens can target dynamically selected Class and Module objects. They still can't target Contracts or closed generic Classes. They are synchronous, thread-confined, non-suspending, and non-escaping.

If the body completes normally, the candidate validates. If validation succeeds, Iris publishes at a safepoint. If the body raises, a capability check fails, a permission check fails, or a conflict is detected, the whole candidate group rolls back and publishes nothing.

## Active revisions and retained Methods

Ordinary sends use the current active revision. Already entered Method frames keep the body selected at call entry. Captured Method or BoundMethod objects keep their old Method identity, but invocation revalidates that the receiver's current MRO still contains the Method's owner.

```iris
let before = counter.value
let again = counter.value
before same? again        // false, each read creates a BoundMethod identity

open class Counter {
  override fun value() -> Integer { 3 }
}

before()                  // invokes the captured old Method after revalidation
counter.value()           // ordinary lookup uses the new Method
```

This snippet is reused from the runtime examples. The distinction matters for tools and optimizers. They can cache, but only with guards that respect revision and Method identity.

## MetaCapabilities

`meta deny` is a static header clause. It narrows what structural operations can happen later. Source can't write an `allow` form to regain denied power.

```iris
class Tool meta deny superclass, native {
  let enabled = load_config().enabled?

  if enabled {
    self.define_method(:debug) { |self: Tool| -> String; "debug" }
  }

  public fun run() -> String { "run" }
}
```

This example is reused from `IRIS-V1-META-EX002`. The conditional `define_method` is dynamic-only because it depends on executable body control flow. It can be found through permitted reflection or dynamic dispatch, but it doesn't become part of already compiled static API.

The v1 capability names include `method_set`, `method_body`, `property_set`, `property_body`, `modules`, `superclass`, `subclass`, `shape`, `class_state_set`, `class_state_write`, `instance_state`, and `native`. Unknown `meta deny` names are errors. Denying one capability doesn't silently deny all the others.

## Decorators transform candidates

Decorators attach to declarations and transform declaration candidate metadata. Multiple decorators run top to bottom. They don't change a Class into a Module, replace nominal identity, rewrite package identity, or bypass MetaCapabilities.

```iris
@logged(level: :info)
@memoized()
public fun total() -> Integer {
  compute_total()
}
```

This is reused from `IRIS-V1-META-EX005`. A declarative decorator has a deterministic static planning phase and a runtime transform inside the declaration's candidate transaction. Runtime-dependent or conditional decoration is dynamic-only until a new static artifact declares it.

## ReflectionPolicy, not reflection tokens

Reflection returns permission-filtered immutable metadata views. It doesn't hand out raw mutable tables or eval-string mutation. Structural changes still go through open or meta transactions.

```iris
let names = Reflection.list_ivars(object)
let old = Reflection.get_ivar(object, :@cache)
Reflection.set_ivar(object, :@cache, compute())
```

This snippet is reused from `IRIS-V1-META-EX004`. Reflection authorization comes from `ReflectionPolicy`: caller package, operation, granted scope, target identity, and target policies. There is no first-class reflection capability token in Iris v1.

## Why bounded dynamism matters

Many dynamic languages let you change a class. Many static languages give callers stable type promises. Iris tries to keep both, but it refuses to make the unstable parts invisible.

An ordinary caller can rely on a declared Method signature, a Contract set, generic invariance, and package identity. A metaprogram can still replace a compatible Method body or add a dynamic-only helper. The validator stands between those worlds. It rejects changes that would downgrade the static spine, hide a partial candidate, or publish a half-mutated runtime.

This makes tools possible without making the language static. A compiler, reflection user, host embedding layer, or conformance suite can name the promises that survive runtime mutation.

## Static promise, dynamic freedom

Dynamic freedom: compatible Class and Module revisions can be constructed at runtime, Methods can be replaced, Modules can be recomposed, dynamic-only members can appear, and decorators can wrap declarations.

Static promise: logical Class identity, static spine facts, Contract obligations, generic arity and invariance, MetaCapabilities, ReflectionPolicy, package identity, and atomic publish-or-rollback remain durable. No ordinary send observes a partial candidate.

## Read the spec

For exact rules, read [01-language-identity.md](../../spec/iris-v1/01-language-identity.md), [03-runtime-object-model.md](../../spec/iris-v1/03-runtime-object-model.md), and [08-modules-metaprogramming.md](../../spec/iris-v1/08-modules-metaprogramming.md):

| Clause | Topic |
| --- | --- |
| `IRIS-V1-IDENTITY-C008` through `IRIS-V1-IDENTITY-C014` | Static promises bounding dynamic behavior. |
| `IRIS-V1-RUNTIME-C009` through `IRIS-V1-RUNTIME-C022` | Logical Classes, active revisions, Method replacement, and rollback basics. |
| `IRIS-V1-META-C030` through `IRIS-V1-META-C043` | Open transactions, candidate isolation, conflicts, safepoint commit, and rollback. |
| `IRIS-V1-META-C044` through `IRIS-V1-META-C052` | Static versus dynamic member visibility and no overload dispatch. |
| `IRIS-V1-META-C072` through `IRIS-V1-META-C084` | MetaCapabilities and operation checks. |
| `IRIS-V1-META-C085` through `IRIS-V1-META-C094` | Decorator phases and restrictions. |
| `IRIS-V1-META-C095` through `IRIS-V1-META-C112` | ReflectionPolicy and reflection views. |
