# Iris v1 Runtime Object Model

Status: Iris v1.2, frozen semantics with owner-approved errata.

IRIS-V1-RUNTIME-C001: This chapter defines runtime values, objecthood, identity, dispatch, logical Class and active revision semantics, Module MRO, Method and BoundMethod identity, construction, properties, raw ivars, class variables, truthiness, missing-message handling, built-in numeric behavior, hashing, and built-in openness for Iris v1. It MUST be read after [README.md](README.md), [01-language-identity.md](01-language-identity.md), and [02-lexical-grammar.md](02-lexical-grammar.md).

IRIS-V1-RUNTIME-C002: This chapter MUST NOT define parser productions, gradual type algebra, package initialization, open-transaction scheduling, reflection APIs, serialization formats, FFI handles, async scheduling, or collection operation details beyond the runtime contracts needed by later chapters. Those later chapters MUST preserve the anchors in this chapter when refining their own surfaces.

## Runtime Value Foundation

IRIS-V1-RUNTIME-C003: Every Iris runtime value MUST be an object. Objecthood means the value has a runtime Class or value-Class relation, accepts message protocols according to this chapter, and participates in equality, hashing, truthiness, and exception raising as specified by its category.

IRIS-V1-RUNTIME-C004: Objecthood does not imply observable identity. A conforming implementation MUST classify each built-in value category as identity-bearing or identity-less according to IRIS-V1-RUNTIME-C006. `same?` MUST accept only identity-bearing operands and MUST raise `IdentityError` for identity-less operands.

IRIS-V1-RUNTIME-C005: `Object` is the root runtime Class for ordinary object behavior. `Object` provides default comparison, truthiness, missing-message, and zero-argument `initialize` behavior as specified below. No second root Class MAY exist.

IRIS-V1-RUNTIME-C006: The following table is normative for built-in identity, default hash, mutability, and openness classification:

| Built-in category     | Identity classification                                                                  | Default hash contract                                                                                                                           | Mutability of value state                                | Dynamic openness                                                   |
| --------------------- | ---------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------- | ------------------------------------------------------------------ |
|                       |                                                                                          |                                                                                                                                                 |                                                          |                                                                    |
| `nil`               | Identity-bearing singleton                                                               | Specification-stable singleton hash                                                                                                             | Singleton state is fixed                                 | Its Class may compose Modules and replace compatible behavior      |
| `false`, `true`   | Identity-bearing singletons                                                              | Specification-stable singleton hash                                                                                                             | Singleton state is fixed                                 | `Bool` Class may compose Modules and replace compatible behavior |
| `Integer` values    | Identity-less immutable values                                                           | Specification-stable numeric hash, except invalid NaN does not apply                                                                            | Mathematical value is fixed                              | `Integer` Class is behaviorally open within safety bounds        |
| `Float32` values    | Identity-less immutable values                                                           | Specification-stable numeric hash, except NaN raises`InvalidKeyError`                                                                         | IEEE-754 interchange bits are fixed                      | `Float32` Class is behaviorally open within safety bounds        |
| `Float64` values    | Identity-less immutable values                                                           | Specification-stable numeric hash, except NaN raises`InvalidKeyError`                                                                         | IEEE-754 interchange bits are fixed                      | `Float64` Class is behaviorally open within safety bounds        |
| Class objects         | Identity-bearing definition objects                                                      | Runtime-local identity hash unless a Method replaces it                                                                                         | Object identity persists across compatible opens         | Logical Class keeps one identity with active revisions             |
| Module objects        | Identity-bearing definition objects                                                      | Runtime-local identity hash unless a Method replaces it                                                                                         | Object identity persists across compatible opens         | Module keeps one identity with active revisions                    |
| Contract objects      | Identity-bearing definition objects                                                      | Runtime-local identity hash unless a Method replaces it                                                                                         | Static requirements are fixed after declaration          | Contracts cannot be opened                                         |
| Contract views        | Identity-less immutable capability values                                                | Derived from receiver hash and Contract Type identity                                                                                           | Receiver relation and Contract identity are fixed        | View dispatch is explicit through`..`                            |
| Method objects        | Identity-bearing callable definition objects                                             | Runtime-local identity hash unless a Method replaces it                                                                                         | Method identity and body are retained                    | Slot replacement creates a new Method                              |
| BoundMethod objects   | Identity-bearing callable objects                                                        | Runtime-local identity hash unless a Method replaces it                                                                                         | Captured receiver relation and Method identity are fixed | Invocation revalidates current owner membership                    |
| Closure objects       | Identity-bearing callable objects                                                        | Runtime-local identity hash unless a Method replaces it                                                                                         | Captured environment binding is fixed by Closure rules   | Each evaluation creates a distinct identity                        |
| ClassRevision objects | Identity-bearing runtime-owned metadata objects                                          | Runtime-local identity hash                                                                                                                     | Read-only metadata and code reference are fixed          | Superseded revisions may be pinned while referenced                |
| String                | Identity-less immutable text value                                                       | Stable text hash defined by the collections chapter                                                                                             | Scalar sequence is fixed                                 | Class behavior may be open within safety bounds                    |
| MutableString         | Identity-bearing mutable text object                                                     | Runtime-local identity hash unless a Method replaces it                                                                                         | Text buffer may mutate through Methods                   | Class behavior may be open within safety bounds                    |
| Symbol                | Identity-less immutable interned-name value                                              | Stable symbol hash defined by the collections chapter                                                                                           | Symbol text is fixed                                     | Class behavior may be open within safety bounds                    |
| Bytes                 | Identity-less immutable binary value                                                     | Stable bytes hash defined by the collections chapter                                                                                            | Byte sequence is fixed                                   | Class behavior may be open within safety bounds                    |
| ByteArray             | Identity-bearing mutable binary object                                                   | Runtime-local identity hash unless a Method replaces it                                                                                         | Byte sequence may mutate through Methods                 | Class behavior may be open within safety bounds                    |
| Tuple                 | Identity-less immutable ordered value                                                    | Stable element hash when all elements are hashable                                                                                              | Element sequence is fixed                                | Class behavior may be open within safety bounds                    |
| Array                 | Identity-bearing mutable ordered object                                                  | Runtime-local identity hash unless a Method replaces it                                                                                         | Elements may mutate through Methods                      | Class behavior may be open within safety bounds                    |
| Hash                  | Identity-bearing mutable mapping object                                                  | Runtime-local identity hash unless a Method replaces it                                                                                         | Entries may mutate through Methods                       | Class behavior may be open within safety bounds                    |
| Range                 | Identity-less immutable interval value                                                   | Stable endpoint hash when endpoints are hashable                                                                                                | Endpoints and inclusivity are fixed                      | Class behavior may be open within safety bounds                    |
| Regex                 | Identity-less immutable pattern value                                                    | Stable pattern hash defined by the collections chapter                                                                                          | Pattern and flags are fixed                              | Class behavior may be open within safety bounds                    |
| Type objects          | Interned identity-bearing Type objects distinct from Class, Module, and Contract objects | Stable for publishable named and composite Types; runtime-local for local anonymous identities, with finer derivation owned by the type chapter | Canonical Type identity is fixed                         | Reflection visibility follows later chapters                       |

IRIS-V1-RUNTIME-C007: Identity-less immutable values MUST NOT gain observable receiver-specific state. They MUST NOT create, recreate, or retain undeclared dynamic ivars, and assignment that would create such state MUST raise `InstanceStateError`.

IRIS-V1-RUNTIME-C008: Identity-bearing ordinary objects MAY carry receiver-specific state according to their Class policy, declared storage, and effective `MetaCapabilities`. Garbage collection, object movement, allocation strategy, interning, unboxing, or reboxing MUST NOT change any observable identity or identity-less classification.

## Logical Classes And Active Revisions

IRIS-V1-RUNTIME-C009: A logical Class is the stable identity visible to Iris programs. A logical Class has exactly one current active revision at any observation point, and ordinary instance sends, Contract-qualified lookup, MRO lookup, and runtime ancestry checks MUST consult that current active revision unless this chapter says an entered frame or retained callable keeps a captured Method body.

IRIS-V1-RUNTIME-C010: Reopening an existing Class MUST mutate the same logical Class identity by publishing a new active revision after validation. References captured before and after the reopen MUST satisfy `same?` because the logical Class object is unchanged. Rebinding a name to a distinct Class object is not reopen and is not a v1 operation for named Class declarations.

IRIS-V1-RUNTIME-C011: Every instance references its stable logical Class. An instance MUST NOT permanently select an old active revision for ordinary future sends. After a successful structural commit, existing and future instances use the logical Class's newly active revision for subsequent ordinary sends and ancestry checks.

IRIS-V1-RUNTIME-C012: Entered Method frames execute the Method identity and body selected at call entry. Replacing or removing the slot after entry MUST NOT alter the body of an already-running frame. Future ordinary lookup uses the new active revision and selected slot state.

IRIS-V1-RUNTIME-C013: Method replacement MUST create a new identity-bearing Method object and install it in the slot. Previously captured Method objects retain their identity and old body while referenced and may remain explicitly invocable through reflection subject to receiver binding validation.

IRIS-V1-RUNTIME-C014: A Method object MUST retain the identity of its lexical owner Class or Module. When a retained old Method executes `super`, lookup MUST use the receiver's current versioned MRO at invocation time and continue after that lexical owner. If the lexical owner is absent from the current MRO, execution MUST raise `InvalidSuperError`.

IRIS-V1-RUNTIME-C015: Reflective Method invocation MUST validate at entry that the receiver's current MRO contains the Method's lexical owner. A Class owner requires an instance whose current MRO includes that Class. A Module owner requires that Module in the receiver MRO. Failure MUST raise `MethodBindingError` even if the body path would not access state or execute `super`.

IRIS-V1-RUNTIME-C016: A `ClassRevision` is a read-only identity-bearing runtime metadata object. It MUST expose at least owner Class, per-Class revision number, static-spine reference, runtime superclass, MRO, Modules, member and property metadata, layout or shape descriptor, package or source revision metadata, and commit metadata through later reflection rules. User code MUST NOT mutate or reactivate a `ClassRevision`.

IRIS-V1-RUNTIME-C017: The origin Class revision number is `1`. Each successful structural publication affecting that logical Class MUST receive the next per-Class integer. Failed or rolled-back candidates MUST NOT consume a per-Class revision number.

IRIS-V1-RUNTIME-C018: Each successful structural transaction group MUST receive one runtime-wide strictly increasing `commit_id` shared by all Class and Module revisions in that group. `commit_id` is runtime-local diagnostic and audit ordering metadata. It MUST NOT contribute to nominal Type identity or cross-runtime stable hashes.

IRIS-V1-RUNTIME-C019: Superseded revisions pinned by Method frames, retained Method objects, native references, tools, or user-held `ClassRevision` references MUST retain the executable metadata needed by those references. Once unreferenced and inactive, heavy revision state MAY be reclaimed, while a lightweight audit record MAY remain.

IRIS-V1-RUNTIME-C020: A rollback operation MUST publish a new validated revision instead of reactivating a historical revision directly. Rollback MUST validate the current static spine, Contract requirements, MRO, visibility, native obligations, and safety constraints before publication. Missing or mismatched historical artifacts MUST raise `RevisionArtifactUnavailableError` and publish nothing.

IRIS-V1-RUNTIME-C021: Same-major package or Class evolution MUST NOT downgrade the current static spine. Added compatible ordinary API MAY remain, but removing required current members, Contracts, generic arity, visibility obligations, native obligations, or superclass bounds MUST fail validation.

IRIS-V1-RUNTIME-C022: The following revision transition table is normative:

| Event                                | Global instance sends during event                       | New sends after success                  | Entered frames                                | Retained Method or BoundMethod                          |
| ------------------------------------ | -------------------------------------------------------- | ---------------------------------------- | --------------------------------------------- | ------------------------------------------------------- |
| Open candidate starts                | Use published active revision                            | Unchanged until commit                   | Continue selected body                        | Continue captured Method identity                       |
| Candidate body reads metadata        | Transaction reads candidate through meta/reflection only | Unchanged until commit                   | Continue selected body                        | Continue captured Method identity                       |
| Candidate fails validation or raises | Published active revision remains                        | Published active revision remains        | Continue selected body                        | Continue captured Method identity                       |
| Candidate commits                    | No partial candidate dispatch is exposed                 | Use new active revision                  | Continue old selected body if already entered | Invoke captured Method after current-owner revalidation |
| Slot replacement commits             | Old slot remains for entered frames                      | Future lookup selects new Method         | Continue old body                             | Captured old Method remains explicitly invocable        |
| Module MRO changes                   | Current active MRO until commit                          | Future lookup uses new MRO               | Current frame keeps selected body             | `super` uses invocation-time current MRO              |
| Rollback commits                     | No historical revision is reactivated in place           | Future lookup uses new rollback revision | Continue selected body                        | Captured Method rules still apply                       |

