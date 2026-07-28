# Iris v1 Async、Resources 与 Diagnostics

状态：Iris v1 草案，语义已冻结。

IRIS-V1-ASYNC-C001: 本章定义 Iris v1 的 async Method、async Closure、`Task<T>`、`Awaitable<T>`、单一 IrisRuntime 调度器、`await`、async 异常传播、未被观察到的失败 Task 诊断、Closeable 资源、普通 `using`、Iterator 清理之间的交互、结构化诊断报告、修订事件交付、`GapEvent` 和审计历史恢复。必须在阅读 [README.md](README.md)、[01-language-identity.md](01-language-identity.md)、[03-runtime-object-model.md](03-runtime-object-model.md) 和 [04-bindings-callables-control-flow.md](04-bindings-callables-control-flow.md) 之后阅读本章。

IRIS-V1-ASYNC-C002: 本章不得定义 native no-GIL shared-memory Iris 线程、取消语义、parser 词元清单、Iterator identity 与 hashing、ordinary ExceptionContext 归属、open transaction 变更规则、Host ABI 函数名，或 实现 event-loop 内部细节。这些 surface 属于本文档命名的 identity、grammar、collections、control、metaprogramming 和 native chapters。

## Async 可调用表面

IRIS-V1-ASYNC-C003: `async fun f(parameters...) -> T { body }` 声明 await 后结果类型 `T`。调用该 Method 返回 `Task<T>`，不是 `T`。无结果 async Method 必须声明或推断 `T` 为 `Nil`，因此调用时返回 `Task<Nil>`。

IRIS-V1-ASYNC-C004: Async Closure 是头部带有 `async` modifier 的 Closure，在 grammar 允许 async Closure headers 的位置，其语义写作 `{ async |parameters| -> T body }`。求值 literal 会在 普通 Closure 捕获规则 下创建 带 identity 的 Closure object。调用它返回 `Task<T>`。

IRIS-V1-ASYNC-C005: Iris v1 没有表示 async void、non-generic `Task`、detached async result，或 fire-and-forget callable 的 source 或 reflection signature。符合规范的实现 必须拒绝这类 signatures。有意忽略 returned Task 的 programs 只能通过 普通 value discard 实现，且该 Task 的 failure diagnostics 仍遵循本章。

IRIS-V1-ASYNC-C006: `Task<T>` 是 带 identity 的 object，表示一个最终完成，完成结果是类型 `T` 的值或一个 captured `ExceptionContext`。Task 默认 equality 和 hash behavior 使用 runtime-local identity。Task identity 不依赖 最终 result value、exception value、awaiter count 或 scheduler queue position。

IRIS-V1-ASYNC-C007: `Awaitable<T>` 是 可 await 的值 以产生 `T` 的 标准 Contract。`Task<T>` 必须实现 `Awaitable<T>`。符合规范的实现 可以提供其他 standard Awaitable types，但只在它们遵守这里说明的相同 completion、exception 和 reuse rules 时才可以。

IRIS-V1-ASYNC-C008: `await expr` 的 awaited type 是来自 `Awaitable<T>` 的 `T`。静态检查 必须拒绝可证明 non-Awaitable 的 operand。动态执行 必须在 operand 在 runtime 不满足 `Awaitable<T>` 时引发 `TypeError` 或更具体的 Contract error。

IRIS-V1-ASYNC-C009: Async Method 和 async Closure bodies 使用与 ordinary callables 相同的 parameter binding、lexical scope、receiver capture、return、raise、try/catch/finally 和 Closure boundary 规则，除非本章细化 挂起行为。Async body 内的 `return expr` 在检查 await 后结果 Contract `T` 后，用该值完成其 Task。

IRIS-V1-ASYNC-C010: 下列 async callable table 是规范要求:

| 源形式 | 调用结果 | await 结果 | 身份 | 复用规则 |
| --- | --- | --- | --- | --- |
| `async fun f(...) -> T` | `Task<T>` | `T` | Method 和 returned Task 都是 identity-bearing | 该 Task 可以被 await 多次 |
| `async fun f(...) -> Nil` | `Task<Nil>` | `nil` | Method 和 returned Task 都是 identity-bearing | 该 Task 可以被 await 多次 |
| Async Closure 字面量 | `Task<T>` | `T` | Closure 和 returned Task 都是 identity-bearing | 该 Task 可以被 await 多次 |
| Ordinary `fun f(...) -> T` | `T` | 不可 await，除非 `T` 自身实现 `Awaitable<U>` | 遵循运行时章节的 Method 规则 | 普通调用结果规则 |

IRIS-V1-ASYNC-EX001: 信息性示例，结果类型:

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

## 单 Runtime 调度器

