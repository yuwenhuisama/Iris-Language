# Iris v1 Types, Contracts, And Generics

Status: Iris v1.1, frozen semantics with owner-approved errata.

IRIS-V1-TYPES-C001: This chapter defines gradual type Contracts, `Dynamic<T>`, runtime type tests and casts, Type objects, top and bottom types, nilability, `NonNil`, union and intersection algebra, callable subtyping, Contract declarations and views, generic constraints and materialization, Type aliases, and `Never` flow for Iris v1. It MUST be read after [README.md](README.md), [01-language-identity.md](01-language-identity.md), [02-lexical-grammar.md](02-lexical-grammar.md), [03-runtime-object-model.md](03-runtime-object-model.md), and [04-bindings-callables-control-flow.md](04-bindings-callables-control-flow.md).

IRIS-V1-TYPES-C002: This chapter MUST NOT define overload dispatch, declaration-site variance, use-site projection, raw generic instance types, implicit generic conversion, recursive Type aliases, higher-kinded types, specialization as source semantics, conditional types, mapped types, arbitrary compile-time execution, or structural auto-conformance. Later chapters MUST preserve these exclusions when refining packages, reflection, conformance, migration, native binding, and library APIs.

## Gradual Contracts And Boundary Enforcement

IRIS-V1-TYPES-C003: Type annotations are optional to write and mandatory to obey once written. An omitted Method parameter or return annotation has static and runtime Contract `Dynamic<Object>`. An implementation MAY infer local facts inside a body for diagnostics or optimization, but it MUST NOT export inferred body facts as signature metadata.

IRIS-V1-TYPES-C004: A written type annotation on a binding, property, parameter, return, generic argument, Contract requirement, native metadata entry, or Type alias target is both a static Contract and a runtime boundary guard. A provable violation MUST be diagnosed before execution. A not-proven boundary MUST check at runtime before publishing or passing the value across that boundary.

IRIS-V1-TYPES-C005: Runtime type guards MUST preserve dynamic dispatch semantics. They validate values, callable results, property writes, generic materialization, native metadata, reflection construction, and Contract views, but they MUST NOT pick a different ordinary selector, define overload dispatch, copy a value, convert data, or change the runtime receiver Class.

IRIS-V1-TYPES-C006: Boundary Contract failures MUST raise or report `TypeError`, `TypeContractError`, or a more specific ordinary Iris diagnostic named by the owning chapter. JIT, native, Host, reflection, Dynamic, and package paths MUST preserve the same boundary checks unless proof against still-valid static spines permits a duplicate physical guard to be removed.

IRIS-V1-TYPES-C007: The following gradual boundary table is normative:

| Boundary                 | Syntax or source     | Static rule                          | Runtime guard                                       | Reflection metadata                              |
| ------------------------ | -------------------- | ------------------------------------ | --------------------------------------------------- | ------------------------------------------------ |
| Omitted Method parameter | `fun f(value)`     | `Dynamic<Object>`                  | Accepts any Object after Dynamic bound check        | Parameter Type is`Dynamic<Object>`             |
| Omitted Method return    | `fun f() { body }` | `Dynamic<Object>`                  | Return value checked against Dynamic bound          | Return Type is`Dynamic<Object>`                |
| Binding annotation       | `let x: T = expr`  | `expr` assignable to `T`         | Stored value must satisfy`T`                      | Binding or debug metadata records`T`           |
| Property write           | `property name: T` | Written value assignable to`T`     | Setter or storage checks`T`                       | Property Type is`T`                            |
| Call argument            | `target(arg)`      | Actual assignable to parameter Type  | Dynamic actual checked before body entry            | Method signature records parameter Types         |
| Callable return          | `fun f() -> T`     | Every normal path assignable to`T` | Returned value checked before caller observes it    | Method signature records return Type             |
| Generic materialization  | `Box<T>`           | Arguments satisfy constraints        | Reflection/Dynamic/native paths recheck constraints | Closed Type records normalized arguments         |
| Contract view            | `value as C`       | Value possibly conforms to`C`      | View construction checks nominal conformance        | View records receiver relation and Contract Type |

IRIS-V1-TYPES-EX001: Informative example, optional annotations still enforce written Contracts:

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

## Type Expression Constructors

IRIS-V1-TYPES-C008: The parser-visible type expression constructors used by this chapter are nominal Type names, closed generic applications `G<T, U>`, callable Types `Closure<(P1, P2, ...) -> R>`, `BoundMethod<...>`, and `Block<...>` per IRIS-V1-TYPES-C094, unions `A | B`, intersections `A & B`, nilability suffix `T?`, `Dynamic<T>`, `Dynamic`, `NonNil`, `Never`, `Nil`, `Object`, parenthesized Type expressions, and transparent Type aliases. `&` binds tighter than `|` inside Type expressions. Parentheses MUST be used for any intended different grouping.

IRIS-V1-TYPES-C009: `Object` is the top Type. Every Iris value, including `nil`, singleton values, identity-bearing values, identity-less values, Class objects, Module objects, Contract objects, Type objects, Methods, BoundMethods, Closures, and future core values, is a subtype of `Object` unless a later chapter explicitly marks a value category outside ordinary Iris values.

IRIS-V1-TYPES-C010: `Never` is the bottom Type. `Never` has no normally constructible runtime values, is assignable to every Type, and is produced by raising expressions, bare re-raise, statically proven nonterminating paths, unreachable paths, and calls whose declared return Type is `Never`.

IRIS-V1-TYPES-C011: `Nil` is the Type of the unique identity-bearing `nil` singleton and is a subtype of `Object`. Concrete Types such as `String` and `User` do not implicitly include `nil`. A program that accepts `nil` with another Type MUST write `T?` or `T | Nil`, except where `T` is already a supertype containing `Nil`.

IRIS-V1-TYPES-C012: `T?` is exact sugar for `T | Nil`. It is not a separate Type constructor after normalization. Type identity, reflection, hashing, assignability, runtime guards, generic arguments, narrowing, and JIT semantics for `T?` MUST be identical to `T | Nil`.

IRIS-V1-TYPES-C013: `NonNil` is an intrinsic marker Type satisfied by every value except `nil`. It declares no Methods, adds no runtime superclass, and cannot be acquired by `Nil` through metaprogramming. `Object & NonNil` is the canonical non-nil top Type.

IRIS-V1-TYPES-C014: `Dynamic<T>` is bounded dynamic message sending. Bare `Dynamic` normalizes to `Dynamic<Object>`. Entering `Dynamic<T>` checks that the value satisfies reified `T`; inside the Dynamic boundary, arbitrary ordinary selectors may be sent without static member validation. Qualified Contract dispatch still requires an explicit Contract view and `..` call.

IRIS-V1-TYPES-C015: Type aliases use `type Name<T...> = TypeExpr where ...` where the `where` clause is optional and follows the generic constraint grammar. Aliases are transparent and create no nominal runtime wrapper. Direct or indirect recursive aliases, including aliases recursive only through containers, MUST be compile-time errors in v1.

IRIS-V1-TYPES-C016: Runtime Type objects are interned identity-bearing objects distinct from Class, Module, and Contract objects. Every normalized reified Type expression has exactly one interned Type identity per applicable runtime and package identity scope. `same?` and default `==` on Type objects compare canonical Type identity.

IRIS-V1-TYPES-C017: The following constructor coverage table is normative:

