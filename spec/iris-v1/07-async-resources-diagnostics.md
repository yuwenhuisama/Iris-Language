# Iris v1 Async, Resources, And Diagnostics

Status: Iris v1 draft, frozen semantics.

IRIS-V1-ASYNC-C001: This chapter defines Iris v1 async Methods, async Closures, `Task<T>`, `Awaitable<T>`, the single IrisRuntime scheduler, `await`, async exception propagation, unobserved failed Task diagnostics, Closeable resources, ordinary `using`, Iterator cleanup interaction, structured diagnostic reporting, revision event delivery, `GapEvent`, and audit history recovery. It MUST be read after [README.md](README.md), [01-language-identity.md](01-language-identity.md), [03-runtime-object-model.md](03-runtime-object-model.md), and [04-bindings-callables-control-flow.md](04-bindings-callables-control-flow.md).

IRIS-V1-ASYNC-C002: This chapter MUST NOT define native no-GIL shared-memory Iris threads, cancellation semantics, parser token inventory, Iterator identity and hashing, ordinary ExceptionContext ownership, open-transaction mutation rules, Host ABI function names, or implementation event-loop internals. Those surfaces belong to the identity, grammar, collections, control, metaprogramming, and native chapters named by this document.

## Async Callable Surface

IRIS-V1-ASYNC-C003: `async fun f(parameters...) -> T { body }` declares the awaited result type `T`. Invoking the Method returns `Task<T>`, not `T`. A no-result async Method MUST declare or infer `T` as `Nil` and therefore returns `Task<Nil>` at invocation.

IRIS-V1-ASYNC-C004: An async Closure is a Closure whose header carries the `async` modifier, written semantically as `{ async |parameters| -> T body }` wherever the grammar admits async Closure headers. Evaluating the literal creates an identity-bearing Closure object under the ordinary Closure capture rules. Invoking it returns `Task<T>`.

IRIS-V1-ASYNC-C005: Iris v1 has no source or reflection signature that means async void, non-generic `Task`, detached async result, or fire-and-forget callable. A conforming implementation MUST reject such signatures. Programs that intentionally ignore a returned Task may do so only by ordinary value discard, and failure diagnostics for that Task still follow this chapter.

IRIS-V1-ASYNC-C006: `Task<T>` is an identity-bearing object representing one eventual completion of type `T` or one captured `ExceptionContext`. Task default equality and hash behavior use runtime-local identity. Task identity does not depend on the eventual result value, exception value, awaiter count, or scheduler queue position.

IRIS-V1-ASYNC-C007: `Awaitable<T>` is the standard Contract for values that can be awaited to produce `T`. `Task<T>` MUST implement `Awaitable<T>`. A conforming implementation MAY provide other standard Awaitable types only when they obey the same completion, exception, and reuse rules stated here.

IRIS-V1-ASYNC-C008: The awaited type of `await expr` is the `T` from `Awaitable<T>`. Static checking MUST reject a provably non-Awaitable operand. Dynamic execution MUST raise `TypeError` or a more specific Contract error when the operand does not satisfy `Awaitable<T>` at runtime.

IRIS-V1-ASYNC-C009: Async Method and async Closure bodies use the same parameter binding, lexical scope, receiver capture, return, raise, try/catch/finally, and Closure-boundary rules as ordinary callables unless this chapter refines suspension behavior. `return expr` inside an async body completes its Task with the value after checking the awaited result Contract `T`.

IRIS-V1-ASYNC-C010: The following async callable table is normative:

| Source form                 | Invocation result | Await result                                                 | Identity                                       | Reuse rule                         |
| --------------------------- | ----------------- | ------------------------------------------------------------ | ---------------------------------------------- | ---------------------------------- |
| `async fun f(...) -> T`   | `Task<T>`       | `T`                                                        | Method and returned Task are identity-bearing  | The Task may be awaited many times |
| `async fun f(...) -> Nil` | `Task<Nil>`     | `nil`                                                      | Method and returned Task are identity-bearing  | The Task may be awaited many times |
| Async Closure literal       | `Task<T>`       | `T`                                                        | Closure and returned Task are identity-bearing | The Task may be awaited many times |
| Ordinary`fun f(...) -> T` | `T`             | Not awaitable unless`T` itself implements `Awaitable<U>` | Method rules from runtime chapter              | Ordinary call result rules         |

IRIS-V1-ASYNC-EX001: Informative example, result typing:

```iris
async fun read_name(path: String) -> String {
  return await File.read_text(path)
}

async fun log_name(path: String) -> Nil {
  print(await read_name(path))
}

let task: Task<String> = read_name("user.ir")
let name: String = await task
```

