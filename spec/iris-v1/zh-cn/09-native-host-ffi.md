# Iris v1 原生 Host 与 FFI

Status: Iris v1 draft, frozen semantics.

IRIS-V1-FFI-C001: 本章定义 Iris v1 的稳定 Host ABI、原生扩展边界、运行时有根句柄模型、原生错误模型、原生元数据绑定、原生载荷所有权、异步完成桥、ABI 版本协商、脚本 FFI 表面，以及 Rust/C++ 集成规则。它 MUST 在阅读 [README.md](README.md)、[01-language-identity.md](01-language-identity.md)、[03-runtime-object-model.md](03-runtime-object-model.md)、[05-types-contracts-generics.md](05-types-contracts-generics.md)、[07-async-resources-diagnostics.md](07-async-resources-diagnostics.md) 和 [08-modules-metaprogramming.md](08-modules-metaprogramming.md) 之后阅读。

IRIS-V1-FFI-C002: 本章 MUST NOT 定义解析器产生式、实现函数名、内部对象布局、Rust crate API、C++ class API、GC 算法、OS 加载器内部、密码学 API、通用网络 API，或标准 `FFI` 子系统以外的任何脚本二进制路径。它只定义可观察边界契约。

## 稳定边界角色

IRIS-V1-FFI-C003: 稳定的 Iris v1 Host/extension binary ABI 是 C ABI。Rust、C++、Zig、C#、Swift 或其他语言绑定 MAY 包装该 ABI，但它们 MUST NOT 成为二进制兼容身份。符合要求的实现 MUST NOT 要求消费者为了 Iris v1 兼容性链接 Rust ABI、C++ ABI、内部对象 ABI、vtable layout、exception ABI、allocator ABI 或托管对象布局。

IRIS-V1-FFI-C004: Host ABI 覆盖运行时生命周期、包与原生制品加载、值句柄、调用、回调、元数据注册、扩展入口、诊断和异步投递。与运行时集成的原生扩展 MUST 通过此 Host ABI，或通过机械绑定到此 ABI 的包装层进入。普通 Iris 脚本 MUST NOT 直接调用 Host ABI tables、extension tables、raw loader APIs 或 runtime handles。

IRIS-V1-FFI-C005: 脚本发起的外部二进制集成路径恰好只有一条: 标准 `FFI` 子系统。`FFI.open` 创建 `FFI::Library`，所有由脚本发起的外部动态库调用都通过该 Library 上绑定的 Methods 发生。任何替代脚本 API 都不得暴露任意 native symbols、Host ABI functions、extension tables、raw runtime handles 或 loader internals。

IRIS-V1-FFI-C006: 以下边界角色表是规范性的:

| 边界角色 | 稳定二进制契约 | 脚本访问 | 值表示 | 错误通道 | 线程规则 |
| --- | --- | --- | --- | --- | --- |
| Host embedding Iris | C Host ABI function tables | 无直接脚本访问 | 不透明的运行时有根句柄 | Status 加 result 或 ExceptionContext handles | 触及堆的调用在 runtime thread 上执行，其他线程使用 post queue |
| Native extension package | 与运行时协商的 C extension ABI | 只通过包元数据和 Iris Methods 可见 | 不透明句柄和已验证载荷描述符 | Status 加 ExceptionContext handles | 句柄使用在 runtime thread 上执行，workers 使用 post queue |
| Rust wrapper | C ABI 上的便利包装层 | 无直接脚本访问 | 不透明句柄周围的 RAII 包装 | 把 statuses 转换成包装层错误，不让 ABI panics 穿越 | 调用前检查运行时所有权和线程亲和性 |
| C++ wrapper | C ABI 上的便利包装层 | 无直接脚本访问 | 不透明句柄周围的 RAII 包装 | 在 ABI 边缘捕获或禁止 C++ exceptions | 调用前检查运行时所有权和线程亲和性 |
| Script `FFI` | 标准 Iris API | 唯一脚本二进制路径 | `FFI::Library` objects 和 bound Method metadata，不是 Host handles | 普通 Iris exceptions 和 ExceptionContext | Library Methods 在 runtime thread 上运行，外部工作 post back |

IRIS-V1-FFI-N001: Historical note: 旧 headers 和 pointer extension 暴露 C++ objects、raw `void*` payload access、direct class registration 和 ad hoc extension entry points。它们只是考古材料。Iris v1 保留扩展目标，但用 C tables、不透明句柄、元数据验证和运行时拥有的载荷描述符替换该模型。

## 运行时有根句柄

IRIS-V1-FFI-C007: 每个跨越 Host ABI 的 Iris value、object、Class、Module、Contract、Method、Closure、Type、Task、ExceptionContext、metadata object 或 native wrapper value，MUST 作为由一个 IrisRuntime 拥有的不透明句柄跨越。Raw managed object pointer、raw Class pointer、raw Method pointer、raw GC address、object layout address、vtable address 或 interior pointer MUST NOT 跨越 ABI。

IRIS-V1-FFI-C008: 不透明值句柄在其存活期间对目标保持强引用根。释放 handle 会移除该根。Scoped 或 local handles 是普通有根值句柄，其生命周期受 handle frame 限定。关闭 frame 会批量释放该 frame 中每个存活句柄。符合要求的实现 MAY 在内部移动、压缩、pin、intern 或重新分配托管值，只要存活句柄继续表示同一个 Iris identity，或继续满足先前章节要求的无对象身份的值语义。