| Constructor      | Syntax                 | Normalization                                                              | Assignability                                                 | Runtime guard                                   | Reflection                                             |
| ---------------- | ---------------------- | -------------------------------------------------------------------------- | ------------------------------------------------------------- | ----------------------------------------------- | ------------------------------------------------------ |
| Nominal Class    | `User`               | Canonical named Type identity                                              | Subclass and self assign to it                                | Current runtime ancestry check                  | `kind: class`, name, package, Class object           |
| Nominal Contract | `Readable`           | Canonical named Type identity                                              | Explicit conforming Classes assign to it                      | Nominal Contract conformance check              | `kind: contract`, name, package, Contract object     |
| Closed generic   | `Box<String>`        | Interned by definition and normalized arguments                            | Invariant exact construction plus subtype rules inside bounds | Argument constraints and nominal membership     | `kind: generic`, definition, arguments               |
| Callable         | `(String) -> Object` | Canonical parameter categories and return Type                             | Parameter contravariance, return covariance, same call shape  | Callable value plus boundary checks             | `kind: callable`, parameters, return                 |
| Union            | `A \| B`              | Flatten, sort, dedup, absorb subtypes, bottom identity                     | Source assignable when assignable to at least one branch      | Any branch guard succeeds                       | `kind: union`, normalized members                    |
| Intersection     | `A & B`              | Flatten, sort, dedup, absorb supertypes, top identity, impossible to Never | Source assignable when it satisfies every member              | All member guards succeed                       | `kind: intersection`, normalized members             |
| Nilable          | `T?`                 | `T \| Nil`                                                                | Same as normalized union                                      | Same as normalized union                        | Reflected as normalized union, display may prefer`?` |
| Dynamic          | `Dynamic<T>`         | Bare`Dynamic` to `Dynamic<Object>`                                     | `T` to `Dynamic<T>`, bounded Dynamic widening             | Check bound on entry and on exit to static Type | `kind: dynamic`, bound                               |
| NonNil           | `NonNil`             | Intrinsic marker,`Nil & NonNil` to `Never`                             | Any non-nil value assignable                                  | Rejects only`nil`                             | `kind: marker`, name `NonNil`                      |
| Object           | `Object`             | Top, empty intersection identity                                           | Every value assignable                                        | Always succeeds for Iris values                 | `kind: class`, root Class object                     |
| Never            | `Never`              | Bottom, empty union identity                                               | Assignable to every Type                                      | No normal value can satisfy construction        | `kind: never`                                        |
| Alias            | `Name<T>`            | Expands then normalizes target                                             | Same as expanded target                                       | Same as expanded target                         | Canonical target Type identity                         |

IRIS-V1-TYPES-EX002: Informative example, Type expression constructors:

```iris
type Name = String & NonNil
type MaybeUser<T> = User<T>?

let printer: (Object) -> Nil = { |value: Object| -> Nil; print(value) }
let item: Dynamic<Object> = load_external_value()
let checked = item as? Name
```

## Union, Intersection, Nil, And NonNil Algebra

IRIS-V1-TYPES-C018: Union and intersection Types are commutative, associative, idempotent, flattened, sorted by stable Type identity order, and interned after normalization. Duplicate members MUST collapse. Reordered or regrouped equivalent expressions MUST produce the same Type object identity.

IRIS-V1-TYPES-C019: Nominal subtype absorption applies during normalization. If `Dog <: Animal`, then `Dog | Animal` normalizes to `Animal`, and `Dog & Animal` normalizes to `Dog`. Absorption MUST use immutable nominal superclass, declared Contract, and closed generic identity facts, not structural member shape.

IRIS-V1-TYPES-C020: `T | Never` normalizes to `T`, `T & Never` normalizes to `Never`, the empty union identity is `Never`, and `T & Object` normalizes to `T`. `Object | T` normalizes to `Object`, and the empty intersection identity is `Object`.

IRIS-V1-TYPES-C021: `Object | Nil` and `Object?` normalize to `Object`. `Nil & Object` normalizes to `Nil`. `Never?` normalizes to `Nil` because `Never | Nil` has exactly the `Nil` inhabitant.

IRIS-V1-TYPES-C022: Intersecting a union with `NonNil` removes its `Nil` member semantically. `(A | Nil) & NonNil` normalizes to `A & NonNil`, and if `A` is known non-nil, then to `A`. `Nil & NonNil` normalizes to `Never`. For an unconstrained type parameter `T`, `T & NonNil` remains explicit and MUST NOT simplify to `T`.

IRIS-V1-TYPES-C023: Canonical Type identity MUST NOT expand intersections over unions or unions over intersections into canonical DNF or CNF. Assignability, overlap, and narrowing algorithms MAY reason with distributive equivalence on demand, but they MUST NOT intern or expose exponential expanded Type graphs as the canonical form.

IRIS-V1-TYPES-C024: A statically impossible intersection normalizes to `Never` when nominal finality, Contract requirements, generic invariance, or other frozen facts prove no runtime value can satisfy all constituents. If openness prevents proof, the Type MAY remain as a guarded intersection and runtime conformance validation decides each candidate.

IRIS-V1-TYPES-C025: Union member access is permitted only for members present on every branch with one safe call contract. For each same-named member across union branches, static checking MUST model each branch's accepted argument-count set, including required parameters, optional parameters, positional rest, keyword-only parameters, keyword rest, and optional block shapes. A call arity is permitted only when it belongs to every branch's accepted set. Variadic or rest acceptance in one branch MUST NOT override another branch's finite restriction. For the actual positions, keywords, and block channel supplied by an accepted arity, the permitted input domain is the intersection of accepted branch domains, and the result Type is the normalized union of branch return Types. The common API need not be representable as one simplified declaration. If argument-count sets, parameter modes, visibility, effects, Contracts, or Types cannot produce one all-branches-safe call, the member is unavailable until flow narrowing.

IRIS-V1-TYPES-C026: Intersection same-name obligations require one compatible implementation. Iris MUST NOT create overload sets, pick by static argument Type, prefer declaration order, or choose arbitrarily. Incompatible same-name requirements either make the type impossible, make the declaration unsatisfiable, or require explicit qualified Contract slots.

IRIS-V1-TYPES-C027: The following algebra vector table is normative. The conformance chapter MUST preserve these vector IDs or map them to machine-readable records with the same expected normalized Type identity:

| Vector ID              | Law                             | Expression                                            | Expected normalized Type                         |
| ---------------------- | ------------------------------- | ----------------------------------------------------- | ------------------------------------------------ |
| `IRIS-V1-TYPES-V001` | Commutativity, union            | `String \| Integer` and `Integer \| String`         | Same interned union Type                         |
| `IRIS-V1-TYPES-V002` | Commutativity, intersection     | `Readable & Closeable` and `Closeable & Readable` | Same interned intersection Type                  |
| `IRIS-V1-TYPES-V003` | Idempotence, union              | `String \| String`                                   | `String`                                       |
| `IRIS-V1-TYPES-V004` | Idempotence, intersection       | `String & String`                                   | `String`                                       |
| `IRIS-V1-TYPES-V005` | Absorption, union               | `Dog \| Animal` where `Dog <: Animal`              | `Animal`                                       |
| `IRIS-V1-TYPES-V006` | Absorption, intersection        | `Dog & Animal` where `Dog <: Animal`              | `Dog`                                          |
| `IRIS-V1-TYPES-V007` | Bottom union identity           | `String \| Never`                                    | `String`                                       |
| `IRIS-V1-TYPES-V008` | Bottom intersection annihilator | `String & Never`                                    | `Never`                                        |
| `IRIS-V1-TYPES-V009` | Top intersection identity       | `String & Object`                                   | `String`                                       |
| `IRIS-V1-TYPES-V010` | Top union absorber              | `String \| Object`                                   | `Object`                                       |
| `IRIS-V1-TYPES-V011` | Nilability sugar                | `String?`                                           | `String \| Nil` normalized optional display     |
| `IRIS-V1-TYPES-V012` | Object optional collapse        | `Object?`                                           | `Object`                                       |
| `IRIS-V1-TYPES-V013` | Never optional collapse         | `Never?`                                            | `Nil`                                          |
| `IRIS-V1-TYPES-V014` | NonNil nil removal              | `(String \| Nil) & NonNil`                           | `String`                                       |
| `IRIS-V1-TYPES-V015` | NonNil nil impossibility        | `Nil & NonNil`                                      | `Never`                                        |
| `IRIS-V1-TYPES-V016` | No canonical distribution       | `A & (B \| C)`                                       | Compact intersection containing the union member |