## Single Runtime Scheduler

IRIS-V1-ASYNC-C011: One IrisRuntime executes Iris code on a single cooperative scheduler and shared heap. Iris Task interleaving MUST occur only at `await` of an incomplete Awaitable or at scheduler boundaries that later Host or runtime APIs explicitly define. Ordinary expression evaluation and Method dispatch MUST NOT be preempted by another Iris Task between such boundaries.

IRIS-V1-ASYNC-C012: Invoking an async Method or async Closure starts execution synchronously on the current IrisRuntime. The body runs until it completes, raises, or reaches the first `await` whose Awaitable is incomplete. Creating the returned Task and starting this initial run are one call operation from the caller's point of view.

IRIS-V1-ASYNC-C013: Awaiting an already-complete Awaitable MUST continue synchronously in the current Task without enqueueing a later continuation solely for fairness. Awaiting an incomplete Awaitable MUST register the current continuation, suspend the current Task, and return control to the scheduler or Host event-loop driver.

IRIS-V1-ASYNC-C014: Ready continuations created by Iris runtime actions MUST be enqueued in deterministic FIFO order. External IO or Host completions enter the scheduler in the order the Host posts them to the IrisRuntime. Iris v1 does not promise an ordering between independent external completions before they are posted.

IRIS-V1-ASYNC-C015: Iris code has no implicit blocking wait, hidden synchronous Task join, sleep-until-completion primitive, or property read that blocks until a Task completes. Host APIs MAY drive or run the event loop according to the native chapter, but that Host control surface is not an Iris source-level blocking wait.

IRIS-V1-ASYNC-C016: Native no-GIL shared-memory Iris threads are `DEFERRED V1`. A conforming v1 implementation MUST NOT expose language semantics that require multiple Iris threads racing over the same managed heap, and implementation architecture MUST NOT require a permanent global interpreter lock as a semantic or ABI premise. This clause does not define a no-GIL memory model, data-race behavior, atomicity rule, or shared-memory thread API; future semantics remain unspecified.

IRIS-V1-ASYNC-C017: Task, thread, and async cancellation semantics are `DEFERRED V1`. Iris v1 defines no cancellation type, cancellation point, masking rule, asynchronous interruption rule, `CancellationError` Contract, or Host-injected unwinding model. Cooperative flags are ordinary library or application values and do not change the language semantics of `await`, `finally`, `using`, or Iterator cleanup.

IRIS-V1-ASYNC-C018: Await, scheduler yield, thread transfer, and escaping transaction capability are prohibited inside open or revision transactions. Any `await` expression lexically inside an open transaction body, or inside a callable that could escape and resume with transaction authority, MUST be rejected before execution. The metaprogramming chapter owns transaction shape and authority rules; this chapter owns the async suspension reason for the prohibition.

IRIS-V1-ASYNC-C019: The following scheduler state table is normative:

| Event                          | Current Task state             | Scheduler action                                | Observable result                                                            | Error or diagnostic rule                         |
| ------------------------------ | ------------------------------ | ----------------------------------------------- | ---------------------------------------------------------------------------- | ------------------------------------------------ |
| Async invocation starts        | New Task running synchronously | None before first incomplete await              | Caller receives Task object after initial run reaches boundary or completion | Body setup failure completes Task as failed      |
| Body returns before suspension | Completed with value           | No continuation                                 | Awaiters receive same immutable value                                        | Return Contract failure completes Task as failed |
| Body raises before suspension  | Completed failed               | No continuation                                 | Await repropagates captured ExceptionContext                                 | Unobserved failure diagnostic may later fire     |
| `await` complete Awaitable   | Still running                  | No enqueue required                             | Await expression yields value or rethrows immediately                        | Awaited failure links async stack                |
| `await` incomplete Awaitable | Suspended                      | Register continuation and yield runtime control | Caller of scheduler may run another ready Task                               | Operand Contract failure prevents suspension     |
| Awaitable completes later      | Ready                          | Enqueue continuation FIFO after post            | Continuation resumes at await site                                           | Failure resumes by repropagation                 |

## Await Completion And Exceptions

IRIS-V1-ASYNC-C020: A Task completion is immutable. Once a Task completes with a value or a captured `ExceptionContext`, later awaits, diagnostic observers, and Host bridges MUST observe that same completion category and same value or context identity.

IRIS-V1-ASYNC-C021: A `Task<T>` MAY be awaited multiple times. Every successful await of a completed Task returns the same completed value according to ordinary value identity or value semantics. Every failed await repropagates the same captured ExceptionContext with an added async await-site link owned by diagnostics, not a replacement of the original context.

