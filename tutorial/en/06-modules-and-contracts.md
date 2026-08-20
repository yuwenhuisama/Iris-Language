# Modules and Contracts

This chapter shows how Iris separates behavior reuse from promises. A `module` supplies Methods that a Class or another Module can mix in. A `contract` declares an obligation surface that a Class explicitly promises with `for` and satisfies with `impl`. The two work together, but they aren't interchangeable: Modules affect lookup order, while Contracts define checked slots and views.

```iris
module A { fun trace() -> Symbol { :A } }
module B mixin A { override fun trace() -> Symbol { :B } }
class C extends Object mixin A, B {}

C.new().trace()           // selects B before A
```

The snippet comes from `IRIS-V1-RUNTIME-C047` and `IRIS-V1-RUNTIME-C053`. A Class has one superclass chain, then composed Modules. Header `mixin A, B` is applied left to right, but lookup checks the closest Module first, so the effective order is `C`, then `B`, then `A`, then the superclass MRO.

## Modules add behavior

A Module is a named object that can hold Methods. A Class can compose Modules in its declaration. Another Module can also compose Modules, which lets common behavior collect into reusable layers.

```iris
module Trace {
  fun trace() -> Symbol { :trace }
}

class Job extends Object mixin Trace {}
```

This is not multiple inheritance. A Class still has one superclass. Modules are inserted into the Method lookup order and are deduplicated by Module identity. Re-including the same Module doesn't create a second copy or move the old one.

Module Methods run with the current receiver. If a Module Method reads `@state`, it reads the receiver's slot named `@state`, not storage owned by the Module. Private Method access is different: Module composition doesn't grant private access unless the composition edge explicitly asks for it, which is written by marking that entry `private` in the mixin list.

```iris
class Job extends Object mixin Trace private, Audit {}
```

The marker sits on the individual composition edge, so `Trace` gets private authorization here and `Audit` does not. Raw `@x` access on the current receiver is a separate rule and doesn't depend on this marker.

## Contracts declare promises

A Contract is Iris's explicit obligation surface. A Contract body contains requirements, not executable Method bodies or storage. A requirement is a Method declaration with a signature and no body; writing a body inside a Contract is rejected as `CONTRACT_METHOD_BODY_FORBIDDEN`. A Class opts into a Contract with `for`, then marks the satisfying member with `impl`.


```iris
contract Printable<T> where T: Object {
  fun print(value: T) -> String
}

class User for Printable<User> {
  impl fun print(value: User) -> String {
    value.name
  }
}

let view = User.new() as Printable<User>
view..print(User.new())
```

This example is reused from `IRIS-V1-TYPES-EX006`. The `for Printable<User>` declaration is a static spine fact for `User`. Later dynamic changes can replace compatible bodies, but they can't erase the fact that `User` promised `Printable<User>`.

The `impl` marker matters. It says the Method is meant to satisfy a Contract requirement. If the Method also replaces an inherited or mixed-in Method, Iris writes both markers, for example `override impl fun draw() -> Nil { ... }`.

## Qualified Contract slots

Ordinary dispatch doesn't overload by static type. If two Contracts require the same selector with incompatible signatures, Iris doesn't guess. You write explicit qualified implementations and call them through a Contract view with `..`.

```iris
contract Parser {
  fun process(input: String) -> Object
}

contract Validator {
  fun process(input: Object) -> Bool
}

class Tool for Parser, Validator {
  impl Parser::process(input: String) -> Object { input }
  impl Validator::process(input: Object) -> Bool { true }
}

let parser = Tool.new() as Parser
parser..process("source")
```

This is reused from `IRIS-V1-TYPES-EX007`. The declaration form uses `Parser::process` to name a Contract slot. The expression form uses `..process` to call the qualified slot. A single dot keeps ordinary dispatch.

```iris
let view = parser as ParserContract
parser.process(input)     // ordinary selector process
view.process(input)       // still ordinary selector process on the receiver
view..process(input)      // qualified Contract slot ParserContract::process
```

This snippet is reused from the runtime examples. It is the difference to remember: `as ParserContract` checks or constructs a Contract view, but `view.process(input)` is still ordinary selector lookup. Only `view..process(input)` selects the Contract-qualified namespace.

## Static promise, dynamic freedom

Dynamic freedom: a Class can mix in Modules, replace compatible Method bodies, and form Contract views at runtime. Module order can change through a validated open transaction.

Static promise: the Class has one stable superclass promise, one declared Contract set, one non-overloaded ordinary selector namespace, and one explicit Contract-qualified namespace. A later dynamic change must preserve the static spine before it can publish.

That split is why Contract dispatch is explicit. Iris lets runtime behavior move, but it refuses to hide a type-directed overload decision inside a normal call.

The declared Contract set is one of the strongest static facts a Class has. A runtime attempt to remove one — spelled `remove_contract(contract)` or `Reflection::Class.remove_contract(target, contract)` — exists only so the refusal is observable: it is rejected with `TypeContractError` before publication, and the Class keeps its Contracts and its active revision. A runtime superclass change is likewise rejected when the proposed ancestry would drop an ancestor carrying a static fact the target relies on.

## Kernel is always in scope

`Kernel` is the language-core Module composed into `Object`. It carries the declarations that must be visible everywhere without an import, such as the `Block<S>` Type alias from chapter 04. It is an ordinary Module with ordinary composition and lookup rules, and it is not a second root Class; it may not supply or shadow `Object`'s default comparison, truthiness, missing-message, or zero-argument `initialize`.

## Read the spec

For exact rules, read [03-runtime-object-model.md](../../spec/iris-v1/03-runtime-object-model.md) and [05-types-contracts-generics.md](../../spec/iris-v1/05-types-contracts-generics.md):

| Clause | Topic |
| --- | --- |
| `IRIS-V1-RUNTIME-C046` through `IRIS-V1-RUNTIME-C053` | Module composition and MRO ordering. |
| `IRIS-V1-RUNTIME-C030` through `IRIS-V1-RUNTIME-C033` | Contract view dispatch and missing qualified slots. |
| `IRIS-V1-RUNTIME-C163` | `Kernel` as the always-visible language-core Module. |
| `IRIS-V1-TYPES-C041` through `IRIS-V1-TYPES-C053` | Contract declarations, `for`, `impl`, qualified implementations, views, equality, and hashing. |
| `IRIS-V1-TYPES-C098` and `IRIS-V1-TYPES-C099` | Protected ancestry and the refused `remove_contract`. |
| `IRIS-V1-GRAMMAR-C061` | The `private` marker on a mixin entry. |
| `IRIS-V1-GRAMMAR-C062` | Bodyless Method declarations as Contract requirements. |
| `IRIS-V1-IDENTITY-C009` and `IRIS-V1-IDENTITY-C010` | No static type overload dispatch. |