IRIS-V1-TYPES-EX003: Informative example, nil narrowing and union calls:

```iris
let name: String? = load_name()

if name != nil {
  let strong: String = name
}

let value: String | MutableString = load_text()
value.length()
```

## Assignability, Tests, Casts, And Flow Narrowing

IRIS-V1-TYPES-C028: `value is T` evaluates `value` once and returns `Bool`. It checks reified Types, including nominal Classes, current runtime ancestry, Contracts, normalized unions and intersections, closed invariant generic arguments, callable Types, value Types, meta Types, `NonNil`, `Never`, and `Dynamic<T>` bounds. It MUST NOT call `to_bool` on the value and MUST NOT perform data conversion.

IRIS-V1-TYPES-C029: `value as T` evaluates `value` once, proves or checks that the value satisfies reified `T`, returns the same underlying object or value on success, and raises `TypeError` on failure. Proven casts MAY eliminate runtime guards. Statically impossible casts MUST be compile-time errors.

IRIS-V1-TYPES-C030: `value as? T` evaluates `value` once and returns `T?`. It returns the same underlying object or value on success and `nil` on failed runtime check. Statically impossible safe casts MUST be diagnosed rather than used as a hidden fallback idiom.

IRIS-V1-TYPES-C031: Casts never perform data conversion, copying, allocation of a replacement object, collection element conversion, numeric conversion, text conversion, or generic argument conversion. `as` cannot cross invariant generic arguments. Programs that need converted data MUST call explicit APIs.

IRIS-V1-TYPES-C032: `as ContractType` constructs a checked Contract view. A storable Contract view carries the receiver relation plus Contract identity or conformance. Ordinary `view.member()` remains an unqualified message forwarded to the receiver. Qualified invocation MUST use `view..member()` or `(value as ContractType)..member()`.

IRIS-V1-TYPES-C033: Flow narrowing from `is`, `as?`, nil comparison, typed catch, and match Type patterns uses intersection on true paths and exclusion of the proven branch where the Type algebra can represent it. Iris v1 does not introduce general type subtraction beyond the explicit `NonNil` and nil-removal rules in this chapter.

IRIS-V1-TYPES-C034: A typed catch that names a Class or Contract performs one checked narrowing before the catch body runs. The catch binding is immutable and has that static narrowed Type. A later Class revision cannot remove or weaken the declared facts that justified the narrowing.

IRIS-V1-TYPES-C035: The following runtime type operation table is normative:

| Operation     | Syntax                                | Success result                   | Failure result      | Flow effect                                                       | Boundary preservation               |
| ------------- | ------------------------------------- | -------------------------------- | ------------------- | ----------------------------------------------------------------- | ----------------------------------- |
| Type test     | `value is T`                        | `true`                         | `false`           | True path narrows by`T`; false path excludes when representable | No conversion or selector change    |
| Strict cast   | `value as T`                        | Same value typed as`T`         | Raises`TypeError` | Following expression has`T`                                     | No conversion or invariant crossing |
| Safe cast     | `value as? T`                       | Same value typed as`T`         | `nil`             | Result Type is`T?`                                              | No conversion or invariant crossing |
| Contract view | `value as C`                        | View or same storable view value | Raises`TypeError` | Result can use`..` for qualified slots                          | Ordinary dot still unqualified      |
| Dynamic entry | `value as Dynamic<T>` or assignment | Same value with Dynamic bound    | Raises`TypeError` | Static member checking disabled inside bound                      | Later static boundary rechecks      |

IRIS-V1-TYPES-EX004: Informative example, casts preserve the value:

```iris
let object: Object = load_value()

if object is String {
  object.length()
}

let maybe_user: User? = object as? User
let printable = object as Printable
printable..print()
```

## Callable Types And Method Contract Compatibility

IRIS-V1-TYPES-C036: Superseded by IRIS-V1-TYPES-C094 and IRIS-V1-TYPES-C096 in the v1.11 errata, which reify callable kind as `Closure<S>` and `BoundMethod<S>` and make callable Type arguments invariant. The superseded text read: BoundMethod and Closure values share ordinary callable Types written `(P1, P2, ...) -> R`. The full callable Type includes arity, parameter category, keyword names, rest and keyword-rest channels, optional block channel, parameter Types, return Type, and required runtime Contracts.

IRIS-V1-TYPES-C037: Callable assignability uses standard function subtyping. An implementation or source callable is assignable to a promised callable Type only when it accepts every call shape the promise permits and returns values assignable to the promised return Type. Parameter positions are contravariant, return positions are covariant, and callable Types inside parameter or return positions apply the same rule recursively.

IRIS-V1-TYPES-C038: Method implementation compatibility for superclass, Contract, Module, intersection, open, package upgrade, native metadata, and dynamic replacement uses the callable subtyping rule in IRIS-V1-TYPES-C037. A compatible body replacement may change implementation details but MUST preserve the static promise before atomic publication.

IRIS-V1-TYPES-C039: Unbound Method objects are reflective definition objects, not ordinary callable values. They use a distinct reified Method metadata Type and require explicit receiver binding or reflective invocation validation before execution. Assigning an unbound Method object to `(P...) -> R` MUST fail unless an explicit binding operation produces a BoundMethod.

IRIS-V1-TYPES-C040: Callable compatibility creates no overload. Same selector, same qualified Contract slot, or same property accessor location still has one selected Method identity. Static annotations, expected return Type, union branch, or generic argument MUST NOT choose among multiple implementations.

IRIS-V1-TYPES-EX005: Informative example, callable subtyping:

```iris
fun accepts_object(value: Object) -> String {
  value.to_string()
}

let string_to_object: (String) -> Object = accepts_object
```

## Contract Declarations, Inheritance, Implementations, And Views

IRIS-V1-TYPES-C041: Contract is the exclusive Iris v1 term for the explicit static and dynamic obligation surface. Contract declarations use `contract Name<T...> extends ParentA, ParentB where T: ConstraintExpr meta deny capability { ... }`, with each allowed clause appearing at most once and in grammar order. A Contract without `extends` has no parent Contracts.

IRIS-V1-TYPES-C042: Contract bodies may declare public or protected instance Method requirements, Class-object Method requirements, property requirements, and generic Method requirements with `where` constraints. Contract bodies MUST NOT contain Method bodies, default implementations, stored state, initializers, raw ivars, private requirements, executable statements, or open operations.

IRIS-V1-TYPES-C043: A Contract may inherit multiple named Contracts. Inheritance forms the union of static requirements without implementation MRO, `super`, stored state, or Method bodies. Compatible same-name requirements merge into one obligation under callable compatibility. Incompatible same-name requirements make the Contract declaration a compile-time error unless they are represented through explicit qualified Contract slots as separate identities.

IRIS-V1-TYPES-C044: Class Contract conformance is declared in the Class header using `for`, as in `class Sprite extends Node for Drawable, Serializable { ... }`. Only Classes declare instance Contract conformance with `for`. Module composition never transfers nominal conformance by itself, and a final Class static spine must list each claimed Contract explicitly.

IRIS-V1-TYPES-C045: Declared Contract conformance is immutable for a given nominal Class declaration and revision static spine. Metaprogramming cannot add, remove, rename, weaken, or incompatibly replace declared Contract facts in place. Candidate revisions, opens, package upgrades, native metadata, and reflection construction MUST validate the complete static spine before publication.

IRIS-V1-TYPES-C046: A member intended to satisfy a declared Contract requirement MUST use `impl`. The compiler rejects `impl` when no declared Contract member is satisfied and rejects an unmarked Class-provided implementation where an explicit implementation is required. If the declaration also replaces an inherited or Module Method, both modifiers are written, for example `override impl fun draw() -> Nil`.