IRIS-V1-ASYNC-C022: If an async body raises, the Task captures the produced `ExceptionContext` instead of throwing it synchronously to the caller that invoked the async callable. The caller receives the Task unless failure occurs before a Task object can be created, which is limited to ordinary call setup errors such as receiver or argument evaluation failures.

IRIS-V1-ASYNC-C023: Awaiting a failed Task MUST repropagate through ordinary raise/catch machinery while preserving the original captured ExceptionContext identity as the async failure root. Diagnostics MUST expose the original stack, re-raise sites, await-site chain, cause, and suppressed cleanup contexts without moving that metadata onto the raised value.

IRIS-V1-ASYNC-C024: Async stack linkage is diagnostic metadata. It MUST NOT change catch matching, raised object identity, `ExceptionContext.value`, `ExceptionContext.cause`, or `ExceptionContext.suppressed`. Public formatters MAY display async call and await boundaries using structured records.

IRIS-V1-ASYNC-C025: If an `await` occurs inside `try`, `catch`, or `finally`, suspension preserves the pending control state, active cleanup stack, active catch context for bare `raise`, lexical bindings, current receiver, and return target. Resumption MUST continue as if the callable had paused at that source point.

IRIS-V1-ASYNC-C026: Bare `raise` after resuming an awaited catch helper remains valid only while the synchronous dynamic catch extent is still active in that Task. Saving a Closure during catch and invoking it after the catch completes still raises `NoActiveExceptionError` under the control chapter.

IRIS-V1-ASYNC-C027: Unobserved failed Tasks MUST NOT disappear silently. A runtime MUST emit a structured diagnostic event when a failed Task becomes eligible for unobserved-failure reporting under the runtime's defined observation policy. Observing means awaiting the Task, explicitly querying its completion through a standard diagnostic API, or passing it to a Host bridge that records the failure.

IRIS-V1-ASYNC-C028: Unobserved-failure reporting MUST include the Task identity, Task result type, captured ExceptionContext, creation location where available, first suspension location where available, and observation state. Reporting MUST NOT mutate the Task, mark its failure as handled for future awaits, or remove the captured context.

IRIS-V1-ASYNC-C029: The following await and failure table is normative:

| Scenario                              | Await expression result     | Task completion                    | ExceptionContext behavior                      | Diagnostic behavior                |
| ------------------------------------- | --------------------------- | ---------------------------------- | ---------------------------------------------- | ---------------------------------- |
| Await successful completed Task       | Returns`T`                | Immutable success                  | None created by await                          | No failure diagnostic              |
| Await incomplete Task                 | Suspends, then returns`T` | Immutable success after completion | Pending cleanup state resumes                  | No failure diagnostic              |
| Await failed Task                     | Repropagates failure        | Immutable failed context           | Same root context, async await-site link added | Counts as observed                 |
| Async body raises and nobody observes | No await result             | Immutable failed context           | Context retained by Task until release         | Emits unobserved Task event        |
| Await operand is not Awaitable        | No suspension               | No Task completion change          | Ordinary TypeError context                     | Ordinary exception diagnostic only |

IRIS-V1-ASYNC-EX002: Informative example, repeated await observes the same completion:

```iris
async fun value() -> Integer { 7 }

let task = value()
let first = await task
let second = await task
first == second  // true
```

## Resources, Closeable, And Using

IRIS-V1-ASYNC-C030: The standard resource Contract is `contract Closeable { fun close() -> Nil }`. `close()` is an ordinary Method send. It is not syntax, not a keyword, and not a primitive bypass.

IRIS-V1-ASYNC-C031: Every standard Closeable resource close MUST be idempotent. Repeated `close()` calls after successful release MUST succeed with no additional effect and return `nil`. A first actual cleanup attempt may raise; later calls MUST NOT duplicate released-resource side effects.

IRIS-V1-ASYNC-C032: The standard ordinary helper `using(resource: Closeable, &block)` MUST invoke the block, then close the resource through `try/finally` equivalent control. It returns the block result when the block and close both complete normally. `using` is an ordinary Method or helper name and MUST remain an identifier, not a reserved word.

IRIS-V1-ASYNC-C033: If the `using` block raises and `close()` also raises, the block's ExceptionContext remains primary and the close failure context is appended to the primary context's runtime-owned suppressed list. If the block completes normally and close raises, the close failure becomes the primary propagated exception and no block result is returned.

IRIS-V1-ASYNC-C034: Async code MAY use `try/finally`, `using`, and Closeable resources across suspension. If a resource is acquired before an incomplete await inside the protected body, the runtime MUST preserve the cleanup obligation and run it exactly once when control exits the protected region, subject to idempotent close rules.