IRIS-V1-RUNTIME-N001: Implementation note: Inline caches, primitive numeric paths, and compiled code may remember revisions, Method identities, lookup versions, and representation assumptions only as guarded dependencies. A changed dependency must invalidate, fall back, or deoptimize before a later affected send observes stale behavior.

## Dispatch And Selector Namespaces

IRIS-V1-RUNTIME-C023: Ordinary message identity is the complete selector name plus its ordinary call shape. Static annotations, inferred types, union branches, generic arguments, expected return types, or declaration order MUST NOT choose a different ordinary selector or overload.

IRIS-V1-RUNTIME-C024: Iris v1 MUST NOT support Method overloading. One complete selector in one Class revision or MRO slot namespace maps to at most one Method. Redefining the same complete selector is replacement and requires the applicable `override` authorization; it is never an overload set.

IRIS-V1-RUNTIME-C025: Ordinary selector lookup MUST evaluate receiver and arguments according to the grammar and control-flow chapters, resolve the receiver's current active revision and MRO, apply visibility and arity/type checks, and invoke the selected Method. Existing selector mismatches MUST raise `ArgumentError` for arity mismatch or `TypeError` for runtime argument or block Contract failure. They MUST NOT invoke `method_missing`.

IRIS-V1-RUNTIME-C026: A named infix call `receiver selector argument` MUST be the same observable ordinary send as `receiver.selector(argument)` for one-argument Methods. Parsing never consults Method tables. A Method declaration MUST NOT change named infix precedence, associativity, or parse validity.

IRIS-V1-RUNTIME-C027: Symbolic operators that are overloadable in the grammar MUST be ordinary Method sends. For `a OP b`, the runtime MUST evaluate `a`, then evaluate `b`, then resolve and invoke selector `OP` on the runtime Class and active revision of `a` with `b` as the argument. The built-in operator Method contract, not a global promotion table, determines accepted operands and result type.

IRIS-V1-RUNTIME-C028: `!`, `&&`, `||`, `&&=`, and `||=` MUST be non-overloadable core control-flow forms that use `to_bool` as specified in IRIS-V1-RUNTIME-C093 through IRIS-V1-RUNTIME-C098. They MUST NOT be installed as Class or Module Method selectors. `&`, `|`, `^`, and `~` remain overloadable operator messages.

IRIS-V1-RUNTIME-C029: `same?` is a non-overloadable primitive identity comparison. It MUST evaluate the left operand once, evaluate the right operand once, accept only identity-bearing operands, bypass Method lookup, `<=>`, `==`, proxies, and user replacement, and return `Bool`. If either operand is identity-less, it MUST raise `IdentityError`.

IRIS-V1-RUNTIME-C030: Contract-qualified dispatch has a distinct selector namespace from ordinary dispatch. The source form `(value as ContractType)..member(args...)` or a stored Contract view followed by `..member(args...)` MUST choose the slot identity containing the Contract identity plus member selector. Ordinary `value.member(args...)` and `view.member(args...)` MUST remain unqualified ordinary sends.

IRIS-V1-RUNTIME-C031: A Contract view is an immutable identity-less capability value carrying an underlying receiver relation and Contract identity or conformance. Repeated view creation MAY allocate nothing. `same?` on a Contract view MUST raise `IdentityError`.

IRIS-V1-RUNTIME-C032: Built-in Contract view equality over identity-bearing receivers MUST require the same receiver identity and same Contract identity. Built-in Contract view equality over identity-less receivers MUST require receiver equality under current equality behavior and same Contract identity.

IRIS-V1-RUNTIME-C033: Missing ordinary selectors MUST invoke `method_missing` as specified in IRIS-V1-RUNTIME-C099 and IRIS-V1-RUNTIME-C100. Missing qualified Contract slots MUST raise `ContractDispatchError` and MUST NOT invoke ordinary `method_missing`, fall back to unqualified lookup, search another Contract, or use a dynamic Contract-missing hook.

IRIS-V1-RUNTIME-C034: Visibility denial MUST bypass `method_missing`. If lookup finds a Method or property selector but the caller lacks private or protected access, dispatch MUST raise `MethodVisibilityError` directly.

IRIS-V1-RUNTIME-C035: The following dispatch namespace table is normative:

| Source form                       | Selector namespace                                  | Dynamic owner used                                     | Missing behavior                             | Overload allowed |
| --------------------------------- | --------------------------------------------------- | ------------------------------------------------------ | -------------------------------------------- | ---------------- |
| `value.member(args...)`         | Ordinary selector`member`                         | Receiver logical Class current active revision and MRO | `method_missing(:member, args, block)`     | No               |
| `value selector arg`            | Ordinary selector`selector`                       | Receiver logical Class current active revision and MRO | `method_missing(:selector, [arg], nil)`    | No               |
| `value + arg`                   | Ordinary selector`+`                              | Receiver logical Class current active revision and MRO | `method_missing(:+, [arg], nil)`           | No               |
| `value.name`                    | Ordinary property getter selector`name`           | Receiver logical Class current active revision and MRO | `method_missing(:name, [], nil)`           | No               |
| `value.name = rhs`              | Ordinary property setter selector`name=`          | Receiver logical Class current active revision and MRO | `method_missing(:name=, [rhs], nil)`       | No               |
| `(value as C)..member(args...)` | Qualified Contract slot`C::member`                | Receiver Class implementation of that Contract slot    | `ContractDispatchError`                    | No               |
| `view..member(args...)`         | Qualified Contract slot from view Contract identity | Receiver Class implementation of that Contract slot    | `ContractDispatchError`                    | No               |
| `view.member(args...)`          | Ordinary selector`member`                         | Underlying receiver ordinary dispatch                  | `method_missing(:member, args, block)`     | No               |
| `same?` or `.same?`           | Primitive identity comparison                       | None                                                   | `IdentityError` for identity-less operands | No               |

## Methods, BoundMethods, Closures, And Class Objects

IRIS-V1-RUNTIME-C036: A Method is an identity-bearing callable definition object installed in a Class, Module, Contract-qualified slot implementation, class-object singleton Method table, or related receiver surface. Method default equality MUST be identity-only through root behavior.

IRIS-V1-RUNTIME-C037: Method aliasing MUST create another slot referencing the same Method identity. Removing a slot MUST remove only the current owner local slot and may expose an ancestor implementation. Undefining a slot MUST install a tombstone that blocks ancestor lookup; an ordinary call then counts as absent and may invoke `method_missing`.

IRIS-V1-RUNTIME-C038: Reading or binding an instance Method MUST create a BoundMethod that captures the invocation receiver relation and exact Method identity resolved at binding time. It MUST NOT capture a historical MRO snapshot. Later slot replacement MUST NOT redirect the BoundMethod.

IRIS-V1-RUNTIME-C039: Each BoundMethod invocation MUST revalidate that the receiver's current Class and MRO contain the captured Method's lexical owner. If absent, invocation MUST raise `MethodBindingError`. If the body executes `super`, `super` MUST use the invocation-time current MRO under IRIS-V1-RUNTIME-C014.

IRIS-V1-RUNTIME-C040: Every evaluation that reads or binds `obj.method` MUST create a distinct identity-bearing BoundMethod, even when receiver and resolved Method are unchanged. `obj.method same? obj.method` is therefore `false` because the operands are two evaluated bindings. A saved BoundMethod reference is `same?` itself.

IRIS-V1-RUNTIME-C041: BoundMethod default equality MUST be identity-only. It MUST NOT compare captured receiver and Method components structurally by default. Reflection MAY expose those components for explicit comparison under later reflection rules.

IRIS-V1-RUNTIME-C042: Each evaluation of a Closure expression MUST create a new identity-bearing Closure object with its own captured environment. Closure default equality MUST be identity-only. Iris MUST NOT structurally compare executable code or captured environments.

IRIS-V1-RUNTIME-C043: A Class object is itself an identity-bearing object and an instance of `Class`. A `class fun` declaration MUST install a singleton Method on that specific Class object.

IRIS-V1-RUNTIME-C044: Sending a message to a Class object MUST search that object's singleton Methods first, then corresponding logical runtime-superclass Class objects in order, then ordinary instance Methods supplied by `Class`. `super(...)` inside a Class-object singleton Method MUST continue along that Class-object chain. V1 MUST NOT expose separate metaclass declaration syntax.

IRIS-V1-RUNTIME-C045: Class-object raw `@x` state belongs to the Class object receiver itself. A subclass Class object has its own Class-object `@x` state. This storage is distinct from hierarchy class variables named with `@@`.

## Module Composition And MRO

IRIS-V1-RUNTIME-C046: A Class has single Class inheritance plus Module composition. Lookup MUST start with the logical Class's own active members, then composed Modules in reverse composition order, then recursively the runtime superclass MRO.

IRIS-V1-RUNTIME-C047: Header `mixin A, B` MUST be applied left-to-right and produce lookup order `Class, B, A, Super...` before nested expansion. Sequential dynamic `include(A); include(B)` in one committed transaction MUST produce the same relative order if both edges are present.

IRIS-V1-RUNTIME-C048: Nested Module composition MUST expand deterministically and deduplicate by closed Module identity at the closest-to-Class occurrence. Re-including an already-present closed Module MUST be idempotent: it creates no duplicate, does not move or reorder the existing occurrence, and causes no structural revision unless other edge metadata changes.

IRIS-V1-RUNTIME-C049: V1 provides no dedicated Module reorder API. A program MAY reorder composition only by removing and including Modules inside one open transaction. Candidate validation, atomic commit, the `modules` capability, Module `Self` constraints, MRO validity, Contract implementation, and composition-edge authorization MUST all reapply.

IRIS-V1-RUNTIME-C050: Module composition grants no private Method access by default. Private access from Module Methods to host Class private Methods requires explicit authorization at the static or dynamic composition edge and is scoped to that host logical Class, closed Module identity, and edge revision.

IRIS-V1-RUNTIME-C051: A composed Module Method executing on a host receiver MAY read and write unqualified raw ivars on the current receiver according to raw ivar rules, whether or not the composition edge grants private Method access. Private authorization affects private selectors only.

IRIS-V1-RUNTIME-C052: Removing a Module edge MUST atomically revoke that edge's private authorization and recompute MRO, effective `MetaCapabilities`, Contract satisfaction, and lookup versions. Module global state MUST NOT retain host private access.

IRIS-V1-RUNTIME-C053: The following MRO table is normative:

| Composition state                                | Resulting lookup order before superclass recursion   | Revision effect                                        |
| ------------------------------------------------ | ---------------------------------------------------- | ------------------------------------------------------ |
| `class C extends S {}`                         | `C, S...`                                          | Origin revision contains no Module edges               |
| `class C extends S mixin A, B {}`              | `C, B, A, S...`                                    | Static mixin edges are in origin revision              |
| `include(A); include(B)` from no prior Modules | `C, B, A, S...`                                    | One committed dynamic revision if transaction succeeds |
| `B mixin A`; `C mixin A, B`                  | `C, B, A, S...`                                    | Nested`A` is deduplicated at closest occurrence      |
| Re-include already-present`A`                  | Existing order unchanged                             | No structural revision unless edge metadata changes    |
| Remove`A`, then include `A private`          | `A` appears at new edge position with new metadata | New revision if transaction succeeds                   |

## Construction Lifecycle

IRIS-V1-RUNTIME-C054: Standard construction is the Class-object Method `new`. `A.new(args...)` MUST allocate a complete memory-safe instance, execute stored property initializers as specified in IRIS-V1-RUNTIME-C057, invoke the final dynamic `initialize(args...)`, validate or ignore its `Nil` result according to the callable/type chapters, and return the instance on success.

IRIS-V1-RUNTIME-C055: `Object` MUST provide a default zero-argument `initialize`. `initialize` is private by default. Iris v1 MUST NOT provide Class-named constructor syntax or constructor overloads.

IRIS-V1-RUNTIME-C056: `A.new(args...)` MUST snapshot `A`'s active revision at construction start for allocation, layout, stored property initialization, and initial `initialize` dispatch. If a new active revision commits before construction completes, construction continues with the captured construction revision. On success, the instance references logical Class `A`, and later ordinary sends use `A`'s then-current active revision.

