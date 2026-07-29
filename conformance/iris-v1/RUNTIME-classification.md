# RUNTIME Vector Classification

This document classifies the 109 unique committed `IRIS-V1-RUNTIME` vectors in chapter 03 at `spec/iris-v1/03-runtime-object-model.md`. The chapter contains 125 table rows: 10 unique Stable Hash rows at lines 418-427, 36 overview rows at lines 526-561, and 79 detailed Runtime Coverage rows at lines 570-648. Sixteen IDs occur in both the overview and coverage tables (`V011`-`V017`, `V019`, `V026`, `V036`, `V039`-`V043`, and `V048`). Each is one vector represented at two granularities, not two definitions. Thus `10 + 36 + 79 - 16 = 109` unique IDs.

`executable` means that the chapter gives a concrete controlled fixture or directly assertable runtime value/bit/hash artifact and a concrete observable within the milestone 2 object-kernel boundary. `no-fixture` means the overview gives only an incomplete prose scenario. `needs-subsystem` means the vector is concrete but its required observable depends on a subsystem excluded from that boundary. `differential` requires the interpreter/JIT comparison that does not yet exist.

**The `needs-subsystem` reasons below were written before milestone 2 and several are now stale.** Array literals and the `Reflection::*` namespace are implemented, so any reason citing "the excluded collections library", "Array literals", or an excluded reflection API must be re-probed rather than trusted. `V105` was reclassified `executable` on exactly those grounds, and probing it uncovered a real `IRIS-V1-RUNTIME-C131` exactness defect that the stale bucket had been hiding from the suite. Rows still citing collections or an excluded reflection API at the time of writing are `V068`, `V075`, `V076`, `V091`, `V092`, and `V101`; each states a concrete source and a concrete expected aggregate, so each is a candidate for the same treatment once the built-in protocols it sends are in place. `V072` is NOT in that group: it needs a Hash literal, which genuinely is unimplemented.

