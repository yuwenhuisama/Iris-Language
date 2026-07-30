# RUNTIME Vector Classification

This document classifies the 109 unique committed `IRIS-V1-RUNTIME` vectors in chapter 03 at `spec/iris-v1/03-runtime-object-model.md`. The chapter contains 125 table rows: 10 unique Stable Hash rows at lines 418-427, 36 overview rows at lines 526-561, and 79 detailed Runtime Coverage rows at lines 570-648. Sixteen IDs occur in both the overview and coverage tables (`V011`-`V017`, `V019`, `V026`, `V036`, `V039`-`V043`, and `V048`). Each is one vector represented at two granularities, not two definitions. Thus `10 + 36 + 79 - 16 = 109` unique IDs.

`executable` means that the chapter gives a concrete controlled fixture or directly assertable runtime value/bit/hash artifact and a concrete observable within the milestone 2 object-kernel boundary. `no-fixture` means the overview gives only an incomplete prose scenario. `needs-subsystem` means the vector is concrete but its required observable depends on a subsystem excluded from that boundary. `differential` requires the interpreter/JIT comparison that does not yet exist.

**The `needs-subsystem` reasons below were written before milestone 2 and several are now stale.** Array literals and the `Reflection::*` namespace are implemented, so any reason citing "the excluded collections library", "Array literals", or an excluded reflection API must be re-probed rather than trusted. `V105` was reclassified `executable` on exactly those grounds, and probing it uncovered a real `IRIS-V1-RUNTIME-C131` exactness defect that the stale bucket had been hiding from the suite. Every row that cited collections or an excluded reflection API has since been re-probed, and each now states the concrete feature it actually waits on: `V068` and `V105` were reclassified `executable`, `V075` shares the root-Class blocker, `V091` needs `@@x` hierarchy cells, `V092` needs ordered stored-property initialization, `V101` needs built-in Class openness, and `V072` needs a Hash literal. All 21 reasons were systematically re-probed in milestone 2 and rewritten to name the concrete feature each waits on; six were narrowed to a strictly smaller blocker, and two rows turned out to need no implementation at all. The lesson stands: a bucket reason is a claim to re-probe, not a fact, because a bucketed vector is never executed and so can never fail.

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
| `IRIS-V1-RUNTIME-V020` | `:535` | positive | executable | Fixture authored in milestone 2 with owner approval, transcribing qualified and ordinary dispatch verbatim from the frozen overview row. No value the row does not state was invented. |
| `IRIS-V1-RUNTIME-V021` | `:536` | failure | executable | Fixture authored in milestone 2 with owner approval, transcribing a missing qualified slot verbatim from the frozen overview row. No value the row does not state was invented. |
| `IRIS-V1-RUNTIME-V022` | `:537` | failure | executable | Fixture authored in milestone 2 with owner approval, transcribing a private Method without access verbatim from the frozen overview row. No value the row does not state was invented. |
| `IRIS-V1-RUNTIME-V023` | `:538` | positive | executable | Fixture authored in milestone 2 with owner approval, transcribing mixin lookup order verbatim from the frozen overview row. No value the row does not state was invented. |
| `IRIS-V1-RUNTIME-V024` | `:539` | positive | no-fixture | The row asserts that the MRO contains closed `A` once at its nearest occurrence, but `Reflection::Class.ancestors` returns only Class entries and omits Modules, so Module deduplication has no observation path. Authoring it would require inventing a Module-visible ancestry API the chapter does not define. |
| `IRIS-V1-RUNTIME-V025` | `:540` | positive | no-fixture | The row needs construction to capture a revision, a later open to commit, and a send after it, which is the same out-of-band scheduling that `V013` and `V083` wait on and that `01-language-identity.md` lists as PROHIBITED inside a transaction. |
| `IRIS-V1-RUNTIME-V026` | `:541`, `:648` | negative | executable | Controlled `initialize`-escape fixture checks sentinel propagation and a memory-safe escaped `A`. |
| `IRIS-V1-RUNTIME-V027` | `:542` | positive | executable | Fixture authored in milestone 2 with owner approval, transcribing chained setter marker propagation verbatim from the frozen overview row. No value the row does not state was invented. |
| `IRIS-V1-RUNTIME-V028` | `:543` | positive | executable | Fixture authored in milestone 2 with owner approval, transcribing a missing undeclared ivar read verbatim from the frozen overview row. No value the row does not state was invented. |
| `IRIS-V1-RUNTIME-V029` | `:544` | failure | executable | Fixture authored in milestone 2 with owner approval, transcribing identity-less numeric receiver state verbatim from the frozen overview row. No value the row does not state was invented. |
| `IRIS-V1-RUNTIME-V030` | `:545` | positive | executable | Fixture authored in milestone 2 with owner approval, transcribing `&&` short circuit verbatim from the frozen overview row. No value the row does not state was invented. |
| `IRIS-V1-RUNTIME-V031` | `:546` | positive | executable | Fixture authored in milestone 2 with owner approval, transcribing `||` short circuit verbatim from the frozen overview row. No value the row does not state was invented. |
| `IRIS-V1-RUNTIME-V032` | `:547` | failure | executable | Fixture authored in milestone 2 with owner approval, transcribing a non-Bool `to_bool` verbatim from the frozen overview row. No value the row does not state was invented. |
| `IRIS-V1-RUNTIME-V033` | `:548` | positive | executable | Fixture authored in milestone 2 with owner approval, transcribing the `method_missing` truthiness fallback verbatim from the frozen overview row. No value the row does not state was invented. |
| `IRIS-V1-RUNTIME-V034` | `:549` | failure | executable | Fixture authored in milestone 2 with owner approval, transcribing an arity mismatch verbatim from the frozen overview row. No value the row does not state was invented. |
| `IRIS-V1-RUNTIME-V035` | `:550` | positive | executable | Inline expressions assert `Float64.nan == Float64.nan` is `false` and `!=` is `true`. |
| `IRIS-V1-RUNTIME-V036` | `:551`, `:632` | positive | executable | Concrete bit-source asserts positive/negative zero equality and equal public hashes. |
| `IRIS-V1-RUNTIME-V037` | `:552` | failure | executable | Fixture authored in milestone 2 with owner approval, transcribing a NaN Hash key verbatim from the frozen overview row. No value the row does not state was invented. |
| `IRIS-V1-RUNTIME-V038` | `:553` | positive | executable | Fixture authored in milestone 2 with owner approval, transcribing signaling-NaN bit round trip verbatim from the frozen overview row. No value the row does not state was invented. |
| `IRIS-V1-RUNTIME-V039` | `:554`, `:633` | positive | executable | Source asserts floor-division results `[Integer(-3), Integer(-3), Integer(2)]`. |
| `IRIS-V1-RUNTIME-V040` | `:555`, `:634` | positive | executable | Source asserts modulo results `[Integer(1), Integer(-1), Integer(-1)]`. |
| `IRIS-V1-RUNTIME-V041` | `:556`, `:635` | negative | executable | Three concrete zero-exponent sources each raise `DomainError`. |
| `IRIS-V1-RUNTIME-V042` | `:557`, `:637` | positive | executable | Concrete negative finite float source produces quiet `Float64` NaN. |
| `IRIS-V1-RUNTIME-V043` | `:558`, `:638` | positive | executable | Source asserts infinite-sign two's-complement bitwise results. |
| `IRIS-V1-RUNTIME-V044` | `:559` | failure | executable | Fixture authored in milestone 2 with owner approval, transcribing protected built-in superclass mutation verbatim from the frozen overview row. No value the row does not state was invented. |
| `IRIS-V1-RUNTIME-V045` | `:560` | positive | executable | Fixture authored in milestone 2 with owner approval, transcribing the stable numeric hashes verbatim from the frozen overview row. No value the row does not state was invented. |
| `IRIS-V1-RUNTIME-V046` | `:579` | positive | executable | Concrete integer division source returns `Float64(2.5)` and `Float64(2.0)`. |
| `IRIS-V1-RUNTIME-V047` | `:587` | positive | executable | Concrete fused-arithmetic sources yield NaN at each float width. |
| `IRIS-V1-RUNTIME-V048` | `:561`, `:639` | positive | executable | Concrete infinity `mul_add` source returns quiet `Float64` NaN without exception. |
| `IRIS-V1-RUNTIME-V052` | `:570` | differential | differential | Requires interpreter and JIT agreement on arbitrary-precision integer representation. |
| `IRIS-V1-RUNTIME-V053` | `:571` | positive | executable | Concrete float-literal Type observation. Reclassified in milestone 2 using the standing decision that `type_of` is vector observation notation rather than a selector; the runner asserts the runtime Type directly. |
| `IRIS-V1-RUNTIME-V054` | `:572` | negative | executable | Concrete malformed-float source. Reclassified in milestone 2 after the RUNTIME runner gained a `diagnostics` expectation branch reusing the GRAMMAR collector; `LEX_BAD_FLOAT_SUFFIX` was already emitted and merely unobservable from this chapter. |
| `IRIS-V1-RUNTIME-V055` | `:573` | positive | executable | Controlled dispatch/replacement fixture asserts values and exact evaluation log. |
| `IRIS-V1-RUNTIME-V056` | `:574` | positive | executable | Concrete mixed-width arithmetic source asserts rounded values and result widths. |
| `IRIS-V1-RUNTIME-V057` | `:575` | positive | executable | Concrete float division source asserts signed infinities and NaN without Iris error. |
| `IRIS-V1-RUNTIME-V058` | `:576` | positive | executable | Concrete max-finite multiplication source asserts positive infinity by width. |
| `IRIS-V1-RUNTIME-V059` | `:577` | positive | executable | Concrete NaN comparison source asserts `false`, `false`, and `nil`. |
| `IRIS-V1-RUNTIME-V060` | `:578` | negative | executable | Concrete integer division/div/mod zero sources each raise `DivisionByZeroError`. |
| `IRIS-V1-RUNTIME-V061` | `:580` | positive | executable | Class fixture asserts ordinary and named-infix sends select the same `scale` Method. |
| `IRIS-V1-RUNTIME-V062` | `:581` | positive | executable | Concrete exponentiation source asserts integer, reciprocal Float64, and negative Float32 infinity. |
| `IRIS-V1-RUNTIME-V063` | `:582` | positive | executable | Concrete large/negative-shift source asserts exact Integer results. |
| `IRIS-V1-RUNTIME-V064` | `:583` | negative | needs-subsystem | Confirmed in milestone 2. Needs a resource-limit/quota harness to observe `ResourceError` or `MemoryLimitError`; no such harness exists. |
| `IRIS-V1-RUNTIME-V065` | `:584` | positive | executable | Concrete special-value sends assert canonical values/types and absent bare names. |
| `IRIS-V1-RUNTIME-V066` | `:585` | differential | differential | Explicitly compares interpreter and JIT under controlled host rounding mode. |
| `IRIS-V1-RUNTIME-V067` | `:586` | positive | executable | Concrete `mul_add` and multiply-then-add bit results assert one-rounding distinction. |
| `IRIS-V1-RUNTIME-V068` | `:588` | positive | executable | Concrete float-bit source with a concrete asserted aggregate. Reclassified in milestone 2: Array literals are implemented, and the `C115` bit-classification Methods plus `C113` `to_bits` this row sends were implemented alongside it. |
| `IRIS-V1-RUNTIME-V069` | `:589` | negative | executable | Concrete out-of-range `from_bits` calls each raise `RangeError`. |
| `IRIS-V1-RUNTIME-V070` | `:590` | negative | executable | Three controlled sources assert numeric ivar `InstanceStateError` and numeric identity `IdentityError`. |
| `IRIS-V1-RUNTIME-V071` | `:591` | positive | executable | Concrete numeric equality source asserts exactness boundary and equal numeric hashes. |
| `IRIS-V1-RUNTIME-V072` | `:592` | negative | executable | Concrete NaN-key source. Reclassified in milestone 2: the row needs only `C134` construction-time rejection, not a Hash container, and `hash_literal` simply had no parser production though the lexer already emitted its token. |
| `IRIS-V1-RUNTIME-V073` | `:593` | differential | differential | Explicitly requires exact public hashes to agree between interpreter and JIT. |
| `IRIS-V1-RUNTIME-V074` | `:594` | negative | executable | Concrete unavailable-selector send asserts `MessageNotFoundError`. |
| `IRIS-V1-RUNTIME-V075` | `:595` | positive | executable | Concrete comparison source with a concrete asserted aggregate and call log. Reclassified in milestone 2 once the `C083` root `<=>`, the `C084` derivation and the `C086` identity fast path were implemented. |
| `IRIS-V1-RUNTIME-V076` | `:596` | positive | executable | Concrete singleton comparison source with a concrete asserted aggregate. Reclassified in milestone 2 after the `C091`/`C092` protocols and the `C005` root Class both landed. |
| `IRIS-V1-RUNTIME-V077` | `:597` | positive | executable | Concrete callable-identity source. Reclassified in milestone 2 once closure literals parsed and `C042` identity-only equality replaced the TypeError that callables previously raised on `==`. |
| `IRIS-V1-RUNTIME-V078` | `:598` | positive | executable | Concrete definition-identity source. Reclassified in milestone 2 once Contract objects gained their own identity; Method alias identity and Class/Module/Type comparison already held. |
| `IRIS-V1-RUNTIME-V079` | `:599` | positive | needs-subsystem | Confirmed in milestone 2. Needs a compacting-GC subsystem and its controlled compaction hook. |
| `IRIS-V1-RUNTIME-V080` | `:600` | negative | executable | Concrete duplicate-selector source. Reclassified in milestone 2 after two real defects were fixed: a duplicate selector inside one class body was silently accepted, and a rejected declaration still published its Class. |
| `IRIS-V1-RUNTIME-V081` | `:601` | positive | executable | Concrete qualified-dispatch source. Reclassified in milestone 2 once `impl`, `impl C::member`, `as`/`as?` and Contract views landed. A second independent source proves the `C049` view check. |
| `IRIS-V1-RUNTIME-V082` | `:602` | positive | executable | Concrete compatible-open source with a concrete asserted aggregate. Reclassified in milestone 2 after probing showed nominal Type identity across a compatible open already holds; no code change was needed. |
| `IRIS-V1-RUNTIME-V083` | `:603` | positive | executable | Controlled construction/revision schedule asserts captured construction revision and active later dispatch. |
| `IRIS-V1-RUNTIME-V084` | `:604` | negative | executable | Two concrete absent-selector sends each raise `MessageNotFoundError`. |
| `IRIS-V1-RUNTIME-V085` | `:605` | positive | needs-subsystem | Confirmed in milestone 2. Needs `migrate_revision` and revision reactivation, neither of which exists. |
| `IRIS-V1-RUNTIME-V086` | `:606` | positive | needs-subsystem | Re-probed in milestone 2. The behavioural half is correct through property accessors: one instance reads the last written value and another reads nil. Two things block it, and neither is Dynamic construction: the row asserts a `String` value, which the runtime has no representation for, and a `Dynamic<Object>` STATIC slot Type, which `C068` states as a static fact with no runtime observation path. |
| `IRIS-V1-RUNTIME-V087` | `:607` | positive | needs-subsystem | Re-probed in milestone 2 and NARROWED to something unobservable. The escape-write half is correct and verified by a regression test: a Closure created in an instance Method keeps writing its captured receiver's ivars after return, per `C072`. The rebind half cannot be observed at all, because `D-311` states the captured receiver cannot be rebound through source, reflection, Dynamic or native APIs, so no conforming implementation offers an operation to attempt it. |
| `IRIS-V1-RUNTIME-V088` | `:608` | negative | needs-subsystem | Re-probed in milestone 2 and NARROWED further. THREE of the four paths already raise `MethodVisibilityError` correctly: external send, subclass send, and reflective invoke; declaring-Class code succeeds. Only the `Dynamic<T>` send that `D-313` also requires remains, and `Dynamic<T>` does not parse. |
| `IRIS-V1-RUNTIME-V089` | `:609` | positive | executable | Concrete composed-Module ivar fixture asserts state access and private-edge behavior. |
| `IRIS-V1-RUNTIME-V090` | `:610` | positive | executable | Concrete Module/Class fixture asserts `:B` dispatch and a single closed `A` MRO entry. |
| `IRIS-V1-RUNTIME-V091` | `:611` | positive | executable | Concrete class-variable storage source with a concrete asserted aggregate. Reclassified in milestone 2 after probing showed the shared cell, the independent Class-object ivars and the `C162` redeclaration rejection were ALL already implemented; no code change was needed. |
| `IRIS-V1-RUNTIME-V092` | `:612` | positive | executable | Concrete construction-order source. Reclassified in milestone 2: the `D-446` ordering was already correct, and only the `quoted_symbol` production was missing, without which reflection cannot name the `C061` setter selector `name=`. |
| `IRIS-V1-RUNTIME-V093` | `:613` | negative | executable | Concrete bare/explicit `super` cases assert parse rejection and `NoSuperMethodError`. |
| `IRIS-V1-RUNTIME-V094` | `:614` | positive | executable | Concrete alias/remove/undef sequence asserts shared Method identity, ancestor exposure, and missing-message. |
| `IRIS-V1-RUNTIME-V095` | `:615` | positive | executable | Concrete Class-object Method source asserts singleton selection and Class-object-chain lookup. |
| `IRIS-V1-RUNTIME-V096` | `:616` | positive | executable | Concrete property fixture asserts getter `name`, setter `name=`, and `:get`/`:set` values. |
| `IRIS-V1-RUNTIME-V097` | `:617` | positive | executable | Concrete truth test asserts one `to_bool` send and singleton truthiness. |
| `IRIS-V1-RUNTIME-V098` | `:618` | negative | executable | Concrete truthiness-protocol source. Reclassified in milestone 2 after a real `C037` defect was fixed: the evaluator discarded the assignment operator, so `&&=` and `||=` never sent `to_bool` and always wrote. ||=` forms all evaluate correctly now; the row's expectation is prose describing a `to_bool` that raises, so it needs authoring as a fixture rather than more implementation. ||=` propagation. |
| `IRIS-V1-RUNTIME-V099` | `:619` | positive | executable | Concrete missing-message source. Reclassified in milestone 2 once `trailing_block` parsed and `C099` passed it as the separate block parameter. The assertion INVOKES the block, because comparing `b same? b` could not distinguish a Closure from nil. |
| `IRIS-V1-RUNTIME-V100` | `:620` | positive | executable | Concrete runtime-superclass fixture. Reclassified in milestone 2 once `is`, nominal Type objects, `.type` and `subtype?` landed. A second independent source asserts ancestry DIRECTION, since the runner renders Class values opaquely. |
| `IRIS-V1-RUNTIME-V101` | `:621` | positive | executable | Concrete built-in openness source. Reclassified in milestone 2 after one over-broad reopen guard was narrowed: it had rejected any mixin on a built-in Class, conflating `C148`, which permits Module composition, with `C150`, which protects only the superclass. |
| `IRIS-V1-RUNTIME-V102` | `:622` | positive | executable | Concrete integer-modulo source. Reclassified in milestone 2 after probing showed the `mod` Method, the named infix and the `%` rejection ALL already hold; no code change was needed. |
| `IRIS-V1-RUNTIME-V103` | `:623` | positive | executable | Concrete special-value getter/setter replacement fixture observes replacement and no implicit storage. |
| `IRIS-V1-RUNTIME-V104` | `:624` | positive | executable | Concrete NaN arithmetic source asserts quiet NaN at receiver/common width. |
| `IRIS-V1-RUNTIME-V105` | `:625` | positive | executable | Concrete mixed-numeric comparison source with a concrete asserted aggregate. Reclassified in milestone 2: the earlier `needs-subsystem` reason cited an excluded collections library, but Array literals are implemented, and probing this row exposed a real `C131` exactness defect in `Numeric::compare`. |
| `IRIS-V1-RUNTIME-V106` | `:626` | positive | executable | Controlled built-in Method replacements assert independent `<=>` and `==` behavior. |
| `IRIS-V1-RUNTIME-V107` | `:627` | negative | executable | Concrete return-Contract source. Reclassified in milestone 2 after `D-094`'s two failure modes were separated: a wrong Integer is a `ComparisonContractError` while a non-Integer, non-nil result is a `TypeError`. It needed no `Dynamic<T>` syntax, contrary to the earlier recorded reason. |
| `IRIS-V1-RUNTIME-V108` | `:628` | negative | executable | Controlled MetaCapabilities validation asserts no Child publication after denied subclass creation. |
| `IRIS-V1-RUNTIME-V109` | `:629` | positive | executable | Controlled side-effect fixture asserts `same?` is true without calling replaceable comparison Methods. |
| `IRIS-V1-RUNTIME-V110` | `:630` | negative | needs-subsystem | Re-probed in milestone 2. All five reserved-selector declarations ARE rejected and the runner now observes diagnostics, but `IRIS-V1-GRAMMAR-C054` names no stable category for this case, so the emitted `PARSE_UNEXPECTED_TOKEN` would be an authored code. Marking it `status:authored-expect` was tried and reverted because the runner reports such a row without comparing anything, which would assert nothing while looking verified. |
| `IRIS-V1-RUNTIME-V111` | `:631` | negative | needs-subsystem | Re-probed in milestone 2. All four sources ARE rejected, but three of them fail as a PARSE diagnostic rather than the static rebinding rejection the row requires, and `const K = ...; K = ...` fails as an immutable-binding error. The row needs a declaration-validation rejection distinguishable from a parse failure, which no clause currently names a stable code for. |
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