IRIS-V1-ASYNC-C011: 一个 IrisRuntime 在单一协作式调度器和共享堆上执行 Iris 代码。Iris Task 的交错执行只能发生在对未完成 Awaitable 执行 `await` 时，或发生在后续 Host 或 runtime APIs 明确定义的调度器边界处。普通表达式求值和 Method 分派不得在这些边界之间被另一个 Iris Task 抢占。

IRIS-V1-ASYNC-C012: 调用 async Method 或 async Closure 会在当前 IrisRuntime 上同步开始执行。Body 运行到完成、raise，或到达第一个其 Awaitable incomplete 的 `await`。从 caller 角度看，创建 returned Task 和开始这次 初始运行 是一个 call operation。

IRIS-V1-ASYNC-C013: await 已完成 Awaitable 必须在 当前 Task 中同步继续，不得仅为 fairness 而 enqueue later continuation。Awaiting 未完成 Awaitable 必须注册 当前 continuation，suspend 当前 Task，并将控制权交回 scheduler 或 Host event-loop driver。

IRIS-V1-ASYNC-C014: Iris runtime actions 创建的 ready continuations 必须以 确定性 FIFO 顺序 enqueue。外部 IO 或 Host completions 按 Host 将它们 post 到 IrisRuntime 的顺序进入 scheduler。Iris v1 不承诺 独立 external completions 在 post 前的顺序。

IRIS-V1-ASYNC-C015: Iris 代码 没有 隐式阻塞等待、隐藏同步 Task join、sleep-until-completion primitive，或会阻塞直到 Task 完成的 property read。Host APIs 可以按 native chapter drive 或 run event loop，但该 Host control surface 不是 Iris source-level blocking wait。

IRIS-V1-ASYNC-C016: Native no-GIL shared-memory Iris threads 是 `DEFERRED V1`。符合规范的 v1 实现 不得暴露需要多个 Iris threads 在同一 managed heap 上 racing 的 语言语义，implementation architecture 也不得要求 permanent global interpreter lock 作为 semantic 或 ABI premise。本 clause 不定义 no-GIL memory model、data-race behavior、atomicity rule 或 shared-memory thread API。未来 semantics 仍未指定。

IRIS-V1-ASYNC-C017: Task、thread 和 async 取消语义 是 `DEFERRED V1`。Iris v1 不定义 取消类型、取消点、masking rule、asynchronous interruption rule、`CancellationError` Contract，或 Host 注入 unwinding model。Cooperative flags 是普通 library 或 application values，不改变 `await`、`finally`、`using` 或 Iterator cleanup 的 语言语义。

IRIS-V1-ASYNC-C018: open 或 revision transactions 内禁止 await、scheduler yield、thread transfer 和 escaping transaction capability。任何 lexically inside open transaction body 的 `await` expression，或位于可 escape 并带 transaction authority resume 的 callable 内的 `await` expression，都必须在执行前被拒绝。Metaprogramming chapter 拥有 transaction shape 和 authority rules。本章拥有该 prohibition 的 async suspension reason。

IRIS-V1-ASYNC-C019: 下列 scheduler state table 是规范要求:

| 事件 | 当前 Task 状态 | 调度器动作 | 可观察结果 | 错误或诊断规则 |
| --- | --- | --- | --- | --- |
| async 调用开始 | 新 Task 同步运行 | 第一次 incomplete await 前无动作 | 初始运行到达边界或完成后，调用方收到 Task object | 主体设置失败会将 Task 完成为 failed |
| 主体在挂起前返回 | 已带值完成 | 无 continuation | await 方收到同一个 immutable value | 返回 Contract 失败会将 Task 完成为 failed |
| 主体在挂起前 raise | 已失败完成 | 无 continuation | await 重新传播 captured ExceptionContext | unobserved failure 诊断之后可以触发 |
| `await` 已完成 Awaitable | 仍在运行 | 不需要 enqueue | await expression 立即产生值或重新抛出 | 被 await 的 failure 链接 async stack |
| `await` 未完成 Awaitable | 已挂起 | 注册 continuation 并交出 runtime 控制权 | scheduler 调用方可以运行另一个 ready Task | operand Contract 失败阻止挂起 |
| Awaitable 稍后完成 | Ready | post 后按 FIFO enqueue continuation | continuation 在 await site 恢复 | failure 通过重新传播恢复 |

## Await 完成与异常

IRIS-V1-ASYNC-C020: Task 的完成结果是 immutable。一旦 Task 产生 value 或 captured `ExceptionContext`，后续 awaits、diagnostic observers 和 Host bridges 必须观察同一 completion category 和同一 value 或 context identity。

IRIS-V1-ASYNC-C021: `Task<T>` 可以被 await 多次。对已完成 Task 的每次 successful await，都必须按照普通的值身份或值语义返回同一个完成值。每次 failed await 都会重新传播同一个 captured ExceptionContext，并添加由 diagnostics 拥有的 async await-site link，而不是替换原始 context。