| Vector ID | Chapter 03 line(s) | Category | Bucket | Concrete runtime artifact or reason |
| --- | --- | --- | --- | --- |
| `IRIS-V1-RUNTIME-V001` | `:418` | stable hash | executable | Canonical numeric bytes `0000`, BLAKE3 digest prefix, and public Integer hash for zero. |
| `IRIS-V1-RUNTIME-V002` | `:419` | stable hash | executable | Canonical bytes and exact public hash for mathematical one across numeric widths. |
| `IRIS-V1-RUNTIME-V003` | `:420` | stable hash | executable | Canonical bytes and exact public hash for mathematical negative one. |
| `IRIS-V1-RUNTIME-V004` | `:421` | stable hash | executable | Canonical bytes and exact public hash for mathematical two. |
| `IRIS-V1-RUNTIME-V005` | `:422` | stable hash | executable | Canonical bytes and exact public hash for exact `3/2`. |
| `IRIS-V1-RUNTIME-V006` | `:423` | stable hash | executable | Canonical infinity byte and exact public Integer hash. |
| `IRIS-V1-RUNTIME-V007` | `:424` | stable hash | executable | Canonical negative-infinity byte and exact public Integer hash. |
| `IRIS-V1-RUNTIME-V008` | `:425` | stable hash | executable | Singleton input byte and exact `nil` public hash. |
| `IRIS-V1-RUNTIME-V009` | `:426` | stable hash | executable | Singleton input byte and exact `false` public hash. |
| `IRIS-V1-RUNTIME-V010` | `:427` | stable hash | executable | Singleton input byte and exact `true` public hash. |
| `IRIS-V1-RUNTIME-V011` | `:526`, `:641` | positive | executable | Controlled Class-open fixture asserts `[before same? A, A.added()] == [true, :added]`. |
| `IRIS-V1-RUNTIME-V012` | `:527`, `:642` | positive | executable | Existing instance sends the replacement Method and returns `:new`. |
| `IRIS-V1-RUNTIME-V013` | `:528`, `:643` | positive | executable | Deterministic frame fixture asserts entered old body then subsequent new body: `[:old, :new]`. |
| `IRIS-V1-RUNTIME-V014` | `:529`, `:645` | negative | executable | Retained Method invocation after owner removal raises `MethodBindingError` before body entry. |
| `IRIS-V1-RUNTIME-V015` | `:530`, `:644` | negative | executable | Retained Method whose `super` owner left current MRO raises `InvalidSuperError`. |
| `IRIS-V1-RUNTIME-V016` | `:531`, `:646` | positive | executable | `obj.method same? obj.method` returns `false` for fresh BoundMethods. |
| `IRIS-V1-RUNTIME-V017` | `:532`, `:647` | positive | executable | Saved BoundMethod fixture asserts `[true, :old, :new]`. |
| `IRIS-V1-RUNTIME-V018` | `:533` | failure | executable | Inline runnable expression `Integer(1) same? Integer(1)` must raise `IdentityError`. |
| `IRIS-V1-RUNTIME-V019` | `:534`, `:640` | positive | executable | Concrete singleton source asserts `[true, true, true]` for `nil`, `true`, and `false`. |
| `IRIS-V1-RUNTIME-V020` | `:535` | positive | no-fixture | Names distinct qualified and ordinary bodies but supplies neither declarations nor receiver/setup. |
| `IRIS-V1-RUNTIME-V021` | `:536` | failure | no-fixture | Names a missing Contract slot but supplies no Contract, receiver, or source fixture. |
| `IRIS-V1-RUNTIME-V022` | `:537` | failure | no-fixture | Gives only a visibility scenario, without a Class, caller, or source fixture. |
| `IRIS-V1-RUNTIME-V023` | `:538` | positive | no-fixture | Describes two Module composition forms but supplies no Module declarations or runnable program. |
| `IRIS-V1-RUNTIME-V024` | `:539` | positive | no-fixture | Describes nested Module deduplication without concrete Module/Class fixture. |
| `IRIS-V1-RUNTIME-V025` | `:540` | positive | no-fixture | Describes concurrent construction/revision timing without deterministic fixture or schedule. |
| `IRIS-V1-RUNTIME-V026` | `:541`, `:648` | negative | executable | Controlled `initialize`-escape fixture checks sentinel propagation and a memory-safe escaped `A`. |
| `IRIS-V1-RUNTIME-V027` | `:542` | positive | no-fixture | Assignment text lacks declarations of `a`, `b`, setters, and the marker fixture. |
| `IRIS-V1-RUNTIME-V028` | `:543` | positive | no-fixture | Missing-ivar scenario supplies no receiver Class/object source. |
| `IRIS-V1-RUNTIME-V029` | `:544` | failure | no-fixture | Numeric ivar-write scenario has no concrete expression or fixture. |
| `IRIS-V1-RUNTIME-V030` | `:545` | positive | no-fixture | `side_effect()` is unspecified, so the terse expression is not runnable as written. |
| `IRIS-V1-RUNTIME-V031` | `:546` | positive | no-fixture | `side_effect()` is unspecified, so the terse expression is not runnable as written. |
| `IRIS-V1-RUNTIME-V032` | `:547` | failure | no-fixture | No value/Class implementing non-Bool `to_bool` is provided. |
| `IRIS-V1-RUNTIME-V033` | `:548` | positive | no-fixture | Describes method removal and fallback but provides no Class/object fixture. |
| `IRIS-V1-RUNTIME-V034` | `:549` | failure | no-fixture | Names an arity mismatch without selector declaration, receiver, or call source. |
| `IRIS-V1-RUNTIME-V035` | `:550` | positive | executable | Inline expressions assert `Float64.nan == Float64.nan` is `false` and `!=` is `true`. |
| `IRIS-V1-RUNTIME-V036` | `:551`, `:632` | positive | executable | Concrete bit-source asserts positive/negative zero equality and equal public hashes. |
| `IRIS-V1-RUNTIME-V037` | `:552` | failure | no-fixture | Hash insertion is named but no Hash/container fixture or concrete NaN source is supplied. |
| `IRIS-V1-RUNTIME-V038` | `:553` | positive | no-fixture | `bits` is a placeholder, not a concrete signaling-NaN bit pattern. |
| `IRIS-V1-RUNTIME-V039` | `:554`, `:633` | positive | executable | Source asserts floor-division results `[Integer(-3), Integer(-3), Integer(2)]`. |
| `IRIS-V1-RUNTIME-V040` | `:555`, `:634` | positive | executable | Source asserts modulo results `[Integer(1), Integer(-1), Integer(-1)]`. |
| `IRIS-V1-RUNTIME-V041` | `:556`, `:635` | negative | executable | Three concrete zero-exponent sources each raise `DomainError`. |
| `IRIS-V1-RUNTIME-V042` | `:557`, `:637` | positive | executable | Concrete negative finite float source produces quiet `Float64` NaN. |
| `IRIS-V1-RUNTIME-V043` | `:558`, `:638` | positive | executable | Source asserts infinite-sign two's-complement bitwise results. |
| `IRIS-V1-RUNTIME-V044` | `:559` | failure | no-fixture | Transaction group, protected Class, and mutation operation are prose only. |
| `IRIS-V1-RUNTIME-V045` | `:560` | positive | no-fixture | References V001-V010 expected outputs but supplies no executable aggregate source/fixture. |
| `IRIS-V1-RUNTIME-V046` | `:579` | positive | executable | Concrete integer division source returns `Float64(2.5)` and `Float64(2.0)`. |
| `IRIS-V1-RUNTIME-V047` | `:587` | positive | executable | Concrete fused-arithmetic sources yield NaN at each float width. |
| `IRIS-V1-RUNTIME-V048` | `:561`, `:639` | positive | executable | Concrete infinity `mul_add` source returns quiet `Float64` NaN without exception. |
| `IRIS-V1-RUNTIME-V052` | `:570` | differential | differential | Requires interpreter and JIT agreement on arbitrary-precision integer representation. |
| `IRIS-V1-RUNTIME-V053` | `:571` | positive | needs-subsystem | `type_of` observations require the excluded reflection/type-introspection API. |
| `IRIS-V1-RUNTIME-V054` | `:572` | negative | needs-subsystem | Requires lexer/parser diagnostic subsystem for `LEX_BAD_FLOAT_SUFFIX`. |
| `IRIS-V1-RUNTIME-V055` | `:573` | positive | executable | Controlled dispatch/replacement fixture asserts values and exact evaluation log. |
| `IRIS-V1-RUNTIME-V056` | `:574` | positive | executable | Concrete mixed-width arithmetic source asserts rounded values and result widths. |
| `IRIS-V1-RUNTIME-V057` | `:575` | positive | executable | Concrete float division source asserts signed infinities and NaN without Iris error. |
| `IRIS-V1-RUNTIME-V058` | `:576` | positive | executable | Concrete max-finite multiplication source asserts positive infinity by width. |
| `IRIS-V1-RUNTIME-V059` | `:577` | positive | executable | Concrete NaN comparison source asserts `false`, `false`, and `nil`. |
| `IRIS-V1-RUNTIME-V060` | `:578` | negative | executable | Concrete integer division/div/mod zero sources each raise `DivisionByZeroError`. |
| `IRIS-V1-RUNTIME-V061` | `:580` | positive | executable | Class fixture asserts ordinary and named-infix sends select the same `scale` Method. |
| `IRIS-V1-RUNTIME-V062` | `:581` | positive | executable | Concrete exponentiation source asserts integer, reciprocal Float64, and negative Float32 infinity. |
| `IRIS-V1-RUNTIME-V063` | `:582` | positive | executable | Concrete large/negative-shift source asserts exact Integer results. |
| `IRIS-V1-RUNTIME-V064` | `:583` | negative | needs-subsystem | Requires excluded resource-limit/quota harness to observe `ResourceError` or `MemoryLimitError`. |
| `IRIS-V1-RUNTIME-V065` | `:584` | positive | executable | Concrete special-value sends assert canonical values/types and absent bare names. |
| `IRIS-V1-RUNTIME-V066` | `:585` | differential | differential | Explicitly compares interpreter and JIT under controlled host rounding mode. |
| `IRIS-V1-RUNTIME-V067` | `:586` | positive | executable | Concrete `mul_add` and multiply-then-add bit results assert one-rounding distinction. |
| `IRIS-V1-RUNTIME-V068` | `:588` | positive | needs-subsystem | The concrete source/result is an Array literal, requiring the excluded collections library. |
| `IRIS-V1-RUNTIME-V069` | `:589` | negative | executable | Concrete out-of-range `from_bits` calls each raise `RangeError`. |
| `IRIS-V1-RUNTIME-V070` | `:590` | negative | executable | Three controlled sources assert numeric ivar `InstanceStateError` and numeric identity `IdentityError`. |
| `IRIS-V1-RUNTIME-V071` | `:591` | positive | executable | Concrete numeric equality source asserts exactness boundary and equal numeric hashes. |
| `IRIS-V1-RUNTIME-V072` | `:592` | negative | needs-subsystem | The required `%{ Float64.nan: 1 }` assertion needs the excluded Hash collections library. |
| `IRIS-V1-RUNTIME-V073` | `:593` | differential | differential | Explicitly requires exact public hashes to agree between interpreter and JIT. |
| `IRIS-V1-RUNTIME-V074` | `:594` | negative | executable | Concrete unavailable-selector send asserts `MessageNotFoundError`. |
| `IRIS-V1-RUNTIME-V075` | `:595` | positive | needs-subsystem | Fixture's asserted value is an Iris Array literal, requiring collections. |
| `IRIS-V1-RUNTIME-V076` | `:596` | positive | needs-subsystem | Concrete source and asserted aggregate. The earlier collections reason is stale: Array literals are implemented and the `C091`/`C092` singleton comparison protocols this row needs were implemented in milestone 2. It now needs only `Object` as an instantiable root Class for its final term `nil < Object.new()`. |
| `IRIS-V1-RUNTIME-V077` | `:597` | positive | needs-subsystem | Requires the callable/Closure subsystem for `closure()` identity/equality. |
| `IRIS-V1-RUNTIME-V078` | `:598` | positive | needs-subsystem | Requires Contract and Type objects from the excluded contracts/generics type system. |
| `IRIS-V1-RUNTIME-V079` | `:599` | positive | needs-subsystem | Requires the excluded compacting-GC subsystem and its controlled compaction hook. |
| `IRIS-V1-RUNTIME-V080` | `:600` | negative | needs-subsystem | Requires declaration validation against the excluded contracts/generics type system. |
| `IRIS-V1-RUNTIME-V081` | `:601` | positive | needs-subsystem | Qualified Contract dispatch requires the excluded contracts/generics type system. |
| `IRIS-V1-RUNTIME-V082` | `:602` | positive | needs-subsystem | Required invariant includes preserved Contract identity, requiring the excluded contracts system. |
| `IRIS-V1-RUNTIME-V083` | `:603` | positive | executable | Controlled construction/revision schedule asserts captured construction revision and active later dispatch. |
| `IRIS-V1-RUNTIME-V084` | `:604` | negative | executable | Two concrete absent-selector sends each raise `MessageNotFoundError`. |
| `IRIS-V1-RUNTIME-V085` | `:605` | positive | needs-subsystem | Requires reflection/meta-operation API to invoke `migrate_revision` and reactivate a revision. |
| `IRIS-V1-RUNTIME-V086` | `:606` | positive | needs-subsystem | Required `Dynamic<Object>` slot-type observation needs the excluded contracts/type system. |
| `IRIS-V1-RUNTIME-V087` | `:607` | positive | needs-subsystem | Requires the callable/Closure subsystem for escaped Closure capture and rebinding. |
| `IRIS-V1-RUNTIME-V088` | `:608` | negative | needs-subsystem | Required Dynamic and reflection sends need excluded type/reflection APIs. |
| `IRIS-V1-RUNTIME-V089` | `:609` | positive | executable | Concrete composed-Module ivar fixture asserts state access and private-edge behavior. |
| `IRIS-V1-RUNTIME-V090` | `:610` | positive | executable | Concrete Module/Class fixture asserts `:B` dispatch and a single closed `A` MRO entry. |
| `IRIS-V1-RUNTIME-V091` | `:611` | positive | needs-subsystem | Asserted result is an Array literal, requiring collections. |
| `IRIS-V1-RUNTIME-V092` | `:612` | positive | needs-subsystem | Uses a collection-backed construction log and requires excluded reflection API observations. |
| `IRIS-V1-RUNTIME-V093` | `:613` | negative | executable | Concrete bare/explicit `super` cases assert parse rejection and `NoSuperMethodError`. |
| `IRIS-V1-RUNTIME-V094` | `:614` | positive | executable | Concrete alias/remove/undef sequence asserts shared Method identity, ancestor exposure, and missing-message. |
| `IRIS-V1-RUNTIME-V095` | `:615` | positive | executable | Concrete Class-object Method source asserts singleton selection and Class-object-chain lookup. |
| `IRIS-V1-RUNTIME-V096` | `:616` | positive | executable | Concrete property fixture asserts getter `name`, setter `name=`, and `:get`/`:set` values. |
| `IRIS-V1-RUNTIME-V097` | `:617` | positive | executable | Concrete truth test asserts one `to_bool` send and singleton truthiness. |
| `IRIS-V1-RUNTIME-V098` | `:618` | negative | needs-subsystem | Requires excluded control-flow/assignment evaluator semantics for `if`, `&&=`, and `||=` propagation. |
| `IRIS-V1-RUNTIME-V099` | `:619` | positive | needs-subsystem | Required block argument is a Closure, requiring the callable/Closure subsystem. |
| `IRIS-V1-RUNTIME-V100` | `:620` | positive | needs-subsystem | Requires excluded reflection/type APIs and Array result to assert runtime-superclass reflection. |
| `IRIS-V1-RUNTIME-V101` | `:621` | positive | needs-subsystem | Concrete source/result uses Array literals, requiring collections despite kernel-relevant built-in openness. |
| `IRIS-V1-RUNTIME-V102` | `:622` | positive | needs-subsystem | Required `%` rejection needs lexer/parser diagnostic support outside the object kernel. |
| `IRIS-V1-RUNTIME-V103` | `:623` | positive | executable | Concrete special-value getter/setter replacement fixture observes replacement and no implicit storage. |
| `IRIS-V1-RUNTIME-V104` | `:624` | positive | executable | Concrete NaN arithmetic source asserts quiet NaN at receiver/common width. |
| `IRIS-V1-RUNTIME-V105` | `:625` | positive | executable | Concrete mixed-numeric comparison source with a concrete asserted aggregate. Reclassified in milestone 2: the earlier `needs-subsystem` reason cited an excluded collections library, but Array literals are implemented, and probing this row exposed a real `C131` exactness defect in `Numeric::compare`. |
| `IRIS-V1-RUNTIME-V106` | `:626` | positive | executable | Controlled built-in Method replacements assert independent `<=>` and `==` behavior. |
| `IRIS-V1-RUNTIME-V107` | `:627` | negative | needs-subsystem | Requires excluded Dynamic return-Contract enforcement from contracts/generics. |
| `IRIS-V1-RUNTIME-V108` | `:628` | negative | executable | Controlled MetaCapabilities validation asserts no Child publication after denied subclass creation. |
| `IRIS-V1-RUNTIME-V109` | `:629` | positive | executable | Controlled side-effect fixture asserts `same?` is true without calling replaceable comparison Methods. |
| `IRIS-V1-RUNTIME-V110` | `:630` | negative | needs-subsystem | Requires parser/declaration-validation subsystem to reject reserved control-flow selector declarations. |
| `IRIS-V1-RUNTIME-V111` | `:631` | negative | needs-subsystem | Requires parser/static-binding declaration-validation subsystem. |
| `IRIS-V1-RUNTIME-V112` | `:636` | positive | executable | Concrete `0 ** -1` source asserts positive `Float64` infinity without error. |