IRIS-V1-ASYNC-C035: Iterator cleanup remains governed by the Iterator and control chapters. This chapter delegates Iterator identity, equality, hashing, source retention, natural exhaustion, early close identity effects, and concrete container traversal to the collections chapter. It only requires async suspension and resource helpers to preserve the already-defined Iterator cleanup obligations.

IRIS-V1-ASYNC-C036: Language traversal over an Iterator that crosses an `await` MUST retain the active Iterator and cleanup obligation while suspended. On natural exhaustion, early loop exit, return, body exception, awaited exception, or outer unwinding after resumption, the Iterator MUST be closed according to the control chapter before the pending transfer commits.

IRIS-V1-ASYNC-C037: Cleanup remains LIFO across async suspension. Cleanup failures after resumption MUST use the same primary-plus-suppressed ordering as synchronous cleanup. A cleanup failure MUST NOT be lost because the Task was suspended, resumed, or later reported as failed.

IRIS-V1-ASYNC-C038: The following resource cleanup table is normative:

| Protected operation                        | Exit path                   | Required cleanup                          | Primary result or exception                 | Suppressed or diagnostic rule                        |
| ------------------------------------------ | --------------------------- | ----------------------------------------- | ------------------------------------------- | ---------------------------------------------------- |
| `using` block returns                    | Normal                      | Call`close()` once                      | Block value                                 | Close failure becomes primary                        |
| `using` block raises                     | Exception                   | Call`close()` once                      | Block ExceptionContext                      | Close failure appended as suppressed                 |
| Async body awaits inside using             | Suspension then normal exit | Preserve close obligation while suspended | Block value after resumption                | No diagnostic if close succeeds                      |
| Async body awaits failed Task inside using | Resumed exception           | Close before propagation                  | Awaited ExceptionContext                    | Close failure appended as suppressed                 |
| `for` body awaits then breaks            | Loop transfer               | Close Iterator before break commits       | Break value or`nil`                       | Close failure becomes primary if no exception exists |
| Task fails during cleanup                  | Exception                   | Continue LIFO cleanup                     | First cleanup failure if no earlier primary | Later failures appended in order                     |

IRIS-V1-ASYNC-EX003: Informative example, ordinary using:

```iris
let text = using(File.open("data.txt")) { |file: File| -> String
  file.read_all()
}
```

## Structured Diagnostics And Event Streams

IRIS-V1-ASYNC-C039: Ordinary `ExceptionContext` ownership remains in [04-bindings-callables-control-flow.md](04-bindings-callables-control-flow.md). This chapter uses that object for async and diagnostic event payloads but MUST NOT add mutable fields, object-attached exception metadata, or user-writable cause or suppressed edges.

IRIS-V1-ASYNC-C040: Diagnostic events that carry failures MUST carry `ExceptionContext` objects, not bare raised values, formatted strings, or host-only error records. Public `ExceptionContext` getters may be dynamically replaced for ordinary property reads, but runtime diagnostic channels MUST read protected internal records.

IRIS-V1-ASYNC-C041: `ExceptionContext` diagnostic views MUST expose typed immutable values for `value`, `cause`, `suppressed`, `original_stack`, `re_raise_sites`, and `raise_location` as defined by the control chapter. Async diagnostics MAY add immutable async stack and await-site records outside the ExceptionContext graph.

IRIS-V1-ASYNC-C042: Discarded contexts from `finally` overriding a pending exception MUST be emitted only through the protected runtime diagnostic or event stream. They MUST NOT be attached as artificial cause edges, suppressed entries, Task failures, or raised-object fields.

IRIS-V1-ASYNC-C043: Diagnostic delivery loss MUST never be silent. If an in-memory diagnostic subscriber queue drops ordinary diagnostic events because of capacity, it MUST report a structured gap or loss event to that subscriber before later events are delivered. Zero-loss requirements require a configured persistent sink.

IRIS-V1-ASYNC-C044: A diagnostic subscriber failure MUST be isolated from the runtime operation that produced the diagnostic. The failure MUST create its own ExceptionContext, be reported to the runtime event-error channel or configured handler, and MUST NOT roll back commits, complete Tasks differently, suppress other subscribers, or retry without explicit user action.

## Revision Events And History Recovery

IRIS-V1-ASYNC-C045: Iris v1 has no immediate Ruby-style meta hooks such as `included`, `method_added`, `inherited`, or equivalents during candidate construction, validation, or commit. Revision observation uses read-only after-commit events.

