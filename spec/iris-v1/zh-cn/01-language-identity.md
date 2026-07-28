# Iris v1 语言身份

状态：Iris v1 草案，语义已冻结。

翻译说明：本文是 `../01-language-identity.md` 的忠实简体中文翻译。代码、标识符、稳定 ID、D-ID、字面量和规范性术语保留原文形式或在中文译词后保留英文令牌，以便核对。

IRIS-V1-IDENTITY-C001: 本章定义 Iris v1 身份、兼容性承诺、规范性词汇、实现边界、版本身份和 v1 延后项。它必须 (MUST) 在 [README.md](README.md) 之后阅读，并在语法、运行时、类型、包、库、迁移、一致性和可追溯性章节之前阅读。

IRIS-V1-IDENTITY-C002: 本章不得 (MUST NOT) 定义解析器语法、运行时 dispatch 算法、对象布局、库字节格式、FFI 调用细节、一致性向量模式或迁移方案。这些细节属于后续章节；当它们依赖身份、兼容性、版本控制或延后规则时，必须 (MUST) 交叉链接回本章。

## 现代继承者身份

IRIS-V1-IDENTITY-C003: Iris v1 是 Legacy Iris 的现代继承者。它必须 (MUST) 保留冻结决策记录的定义性语言身份，但不得 (MUST NOT) 承诺遗留源码恢复、遗留语法恢复、遗留解析器行为、遗留扩展 ABI 行为或遗留实现怪癖。与 Legacy Iris 的每项有意不兼容都必须 (MUST) 记录在 [11-migration-divergence.md](11-migration-divergence.md) 中。

IRIS-V1-IDENTITY-C004: Iris v1 不得 (MUST NOT) 被视为无关的新语言。符合要求的规范、实现、迁移文档或一致性套件必须 (MUST) 保留核心身份：每个值都是对象，运算符是消息，dispatch 是运行时动态的，组合使用单 Class 继承加 Module 和 Contract，词法 Closure 和尾随块编程仍是核心，Class 和 Method 具有运行时身份，存在受约束的运行时变更，任何对象都可以作为异常抛出，并且语言为 Host 嵌入和原生扩展支持而设计。

IRIS-V1-IDENTITY-C005: 当模仿 Rust、Kotlin、TypeScript、Ruby、C# 或任何 Host 语言表面会违背冻结的 Iris 决策时，Iris v1 不得 (MUST NOT) 这样模仿。Host 实现语言可以 (MAY) 仅在本章的实现独立性规则和 [09-native-host-ffi.md](09-native-host-ffi.md) 的原生边界规则下影响实现策略。

IRIS-V1-IDENTITY-N001: Informative note: 评审考古资料将 Iris 描述为一种完全对象导向的消息语言，具有 Module、Contract、block、动态 Class 变更和原生 Host 亲和性。本章通过冻结决策，而不是通过旧实现，使这条线成为规范。

## 对象、消息和静态承诺原则

IRIS-V1-IDENTITY-C006: Iris v1 使用对象性作为根语义模型。每个运行时值都必须 (MUST) 是对象，且 `Object` 必须 (MUST) 是类型章节指定的顶层类型。数值、单例值、Class 对象、Module 对象、Contract 对象、Method 对象、Closure 对象、Type 对象、包对象、原生句柄、异步 Task 和诊断值，必须 (MUST) 具有其后续章节分配的身份行为，或在后续章节中被明确分类为无身份的值对象。

IRIS-V1-IDENTITY-C007: Iris v1 使用消息发送作为行为模型。运算符、普通 Method、具名中缀调用、属性协议、Contract 限定发送、元编程操作、异步入口点和绑定到 FFI 库的函数，必须 (MUST) 被指定为消息或可调用协议，除非后续章节明确定义诸如 `same?` 的原语旁路。

IRIS-V1-IDENTITY-C008: Iris v1 使用静态承诺约束动态行为。声明的超类、声明的 Contract、可见成员名和签名、类型化属性、泛型约束、原生布局义务、包身份和 API major 身份等静态事实，必须 (MUST) 在动态变更、open 事务、包升级、反射、原生绑定和优化期间得到保留。