IRIS-V1-FFI-C009: Handles 恰好属于一个 runtime。Host、extension、wrapper 或 FFI binding MUST NOT 把 handle 传给另一个 runtime、把 handle numeric values 当作稳定 identity 比较、持久化 handle bytes、从 handle numbers 派生 hashes，或假定释放后不会重用。Runtime destruction 会使该 runtime 的每个 handle 无效，并让之后使用 handle 以 closed 或 invalid-runtime status 失败。

IRIS-V1-FFI-C010: 每个 ABI value handle 都是有根句柄。其 release mode 可以是 explicit release 或 scoped frame release，但任一模式 MUST 让目标在 release 发生前保持强引用根。非 handles 的 temporary call-frame references MAY 只存在于一个 ABI call implementation 内。它们 MUST NOT 被存储、返回、放入 ABI records、当成 value handles，或逃逸出该 call。Wrapper MAY 用语言 scopes 强制 handle lifetime，但 C ABI contract 仍是权威。

IRIS-V1-FFI-C011: 以下 handle 表是规范性的:

| 句柄类别 | 成根效果 | 是否可跨越 C ABI | 是否可在 runtime thread 外使用 | 释放规则 | 误用时失败 |
| --- | --- | --- | --- | --- | --- |
| Runtime handle | 拥有 runtime identity，不是 Iris value root | Yes | 仅用于明确标为 thread-safe 的 lifecycle 或 post operations | Runtime lifecycle rules | Closed 或 invalid-runtime status |
| Explicit-release value handle | 对一个 Iris value 或 metadata object 保持强引用根，直到 explicit release | Yes | No | Explicit release | Invalid-handle 或 thread-affinity status |
| Scoped value handle | 对一个 Iris value 或 metadata object 保持强引用根，直到其 handle frame 被释放 | Yes | No | Scoped frame release，如果表声明可提前 explicit release 也允许 | Invalid-handle 或 thread-affinity status |
| Completion token | 授权一次 Task completion post | Yes | 仅可从 worker threads post | 恰好消耗一次，或在 runtime shutdown 时 closed | Duplicate-completion 或 closed status |

## 线程亲和与 Post Queue

IRIS-V1-FFI-C012: 所有触及 Iris handles、托管堆、Class 或 Module 元数据、Type 元数据、Method 派发、ReflectionPolicy、MetaCapabilities、调度器、Task 状态或 ExceptionContext 图的 ABI calls，MUST 在所属 runtime event-loop thread 上运行。从另一线程调用这类 API MUST 返回 thread-affinity status，并且 MUST NOT 与托管堆竞争。

IRIS-V1-FFI-C013: 外部线程 MAY 执行不检查、解引用、比较、保留、释放或以其他方式使用 Iris handles 的外部工作。外部线程 MAY 通过 thread-safe post queue 向所属 runtime post 已复制或外部拥有的数据、statuses 和 completion tokens。Runtime thread 把 posted data 转换为 Iris values、观察 handles，并完成 Tasks。

IRIS-V1-FFI-C014: Post queue 是 v1 中 external worker threads 进入一个 runtime 的唯一跨线程入口。它 MUST 为来自同一 producer 的已接受 posts 按提交顺序保持 FIFO ordering。Iris v1 不规定独立 external producers 的 posts 被一个 runtime queue 接受之前的相对顺序。

IRIS-V1-FFI-C015: 未来 native no-GIL shared-memory Iris threads 需要之后的 ABI major version。Iris v1 Host ABI implementations MUST NOT 把 worker-thread handle use、worker-thread heap access 或 shared managed-memory mutation 作为此 ABI 的兼容扩展暴露。

IRIS-V1-FFI-C016: 以下线程矩阵是规范性的:

| 操作 | Runtime thread | External worker thread | 必需失败 |
| --- | --- | --- | --- |
| Create or release a value handle | Allowed | Prohibited，除非 function table 声明 thread-safe release primitive | Thread-affinity status |
| 通过 handle 读取、写入或调用 | Allowed | Prohibited | Thread-affinity status |
| 从 posted native data 构建 Iris values | Allowed | Prohibited | Thread-affinity status |
| Perform blocking OS or device work without Iris handles | Host 允许 blocking 时 Allowed | Allowed | Not applicable |
| Post copied completion data | Allowed | 只通过 post queue Allowed | Closed status after shutdown |
| Complete a Task directly | 只通过 runtime completion processing Allowed | Prohibited | Thread-affinity 或 duplicate-completion status |

## Status、Results 与 ExceptionContext

IRIS-V1-FFI-C017: 每个可能失败的 C ABI operation MUST 返回 status code，并把普通结果、抛出的值或 `ExceptionContext` objects 放入显式 out handles 或 result records。当 operation raised 或 captured Iris exception 时，只有 status 不够。仅字符串错误通道不足以符合要求。

IRIS-V1-FFI-C018: Native code MAY 只能通过创建或传播 `ExceptionContext` 的 ABI operations 创建普通 Iris errors、raise Iris values，或返回 failed Task completions。被抛出的 Iris value 仍是 Iris object 或 value，而 stack、cause、suppressed cleanup failures、async links 和 native bridge records 属于 `ExceptionContext`，如 [04-bindings-callables-control-flow.md](04-bindings-callables-control-flow.md) 和 [07-async-resources-diagnostics.md](07-async-resources-diagnostics.md) 所定义。