IRIS-V1-ASYNC-C046: After-commit revision events are publicly subscribable subject to ReflectionPolicy filtering. Event payloads MUST identify commit ID, targets, old and new revision or audit identities, change summaries, initiator, source or package metadata, and permitted structural metadata. They MUST NOT expose private Method bodies, raw private data, native secrets, inaccessible values, candidate mutation handles, or rollback authority.

IRIS-V1-ASYNC-C047: Revision events are delivered asynchronously after the structural commit returns. Structural commit publishes at safepoint, resumes the runtime, and allows open or upgrade callers to continue without waiting for subscribers. Subscriber code MUST NOT run in the safepoint or commit path.

IRIS-V1-ASYNC-C048: Immutable revision events enter runtime queues ordered by `commit_id`. For one event and commit, subscribers run in subscription order. Subscriber failures are isolated: a failure creates a full ExceptionContext, is recorded on the runtime event-error channel, is passed to the configured handler, and MUST NOT roll back the commit, affect other subscribers, propagate to the completed open or upgrade caller, or trigger automatic retry.

IRIS-V1-ASYNC-C049: The runtime MUST provide a semantic flush or wait surface for tests and controlled shutdown. The surface waits until all revision events accepted into subscriber queues before the flush boundary have either been delivered to their subscribers, converted into delivered `GapEvent` records for that subscriber, or reported through the event-error channel for subscriber failure. The surface MUST NOT run subscriber code in the original safepoint or commit path and MUST NOT roll back, retry, or reinterpret a completed commit.

IRIS-V1-ASYNC-C050: A successful flush or wait completion means the preceding delivery condition is satisfied for the selected runtime scope. If subscriber code fails during flushing, the subscriber failure remains isolated under IRIS-V1-ASYNC-C048 and is included in the event-error channel; the flush or wait surface reports that event delivery completed with recorded subscriber errors rather than propagating those subscriber exceptions as commit failure. If shutdown closes delivery before all accepted events reach a terminal delivered, gapped, or error-recorded state, the surface MUST report incomplete delivery with enough structured state for diagnostics.

IRIS-V1-ASYNC-C051: Each revision-event subscriber has a bounded queue. A slow or full subscriber MUST NOT block safepoint, commit, or open completion. Dropped event ranges MUST be coalesced into `GapEvent(from_commit, to_commit)` and delivered before later retained events for that subscriber.

IRIS-V1-ASYNC-C052: A `GapEvent` means the subscriber missed every commit event in the inclusive range `from_commit..=to_commit` that is not otherwise delivered individually. It MUST carry commit IDs, subscriber identity where visible, drop reason category, and recovery hint metadata. It MUST NOT pretend that no change occurred.

IRIS-V1-ASYNC-C053: `RevisionHistory.events(from_commit:, to_commit:) -> Iterator<RevisionAuditEvent>` returns retained lightweight audit events in commit order. If any requested portion is unavailable, it MUST raise `AuditHistoryUnavailableError` and return no partial sequence as complete.

IRIS-V1-ASYNC-C054: Iterators returned from `RevisionHistory.events` follow the canonical `Iterator<T>` and `Iteration<T>` protocols and Closeable cleanup rules. Audit-history iteration MUST close or release retained history resources on natural exhaustion, early close, loop exit, exception, and async suspension cleanup.

IRIS-V1-ASYNC-C055: Zero-loss audit requirements MUST use a separately configured persistent Host or runtime sink. An unbounded in-memory subscriber queue MUST NOT be required for conformance and MUST NOT be used as the semantic guarantee for audit recovery.

IRIS-V1-ASYNC-C056: The following revision event table is normative:

| Event path                   | Commit path behavior                             | Subscriber behavior                         | Loss behavior                                  | Recovery behavior                                                       |
| ---------------------------- | ------------------------------------------------ | ------------------------------------------- | ---------------------------------------------- | ----------------------------------------------------------------------- |
| Successful structural commit | Publishes atomically and returns without waiting | Event queued by commit ID                   | None if subscriber queue has room              | Audit event retained subject to history policy                          |
| Subscriber callback raises   | Commit remains complete                          | Failure context sent to event-error channel | Other subscribers continue                     | Handler may inspect ExceptionContext                                    |
| Subscriber queue full        | Commit still returns                             | Queue records dropped range                 | `GapEvent(from_commit, to_commit)` delivered | Subscriber may call`RevisionHistory.events`                           |
| Flush after queued events    | Commit already returned                          | Delivers queued events or gap/error records | Loss remains explicit through`GapEvent`      | Reports success, success-with-subscriber-errors, or incomplete delivery |
| History range retained       | No commit effect                                 | Iterator yields audit events in order       | None                                           | Natural exhaustion returns`Iteration.done`                            |
| History range pruned         | No commit effect                                 | No partial-complete result                  | None hidden                                    | Raises`AuditHistoryUnavailableError`                                  |