IRIS-V1-IDENTITY-C009: Iris v1 必须 (MUST) 保持普通消息身份独立于静态类型选择。静态注解、推断类型、预期返回类型、并集分支、泛型参数或 Contract 声明可以 (MAY) 验证并收窄一次发送，但不得 (MUST NOT) 静默选择另一个普通选择器或重载。显式 Contract 限定 dispatch 是 v1 中唯一按 Contract 身份选择 Contract 槽位的源码机制。

IRIS-V1-IDENTITY-C010: Iris v1 不得 (MUST NOT) 提供按静态类型、泛型参数、并集分支、预期结果、声明顺序或实现顺序进行的重载 dispatch。无法共享一个兼容实现的同名义务必须 (MUST) 使用类型章节和运行时章节指定的显式限定 Contract 机制。

IRIS-V1-IDENTITY-C011: Iris v1 中的动态变更必须 (MUST) 受候选事务、针对静态脊柱的验证、MetaCapabilities、包和 ReflectionPolicy 权限，以及原子发布约束。无效候选不得 (MUST NOT) 部分变更活动的 Class、Module、Contract、包或元数据状态。

IRIS-V1-IDENTITY-C012: Class、Module、Contract、泛型定义、包或原生绑定的静态脊柱，必须 (MUST) 始终是编译代码、类型化绑定、反射元数据、Host/native 绑定和一致性的持久承诺。动态表面可以 (MAY) 只在后续章节定义兼容演进规则的位置演进。

## 动态行为、静态承诺

IRIS-V1-IDENTITY-C013: Iris v1 的动态/静态模型是以下紧凑契约：