IRIS-V1-ASYNC-C022: 如果 async body raises，Task 捕获生成的 `ExceptionContext`，而不是将它同步 throw 给调用 async callable 的 caller。Caller 会收到 Task，除非 failure 发生在 Task object 可创建之前，这只限 ordinary call setup errors，例如 receiver 或 argument evaluation failures。

IRIS-V1-ASYNC-C023: Awaiting failed Task 必须通过 ordinary raise/catch machinery repropagate，同时保留 original captured ExceptionContext identity 作为 async failure root。Diagnostics 必须暴露 original stack、re-raise sites、await-site chain、cause 和 suppressed cleanup contexts，不得将这些 metadata 移到 raised value 上。

IRIS-V1-ASYNC-C024: Async stack linkage 是 diagnostic metadata。它不得改变 catch matching、raised object identity、`ExceptionContext.value`、`ExceptionContext.cause` 或 `ExceptionContext.suppressed`。Public formatters 可以使用 structured records 显示 async call 和 await 边界。

IRIS-V1-ASYNC-C025: 如果 `await` 发生在 `try`、`catch` 或 `finally` 内，suspension 会保留 pending control state、active cleanup stack、bare `raise` 的 active catch context、lexical bindings、current receiver 和 return target。Resumption 必须像 callable 在该 source point 暂停后继续一样进行。

IRIS-V1-ASYNC-C026: Resuming awaited catch helper 后的 bare `raise` 只在 synchronous dynamic catch extent 仍在该 Task 中 active 时有效。在 catch 中保存 Closure 并在 catch 完成后调用它，仍会按 control chapter 引发 `NoActiveExceptionError`。

IRIS-V1-ASYNC-C027: Unobserved failed Tasks 不得 silent disappear。Runtime 必须在 failed Task 根据 runtime 定义的 observation policy 有资格进行 unobserved-failure reporting 时发出 structured diagnostic event。Observing 指 await Task、通过 standard diagnostic API 显式查询其 completion，或将其传给记录 failure 的 Host bridge。

IRIS-V1-ASYNC-C028: Unobserved-failure reporting 必须包括 Task identity、Task result type、captured ExceptionContext、可用时的 creation location、可用时的 first suspension location，以及 observation state。Reporting 不得 mutate Task、将其 failure 标为 handled 以影响 future awaits，或移除 captured context。

IRIS-V1-ASYNC-C029: 下列 await and failure table 是规范要求:

| 场景 | await expression 结果 | Task 完成状态 | ExceptionContext 行为 | 诊断行为 |
| --- | --- | --- | --- | --- |
| await 成功完成的 Task | 返回 `T` | immutable success | await 不创建任何项 | 无 failure diagnostic |
| await incomplete Task | 挂起，然后返回 `T` | 完成后 immutable success | pending cleanup state 恢复 | 无 failure diagnostic |
| await failed Task | 重新传播 failure | immutable failed context | 同一个 root context，并添加 async await-site link | 计为已观察 |
| async body raises 且无人观察 | 无 await 结果 | immutable failed context | context 由 Task 保留直到释放 | 发出 unobserved Task event |
| await operand 不是 Awaitable | 不挂起 | Task 完成状态不变 | ordinary TypeError context | 只有 ordinary exception diagnostic |

IRIS-V1-ASYNC-EX002: 信息性示例，重复 await 观察同一次完成:

```iris
async fun value() -> Integer { 7 }

let task = value()
let first = await task
let second = await task
first == second  // true
```

## 资源、Closeable 与 Using

IRIS-V1-ASYNC-C030: 标准资源 Contract 是 `contract Closeable { fun close() -> Nil }`。`close()` 是普通 Method 发送。它不是语法形式，不是关键字，也不是绕过普通调用机制的 primitive。

IRIS-V1-ASYNC-C031: 每个 standard Closeable resource close 必须 idempotent。Successful release 后重复 `close()` calls 必须成功且无额外 effect，并返回 `nil`。第一次 actual cleanup attempt 可以 raise。后续 calls 不得 duplicate released-resource side effects。

IRIS-V1-ASYNC-C032: Standard ordinary helper `using(resource: Closeable, &block)` 必须调用 block，然后通过等价于 `try/finally` 的 control close resource。当 block 和 close 都正常完成时，它返回 block result。`using` 是 ordinary Method 或 helper name，且必须保持为 identifier，不是 reserved word。

IRIS-V1-ASYNC-C033: 如果 `using` block raises 且 `close()` 也 raises，block 的 ExceptionContext 保持 primary，close failure context 被追加到 primary context 的 runtime-owned suppressed list。如果 block 正常完成而 close raises，close failure 成为 primary propagated exception，且不返回 block result。

IRIS-V1-ASYNC-C034: Async code 可以跨 suspension 使用 `try/finally`、`using` 和 Closeable resources。如果 resource 在 protected body 内 incomplete await 之前 acquired，runtime 必须保留 cleanup obligation，并在 control 退出 protected region 时正好运行一次，受 idempotent close rules 约束。