IRIS-V1-FFI-C019: Rust panics、C++ exceptions、SEH exceptions、host language unwinding、long jumps 或 foreign exceptions MUST NOT 跨越 C ABI boundary。Wrappers MUST 在可恢复 language-level unwinding 到达 C 之前捕获或阻止它。如果内存破坏、栈破坏或另一不可恢复 condition 阻止安全转换，实现 MAY abort，而不是返回 undefined state。

IRIS-V1-FFI-C020: Iris unwinding MUST NOT long jump across native frames。发生 raise 的 native callback 或 Host call 通过 ABI 返回 status 和 `ExceptionContext` handle。Wrapper APIs 可以把它表现为 language-native error object，但 C boundary 仍观察 status 加 handles。

IRIS-V1-FFI-C021: 以下错误表是规范性的:

| 事件 | C ABI 结果 | Iris 可见性 | 必须保留的数据 |
| --- | --- | --- | --- |
| 成功调用返回值 | Success status 加 result handle 或 primitive result record | 调用方观察到返回的 Iris value | Runtime ownership 和 Type Contract |
| Iris Method 抛出异常 | Failure status 加 ExceptionContext handle | 普通 catch 或 failed Task 观察同一 context | 抛出的值、cause、suppressed contexts、stack records |
| Native callback 抛出 Iris value | Failure status 加 ExceptionContext handle | 按普通 Iris propagation 传播 | Native bridge frame 加抛出的值 |
| 线程亲和性违规 | Thread-affinity status，无 heap mutation | Host 或 wrapper error，不是 Iris heap race | Runtime identity 和 attempted operation category |
| Rust panic 或 C++ exception 位于 wrapper 边缘 | 安全时转换为 native error status | 只有 wrapper 通过 ABI 创建时才是普通 Iris error | Panic 或 exception text 可作为 diagnostic，但不是唯一 Iris error |
| Runtime 已经关闭 | Closed status | 除非 runtime 能安全报告，否则不创建新 Iris value | Runtime identity 和 shutdown state |

## 原生元数据绑定

IRIS-V1-FFI-C022: Native packages MUST 为每个导出的 Iris Class、Module、Contract、Method、property、generic declaration、native layout obligation、MetaCapabilities policy、package identity、permissions、required ABI version 和 native artifact digest 随附 Iris-readable compile-time metadata。Compiler 只有在验证这些元数据完整且与先前章节一致后，才把它导入为 static API。

IRIS-V1-FFI-C023: Runtime native load MUST 验证已加载 native artifact 与 package resolution 和 lockfiles 选定的 metadata 匹配。Verification MUST 包括 package identity、API major、Iris language major、ABI major and minor requirements、declared feature bits、artifact digest、native layout descriptors、Type and Contract signatures、Method signatures、MetaCapabilities，以及与 ReflectionPolicy 相关的 package identity。

IRIS-V1-FFI-C024: Runtime-only native registration MAY 只通过与 Iris metaprogramming 相同的 open transaction、MetaCapabilities、ReflectionPolicy、static-spine、Contract 和 package rules 创建 dynamic-only members。Runtime-only native registration MUST NOT 扩展已经编译的 static API、绕过 import rules、绕过 `impl` 或 `override`，或让 compiler 认为某成员在 compile time 已存在。

IRIS-V1-FFI-C025: Native Method binding MUST 保持 metadata 记录的 callable signature。参数和返回值通过 handles 或 declared primitive interop records 跨越 native boundary，每个 written Contract 仍是 static promise，也是在 [05-types-contracts-generics.md](05-types-contracts-generics.md) 下的 runtime boundary guard。

IRIS-V1-FFI-C026: 以下元数据绑定表是规范性的:

| 元数据项 | 编译时作用 | 运行时验证 | 失败规则 |
| --- | --- | --- | --- |
| Package identity and API major | 建立 nominal Type 和 package identity | 必须匹配 manifest、lock、artifact 和 runtime load request | 在 executable native code 暴露前中止加载 |
| Class, Module, Contract, and Type metadata | 提供 static API 和 reflection facts | 必须匹配 artifact registration tables 和 static spine | 拒绝 load 或 candidate publication |
| Method and property signatures | 提供 call、`impl`、`override` 和 Contract checks | 必须匹配 native callable descriptors | 在 calls 可发生前拒绝 binding |
| Native layout and payload descriptors | 提供 shape 和 GC obligations | 必须验证 size、alignment、trace、drop 和 ownership records | 拒绝 publication 或 package load |
| MetaCapabilities and ReflectionPolicy identity | 提供 authority 和 visibility checks | 必须匹配 manifest、package 和 Host grants | Deny operation 或 abort candidate |
| Artifact digest and ABI requirements | 提供 reproducible binding target | 必须匹配 loaded binary 和 negotiated table version | 拒绝 load |

## Native Payloads、GC 与 Closeable Resources

IRIS-V1-FFI-C027: 存储 native payloads 的 Native-backed Classes MUST 注册 runtime-owned payload descriptors。Descriptor 说明 size、alignment、construction policy、destruction policy、optional trace obligations、optional final memory cleanup、native layout compatibility，以及 payload 是否代表 external resource。Runtime 控制附加到 Iris objects 的载荷存储的 allocation 和 lifetime。