IRIS-V1-RUNTIME-C057: Stored property initializers MUST execute superclass-to-subclass and declaration order before the explicit `initialize` Method. Parent `initialize` logic runs only through explicit `super(...)`; the runtime MUST NOT automatically call every ancestor initializer.

IRIS-V1-RUNTIME-C058: If `initialize` or a stored property initializer raises, `new` MUST propagate the exception and return no instance. Any allocated object remains memory-safe and ordinary if `self` escaped. The runtime MUST NOT poison, revoke, scan, auto-close, run special finalization, undo state, or roll back external effects.

IRIS-V1-RUNTIME-C059: Iris v1 provides no automatic construction and revision reconciliation. It MUST NOT provide `new_current`, `new_checked`, constructor retry, revision lock, automatic post-construction reinitialization, automatic instance state migration, or hidden constructor side-effect compensation.

IRIS-V1-RUNTIME-C060: The conventional `migrate_revision(from: ClassRevision, to: ClassRevision) -> Nil` Method MAY be implemented and explicitly invoked by applications on objects they track. Runtime open or commit MUST NOT enumerate live instances, invoke this Method automatically, or provide Class-wide live-instance enumeration.

## Properties, Raw Ivars, And Class Variables

IRIS-V1-RUNTIME-C061: Properties are explicit Method selectors. A getter declaration `property fun name() -> T` defines selector `name`. A setter declaration `property fun name=(value: T) -> R` defines selector `name=`. Property read `obj.name` MUST send the zero-argument getter selector, and assignment `obj.name = value` MUST send the one-argument setter selector.

IRIS-V1-RUNTIME-C062: A property setter MAY return any value allowed by its declared and runtime contract. `obj.name = value` MUST yield the setter Method result, not necessarily the right-hand value or `nil`.

IRIS-V1-RUNTIME-C063: Assignment MUST be right-associative. In `a.x = b.y = v`, the inner setter executes first and the outer setter receives the inner setter's actual result. Static and dynamic checks MUST use that actual expression result.

IRIS-V1-RUNTIME-C064: Binding, class-variable, shared, and raw-ivar assignment MUST yield the actual stored value. Property and index assignment MUST yield the setter Method result. Compound assignment MUST evaluate target location and RHS once, read the target once, send the ordinary operator Method, write once through the target's normal write path, and yield that write operation's result.

IRIS-V1-RUNTIME-C065: Stored property shorthand MUST create revision-level typed instance storage plus explicit accessor Method selectors. Generated accessor bodies MAY be compatibly replaced by `property fun` declarations through authorized mutation. Contract exposure is through property Methods, never raw storage.

IRIS-V1-RUNTIME-C066: Raw `@x` syntax MUST access the single slot named `@x` on the current receiver. Slot identity is `(receiver, name)`, not declaring Class or Module. Parent, subclass, and composed Module Methods executing on the same receiver see and write the same slot.

IRIS-V1-RUNTIME-C067: Source syntax MUST NOT allow `other.@x` or `obj.@@x` or `A.@@x`. Cross-instance and external access to implementation state MUST use properties, Methods, or permission-checked reflection defined by later chapters.

IRIS-V1-RUNTIME-C068: Reading an undeclared dynamic raw ivar MUST have static type `Dynamic<Object>`. If the receiver lacks the slot, the read MUST return `nil` and MUST NOT create instance state.

IRIS-V1-RUNTIME-C069: When effective `instance_state` is present, assignment to an absent undeclared raw ivar on an identity-bearing object MAY create the slot. When effective `instance_state` is denied, first assignment to an absent undeclared raw ivar, including recreation after deletion, MUST raise `InstanceStateError`. Existing dynamic ivars remain readable and writable unless another rule forbids the write.

IRIS-V1-RUNTIME-C070: Denied `instance_state` MUST still permit deletion of an existing undeclared dynamic ivar for cleanup. Deletion MUST NOT delete other instances' state and MUST NOT alter Class-wide shape. Later assignment to the absent name is a new expansion and MUST obey IRIS-V1-RUNTIME-C069.

IRIS-V1-RUNTIME-C071: Undeclared dynamic ivars MUST have no fixed value-type contract. An existing undeclared dynamic ivar MAY hold any `Object` value, including `nil`, and later writes MAY change runtime value type. Programs that need stable types MUST use declared typed properties or storage.

IRIS-V1-RUNTIME-C072: A Closure created in an instance Method MUST capture its current receiver and MAY continue reading and writing that receiver's raw ivars after escape. The captured receiver and private capability MUST NOT be rebound through source, reflection, Dynamic, native, or Host APIs.

IRIS-V1-RUNTIME-C073: `@@name` denotes a declared hierarchy binding cell anchored to a logical Class. It is not an ivar or property of the Class object. The declaring Class and its subclass hierarchy share that cell according to lexical Class-variable rules.

IRIS-V1-RUNTIME-C074: Class variables MUST be lexically accessible from instance Methods and Class Methods owned by the declaring Class or its subclasses. Lookup follows the Method's immutable static lexical Class hierarchy and MUST NOT change with runtime-superclass revision changes. Public access MUST use Methods or properties.

IRIS-V1-RUNTIME-C075: Class-object raw ivars and hierarchy class variables are distinct storage categories. A Class object `A` MAY have ordinary receiver-name `@x` state independent from subclass Class object `B`'s `@x`, while `@@y` anchored to `A` is shared through the hierarchy and cannot be shadowed or redeclared by `B`.

IRIS-V1-RUNTIME-C076: The following storage table is normative:

| Syntax or declaration      | Storage owner                                               | Visibility surface                              | Creation authority                                                           | Assignment result                     |
| -------------------------- | ----------------------------------------------------------- | ----------------------------------------------- | ---------------------------------------------------------------------------- | ------------------------------------- |
| `@x`                     | Current receiver slot named`@x`                           | Current-receiver implementation code only       | `instance_state` for undeclared expansion, or declared storage rules       | Stored value                          |
| `property fun x()`       | Method selector`x`                                        | Message/property API                            | `property_set` for slot, `property_body` for compatible body replacement | Getter result                         |
| `property fun x=(v)`     | Method selector`x=`                                       | Message/property API                            | `property_set` for slot, `property_body` for compatible body replacement | Setter result                         |
| Stored`property x: T`    | Revision-level typed instance storage plus accessors        | Accessor Methods only                           | `property_set` plus `shape`; native storage also requires `native`     | Setter result for property assignment |
| Class-object`@x`         | The Class object receiver                                   | Class-object implementation code                | Object instance-state policy for that Class object                           | Stored value                          |
| `@@x`                    | Declared hierarchy binding cell anchored to a logical Class | Lexically authorized Class and subclass Methods | Declared storage rules and class-state capabilities                          | Stored value                          |
| Shared/class property slot | Class/shared storage metadata                               | Methods or meta APIs                            | `property_set & class_state_set`; native storage also requires `native`  | Setter or meta write result           |

## Visibility And Super

IRIS-V1-RUNTIME-C077: Class instance Methods, Class-object singleton Methods, Module Methods, properties, and module-main Methods default to private unless declared otherwise. Visibility prefixes are declaration-local and MUST NOT create stateful visibility sections.

IRIS-V1-RUNTIME-C078: A private Method selector MAY be sent only from implementation code lexically authorized by the declaring logical Class and any separately authorized Modules. Subclass code, external callers, `Dynamic<T>`, and ordinary reflection MUST NOT invoke a private Method merely because the selector exists.

IRIS-V1-RUNTIME-C079: A protected Method declared by Class `A` MAY be called from implementation code lexically belonging to `A` or a nominal subclass, with receiver restricted to current `self` or an instance in that permitted hierarchy. External callers and `Dynamic<T>` MUST NOT bypass protected visibility.

IRIS-V1-RUNTIME-C080: Protected properties MUST follow protected Method visibility. Raw ivars remain current-receiver slots and are not inherited protected fields.

IRIS-V1-RUNTIME-C081: `super(args..., key..., &block)` MUST call the same complete selector after the current Method lexical owner in the receiver's current MRO or Class-object singleton Method chain. Bare `super` and implicit argument forwarding MUST be illegal. If no successor exists, dispatch MUST raise `NoSuperMethodError` and MUST NOT invoke `method_missing`.

IRIS-V1-RUNTIME-C082: Qualified Contract Method `super` MUST continue within the explicit qualified slot and current MRO rules for that Contract slot. It MUST NOT fall back to ordinary selector lookup.

## Equality, Ordering, Hashing, And Identity

IRIS-V1-RUNTIME-C083: Root `Object` MUST provide `<=> (other: Object) -> Integer?`, with default implementation returning `nil` for every operand. No global address order or object-ID order MAY be exposed by default.

IRIS-V1-RUNTIME-C084: Default comparison Methods `==`, `!=`, `<`, `<=`, `>`, and `>=` MUST be ordinary one-argument Method slots. Their unmodified default implementations derive from the current visible `<=>` response. `<=>` result `Integer(-1)`, `Integer(0)`, `Integer(1)`, and `nil` map to comparison results as follows: zero means equality, negative one means less-than, positive one means greater-than, and `nil` means unordered with `== false`, `!= true`, and all four ordered comparisons false.

IRIS-V1-RUNTIME-C085: Default-delegating comparison Methods MUST accept from `<=>` only exact `Integer(-1)`, `Integer(0)`, `Integer(1)`, or `nil`. Any other integer or type MUST raise `ComparisonContractError` or `TypeError` according to whether it violates the dynamic comparison protocol or the declared return type.

IRIS-V1-RUNTIME-C086: For identity-bearing objects whose `==` slot still uses the root/default implementation, equality MUST first test whether both references denote the same object. If so, it returns `true` without consulting `<=>`. Otherwise it invokes the current visible `<=>` and returns `true` only for exact `Integer(0)`. Default `!=` is complementary while its own default slot remains installed.

IRIS-V1-RUNTIME-C087: Users MAY replace `<=>` or any comparison slot independently through authorized dynamic mutation. Replacing `<=>` affects still-default-delegating comparison Methods, but MUST NOT override a separately replaced comparison slot. Users assume responsibility for resulting equality, ordering, and hash consistency.

IRIS-V1-RUNTIME-C088: Ordinary identity-bearing objects and identity-bearing runtime or meta objects MAY be Hash keys and use a runtime-stable identity hash in `0..2^64-1`. The same object MUST retain that hash throughout its lifetime and across GC movement. The identity hash MUST NOT be stable across processes or expose a raw memory address.

IRIS-V1-RUNTIME-C089: `nil`, `false`, and `true` are valid Hash keys and MUST override generic identity hashing with specification-stable singleton hashes. Numeric values MUST use specification-stable numeric hashes. Hash tables MAY secretly remix public hashes with runtime/container seeds, but public `hash` results MUST remain as specified while the built-in Method remains selected.

IRIS-V1-RUNTIME-C090: Contract-view public hash MUST use BLAKE3 derive-key mode with exact ASCII context `Iris Language v1 contract view hash` and input `receiver_public_hash_u64_le || contract_type_hash_u64_le`. Digest reduction MUST use the first eight digest bytes as unsigned little-endian. Receiver hash failure MUST propagate.

IRIS-V1-RUNTIME-C091: `Bool#<=>(other: Object) -> Integer?` MUST define total Bool order: equal singleton values return `Integer(0)`, `false <=> true` returns `Integer(-1)`, `true <=> false` returns `Integer(1)`, and a non-Bool operand returns `nil`. Bool MUST remain distinct from numeric types and MUST NOT equal or order with `Integer(0)` or `Integer(1)`.

IRIS-V1-RUNTIME-C092: `nil#<=>(other: Object) -> Integer?` MUST return `Integer(0)` only when `other` is the same `nil` singleton, and `nil` for every non-nil value. `nil` MUST NOT be a global minimum or maximum.

## Truthiness And Missing Messages

IRIS-V1-RUNTIME-C093: Truthiness is the dynamic `to_bool() -> Bool` protocol. Conditions, logical negation, logical operators, and logical assignment MUST evaluate the operand once, send `to_bool` once, require an actual `Bool`, and MUST NOT recursively convert the result.

IRIS-V1-RUNTIME-C094: Root `Object` initially provides `to_bool() -> Bool` returning `true`. `Nil` returns `false`. `Bool` returns itself, so `false.to_bool()` is `false` and `true.to_bool()` is `true`.

IRIS-V1-RUNTIME-C095: If `to_bool` raises, the exception MUST propagate unchanged. If it returns a non-Bool normally, the runtime MUST raise `TypeContractError`. Conditional bodies, RHS expressions, and logical-assignment writeback that depend on the truth test MUST NOT run after a failed truth test.

