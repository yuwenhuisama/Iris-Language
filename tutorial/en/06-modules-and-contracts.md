# Modules and Contracts

This chapter explains how Iris separates behavior reuse from contract promises. A `module` supplies reusable methods that a class or another module can mix in. A `contract` declares an explicit obligation surface that a class promises with `for` and satisfies with `impl`. The two mechanisms work together, but they are never interchangeable. Modules determine runtime method lookup order, while contracts define checked requirements, views, and qualified slots.

<!-- iris-example: {"id":"06-composition","mode":"vm","stdout":"B\n"} -->
```iris
module A {
  public fun trace() -> String { "A" }
}

module B mixin A {
  public override fun trace() -> String { "B" }
}

class C mixin A, B {}

print(C.new().trace())
```

Expected terminal output:

```text
B
```

A class has exactly one superclass chain, followed by composed modules. The header `mixin A, B` is applied left to right, but method lookup checks the closest module first. The effective search order is `C`, then `B`, then `A`, and finally the superclass chain.

## Modules add behavior

A module is a named object that holds methods. Classes compose modules in their declaration headers. A module can also compose other modules, allowing shared functionality to assemble into structured layers.

<!-- iris-example: {"id":"06-modules-add-behavior","mode":"vm","stdout":"Hello, Iris\n"} -->
```iris
module Greeter {
  public fun hello(name: String) -> String {
    "Hello, " + name
  }
}

class User mixin Greeter {}

let u = User.new()
print(u.hello("Iris"))
```

Expected terminal output:

```text
Hello, Iris
```

Module mixins do not create multiple inheritance. A class retains a single superclass. Composed modules are inserted into the method resolution order (MRO) and deduplicated by module identity. Mixing in the same module more than once never creates duplicate entries or alters earlier precedence.

Module methods execute in the context of the current receiver. When a module method reads `@state`, it accesses the receiver instance variable `@state`, not storage owned by the module. Private method access is strictly bounded: module composition does not grant private access unless the mixin edge explicitly requests it with a `private` marker in the mixin list.

**Specification-only (not executed):** This declaration-only illustration shows private mixin authorization syntax (`IRIS-V1-GRAMMAR-C061`); it does not exercise a private call. Under `IRIS-V1-RUNTIME-C050`, the edge authorizes Module methods to access the host Class's private methods, not the reverse.

<!-- iris-example: {"id":"06-mixin-private-access","mode":"spec-only","reason":"Declaration-only illustration of Module access to host Class private methods; no private call is exercised"} -->
```iris
module Trace {
  fun internal_log() -> Nil { nil }
}

class Job mixin Trace private {}
```

The `private` keyword sits directly on the mixin edge. Here, `Trace` receives authorization to call `Job`'s private methods; unmarked mixins do not. Authorization is scoped to the host logical Class, closed Module identity, and edge revision.

## Contracts declare promises

A contract is an explicit obligation surface. A contract body contains method requirements without executable bodies or stored fields. Writing a method body inside a contract is an error. A class opts into a contract using `for`, then marks each satisfying method with `impl`.

<!-- iris-example: {"id":"06-contracts-declare-promises","mode":"vm","stdout":"Beep boop\n"} -->
```iris
contract Speaker {
  fun speak() -> String
}

class Robot for Speaker {
  public impl fun speak() -> String {
    "Beep boop"
  }
}

let bot = Robot.new() as Speaker
print(bot..speak())
```

Expected terminal output:

```text
Beep boop
```

The declaration `for Speaker` creates a permanent static fact on `Robot`. Dynamic updates can replace compatible method bodies later, but runtime mutation cannot erase the promise that `Robot` implements `Speaker`.

The `impl` keyword indicates that a method satisfies a contract requirement. If a method replaces an inherited or mixed-in method while also fulfilling a contract, both keywords are written together, such as `override impl fun draw() -> Nil { ... }`.