IRIS-V1-FFI-C028: Native payload trace logic MUST 只通过为 tracing 提供的 ABI mechanism 报告 managed handles 或 roots。它 MUST NOT 解引用已移动的 managed objects、保留 raw managed pointers、创建新的 Iris values、调用任意 Iris Methods、raise Iris exceptions、阻塞外部 IO，或依赖 worker-thread heap access。

IRIS-V1-FFI-C029: Native payload drop 或 final memory cleanup 在 GC-safe runtime context 中运行。它 MUST NOT raise into Iris，MUST NOT 调用 ordinary Iris callbacks，且 MUST NOT 分配 managed objects，除非 ABI 明确说明 cleanup context 允许该分配。它还 MUST NOT 执行用户可见的确定性资源释放。Final memory cleanup 中的 failures 是 runtime diagnostics，不是可捕获 language results。

IRIS-V1-FFI-C030: Files、sockets、GPU objects、database connections、OS handles 和类似 external resources MUST 使用显式幂等 `Closeable` behavior 进行确定性释放。GC timing 不是资源管理承诺。Native-backed Iris object 可以同时拥有用于内存安全的 payload descriptor 和用于 deterministic external release 的 `Closeable` Method。

IRIS-V1-FFI-C031: Extension-owned raw object-pointer conventions 被禁止。Native extension MAY 在 runtime-owned payload storage 或 external resource records 内保留自己的 non-managed pointers，但这些 pointers 不是 Iris object pointers，且 MUST NOT 作为 raw addresses 暴露给 scripts。

IRIS-V1-FFI-C032: 以下原生所有权表是规范性的:

| 原生状态 | 所有者 | 确定性释放 | GC 或运行时清理 | 是否可 raise into Iris |
| --- | --- | --- | --- | --- |
| Runtime-rooted handle | IrisRuntime handle table | Explicit release 或 frame release | Runtime destruction invalidates | Release failure 只报告 status |
| Native payload storage | IrisRuntime payload allocator | Object API 可单独 close external resource | Descriptor drop 或 final cleanup | No |
| External file or socket | Native resource wrapper 加 Iris object | 通过 `Closeable` 的幂等 `close()` | Final cleanup 可作为最后手段 release | `close()` 可 raise ordinary Iris error，final cleanup 不可 |
| Managed object reference from payload | IrisRuntime handle 或 trace-reported root | Handle release 或 descriptor update | Trace keeps reachable targets alive | No from trace or final cleanup |
| Raw native pointer for external library | `FFI::Library` 或 native payload descriptor | `FFI::Library.close()` 或 resource `close()` | Library 或 payload final cleanup | Close Method 可 raise，final cleanup 不可 |

## 原生异步完成

IRIS-V1-FFI-C033: 返回 `Task<T>` 的 native async Method MUST 获得 runtime-owned completion token 或 completion source，并与该 Task 绑定。Token 是不透明的，属于一个 runtime，并且最多授权一次 completion。Returned Task 遵循 [07-async-resources-diagnostics.md](07-async-resources-diagnostics.md) 中的 identity、completion、await 和 unobserved-failure rules。

IRIS-V1-FFI-C034: 执行 native async work 的 Worker threads MUST 只使用 thread-safe post queue，并携带 copied 或 externally owned data、status 和 completion token。它们 MUST NOT 读取 Iris handles、创建 Iris values、解析 Type Contracts、直接完成 Tasks、检查 ExceptionContext graphs，或触及 managed heap。

IRIS-V1-FFI-C035: Runtime thread 接收 posted completion，验证 token，在 declared `T` Contract 下把 posted data 转换为 Iris values，为 failure 创建或传播 `ExceptionContext`，并恰好完成 Task 一次。Duplicate completion attempt MUST 以 duplicate-completion status 失败，并且 MUST NOT 改变已经完成的 Task。

IRIS-V1-FFI-C036: Runtime shutdown 后 posting MUST 返回 closed status。如果 native async operation post completion 前发生 shutdown close，runtime MUST 按其 shutdown policy complete 或 diagnose，且不允许 worker thread 在 close 后使用 Iris handles。

IRIS-V1-FFI-C037: 以下异步桥表是规范性的:

| 阶段 | 原生侧权限 | Runtime thread 职责 | 失败规则 |
| --- | --- | --- | --- |
| Async Method call starts | 在 runtime thread 接收 declared arguments | 创建 Task 和 completion token | Setup error 返回 status 和 ExceptionContext |
| Worker runs external work | 不使用 Iris handles，拥有或复制 external data | 无，直到 post accepted | Worker handle use is prohibited |
| Worker posts success | 发送 copied result data 和 token | 转换为 Iris value 并检查 `T` | 转换失败会把 Task 完成为 failed |
| Worker posts failure | 发送 error descriptor 和 token | 创建 ExceptionContext 并 fail Task | Missing context data becomes native bridge error context |
| Duplicate post | 第一次 consumed token 后无 authority | 保持 Task unchanged | Duplicate-completion status |
| Runtime closes | 之后的 posts 返回 closed status | 按 shutdown policy invalidates tokens | No heap access from workers |

## Function-Table 版本化

IRIS-V1-FFI-C038: C ABI evolution MUST 使用 negotiated、versioned function tables 和 size-tagged records。Runtime、Host、extension 和 wrapper participants 在任何 extension code 获得创建或观察 Iris values 的 authority 之前，声明 required ABI major、minimum minor、supported feature bits、record sizes 和 table sizes。

