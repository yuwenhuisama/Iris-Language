# Iris v1 规范索引

状态：Iris v1.22，冻结语义并附所有者批准的勘误。

翻译说明：本文是 `../README.md` 的忠实简体中文翻译。代码、标识符、稳定 ID、D-ID、字面量和规范性术语保留原文形式或在中文译词后保留英文令牌，以便核对。

IRIS-V1-TRACE-C013: 本目录是正式 Iris v1 规范产物的唯一所在地。已批准的语义来源是 `spec/drafts/iris-language-specification.md`。`legacy/Document/` 下的历史文件、遗留脚本、旧的生成解析器输出和现有实现代码只作为证据。除非某个冻结决策明确采纳某项行为，否则它们不是规范性依据。

IRIS-V1-TRACE-C001: v1 的语言语义已经关闭。作者必须 (MUST) 保留冻结决策，包括取代早期措辞的后续修订。明显空缺必须 (MUST) 记录为 `DEFERRED V1` 或在写作前上报。作者不得 (MUST NOT) 添加、移除、重新解释或静默补全语言特性。

## 产物清单

IRIS-V1-TRACE-C014: Iris v1 规范集恰好包含以下 14 个产品产物，并按此顺序排列：

| 顺序 | 产物 | 用途 |
| --- | --- | --- |
| 1 | `README.md` | 规范性文档索引、编辑契约、术语、状态和阅读顺序。 |
| 2 | `01-language-identity.md` | 目标、兼容性身份、规范性约定、版本控制和 v1 延后项。 |
| 3 | `02-lexical-grammar.md` | 源编码、词元、字面量、关键字、优先级、结合性和合并语法。 |
| 4 | `03-runtime-object-model.md` | 值、身份、相等性/哈希、Class/Module/Contract、MRO、修订、Method、属性和内建项。 |
| 5 | `04-bindings-callables-control-flow.md` | 绑定、作用域、可调用形式、参数、Closure 捕获、调用、赋值、条件、循环、match 和异常。 |
| 6 | `05-types-contracts-generics.md` | 渐进 Contract、Dynamic、并集与交集、可 nil 性、转换、推断、泛型、Contract 视图和 Never。 |
| 7 | `06-collections-text-regex.md` | Tuple、Array、Hash、Range、迭代、String、MutableString、Symbol、Bytes、ByteArray、Regex 和哈希边界。 |
| 8 | `07-async-resources-diagnostics.md` | Task/Awaitable、单线程调度器、Closeable/using、ExceptionContext、stack/cause/suppressed 和诊断流。 |
| 9 | `08-modules-metaprogramming.md` | 包、模块、import/export、可执行声明体、open 事务、修订、装饰器、MetaCapabilities 和 ReflectionPolicy。 |
| 10 | `09-native-host-ffi.md` | 稳定 C Host/扩展 ABI 边界、句柄、GC/native 载荷、异步完成、FFI 脚本 API 和 Rust/C++ 集成规则。 |
| 11 | `10-serialization-standard-library.md` | JSON/IrisValue 范围、Encoding API、核心包与标准包，以及延后的库领域。 |
| 12 | `11-migration-divergence.md` | 每一项有意的遗留差异和迁移示例。 |
| 13 | `12-conformance.md` | 示例/向量 ID 模式、正向/负向/诊断/差异覆盖和冻结标准。 |
| 14 | `traceability-matrix.md` | 每个冻结 D-ID 映射到精确的规范性条款、示例/向量、差异条目或明确延后状态。 |

IRIS-V1-TRACE-C002: Iris v1 的其他正式产品产物不得 (MUST NOT) 属于本目录。若后续任务需要支持脚本或生成证据，它们必须 (MUST) 位于这个 14 文件产品清单之外，除非计划已更新。

## 编辑契约

IRIS-V1-TRACE-C003: 规范性文本定义符合 Iris v1 的实现、工具、规范或程序必须满足的要求。当意图表达要求时，规范性条款必须 (MUST) 使用 RFC 2119 风格术语：`MUST`、`MUST NOT`、`SHOULD`、`SHOULD NOT` 和 `MAY`。这些术语在规范性条款中承载其通常的规范含义。

IRIS-V1-TRACE-C004: 信息性文本解释理由、给出示例、描述考古资料，或说明实现自由度。信息性文本必须 (MUST) 使用下列前缀之一明确标注：

| 标签 | 用途 |
| --- | --- |
| `Informative note:` | 不创建要求的解释性文字。 |
| `Informative example:` | 不可执行或说明性的示例文字。 |
| `Implementation note:` | 允许的实现策略或警告，而不是语义规则。 |
| `Historical note:` | 遗留行为或来源考古，而不是 v1 规则。 |