IRIS-V1-ASYNC-EX004: Informative example, gap recovery shape:

```iris
events.subscribe_revision_changes() { |event: RevisionEvent| -> Nil
  match event {
    is GapEvent gap => {
      for audit in RevisionHistory.events(from_commit: gap.from_commit, to_commit: gap.to_commit) {
        rebuild_index(audit)
      }
    }
    is RevisionAuditEvent audit => rebuild_index(audit)
  }
}
```

## Async And Diagnostics Vectors

IRIS-V1-ASYNC-C057: The following vector table is normative. The conformance chapter MUST preserve these vector IDs or map them to machine-readable records with the same observable outcomes:

| Vector ID | Category | Applicability | Source/Input | Expected observable | Decisions |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-ASYNC-V001` | positive | interpreter required; JIT required; native not applicable | `async fun f() -> Integer { 1 }` invoked | Invocation returns `Task<Integer>` and await returns `1`. | `D-487`, `D-488` |
| `IRIS-V1-ASYNC-V002` | positive | interpreter required; JIT required; native not applicable | `async fun f() -> Nil {}` invoked | Invocation returns `Task<Nil>` and await returns `nil`. | `D-488` |
| `IRIS-V1-ASYNC-V003` | diagnostic | compiler required; JIT not applicable; native not applicable | Async callable declares void-style async or unparameterized Task result | Static diagnostic rejects the signature. | `D-488` |
| `IRIS-V1-ASYNC-V004` | positive | interpreter required; JIT required; native not applicable | Async invocation reaches no incomplete await | Body completes synchronously into Task. | `D-487` |
| `IRIS-V1-ASYNC-V005` | positive | interpreter required; JIT required; native not applicable | Await already-complete Task | Await continues synchronously. | `D-487`, `D-489` |
| `IRIS-V1-ASYNC-V006` | positive | interpreter required; JIT required; native not applicable | Await incomplete Task | Current Task suspends and continuation resumes FIFO after completion post. | `D-487`, `D-490` |
| `IRIS-V1-ASYNC-V007` | diagnostic | compiler required; JIT not applicable; native not applicable | `await` outside async Method or async Closure | Static placement diagnostic. | `D-489` |
| `IRIS-V1-ASYNC-V008` | negative | interpreter required; JIT required; native not applicable | `await` operand lacks `Awaitable<T>` | `TypeError` or Contract diagnostic without suspension. | `D-489` |
| `IRIS-V1-ASYNC-V009` | negative | interpreter required; JIT required; native not applicable | Failed Task awaited twice | Same captured ExceptionContext root is observed both times. | `D-489` |
| `IRIS-V1-ASYNC-V010` | diagnostic | interpreter required; JIT required; native not applicable | Failed Task is never observed | Structured unobserved Task diagnostic is emitted. | `D-489` |
| `IRIS-V1-ASYNC-V011` | diagnostic | compiler required; JIT not applicable; native not applicable | Source tries implicit blocking wait for Task | Static or missing-member diagnostic; no blocking wait semantics. | `D-490` |
| `IRIS-V1-ASYNC-V012` | diagnostic | compiler required; JIT not applicable; native not applicable | Source treats a cooperative flag as language cancellation | Deferred-status diagnostic or ordinary library behavior only. | `D-472` |
| `IRIS-V1-ASYNC-V013` | diagnostic | compiler required; JIT not applicable; native not applicable | `await` appears in open transaction authority | Static transaction-suspension diagnostic before publication. | `D-490` |
| `IRIS-V1-ASYNC-V014` | positive | interpreter required; JIT required; native not applicable | `using` block returns and close succeeds | Helper returns block value. | `D-470` |
| `IRIS-V1-ASYNC-V015` | negative | interpreter required; JIT required; native not applicable | `using` block raises and close raises | Block context remains primary and close context is suppressed. | `D-470` |
| `IRIS-V1-ASYNC-V016` | negative | interpreter required; JIT required; native not applicable | `using` block returns and close raises | Close failure is primary and no block result returns. | `D-470` |
| `IRIS-V1-ASYNC-V017` | positive | interpreter required; JIT required; native not applicable | Async body awaits inside `using` then returns | Resource closes after resumed exit. | `D-490` |
| `IRIS-V1-ASYNC-V018` | positive | interpreter required; JIT required; native not applicable | Async `for` body awaits then breaks | Iterator closes before break commits. | `D-490` |
| `IRIS-V1-ASYNC-V019` | diagnostic | interpreter required; JIT required; native not applicable | `finally` overrides pending exception | Discarded context is emitted to diagnostics only. | `D-490` |
| `IRIS-V1-ASYNC-V020` | diagnostic | interpreter required; JIT required; native not applicable | Revision event subscriber raises | Commit remains complete and event-error channel receives ExceptionContext. | `D-324` |
| `IRIS-V1-ASYNC-V021` | positive | interpreter required; JIT required; native not applicable | Subscriber queue drops commits | `GapEvent(from_commit, to_commit)` is delivered before newer events. | `D-326` |
| `IRIS-V1-ASYNC-V022` | positive | interpreter required; JIT required; native not applicable | Flush waits after queued revision event | Reports terminal delivery after subscriber delivery, gap delivery, or recorded subscriber error. | `D-325`, `D-490` |
| `IRIS-V1-ASYNC-V023` | negative | interpreter required; JIT required; native not applicable | Shutdown closes delivery before flush terminal state | Reports incomplete delivery with structured diagnostic state. | `D-325` |
| `IRIS-V1-ASYNC-V024` | positive | interpreter required; JIT required; native not applicable | Gap consumer requests retained history | Iterator yields RevisionAuditEvent values in commit order. | `D-327` |
| `IRIS-V1-ASYNC-V025` | negative | interpreter required; JIT required; native not applicable | Gap consumer requests pruned history | `AuditHistoryUnavailableError`; no partial-complete stream. | `D-327` |

## Async Coverage Vectors

IRIS-V1-ASYNC-C062: The following vectors are normative traceability vectors with concrete async, resource, and diagnostic observations.

| Vector ID | Category | Applicability | Source/Input | Expected observable | Decisions |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-ASYNC-V026` | positive   | interpreter required; JIT required; native not applicable | `async fun value() -> Integer { 7 }; let task = value(); [task.class_name, await task, await task]`                      | `["Task", 7, 7]`; one immutable successful completion is awaited repeatedly.                              | `D-487`, `D-488`, `D-489`                                             |
| `IRIS-V1-ASYNC-V027` | positive   | interpreter required; JIT required; native not applicable | Call`using` with `CountingCloseable.new()` and a block returning `:body`, then call `resource.close()` again.      | `:body`; close count is one and repeated successful `close()` returns `nil`.                          | `D-470`, `D-471`                                                        |
| `IRIS-V1-ASYNC-V028` | diagnostic | interpreter required; JIT required; native not applicable | Configure a bounded revision subscriber, commit three revisions without draining, then flush and inspect delivered events. | `GapEvent(from_commit: 1, to_commit: 2)` precedes retained commit `3`; flush reports terminal delivery. | `D-322`, `D-323`, `D-324`, `D-325`, `D-326`, `D-327`, `D-490` |