IRIS-V1-FFI-C039: ABI major mismatch MUST 拒绝 load 或 attachment。兼容 minor versions MAY append fields、append function table entries、add feature bits，或 add optional records。Participant MUST NOT 把 older record 重新解释为 newer larger layout，除非 supplied size 覆盖要读取的 field 且 required feature bit 存在。

IRIS-V1-FFI-C040: Symbol IDs、Type IDs、handle IDs、table slots、feature-bit numbers 和 native binding IDs 是 runtime-local，除非本规范某条款显式标记该值跨 runtimes 稳定。它们 MUST NOT 被持久化，或被用作 public Type hashes、package identity 或 cross-run ABI identity。

IRIS-V1-FFI-C041: Rust wrapper crate、C++ wrapper library 或其他 binding MAY pin 一个或多个受支持 C ABI versions，并提供 RAII handles、ownership checks、callback guards、panic 或 exception barriers，以及 typed conversion helpers。这类 wrappers MUST 报告其 underlying C ABI version，并且在 negotiated C table 不能满足其 safety assumptions 时 MUST fail closed。

IRIS-V1-FFI-C042: 以下版本化表是规范性的:

| 变更 | ABI 兼容性 | 所需机制 |
| --- | --- | --- |
| 添加可选 helper function | Minor-compatible | 追加 table entry、增加 table size、设置 feature bit |
| 添加旧代码无法满足的必需语义规则 | Major-breaking | New ABI major |
| 在 record tail 添加 field | Minor-compatible | Size-tagged record，只在 size 覆盖 field 时读取 |
| 改变现有 field 或 status 的含义 | Major-breaking | New ABI major |
| 添加带旧 fallback meaning 的 status code | 只有旧 participants 能处理 generic failure 时才 Minor-compatible | Feature bit 或 documented fallback |
| 承诺 Rust 或 C++ object layout 作为 binary contract | Prohibited | Iris v1 不允许 |

## 脚本 FFI Library 表面

IRIS-V1-FFI-C043: `FFI` 是脚本发起 external binary calls 的标准 namespace 和 service Class。`FFI.open(path, declarations: ...) -> FFI::Library` 加载一个 Host-authorized dynamic library，并返回 identity-bearing `FFI::Library` object。Host 或 package manifest MUST 在加载或调用成功前授予精确 conceptual permission scopes `ffi.load` 和 `ffi.call`。这些 scopes 是 [08-modules-metaprogramming.md](08-modules-metaprogramming.md) 下的 manifest permission requests 和 Host grants。Iris package、dependency、native artifact 或 FFI sidecar MUST NOT 自行授权它们。

IRIS-V1-FFI-C044: `FFI::Library` 拥有并 pin native library lifetime、其 bound symbol descriptors、bound callbacks、由 FFI values 表示的 native pointers、close state 和 permission scope。它 MUST 实现幂等 `Closeable`。多个 `FFI::Library` objects 相互独立，即使它们指向同一 filesystem artifact，除非 Host 显式共享 loader state 且不改变 observable Library identity。

IRIS-V1-FFI-C045: 每个 script-callable native symbol 在 invocation 前 MUST 有显式 verified signature。Signatures 可以来自传给 `FFI.open` 的 sidecar declaration file，也可以来自程序式 `library.bind(:symbol, signature, options)` operation。Sidecar 和 programmatic signatures 具有等价 validation、permission checks 和 reflection metadata。Unbound symbols MUST NOT 被 invoked。

IRIS-V1-FFI-C046: V1 不向 scripts 暴露 signature-less raw unsafe call、address-call primitive、arbitrary `dlsym` object、Host ABI function pointer call 或 unchecked callback trampoline。实现 MUST NOT 提供 conforming-mode escape hatch 来在没有本章要求的 signature metadata 时调用 native symbol。

IRIS-V1-FFI-C047: FFI signature MUST 至少声明 symbol name、C calling convention、parameter count、parameter C types、result C type、integer widths and signedness、floating widths、pointer nullability、pointer ownership and lifetime、text encoding、buffer length relations、callback metadata、error and result convention、thread behavior、close 或 release obligations，以及 function may block。缺少必需 declaration data MUST 在任何 call 发生前拒绝 binding。

IRIS-V1-FFI-C048: Bound FFI symbols 成为 `FFI::Library` object 上的 dynamic-only Methods。它们不改变 package static API，不创建 Host ABI access，不暴露 extension function tables，也不绕过 Type Contracts。Reflection MAY 在 ReflectionPolicy 和 FFI permissions 约束下显示其 verified signatures。

IRIS-V1-FFI-C049: V1 FFI 只支持 stable C ABI calls。想要 script FFI access 的 Rust、C++ 或其他 implementation-language libraries MUST 导出 C-compatible wrapper symbols 和 C-compatible data representations，并由 signature metadata 声明。Host embedding 的 Rust wrapper API 不是 script FFI，script FFI MUST NOT 承诺 Rust ABI compatibility。

IRIS-V1-FFI-C050: 以下 FFI binding 表是规范性的:

| FFI 表面 | 必需声明 | 脚本结果 | 所有权和关闭规则 | 失败规则 |
| --- | --- | --- | --- | --- |
| `FFI.open(path, declarations: sidecar)` | Sidecar signatures 和 permissions | 新的 `FFI::Library` | Library pins native artifact until close | Missing grant、invalid sidecar 或 load failure raises ordinary Iris error |
| `library.bind(:symbol, signature, options)` | Programmatic complete signature | 该 Library 上的 dynamic-only Method | Bound descriptor tied to Library lifetime | Incomplete signature 或 denied symbol raises ordinary Iris error |
| Call bound Method | Existing verified signature | 从 C result 转换得到的 Iris value | Signature ownership rules decide copying、borrowing 或 Closeable wrapper | C error convention maps to Iris exception 或 result by signature |
| Pass callback to C | Callback signature 和 lifetime | C 只通过 runtime-approved callback path 重新进入 | Runtime-rooted callback handle released by declared rule | Callback after release 或 close returns declared native failure |
| `library.close()` | None beyond Library state | successful idempotent close 时为 `nil` | Releases bound descriptors and native loader reference | deterministic close 运行时 close failure raises ordinary Iris error |

## Safe Rust 与 C++ Extern C 指引

IRIS-V1-FFI-C051: 面向稳定分发的 Rust integration SHOULD 暴露 `extern "C"` functions 并使用 negotiated C tables。Safe Rust wrappers SHOULD 用 RAII 拥有 handle release、在 wrapper values 中编码 runtime ownership、防止 release 后使用 handle、对 heap-touching handles 阻止 `Send` 或 cross-thread use、在 recovery 安全的 wrapper boundaries 捕获 panics，并把 statuses 转换成 typed wrapper errors 而不隐藏 `ExceptionContext` handles。

IRIS-V1-FFI-C052: 面向稳定分发的 C++ integration SHOULD 暴露 `extern "C"` functions 并使用 negotiated C tables。C++ wrappers SHOULD 用 RAII 拥有 handle release、避免 across C callbacks throw、在 recovery 安全时于 C edge 之前转换 exceptions、避免把 C++ object layout 暴露为 ABI，并把 Iris handles 视为 opaque runtime-owned capabilities。

IRIS-V1-FFI-C053: Safe wrapper MUST NOT 承诺其 language-native object identity、destructor timing、thread model、allocator、panic 或 exception type、generic type，或 vtable layout 是 Iris stable ABI。其 safety claims 只局限于 wrapper version 及其 negotiated C ABI version。

## 示例与一致性向量

IRIS-V1-FFI-EX001: Informative example: 脚本 FFI 只绑定已声明的 C signatures:

```iris
let library = FFI.open("mathlib", declarations: "mathlib.ffi")
let value = library.hypot(3.0f64, 4.0f64)
library.close()
```

IRIS-V1-FFI-EX002: Informative example: 原生异步桥形状:

```iris
async fun load_image(path: String) -> Bytes {
  return await NativeImages.read(path)
}
```

IRIS-V1-FFI-C054: 下列向量表是规范性的。一致性章节 MUST 保留这些 vector IDs，或把它们映射到具有相同 observable outcomes 的 machine-readable records:

| Vector ID | Kind | 场景 | 预期结果 |
| --- | --- | --- | --- |
| `IRIS-V1-FFI-V001` | Failure | Native extension 试图把 raw managed object pointer 跨越 C ABI 传递 | ABI validation 拒绝，或 runtime 返回 invalid-boundary status |
| `IRIS-V1-FFI-V002` | Positive | 强句柄在 GC 移动或压缩后仍存活 | Handle 仍表示同一 Iris identity 或值语义 |
| `IRIS-V1-FFI-V003` | Failure | Runtime A 的 handle 被用于 runtime B | Invalid-runtime status，且没有 heap mutation |
| `IRIS-V1-FFI-V004` | Failure | Worker thread 通过 Iris handle 读取或调用 | Thread-affinity status |
| `IRIS-V1-FFI-V005` | Positive | Worker thread post 已复制 completion data 和 token | Runtime thread 用已检查的值完成 Task |
| `IRIS-V1-FFI-V006` | Failure | Native async operation 两次 post 同一 completion token | 第一次 completion 保持有效，第二次 post 得到 duplicate-completion status |
| `IRIS-V1-FFI-V007` | Positive | Native Method raise Iris value | Caller 收到 status 加 ExceptionContext，catch 观察到普通 Iris exception |
| `IRIS-V1-FFI-V008` | Failure | C++ exception 或 Rust panic 跨越 C ABI boundary | Wrapper 或 runtime 拒绝边界穿越，且没有 undefined Iris state |
| `IRIS-V1-FFI-V009` | Failure | Native artifact metadata digest 与 loaded binary 不匹配 | Package 或 extension load 在 binding 前中止 |
| `IRIS-V1-FFI-V010` | Failure | Runtime-only native registration 试图扩展已编译 static API | Member 保持 dynamic-only，或 registration 被拒绝 |
| `IRIS-V1-FFI-V011` | Positive | Native payload 只 trace managed handle roots | 被引用的 managed values 保持存活，trace 不执行任意 Iris call |
| `IRIS-V1-FFI-V012` | Positive | Closeable native resource 被关闭两次 | 两次调用都按幂等 close contract 完成 |
| `IRIS-V1-FFI-V013` | Failure | Final payload cleanup 试图 raise into Iris | Runtime 报告 diagnostic 或拒绝 descriptor，final cleanup 不发生 Iris propagation |
| `IRIS-V1-FFI-V014` | Failure | Extension negotiation 期间发生 ABI major mismatch | Load rejected |
| `IRIS-V1-FFI-V015` | Positive | ABI minor-compatible table 以 size tag 追加可选 field | 旧 participant 忽略 field，新 participant 只在 size 和 feature 允许时读取 |
| `IRIS-V1-FFI-V016` | Positive | `FFI.open` 使用有效 sidecar 和 grants | 返回 identity-bearing `FFI::Library` |
| `IRIS-V1-FFI-V017` | Failure | Script 调用未绑定 native symbol | Ordinary Iris error，且没有 native call |
| `IRIS-V1-FFI-V018` | Failure | Script 请求 signature-less unsafe call | Static 或 runtime diagnostic，且没有 native call |
| `IRIS-V1-FFI-V019` | Positive | `library.bind` 声明完整 C signature | Bound symbol 作为 dynamic-only Library Method 出现 |
| `IRIS-V1-FFI-V020` | Failure | FFI declaration 在需要时省略 pointer ownership、nullability 或 error convention | Binding 在 call 前被拒绝 |