## Classification Totals

| Classification | Count |
| --- | ---: |
| executable | 60 |
| no-fixture | 18 |
| needs-subsystem | 28 |
| differential | 3 |
| unclear | 0 |
| **Total** | **109** |

The bucket sum is explicitly `60 + 18 + 28 + 3 + 0 = 109`. The realistic milestone 2 executable target is **60** vectors, not 109.

## Overview-Only Vectors

Twenty IDs occur only in the terse Runtime Conformance Vectors overview rather than the detailed coverage table: `V018`, `V020`-`V025`, `V027`-`V035`, `V037`, `V038`, `V044`, and `V045`.

| Vector ID | Status | Reason |
| --- | --- | --- |
| `V018` | executable | Its inline `Integer(1) same? Integer(1)` expression and `IdentityError` expectation are complete. |
| `V020`-`V025` | no-fixture | Each names behavior but omits the declarations, objects, schedule, or setup needed to run it. |
| `V027`-`V034` | no-fixture | Each is an incomplete scenario: required receivers, setters, calls, methods, or setup are absent. |
| `V035` | executable | Its two inline NaN comparisons have exact Boolean expectations. |
| `V037`-`V038` | no-fixture | Hash insertion has no container fixture; `bits` is a placeholder rather than a value. |
| `V044`-`V045` | no-fixture | The transaction is prose-only; V045 references other rows rather than providing an aggregate fixture. |