IRIS-V1-TYPES-C047: One `impl` member automatically satisfies every declared same-name Contract requirement whose slot is compatible with that single dynamic Method signature. It need not list targets. If same-name Contract requirements are incompatible, Iris MUST NOT choose by Contract order and MUST NOT form overloads. The declaration must use explicit qualified Contract implementations for the incompatible slots.

IRIS-V1-TYPES-C048: A qualified Contract implementation is declared `impl ContractType::member(...) -> R { ... }`, with `override` added when required by inherited or Module replacement rules. `::` is declaration and name qualification. It does not choose ordinary expression dispatch.

IRIS-V1-TYPES-C049: An explicit qualified Contract call is written `(value as ContractType)..member(args...)` or `view..member(args...)`. The `as ContractType` part proves or checks the nominal Contract view. The `..member` token chooses the Contract-qualified slot identity. Ordinary `value.member(args...)` always sends the unqualified selector.

IRIS-V1-TYPES-C050: Contract views are immutable identity-less capability values. `same?` on a Contract view MUST raise `IdentityError`. Repeated checked view construction MAY be allocation-free. Built-in view equality over identity-bearing receivers requires the same receiver identity and same Contract identity. Built-in view equality over identity-less receivers requires receiver equality under current equality plus the same Contract identity.

IRIS-V1-TYPES-C051: Contract view public hash derives from receiver public hash and Contract Type public hash. The derivation uses BLAKE3 derive-key mode with exact ASCII context `Iris Language v1 contract view hash` and input `receiver_public_hash_u64_le || contract_type_hash_u64_le`; digest reduction uses the first eight digest bytes as an unsigned little-endian `Integer`. Receiver hash failure propagates.

IRIS-V1-TYPES-C052: Named Contract Type public hash is based on canonical package identity, fully qualified Contract name, Iris language major, type kind, generic arity, API major, and closed argument identities when applicable. It MUST NOT depend on structural member shape, runtime allocation, filesystem path, display name, source hash, or machine-local build data.

IRIS-V1-TYPES-C053: The following Contract table is normative:

| Surface                  | Syntax                                              | Normalization or identity                        | Assignability                                  | Runtime guard                                     | Reflection                                           |
| ------------------------ | --------------------------------------------------- | ------------------------------------------------ | ---------------------------------------------- | ------------------------------------------------- | ---------------------------------------------------- |
| Contract declaration     | `contract C<T> extends P where T: Object { ... }` | Stable nominal Contract identity                 | Classes with explicit`for C<T>` satisfy it   | Static spine and candidate validation             | Contract object plus`.type`, parents, requirements |
| Class conformance        | `class A for C { ... }`                           | Immutable static spine fact                      | `A` instances assign to `C`                | Construction/open/upgrade validates requirements  | Class metadata lists Contracts                       |
| Implementation           | `impl fun m() -> R`                               | One Method slot satisfies compatible obligations | Compatible Method promise                      | Publication validates Method contract             | Method metadata records impl relation                |
| Qualified implementation | `impl C::m() -> R`                                | Distinct Contract slot identity                  | Satisfies that qualified slot                  | Qualified dispatch validates receiver conformance | Requirement and Method metadata link slot            |
| Contract view            | `value as C`                                      | Receiver relation plus Contract identity         | View assignable to`C` and Contract view Type | Checked nominal conformance                       | View Type records Contract and receiver relation     |
| Qualified call           | `view..m()`                                       | Uses Contract slot namespace                     | Requires view or checked`as C`               | Missing slot raises Contract dispatch error       | Call metadata names qualified slot                   |

IRIS-V1-TYPES-EX006: Informative example, Contract declaration and implementation:

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

IRIS-V1-TYPES-EX007: Informative example, incompatible same-name Contract slots:

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

## Generics, Constraints, Inference, And Materialization

IRIS-V1-TYPES-C054: Iris v1 supports user-defined generic Class, Contract, Module, and Method declarations with runtime-preserved arguments and metadata. Generic syntax uses angle brackets in declaration and Type grammar contexts, as in `class Box<T>`, `contract Iterable<T>`, `module Helpers<T>`, `fun map<T, U>(value: T) -> U`, and `Box<String>`.

IRIS-V1-TYPES-C055: Generic constraints use `where` clauses. The grammar separates constraint assignments with commas, as in `where T: A & B, U: C | D`. A comma starts a different parameter or `Self` constraint. `&`, `|`, parentheses, and Type aliases compose the Type expression for one constrained parameter. Comma MUST NOT mean intersection.

IRIS-V1-TYPES-C056: An unconstrained type parameter has implicit upper bound `Object`, not `Dynamic`, not `Object & NonNil`, and not a hidden non-nil bound. `Nil` is a valid argument for an unconstrained type parameter. Excluding `nil` requires an explicit constraint such as `where T: NonNil` or `where T: Object & NonNil`.

IRIS-V1-TYPES-C057: Generic declarations may constrain a parameter by peer parameters from the same declaration. Declaration order does not restrict references. The compiler and runtime solve the dependency graph using union and intersection normalization. Unsatisfiable or non-uniquely resolvable cyclic relations are compile-time declaration errors.

IRIS-V1-TYPES-C058: V1 permits restricted nominal F-bounded constraints, such as `where T: Comparable<T>`. A concrete argument satisfies such a bound only through explicit nominal Class or Contract conformance after substitution. Matching member shape is insufficient. Higher-kinded, arbitrary recursive type functions, fixed-point programs, and invalid recursive constraints are rejected.

IRIS-V1-TYPES-C059: Generic Class and Contract instantiations are strictly invariant. `G<A>` and `G<B>` have no subtype, assignment, or cast relation merely because `A <: B`. `as` and `as?` cannot cross invariant generic arguments. Collection or container conversion is explicit user code.

IRIS-V1-TYPES-C060: Every generic Class, Contract, Module, and Method application has fixed full arity. V1 has no default generic type arguments. Missing trailing arguments are arity errors and never mean `Object`, `Dynamic<Object>`, or an inferred default.

IRIS-V1-TYPES-C061: Bare generic Class names denote definition metadata only. For `class Box<T>`, bare `Box` is the identity-bearing generic definition object for reflection and opening the definition. It is not an instance Type, raw generic Type, or shorthand for `Box<Object>`. Instance annotations and ordinary construction require a closed `Box<Type>` or construction-site `Box<_>`.

IRIS-V1-TYPES-C062: Closed generic constructions are interned by generic definition identity and normalized ordered Type argument identities. Repeated `Box<String>` references produce the same closed Type and closed logical Class identity in one runtime. Differing definitions or differing normalized arguments produce different identities.

IRIS-V1-TYPES-C063: Opening a generic Class or Module definition builds a candidate definition plus substituted candidates for every already interned closed construction. All static spines, constraints, Contracts, MRO, Modules, native obligations, and layout obligations MUST validate as one transaction. `open class Box<String>` and programmatic `Box<String>.open` are errors in v1.

IRIS-V1-TYPES-C064: Ordinary generic class-level storage is per closed construction. A class-level property declared on `Generic<T>` has independent storage in `Generic<String>` and `Generic<User>`. A `shared class property` belongs to the unapplied generic definition and MUST NOT reference the definition's type parameters directly or indirectly.

IRIS-V1-TYPES-C065: Definition-wide shared generic properties are read and written only through the unapplied generic definition object, such as `Cache.count`. Closed Classes such as `Cache<String>` do not inherit or forward that accessor. Reflection MUST mark definition ownership versus per-closed ownership.

IRIS-V1-TYPES-C066: Shared generic property initializers run once during origin module initialization in source declaration order. Per-closed class property initializers run once when the closed Class is first materialized, inside the closed-Class creation transaction. A failed closed-Class materialization MUST discard candidate Class, property, Method, MRO, Contract, interning, and JIT-cache state and MUST publish and intern nothing for that closed construction. It MUST NOT automatically undo filesystem, network, database, native, logging, mutation to already-published objects, or other external side effects. Later requests MAY retry materialization and rerun initializers, and the initializer author owns cleanup, compensation, retry safety, and duplicate external side effects.