IRIS-V1-RUNTIME-C096: If `to_bool` is absent after permitted Method removal, truth testing MUST invoke `method_missing(:to_bool, [], nil)` once. Its result MUST be `Bool` or raise `TypeContractError`. Missing or recursive fallback MUST terminate with ordinary message-missing failure and MUST NOT recursively truth-test fallback results.

IRIS-V1-RUNTIME-C097: `!x` MUST send `x.to_bool()` once and return the Bool negation. `a && b` MUST test `a`; when false it returns original `a` without evaluating `b`, and when true it evaluates and returns `b`. `a || b` MUST test `a`; when true it returns original `a`, and when false it evaluates and returns `b`.

IRIS-V1-RUNTIME-C098: `target &&= rhs` and `target ||= rhs` are control-flow assignment forms, not Method selectors. The target location MUST be evaluated and read once. `&&=` MUST evaluate and write RHS only when the current value is truthy. `||=` MUST evaluate and write RHS only when the current value is falsy. A no-write path yields the current value; a write path yields the normal write-back result.

IRIS-V1-RUNTIME-C099: Root `Object` MUST declare ordinary dynamically replaceable `method_missing(selector: Symbol, arguments: Array<Object>, block: Closure?) -> Object`. Runtime MUST invoke it after ordinary selector lookup genuinely fails, passing the exact selector Symbol, an immutable or snapshot positional argument Array, and the trailing block or `nil`.

IRIS-V1-RUNTIME-C100: The default `method_missing` implementation MUST raise `MessageNotFoundError` with receiver Class or active revision, selector, arity, source location, and visibility context. Runtime MUST NOT recursively re-enter `method_missing` for a missing `method_missing` lookup itself.

## Built-In Numeric Model

IRIS-V1-RUNTIME-C101: `Integer` is semantically arbitrary precision. Hidden immediate or fixed-width representations and BigInt promotion MUST NOT be observable as different Iris types, identity, dispatch behavior, equality behavior, or hash behavior. Integer arithmetic MUST have no fixed-width overflow or wrapping semantics.

IRIS-V1-RUNTIME-C102: `Float32` and `Float64` are distinct built-in language types with IEEE-754 binary32 and binary64 interchange semantics. Unsuffixed source float literals produce `Float64`; `f32` and `f64` suffixes select the corresponding width under the grammar chapter.

IRIS-V1-RUNTIME-C103: Built-in `Integer`, `Float32`, and `Float64` arithmetic Methods MAY accept operands from the other built-in numeric Classes. Each receiver Class's Method contract defines conversion, precision, exceptional values, and result type. No global numeric promotion rule, symmetric dispatch rule, or user-defined numeric coercion table exists.

IRIS-V1-RUNTIME-C104: For built-in arithmetic where the receiver is `Integer` and the argument is `Float32` or `Float64`, the receiver MUST convert to the argument's floating width using IEEE-754 rounding, execute at that width, and return that same floating type. Precision loss is observable.

IRIS-V1-RUNTIME-C105: For built-in arithmetic where the receiver is `Float32` or `Float64` and the argument is `Integer`, the argument MUST convert to the receiver's width using IEEE-754 rounding, execute at that width, and return the receiver's floating type. Precision loss is observable.

IRIS-V1-RUNTIME-C106: Built-in arithmetic between `Float32` and `Float64` MUST widen the `Float32` operand to `Float64`, execute at `Float64`, and return `Float64`, independent of operand order. Same-width float arithmetic MUST return the same width.

IRIS-V1-RUNTIME-C107: Every primitive built-in `Float32` and `Float64` `+`, `-`, `*`, and `/` step MUST round to the established result width using IEEE-754 `roundTiesToEven`. Iris exposes no mutable global or thread-local rounding mode that changes these operators.

IRIS-V1-RUNTIME-C108: Source-level separate multiply and add operations MUST round independently according to the expression tree. Implementations MUST NOT contract `a * b + c` into fused multiply-add if doing so could change observable bits or special-value behavior. The explicit `mul_add` Method is the only standard fused operation.

IRIS-V1-RUNTIME-C109: Every primitive `Float32` or `Float64` result MUST be semantically rounded to its declared width before it becomes an operand of a later operation. Wider physical registers MAY be used only when every later observation is exactly equivalent to using the already-rounded target-width bits.

IRIS-V1-RUNTIME-C110: Floating division by zero MUST follow non-trapping IEEE-754 semantics. Nonzero finite divided by positive or negative zero returns the correspondingly signed infinity. Floating zero divided by floating zero returns NaN. These outcomes MUST NOT raise `DivisionByZeroError`.

IRIS-V1-RUNTIME-C111: Floating overflow from finite arithmetic or Integer-to-float conversion beyond finite range MUST produce signed infinity under IEEE-754 rounding. It MUST NOT raise `FloatOverflowError` or `NumericConversionError` and MUST NOT saturate to maximum finite value.

IRIS-V1-RUNTIME-C112: Ordinary floating operations that propagate or create NaN MUST guarantee only a quiet NaN at the established result width. They MUST NOT specify NaN payload, input-payload choice, or sign bit. Code requiring exact bits MUST use `to_bits` and `from_bits`.

IRIS-V1-RUNTIME-C113: `Float32#to_bits()` MUST return a nonnegative `Integer` in `0..2^32-1`, and `Float64#to_bits()` MUST return a nonnegative `Integer` in `0..2^64-1`. `Float32.from_bits(bits)` and `Float64.from_bits(bits)` MUST accept exactly that width's range and MUST raise `RangeError` for negative or oversized values.

IRIS-V1-RUNTIME-C114: `from_bits(bits).to_bits() == bits` MUST hold for every valid 32-bit or 64-bit IEEE interchange bit pattern, including signaling-NaN encodings. Construction, storage, argument passing, return, copying, and `to_bits` MUST NOT automatically quiet or canonicalize a signaling NaN.

IRIS-V1-RUNTIME-C115: `is_nan`, `is_signaling_nan`, `is_infinite`, `is_finite`, `is_normal`, `is_subnormal`, `is_zero`, and `sign_bit` MUST be pure bit-classification Methods. They MUST NOT perform ordinary floating arithmetic, quiet a signaling NaN, change payload or sign, raise a floating exception, or change `to_bits()`.

IRIS-V1-RUNTIME-C116: `Float32.nan`, `Float32.infinity`, `Float64.nan`, and `Float64.infinity` MUST initially be get-only properties on the corresponding Class objects. The getters return canonical values of that width while the built-in getter remains selected. Negative infinity is produced by ordinary unary negation. No global `nan` or `inf` literal exists.

IRIS-V1-RUNTIME-C117: Built-in special-value getters are ordinary dynamically mutable Class-object property getter Methods. Authorized mutation MAY replace or remove them, and after replacement the expressions need not return canonical IEEE values. Authorized mutation MAY add a setter; adding a setter creates assignment-message behavior only and does not automatically allocate backing storage or synchronize with the getter.

IRIS-V1-RUNTIME-C118: Built-in `Float32.mul_add` and `Float64.mul_add` MUST accept `Integer`, `Float32`, and `Float64` as both multiplier and addend. The common result width is `Float64` if receiver, multiplier, or addend is `Float64`; otherwise it is the receiver's width. Integer operands participate as exact mathematical integers in the fused expression. The exact product-plus-add value is rounded once to the common result width.

IRIS-V1-RUNTIME-C119: Built-in `mul_add` MUST return NaN at the common result width when either fused multiplicand is positive or negative infinity and the other is numeric zero. It MUST also return NaN when an infinite product and infinite addend have opposite signs. These cases MUST NOT raise an Iris exception.

IRIS-V1-RUNTIME-C120: `Integer / Integer` MUST perform real-number division and return `Float64`, including exactly divisible operands. Both operands are interpreted under the `Float64` conversion and IEEE-754 rules, so precision loss and signed infinity are permitted.

IRIS-V1-RUNTIME-C121: Built-in `Integer.div` MUST compute floor division for every nonzero integer divisor. Division by zero in `/`, `div`, `mod`, or any integer division or remainder operation MUST raise `DivisionByZeroError`.

IRIS-V1-RUNTIME-C122: Built-in `Integer.mod` MUST be defined as `a - (a div b) * b` for nonzero `b`. It MUST satisfy `a == (a div b) * b + (a mod b)` and `abs(a mod b) < abs(b)`. Every nonzero result has the same sign as divisor `b`.

IRIS-V1-RUNTIME-C123: Built-in `Integer ** Integer` MUST return arbitrary-precision `Integer` for every nonnegative exponent. A negative integer exponent MUST use `Float64` reciprocal-power semantics and return `Float64`.

IRIS-V1-RUNTIME-C124: `Integer(0) ** Integer(0)` MUST raise `DomainError`. `Integer(0)` to a positive integer exponent returns `Integer(0)`. `Integer(0)` to a negative integer exponent returns `Float64(+Infinity)` and MUST NOT raise `DivisionByZeroError` or `DomainError`.

IRIS-V1-RUNTIME-C125: For `Float32` and `Float64`, every combination of positive or negative zero base and positive or negative floating-point zero exponent MUST raise `DomainError`. For zero float bases with a negative nonzero `Integer` exponent, positive zero returns positive infinity, negative zero with odd-magnitude exponent returns negative infinity, and negative zero with even-magnitude exponent returns positive infinity at the base width.

IRIS-V1-RUNTIME-C126: A negative finite `Float32` or `Float64` base raised to a non-integer floating exponent with no real result MUST return NaN at the established result width and MUST NOT raise `DomainError`. Float exponentiation width MUST follow ordinary float width rules: `Float32` with `Integer` or `Float32` returns `Float32`, `Float64` with `Integer` or `Float64` returns `Float64`, and mixed `Float32`/`Float64` returns `Float64`.

IRIS-V1-RUNTIME-C127: Built-in `Integer` bitwise operators `&`, `|`, `^`, prefix `~`, `<<`, and `>>` MUST use an abstract infinite sign-extended two's-complement model. `~x == -x - 1`, `~0 == -1`, and right shift is arithmetic.

IRIS-V1-RUNTIME-C128: Integer shifts MUST accept counts of any sign. A negative count reverses direction with the exact arbitrary-precision absolute value: `x << -n` is equivalent to `x >> n`, and `x >> -n` is equivalent to `x << n` for positive `n`. Zero count is identity. Negative counts MUST NOT raise `RangeError`.

IRIS-V1-RUNTIME-C129: Every nonnegative right shift count has a mathematical result. Under infinite sign extension, sufficiently large right shifts of nonnegative values stabilize at `0`, and sufficiently large right shifts of negative values stabilize at `-1`. A mathematically valid huge allocation MAY raise `ResourceError` or `MemoryLimitError`; host limits MUST NOT alter successful mathematical results.

IRIS-V1-RUNTIME-C130: Built-in numeric `==` and `!=` MUST compare exact mathematical value across `Integer`, `Float32`, and `Float64`, except NaN. An arbitrary-precision `Integer` MUST NOT first be rounded to float for equality. Positive and negative floating zero compare equal. Every NaN compares unequal to every value, including itself and another NaN.

IRIS-V1-RUNTIME-C131: Built-in numeric `<`, `<=`, `>`, `>=`, and `<=>` over finite values MUST use exact mathematical values and remain symmetric across operand order. Infinities compare by extended-real position. If either operand is NaN, `<=>` returns `nil` and the four ordered comparisons return `false` without raising.

IRIS-V1-RUNTIME-C132: Built-in numeric `<=>` Methods MUST accept `other: Object` and return `Integer?`. For `Integer`, `Float32`, or `Float64`, they return exact `Integer(-1)`, `Integer(0)`, `Integer(1)`, or `nil` for NaN. A nonnumeric object returns `nil`.

IRIS-V1-RUNTIME-C133: Built-in numeric comparison Methods are ordinary dynamically replaceable Methods. The frozen numeric comparison rules specify their unmodified standard behavior only. Users replacing them assume responsibility for any resulting symmetry, transitivity, ordering, or equality/hash inconsistency.

IRIS-V1-RUNTIME-C134: No quiet or signaling NaN of either float width is a valid Hash key. Hash construction, insertion, key update, or direct built-in public `hash` on NaN MUST raise `InvalidKeyError`.

IRIS-V1-RUNTIME-C135: The following built-in numeric operation table is normative:

| Operation family                   | Receiver and operands                            | Result and exceptional behavior                                                            |
| ---------------------------------- | ------------------------------------------------ | ------------------------------------------------------------------------------------------ |
| `Integer + - *` with `Integer` | Exact arbitrary-precision integer arithmetic     | `Integer`; allocation failure raises `ResourceError` or `MemoryLimitError`           |
| `Integer / Integer`              | Real division through`Float64` conversion      | `Float64`; zero divisor raises `DivisionByZeroError`                                   |
| `Integer div Integer`            | Floor quotient                                   | `Integer`; zero divisor raises `DivisionByZeroError`                                   |
| `Integer mod Integer`            | `a - (a div b) * b`                            | `Integer`; zero divisor raises `DivisionByZeroError`                                   |
| `Integer ** Integer`             | Nonnegative exponent                             | `Integer`; `0 ** 0` raises `DomainError`                                             |
| `Integer ** Integer`             | Negative exponent                                | `Float64`; `0 ** negative` returns `+Infinity`                                       |
| `Integer OP Float32`             | Receiver converted to`Float32`                 | `Float32`                                                                                |
| `Integer OP Float64`             | Receiver converted to`Float64`                 | `Float64`                                                                                |
| `Float32 OP Integer`             | Argument converted to`Float32`                 | `Float32`                                                                                |
| `Float64 OP Integer`             | Argument converted to`Float64`                 | `Float64`                                                                                |
| `Float32 OP Float32`             | IEEE-754 binary32                                | `Float32`                                                                                |
| `Float64 OP Float64`             | IEEE-754 binary64                                | `Float64`                                                                                |
| `Float32 OP Float64` or reverse  | Widen`Float32` to `Float64`                  | `Float64`                                                                                |
| Float division by zero             | IEEE-754 non-trapping                            | Signed infinity or NaN, no`DivisionByZeroError`                                          |
| Float overflow                     | IEEE-754 overflow                                | Signed infinity, no overflow exception                                                     |
| `FloatN ** Integer`              | Base width retained unless mixed with`Float64` | `FloatN` by width rules, zero negative exponent uses signed-zero parity                  |
| `FloatN ** FloatN`               | Floating exponentiation                          | Same width; zero to zero raises`DomainError`; negative finite to non-integer returns NaN |
| Mixed float exponentiation         | Any`Float64` operand                           | `Float64`                                                                                |
| `mul_add`                        | Exact fused product plus add                     | Common width, one final rounding, invalid infinity cases return NaN                        |
| `to_bits` and `from_bits`      | Exact bit reinterpretation                       | Round-trips every bit pattern; range violation raises`RangeError`                        |
| Bitwise`& \| ^ ~ << >>`           | `Integer` only built-in standard operators     | Infinite sign-extended two's-complement; huge allocation may raise`ResourceError`        |

## Stable Hash Contracts And Vectors

IRIS-V1-RUNTIME-C136: Built-in public numeric `hash` MUST return a nonnegative `Integer` in `0..2^64-1`. For equal built-in numeric values under ordinary `==`, it MUST return the same value across concrete numeric types and float widths, including positive and negative zero. Unequal values MAY collide.

IRIS-V1-RUNTIME-C137: Public numeric `hash` MUST be stable across processes, platforms, interpreter/JIT modes, pointer widths, and conforming implementations within Iris language major version 1 while the built-in Method remains selected. Hash containers MUST NOT use the public value directly as a bucket index without secret randomized internal mixing.

IRIS-V1-RUNTIME-C138: Numeric hash computation MUST use BLAKE3 derive-key mode with exact ASCII context `Iris Language v1 stable numeric hash`. It MUST compute the standard 32-byte digest over the canonical numeric encoding, take digest bytes `0..8`, interpret those bytes as unsigned little-endian, and return that `Integer`.

IRIS-V1-RUNTIME-C139: Canonical numeric encodings MUST begin with one top-level byte: `0x00` for finite mathematical values, `0x01` for positive infinity, and `0x02` for negative infinity. Concrete numeric type and float width MUST be omitted. NaN MUST have no canonical hash encoding.

IRIS-V1-RUNTIME-C140: A finite zero numeric encoding MUST be exactly `[0x00, 0x00]`. A nonzero finite value MUST use exact normal form `sign * odd_significand * 2^exponent`, with all factors of two removed from the positive odd significand. After top-level finite tag `0x00`, subtag `0x01` means positive and `0x02` means negative.

IRIS-V1-RUNTIME-C141: A normalized positive odd significand MUST encode as `ULEB128(byte_length) || big_endian_magnitude`. `byte_length` MUST use the shortest canonical ULEB128 and be at least one. Magnitude MUST be the shortest unsigned big-endian byte sequence with no leading zero and an odd positive value.

IRIS-V1-RUNTIME-C142: The signed arbitrary-precision exponent MUST map to a nonnegative arbitrary-precision integer with mathematical ZigZag: `2*e` when `e >= 0`, and `-2*e - 1` when `e < 0`. The mapped value MUST encode as shortest canonical ULEB128. Host-width shift or XOR formulas MAY be used only when exactly equivalent without overflow or width dependence.

IRIS-V1-RUNTIME-C143: Positive infinity MUST encode exactly as `[0x01]`, and negative infinity exactly as `[0x02]`. No width, sign field, payload, length, or trailing byte is present.

IRIS-V1-RUNTIME-C144: Canonical numeric hash bytes are specification-internal. A conforming implementation MUST NOT expose them as a standard Iris Method such as `canonical_numeric_bytes`. User programs observe only the final public hash, while conformance tools MAY inspect canonical bytes.

IRIS-V1-RUNTIME-C145: Singleton hashes for `nil`, `false`, and `true` MUST use BLAKE3 derive-key mode with exact ASCII context `Iris Language v1 stable singleton hash` and one-byte inputs `[0x00]`, `[0x01]`, and `[0x02]` respectively. Digest reduction MUST use the first eight bytes as unsigned little-endian.

IRIS-V1-RUNTIME-C146: The following numeric and singleton hash vector table is normative. Digest prefixes and `hash` values were computed with the BLAKE3 derive-key contexts named in IRIS-V1-RUNTIME-C138 and IRIS-V1-RUNTIME-C145.

| Vector ID                | Value                                                             | Canonical input hex | Context                                    | Digest bytes`0..8` hex | Public hash Integer      |
| ------------------------ | ----------------------------------------------------------------- | ------------------- | ------------------------------------------ | ------------------------ | ------------------------ |
| `IRIS-V1-RUNTIME-V001` | `Integer(0)`, `Float32(+0.0)`, `Float64(-0.0)`              | `0000`            | `Iris Language v1 stable numeric hash`   | `c030472a1b58c53c`     | `4379003086384345280`  |
| `IRIS-V1-RUNTIME-V002` | `Integer(1)`, exact `Float32(1.0)`, exact `Float64(1.0)`    | `0001010100`      | `Iris Language v1 stable numeric hash`   | `384e0f3cb1fc5bf7`     | `17824117788395916856` |
| `IRIS-V1-RUNTIME-V003` | `Integer(-1)`, exact `Float32(-1.0)`, exact `Float64(-1.0)` | `0002010100`      | `Iris Language v1 stable numeric hash`   | `0cb716ef77f96039`     | `4134578751433783052`  |
| `IRIS-V1-RUNTIME-V004` | `Integer(2)` and exact `Float64(2.0)`                         | `0001010102`      | `Iris Language v1 stable numeric hash`   | `15e928f2541b81c4`     | `14159628755083520277` |
| `IRIS-V1-RUNTIME-V005` | Exact mathematical`3/2` as `Float32(1.5)` or `Float64(1.5)` | `0001010301`      | `Iris Language v1 stable numeric hash`   | `6fd5e66f00ac66c4`     | `14152187996935738735` |
| `IRIS-V1-RUNTIME-V006` | Positive infinity, either float width                             | `01`              | `Iris Language v1 stable numeric hash`   | `60fe3108fddb52bd`     | `13642208101069356640` |
| `IRIS-V1-RUNTIME-V007` | Negative infinity, either float width                             | `02`              | `Iris Language v1 stable numeric hash`   | `e99dd5adce797260`     | `6949751103572778473`  |
| `IRIS-V1-RUNTIME-V008` | `nil`                                                           | `00`              | `Iris Language v1 stable singleton hash` | `d37c681abf4074a4`     | `11850167709044604115` |
| `IRIS-V1-RUNTIME-V009` | `false`                                                         | `01`              | `Iris Language v1 stable singleton hash` | `24d2464b3697b5f8`     | `17921396551637717540` |
| `IRIS-V1-RUNTIME-V010` | `true`                                                          | `02`              | `Iris Language v1 stable singleton hash` | `40168bee0b35dfc4`     | `14186115676603356736` |

IRIS-V1-RUNTIME-C147: The following Contract-view hash example vector is normative for derivation shape. Given receiver public hash `nil.hash == 11850167709044604115` and Contract Type public hash `0x0123456789abcdef`, the view-hash input MUST be `d37c681abf4074a4efcdab8967452301`, the context MUST be `Iris Language v1 contract view hash`, digest bytes `0..8` MUST be `2a88da355f670591`, and the public hash MUST be `10449872169006172202`.

## Built-In Openness And Meta Safety

IRIS-V1-RUNTIME-C148: `Nil`, `Bool`, `Integer`, `Float32`, and `Float64` stable Classes MAY be opened to gain or remove dynamic members, compose Modules, replace compatible Method implementations, and alter property protocols when their effective `MetaCapabilities` and static spine permit it.

IRIS-V1-RUNTIME-C149: Built-in value Class openness MUST NOT break singleton identity, numeric immutability, float bit immutability, hidden representation safety, GC tracing, native layout, declared Contracts, static spine obligations, or primitive/JIT guard validity.

IRIS-V1-RUNTIME-C150: The runtime superclass of `Nil`, `Bool`, `Integer`, `Float32`, and `Float64` MUST NOT be changed by user meta APIs. Any declarative or programmatic superclass mutation attempt for these protected Classes MUST raise a meta-operation exception, abort the complete transaction group, and publish no candidate revision.

IRIS-V1-RUNTIME-C151: All syntax, reflection, Dynamic, package, native, Host, helper, and indirect meta-operation paths MUST lower to the same operation-level `MetaCapabilities` and structural safety checks. User code MUST NOT acquire or fabricate internal capabilities to bypass protected built-in superclass restrictions.

IRIS-V1-RUNTIME-C152: Effective `MetaCapabilities` MUST be immutable policy for a Class revision. Source-expressible `meta deny` MAY only narrow the default capability set. It MUST NOT grant runtime-internal powers or later widen an inherited, Module-provided, or Contract-required denial.

IRIS-V1-RUNTIME-C153: Denies MUST accumulate through the static superclass chain, runtime superclass chain, composed Modules, and declared Contracts according to later metaprogramming rules. Candidate validation MUST compute the effective set before applying or committing structural operations, and losing authorization MUST cause atomic failure.

IRIS-V1-RUNTIME-C154: `method_set` and `method_body` MUST be separate capabilities. `property_set` and `property_body` MUST be separate capabilities. Class/shared storage shape and writes MUST use `class_state_set` and `class_state_write` as separate capabilities. No hotfix, reflection, native, or Host path MAY bypass these capabilities.

IRIS-V1-RUNTIME-C155: The following openness table is normative:

| Target            | Initially allowed dynamic behavior                                                                        | Intrinsic protected behavior                                                                                  |
| ----------------- | --------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `Nil` Class     | Compatible Methods, properties, Modules,`to_bool`, `method_missing` behavior                          | Singleton identity and protected superclass                                                                   |
| `Bool` Class    | Compatible Methods, properties, Modules, comparison behavior                                              | Two singleton identities, Bool distinctness from numbers, protected superclass                                |
| `Integer` Class | Compatible Methods, properties, Modules, numeric Method replacement                                       | Identity-less value semantics, arbitrary precision value immutability, no dynamic ivars, protected superclass |
| `Float32` Class | Compatible Methods, properties, Modules, special-value getter/setter mutation, numeric Method replacement | Identity-less value semantics, 32-bit interchange immutability, no dynamic ivars, protected superclass        |
| `Float64` Class | Compatible Methods, properties, Modules, special-value getter/setter mutation, numeric Method replacement | Identity-less value semantics, 64-bit interchange immutability, no dynamic ivars, protected superclass        |
| User Class        | Capabilities allowed by static spine and effective policy                                                 | Static spine, declared Contract set, native/layout safety, accumulated denies                                 |
| Module            | Compatible Methods, Module composition, declared policy narrowing                                         | Module identity, static header promises, no self-granted host private access                                  |
| Contract          | Static requirement declaration only                                                                       | No bodies, no stored state, no initializers, no raw ivars, no open operation                                  |