| 层 | 可以动态变化的内容 | 仍保持承诺的内容 | 所属章节 |
| --- | --- | --- | --- |
| 对象和 Method | 兼容 Method 体、非 Contract 成员、属性访问器、Module 组合、类对象状态、策略允许的元数据 | 值对象性、选择器身份、Method 身份规则、声明的 Class 或 Contract 承诺、身份分类 | [03-runtime-object-model.md](03-runtime-object-model.md), [04-bindings-callables-control-flow.md](04-bindings-callables-control-flow.md) |
| 类型和 Contract | 运行时检查视图、边界内的 Dynamic 发送、兼容实现、具象化泛型实例化 | 无重载、不变泛型、Contract 槽位身份、声明要求、`Dynamic<T>` 边界检查、`Never` 流意义 | [05-types-contracts-generics.md](05-types-contracts-generics.md) |
| 元编程 | 候选 Class 或 Module 修订、open 事务、装饰器、反射可见动态成员 | 静态脊柱、MetaCapabilities、ReflectionPolicy、同步非逃逸事务边界、原子发布或回滚 | [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| 包 | 通过显式事务进行同 major 兼容热升级 | 包 ID、API major 身份、每个 API major 一个活动实现、稳定名义 Type 身份、被引用时保留旧修订 | [08-modules-metaprogramming.md](08-modules-metaprogramming.md), [11-migration-divergence.md](11-migration-divergence.md) |
| 原生和 FFI | 元数据背后的扩展实现、脚本绑定的 FFI 库 Method、通过运行时投递完成异步 | 稳定 C ABI 边界、不透明句柄、运行时线程亲和性、显式签名、不泄漏原始托管指针 | [09-native-host-ffi.md](09-native-host-ffi.md) |
| 核心包和标准包 | 核心 ABI 之外独立版本化的官方包 | 核心运行时包集、稳定格式边界、显式 Contract 驱动序列化、不因内部 BLAKE3 产生隐藏加密承诺 | [10-serialization-standard-library.md](10-serialization-standard-library.md) |

IRIS-V1-IDENTITY-C014: 前面的模型表是规范性的。后续章节必须 (MUST) 细化所列层级，同时不得违背动态演进与静态承诺之间的分离。

IRIS-V1-IDENTITY-EX001: Informative example: 包升级可以替换兼容 Method 体并发布新的活动 Class 修订。依赖旧 Method 签名的静态调用方仍拥有相同静态承诺，而如果升级提交，后续发送会使用新活动修订。

## 规范性词汇和编辑规则

IRIS-V1-IDENTITY-C015: RFC 术语 `MUST`、`MUST NOT`、`SHOULD`、`SHOULD NOT` 和 `MAY` 具有 [README.md](README.md) 赋予的含义。本章中包含规范性要求的段落必须 (MUST) 以 `IRIS-V1-IDENTITY-Cnnn` 条款 ID 开头。

IRIS-V1-IDENTITY-C016: 本章中的信息性说明、示例、实现说明和历史说明必须 (MUST) 使用 [README.md](README.md) 定义的前缀标注。信息性文本不得 (MUST NOT) 创建要求，即使它解释冻结决策。

IRIS-V1-IDENTITY-C017: 官方 Iris v1 术语必须 (MUST) 使用 [README.md](README.md) 定义的 `Contract`、`MutableString`、`ReflectionPolicy`、logical Class、active revision、Method、BoundMethod、Closure、FFI、Host ABI 和 `DEFERRED V1`。历史术语可以 (MAY) 只出现在明确标注的历史或迁移文本中。

IRIS-V1-IDENTITY-C018: 后续章节必须 (MUST) 对兼容性承诺、实现独立性、语义版本身份、包身份、核心/标准边界和 v1 延后项使用指向本章的精确交叉链接，而不是松散复述这些规则。

## 实现独立性

IRIS-V1-IDENTITY-C019: Iris v1 语义必须 (MUST) 独立于解释器、JIT、AOT 编译器、运行时表示、Host 编译器、操作系统、指针宽度、分配策略、内联、隐藏数值表示、容器桶策略、GC 移动或原生包装语言。符合要求的实现可以 (MAY) 只在可观察 Iris 行为仍等同于冻结语义时优化。

IRIS-V1-IDENTITY-C020: 优化代码可以 (MAY) 使用原语、专用表示、内联缓存、拆箱、专用机器码、消除的守卫或提升的检查，但前提是它保留所选 Method、接收者 Class 或 active revision、操作数表示假设、查找层级版本、动态 dispatch 钩子、静态脊柱承诺，以及后续章节要求的每项其他依赖。依赖变化必须 (MUST) 在受影响的后续发送观察到陈旧行为之前使其失效、回退或去优化。

IRIS-V1-IDENTITY-C021: 公共稳定哈希、Type 身份、包身份、一致性结果、诊断类别和序列化格式兼容性，不得 (MUST NOT) 依赖进程地址、Host `size_t`、文件系统路径、源码显示名、机器本地构建数据、对象布局、原始指针身份、运行时分配顺序、JIT 选择或解释器/JIT 模式，除非后续章节明确将某个值标记为运行时本地。

IRIS-V1-IDENTITY-C022: 符合要求的实现必须 (MUST) 将普通 Iris 失败暴露为普通可捕获 Iris 异常，或在后续章节要求时暴露为结构化诊断。它不得 (MUST NOT) 将冻结语言失败暴露为 Rust panic、跨越 C ABI 的 C++ 异常、断言、进程中止、JIT 崩溃、未定义行为或仅 Host 可见的错误字符串。

IRIS-V1-IDENTITY-C023: v1 的 Host ABI 兼容性契约是 C，而不是 Rust、C++ 或内部对象 ABI。Rust 和 C++ 包装器可以 (MAY) 提供更安全或更符合人体工学的 API，但它们不得 (MUST NOT) 成为 Iris v1 的二进制兼容性身份。

## 语义版本和包身份

IRIS-V1-IDENTITY-C024: Iris 语言 major 版本是公共稳定哈希算法、具名名义 Type 身份、包兼容性、一致性向量、标准格式兼容性和迁移台账的语义身份的一部分。破坏冻结语言 major 语义契约的变更必须 (MUST) 要求未来的 Iris 语言 major 版本。

IRIS-V1-IDENTITY-C025: 在一个 Iris 语言 major 版本内，后续章节标记为规范稳定的值的公共稳定哈希映射，必须 (MUST) 在符合要求的实现、平台、进程、解释器/JIT 模式，以及 minor 或 patch 修订之间保持不变。后续语言 major 可以 (MAY) 只有在冻结决策要求迁移或复现时，通过显式版本化的遗留入口点采用新映射。

IRIS-V1-IDENTITY-C026: 可发布包必须 (MUST) 声明全局唯一的反向域名风格 `package_id` 和 `api_major`。具名名义 Type 身份必须 (MUST) 包含包 ID、包 API major、完全限定名、Iris 语言 major、类型种类、泛型元数，以及适用时的封闭实参身份。它必须 (MUST) 排除文件系统路径、显示名、源码哈希和机器/构建数据。

IRIS-V1-IDENTITY-C027: 共享 `(package_id, api_major)` 的包 minor 和 patch 版本必须 (MUST) 保留名义 Type 身份。破坏超类、Contract、成员、泛型、原生布局或兼容静态 API 承诺，必须 (MUST) 要求新的包 API major。

IRIS-V1-IDENTITY-C028: 运行时必须 (MUST) 对每个 `(package_id, api_major)` 最多激活一个包实现修订。同 major 包实现不得 (MUST NOT) 通过加载上下文、加载顺序或隐藏身份拆分共存。共存的不兼容实现需要不同 API major，因此需要不同的名义身份。

IRIS-V1-IDENTITY-C029: 同 major 包热升级必须 (MUST) 是显式且事务性的。它必须 (MUST) 保留兼容稳定身份，将所有受影响的运行时和原生义务作为候选进行验证，原子发布，并在失败时让旧包保持完全活动。

IRIS-V1-IDENTITY-C030: 无清单本地脚本具有运行时本地包身份。它们的具名 Type 可以 (MAY) 在一个运行时内保持稳定，但不得 (MUST NOT) 声称跨进程稳定公共 Type 哈希、可发布序列化契约或包 API 兼容性。

## 核心和标准边界

IRIS-V1-IDENTITY-C031: Iris v1 核心包和稳定运行时包包括核心 object、Type、Contract、reflection、collections、text、bytes、Regex、Async Task 和 event loop、IO、File、Path、Encoding、Unicode、JSON、IrisValue、FFI、Package、diagnostics 和 testing。[10-serialization-standard-library.md](10-serialization-standard-library.md) 必须 (MUST) 细化此边界，同时不得将独立版本化的标准包变成语言核心 ABI。

IRIS-V1-IDENTITY-C032: HTTP、通用网络协议、加密 API、数据库、GUI API、高级 PCRE 风格 Regex 引擎和类似设施，必须 (MUST) 是独立版本化的官方标准包，而不是 Iris v1 语言核心 ABI。为冻结哈希和完整性而使用的内部 BLAKE3，不得 (MUST NOT) 暗示公共核心加密套件。

## V1 延后项和非目标

IRIS-V1-IDENTITY-C033: 下表是规范性的。每一行都是 Iris v1 的显式 `DEFERRED V1`、`OUT OF SCOPE` 或 `PROHIBITED` 项，并且不得 (MUST NOT) 由其他规范性文字暗示。

| 项目 | 状态 | V1 规则 | 相关章节 |
| --- | --- | --- | --- |
| 将遗留源码兼容性作为恢复目标 | `PROHIBITED` | Iris v1 是现代继承者，不是遗留源码兼容性承诺。 | [11-migration-divergence.md](11-migration-divergence.md) |
| Rust runtime、parser、VM、GC、JIT、extension SDK 或 workspace crates | `OUT OF SCOPE` | 本规范波次只定义语言和边界契约。 | [12-conformance.md](12-conformance.md) |
| 默认将旧的生成解析器文件、旧 PDF 文本、遗留脚本或当前实现行为视为规范性依据 | `PROHIBITED` | 历史产物仅为证据，除非冻结决策采纳它们。 | [README.md](README.md), [11-migration-divergence.md](11-migration-divergence.md) |
| 解释器、JIT 或原生 lowering 之间的后端特定语言语义 | `PROHIBITED` | 可观察语言行为必须匹配所有符合要求的后端。 | [12-conformance.md](12-conformance.md) |
| 原生无 GIL 共享内存 Iris 线程 | `DEFERRED V1` | V1 有一个 IrisRuntime 协作调度器和共享堆。未来语义未指定。 | [07-async-resources-diagnostics.md](07-async-resources-diagnostics.md) |
| Task、thread 或 async 取消语义 | `DEFERRED V1` | V1 没有取消类型、取消点、屏蔽、异步中断或 `CancellationError` Contract。 | [07-async-resources-diagnostics.md](07-async-resources-diagnostics.md) |
| 非泛型 `Task`、async void 或 fire-and-forget async 签名 | `PROHIBITED` | Async 调用返回 `Task<T>`，无结果 async 使用 `Task<Nil>`。 | [07-async-resources-diagnostics.md](07-async-resources-diagnostics.md) |
| Iris 代码中的隐式阻塞等待 | `PROHIBITED` | Host API 可以驱动事件循环，但 Iris 代码没有隐藏的阻塞 await 替代物。 | [07-async-resources-diagnostics.md](07-async-resources-diagnostics.md) |
| `defer` 语句、关键字或清理机制 | `PROHIBITED` | D-468 将清理保留在显式 `try/finally`、迭代器 close、显式 `close` 和标准 helper 上；这里不保留 v1 清理语法。 | [07-async-resources-diagnostics.md](07-async-resources-diagnostics.md), [02-lexical-grammar.md](02-lexical-grammar.md) |
| open/revision 事务内部的 Await、调度器 yield、线程转移或逃逸事务能力 | `PROHIBITED` | Open/revision 事务是同步、线程受限、不可挂起且不可逃逸的。 | [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| 自动 meta 事务重试、rebase、merge 或隐藏块重新执行 | `DEFERRED V1` | 用户可以显式重试并自行负责外部效果。 | [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| open 事务期间的候选实例预览 | `DEFERRED V1` | 候选状态仅通过事务感知的元数据/反射可见。 | [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| Ruby 风格即时 meta hook，例如 `included`、`method_added`、`inherited` 或等价项 | `PROHIBITED` | D-322 使用提交后修订事件，而不是候选构建、验证或提交期间的回调。 | [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| 自动构造/修订协调 | `DEFERRED V1` | 没有 `new_current`、`new_checked`、构造器重试、修订锁、自动构造后重新初始化或实例状态迁移。 | [03-runtime-object-model.md](03-runtime-object-model.md) |
| 自动活动实例枚举或隐式 `migrate_revision` 调用 | `DEFERRED V1` | `migrate_revision` 是普通约定，只由应用代码调用。 | [03-runtime-object-model.md](03-runtime-object-model.md) |
| opens、upgrades、decorators、migrations、initializers、native calls 或 IO 的隐藏外部效果回滚 | `PROHIBITED` | 运行时回滚只覆盖运行时拥有的候选状态，除非后续章节定义显式清理。 | [07-async-resources-diagnostics.md](07-async-resources-diagnostics.md), [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| 重载集或隐式类型导向 dispatch | `PROHIBITED` | Iris v1 使用一个普通选择器身份加显式 Contract 限定槽位。 | [05-types-contracts-generics.md](05-types-contracts-generics.md) |
| 声明点变型、使用点投影或泛型 `Actual as Exposed` 语法 | `DEFERRED V1` | 泛型 Class 和 Contract 实例化是不变的。 | [05-types-contracts-generics.md](05-types-contracts-generics.md) |
| 作为源码语义的泛型特化、SFINAE、任意编译期执行、非类型参数、依赖类型、高阶类型、可变类型参数、条件类型、映射类型或类型级元编程 | `DEFERRED V1` | 内部 JIT 特化只在不可观察时允许。 | [05-types-contracts-generics.md](05-types-contracts-generics.md) |
| 默认泛型类型参数 | `DEFERRED V1` | V1 应用使用固定完整元数，`_` 只在后续章节允许局部推断的位置使用。 | [05-types-contracts-generics.md](05-types-contracts-generics.md) |
| 递归 Type alias | `DEFERRED V1` | V1 alias 是透明、泛型且非递归的。 | [05-types-contracts-generics.md](05-types-contracts-generics.md) |
| 原始泛型实例类型或裸泛型 Class 构造 | `PROHIBITED` | 裸泛型 Class 名表示定义元数据，而不是实例类型。 | [05-types-contracts-generics.md](05-types-contracts-generics.md) |
| 独立打开封闭泛型 Class 或 Module | `PROHIBITED` | Opens 目标是未应用的泛型定义，并以事务方式传播。 | [05-types-contracts-generics.md](05-types-contracts-generics.md), [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| Proc/lambda 拆分、非局部返回 Closure 或 `LocalJumpError` 模型 | `PROHIBITED` | D-421 保持 v1 Closure return 局限于 Closure，并且不将 Closure 拆成 Proc/lambda 家族。 | [04-bindings-callables-control-flow.md](04-bindings-callables-control-flow.md) |
| Contract Method 体、默认实现、存储状态、初始化器、原始 ivar 或 private requirement | `PROHIBITED` | Contract 是具有限定槽位身份的纯静态承诺。 | [05-types-contracts-generics.md](05-types-contracts-generics.md) |
| 打开 Contract | `PROHIBITED` | `open contract` 不是 v1 操作。 | [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| 原始可变反射表或 eval-string 主要变更 | `PROHIBITED` | 反射暴露经权限过滤的不可变视图，变更使用事务。 | [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| 编译器 AST 作为通用 v1 反射 API | `DEFERRED V1` | 反射有固定的最小类型化核心 API。 | [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| 历史一等 `ReflectionCapability` 令牌模型 | `PROHIBITED` | V1 使用 `ReflectionPolicy` 和 `MetaCapabilities`。 | [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| 更细粒度的一等委托反射令牌 | `DEFERRED V1` | D-329 将 v1 授权保留在 Host 配置的 `ReflectionPolicy` inspect/mutate 范围内；委托令牌系统是后续工作，不同于被禁止的历史 `ReflectionCapability` 令牌模型。 | [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| 静态 open extension 的传递传播 | `PROHIBITED` | 静态 extension 可见性要求直接 import 导出 module。 | [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| 通配符 import 或运行时字符串语言 import | `PROHIBITED` | Import 是静态、显式、可别名/可选择的，并且绝不通配。 | [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| 部分初始化或惰性循环 Module | `PROHIBITED` | Module 初始化是无环确定性 DAG。 | [08-modules-metaprogramming.md](08-modules-metaprogramming.md) |
| 脚本访问 Host ABI、扩展函数表、原始运行时句柄、加载器内部，或 FFI 之外的任意原生符号 | `PROHIBITED` | 源自脚本的外部二进制调用只使用标准 FFI 子系统。 | [09-native-host-ffi.md](09-native-host-ffi.md) |
| 稳定 Rust、C++、内部对象、指针、布局或 vtable ABI | `PROHIBITED` | C 是稳定 Host/扩展 ABI。 | [09-native-host-ffi.md](09-native-host-ffi.md) |
| 无签名或原始 unsafe FFI 调用 | `PROHIBITED` | FFI 调用要求显式 sidecar 或程序化签名。 | [09-native-host-ffi.md](09-native-host-ffi.md) |
| 自动反射式对象或 ivar 序列化 | `PROHIBITED` | 序列化是显式且 Contract 驱动的。 | [10-serialization-standard-library.md](10-serialization-standard-library.md) |
| 语言身份章节内的 IrisValue 字节/tag 微模式 | `DEFERRED V1 to format specification` | 本章只固定格式责任。 | [10-serialization-standard-library.md](10-serialization-standard-library.md) |
| 公共核心加密套件来自内部 BLAKE3 使用 | `PROHIBITED` | 加密 API 是独立的官方标准包。 | [10-serialization-standard-library.md](10-serialization-standard-library.md) |
| HTTP、通用网络协议、数据库、GUI 和类似设施进入语言核心 ABI | `DEFERRED V1 to standard packages` | 这些领域作为独立版本化的官方标准包发布。 | [10-serialization-standard-library.md](10-serialization-standard-library.md) |

IRIS-V1-IDENTITY-C034: 如果后续章节需要 v1 延后表中列出的功能，它必须 (MUST) 保留所列状态，缩窄后续章节以避开该功能，或在声称 v1 一致性之前更新已批准决策来源。后续章节不得 (MUST NOT) 静默地将延后项转换为规范性语义。

## 本章的可追溯性覆盖

IRIS-V1-IDENTITY-C035: 未来的可追溯性矩阵必须 (MUST) 将本章映射到 D-001、D-002、D-073、D-074、D-075、D-076、D-077、D-078、D-079、D-080、D-081、D-082、D-083、D-084、D-085、D-086、D-087、D-174、D-175、D-176、D-177、D-178、D-179、D-180、D-181、D-242、D-243、D-244、D-245、D-246、D-247、D-248、D-249、D-250、D-322、D-329、D-421、D-452、D-453、D-454、D-455、D-456、D-457、D-458、D-468、D-472、D-487、D-488、D-489、D-490 和 D-504。当后续章节细化本身份章节只框定的细节时，它也可以 (MAY) 链接相邻决策。

IRIS-V1-IDENTITY-N002: Informative note: D-073 到 D-087 定义稳定公共数值哈希身份。本章只捕获版本控制和实现独立性承诺。规范字节语法和哈希向量属于 [06-collections-text-regex.md](06-collections-text-regex.md) 和 [12-conformance.md](12-conformance.md)。


## 身份覆盖向量

IRIS-V1-IDENTITY-C036: 以下向量是带有具体审计输入的规范性可追溯性向量。它们覆盖身份和兼容性决策，而不创建实现代码。

| 向量 ID | 类别 | 适用性 | 源/输入 | 预期可观察结果 | 决策 |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-IDENTITY-V001` | diagnostic | documentation validator required; interpreter not applicable; JIT not applicable; native not applicable | 验证固定的 Iris v1 产物集和语义来源声明。 | `spec/iris-v1` 下恰好存在 14 个产品产物；`.omo/drafts/iris-language-specification.md` 是已批准的语义来源；没有实现文件或考古资料是规范性的。 | `D-000`, `D-001` |
| `IRIS-V1-IDENTITY-V002` | diagnostic | documentation validator required; interpreter optional; JIT optional; native optional | 验证使用对象值、动态运算符 dispatch、Closure、抛出任意对象和 Host/native 边界声明的源码语料。 | 只有当这些身份表面存在，且没有被源码兼容的 Legacy Iris 恢复声明替代时，语料才被接受。 | `D-002` |

## 身份覆盖用例

这些行提供本章所拥有身份决策的具体文档和源码输入。

| 向量 ID | 类别 | 适用性 | 源/输入 | 预期可观察结果 | 决策 |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-IDENTITY-V012` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | 文档夹具：`spec/iris-v1/README.md` 加 `01-language-identity.md` 声明 Iris v1 是现代继承者，并且 `11-migration-divergence.md` 记录每项遗留不兼容。 | 只有当兼容性声明存在，且没有条款承诺遗留源码、解析器、ABI 或实现恢复时，文档验证才成功。 | `D-001` |
| `IRIS-V1-IDENTITY-V013` | positive | compiler required; interpreter required; JIT required; native not applicable | Iris source: `class Probe { fun +(other: Object) { 7 } } let p = Probe.new(); let result = p + Object.new(); let block = { raise p }; try { block() } catch value, context { [result, value, context.value] }`; 单独的原生夹具声明 C Host ABI 扩展入口。 | 解释器和 JIT 返回 tuple `[7, p, p]`；`p` 被抛出并原样捕获；源码接受 Class、Method、Closure、operator-message 和 arbitrary-object exception 表面。原生声明无需原生调用即可被接受。 | `D-002` |
| `IRIS-V1-IDENTITY-V014` | positive | interpreter required; JIT required; native not applicable | Iris source fixture 创建两个 `Widget` 实例，提交一个兼容的 `open class Widget` 修订，显式调用 `first.migrate_revision()`，并从两个实例读取修订标记。 | 显式迁移的 `first` 报告新标记；`second` 保留其先前标记，直到应用代码调用其普通 `migrate_revision` Method。提交时不发生隐式枚举或迁移。 | `D-264` |