IRIS-V1-TYPES-C067: Closed generic materialization validates every normalized `where` constraint before interning or publishing. Reflection, Dynamic, plugin, deserialization, native construction, and package-loading paths MUST perform runtime validation unless proven duplicate by still-valid static facts. Failure raises `TypeContractError` and publishes no closed Class.

IRIS-V1-TYPES-C068: Generic Method type inference is local and bounded. Omitted Method type arguments are inferred only from actual arguments' static Types and an explicit immediate expected result Type. Inference MUST NOT inspect Method bodies, runtime values, arbitrary later uses, or whole-program state.

IRIS-V1-TYPES-C069: Inference combines multiple lower-bound candidates for one type parameter as their normalized union and multiple upper-bound candidates as their normalized intersection. The result must be one unique most-specific substitution that satisfies every bound and `where` expression. Otherwise the call requires explicit type arguments or is rejected.

IRIS-V1-TYPES-C070: An explicit expected result from an annotated assignment, return Contract, argument position, or equivalent immediate context MAY infer Method type arguments, including argument-free factories such as `let user: User = make()`. A standalone unconstrained call MUST NOT default the type parameter to `Object`.

IRIS-V1-TYPES-C071: Partial explicit Method type arguments use full-arity angle brackets with `_` placeholders, as in `choose<String, _>(value)`. Each `_` requests local inference for that position. Omitting trailing positions is an arity error. `_` in this position is a type-argument placeholder, not a binding and not a fresh user-named type variable.

IRIS-V1-TYPES-C072: Generic Class construction may use `_` only in a full-arity construction-site type argument list, such as `Box<_>.new(value)`. It is inferred from constructor arguments and immediate expected result Type. `_` is forbidden in persistent Type positions such as variable, property, parameter, return annotations, Contract declarations, base Types, stored reflection metadata, and generic constraints.

IRIS-V1-TYPES-C073: Generic Modules are reified and interned like other generic definitions. Generic Module arguments are supplied explicitly at the mix or include site and are not inferred from the host Class. Intrinsic `Self` may appear in Module `where` constraints and denotes the eventual receiver or host. V1 has no `_` inference for generic Module type arguments.

IRIS-V1-TYPES-C074: The following generic feature table is normative:

| Feature                      | Syntax                                   | Normalization or materialization                | Assignability                                  | Runtime guard                          | Reflection                                             |
| ---------------------------- | ---------------------------------------- | ----------------------------------------------- | ---------------------------------------------- | -------------------------------------- | ------------------------------------------------------ |
| Generic declaration          | `class Box<T> where T: Object {}`      | Definition identity plus parameter list         | Not an instance Type                           | Declaration constraints validated      | Generic definition metadata                            |
| Closed application           | `Box<String>`                          | Interned by definition and normalized arguments | Invariant exact construction                   | Materialization validates constraints  | Closed Type and logical Class metadata                 |
| Method inference             | `map<T, _>(value)`                     | Unique local substitution                       | Instantiated Method signature must be callable | Call boundary checks substituted Types | Inferred arguments appear in call metadata if exposed  |
| Class construction inference | `Box<_>.new(value)`                    | Full arity with local placeholders              | Creates closed Class if unique                 | Constructor and constraints checked    | Closed construction metadata                           |
| F-bound                      | `where T: Comparable<T>`               | Substitution then nominal conformance           | Argument accepted only by explicit conformance | Materialization revalidates            | Constraint graph records self reference                |
| Shared property              | `shared class property count: Integer` | One slot on generic definition                  | Not per closed Type                            | Initializer checked once               | Reflection marks definition storage                    |
| Per-closed property          | `class property value: T`              | One substituted slot per closed construction    | T-specific value Contract                      | Materialization initializer checked    | Reflection marks closed storage                        |
| Generic Module               | `mixin Helpers<User>`                  | Closed Module interned by explicit arguments    | Host must satisfy`Self` constraints          | Composition validates transactionally  | Module metadata records arguments and Self constraints |

IRIS-V1-TYPES-EX008: Informative example, generic constraints and invariance:

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

IRIS-V1-TYPES-EX009: Informative example, local generic inference:

```iris
fun make<T>() -> T where T: Object {
  load_value() as T
}

let user: User = make()
let box = Box<_>.new(user)
```

## Type Objects, Reflection, And Stable Type Identity

IRIS-V1-TYPES-C075: Type reflection exposes at least `kind`, `arguments`, `members`, `subtype?`, and `assignable?` for Type objects. Reflection MUST report normalized canonical Type identity, not source spelling, except where display APIs choose a preferred surface such as `T?`.

IRIS-V1-TYPES-C076: Class, Module, and Contract objects expose related `.type` metadata, but a Type object is not itself the Class, Module, or Contract object. This distinction allows union, intersection, `Dynamic<T>`, callable, `Never`, `NonNil`, nilable, and closed generic constructions to share one reflection API.

IRIS-V1-TYPES-C077: Type public hash is stable for publishable named and composite Types whose components have stable package identities. It is runtime-local for local anonymous or manifestless identities. Stable Type hash derivations MUST include Type kind and normalized component Type identities so that `A | B` and `B | A` hash identically after normalization.

IRIS-V1-TYPES-C078: Type reflection and hash identity MUST NOT depend on display name alone, member structural shape, runtime allocation order, filesystem path, source hash, or machine-local build data. Publishable nominal identity follows package ID, package API major, Iris language major, fully qualified name, Type kind, generic arity, and closed argument identities.

IRIS-V1-TYPES-C079: `Type#subtype?(other)` and `Type#assignable?(other)` MUST use the same nominal, generic invariance, callable subtyping, Dynamic-boundary, union and intersection, nilability, and `NonNil` rules as the compiler and runtime guards. They are queries over the current valid Type facts, not permission to bypass boundary checks.

IRIS-V1-TYPES-EX010: Informative example, Type reflection identity:

```iris
let a = (String | Nil).type
let b = String?.type

a same? b  // true
```

## Never Flow, Diagnostics, And Unreachable Code

IRIS-V1-TYPES-C080: `raise`, bare re-raise, statically proven nonterminating loops, calls declared `-> Never`, and statically unreachable paths have Type `Never`. `Never` paths do not widen result unions for `if`, `match`, loops, `try`, callable returns, or generic inference.

IRIS-V1-TYPES-C081: A callable declared `-> Never` MUST NOT complete normally. If normal completion is statically provable, it is a compile-time error. If a Dynamic, reflection, native, or Host path lets normal completion reach the boundary, the runtime MUST raise `TypeError` before a caller observes a normal result.

IRIS-V1-TYPES-C082: Statically unreachable statements receive a default compiler warning. They remain type-checked for diagnostics, name resolution, malformed syntax, and invalid Type use. Tool policy MAY promote the warning to an error without changing language semantics.

IRIS-V1-TYPES-C083: Exhaustive `match`, definite return, definite assignment, typed catch, and branch result analysis MUST treat `Never` as bottom. A branch that can only raise or call a `Never` callable contributes no value Type to the enclosing expression result.

IRIS-V1-TYPES-EX011: Informative example, Never flow:

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

## Exclusions And Traceability Notes

IRIS-V1-TYPES-C084: Iris v1 has no overload sets, no implicit type-directed dispatch, no declaration-site variance, no use-site projection, no raw generic instance Type, no implicit generic conversion, no default generic type arguments, no recursive Type aliases, no higher-kinded Types, no dependent Types, no non-type generic parameters, no variadic type parameters, no conditional Types, no mapped Types, no source-level specialization semantics, and no structural auto-conformance.

IRIS-V1-TYPES-C085: `Dynamic<T>` never erases boundary checks. Values entering a Dynamic bound are checked against `T`; values leaving Dynamic for a static boundary are checked against that boundary; parameter, return, property, generic, native, reflection, and Contract-view guards remain in force.