## Examples

IRIS-V1-RUNTIME-EX001: Informative example, logical Class and active revision:

```iris
let klass = Counter
let first = Counter.new()

open class Counter {
  override fun value() -> Integer { 2 }
}

klass same? Counter        // true, reopen kept the logical Class identity
first.value()             // later send uses the current active revision
```

IRIS-V1-RUNTIME-EX002: Informative example, BoundMethod identity and replacement:

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

IRIS-V1-RUNTIME-EX003: Informative example, ordinary and Contract-qualified namespaces:

```iris
let view = parser as ParserContract
parser.process(input)     // ordinary selector process
view.process(input)       // still ordinary selector process on the receiver
view..process(input)      // qualified Contract slot ParserContract::process
```

IRIS-V1-RUNTIME-EX004: Informative example, Module MRO order:

```iris
module A { fun trace() -> Symbol { :A } }
module B mixin A { override fun trace() -> Symbol { :B } }
class C extends Object mixin A, B {}

C.new().trace()           // selects B before A
```

IRIS-V1-RUNTIME-EX005: Informative example, numeric edge behavior:

```iris
5 / 2                     // Float64(2.5)
-5 div 2                  // Integer(-3)
-5 mod 2                  // Integer(1)
0 ** -1                   // Float64.infinity
Float64.from_bits(0x8000000000000000).hash == Float64.from_bits(0x0000000000000000).hash
```

## Runtime Conformance Vectors

IRIS-V1-RUNTIME-C156: The following runtime vector table is normative. The conformance chapter MUST preserve these vector IDs or map them to machine-readable records with the same observable outcomes:

| Vector ID                | Kind     | Scenario                                                                          | Expected result                                                           |
| ------------------------ | -------- | --------------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| `IRIS-V1-RUNTIME-V011` | Positive | Reopen a Class and compare captured Class object with current name using`same?` | `true`                                                                  |
| `IRIS-V1-RUNTIME-V012` | Positive | Existing instance sends after successful compatible Method replacement            | New active Method result                                                  |
| `IRIS-V1-RUNTIME-V013` | Positive | Entered old frame continues while replacement commits                             | Entered frame returns old body result                                     |
| `IRIS-V1-RUNTIME-V014` | Failure  | Retained Method invoked on receiver whose current MRO lacks lexical owner         | `MethodBindingError` at entry                                           |
| `IRIS-V1-RUNTIME-V015` | Failure  | Retained Method executes`super` after owner removed from current MRO            | `MethodBindingError` at entry under `IRIS-V1-RUNTIME-C015`             |
| `IRIS-V1-RUNTIME-V016` | Positive | `obj.method same? obj.method`                                                   | `false`                                                                 |
| `IRIS-V1-RUNTIME-V017` | Positive | Saved BoundMethod compared to itself with`same?`                                | `true`                                                                  |
| `IRIS-V1-RUNTIME-V018` | Failure  | `Integer(1) same? Integer(1)`                                                   | `IdentityError`                                                         |
| `IRIS-V1-RUNTIME-V019` | Positive | `nil same? nil`, `true same? true`, `false same? false`                     | all`true`                                                               |
| `IRIS-V1-RUNTIME-V020` | Positive | `(value as C)..m()` and `value.m()` installed with distinct bodies            | Qualified call selects only`C::m`; ordinary call selects ordinary `m` |
| `IRIS-V1-RUNTIME-V021` | Failure  | Missing qualified Contract slot                                                   | `ContractDispatchError`, no `method_missing`                          |
| `IRIS-V1-RUNTIME-V022` | Failure  | Private Method exists but caller lacks access                                     | `MethodVisibilityError`, no `method_missing`                          |
| `IRIS-V1-RUNTIME-V023` | Positive | `mixin A, B` and sequential `include(A); include(B)`                          | Lookup order places`B` before `A`                                     |
| `IRIS-V1-RUNTIME-V024` | Positive | Nested Module dedupe with`B mixin A` and `C mixin A, B`                       | MRO contains closed`A` once at nearest occurrence                       |
| `IRIS-V1-RUNTIME-V025` | Positive | Construction revision commits old layout, then send after later open              | Construction uses captured revision; later send uses active revision      |
| `IRIS-V1-RUNTIME-V026` | Failure  | `initialize` raises after `self` escaped                                      | `new` propagates; escaped object remains ordinary and memory-safe       |
| `IRIS-V1-RUNTIME-V027` | Positive | `a.x = b.y = v` where inner setter returns marker                               | Outer setter receives marker                                              |
| `IRIS-V1-RUNTIME-V028` | Positive | Missing undeclared`@x` read                                                     | Returns`nil`, creates no slot                                           |
| `IRIS-V1-RUNTIME-V029` | Failure  | Identity-less numeric value creates undeclared`@x`                              | `InstanceStateError`                                                    |
| `IRIS-V1-RUNTIME-V030` | Positive | `false && side_effect()`                                                        | RHS not evaluated; returns`false`                                       |
| `IRIS-V1-RUNTIME-V031` | Positive | `true \|\| side_effect()`                                                         | RHS not evaluated; returns`true`                                        |
| `IRIS-V1-RUNTIME-V032` | Failure  | `to_bool` returns non-Bool                                                      | `TypeContractError`                                                     |
| `IRIS-V1-RUNTIME-V033` | Positive | Remove`to_bool`, define `method_missing(:to_bool, [], nil)` returning Bool    | Truth test uses fallback once                                             |
| `IRIS-V1-RUNTIME-V034` | Failure  | Existing selector arity mismatch                                                  | `ArgumentError`, no `method_missing`                                  |
| `IRIS-V1-RUNTIME-V035` | Positive | `Float64.nan == Float64.nan` and `Float64.nan != Float64.nan`                 | `false`, `true`                                                       |
| `IRIS-V1-RUNTIME-V036` | Positive | Positive and negative float zero equality and hash                                | Equal and same hash                                                       |
| `IRIS-V1-RUNTIME-V037` | Failure  | Hash insertion with any quiet or signaling NaN                                    | `InvalidKeyError`                                                       |
| `IRIS-V1-RUNTIME-V038` | Positive | `Float64.from_bits(bits).to_bits()` for signaling-NaN bits                      | Same bits                                                                 |
| `IRIS-V1-RUNTIME-V039` | Positive | `-5 div 2`, `5 div -2`, `-5 div -2`                                         | `-3`, `-3`, `2`                                                     |
| `IRIS-V1-RUNTIME-V040` | Positive | `-5 mod 2`, `5 mod -2`, `-5 mod -2`                                         | `1`, `-1`, `-1`                                                     |
| `IRIS-V1-RUNTIME-V041` | Failure  | `0 ** 0` for Integer or zero Float exponent cases                               | `DomainError`                                                           |
| `IRIS-V1-RUNTIME-V042` | Positive | Negative finite float to non-integer float exponent                               | NaN, no`DomainError`                                                    |
| `IRIS-V1-RUNTIME-V043` | Positive | `~0`, `-3 >> 1`, `x << -2`                                                  | `-1`, `-2`, same as `x >> 2`                                        |
| `IRIS-V1-RUNTIME-V044` | Failure  | Protected built-in superclass mutation in transaction group                       | Meta-operation exception and no candidate publication                     |
| `IRIS-V1-RUNTIME-V045` | Positive | Stable hash vectors V001 through V010                                             | Exact Integer outputs in IRIS-V1-RUNTIME-C146                             |
| `IRIS-V1-RUNTIME-V048` | Positive | `Float64.infinity.mul_add(1, Float64(-Infinity))`                                 | NaN at `Float64`, no Iris exception                                        |


## Runtime Coverage Vectors

IRIS-V1-RUNTIME-C160: The following records are normative. Each Source/Input cell is executable Iris source or a complete controlled runtime fixture. Each decision is directly covered by its stated observable.

IRIS-V1-RUNTIME-C161: A stored property named `name` creates the declared raw current-receiver slot `@name` in its revision-level typed instance storage. Its generated getter MUST read that exact slot, and its generated setter MUST check and write that exact slot. A compatible `property fun` replacement changes only the accessor Method body: it creates no second backing slot and does not implicitly read or write the slot unless its body explicitly uses `@name`; any such raw access is the same `(receiver, name)` slot required by IRIS-V1-RUNTIME-C066.

IRIS-V1-RUNTIME-C162: A `shared_decl` written per IRIS-V1-GRAMMAR-C059 creates the declared hierarchy binding cell that IRIS-V1-RUNTIME-C073 describes, anchored to the enclosing logical Class or Module. `shared let` creates an immutable cell and any later assignment MUST fail; `shared mut` creates an assignable cell. Because IRIS-V1-RUNTIME-C075 forbids a subclass from shadowing or redeclaring an anchored cell, a `shared_decl` whose name is already anchored anywhere on the declaring Class's static lexical ancestry MUST be rejected as a duplicate declaration, and the transaction MUST publish no candidate revision. Assignment to a name that no `shared_decl` ever declared continues to fail as missing declared storage under IRIS-V1-CONTROL-C009.