## Qualified Contract slots

Ordinary message sends never overload by static type. If two contracts require identical selectors with differing signatures, Iris avoids ambiguity through qualified contract slots. You implement each contract requirement explicitly and invoke them through contract views using `..`.

<!-- iris-example: {"id":"06-qualified-contract-slots","mode":"vm","stdout":"parse: text\nvalidate: text\n"} -->
```iris
contract Parser {
  fun process(input: String) -> String
}

contract Validator {
  fun process(input: String) -> String
}

class Tool for Parser, Validator {
  public impl fun Parser::process(input: String) -> String {
    "parse: " + input
  }
  public impl fun Validator::process(input: String) -> String {
    "validate: " + input
  }
}

let tool = Tool.new()
let p = tool as Parser
let v = tool as Validator
print(p..process("text"))
print(v..process("text"))
```

Expected terminal output:

```text
parse: text
validate: text
```

In declarations, `Contract::selector` assigns a method to a specific contract slot. In expressions, `view..selector` dispatches directly to that qualified slot. A single dot `view.selector` remains an ordinary receiver message send.

## Static promise, dynamic freedom

Iris balances dynamic adaptability with static consistency.

**Dynamic freedom**
- Classes can mix in modules at declaration or during validated open transactions.
- Method bodies can be replaced dynamically with compatible implementations.
- Contract views can be constructed at runtime across conforming instances.

**Static promises**
- A class retains a single fixed superclass.
- Declared contract obligations cannot be silently stripped or bypassed.
- Ordinary selectors and contract-qualified slots occupy distinct namespaces.
- Attempts to remove declared contracts at runtime via reflection fail immediately with `TypeContractError`.

## Kernel is always in scope

`Kernel` is the language core module composed into `Object`. It provides declarations that must remain visible everywhere without explicit imports, including built-in type aliases and fundamental utilities.

Because `Kernel` is an ordinary module, standard composition and lookup rules apply. It is not a secondary root class. `Kernel` cannot shadow or replace `Object` primitives, including default comparison, truthiness evaluation, missing-message handling, or zero-argument initialization.

**Hands-on Exercise**

Create a contract `Describable` with a required method `fun describe() -> String`. Define a module `Tagged` providing `public fun tag() -> String { "[tag]" }`. Then declare a class `Item for Describable mixin Tagged` satisfying `describe`. Instantiate `Item`, cast it to `Describable`, and print both its description and its tag. Run the script with `./target/debug/iris --vm item.iris`.

Expected terminal output:

```text
item-ready
[tag]
```

## Read the spec

To study normative definitions and exact semantics, refer to [03-runtime-object-model.md](../../spec/iris-v1/03-runtime-object-model.md) and [05-types-contracts-generics.md](../../spec/iris-v1/05-types-contracts-generics.md):

| Clause | Topic |
| --- | --- |
| `IRIS-V1-RUNTIME-C046` through `IRIS-V1-RUNTIME-C053` | Module composition and MRO ordering. |
| `IRIS-V1-RUNTIME-C030` through `IRIS-V1-RUNTIME-C033` | Contract view dispatch and missing qualified slots. |
| `IRIS-V1-RUNTIME-C163` | `Kernel` as the always-visible language core module. |
| `IRIS-V1-TYPES-C041` through `IRIS-V1-TYPES-C053` | Contract declarations, `for`, `impl`, qualified implementations, views, equality, and hashing. |
| `IRIS-V1-TYPES-C098` and `IRIS-V1-TYPES-C099` | Protected ancestry and refused `remove_contract`. |
| `IRIS-V1-GRAMMAR-C061` | The `private` marker on a mixin entry. |
| `IRIS-V1-GRAMMAR-C062` | Bodyless method declarations as contract requirements. |
| `IRIS-V1-IDENTITY-C009` and `IRIS-V1-IDENTITY-C010` | Prohibition of static type overload dispatch. |