IRIS-V1-TYPES-C086: Contract terminology in this chapter is exclusive and canonical. Historical or non-canonical names for this concept MUST NOT appear in normative Iris v1 type, reflection, migration, native, library, or conformance artifacts except as clearly labeled history or migration replacement text outside this chapter.

IRIS-V1-TYPES-C087: Foundation conflict report: the historical design review examples at `legacy/Document/Iris Revival Design Review.md:831` through `:837` use `Array[Integer]`, while the frozen grammar and generic decisions require `Array<Integer>`. The same review section uses an older protocol word for Contract. This chapter follows the approved draft and current foundations: angle-bracket generics and canonical Contract terminology.

IRIS-V1-TYPES-C088: This chapter consolidates D-172 through D-220, D-233 through D-241, and D-452 through D-458. Later revised wording in the approved draft wins over older historical review phrasing.

IRIS-V1-TYPES-C089: Chapter-owned decision IDs are `D-172`, `D-173`, `D-174`, `D-175`, `D-176`, `D-177`, `D-178`, `D-179`, `D-180`, `D-181`, `D-182`, `D-183`, `D-184`, `D-185`, `D-186`, `D-187`, `D-188`, `D-189`, `D-190`, `D-191`, `D-192`, `D-193`, `D-194`, `D-195`, `D-196`, `D-197`, `D-198`, `D-199`, `D-200`, `D-201`, `D-202`, `D-203`, `D-204`, `D-205`, `D-206`, `D-207`, `D-208`, `D-209`, `D-210`, `D-211`, `D-212`, `D-213`, `D-214`, `D-215`, `D-216`, `D-217`, `D-218`, `D-219`, `D-220`, `D-233`, `D-234`, `D-235`, `D-236`, `D-237`, `D-238`, `D-239`, `D-240`, `D-241`, `D-452`, `D-453`, `D-454`, `D-455`, `D-456`, `D-457`, and `D-458`.

IRIS-V1-TYPES-C090: Locally referenced but delegated syntax, runtime, package, reflection, collection, and meta-header decision IDs are `D-077`, `D-242`, `D-243`, `D-244`, `D-245`, `D-246`, `D-247`, `D-248`, `D-249`, `D-250`, `D-258`, `D-259`, `D-260`, `D-261`, `D-262`, `D-275`, `D-276`, `D-277`, `D-278`, `D-279`, `D-280`, `D-281`, `D-282`, `D-283`, `D-298`, `D-466`, `D-495`, `D-507`, `D-508`, `D-509`, and `D-510`. Chapter-owned IDs listed in IRIS-V1-TYPES-C089 may still have non-type syntax, runtime dispatch, package identity, reflection, or metaprogramming anchors in other chapters; that does not make them delegated for this chapter's type semantics. Other chapters remain authoritative for details outside this chapter's type semantics.

IRIS-V1-TYPES-C091: The conformance chapter MUST include positive, failure, diagnostic, and reflection vectors for every Type constructor in IRIS-V1-TYPES-C017, every algebra vector in IRIS-V1-TYPES-C027, Contract qualified dispatch and view hash behavior, generic invariance and materialization, Dynamic boundary preservation, Type alias transparency, and `Never` flow.

## Type Coverage Vectors

IRIS-V1-TYPES-C092: The following vectors are normative traceability vectors with concrete type-checking inputs and expected observations.

IRIS-V1-TYPES-C093: In a Type-expression position, `typeof(expression)` denotes the normalized static Type of `expression`; its operand is type-checked but not evaluated. It therefore copies the Type available at that program point, including applicable flow narrowing, rather than inspecting a runtime value or invoking a Method. If that static Type is not known, including when it comes from an omitted Method return annotation, `typeof(expression)` is `Dynamic<Object>`. This construct creates no overload dispatch and MUST NOT cause static Type, generic arguments, expected result, declaration order, or body facts to select a different ordinary Method, consistent with IRIS-V1-IDENTITY-C010 and IRIS-V1-TYPES-C003.

IRIS-V1-TYPES-C094: The v1.11 errata reifies callable kind in the Type system. `Closure<S>` is the Type of a Closure value and `BoundMethod<S>` is the Type of a BoundMethod value, where `S` is the callable signature `(P1, P2, ...) -> R` defined by IRIS-V1-TYPES-C008. The signature is no longer a Type by itself: every callable annotation MUST name its kind. This supersedes IRIS-V1-TYPES-C036 and IRIS-V1-CONTROL-C018, which made BoundMethod and Closure share one unqualified callable Type, and supersedes the corresponding part of `D-425`. `IRIS-V1-CONTROL-C021` already makes the three callable kinds normative, and `D-425` already assigned unbound Methods a distinct reified Type, so this revision makes that treatment uniform across all three kinds rather than introducing a new concept.

IRIS-V1-TYPES-C095: `Block<S>` is a language-core Type alias declared as `type Block<S> = BoundMethod<S> | Closure<S>`. It is the Type of the trailing-block channel of IRIS-V1-GRAMMAR-C050, so a `block_parameter` accepts either bound callable kind while its annotation still names the kinds it admits. As a Type alias under IRIS-V1-TYPES-C015 it is transparent and creates no nominal runtime wrapper, and it normalizes as an ordinary union under IRIS-V1-TYPES-C018. `Block` is owned by the language core alongside `Object`, `Never`, and `NonNil`, not by the standard library.

IRIS-V1-TYPES-C096: Callable Type arguments are INVARIANT like every other generic argument under IRIS-V1-TYPES-C028 and IRIS-V1-TYPES-C031. `Closure<(Integer) -> Object>` is therefore not assignable to `Closure<(Integer) -> Symbol>` and the reverse does not hold either. This supersedes the parameter-contravariance and return-covariance assignability rule previously stated by IRIS-V1-TYPES-C036 and IRIS-V1-CONTROL-C018. Signature compatibility is checked AT THE CALL SITE against the invoked callable's declared signature, under the ordinary argument and return boundary rules of IRIS-V1-TYPES-C007, rather than through variance between callable Types.