| Vector ID | Category | Applicability | Source/Input | Expected observable | Decisions |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-RUNTIME-V052` | differential | interpreter required; JIT required; native not applicable | `((2 ** 200) + 1) - (2 ** 200)` | `Integer(1)` with Type `Integer`; interpreter and JIT agree; no representation Type split. | `D-003` |
| `IRIS-V1-RUNTIME-V053` | positive | interpreter required; JIT required; native not applicable; reason: pure Iris numeric source has no native boundary | `type_of(1.0f32); type_of(1.0f64); type_of(1.0); type_of(1e10f64)` | `type`: `Float32`, `Float64`, `Float64`, `Float64`, in order. | `D-005`, `D-006`, `D-007` |
| `IRIS-V1-RUNTIME-V054` | negative | interpreter required; JIT not applicable; native not applicable; reason: lexical rejection | `1.0F32` | `LEX_BAD_FLOAT_SUFFIX` in lexical phase. | `D-007` |
| `IRIS-V1-RUNTIME-V055` | positive | interpreter required; JIT required; native not applicable; reason: pure Iris dispatch source has no native boundary | `fixture: {source: "Probe.left() + Probe.right()", setup: "left appends :left once and returns receiver; right appends :right once and returns argument; receiver + initially returns :old, then an authorized open replaces + with a Method returning :new", runs: 2}` | `value`: `[:old, :new]`; `side_effects`: evaluation log is exactly `[:left, :right, :left, :right]`. | `D-008` |
| `IRIS-V1-RUNTIME-V056` | positive | interpreter required; JIT required; native not applicable; reason: pure Iris numeric source has no native boundary | `type_and_value(16777217 + 0.0f32); type_and_value(0.0f32 + 16777217); type_and_value(1.5f32 + 2.25f64); type_and_value(2.25f64 + 1.5f32)` | First two values are `Float32(16777216.0)`; last two are `Float64(3.75)`. | `D-010`, `D-011`, `D-012` |
| `IRIS-V1-RUNTIME-V057` | positive | interpreter required; JIT required; native not applicable | `1.0f64 / 0.0f64; 1.0f64 / -0.0f64; 0.0f64 / 0.0f64` | `+infinity`, `-infinity`, NaN; no `DivisionByZeroError`. | `D-013` |
| `IRIS-V1-RUNTIME-V058` | positive | interpreter required; JIT required; native not applicable; reason: pure Iris numeric source has no native boundary | `Float32.from_bits(0x7f7fffff) * 2.0f32; Float64.from_bits(0x7fefffffffffffff) * 2.0f64` | `value`: positive infinity at `Float32`, then positive infinity at `Float64`; no Iris error. | `D-014` |
| `IRIS-V1-RUNTIME-V059` | positive | interpreter required; JIT required; native not applicable | `Float64.nan == Float64.nan; Float64.nan < 1.0; Float64.nan <=> 1.0` | `false`, `false`, `nil`. | `D-015`, `D-018` |
| `IRIS-V1-RUNTIME-V060` | negative | interpreter required; JIT required; native not applicable | `1 / 0; 1 div 0; 1 mod 0` | Each raises `DivisionByZeroError`. | `D-019` |
| `IRIS-V1-RUNTIME-V046` | positive | interpreter required; JIT required; native not applicable | `Integer(5) / Integer(2); Integer(4) / Integer(2)` | `Float64(2.5)` and `Float64(2.0)`. | `D-020` |
| `IRIS-V1-RUNTIME-V061` | positive | interpreter required; JIT required; native not applicable; reason: pure Iris dispatch source has no native boundary | `class A { public fun scale(value: Integer) -> Integer { value * 2 } }; let a = A.new(); a.scale(3); a scale 3` | `value`: `Integer(6)`, then `Integer(6)`; both forms select the same `scale` Method. | `D-021`, `D-022` |
| `IRIS-V1-RUNTIME-V062` | positive | interpreter required; JIT required; native not applicable | `2 ** 10; 2 ** -3; Float32.from_bits(0x80000000) ** -3` | `Integer(1024)`, `Float64(0.125)`, negative `Float32` infinity. | `D-026`, `D-030`, `D-032` |
| `IRIS-V1-RUNTIME-V063` | positive | interpreter required; JIT required; native not applicable | `1 >> 1000000; -1 >> 1000000; -3 >> 1000000; 1 << -2` | `0`, `-1`, `-1`, `0`. | `D-036`, `D-037` |
| `IRIS-V1-RUNTIME-V064` | negative | interpreter required; JIT required; native not applicable; reason: controlled runtime quota fixture has no native API boundary | `fixture: {memory_limit_bytes: 1048576, source: "1 << 1000000000"}` | `error`: `ResourceError` or `MemoryLimitError`, phase `runtime`; no `RangeError`, value, or process failure. | `D-038` |
| `IRIS-V1-RUNTIME-V065` | positive | interpreter required; JIT required; native not applicable | `Float32.nan; Float64.infinity; -Float64.infinity` | Canonical values with Types `Float32`, `Float64`, `Float64`; bare `nan` and `inf` are absent names. | `D-051`, `D-052` |
| `IRIS-V1-RUNTIME-V066` | differential | interpreter required; JIT required; native not applicable; reason: host-rounding fixture controls execution without crossing a native API | `fixture: {host_rounding_mode: toward_negative, source: "(Float32.from_bits(0x3f800001) * Float32.from_bits(0x3f800001)).to_bits()"}` | `backends`: interpreter and JIT; `equivalence`: exact result bits under IEEE-754 `roundTiesToEven`, independent of host rounding mode. | `D-055`, `D-057` |
| `IRIS-V1-RUNTIME-V067` | positive | interpreter required; JIT required; native not applicable; reason: pure Iris fused arithmetic has no native boundary | `Float32.from_bits(0x3f800001).mul_add(Float32.from_bits(0x3f800001), Float32.from_bits(0xbf800000)); Float32.from_bits(0x3f800001) * Float32.from_bits(0x3f800001) + Float32.from_bits(0xbf800000); Float32.from_bits(0x3f800000).mul_add(2, 3.0)` | First and second Float32 results have the same encoding; final Type is `Float64`. | `D-056`, `D-058`, `D-059` |
| `IRIS-V1-RUNTIME-V047` | positive | interpreter required; JIT required; native not applicable | `Float64.infinity.mul_add(0, 1); Float32.infinity.mul_add(0, 1)` | NaN at the common result width; no Iris exception. | `D-060` |
| `IRIS-V1-RUNTIME-V068` | positive | interpreter required; JIT required; native not applicable; reason: pure Iris float-bit source has no native boundary | `let x = Float64.from_bits(0x7ff0000000000001); [x.is_nan(), x.is_signaling_nan(), x.is_infinite(), x == x, x < 0.0, x <=> 0.0, x.to_bits()]` | `value`: `[true, true, false, false, false, nil, 0x7ff0000000000001]`; no Iris error. | `D-063`, `D-065`, `D-066`, `D-067` |
| `IRIS-V1-RUNTIME-V069` | negative | interpreter required; JIT required; native not applicable | `Float64.from_bits(-1); Float32.from_bits(2 ** 32); Float64.from_bits(2 ** 64)` | Each raises `RangeError`; no truncation. | `D-064` |
| `IRIS-V1-RUNTIME-V070` | negative | interpreter required; JIT required; native not applicable; reason: independent pure Iris sources have no native boundary | `fixture: {independent_sources: ["let n = 1; n.@x = 2", "let f = 1.0f32; f.@x = 2", "1.0f64 same? 1.0f64"]}` | `error`: `InstanceStateError`, `InstanceStateError`, and `IdentityError`, respectively, phase `runtime`. | `D-068`, `D-069`, `D-070` |
| `IRIS-V1-RUNTIME-V071` | positive | interpreter required; JIT required; native not applicable | `Integer(9007199254740993) == 9007199254740993.0f64; 1 == 1.0f32` | `false`, then `true`; equal numbers have equal public hashes. | `D-017`, `D-071` |
| `IRIS-V1-RUNTIME-V072` | negative | interpreter required; JIT required; native not applicable | `Float32.nan.hash; Float64.nan.hash; %{ Float64.nan: 1 }` | Every evaluation raises `InvalidKeyError`. | `D-072`, `D-080` |
| `IRIS-V1-RUNTIME-V073` | differential | interpreter required; JIT required; native not applicable | `fixture: public_hash_inputs=[Integer(0),Float64.from_bits(0),nil,false,true]` | Exact C146 64-bit public hashes agree across backends. | `D-073`, `D-074`, `D-075`, `D-076`, `D-077`, `D-078`, `D-079`, `D-081`, `D-082`, `D-083`, `D-084`, `D-085`, `D-086`, `D-112`, `D-113` |
| `IRIS-V1-RUNTIME-V074` | negative | interpreter required; JIT required; native not applicable | `Integer(1).canonical_numeric_bytes()` | `MessageNotFoundError`; canonical hash bytes are not public API. | `D-087` |
| `IRIS-V1-RUNTIME-V075` | positive | interpreter required; JIT required; native not applicable; reason: controlled Iris comparison fixture has no native boundary | `fixture: {source: "let a = Probe.new(); let same = a; let b = Probe.new(); [a == same, a <=> b, a == b, a != b, a < b]", setup: "Probe#<=> returns nil and increments calls"}` | `value`: `[true, nil, false, true, false]`; `side_effects`: `<=>` is not called for same-reference equality and is called once for each distinct-reference comparison. | `D-090`, `D-096`, `D-097` |
| `IRIS-V1-RUNTIME-V076` | positive | interpreter required; JIT required; native not applicable; reason: pure Iris singleton comparison source has no native boundary | `[false <=> false, false <=> true, true <=> false, true == 1, false == 0, nil <=> nil, nil <=> false, nil < Object.new()]` | `value`: `[Integer(0), Integer(-1), Integer(1), false, false, Integer(0), nil, false]`. | `D-114`, `D-115` |
| `IRIS-V1-RUNTIME-V077` | positive | interpreter required; JIT required; native not applicable | `obj.method == obj.method; closure.call() == closure.call(); obj.method same? obj.method` | Each independently evaluated callable compares unequal and is not identical. | `D-107`, `D-108` |
| `IRIS-V1-RUNTIME-V078` | positive | interpreter required; JIT required; native not applicable | alias one Method; compare it, a same-body Method, and Class/Module/Contract/Type objects | Alias is identical; distinct definitions compare unequal by default. | `D-109`, `D-110` |
| `IRIS-V1-RUNTIME-V079` | positive | interpreter required; JIT required; native not applicable | `fixture: object_hash_before_compact_gc; compact_gc; object_hash_after_compact_gc` | Same runtime-local hash in `0..2^64-1`; fixture exposes no address. | `D-111` |
| `IRIS-V1-RUNTIME-V080` | negative | interpreter required; JIT not applicable; native not applicable; reason: declaration validation precedes executable or native dispatch | `class A { fun f(value: Integer) -> Symbol { :integer }; fun f(value: String) -> Symbol { :string } }` | `error`: declaration-validation error anchored to IRIS-V1-RUNTIME-C024; `side_effects`: no overload set or Class publication. | `D-235`, `D-358` |
| `IRIS-V1-RUNTIME-V081` | positive | interpreter required; JIT required; native not applicable | `(value as C)..m(); value.m()` with distinct bodies | `:qualified`, then `:ordinary`; lookup namespaces stay separate. | `D-238`, `D-355` |
| `IRIS-V1-RUNTIME-V082` | positive | interpreter required; JIT required; native not applicable; reason: controlled Class open has no native boundary | `fixture: {source: "let before = A; let value = A.new(); open class A { override public fun m() { :new } }; [before same? A, value.m()]", setup: "A#m initially returns :old and A conforms to Contract C"}` | `value`: `[true, :new]`; nominal Type and Contract identity are unchanged. | `D-258`, `D-261` |
| `IRIS-V1-RUNTIME-V083` | positive | interpreter required; JIT required; native not applicable | `fixture: begin A.new; commit compatible A open before initialize returns; call instance.m` | allocation/initialize use captured revision; later send uses active revision. | `D-259`, `D-442` |
| `IRIS-V1-RUNTIME-V084` | negative | interpreter required; JIT required; native not applicable | `A.new_current(); A.new_checked()` | Both absent selectors raise `MessageNotFoundError`. | `D-260` |
| `IRIS-V1-RUNTIME-V085` | positive | interpreter required; JIT required; native not applicable; reason: controlled revision fixture has no native boundary | `fixture: {setup: "A#migrate_revision increments calls and returns nil; tracked instance a", actions: ["commit compatible open", "observe calls", "a.migrate_revision(old_revision, new_revision)", "attempt old_revision.reactivate()"]}` | `value`: calls before explicit invocation `0`, explicit result `nil`; revision reactivation raises a meta-operation error. | `D-264`, `D-265`, `D-266` |
| `IRIS-V1-RUNTIME-V086` | positive | interpreter required; JIT required; native not applicable | `a.@x=1; a.@x="s"; b.@x` for distinct instances | `a` reads `"s"`, `b` reads nil; slot Type is `Dynamic<Object>`. | `D-307`, `D-308`, `D-309` |
| `IRIS-V1-RUNTIME-V087` | positive | interpreter required; JIT required; native not applicable | escaped Closure from instance Method increments `@x` after return | It mutates original receiver; rebind raises `MethodBindingError`. | `D-310`, `D-311` |
| `IRIS-V1-RUNTIME-V088` | negative | interpreter required; JIT required; native not applicable | external, subclass, Dynamic, and reflection sends to private `A#p` | `MethodVisibilityError`; declaring Class code succeeds. | `D-313`, `D-356`, `D-444` |
| `IRIS-V1-RUNTIME-V089` | positive | interpreter required; JIT required; native not applicable | composed `M#bump` reads/writes `@x` with and without private edge authority | Both update `@x`; only private selector access changes. | `D-315`, `D-317`, `D-318` |
| `IRIS-V1-RUNTIME-V090` | positive | interpreter required; JIT required; native not applicable | `class C mixin A,B`; `B mixin A`; each defines `trace` | `C.new().trace()` is `:B`; closed `A` appears once in MRO. | `D-319`, `D-320` |
| `IRIS-V1-RUNTIME-V091` | positive | interpreter required; JIT required; native not applicable; reason: controlled storage fixture has no native boundary | `fixture: {source: "A.set_shared(1); B.set_shared(2); A.set_own(:a); B.set_own(:b); [A.shared(), B.shared(), A.own(), B.own()]", setup: "A declares @@x; B extends A; set_shared/shared lexically access @@x; set_own/own access each Class object's @x"}` | `value`: `[Integer(2), Integer(2), :a, :b]`; subclass `@@x` redeclaration is rejected. | `D-430`, `D-435`, `D-436` |
| `IRIS-V1-RUNTIME-V092` | positive | interpreter required; JIT required; native not applicable; reason: controlled construction fixture has no native boundary | `fixture: {source: "Child.new().log", setup: "Base stored-property initializer appends :base; Child stored-property initializer appends :child; Child#initialize appends :initialize without super"}` | `value`: `[:base, :child, :initialize]`; reflection reports explicit getter and setter Methods for both stored properties. | `D-445`, `D-446` |
| `IRIS-V1-RUNTIME-V093` | negative | interpreter required; JIT not applicable; native not applicable; reason: parse/runtime boundary | bare `super`; explicit `super()` with no successor | bare form is rejected; explicit call raises `NoSuperMethodError`. | `D-447` |
| `IRIS-V1-RUNTIME-V094` | positive | interpreter required; JIT required; native not applicable | alias `f` to `g`, remove `g`, then undef `f` above ancestor | Alias shares Method identity; remove exposes ancestor; undef reaches missing-message path. | `D-448` |
| `IRIS-V1-RUNTIME-V095` | positive | interpreter required; JIT required; native not applicable | `class fun build(){:class}; A.build()` and inherited Class-object send | singleton Method is selected; lookup continues along Class-object chain. | `D-451` |
| `IRIS-V1-RUNTIME-V096` | positive | interpreter required; JIT required; native not applicable | property `name` getter returns `:get`; setter returns `:set`; evaluate `a.name` and `a.name=1` | Getter and setter selectors are `name` and `name=`; results are `:get` and `:set`. | `D-343` |
| `IRIS-V1-RUNTIME-V097` | positive | interpreter required; JIT required; native not applicable | custom `to_bool` increments a counter then returns true; evaluate `if value { :yes }` | counter is `1`; result is `:yes`; `nil`, `false`, and `true` use false, false, true. | `D-350` |
| `IRIS-V1-RUNTIME-V098` | negative | interpreter required; JIT required; native not applicable | `to_bool` raises `:sentinel` during `if`, logical-and, and logical-or assignment | `:sentinel` propagates unchanged; branch, RHS, and writeback do not run. | `D-352` |
| `IRIS-V1-RUNTIME-V099` | positive | interpreter required; JIT required; native not applicable | missing `obj.missing(1) { :block }` with `method_missing` recording inputs | called once with Symbol `:missing`, snapshot `[1]`, and the Closure; missing `method_missing` raises direct `MessageNotFoundError`. | `D-354` |
| `IRIS-V1-RUNTIME-V100` | positive | interpreter required; JIT required; native not applicable; reason: controlled runtime-superclass fixture has no native boundary | `fixture: {setup: "instance is created from A; NewBase satisfies A's static superclass bound", action: "commit A runtime-superclass change to NewBase", source: "[instance is NewBase, type_of(A).subtype?(type_of(NewBase)), Reflection::Class.ancestors(A)]"}` | `value`: `[true, true, [A, NewBase, Object]]`. | `D-262` |
| `IRIS-V1-RUNTIME-V101` | positive | interpreter required; JIT required; native not applicable; reason: controlled built-in Class open has no native boundary | `fixture: {actions: ["compose Module Marker into Nil", "add mark() returning :bool to Bool", "add mark() returning :integer to Integer"], source: "[nil.marker(), true.mark(), 1.mark(), nil same? nil]"}` | `value`: `[:nil, :bool, :integer, true]`; numeric receiver-state creation still raises `InstanceStateError`. | `D-285`, `D-286` |
| `IRIS-V1-RUNTIME-V102` | positive | interpreter required; JIT required; native not applicable | `5.mod(2); 5 mod 2; 5 % 2` | Method and named infix both return `Integer(1)`; `%` is rejected by grammar. | `D-024` |
| `IRIS-V1-RUNTIME-V103` | positive | interpreter required; JIT required; native not applicable | `Float64.infinity` getter replacement, then add a setter that records its argument | Replaced getter result is observed; setter records its argument without creating implicit backing storage. | `D-053`, `D-054` |
| `IRIS-V1-RUNTIME-V104` | positive | interpreter required; JIT required; native not applicable | `Float32.nan + 1.0f32; Float64.infinity - Float64.infinity` | Both produce quiet NaN at the receiver/common width; payload and sign are not asserted. | `D-062` |
| `IRIS-V1-RUNTIME-V105` | positive | interpreter required; JIT required; native not applicable; reason: pure Iris numeric comparisons have no native boundary | `[9007199254740993 < 9007199254740992.0f64, 1.5f32 <=> 1.5f64, Float64.infinity > 10 ** 1000, Float64(-Infinity) < Integer(-10), Float64.nan <=> 0]` | `value`: `[false, Integer(0), true, true, nil]`; reverse-order finite comparisons are complementary. | `D-088`, `D-092`, `D-095` |
| `IRIS-V1-RUNTIME-V106` | positive | interpreter required; JIT required; native not applicable; reason: controlled numeric Method open has no native boundary | `fixture: {actions: ["replace Integer#<=> with a Method returning 1", "observe 1 < 2 and 1 > 2", "replace Integer#== with a Method returning true", "observe 1 == 2 and 1 > 2"]}` | `value`: `[false, true, true, true]`; separately replaced `==` does not replace `<=>`. | `D-089`, `D-091` |
| `IRIS-V1-RUNTIME-V107` | negative | interpreter required; JIT required; native not applicable; reason: the Dynamic runtime return-Contract boundary has no native API boundary | `fixture: {source: "Dynamic<Probe>(Probe.new()) == Object.new()", setup: "Probe#<=> is declared to return Dynamic<Object> and returns true"}` | `error`: `TypeError`, phase `runtime`; no Bool comparison result. | `D-093`, `D-094` |
| `IRIS-V1-RUNTIME-V108` | negative | interpreter required; JIT not applicable; native not applicable; reason: declaration validation precedes executable or native dispatch | `fixture: {source: "class Child extends Base {}", setup: "Base active revision denies subclass creation through MetaCapabilities"}` | `error`: meta-operation exception, phase `declaration validation`; `side_effects`: `Child` is not published. | `D-450` |
| `IRIS-V1-RUNTIME-V109` | positive | interpreter required; JIT required; native not applicable; reason: pure Iris identity source has no native boundary | `fixture: {source: "left() same? right()", setup: "left and right each append once and return the same Object; Object#== and Object#<=> raise if called"}` | `value`: `true`; `side_effects`: log is exactly `[:left, :right]` and neither replaceable comparison Method is called. | `D-098` |
| `IRIS-V1-RUNTIME-V110` | negative | interpreter required; JIT not applicable; native not applicable; reason: declarations of core control-flow spellings are rejected before execution | `fixture: {independent_sources: ["class A { fun !(value) { value } }", "class A { fun &&(value) { value } }", "class A { fun \|\|(value) { value } }", "class A { fun &&=(value) { value } }", "class A { fun \|\|=(value) { value } }"]}` | `error`: declaration-validation rejection for each source; `side_effects`: no Method slot is installed. | `D-351` |
| `IRIS-V1-RUNTIME-V111` | negative | interpreter required; JIT not applicable; native not applicable; reason: named declaration rebinding is static validation | `fixture: {independent_sources: ["class A {}; A = class {}", "module M {}; M = module {}", "contract C {}; C = contract {}", "const K = Object.new(); K = Object.new()"]}` | `error`: static rebinding rejection for each source; `side_effects`: no binding changes. | `D-449` |
| `IRIS-V1-RUNTIME-V036` | positive | interpreter required; JIT required; native not applicable; reason: pure Iris numeric source has no native boundary | `Float64.from_bits(0x0000000000000000) == Float64.from_bits(0x8000000000000000); Float64.from_bits(0x0000000000000000).hash == Float64.from_bits(0x8000000000000000).hash` | `value`: `true`, then `true`; positive and negative zero have equal public hashes. | `D-016` |
| `IRIS-V1-RUNTIME-V039` | positive | interpreter required; JIT required; native not applicable; reason: pure Iris integer source has no native boundary | `[-5 div 2, 5 div -2, -5 div -2]` | `value`: `[Integer(-3), Integer(-3), Integer(2)]`. | `D-023` |
| `IRIS-V1-RUNTIME-V040` | positive | interpreter required; JIT required; native not applicable; reason: pure Iris integer source has no native boundary | `[-5 mod 2, 5 mod -2, -5 mod -2]` | `value`: `[Integer(1), Integer(-1), Integer(-1)]`. | `D-025` |
| `IRIS-V1-RUNTIME-V041` | negative | interpreter required; JIT required; native not applicable; reason: pure Iris exponentiation source has no native boundary | `fixture: {independent_sources: ["0 ** 0", "0.0f32 ** 0.0f32", "Float64.from_bits(0x8000000000000000) ** -0.0f64"]}` | `error`: each source raises `DomainError`, phase `runtime`; no numeric value is returned. | `D-027`, `D-029` |
| `IRIS-V1-RUNTIME-V112` | positive | interpreter required; JIT required; native not applicable; reason: pure Iris integer exponentiation has no native boundary | `type_and_value(0 ** -1)` | `value`: positive infinity; `type`: `Float64`; no `DivisionByZeroError` or `DomainError`. | `D-028` |
| `IRIS-V1-RUNTIME-V042` | positive | interpreter required; JIT required; native not applicable; reason: pure Iris float exponentiation has no native boundary | `type_and_value((-2.0f64) ** 0.5f64)` | `value`: quiet NaN; `type`: `Float64`; no `DomainError`. | `D-031` |
| `IRIS-V1-RUNTIME-V043` | positive | interpreter required; JIT required; native not applicable; reason: pure Iris integer bitwise source has no native boundary | `[~0, -3 >> 1, 8 << -2]` | `value`: `[Integer(-1), Integer(-2), Integer(2)]`; built-in bitwise semantics use infinite sign-extended two's complement. | `D-034`, `D-035` |
| `IRIS-V1-RUNTIME-V048` | positive | interpreter required; JIT required; native not applicable; reason: pure Iris fused arithmetic has no native boundary | `type_and_value(Float64.infinity.mul_add(1, Float64(-Infinity)))` | `value`: quiet NaN; `type`: `Float64`; no Iris exception. | `D-061` |
| `IRIS-V1-RUNTIME-V019` | positive | interpreter required; JIT required; native not applicable; reason: pure Iris singleton identity source has no native boundary | `[nil same? nil, true same? true, false same? false]` | `value`: `[true, true, true]`; all three operands are identity-bearing singletons. | `D-099` |
| `IRIS-V1-RUNTIME-V011` | positive | interpreter required; JIT required; native not applicable; reason: controlled Class open has no native boundary | `fixture: {source: "let before = A; open class A { public fun added() { :added } }; [before same? A, A.added()]", setup: "A is a named logical Class"}` | `value`: `[true, :added]`; reopen publishes a new active revision without replacing Class identity. | `D-100` |
| `IRIS-V1-RUNTIME-V012` | positive | interpreter required; JIT required; native not applicable; reason: controlled Method replacement has no native boundary | `fixture: {source: "let value = A.new(); open class A { override public fun m() { :new } }; value.m()", setup: "A#m initially returns :old"}` | `value`: `:new`; the existing instance's future send selects the replacement Method. | `D-101` |
| `IRIS-V1-RUNTIME-V013` | positive | interpreter required; JIT required; native not applicable; reason: deterministic Method-frame scheduler fixture has no native boundary | `fixture: {setup: "A#m enters and pauses before returning :old", schedule: ["enter value.m()", "commit replacement A#m returning :new", "resume entered frame", "call value.m() again"]}` | `value`: `[:old, :new]`; the entered frame keeps its selected body and the later send selects the replacement. | `D-101` |
| `IRIS-V1-RUNTIME-V015` | negative | interpreter required; JIT required; native not applicable; reason: controlled Method/MRO fixture has no native boundary | `fixture: {setup: "retain Module M Method m whose body executes super(); receiver current MRO initially contains M", actions: ["remove M from receiver Class MRO", "invoke retained Method on receiver"]}` | `error`: `MethodBindingError`, phase `runtime`, because `IRIS-V1-RUNTIME-C015` entry validation precedes the `super` path in `IRIS-V1-RUNTIME-C014`. | `D-102`, `D-103` |
| `IRIS-V1-RUNTIME-V014` | negative | interpreter required; JIT required; native not applicable; reason: controlled reflective Method fixture has no native boundary | `fixture: {setup: "retain Method A#m", actions: ["change receiver Class MRO so A is absent", "reflectively invoke retained Method on receiver"]}` | `error`: `MethodBindingError` at invocation entry; the Method body does not execute. | `D-104` |
| `IRIS-V1-RUNTIME-V016` | positive | interpreter required; JIT required; native not applicable; reason: pure Iris BoundMethod identity source has no native boundary | `obj.method same? obj.method` | `value`: `false`; each Method read creates a distinct BoundMethod identity. | `D-106` |
| `IRIS-V1-RUNTIME-V017` | positive | interpreter required; JIT required; native not applicable; reason: controlled BoundMethod replacement has no native boundary | `fixture: {source: "let saved = obj.method; open class A { override public fun method() { :new } }; [saved same? saved, saved(), obj.method()]", setup: "obj is an A and original A#method returns :old"}` | `value`: `[true, :old, :new]`; the saved BoundMethod retains its Method identity while later lookup selects the replacement. | `D-105`, `D-106` |
| `IRIS-V1-RUNTIME-V026` | negative | interpreter required; JIT required; native not applicable; reason: controlled construction failure has no native boundary | `fixture: {source: "let escaped = nil; class A { public fun initialize() { escaped = self; raise :sentinel } }; A.new()", observations_after_catch: ["escaped is A", "escaped.to_bool()"]}` | `error`: raised value `:sentinel`, phase `runtime`; `side_effects`: `new` returns no instance, while escaped is an ordinary memory-safe `A` and `escaped.to_bool()` is `true`. | `D-443` |