IRIS-V1-TRACE-C005: 每项规范性行为、语法规则、诊断规则、语义边界或延后项都必须 (MUST) 有稳定条款 ID。每个可执行示例和每个一致性向量都必须 (MUST) 有稳定 ID。没有这些 ID 之一的段落不得 (MUST NOT) 包含规范性要求。

IRIS-V1-TRACE-C006: 每章作者必须 (MUST) 保持当前术语规范。历史过期术语背景：`Interface`、`StringBuilder`、一等 `ReflectionCapability` 令牌，以及永久绑定实例的 ClassRevision 都是非规范历史名称。它们必须 (MUST) 只出现在明确标注的历史或迁移文本中，或出现在明确指出被替代历史术语的规范替换术语表行中。

## 稳定 ID

IRIS-V1-TRACE-C007: 条款、示例和向量 ID 一经发布就是永久的。不要为了给新材料腾出空间而重新编号现有 ID。改为添加带后缀的子 ID 或下一个可用序号。

| ID 类型 | 语法 | 示例 | 含义 |
| --- | --- | --- | --- |
| 规范性条款 | `IRIS-V1-<chapter>-C<nnn>` | `IRIS-V1-RUNTIME-C001` | 必需规则或明确延后项。 |
| 信息性说明 | `IRIS-V1-<chapter>-N<nnn>` | `IRIS-V1-GRAMMAR-N001` | 非规范性解释说明。 |
| 示例 | `IRIS-V1-<chapter>-EX<nnn>` | `IRIS-V1-TYPES-EX001` | 绑定到一个或多个条款的源代码示例。 |
| 一致性向量 | `IRIS-V1-<chapter>-V<nnn>` | `IRIS-V1-FFI-V001` | 机器可读的正向、负向、诊断或差异测试用例。 |
| 迁移行 | `IRIS-V1-MIG-<nnn>` | `IRIS-V1-MIG-001` | 差异台账中的遗留行为处置。 |
| 可追溯性行 | `D-<nnn>` | `D-509` | 来自已批准草案的冻结决策标识符。 |

IRIS-V1-TRACE-C015: 章节代码固定如下：

| 产物 | 章节代码 |
| --- | --- |
| `01-language-identity.md` | `IDENTITY` |
| `02-lexical-grammar.md` | `GRAMMAR` |
| `03-runtime-object-model.md` | `RUNTIME` |
| `04-bindings-callables-control-flow.md` | `CONTROL` |
| `05-types-contracts-generics.md` | `TYPES` |
| `06-collections-text-regex.md` | `COLLECTIONS` |
| `07-async-resources-diagnostics.md` | `ASYNC` |
| `08-modules-metaprogramming.md` | `META` |
| `09-native-host-ffi.md` | `FFI` |
| `10-serialization-standard-library.md` | `LIBRARY` |
| `11-migration-divergence.md` | `MIGRATION` |
| `12-conformance.md` | `CONFORMANCE` |
| `traceability-matrix.md` | `TRACE` |

IRIS-V1-TRACE-C008: 标题锚点应当 (SHOULD) 是从标题文本生成的普通 GitHub Markdown 锚点。条款 ID 必须 (MUST) 出现在标题中或条款段落开头，以便自动检查无需解析正文即可找到它们。

## 规范术语

IRIS-V1-TRACE-C016: 在 Iris v1 规范中一致使用这些术语：

| 术语 | 本规范中的定义 |
| --- | --- |
| Iris v1 | 第一个正式复兴的 Iris 语言规范，是遗留 Iris 的现代继承者，带有已冻结的 v1 语义。 |
| Legacy Iris | 作为考古和迁移证据使用的历史语言、文档、脚本和实现，不自动具有权威性。 |
| Contract | Iris 对显式接口/协议风格静态和动态义务表面的规范术语。规范替换背景：在官方 v1 术语中使用 `Contract`，不要使用历史 `Interface`。 |
| MutableString | 规范的可变文本类型。规范替换背景：在官方 v1 术语中使用 `MutableString`，不要使用历史 `StringBuilder`。 |
| ReflectionPolicy | 由运行时和 Host 控制的策略，按调用方包、目标、操作和范围授予或拒绝反射检查与变更。规范替换背景：它不是历史上的一等 `ReflectionCapability` 令牌。 |
| logical Class | Iris 程序在允许的动态变更期间可见的稳定 Class 身份。一个 logical Class 可以有变化中的 active revision。 |
| active revision | logical Class、Module 或相关运行时声明表面的当前已提交修订，dispatch 和 reflection 在给定时刻观察到它。 |
| Method | 安装在 Class、Module、Contract 视图或相关接收者表面上的可调用成员。在章节规则如此规定时，运算符是 Method 发送。 |
| BoundMethod | 与接收者或绑定上下文耦合以便调用的 Method。其精确身份和捕获规则属于运行时章节和可调用章节。 |
| Closure | 具有捕获环境的词法可调用值，按可调用与控制流章节规定。 |
| FFI | 标准的脚本级外部二进制集成子系统。对 v1 而言，它是源自脚本的外部二进制调用的唯一路径，并通过声明签名支持稳定 C ABI 绑定。 |
| Host ABI | 主程序和原生扩展用于嵌入或扩展 Iris 的稳定 C 边界。Rust 包装器是便利 API，不是二进制兼容性契约。 |
| DEFERRED V1 | v1 语义契约的一个具名非目标。延后项受 IRIS-V1-TRACE-C005 管辖，且不得 (MUST NOT) 被规范性文字暗示。 |