IRIS-V1-ASYNC-C035: Iterator cleanup 仍由 Iterator 和 control chapters 管辖。本章将 Iterator identity、equality、hashing、source retention、natural exhaustion、early close identity effects 和 concrete container traversal 委托给 collections chapter。它只要求 async suspension 和 resource helpers 保留已经定义的 Iterator cleanup obligations。

IRIS-V1-ASYNC-C036: 跨越 `await` 的 Iterator 上的 language traversal 必须在 suspended 期间保留 active Iterator 和 cleanup obligation。在 natural exhaustion、early loop exit、return、body exception、awaited exception，或 resumption 后的 outer unwinding 上，Iterator 必须按 control chapter 在 pending transfer commit 前 closed。

IRIS-V1-ASYNC-C037: Cleanup 在 async suspension 之间仍保持 LIFO。Resumption 后的 cleanup failures 必须使用与 synchronous cleanup 相同的 primary-plus-suppressed ordering。Cleanup failure 不得因为 Task 曾 suspended、resumed 或随后被报告 failed 而丢失。

IRIS-V1-ASYNC-C038: 下列 resource cleanup table 是规范要求:

| 受保护操作 | 退出路径 | 必需清理 | primary 结果或异常 | suppressed 或诊断规则 |
| --- | --- | --- | --- | --- |
| `using` block 返回 | 正常 | 调用一次 `close()` | block value | close failure 成为 primary |
| `using` block raises | 异常 | 调用一次 `close()` | block ExceptionContext | close failure 追加为 suppressed |
| Async body 在 using 内 await | 先挂起后正常退出 | 挂起期间保留 close obligation | 恢复后的 block value | close 成功则无诊断 |
| Async body 在 using 内 await failed Task | 恢复为异常 | 传播前 close | 被 await 的 ExceptionContext | close failure 追加为 suppressed |
| `for` body await 后 break | loop transfer | break 提交前 close Iterator | break value 或 `nil` | 若不存在异常，close failure 成为 primary |
| Task 在 cleanup 期间失败 | 异常 | 继续 LIFO cleanup | 如果没有更早 primary，则第一项 cleanup failure 为 primary | 后续 failure 按顺序追加 |

IRIS-V1-ASYNC-EX003: 信息性示例，普通 using:

```iris
let text = using(File.open("data.txt")) { |file: File| -> String
  file.read_all()
}
```

## 结构化诊断与事件流

IRIS-V1-ASYNC-C039: Ordinary `ExceptionContext` ownership 仍在 [04-bindings-callables-control-flow.md](04-bindings-callables-control-flow.md)。本章将该 object 用于 async 和 diagnostic event payloads，但不得添加 mutable fields、object-attached exception metadata，或 user-writable cause or suppressed edges。

IRIS-V1-ASYNC-C040: 携带 failures 的 diagnostic events 必须携带 `ExceptionContext` objects，而不是 bare raised values、formatted strings 或 host-only error records。Public `ExceptionContext` getters 可以为 ordinary property reads 被 dynamically replaced，但 runtime diagnostic channels 必须读取 protected internal records。

IRIS-V1-ASYNC-C041: `ExceptionContext` diagnostic views 必须暴露 control chapter 定义的 `value`、`cause`、`suppressed`、`original_stack`、`re_raise_sites` 和 `raise_location` 的 typed immutable values。Async diagnostics 可以在 ExceptionContext graph 外添加 immutable async stack 和 await-site records。

IRIS-V1-ASYNC-C042: `finally` 覆盖 pending exception 后丢弃的 contexts 只能通过 protected runtime diagnostic 或 event stream 发出。它们不得作为 artificial cause edges、suppressed entries、Task failures 或 raised-object fields 附加。

IRIS-V1-ASYNC-C043: Diagnostic delivery loss 必须绝不 silent。如果 in-memory diagnostic subscriber queue 因 capacity 丢弃 ordinary diagnostic events，它必须在 later events delivered 前向该 subscriber 报告 structured gap 或 loss event。Zero-loss requirements 需要 configured persistent sink。

IRIS-V1-ASYNC-C044: Diagnostic subscriber failure 必须与产生 diagnostic 的 runtime operation 隔离。该 failure 必须创建自己的 ExceptionContext，报告到 runtime event-error channel 或 configured handler，且不得 roll back commits、以不同方式 完成 Tasks、suppress other subscribers，或在没有 explicit user action 的情况下 retry。

## 修订事件与历史恢复

IRIS-V1-ASYNC-C045: Iris v1 没有 Ruby-style immediate meta hooks，例如在 candidate construction、validation 或 commit 期间的 `included`、`method_added`、`inherited` 或 equivalents。Revision observation 使用 read-only after-commit events。