## Traceability Notes

IRIS-V1-ASYNC-C058: This chapter owns the async, resource-helper, diagnostic-stream, and revision-event portions of D-322 through D-327, D-470 through D-472, and D-487 through D-490. It references D-127 through D-138 only for `Iterator<T>`, `Iteration<T>`, and `close()` protocol boundaries. Iterator identity, equality, hashing, source retention, and concrete traversal behavior remain delegated to the collections chapter.

IRIS-V1-ASYNC-C059: This chapter references D-139 through D-171 and D-469 through D-473 for cleanup precedence, raise/catch/finally syntax, ExceptionContext identity, cause, suppressed entries, discarded contexts, and typed immutable diagnostics. Ordinary ExceptionContext ownership remains in [04-bindings-callables-control-flow.md](04-bindings-callables-control-flow.md); this chapter preserves those rules for async suspension and event payloads.

IRIS-V1-ASYNC-C060: Chapter-owned decision IDs are `D-322`, `D-323`, `D-324`, `D-325`, `D-326`, `D-327`, `D-470`, `D-471`, `D-472`, `D-487`, `D-488`, `D-489`, and `D-490`.

IRIS-V1-ASYNC-C061: Referenced non-owned decision IDs are `D-127`, `D-128`, `D-129`, `D-130`, `D-131`, `D-132`, `D-133`, `D-134`, `D-135`, `D-136`, `D-137`, `D-138`, `D-139`, `D-140`, `D-141`, `D-142`, `D-143`, `D-144`, `D-145`, `D-146`, `D-147`, `D-148`, `D-149`, `D-150`, `D-151`, `D-152`, `D-153`, `D-154`, `D-155`, `D-156`, `D-157`, `D-158`, `D-159`, `D-160`, `D-161`, `D-162`, `D-163`, `D-164`, `D-165`, `D-166`, `D-167`, `D-168`, `D-169`, `D-170`, `D-171`, `D-469`, and `D-473`.

