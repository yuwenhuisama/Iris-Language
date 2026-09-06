# Dynamic Meets Static

This chapter shows how Iris unites dynamic metaprogramming with static boundary stability. Iris allows programs to reopen classes and modules, publish active revisions, inspect metadata, declare decorators, and define dynamic members. These capabilities are strictly bounded. Candidate updates must pass validation against the static spine, MetaCapabilities, reflection policies, contract promises, and transactional isolation rules before changes publish.

<!-- iris-example: {"id":"09-open-class-reopen","mode":"vm","stdout":"2\n"} -->
```iris
class Counter {
  public fun value() -> Integer { 1 }
}

let c = Counter.new()

open class Counter {
  public override fun value() -> Integer { 2 }
}

print(c.value())
```

Expected terminal output:

```text
2
```

Reopening a class never creates a new class object. It preserves logical class identity and commits an updated active revision upon validation.

## Open transactions

Declarative `open class` and `open module` forms share an identical transaction model with programmatic `Class#open` and `Module#open`. The transaction block constructs a candidate set in isolation. Code outside the transaction continues to observe the published active revision until safepoint publication.

**Specification-only (not executed):** This programmatic-open illustration assumes an existing `Tool` class, omitted here.

<!-- iris-example: {"id":"09-programmatic-open","mode":"spec-only","reason":"Specification example illustrating programmatic Class#open dynamic method definition"} -->
```iris
Tool.open() { |target: Class| -> Nil
  target.define_method(:status) { |self: Tool| -> Symbol; :ready }
}
```

Programmatic open transactions target dynamically resolved classes and modules. They cannot target Contracts or closed generic Classes or Modules (`IRIS-V1-META-C033`). They are synchronous, thread-confined, non-suspending, and cannot escape their lexical scope. If an error occurs, the entire candidate set rolls back without publishing.

## Active revisions and retained Methods

Regular message sends look up methods using the current active revision. Running activation frames retain the specific method body selected at call entry. Captured method references retain their original method identity, though invocation revalidates that the receiver's current MRO still includes the method's owner.

<!-- iris-example: {"id":"09-retained-methods","mode":"vm","stdout":"1\n2\n"} -->
```iris
class Counter {
  public fun value() -> Integer { 1 }
}

let c = Counter.new()
let retained = c.value

open class Counter {
  public override fun value() -> Integer { 2 }
}

print(retained.call())
print(c.value())
```

Expected terminal output:

```text
1
2
```

The bound method reference `retained` keeps its original implementation, while subsequent direct message sends `c.value()` resolve to the newly published revision.

## MetaCapabilities

The `meta deny` header clause enforces static restrictions on dynamic capability. Once a class denies a capability, dynamic metaprogramming cannot restore that power.

**Specification-only (not executed):** This capability illustration depends on an application-defined `load_config()` and its result's `enabled?` member, neither supplied here.

<!-- iris-example: {"id":"09-meta-capabilities","mode":"spec-only","reason":"Specification example illustrating static meta deny capability restrictions"} -->
```iris
class Tool meta deny superclass, native {
  let enabled = load_config().enabled?

  if enabled {
    self.define_method(:debug) { |self: Tool| -> String; "debug" }
  }

  public fun run() -> String { "run" }
}
```

Standard capabilities include `method_set`, `method_body`, `property_set`, `property_body`, `modules`, `superclass`, `subclass`, `shape`, `class_state_set`, `class_state_write`, `instance_state`, and `native`. Unknown names trigger compile-time errors. Denying one capability never implicitly revokes others.

## Imports name a package explicitly

Import declarations name a module in the current package or cross a package boundary using `pkg::Module`. The package segment is the globally unique reverse-domain identifier. Iris rejects wildcard imports and runtime-string imports.

**Specification-only (not executed):** These import forms assume an external `org.dep` package exporting `Codec`, `encode`, and `decode`; the dependency is not supplied here.

<!-- iris-example: {"id":"09-import-paths","mode":"spec-only","reason":"Specification example illustrating package-qualified imports and override import modifiers"} -->
```iris
import org.dep::Codec
from org.dep::Codec import encode, decode

override import org.dep::Codec
```

The `override` modifier on an import explicitly authorizes compatible extensions provided by the target package, preventing accidental collisions while preserving static contracts.

## Decorators transform candidates

Decorators attach to declarations. Under `IRIS-V1-META-C087`, pure deterministic static planning runs in the compiler or linker, while runtime transformation runs inside the declaration's origin or open candidate transaction. Multiple decorators execute in written top-to-bottom order (`IRIS-V1-META-C086`). They cannot change class nominal identity, alter package identifiers, or bypass `meta deny` policies.

**Specification-only (not executed):** This application-order illustration omits the `logged` and `memoized` decorators and the application method `compute_total`.

<!-- iris-example: {"id":"09-decorator-syntax","mode":"spec-only","reason":"Specification example illustrating method decorator application"} -->
```iris
@logged(level: :info)
@memoized()
public fun total() -> Integer {
  compute_total()
}
```

A decorator is an ordinary class that implements one of five standard contracts: `ClassDecorator`, `ModuleDecorator`, `ContractDecorator`, `MethodDecorator`, or `PropertyDecorator`.

