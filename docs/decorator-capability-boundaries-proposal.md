# Decorator Capability Boundaries Proposal

**Status: approved decision record; historical proposal superseded by the formal v1.35 decorator errata.** The owner said `按照建议开始`, approving all recommendations in the preceding decision table, including the expressly identified exception to TRACE-C021 for C125's wrong-kind rejection phase. No further semantic approval is pending. The approval checklist below was recorded before editing the formal clauses. The historical proposal following it is retained as design history: its pending-approval language, illustrative APIs and pseudocode are not current requirements. The authoritative APIs are in [chapter 08](../spec/iris-v1/08-modules-metaprogramming.md#v135-decorator-invocation-protocol). This document claims no implemented or passing runtime behavior.

## Approved v1.35 Checklist

| Recorded approval | Formal destination | Status |
| --- | --- | --- |
| First written outermost; A then B phase execution installs A(B(original)). | META-C129 | Approved |
| Initial binding, input checks and defaults before every outer callback; defaults never rerun for an attempt. | META-C130 | Approved |
| Immutable Invocation; concrete bound-name ArgumentChanges; checked argument and block replacement; no receiver/slot/generic retargeting. | META-C131 through C133 | Approved |
| Ordinary next Closure with optional changes and no trailing block; zero, one or sequential multiple attempts; no overlap. | META-C134, C137 | Approved |
| Exact original result guard at every body/wrapper boundary; invariant callable and Task adapters. | META-C135, C136 | Approved |
| Async wrappers registered by synchronous transform; fresh outer Task; activation/Task-scoped next; unfinished-inner diagnosis without cancellation or join. | META-C136, C137, C139 | Approved |
| Fresh zero-argument instance per application per phase; constructor plus complete plan call graph pure; captures persist per published chain; canonical fresh-state replay. | META-C127, C138 | Approved |
| Permanently kind-tagged context-derived empties; read-only context.kind/context.reason; Method and Property wrapping, Class/Module add_method, descriptive-only Contract; ordered persistent fluent operations, no AST or batch member wrapping. | META-C127, C128 | Approved |
| Wrong kind is rejected statically when provable, otherwise at runtime candidate validation with IRIS-DECORATOR-KIND and complete rollback. This explicitly supersedes only C125's unconditional static-phase requirement, under a bounded owner-authorized exception to README TRACE-C021. | META-C139, TRACE-C023 | Approved |
| Preserve published IDs, clauses, examples and vectors; bilingual formal publication and honest corpus-impact/coverage reporting; no runtime, fixture or tooling edits. | CONFORMANCE-C076; defect ledger | Approved |

Pre-edit inventory: both indexes were v1.34; META-C127 through META-C139, TRACE-C023 and CONFORMANCE-C076 were absent from the formal artifacts. The allocation is 13 META clauses, one revision clause and one coverage clause, with no new official example or vector IDs. Baseline verification: `python3 -B tools/spec-audit.py` exited 0 (14 artifacts, 37 migration rows, 614 normative paragraphs, 13 informative paragraphs); `python3 -B tools/corpus-audit.py` exited 1 (966 records, 57 violations). Existing user changes outside the ten-file allowlist are excluded.