| Vector ID | Category | Applicability | Source/Input | Expected observable | Decisions |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-TYPES-V018` | positive | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture: `contract Named { fun name() -> String } class User for Named { impl fun name() -> String { "iris" } } let view = User.new() as Named; view..name()`. | Value `"iris"`; Type `String`; checked view construction and explicit qualified dispatch select `Named::name`. | `D-233`, `D-234`, `D-236`, `D-237`, `D-239`, `D-278` |

## Audit-Exact Conformance Vectors

These rows are normative audit-exact vectors. Each names one concrete audited decision and its observable result.

| Vector ID | Category | Applicability | Source/Input | Expected observable | Decisions |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-TYPES-V200` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Metadata fixture:`Class A for C`; candidate removes `C`.                                                                                                               | `TypeContractError` before commit; `A` still conforms to `C`.                                                                | `D-173` |
| `IRIS-V1-TYPES-V201` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Metadata fixture:`class Dog extends Animal`; candidate changes superclass to `Object`.                                                                                 | `TypeContractError` before commit; `Dog.type.subtype?(Animal.type)` remains `true`.                                          | `D-174` |
| `IRIS-V1-TYPES-V202` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Metadata fixture: compose a Module whose`draw(String)` conflicts with declared Contract `draw(Integer)`.                                                               | `TypeContractError`; prior Class MRO and Contract conformance remain published.                                                  | `D-175` |
| `IRIS-V1-TYPES-V203` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Metadata fixture: open candidate replaces Contract-visible`draw() -> String` with `draw() -> Integer`.                                                                 | `TypeContractError`; no candidate Method or revision is published.                                                               | `D-176` |
| `IRIS-V1-TYPES-V204` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`open contract C { fun m() -> Nil }`.                                                                                                                | Static diagnostic`OPEN_CONTRACT_FORBIDDEN`; no Contract revision exists.                                                         | `D-177` |
| `IRIS-V1-TYPES-V205` | positive   | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`open class A { fun marker() -> String { "open" } } class A { fun marker() -> String { "origin" } } A.new().marker()`.                               | Value`"open"`; Type `String`; declaration collection resolves the origin before the open transaction.                          | `D-178` |
| `IRIS-V1-TYPES-V206` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Metadata fixture:`A.open { add valid m; add incompatible Contract method }`.                                                                                             | `TypeContractError`; reflection exposes neither staged member.                                                                   | `D-179` |
| `IRIS-V1-TYPES-V207` | positive   | compiler required; interpreter required; JIT required; native not applicable | Metadata fixture: inside`A.open`, write property `x`, read `x`, then force validation failure; concurrent reader queries `A.properties`.                           | Inner read returns the staged property; external query sees the old property set; rollback publishes no`x`.                      | `D-180` |
| `IRIS-V1-TYPES-V208` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Metadata fixture: programmatic open adds public`extra()` to `A`; static caller uses `A` while reflective caller invokes `extra`.                                   | Static call is rejected; reflective/Dynamic call is permitted only after successful publication.                                   | `D-181` |
| `IRIS-V1-TYPES-V209` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`class Box<T> {} let raw: Box = Box<String>.new()`.                                                                                                  | Static diagnostic`RAW_GENERIC_TYPE_FORBIDDEN`; no raw instance Type is formed.                                                   | `D-182` |
| `IRIS-V1-TYPES-V210` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`class Box<T> {} let item: Box[String] = Box<String>.new()`.                                                                                         | Parser diagnostic`GENERIC_BRACKET_SYNTAX_FORBIDDEN`; `Box<String>` remains the accepted spelling.                              | `D-183` |
| `IRIS-V1-TYPES-V211` | positive   | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`let value: String \| Integer = "iris"; value is String`.                                                                                             | Value`true`; Type `Bool`; annotation is reified union `String \| Integer`.                                                    | `D-184` |
| `IRIS-V1-TYPES-V212` | positive   | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`(String?).type same? (String \| Nil).type`.                                                                                                          | Value`true`; Type `Bool`; both expressions have one normalized Type identity.                                                  | `D-185` |
| `IRIS-V1-TYPES-V213` | positive   | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`(Dog \| Animal).type same? Animal.type`, where `class Dog extends Animal {}`.                                                                      | Value`true`; Type `Bool`; union absorbs `Dog`.                                                                               | `D-186` |
| `IRIS-V1-TYPES-V214` | positive   | compiler required; interpreter required; JIT required; native not applicable | Metadata fixture: reflect`A & (B \| C)`.                                                                                                                                  | Type kind`intersection`; members are `A` and normalized union `B \| C`, not distributed alternatives.                         | `D-187` |
| `IRIS-V1-TYPES-V215` | positive   | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`(Nil & NonNil).type`.                                                                                                                               | Type`Never`; no normal value satisfies the Type.                                                                                 | `D-188` |
| `IRIS-V1-TYPES-V216` | positive   | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`(String \| Never).type same? String.type`.                                                                                                           | Value`true`; Type `Bool`.                                                                                                      | `D-189` |
| `IRIS-V1-TYPES-V217` | positive   | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`(String & Object).type same? String.type`.                                                                                                          | Value`true`; Type `Bool`.                                                                                                      | `D-190` |
| `IRIS-V1-TYPES-V218` | positive   | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`(Object?).type same? Object.type`.                                                                                                                  | Value`true`; Type `Bool`.                                                                                                      | `D-191` |
| `IRIS-V1-TYPES-V219` | positive   | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`class Box<T> {} let item: Box<Nil> = Box<Nil>.new()`.                                                                                               | Construction succeeds; Type`Box<Nil>`; implicit `T` bound is `Object`.                                                       | `D-192` |
| `IRIS-V1-TYPES-V220` | negative   | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`let value: NonNil = nil`.                                                                                                                           | `TypeContractError` at binding boundary; no value is stored.                                                                     | `D-193` |
| `IRIS-V1-TYPES-V221` | positive   | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`((String \| Nil) & NonNil).type same? String.type`.                                                                                                  | Value`true`; Type `Bool`.                                                                                                      | `D-194` |
| `IRIS-V1-TYPES-V222` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`let text: String \| MutableString = load_text(); text.length(); text.append("x")`.                                                                   | `length()` Type-checks with return Type `Integer`; `append` is unavailable with diagnostic `UNION_MEMBER_NOT_COMMON`.      | `D-195` |
| `IRIS-V1-TYPES-V223` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`let value: A \| B = load(); value.m(); value.m(1)`, where `A#m()` and `B#m(x: Integer)`.                                                         | Both calls are rejected with`UNION_CALL_ARITY_MISMATCH`; accepted-arity intersection is empty.                                   | `D-196` |
| `IRIS-V1-TYPES-V224` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`contract A { fun m(x: String) -> String } contract B { fun m(x: Object) -> Integer } class X for A, B { impl fun m(x: Object) -> String { "x" } }`. | Static diagnostic`CONTRACT_REQUIREMENT_INCOMPATIBLE`; no overload is created.                                                    | `D-197` |
| `IRIS-V1-TYPES-V225` | positive   | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`fun accept(x: Object) -> String { "ok" }; let f: (String) -> Object = accept; f("x")`.                                                              | Value`"ok"`; Type `Object`; parameter contravariance and return covariance hold.                                               | `D-198` |
| `IRIS-V1-TYPES-V226` | negative   | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`class Box<T> {} let target: Box<Object> = Box<String>.new()`.                                                                                       | Static`TypeContractError`; invariant `Box<String>` is not assignable to `Box<Object>`.                                       | `D-199` |
| `IRIS-V1-TYPES-V227` | positive   | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`fun id<T>(x: T) -> T { x }; let result: String = id("iris")`.                                                                                       | Value`"iris"`; inferred Type argument `String`; result Type `String`.                                                        | `D-200` |
| `IRIS-V1-TYPES-V228` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`fun pair<T>(a: T, b: T) -> T { a }; pair("x", 1)`.                                                                                                  | Inferred`T` is normalized `String \| Integer`; if target requires `String`, static `TypeContractError`.                     | `D-201` |
| `IRIS-V1-TYPES-V229` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`fun make<T>() -> T { load_value() as T }; let user: User = make(); make()`.                                                                         | First call infers`User`; standalone call reports `GENERIC_INFERENCE_UNCONSTRAINED`.                                            | `D-202` |
| `IRIS-V1-TYPES-V230` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`fun choose<T,U>(x: T, y: U) -> U { y }; choose<String, _>("x", 1); choose<String>("x", 1)`.                                                         | First call returns`Integer(1)` with `U = Integer`; second reports `GENERIC_ARGUMENT_ARITY`.                                  | `D-203` |
| `IRIS-V1-TYPES-V231` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`class Box<T> { fun initialize(value: T) -> Nil {} } let b: Box<String> = Box<_>.new("x"); let bad: Box<_> = b`.                                     | Construction infers`Box<String>`; persistent annotation reports `GENERIC_PLACEHOLDER_FORBIDDEN`.                               | `D-204` |
| `IRIS-V1-TYPES-V232` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`class Box<T> {} Box.new()`.                                                                                                                         | Static diagnostic`GENERIC_ARGUMENT_ARITY`; bare `Box` is definition metadata, not `Box<Object>`.                             | `D-205` |
| `IRIS-V1-TYPES-V233` | positive   | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`class Box<T> {} Box<String>.type same? Box<String>.type; Box<String>.type same? Box<Integer>.type`.                                                 | Values`true`, then `false`; closed identities are interned by definition and normalized arguments.                             | `D-206` |
| `IRIS-V1-TYPES-V234` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Metadata fixture: materialize`Box<String>` and `Box<Integer>`; open `Box<T>` with a member violating one substituted constraint.                                     | `TypeContractError`; neither definition nor either closed revision changes.                                                      | `D-207` |
| `IRIS-V1-TYPES-V235` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`open class Box<String> { fun m() -> Nil {} }`.                                                                                                      | Static diagnostic`CLOSED_GENERIC_OPEN_FORBIDDEN`.                                                                                | `D-208` |
| `IRIS-V1-TYPES-V236` | positive   | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`class Cache<T> { class property value: T } Cache<String>.value = "s"; Cache<Integer>.value = 1`.                                                    | Reads are`"s"` and `Integer(1)` respectively; storage is independent per closed Class.                                         | `D-209` |
| `IRIS-V1-TYPES-V237` | negative   | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`class Cache<T> { shared class property bad: T }`.                                                                                                   | Static diagnostic`GENERIC_SHARED_PROPERTY_REFERENCES_TYPE_PARAMETER`.                                                            | `D-210` |
| `IRIS-V1-TYPES-V238` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`class Cache<T> { shared class property count: Integer = 0 } Cache.count; Cache<String>.count`.                                                      | `Cache.count` returns `Integer(0)`; closed access reports `MESSAGE_NOT_FOUND`.                                               | `D-211` |
| `IRIS-V1-TYPES-V239` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Metadata fixture: module declares`shared class property first: Integer = 1` then initializer `raise :stop`.                                                            | Module initialization fails; status`not_published`; neither shared property is observable.                                       | `D-212` |
| `IRIS-V1-TYPES-V240` | positive   | compiler required; interpreter required; JIT required; native not applicable | Metadata fixture:`Box<T>` per-closed initializer appends its closed Type to a log; request `Box<String>`, `Box<String>`, `Box<Integer>`.                           | Log is`[Box<String>, Box<Integer>]`; each closed initializer runs once.                                                          | `D-213` |
| `IRIS-V1-TYPES-V241` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Metadata fixture:`Box<T>` initializer records external `"attempt"` then raises during `Box<String>` materialization; request it twice.                               | Each request raises`TypeContractError`; no `Box<String>` intern entry exists; external log is `["attempt", "attempt"]`.      | `D-214` |
| `IRIS-V1-TYPES-V242` | negative   | compiler required; interpreter required; JIT required; native not applicable | Metadata fixture: reflection constructs`Box<Nil>` for `class Box<T> where T: NonNil {}`.                                                                               | `TypeContractError`; status `not_published`; no closed Type identity is interned.                                              | `D-215` |
| `IRIS-V1-TYPES-V243` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`class Pair<T,U> where U: T {} class Bad<T,U> where T: U, U: T {}`.                                                                                  | `Pair<Object,String>` is rejected for its bound; `Bad` reports `GENERIC_CONSTRAINT_CYCLE`.                                   | `D-216` |
| `IRIS-V1-TYPES-V244` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`contract Comparable<T> {} class Box<T> where T: Comparable<T> {} Box<String>.new()`.                                                                | `TypeContractError` unless `String` explicitly conforms to `Comparable<String>`; structural members do not satisfy it.       | `D-217` |
| `IRIS-V1-TYPES-V245` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`class Pair<T,U> {} let p: Pair<String> = Pair<String, Integer>.new()`.                                                                              | Static diagnostic`GENERIC_ARGUMENT_ARITY`; no default `U` is supplied.                                                         | `D-218` |
| `IRIS-V1-TYPES-V246` | positive   | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`module Helpers<T> {} mixin Helpers<String>`.                                                                                                        | Closed Module Type`Helpers<String>` is reified and interned.                                                                     | `D-219` |
| `IRIS-V1-TYPES-V247` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`module Helpers<T> where Self: T {} class Host { mixin Helpers<_> }`.                                                                                | Static diagnostic`GENERIC_MODULE_ARGUMENTS_EXPLICIT`; host inference is not used.                                                | `D-220` |
| `IRIS-V1-TYPES-V248` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`contract C { fun m() -> Nil } class A for C { fun m() -> Nil {} }`.                                                                                 | Static diagnostic`CONTRACT_IMPLEMENTATION_REQUIRES_IMPL`.                                                                        | `D-233` |
| `IRIS-V1-TYPES-V249` | positive   | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture: compatible`contract A` and `contract B` both require `m(Object) -> String`; `class X for A, B { impl fun m(x: Object) -> String { "x" } }`.   | One Method satisfies both requirements;`X.new() as A` and `X.new() as B` each qualify `m` to `"x"`.                        | `D-234`, `D-276` |
| `IRIS-V1-TYPES-V250` | positive   | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`contract C { fun m() -> String } class X for C { impl fun m() -> String { "c" } } let view = X.new() as C; view..m()`.                              | Value`"c"`; Type `String`; only explicit `..m()` selects the Contract slot.                                                  | `D-237` |
| `IRIS-V1-TYPES-V251` | negative   | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`let a = X.new() as C; a same? a`.                                                                                                                   | `IdentityError`; Contract views have no independent identity.                                                                    | `D-239` |
| `IRIS-V1-TYPES-V252` | positive   | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture: two`Integer(1) as NumericContract` views with the same Contract identity, compared with `==`.                                                     | Value`true`; Type `Bool`; equality uses value equality for identity-less receivers.                                            | `D-240` |
| `IRIS-V1-TYPES-V253` | positive   | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`fun f(value) { value }; f.method.parameters[0].type; f.method.return_type`.                                                                         | Both reflected Types are`Dynamic<Object>`; body-local inference is absent from signature metadata.                               | `D-452` |
| `IRIS-V1-TYPES-V254` | negative   | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`let value: Dynamic<String> = 1`.                                                                                                                    | `TypeError` at Dynamic entry; no arbitrary selector send begins.                                                                 | `D-453` |
| `IRIS-V1-TYPES-V255` | positive   | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`let value: Object = "iris"; [value is String, value as? Integer]`.                                                                                  | Value`[true, nil]`; Type `Array<Bool \| Nil>`; values are not converted.                                                        | `D-454` |
| `IRIS-V1-TYPES-V256` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`type Name<T> = Array<T>; Name<String>.type same? Array<String>.type; type Loop = Array<Loop>`.                                                      | First expression returns`true`; second declaration reports `RECURSIVE_TYPE_ALIAS`.                                             | `D-455` |
| `IRIS-V1-TYPES-V257` | positive   | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`class Box<T> {} let t = Box<String>.type; [t same? Box<String>.type, t same? Box<String>]`.                                                         | Values`[true, false]`; `t` is an interned Type object distinct from the Class object.                                          | `D-456` |
| `IRIS-V1-TYPES-V258` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`contract C { fun m() -> Nil { nil } }`.                                                                                                             | Static diagnostic`CONTRACT_METHOD_BODY_FORBIDDEN`; no Contract declaration is published.                                         | `D-275`, `D-457` |
| `IRIS-V1-TYPES-V259` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture:`fun fail() -> Never { nil }`.                                                                                                                       | Static`TypeContractError`; if reached through a Dynamic boundary, runtime raises `TypeError` before normal return is observed. | `D-458` |
| `IRIS-V1-TYPES-V260` | positive | compiler required; interpreter required; JIT required; native not applicable | Metadata fixture: construct named Contract Type `pkg@1::C` twice with different source paths and allocation order, then construct `pkg@2::C`. | The first two public hashes are equal; the API-major-changed Contract Type public hash is different. | `D-242` |
| `IRIS-V1-TYPES-V261` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Iris source fixture: `contract C { fun m() -> Nil } module M for C {} class A for C { impl fun m() -> Nil { nil } }`. | `module M for C` reports `CONTRACT_FOR_CLASS_ONLY`; `A` is accepted and nominally conforms to `C`. | `D-279` |
| `IRIS-V1-TYPES-V262` | positive | compiler required; interpreter required; JIT required; native not applicable | Metadata fixture: Contract view receiver public hash is `1`, Contract Type public hash is `2`, and the view is hashed. | Artifact uses BLAKE3 derive-key context `Iris Language v1 contract view hash`, input `01000000000000000200000000000000`, and public Integer from digest bytes `0..7` as unsigned little-endian. | `D-241` |