IRIS-V1-ASYNC-C046: After-commit revision events 在受 ReflectionPolicy filtering 约束下可被 public subscribe。Event payloads 必须标识 commit ID、targets、old and new revision or audit identities、change summaries、initiator、source or package metadata，以及 permitted structural metadata。它们不得暴露 private Method bodies、raw private data、native secrets、inaccessible values、candidate mutation handles 或 rollback authority。

IRIS-V1-ASYNC-C047: Revision events 在 structural commit 返回后异步 delivered。Structural commit 在 safepoint publish，resume runtime，并允许 open 或 upgrade callers 继续而不等待 subscribers。Subscriber code 不得在 safepoint 或 commit path 中运行。

IRIS-V1-ASYNC-C048: 不可变 revision events 按 `commit_id` order 进入 runtime queues。对一个 event 和 commit，subscribers 按 subscription order 运行。Subscriber failures 被隔离: failure 创建 full ExceptionContext，记录到 runtime event-error channel，传给 configured handler，且不得 roll back commit、影响 other subscribers、propagate 到已完成的 open 或 upgrade caller，或触发 automatic retry。

IRIS-V1-ASYNC-C049: Runtime 必须为 tests 和 controlled shutdown 提供 semantic flush 或 wait surface。该 surface 等待 flush boundary 前被接受进 subscriber queues 的所有 revision events 已 delivered 给其 subscribers、转换为该 subscriber 的 delivered `GapEvent` records，或因 subscriber failure 通过 event-error channel 报告。该 surface 不得在 original safepoint 或 commit path 中运行 subscriber code，且不得 roll back、retry 或 reinterpret 已完成 commit。

IRIS-V1-ASYNC-C050: Successful flush 或 wait completion 表示 selected runtime scope 满足 preceding delivery condition。如果 subscriber code 在 flushing 期间失败，该 subscriber failure 仍按 IRIS-V1-ASYNC-C048 隔离，并包括在 event-error channel 中。Flush 或 wait surface 报告 event delivery completed with recorded subscriber errors，而不是将 subscriber exceptions 作为 commit failure 传播。如果 shutdown 在所有 accepted events 到达 terminal delivered、gapped 或 error-recorded state 之前关闭 delivery，该 surface 必须报告 incomplete delivery，并带有足够 structured state 供 diagnostics 使用。

IRIS-V1-ASYNC-C051: 每个 revision-event subscriber 有 bounded queue。Slow 或 full subscriber 不得 block safepoint、commit 或 open completion。Dropped event ranges 必须 coalesced into `GapEvent(from_commit, to_commit)`，并在该 subscriber 的 later retained events 之前 delivered。

IRIS-V1-ASYNC-C052: `GapEvent` 表示 subscriber missed inclusive range `from_commit..=to_commit` 中每个未单独 delivered 的 commit event。它必须携带 commit IDs、可见时的 subscriber identity、drop reason category 和 recovery hint metadata。它不得假装没有发生 change。

IRIS-V1-ASYNC-C053: `RevisionHistory.events(from_commit:, to_commit:) -> Iterator<RevisionAuditEvent>` 按 commit order 返回 retained lightweight audit events。如果请求范围中任何部分不可用，它必须引发 `AuditHistoryUnavailableError`，且不得把部分序列当作完整结果返回。

IRIS-V1-ASYNC-C054: `RevisionHistory.events` 返回的 Iterators 遵循 canonical `Iterator<T>` 和 `Iteration<T>` protocols，以及 Closeable cleanup rules。Audit-history iteration 必须在 natural exhaustion、early close、loop exit、exception 和 async suspension cleanup 时 close 或 release retained history resources。

IRIS-V1-ASYNC-C055: Zero-loss audit requirements 必须使用单独 configured persistent Host 或 runtime sink。Conformance 不得要求 unbounded in-memory subscriber queue，也不得将它用作 audit recovery 的 semantic guarantee。

IRIS-V1-ASYNC-C056: 下列 revision event table 是规范要求:

| 事件路径 | commit path 行为 | subscriber 行为 | 丢失行为 | 恢复行为 |
| --- | --- | --- | --- | --- |
| successful structural commit | 原子发布并不等待即返回 | event 按 commit ID 入队 | subscriber queue 有空间时无丢失 | audit event 按 history policy 保留 |
| Subscriber callback raises | commit 保持完成 | failure context 发送到 event-error channel | 其他 subscribers 继续 | handler 可以检查 ExceptionContext |
| subscriber queue full | commit 仍返回 | queue 记录 dropped range | delivery `GapEvent(from_commit, to_commit)` | subscriber 可以调用 `RevisionHistory.events` |
| queued events 后 flush | commit 已返回 | 交付 queued events 或 gap/error records | loss 通过 `GapEvent` 保持显式 | 报告 success、success-with-subscriber-errors 或 incomplete delivery |
| history range retained | 无 commit effect | Iterator 按顺序 yield audit events | 无 | natural exhaustion 返回 `Iteration.done` |
| history range pruned | 无 commit effect | 没有 partial-完成 result | 没有隐藏项 | 引发 `AuditHistoryUnavailableError` |