**Specification-only (not executed):** This no-op sketch shows the current contract signatures under `IRIS-V1-META-C124`, using unannotated parameters rather than inventing metadata type names. The runtime supplies the third `context` argument to `transform`; static `plan` receives no transaction context. This fragment does not apply the decorator to a declaration.

<!-- iris-example: {"id":"09-decorator-contract-impl","mode":"spec-only","reason":"Current C124 decorator contract sketch without an application fixture"} -->
```iris
class Stamp for MethodDecorator {
  impl fun plan(declaration, arguments) -> Plan { Plan.empty }
  impl fun transform(declaration, arguments, context) -> Transformation { Transformation.empty }
}
```

The `plan` phase is pure, static, and deterministic. The runtime `transform` phase runs inside the candidate transaction and returns a controlled `Transformation`. Both members are required, even when a phase contributes nothing; `Plan.empty` and `Transformation.empty` express those no-op results.

## ReflectionPolicy, not reflection tokens

Reflection in Iris exposes permission-filtered, immutable metadata views. It never hands out raw mutable tables or strings for dynamic evaluation. Reflection operations reside in `Reflection::*` submodules categorized by target kind.

<!-- iris-example: {"id":"09-reflection-inspection","mode":"vm","stdout":"status\n"} -->
```iris
class Tool {
  public fun status() -> String { "ready" }
}

let method = Reflection::Class.method(Tool, :status)
print(method.selector)
```

Expected terminal output:

```text
status
```

Instance variable reflection operates through `Reflection::Object`:

<!-- iris-example: {"id":"09-reflection-ivars","mode":"vm","stdout":"99\n"} -->
```iris
class Box {}

let b = Box.new()
Reflection::Object.set_ivar(b, :@token, 99)
print(Reflection::Object.get_ivar(b, :@token))
```

Expected terminal output:

```text
99
```

Reflection permissions originate from `ReflectionPolicy`, evaluating caller package, target identity, requested operation, and declared scope. Iris avoids raw capability tokens.

## Why bounded dynamism matters

Iris makes the boundary explicit: runtime changes may replace compatible behavior, but they must preserve the declarations that callers rely on. Bounded dynamism describes this language's rules, not a claim that other dynamic or static languages lack safeguards or adaptability.

Callers can rely on static promises: method signatures, contract conformances, generic invariance, and package identity remain stable. Metaprograms can still publish compatible revisions or dynamically staged extensions. The validator ensures no partial update or contract-breaking candidate ever enters active execution.

## Static promise, dynamic freedom

Iris delivers controlled metaprogramming through clear boundaries.

**Dynamic freedom**
- Compatible class and module revisions can be defined and published at runtime.
- Methods can be added, updated, and rebound dynamically.
- Decorators combine static planning with runtime candidate transformation.

**Static promises**
- Logical class identity remains immutable across all revisions.
- Contract obligations, generic invariants, and superclass spines cannot be compromised.
- `meta deny` permanently restricts forbidden structural mutations.
- Open transactions guarantee atomic safepoint publication or clean rollback.

**Hands-on Exercise**

Declare a class `Configuration` with an instance method `mode() -> String { "debug" }`. Reopen the class with `open class Configuration` to override `mode() -> String { "release" }`. Print the result of calling `mode()` on an existing instance to confirm active revision dispatch. Run with `./target/debug/iris --vm test_open.iris`.

Expected terminal output:

```text
release
```

## Read the spec

For normative specifications on metaprogramming transactions and reflection, consult [01-language-identity.md](../../spec/iris-v1/01-language-identity.md), [03-runtime-object-model.md](../../spec/iris-v1/03-runtime-object-model.md), and [08-modules-metaprogramming.md](../../spec/iris-v1/08-modules-metaprogramming.md):

| Clause | Topic |
| --- | --- |
| `IRIS-V1-IDENTITY-C008` through `IRIS-V1-IDENTITY-C014` | Static promises bounding dynamic behavior. |
| `IRIS-V1-RUNTIME-C009` through `IRIS-V1-RUNTIME-C022` | Logical classes, active revisions, method replacement, and rollback. |
| `IRIS-V1-META-C030` through `IRIS-V1-META-C043` | Open transactions, candidate isolation, conflicts, and safepoint commit. |
| `IRIS-V1-META-C044` through `IRIS-V1-META-C052` | Static versus dynamic member visibility without overload dispatch. |
| `IRIS-V1-META-C072` through `IRIS-V1-META-C084` | MetaCapabilities and permission checks. |
| `IRIS-V1-META-C085` through `IRIS-V1-META-C094` | Decorator evaluation phases and safety bounds. |
| `IRIS-V1-META-C118` through `IRIS-V1-META-C126` | `Reflection::*` surfaces, decorator contracts, `Plan`, and `Transformation`. |
| `IRIS-V1-GRAMMAR-C068` and `IRIS-V1-GRAMMAR-C069` | Package-qualified import syntax and the `override` import modifier. |
| `IRIS-V1-META-C095` through `IRIS-V1-META-C112` | ReflectionPolicy and permission-filtered views. |