Formal handoff: v1.35 is now specified in English and Simplified Chinese under META-C127 through META-C139, TRACE-C023 and CONFORMANCE-C076. The [appended ledger](spec-defects-v1.md#v135-approved-decorator-errata) records the exact API and the bounded C125 exception, as well as V428 through V431 impact. Post-edit `spec-audit.py` exits 0 with 14 product artifacts and no violations; `corpus-audit.py` exits 1 with 966 records and 57 aggregated violations, including 15 newly uncovered IDs in its C057 entry. Published vector rows and fixtures were not changed. No runtime support, passing new vectors or conformance-freeze completion is claimed. The historical pending-approval statements below are superseded, not outstanding requests.

## Historical Proposal (Superseded)

## Existing Baseline And Unresolved Boundary

The authoritative baseline is [chapter 08](../spec/iris-v1/08-modules-metaprogramming.md): META-C085 through C094 fix typed declaration transformation, top-to-bottom phase execution, pure planning, synchronous runtime transformation, capability checks, replay, and filtered reflection. META-C081 distinguishes compatible body replacement from member-set changes. META-C122 through C125 name the five Decorator Contracts, `plan(declaration, arguments)`, `transform(declaration, arguments, context)`, property views, and the minimal `Plan`/`Transformation` surface. They do not supply an invocation protocol for body wrapping.

The constraints this proposal preserves are:

| Existing owner | Relevant boundary |
| --- | --- |
| [Chapter 04](../spec/iris-v1/04-bindings-callables-control-flow.md), CONTROL-C014, C019, C022 through C033 | Three callable kinds; return guards; separate positional, keyword, rest, and block channels; defaults; lexical Closure receiver capture. |
| [Chapter 05](../spec/iris-v1/05-types-contracts-generics.md), TYPES-C031, C059, C094 through C096 | Invariant generic and callable Types; `Block<S>` aliases `BoundMethod<S> \| Closure<S>`; casts are not adapters. |
| [Chapter 03](../spec/iris-v1/03-runtime-object-model.md), RUNTIME-C012 through C015, C030, C037 through C039 | New Method identity on replacement; retained bodies; current-owner validation; separate qualified slots; removal versus tombstones. |
| [Chapter 07](../spec/iris-v1/07-async-resources-diagnostics.md), ASYNC-C003 through C028 | Eager async start, invariant typed Tasks, immutable completion, exception identity, no implicit join/cancellation, unobserved-failure diagnostics. |
| Chapter 08, META-C090, C095, C104 through C107 | No decorator authority grant, mutable reflection handle, or package-permission transfer. |

## Recommended Minimal Surface

Recommend an immutable, noncallable `Invocation` data record and an ordinary `next` Closure with one typed optional `ArgumentChanges` parameter, bound to one layer's prepared incoming payload. The proposed minimal immutable change record would let `next.call(changes)` replace bound argument values and the target's separate block channel; `next.call()` would forward that layer's incoming payload unchanged. Recommend adding inert `Transformation.wrap_method(wrapper)`, `Transformation.wrap_getter(wrapper)`, and `Transformation.wrap_setter(wrapper)` operations. Constructing one would describe a replacement; only candidate validation and atomic publication would install it. No fourth callable runtime kind, AST API, or implicit Closure `self` rebinding is proposed.

Recommend the following operation boundary rather than a universal decorator with arbitrary member access:

| Decorated kind | Recommended admitted operations | Capability and scope |
| --- | --- | --- |
| Class | Existing `add_method`; typed empty result. No addressed-member wrapping in this first surface. | `method_set` plus existing signature, visibility, `override`, `impl`, permission and static-spine checks. Decorate the member itself to wrap it. |
| Module | Existing `add_method`; typed empty result. No addressed-member wrapping in this first surface. | Same checks; retain the declared Module/member ownership model. No composition operation is added. |
| Method | `wrap_method(wrapper)` and typed empty result. No `add_method` on a Method target. | `method_body`; preserve the exact slot, complete signature, visibility, generic parameters and Contract obligations. |
| Property | `wrap_getter(wrapper)`, `wrap_setter(wrapper)`, typed empty result. | `property_body`; only an existing accessor can be wrapped. No missing setter creation, type change, storage or layout change. |
| Contract | Typed empty result and existing descriptive decorator reflection only. | No executable body, member addition, requirement mutation or hidden obligation. No new arbitrary metadata API. |

Recommend treating an accessor operation as `property_body`, not as a `method_body` loophole simply because accessors execute Methods. Denying `method_set` alone would not deny Method wrapping; denying `method_body` would. Property set/body capabilities would remain equally orthogonal. All generated operations would retain META-C081/C090's additional checks, without permission inferred from decorator application syntax.

For kind binding, recommend **context-derived typed empty values**, preserving the spelling `Plan.empty` and `Transformation.empty`. Evaluation during a phase would obtain that phase's immutable target-kind tag; the resulting value would remain permanently tagged. Reusing a Class-tagged empty in a Method phase would fail, rather than retag it. Factories evaluated in a phase would similarly derive the target kind, and reject an operation inadmissible for it. Outside a phase there would be no implicit target and construction would be rejected. This is a new recommended meaning, not something C125 already decides; do not also add explicit-kind factory overloads initially.

Recommend keeping `context` read-only, exposing only target kind and replay reason (`origin`, `open`, `upgrade`, `rollback`, `closed materialization`) as descriptive metadata. It would have no mutation methods, candidate handles, executable original-body handles, or grant-bearing tokens. Kind derivation would use runtime-owned phase data, not mutation of this object. Defer `Plan.metadata` and namespaced descriptor registration: the existing ordered decorator metadata suffices for this proposal, and an extensible descriptor schema would be a separate approval.

## Prepared Invocation And Sequential Attempts

**Owner decision: argument modification is allowed, with original-signature revalidation.** Replacement values must satisfy the exact originally selected closed signature before any inner wrapper or original body executes. This approves permission plus rechecking, not the bound-slot mapping, optional-parameter API, arity or lifetime design recommended below. Receiver, selector/exact qualified slot and closed generic bindings remain fixed; block replacement has its own approval below.

**Owner decision: block replacement is allowed under the explicitly offered conditions.** A new block must be revalidated against the original target's exact closed `Block<S>` contract before any inner wrapper or original body executes. Omitted replacement forwards the exact incoming block identity; explicit `nil` is allowed only for an optional block channel. Replacement code retains its own lexical receiver and package authority, borrowing neither from the original block nor the decorated Method. The block remains a separate channel, never positional; no silent rebinding or `Block` generic covariance is authorized. This approval does not settle the API or approve async wrapping, zero/multiple attempts, retry or lifetime rules.

Recommend this call boundary for every supported Method or accessor:

1. Evaluate receiver, arguments, splats, keywords and block creation in the existing call order; perform ordinary selector resolution and visibility/current-owner checks. Select one exact slot and signature, including a qualified Contract slot where applicable.
2. Bind and validate the complete original input contract and evaluate defaults once, left-to-right in their original lexical environment. A binding, default or parameter failure would prevent every wrapper from running, including a cache-hit wrapper. Preparation would not reorder existing argument/default failure precedence.
3. Freeze a descriptive prepared record and enter the outer wrapper. Give each wrapper its own `next`, pointing to the next captured layer, not to a selector lookup.
4. Each accepted `next.call(...)` would derive an attempt payload from that layer's incoming values and any change record, revalidate replacements against the exact originally selected closed signature, then enter the inner layer with a new immutable `Invocation`, fresh parameter binding cells and fresh positional-rest/keyword-rest containers. Each eventual original-body attempt would likewise get fresh cells and containers. Argument objects, including objects produced by defaults, would remain shared references, not deep copies.
5. Check the original result contract at each layer exit before exposing a successful result to the next outer layer or caller.

Recommend that `next` permit zero, one, or multiple **sequential** inner attempts. Zero permits a cache hit or fallback; multiple permits bounded retry. Its optional change record would default to an empty record; `next` itself would accept no trailing block. That is a proposed API encoding choice, not a block-replacement prohibition: the record carries a replacement for the original target's block channel. Every `next.call()` would forward its own layer's incoming payload, never a previous attempt's replacements; retries could supply a different immutable record each time. Calling it would not reevaluate source operands, default expressions, splats or keyword expressions, repeat source-call preparation, or infer generic bindings anew. A retry is another body attempt within the same prepared call, not another source invocation.

Recommend a bound-slot patch, not source-call-shape rewriting: address declared positional and keyword parameters and positional-rest/keyword-rest categories in `ArgumentChanges`. An omitted patch field would retain the current incoming value, including an already-computed default; explicit `nil` would be a supplied value subject to the original type contract, not absence. Validate the complete resulting payload, including required slots, rest shape/element types and keyword-rest names/value types as appropriate. Unknown new keywords would be invalid unless admitted by the original signature's keyword-rest contract; no arbitrary arity or channel remapping is promised. Exact slot addressing and replacement bookkeeping remain proposed API design.

Recommend an optional `block` field in `ArgumentChanges` with distinct absent and explicit-`nil` states. Absence retains the incoming block, including `nil`; a supplied non-nil value must satisfy the original `Block<S>` contract, even when replacing `nil` in an optional channel. A required channel rejects `nil`, and a target without a block channel rejects a supplied block field; no new channel is created.

Recommend rejecting an invalid replacement through ordinary input/type failure behavior before any inner effects, without changing the original or current layer's `Invocation`. A successful replacement would appear in the inner layer's snapshot, while the outer snapshot would remain unchanged. Neither outcome would roll back shared-object mutations or effects of computing the change record.

Recommend that `Invocation` contain the following read-only snapshots, not live binding cells or mutable frame views:

| Field group | Recommended contents |
| --- | --- |
| Target | Receiver relation, exact selected slot identity and full signature, including qualified Contract identity and accessor kind. No public invocable original-Method handle. |
| Parameters | Current layer's bound values in declared parameter order and declared names/categories. Original-call omission/default-used flags would retain their provenance, distinct from proposed replacement flags identifying updated slots; a replacement would not rewrite the original call history. |
| Call channels | Immutable original evaluated positional and keyword-name/value snapshots, plus current layer's bound channel/rest membership; distinguish provenance from replacement payload, without reconstructing a new source call or flattening channels into an Array. |
| Block | Current layer's block object or `nil`, separately from the immutable original-call block identity/omission provenance and proposed replacement flags. An unchanged forward retains exact incoming identity; a checked replacement is a new current value, never positional. |
| Types | Closed owner type arguments and closed Method type arguments with the selected signature's substitutions. |

Snapshot containers would be immutable, but their receiver/value objects would retain ordinary mutability. Reading a record would not grant reflection into private state. A first attempt's rest-container edits would not change the next attempt's container; its mutation of a shared argument object would remain visible. Fresh cells also matter when an attempt creates escaping Closures over its parameters. Recommend argument and block updates only through a new checked `ArgumentChanges` record, never a mutable `Invocation` path; no replacement receiver is proposed.

## Typed Admission And Return Guards

For synchronous wrapping, recommend this conceptual erased registration callback shape. `changes: ArgumentChanges = empty` denotes one typed optional parameter on an ordinary Closure, not formal Iris callable Type notation or an overload:

```text
Closure<(Invocation, Closure<(changes: ArgumentChanges = empty) -> Object>) -> Object>
```

Recommend a runtime-owned **typed admission adapter** specialized to the original exact closed signature and result contract `R`. The newly constructed ordinary `next` Closure would validate the proposed payload before inner entry, then check the inner result against `R` before exposing it as an ordinary Object value. The callback's Object result would be validated against `R` before successful layer exit. This is invocation and validation, not a covariant cast between a Closure returning `R` and one returning `Object`; TYPES-C096 forbids that covariance.

Recommend retaining the original body's own return guard and adding the exact original `R` guard to **every** wrapper layer, including a layer that never calls `next`. An invalid inner result would raise before an outer callback could observe it and substitute a valid-looking result. An outer wrapper could catch that ordinary failure and recover, but could not turn an unchecked invalid inner return into a successful boundary crossing. `Never`, nilability, generic arguments and accessor return contracts would remain exact; setter assignment would still yield the checked setter result, not automatically the assigned argument.

Recommend rejecting a callback with an incompatible registration signature at admission when provable, otherwise at candidate validation. Runtime return-contract failures would use the existing ordinary Type failure path, not hidden host errors. The adapter would not weaken the installed Method's signature to `Object` or introduce overload dispatch.

## Async Wrapping And Task Lifetime

Recommend supporting async Method wrapping in this initial protocol with a separate async admission mode selected from the target signature. Its erased callback would be an **async Closure**, with an awaited result of `Object`; in invocation-result notation its shape would be:

```text
Closure<(Invocation, Closure<(changes: ArgumentChanges = empty) -> Task<Object>>) -> Task<Object>>
```

This conceptual notation uses the same proposed typed optional parameter; it is not formal Iris callable Type syntax or an instruction to annotate an async body's awaited result as `Task<Object>`. Recommend rejecting sync/async callback mode mismatches rather than silently converting a synchronous target into async or blocking to unwrap a Task.

Recommend typed Task adapters: each inner attempt would produce its own typed `Task<R>`; an explicitly constructed async bridge would await it and produce `Task<Object>` for the erased `next` interface. An outer admission adapter would await the callback result, validate the completion value against exact `R`, and complete the installed Method's `Task<R>`. There would be no `Task<R> as Task<Object>` or reverse cast. Every layer's completion would be checked before the next layer receives a successful value, preserving original-body and wrapper return guards.

Recommend a fresh outer `Task<R>` for each decorated async invocation, distinct from callback and inner-attempt Tasks even for transparent forwarding or cache hits. Cache results, not the caller-visible outer Task identity. Each `next.call(...)` admitted by the scope and in-flight rules would return a fresh adapter Task; repeated awaits on that same Task would retain its completion identity. Async bridge allocations would therefore be observable as distinct Tasks, not hidden covariance conversions. This is an explicit recommended identity choice, not a deduction from top-to-bottom transformation order.

Recommend preserving eager start: prepare inputs and start the wrapper synchronously until completion, failure, or the first incomplete await, with no extra scheduling hop for adapters or already-complete Tasks. Receiver/operand evaluation failures would retain the ordinary pre-Task path; binding/default failures during async body setup would fail the outer Task before callback entry, consistent with ASYNC-C019/C022. Propagated failures would retain the original ExceptionContext root through adapter awaits. A new return-guard failure would create its own ordinary failure context.

Recommend that invalid async replacements fail the adapter Task through ordinary input/type failure behavior before inner entry; the owning callback could observe/catch that failure through `await`. Existing proposed scope, in-flight and settlement rules would still apply, with no new special diagnostic phase or permission to overlap attempts. This failure-delivery detail remains a recommendation, not part of the owner's approval.

Recommend that `transform` merely register an async wrapper Closure; it would remain synchronous and would neither invoke the wrapper nor await it during the transaction. Creating code for future async execution would not grant that code transaction authority. Capturing descriptive metadata would be harmless; a candidate or mutation capability would remain unavailable.

Recommend activation-scoped `next` for synchronous wrappers and callback-Task-scoped `next` for async wrappers:

- A synchronous helper executing within the owning activation/Task could receive and call `next`. Merely storing or passing this ordinary Closure would not be illegal or confer lasting authority.
- Calling after the owning activation completes, from a foreign Task, or while a previous inner attempt remains in flight would raise an ordinary protocol failure before starting another attempt. A newly created async helper Task would be foreign even if its eager prefix runs synchronously.
- The async owner would remain valid across its own suspension and resume. Another attempt would be permitted only after the previous attempt's Task has settled, successfully or exceptionally. Each inner wrapper would own its own scope and Task; admission bridges would not make user calls from foreign Tasks legal.
- Recommend diagnosing an outer callback that completes while one of its `next` attempts is still in flight. A normal outer return would instead fail the outer admission Task with a protocol failure; if it already failed, preserve that primary failure and report the lifetime violation through structured diagnostics.
- The already-started inner Task would continue to completion with its own active inner scopes. There would be **no implicit cancellation or join**, no retroactive abortion of its body, and no authority to launch another attempt from the completed outer scope. Tracking settlement would not count as observing an abandoned failure; adapter propagation would keep an unobserved failure reportable under ASYNC-C027/C028 rather than swallowing it.

The protocol-failure name above is descriptive, not an allocated diagnostic ID. Recommend including the owner activation/Task and violation category without leaking inaccessible arguments. A retained `next` reference would keep ordinary captured data alive but would not extend execution permission. These proposed restrictions belong to the continuation, not to an ordinary original block captured by a replacement Closure: that block can be called normally with its declared signature under its own callable rules, without acquiring `next`'s activation/Task scope restrictions.

## Ordering, Identity, Authority, And Replay

**Owner decision: first written outermost, later written innermost.** For source `@A` followed by `@B`, phase execution remains A then B under META-C086, while the installed chain is `A(B(original))`. Invocation enters A, B, original and exits normally through B then A. Reversing the source order produces `B(A(original))`. This records the owner's selected nesting rule; formal bilingual publication remains pending.

Recommend assembling an ordered wrapper chain from the returned operations: a later wrapper is inserted inside earlier layers, immediately before the underlying body, rather than wrapping the entire accumulated chain. Earlier layers' `next` Closures are bound to their actual inner suffix at invocation time. This assembly rule does not reverse phase execution or rerun earlier decorators.

Recommend publishing a new Method identity for the wrapped chain, while keeping logical Class/Module identity unchanged. A saved Method or BoundMethod would retain its exact old chain and captured decorator state; later ordinary lookup would see the new chain. Current lexical-owner membership would still be validated at retained-call entry, even for a cache hit. Removing or undefining the selected slot would affect future lookup, not retarget a retained Method whose owner remains valid.

Recommend `next` as a direct continuation into captured chain code, never ordinary redispatch, `super`, or access to a tombstoned replacement. The original body would retain its original lexical owner and package; its explicit `super(args...)` would retain chapter 03's current-MRO behavior. Wrapper Closure code would retain its own captured receiver and lexical package. `invocation.receiver` would supply an explicit value, not become wrapper `self`. Reflection permissions would follow the currently executing original or wrapper code under META-C104/C105, without transfer from receiver, caller, decorator application, helper, or `next`.

Recommend replay from an exact **canonical undecorated declaration artifact plus its ordered decorator applications**, rebuilding each chain once. A rebuild would not wrap the already decorated active body again. Origin, declarative open, upgrade, rollback reconstruction and closed generic materialization would each perform the applicable source-ordered replay and current-policy validation. Rollback would reconstruct a new revision against the current static spine and verified historical artifact, not reactivate old executable state.

Recommend a fresh zero-argument decorator instance per application **per phase**. Application arguments would be delivered to `plan`/`transform`, not forwarded to `initialize`. Both Contract members would remain required; an unused phase would return its typed empty. Static purity would cover construction, including `initialize`, and the full plan call graph, not just the visible `plan` body. No instance would be shared between planning and runtime transformation, or between separate applications/replays/closed materializations.

Recommend retaining runtime wrapper captures across ordinary calls to the published chain, so a per-application cache or counter can work. Separate applications would have distinct captured state by default, and old retained chains would retain their old state. Deliberately shared ordinary runtime objects would still be shared; there would be no hidden clone or synchronization guarantee. Replayed chains would get fresh application instances, not silently inherit old caches. Runtime initialization/transform side effects would remain the author's responsibility and would not be rolled back, unlike discarded runtime-owned candidate state.

## Supported Target Classification

Recommend the following explicit classifications for the initial wrapper protocol, rather than implicit implementation-dependent omissions:

| Surface | Recommendation |
| --- | --- |
| Ordinary, Module/main, class-object and operator Methods | Supported where there is an actual wrappable slot, preserving selector and receiver ownership. Non-overloadable primitives are not fabricated as slots. |
| Generic Methods and generic owners | Supported using the exact closed substitutions in preparation and every guard; replay at closed materialization with isolated application state. No erased contract substitution. |
| Contract-qualified implementations | Supported at the exact qualified slot. Never wrap an ordinary same-name slot by accident; do not modify the Contract declaration or its requirements. |
| Generated property accessors | Supported through Property wrapping, preserving getter/setter availability, storage validation and assignment-result semantics. Missing accessors are rejected. |
| Native-backed Methods/accessors | Supported through the same managed admission boundary when compatible body replacement is authorized and ABI metadata supplies the exact signature. No native pointer exposure or ABI/layout rewrite. Native structural changes still require the additional `native` capability; protected/nonreplaceable entries are rejected. |
| Async Methods, including generic/qualified/native-backed ones | Supported through the async mode above, subject to existing native Task completion obligations. Sync Methods returning a Task as an ordinary value remain sync and retain their exact Task-valued result contract. |

## Illustrative Usage And Rejected Alternative

Informative example: the following is deliberately language-neutral pseudocode. `Logged`, `Cached`, `Retry`, `NormalizeName` and `TraceBlock` represent ordinary Classes implementing `MethodDecorator`, each with a pure `plan` returning typed `Plan.empty` and a synchronous `transform` registering the callback shown. `ArgumentChanges` and `invocation.block` illustrate proposed record syntax and current-layer access, not settled APIs; other helper names below are application policy, not proposed core APIs.

```text
@Logged()                         # first written is outermost
@Cached(key = application_key)     # cache hit skips Retry and original, not Logged
@Retry(limit = 2)                  # runtime chain: Logged(Cached(Retry(original)))
METHOD load(original signature unchanged)

Logged.transform: return Transformation.wrap_method(closure(invocation, next):
    log_enter(invocation.slot)
    try: return next.call()
    finally: log_exit(invocation.slot))

Cached.transform: create fresh per-application cache; return Transformation.wrap_method(
    closure(invocation, next):
        key = application_key(invocation)
        if cache.contains(key): return cache[key]
        result = next.call()
        cache[key] = result
        return result)

Retry.transform: return Transformation.wrap_method(closure(invocation, next):
    for attempt in 1..limit:
        try: return next.call()
        catch transient_failure:
            if attempt == limit: rethrow_current_failure)

@NormalizeName()
METHOD greet(name: String) -> String
NormalizeName.transform: return Transformation.wrap_method(closure(invocation, next):
    normalized = normalize_name(invocation.parameters["name"])
    return next.call(ArgumentChanges(positional = {"name": normalized})))

@TraceBlock()
METHOD apply(value: Integer; optional block: Block<(Integer) -> Integer>) -> Integer
TraceBlock.transform: return Transformation.wrap_method(closure(invocation, next):
    original = invocation.block
    if original == nil: return next.call()
    replacement = closure(value: Integer) -> Integer {
        log(value); return original.call(value)
    }
    return next.call(ArgumentChanges(block = replacement)))
```

In this ordering, `Logged` balances its enter/exit once around the logical call, including cache hits and all inner retries; logging itself can fail. A cache hit skips Retry and the original body. A miss enters Retry, which repeats only the original body; Cached stores the eventual successful result. Placing Logged inside Retry would instead log each attempt. Cache keys need deliberate receiver/type/block identity and invalidation policy, and immutable snapshots do not make argument objects safe cache keys. Retry limits need application validation, retries should select recoverable failures, and repeated effects are not transactional. Async variants would use async callbacks and explicit awaits, not run async work in `transform`.

Reject unrestricted redispatch or a Closure such as `wrapper(original, receiver, args, kwargs, block)` returning a new arbitrary callable, not checked argument/block updates. Exposing an invocable original-Method handle or changing receiver, selector, qualified slot or generic bindings could escape the selected contract; capturing and calling the ordinary original block in `TraceBlock` does not expose such a Method handle. The proposed prepared-call protocol supports normalization, logging, caching and bounded sequential retry with checked argument and block replacement; it does not authorize arbitrary target-callable replacement or repeated source-call evaluation.

## Owner Decisions And Remaining Approvals

1. **Argument decision and invocation boundary:** the owner approved argument modification with exact original-signature revalidation before inner execution. Immutable prepared `Invocation`, bound-slot `ArgumentChanges`, the ordinary Closure's optional parameter/arity, omission and replacement flags, zero/one/multiple sequential attempts, and original input/default checks before wrappers remain recommendations awaiting approval. Receiver/selector/qualified-slot/generic bindings stay fixed.
2. **Typed execution:** approve per-layer exact-result admission guards, invariant Closure/Task adapters, fresh outer async Task identity, eager start and scoped `next`, including early-completion diagnosis without implicit cancellation/join. Recommended default: approve as one coherent protocol.
3. **Composition and construction:** first-written-outermost nesting is selected by the owner. Canonical undecorated replay and fresh zero-argument instances per application per phase with full constructor/plan purity remain recommendations awaiting approval; do not interpret the ordering decision as approval of lifecycle or cache isolation rules.
4. **Operation surface:** approve the kind/capability matrix, context-derived typed empties, read-only context, and the explicit generic/qualified/generated/native/async classifications. Recommended default: keep Class/Module addressed-member wrapping and arbitrary `Plan.metadata` out of this initial surface.
5. **Wrong-kind phase conflict:** recommend `IRIS-DECORATOR-KIND` statically when provable, otherwise at runtime candidate validation with complete rollback. META-C125 currently requires phase static even for a runtime-produced wrong-kind Transformation. Changing that decided phase requires an **explicit owner-authorized exception to TRACE-C021**, not ordinary editorial clarification or an assertion that C125 already permits it. Recommended default: approve the exception, preserving static rejection for provable cases.
6. **Block decision:** the owner approved replacement with original closed `Block<S>` revalidation before inner entry, exact incoming identity on omission, `nil` only for an optional channel, retained lexical receiver/package authority and no positional conversion. The optional `ArgumentChanges.block` representation and separate original/current provenance fields remain proposed; this decision approves none of the remaining API, async, attempt, lifetime, lifecycle or operation choices above.

## Proposed Acceptance Scenarios

These are review scenarios, not new official vector IDs or claims of passing tests:

| Scenario | Recommended observable |
| --- | --- |
| Non-block Method, ordinary delegation | Wrapper receives noncallable Invocation and ordinary Closure with a typed optional change record; `next.call()` forwards its incoming payload; no synthetic trailing-block requirement; body runs once. |
| Zero-call cache hit | Body runs zero times; valid original inputs/defaults still run once; result satisfies exact original `R`. |
| Sequential retry | Two attempts after a selected failure; argument/default/splat counters remain one; no implicit rollback of body effects. |
| Fresh attempt state | Rest containers and parameter-capturing cells differ per attempt; shared argument-object mutation remains visible. |
| Unchanged block identity | Without a replacement, each attempt forwards its layer's exact incoming Closure or BoundMethod identity, or `nil`; an inner layer forwards an outer replacement, not the original-call block. No channel flattening. |
| Checked block replacement | `TraceBlock` supplies a matching Integer-to-Integer Closure; inner wrapper/body sees the replacement identity, while outer/original-call records stay unchanged. Calling it logs and calls the captured block with the declared signature. Each Closure retains its own lexical receiver/package; replacement code cannot borrow original block/Method authority, and ordinary captured-block calls do not inherit `next` scope restrictions. |
| Invalid block and optional nil | Wrong block signature or `nil` for a required channel fails before entry with inner wrapper/body counter zero. Optional `nil` is accepted; a matching non-nil replacement may fill an optional incoming `nil`. Omission preserves incoming identity/nil; a target without a block channel rejects a block patch. |
| Parameter/default rejection | Wrong type, duplicate/unknown keyword, unexpected/invalid block or raising default prevents even an outer cache callback; no `method_missing`. Under proposed pre-wrapper validation, a decorator cannot repair an initially invalid call. |
| NormalizeName and invalid type | For `greet(name: String)`, a new record supplies the normalized String. `next.call(ArgumentChanges(positional = {"name": 42}))` fails with inner wrapper/body marker `inner_effects` still zero; original Invocation stays unchanged. Effects of computing changes or mutating shared objects are not rolled back. |
| Nested forwarding and retries | Outer sees `" raw "`, supplies `"raw"`; inner sees `"raw"` and its `next.call()` sends `"raw"` to the body. Outer's snapshot stays `" raw "`; its later `next.call()` forwards `" raw "`, not its last replacement. Sequential retries may instead choose different records. |
| Patch omission, channels and bindings | Omitted fields retain incoming values/computed defaults; explicit `nil` is checked as a value. Positional/keyword/rest/keyword-rest updates obey original shape/types; unknown keywords fail unless admitted by original keyword-rest. Original omission flags stay distinct from replacement flags; defaults/splats are not rerun, generics not reinferred, receiver/slot unchanged and block updates checked in their separate channel. |
| Return rejection | Invalid original-body, inner-wrapper and zero-call result each fails at its own exact-`R` boundary; outer code cannot observe invalid success. |
| Async behavior | Eager prefix runs before call returns; fresh outer Task differs from inner Tasks; repeat awaits preserve completion and failure-root identity; adapters add no fairness hop. |
| Async invalid replacement | Wrong-type change fails the proposed adapter Task through ordinary input/type failure behavior; awaiting callback can catch it; inner side-effect marker stays zero. Retry requires the same owning Task and prior settlement; no special replacement phase or implicit cancellation/join. |
| Lifetime violation | Helper in same scope succeeds; post-completion/foreign-Task/overlapping call fails; early outer return is diagnosed while in-flight work continues and failures remain reportable. |
| Ordered wrappers and state | Transform A then B; enter A then B then body; exit B then A. Reversed source reverses nesting. Logged/Cached/Retry logs once even on a cache hit, which skips Retry and body; a miss retries body without reentering outer layers. Captured-state persistence and isolation remain recommendations. |
| Retained Method | Replacement creates new Method; saved BoundMethod runs old chain; loss of lexical owner raises `MethodBindingError` before wrapper entry. |
| Permission and capability | Wrapper cannot borrow original package reflection access; deny-body rejects wrapping, deny-set alone allows compatible wrapping, denied add-method rolls back. |
| Tombstone and slot isolation | New ordinary call respects undef; saved valid old chain remains explicit; qualified wrapper never redirects to ordinary slot and `next` never redispatches. |
| Replay and purity | Origin/open/upgrade/rollback/closed materialization rebuild each application once; nondeterministic `initialize` is rejected during planning; runtime failure publishes nothing. |
| Kind and supported variants | Correct typed empties work for all five targets; provable wrong kind is static, dynamic wrong kind rolls back; generated accessor, native, generic and qualified cases preserve exact contracts. |

## Revision Impact And Evidence

The [revision procedure](../spec/iris-v1/README.md) in TRACE-C019 through C022 requires recorded semantic approval, preserved published IDs, synchronized English and Simplified Chinese material, and committed-corpus verification before publication. This proposal allocates **no official new IDs and reserves no v1.35 revision**; both remain unassigned until approval and a fresh inventory check.

| Later approval-gated artifact | Recommended impact |
| --- | --- |
| English chapter 08 and `spec/iris-v1/zh-cn/08-modules-metaprogramming.md` | Add approved protocol/operation/lifecycle clauses; explicitly identify any superseded C125 phase requirement; retain C081 and C085 through C094/C122 through C125 anchors. |
| English chapters 03, 04, 05, 07 and matching `zh-cn/` chapters | Cross-link owning identity, prepared-call/default, invariant adapter, Task/lifetime rules; add only approved refinements rather than duplicate or redefine their contracts. |
| English/Chinese README and traceability matrices | Record the actual approved revision and exception; map D-511 through D-514 and capability D-299/D-300 to exact added clauses without rewriting frozen decisions. |
| English/Chinese conformance chapter and chapter vector tables; machine-readable corpus | Assess every old result before publication; add approval-gated coverage for these scenarios without renumbering, deleting, retagging or silently replacing published evidence. |

Observed published fixture mismatches to carry into that review, not repair in this proposal:

| Existing META vector | Published obligation versus committed fixture/record |
| --- | --- |
| [V428](../conformance/iris-v1/vectors/META/IRIS-V1-META-V428.json) | Chapter table describes a returned Module candidate on Class Box. Fixture instead declares a local plan-only `ModuleDecorator` and applies its Class to Box: target-Contract mismatch, not a runtime wrong-kind result. Record asserts the code, not the full phase/no-revision observation. |
| [V429](../conformance/iris-v1/vectors/META/IRIS-V1-META-V429.json) | Table names `Clock.now()`; imported fixture reads mutable `$tick` instead, a mismatch already recorded in the defect ledger. Its local Contract is plan-only and returns Integer rather than Plan; the record does not fully assert phase/no-publication. Preserve the nondeterminism expectation. |
| [V430](../conformance/iris-v1/vectors/META/IRIS-V1-META-V430.json) | Denied `method_set` is exercised, but fixture uses a local transform-only Contract and fluent `Transformation.empty.add_method` shape. Record asserts Box is not published, rather than the table's unchanged revision/method-list observations. It does not validate the full two-member Contract surface. |
| [V431](../conformance/iris-v1/vectors/META/IRIS-V1-META-V431.json) | Table covers phase order through origin/upgrade/rollback, filtered diff and forbidden transform await. Committed fixture only adds two Methods using local transform-only Contracts and probes Methods/decorator names; it contains no upgrade, rollback or await case. Its record category and decision link also differ from the published table. |

Initial review verification reported `python3 -B tools/spec-audit.py`: 14 product artifacts and no violations. `python3 -B tools/corpus-audit.py` examined 966 records and reported 57 violations, matching the planner's earlier baseline count. These include stale inventory entries and incomplete observations; they were not changed for this proposal. These historical audits were not rerun for the argument- or block-decision updates. Structural audit results are not proof of the proposed protocol's behavioral correctness.

An independent read-only review found the initial proposal coherent across call preparation, composition, callable invariance, Task lifetime, kind binding, capabilities, and replay. The owner subsequently selected first-written-outermost nesting, allowed argument modification with original-signature revalidation, and approved block replacement under the explicit conditions recorded above; this draft now reflects all three decisions. The initial review does not cover the argument/block decisions or their proposed API, binding and async behavior, and does not constitute approval or review of subsequent changes. Formal publication still requires the remaining recorded decisions, synchronized bilingual clauses, explicit reporting of changed vector outcomes, and executable acceptance evidence. No future tests are claimed to pass by this document.