## FFI 覆盖向量

IRIS-V1-FFI-C058: 以下向量是带有具体 Host、native 和 FFI 边界观察结果的规范性 traceability vectors。

| Vector ID | Category | 适用性 | 来源/输入 | 预期可观察结果 | Decisions |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-FFI-V059` | negative | interpreter 不适用；JIT 不适用；native required；原因: 加载 native metadata 时会检查 package identity。 | `metadata/manifestless_native.json` 省略 package identity 和 ReflectionPolicy；`fixtures/ffi/manifestless_native.c` 导出一个没有 trusted Host identity 的 C extension entry。 | Native-load status 是 `metadata-mismatch`；diagnostic 是 `ffi.native-package-identity-missing`，severity 为 `error`，phase 为 `native-load`，clause 为 `IRIS-V1-FFI-C022`；reflection metadata 不发布。 | `D-334` |
| `IRIS-V1-FFI-V060` | positive | interpreter 不适用；JIT 不适用；native required；原因: fixture 在 Iris source 执行前附加 C extension table。 | `fixtures/ffi/host_abi_v1.c`: `iris_extension_attach(requested_major=1, minimum_minor=0)` 作为 C 编译并通过 Host table 调用；`metadata/host_abi_v1.json` 声明 `abi_major: 1`、`abi_minor: 0` 和 `c_abi: true`。 | Status 是 `success`；extension 只接收协商后的 C table，且该 table 报告 `abi_major=1`、`abi_minor=0`。不存在 Rust、C++、object-layout、vtable 或 allocator entry。 | `D-491` |
| `IRIS-V1-FFI-V061` | positive | interpreter 不适用；JIT 不适用；native required；原因: fixture 通过 C ABI 观察 Host handles 和 GC。 | `fixtures/ffi/rooted_handle.c`: 创建 integer `41`，把它保留为 `IrisHandle h`，强制一次 collecting allocation cycle，读取 `h`，释放 `h`，然后尝试第二次读取；`metadata/rooted_handle.json` 声明 `handle_kind: rooted_value`。 | 释放前，status 是 `success`，value 是 integer `41`，type 是 `Integer`，且不暴露 managed address。释放后，status 是 `invalid-handle`，value 缺失，且没有 heap mutation。 | `D-492` |
| `IRIS-V1-FFI-V062` | negative | interpreter 不适用；JIT 不适用；native required；原因: fixture 从 external worker 调用 Host handle operation。 | `fixtures/ffi/thread_affinity.c`: worker 调用 `iris_handle_get_int(h)`，然后用 completion token post 已复制的 integer `7`；`metadata/thread_affinity.json` 为被接受的 post path 声明 `worker_uses_handles: false`。 | Worker handle read 返回 status `thread-affinity`，value 缺失，side effect 是 `heap_mutation=false`；copied-data post 返回 `success` 并在 runtime thread 上被消费。 | `D-493` |
| `IRIS-V1-FFI-V063` | negative | interpreter 不适用；JIT 不适用；native required；原因: fixture 通过 native callback boundary raise。 | `fixtures/ffi/native_error.c`: callback `raise_marker` 通过 Host ABI 创建 raised value symbol `:ffi_marker`；`metadata/native_error.json` 声明 `error_convention: status_plus_exception_context`。 | Status 是 `failure`；result handle 缺失；ExceptionContext handle 存在，value 为 `:ffi_marker`，type 为 `ExceptionContext`，并带 native bridge frame。没有 string-only error result，也没有 foreign unwind。 | `D-494` |
| `IRIS-V1-FFI-V064` | negative | interpreter 不适用；JIT 不适用；native required；原因: metadata validation 和 native load 发生在 executable Iris code 之前。 | `metadata/static_api.json` 声明 package `fixture.static_api`、Method `answer() -> Integer`、ABI `1.0` 和 digest `sha256:00`；`fixtures/ffi/static_api.c` 的 artifact digest 不同，为 `sha256:11`。 | Package-load status 是 `metadata-mismatch`；diagnostic 是 `ffi.native-artifact-digest-mismatch`，severity 为 `error`，phase 为 `native-load`，clause 为 `IRIS-V1-FFI-C023`；不发布 Method，且不调用 native code。 | `D-495` |
| `IRIS-V1-FFI-V065` | positive | interpreter required；JIT 不适用；native required；原因: script 通过 native boundary 使用 native-backed Closeable object。 | `fixtures/ffi/payload_resource.c` 声明 runtime-owned payload descriptor `{size: 8, alignment: 8, trace: none, drop: no_raise}` 和 native `close` counter；script source 是 `let r = NativeFixture.resource(); r.close(); r.close()`。 | 两次调用都返回 type 为 `Nil` 的 value `nil`；native close counter 恰好是 `1`；final cleanup 不发出 ExceptionContext，也不发出 ordinary Iris error。 | `D-496` |
| `IRIS-V1-FFI-V066` | negative | interpreter required；JIT 不适用；native required；原因: runtime thread 创建并观察 Task，同时 fixture post native completion。 | `fixtures/ffi/async_once.c` 用一个 completion token 两次 post copied integer `9`；script source 是 `let t = NativeFixture.once(); await t`。 | `await t` 返回 type 为 `Integer` 的 integer `9`；第一次 post status 是 `success`；第二次 post status 是 `duplicate-completion`；Task value 和 completion count 保持 `9` 和 `1`。 | `D-497` |
| `IRIS-V1-FFI-V067` | negative | interpreter 不适用；JIT 不适用；native required；原因: ABI table negotiation 先于 source execution。 | `fixtures/ffi/negotiate_v2.c` 请求 ABI major `2`、minimum minor `0`；`metadata/negotiate_v2.json` 声明 size-tagged request record，size 为 `24`；runtime fixture 暴露 ABI `1.3`。 | Attach status 是 `abi-major-mismatch`；diagnostic 是 `ffi.abi-major-mismatch`，severity 为 `error`，phase 为 `native-load`，clause 为 `IRIS-V1-FFI-C039`；不发布 table，也不运行 extension entry。 | `D-498` |
| `IRIS-V1-FFI-V068` | negative | interpreter required；JIT 不适用；native required；原因: script request 到达 native loader，但不会执行 bound Method。 | Script source 是 `HostABI.load("fixtures/ffi/libfixture")`；package metadata 请求 `ffi.load` 和 `ffi.call`，但不暴露 script-visible Host ABI capability。 | Static validation 以 diagnostic `ffi.host-abi-script-access` 在 phase `static` 拒绝 source；不加载 library，不调用 native symbol，也不创建 `FFI::Library` value。 | `D-499` |
| `IRIS-V1-FFI-V069` | positive | interpreter required；JIT 不适用；native required；原因: `FFI.open` 调用已授权的 native loader。 | Script source 是 `let a = FFI.open("fixtures/ffi/libfixture", declarations: "fixtures/ffi/libfixture.ffi"); let b = FFI.open("fixtures/ffi/libfixture", declarations: "fixtures/ffi/libfixture.ffi"); [a == b, a.class_name]`；manifest 授予 `ffi.load` 和 `ffi.call`。 | Value 是 array `[false, "FFI::Library"]`；element types 是 `Bool` 和 `String`；两次 open 都返回 status `success`，且两个 Library objects 有不同 object identities。 | `D-500` |
| `IRIS-V1-FFI-V070` | negative | interpreter required；JIT 不适用；native required；原因: binding validation 先于 native invocation。 | Sidecar `fixtures/ffi/libfixture.ffi` 声明 `int32 fixture_add(int32, int32)`；script source 是 `let l = FFI.open("fixtures/ffi/libfixture", declarations: "fixtures/ffi/libfixture.ffi"); l.call(:fixture_hidden)`。 | Runtime exception 的 kind 是 `FFI::UnboundSymbolError`，phase 是 `runtime`，message policy 是 `not compared`；diagnostic 缺失；native call counter 保持 `0`，且没有 signature-less 或 raw-address call 可用。 | `D-501` |

## 可追踪性说明

IRIS-V1-FFI-C055: 本章拥有 native 和 script FFI boundary decisions D-491 到 D-501。它保持 identity 章节的规则，即 C 是唯一稳定 Host ABI。它保持 async 章节的规则，即 external completion posts into the runtime scheduler。它保持 metaprogramming 章节的规则，即 native metadata 和 runtime-only registration 遵守 package、transaction、MetaCapabilities 和 ReflectionPolicy checks。它还保持 type 章节的规则，即 native signatures 是 enforced Contracts。

IRIS-V1-FFI-C056: 本章拥有的 decision IDs 是 `D-491`, `D-492`, `D-493`, `D-494`, `D-495`, `D-496`, `D-497`, `D-498`, `D-499`, `D-500`, and `D-501`。

IRIS-V1-FFI-C057: 引用但不由本章拥有的 decision IDs 包括 `D-001`, `D-002`, `D-077`, `D-172`, `D-173`, `D-174`, `D-175`, `D-176`, `D-177`, `D-178`, `D-179`, `D-180`, `D-181`, `D-207`, `D-208`, `D-209`, `D-210`, `D-211`, `D-212`, `D-213`, `D-214`, `D-215`, `D-216`, `D-217`, `D-218`, `D-219`, `D-220`, `D-242`, `D-243`, `D-244`, `D-245`, `D-246`, `D-247`, `D-248`, `D-249`, `D-250`, `D-322`, `D-323`, `D-324`, `D-325`, `D-326`, `D-327`, `D-469`, `D-470`, `D-471`, `D-472`, `D-473`, `D-474`, `D-475`, `D-476`, `D-477`, `D-478`, `D-479`, `D-480`, `D-481`, `D-482`, `D-483`, `D-484`, `D-485`, `D-486`, `D-487`, `D-488`, `D-489`, and `D-490`。