IRIS-V1-ASYNC-EX004: 信息性示例，缺口恢复形态:

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

## Async 与 Diagnostics 向量

IRIS-V1-ASYNC-C057: 下列 vector table 是规范要求。Conformance chapter 必须保留这些 vector IDs，或将其映射到具有相同 observable outcomes 的 machine-readable records:

| 向量 ID | 类别 | 适用性 | Source/Input | 期望可观察结果 | 决策 |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-ASYNC-V001` | positive | 需要 interpreter; 需要 JIT; native 不适用 | `async fun f() -> Integer { 1 }` invoked | 调用返回 `Task<Integer>` 且 await 返回 `1`. | `D-487`, `D-488` |
| `IRIS-V1-ASYNC-V002` | positive | 需要 interpreter; 需要 JIT; native 不适用 | `async fun f() -> Nil {}` invoked | 调用返回 `Task<Nil>` 且 await 返回 `nil`. | `D-488` |
| `IRIS-V1-ASYNC-V003` | diagnostic | 需要 compiler; JIT 不适用; native 不适用 | Async callable declares void-style async or unparameterized Task result | 静态诊断拒绝该签名。 | `D-488` |
| `IRIS-V1-ASYNC-V004` | positive | 需要 interpreter; 需要 JIT; native 不适用 | async invocation 未到达 incomplete await | 主体同步完成到 Task。 | `D-487` |
| `IRIS-V1-ASYNC-V005` | positive | 需要 interpreter; 需要 JIT; native 不适用 | Await already-完成 Task | await 同步继续。 | `D-487`, `D-489` |
| `IRIS-V1-ASYNC-V006` | positive | 需要 interpreter; 需要 JIT; native 不适用 | await incomplete Task | 当前 Task 挂起，完成发布后 continuation 按 FIFO 恢复。 | `D-487`, `D-490` |
| `IRIS-V1-ASYNC-V007` | diagnostic | 需要 compiler; JIT 不适用; native 不适用 | `await` outside async Method or async Closure | 静态位置诊断。 | `D-489` |
| `IRIS-V1-ASYNC-V008` | negative | 需要 interpreter; 需要 JIT; native 不适用 | `await` operand lacks `Awaitable<T>` | `TypeError` or Contract 诊断且不发生挂起。 | `D-489` |
| `IRIS-V1-ASYNC-V009` | negative | 需要 interpreter; 需要 JIT; native 不适用 | failed Task 被 await 两次 | 两次都观察到同一个 captured ExceptionContext root。 | `D-489` |
| `IRIS-V1-ASYNC-V010` | diagnostic | 需要 interpreter; 需要 JIT; native 不适用 | failed Task 从未被观察 | 发出结构化 unobserved Task 诊断。 | `D-489` |
| `IRIS-V1-ASYNC-V011` | diagnostic | 需要 compiler; JIT 不适用; native 不适用 | Source tries 隐式阻塞等待 for Task | 静态或缺失成员诊断；没有阻塞等待语义。 | `D-490` |
| `IRIS-V1-ASYNC-V012` | diagnostic | 需要 compiler; JIT 不适用; native 不适用 | source 将 cooperative flag 当作语言级取消 | 仅有延后状态诊断或普通库行为。 | `D-472` |
| `IRIS-V1-ASYNC-V013` | diagnostic | 需要 compiler; JIT 不适用; native 不适用 | `await` appears in open transaction authority | 发布前给出静态事务挂起诊断。 | `D-490` |
| `IRIS-V1-ASYNC-V014` | positive | 需要 interpreter; 需要 JIT; native 不适用 | `using` block returns and close succeeds | helper 返回 block value。 | `D-470` |
| `IRIS-V1-ASYNC-V015` | negative | 需要 interpreter; 需要 JIT; native 不适用 | `using` block raises and close raises | block context 保持 primary，close context 被 suppressed。 | `D-470` |
| `IRIS-V1-ASYNC-V016` | negative | 需要 interpreter; 需要 JIT; native 不适用 | `using` block returns and close raises | close failure 是 primary，且不返回 block result。 | `D-470` |
| `IRIS-V1-ASYNC-V017` | positive | 需要 interpreter; 需要 JIT; native 不适用 | Async body awaits inside `using` then returns | 恢复后退出时关闭 resource。 | `D-490` |
| `IRIS-V1-ASYNC-V018` | positive | 需要 interpreter; 需要 JIT; native 不适用 | Async `for` body awaits then breaks | break 提交前关闭 Iterator。 | `D-490` |
| `IRIS-V1-ASYNC-V019` | diagnostic | 需要 interpreter; 需要 JIT; native 不适用 | `finally` overrides pending exception | 丢弃的 context 只发送到 diagnostics。 | `D-490` |
| `IRIS-V1-ASYNC-V020` | diagnostic | 需要 interpreter; 需要 JIT; native 不适用 | revision event subscriber raises | commit 保持完成，event-error channel 接收 ExceptionContext。 | `D-324` |
| `IRIS-V1-ASYNC-V021` | positive | 需要 interpreter; 需要 JIT; native 不适用 | subscriber queue drops commits | `GapEvent(from_commit, to_commit)` 在 newer events 之前 delivered. | `D-326` |
| `IRIS-V1-ASYNC-V022` | positive | 需要 interpreter; 需要 JIT; native 不适用 | queued revision event 后 flush 等待 | 在 subscriber delivery、gap delivery 或记录 subscriber error 后报告终止交付。 | `D-325`, `D-490` |
| `IRIS-V1-ASYNC-V023` | negative | 需要 interpreter; 需要 JIT; native 不适用 | shutdown 在 flush terminal state 前关闭 delivery | 报告 incomplete delivery，并带结构化诊断状态。 | `D-325` |
| `IRIS-V1-ASYNC-V024` | positive | 需要 interpreter; 需要 JIT; native 不适用 | gap consumer 请求 retained history | Iterator 按 commit 顺序 yield RevisionAuditEvent values。 | `D-327` |
| `IRIS-V1-ASYNC-V025` | negative | 需要 interpreter; 需要 JIT; native 不适用 | gap consumer 请求 pruned history | `AuditHistoryUnavailableError`; 没有 partial-完成 stream。 | `D-327` |

## Async 覆盖向量

IRIS-V1-ASYNC-C062: 下列向量是规范性可追溯向量，带有具体的 async、resource 和 diagnostic 观察。

| 向量 ID | 类别 | 适用性 | Source/Input | 期望可观察结果 | 决策 |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-ASYNC-V026` | positive | 需要 interpreter; 需要 JIT; native 不适用 | `async fun value() -> Integer { 7 }; let task = value(); [task.class_name, await task, await task]` | `["Task", 7, 7]`; 一个 immutable successful completion 被重复 await. | `D-487`, `D-488`, `D-489` |
| `IRIS-V1-ASYNC-V027` | positive | 需要 interpreter; 需要 JIT; native 不适用 | 以 `CountingCloseable.new()` 调用 `using`，block 返回 `:body`, then call `resource.close()` again. | `:body`; close count 为 one，重复 successful `close()` returns `nil`. | `D-470`, `D-471` |
| `IRIS-V1-ASYNC-V028` | diagnostic | 需要 interpreter; 需要 JIT; native 不适用 | 配置 bounded revision subscriber，提交三个 revisions 而不 drain，然后 flush 并检查 delivered events. | `GapEvent(from_commit: 1, to_commit: 2)` 位于 retained commit 之前 `3`; flush 报告 terminal delivery. | `D-322`, `D-323`, `D-324`, `D-325`, `D-326`, `D-327`, `D-490` |