## Dependency Grouping

Build the `executable` vectors in this dependency order. Vectors classified `needs-subsystem`, `differential`, or `no-fixture` are deliberately excluded from this milestone build-order grouping.

| Object-model capability | Executable vectors |
| --- | --- |
| value foundation | `V018`, `V019`, `V070` |
| logical Class + active revision | `V011`, `V012`, `V013`, `V014`, `V017`, `V026`, `V083`, `V106` |
| dispatch/selector | `V016`, `V055`, `V061`, `V074`, `V084`, `V094`, `V095`, `V103`, `V109` |
| MRO/module composition | `V015`, `V089`, `V090` |
| construction lifecycle | `V026`, `V083` |
| properties/ivars/class-vars | `V070`, `V089`, `V096`, `V103` |
| visibility/super | `V093` |
| equality/ordering/hashing/identity | `V001`-`V010`, `V018`, `V019`, `V035`, `V036`, `V071`, `V094`, `V109` |
| truthiness/missing-message | `V097` |
| numeric model | `V035`, `V036`, `V039`-`V043`, `V046`-`V048`, `V056`-`V063`, `V065`, `V067`, `V069`, `V071`, `V104`, `V112` |
| stable hashing | `V001`-`V010`, `V036`, `V071` |
| meta safety | `V108` |

The repeated appearances in the grouping denote true dependencies, not additional vectors. The recommended implementation order is: value foundation; numeric model and stable hashing; logical Class/revisions; dispatch; MRO/module composition; construction; properties; visibility/super; equality/identity; truthiness/missing-message; meta safety.