## 阅读和依赖顺序

IRIS-V1-TRACE-C009: 读者和作者应当 (SHOULD) 使用此依赖顺序：

| 步骤 | 阅读 | 原因 |
| --- | --- | --- |
| 1 | `README.md` | 建立清单、ID、术语和范围。 |
| 2 | `01-language-identity.md` | 定义语言身份、兼容性姿态、规范性词汇、版本控制和延后项。 |
| 3 | `02-lexical-grammar.md` | 定义后续语义章节引用的解析器可见形式。 |
| 4 | `03-runtime-object-model.md` | 定义值、身份、dispatch、修订、对象模型和内建运行时基础。 |
| 5 | `04-bindings-callables-control-flow.md` | 对名称、可调用值、控制转移和异常依赖语法与运行时模型。 |
| 6 | `05-types-contracts-generics.md` | 对 Contract 和渐进类型依赖身份、运行时和可调用基础。 |
| 7 | `06-collections-text-regex.md` | 对字面量、迭代、文本、Regex 和哈希依赖语法、运行时、控制流和类型 Contract。 |
| 8 | `07-async-resources-diagnostics.md` | 对 Task、Awaitable、清理和诊断依赖可调用/控制语义与运行时对象模型。 |
| 9 | `08-modules-metaprogramming.md` | 对变更、装饰器和反射依赖 Class、Module、Contract、类型和修订规则。 |
| 10 | `09-native-host-ffi.md` | 对原生和脚本二进制边界依赖元数据、Contract、运行时、资源和反射规则。 |
| 11 | `10-serialization-standard-library.md` | 对库范围依赖集合、文本、Regex、FFI 和 Contract 边界。 |
| 12 | `11-migration-divergence.md` | 依赖所有规范性章节，以便每项遗留差异都能指向替换规则。 |
| 13 | `12-conformance.md` | 依赖规范性章节来定义可执行示例、向量、诊断和冻结门槛。 |
| 14 | `traceability-matrix.md` | 依赖完成后的规范集，以便每个冻结决策映射到精确条款、示例、向量、差异行或延后项。 |

## 冻结语义状态

IRIS-V1-TRACE-C017: 已批准草案记录 `status: approved`，并说明当前 v1 范围内的每个语义和设计问题都已冻结或明确延后。D-000 固定此输出目录。D-001 将兼容性身份固定为现代继承者，而不是源码兼容的恢复版。D-002 将核心语言身份固定在以下层面：每个值都是对象、动态消息 dispatch、Closure、受约束的运行时变更、异常抛出、嵌入和原生扩展支持。

IRIS-V1-TRACE-C010: D-509 关闭 v1 保留关键字清单。D-510 固定所列冲突的上下文最长匹配词元处理。D-511 到 D-514 固定装饰器形状、确定性规划、MetaCapabilities 限制、失败行为、重放和反射。后续章节必须 (MUST) 保留这些决策，并将它们链接到精确条款，而不是在正文中松散复述。

## 实现和非目标

IRIS-V1-TRACE-C011: 本规范集不实现 Iris。本目录中的工作不得 (MUST NOT) 修改编译器、解析器、运行时、VM、GC、JIT、CLI、扩展 SDK、FFI 绑定、测试、生成的解析器源码、`legacy/Document/` 或遗留脚本语料。

IRIS-V1-TRACE-C018: 以下是本文档波次的明确非目标：

| 非目标 | 状态 |
| --- | --- |
| Rust runtime、parser、VM、GC、JIT、extension SDK 或 workspace crates | `DEFERRED V1 implementation` |
| 移动、重写或重新标记 `legacy/Document/` 下的考古资料 | `OUT OF SCOPE` |
| 默认将旧 PDF、生成的解析器文件或遗留实现视为规范性依据 | `OUT OF SCOPE` |
| 在解释器和 JIT 之间改变语义的后端特定行为 | `PROHIBITED` |
| 新语言特性或对冻结决策的重新解释 | `PROHIBITED` |
| 超出精确 14 文件清单的额外产品产物 | `PROHIBITED unless the plan changes` |

IRIS-V1-TRACE-C012: 只有在保留冻结 v1 语义所要求的可观察行为时，才可以记录实现自由度。