## 可追溯性说明

IRIS-V1-ASYNC-C058: 本章拥有 D-322 through D-327、D-470 through D-472 和 D-487 through D-490 的 async、resource-helper、diagnostic-stream 和 revision-event 部分。它只为 `Iterator<T>`、`Iteration<T>` 和 `close()` 协议边界引用 D-127 through D-138。Iterator identity、equality、hashing、source retention 和 concrete traversal behavior 仍委托给 collections chapter。

IRIS-V1-ASYNC-C059: 本章引用 D-139 through D-171 和 D-469 through D-473，涉及 cleanup precedence、raise/catch/finally 语法、ExceptionContext identity、cause、suppressed entries、discarded contexts 和 typed immutable diagnostics。Ordinary ExceptionContext ownership 仍在 [04-bindings-callables-control-flow.md](04-bindings-callables-control-flow.md)。本章为 async suspension 和 event payloads 保留这些规则。

IRIS-V1-ASYNC-C060: 本章拥有的决策 ID 为 `D-322`, `D-323`, `D-324`, `D-325`, `D-326`, `D-327`, `D-470`, `D-471`, `D-472`, `D-487`, `D-488`, `D-489`, and `D-490`.

IRIS-V1-ASYNC-C061: 引用但不归本章拥有的决策 ID 为 `D-127`, `D-128`, `D-129`, `D-130`, `D-131`, `D-132`, `D-133`, `D-134`, `D-135`, `D-136`, `D-137`, `D-138`, `D-139`, `D-140`, `D-141`, `D-142`, `D-143`, `D-144`, `D-145`, `D-146`, `D-147`, `D-148`, `D-149`, `D-150`, `D-151`, `D-152`, `D-153`, `D-154`, `D-155`, `D-156`, `D-157`, `D-158`, `D-159`, `D-160`, `D-161`, `D-162`, `D-163`, `D-164`, `D-165`, `D-166`, `D-167`, `D-168`, `D-169`, `D-170`, `D-171`, `D-469`, and `D-473`.