IRIS-V1-ASYNC-N001: Informative note: The native and Host chapters will define how external IO posts completion into the IrisRuntime. This chapter fixes only the Iris-level scheduling, Task, cleanup, and diagnostic obligations those boundaries must preserve.

## Audit-Exact Conformance Vectors

These rows are normative audit-exact vectors. Each is a minimal fixture with one concrete observable.

| Vector ID | Category | Applicability | Source/Input | Expected observable | Decisions |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-ASYNC-V073`  | positive   | interpreter required; JIT required; native not applicable    | Register revision subscriber; commit an`open` whose body appends `:commit` and subscriber appends `:event`.                                                                   | Log is`[:commit, :event]`; no subscriber invocation occurs during candidate construction or commit.                                                           | `D-322` |
| `IRIS-V1-ASYNC-V074`  | positive   | interpreter required; JIT required; native not applicable    | Subscribe without private reflection grant; commit a revision with private Method body and inspect event payload.                                                                   | Payload includes commit ID and permitted summary but no private body, raw data, or rollback handle.                                                             | `D-323` |
| `IRIS-V1-ASYNC-V075`  | diagnostic | interpreter required; JIT required; native not applicable    | First revision subscriber raises`:subscriber`; second subscriber appends `:second`; commit then flush.                                                                          | Commit succeeds, second subscriber receives event, and event-error channel contains`ExceptionContext.value == :subscriber`.                                   | `D-324` |
| `IRIS-V1-ASYNC-V076`  | positive   | interpreter required; JIT required; native not applicable    | Subscriber appends`:event`; code appends `:after_commit` immediately after structural commit and flushes.                                                                       | Log begins`[:after_commit, :event]`; delivery is asynchronous.                                                                                                | `D-325` |
| `IRIS-V1-ASYNC-V077`  | positive   | interpreter required; JIT required; native not applicable    | Bounded subscriber queue drops commits`4` through `6`, then retains commit `7`; flush.                                                                                        | Delivered sequence begins`GapEvent(4, 6)` followed by event `7`.                                                                                            | `D-326` |
| `IRIS-V1-ASYNC-V078`  | negative   | interpreter required; JIT required; native not applicable    | Retain commits`4` through `6` but prune commit `3`; request `RevisionHistory.events(from_commit: 3, to_commit: 6)`.                                                         | `AuditHistoryUnavailableError`; no prefix or suffix is returned as a complete sequence.                                                                       | `D-327` |
| `IRIS-V1-ASYNC-V079`  | positive   | interpreter required; JIT required; native not applicable    | Call`using` with `CountingCloseable.new()` and a block that accepts the resource and returns `:body`.                                                                         | `:body`; `close()` is invoked through the ordinary helper exactly once.                                                                                     | `D-470` |
| `IRIS-V1-ASYNC-V080`  | positive   | interpreter required; JIT required; native not applicable    | Independent standard-resource fixtures call`close()` twice after successful release and call it again after the first actual cleanup raises `:release_failed`.                  | Successful calls return`nil` with release count `1`; after failure, a later call returns `nil` and the released-resource side-effect count remains `1`. | `D-471` |
| `IRIS-V1-ASYNC-V081`  | positive   | interpreter required; JIT required; native not applicable    | Start async task`a`, suspend it on `Gate`; start async task `b`, suspend it on the same `Gate`; complete `Gate`.                                                          | Continuation log is`[:a, :b]`; no interleaving occurs before an incomplete `await`.                                                                         | `D-487` |
| `IRIS-V1-ASYNC-V082`  | diagnostic | compiler required; JIT not applicable; native not applicable | `async fun bad() -> Task { nil }`.                                                                                                                                                | Static generic-Task-result diagnostic; only`Task<T>` results are valid.                                                                                       | `D-488` |
| `IRIS-V1-ASYNC-V083`  | diagnostic | interpreter required; JIT required; native not applicable    | Invoke`async fun fail() -> Nil { raise :x }`, retain no observer for its failed Task, drive the scheduler to the diagnostic checkpoint, and consume the runtime diagnostic event. | Exactly one unobserved-failure diagnostic carries`ExceptionContext.value == :x` and async stack linkage; the failure is not silent.                           | `D-489` |
| `IRIS-V1-ASYNC-V084`  | positive   | interpreter required; JIT required; native not applicable    | Async`using` awaits an incomplete `Gate`, resumes, and returns `:done`.                                                                                                       | Resource closes exactly once after resumption and Task result is`:done`.                                                                                      | `D-490` |