IRIS-V1-ASYNC-N001: 信息性说明：Native 和 Host 章节将定义外部 IO 如何把完成事件发布到 IrisRuntime。本章只固定这些边界必须保留的 Iris 层调度、Task、清理和诊断义务。

## 审计精确一致性向量

这些行是规范性的审计精确向量。每一行都是带有一个具体可观察结果的最小 fixture。

| 向量 ID | 类别 | 适用性 | Source/Input | 期望可观察结果 | 决策 |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-ASYNC-V073` | positive | 需要 interpreter; 需要 JIT; native 不适用 | 注册 revision subscriber；commit 一个 `open` 其 body append `:commit` 且 subscriber append `:event`. | Log 为 `[:commit, :event]`; candidate construction 或 commit 期间不发生 subscriber invocation. | `D-322` |
| `IRIS-V1-ASYNC-V074` | positive | 需要 interpreter; 需要 JIT; native 不适用 | 在没有 private reflection grant 的情况下 subscribe；commit 一个带 private Method body 的 revision 并检查 event payload. | Payload 包含 commit ID 和 permitted summary，但不包含 private body、raw data 或 rollback handle. | `D-323` |
| `IRIS-V1-ASYNC-V075` | diagnostic | 需要 interpreter; 需要 JIT; native 不适用 | 第一个 revision subscriber raises `:subscriber`; 第二个 subscriber appends `:second`; commit 然后 flush. | Commit 成功，第二个 subscriber 收到 event，且 event-error channel 包含 `ExceptionContext.value == :subscriber`. | `D-324` |
| `IRIS-V1-ASYNC-V076` | positive | 需要 interpreter; 需要 JIT; native 不适用 | Subscriber appends `:event`; code appends `:after_commit` 紧接 structural commit 后并 flush. | Log 开始为 `[:after_commit, :event]`; delivery 是 asynchronous. | `D-325` |
| `IRIS-V1-ASYNC-V077` | positive | 需要 interpreter; 需要 JIT; native 不适用 | bounded subscriber queue drops commits `4` through `6`, 然后 retains commit `7`; flush. | delivered sequence 开始为 `GapEvent(4, 6)` 后接 event `7`. | `D-326` |
| `IRIS-V1-ASYNC-V078` | negative | 需要 interpreter; 需要 JIT; native 不适用 | 保留 commits `4` through `6` 但 prune commit `3`; 请求 `RevisionHistory.events(from_commit: 3, to_commit: 6)`. | `AuditHistoryUnavailableError`; 不把 prefix 或 suffix 作为 完成 sequence 返回. | `D-327` |
| `IRIS-V1-ASYNC-V079` | positive | 需要 interpreter; 需要 JIT; native 不适用 | 以 `CountingCloseable.new()` and a block that accepts the resource and returns `:body`. | `:body`; `close()` 通过 ordinary helper 精确调用一次. | `D-470` |
| `IRIS-V1-ASYNC-V080` | positive | 需要 interpreter; 需要 JIT; native 不适用 | 独立 standard-resource fixtures 在 successful release 后调用 `close()` 两次，并在第一次 actual cleanup 引发 `:release_failed` 后再次调用它。 | Successful calls return `nil` release count 为 `1`; after failure, a later call returns `nil` 且 released-resource side-effect count 保持为 `1`. | `D-471` |
| `IRIS-V1-ASYNC-V081` | positive | 需要 interpreter; 需要 JIT; native 不适用 | 启动 async task `a`, 让它在 `Gate`; 启动 async task `b`, 让它在 the same `Gate`; 完成 `Gate`. | Continuation log 为 `[:a, :b]`; no interleaving occurs before an incomplete `await`. | `D-487` |
| `IRIS-V1-ASYNC-V082` | diagnostic | 需要 compiler; JIT 不适用; native 不适用 | `async fun bad() -> Task { nil }`. | static generic-Task-result diagnostic；只有 `Task<T>` 结果为 valid. | `D-488` |
| `IRIS-V1-ASYNC-V083` | diagnostic | 需要 interpreter; 需要 JIT; native 不适用 | 调用 `async fun fail() -> Nil { raise :x }`, 不保留其 failed Task 的 observer，驱动 scheduler 到 diagnostic checkpoint，并消费 runtime diagnostic event. | 恰好一个 unobserved-failure diagnostic 携带 `ExceptionContext.value == :x` 以及 async stack linkage；该 failure 不是 silent. | `D-489` |
| `IRIS-V1-ASYNC-V084` | positive | 需要 interpreter; 需要 JIT; native 不适用 | Async `using` awaits an incomplete `Gate`, 恢复并返回 `:done`. | resumption 后 resource 精确关闭一次，Task result 为 `:done`. | `D-490` |
