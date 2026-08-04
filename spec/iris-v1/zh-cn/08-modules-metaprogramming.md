# Iris v1 模块与元编程

Status: Iris v1 draft, frozen semantics.

IRIS-V1-META-C001: 本章定义 Iris v1 的包身份、Module 源结构、导入、导出、再导出、初始化顺序、manifest 与 lock 语义、声明体执行、声明式与程序式 open 事务、候选隔离、安全点提交、冲突处理、修订审计与回滚入口点、Module 组合授权、装饰器、MetaCapabilities、ReflectionPolicy、反射视图、`respond_to?`、原始 ivar 反射，以及静态成员可见性与动态成员可见性。它 MUST 在阅读 [README.md](README.md)、[01-language-identity.md](01-language-identity.md)、[02-lexical-grammar.md](02-lexical-grammar.md)、[03-runtime-object-model.md](03-runtime-object-model.md)、[04-bindings-callables-control-flow.md](04-bindings-callables-control-flow.md) 和 [05-types-contracts-generics.md](05-types-contracts-generics.md) 之后阅读。

IRIS-V1-META-C002: 本章 MUST NOT 重新定义普通派发、Method 与 BoundMethod 身份、Contract 类型代数、Contract 视图哈希、异步调度、ExceptionContext 所有权、原生 ABI 句柄、源 token 清单，或稳定序列化格式。这些规则仍由先前的运行时、可调用、类型、异步、原生和库章节拥有。本章只说明这些表面必须保持的 Module、包、事务、装饰器、能力与反射义务。

## 包身份、Manifests 与 Locks

IRIS-V1-META-C003: 可发布包 MUST 在 `iris.toml` 中声明全局唯一的反向域名风格 `package_id`、`api_major`、包语义版本、Iris 语言主版本或版本范围、有序源或 Module 条目、依赖约束、权限请求、适用时的原生制品声明，以及入口 Modules。无 manifest 的本地脚本 MAY 只使用运行时本地包身份运行，如 [01-language-identity.md](01-language-identity.md) 所定义。

IRIS-V1-META-C004: 命名 nominal Types、Modules、Classes、Contracts 以及包拥有的反射元数据的包身份 MUST 使用 `(package_id, api_major, Iris language major, fully qualified name, type kind, generic arity, closed arguments where applicable)`。它 MUST NOT 依赖文件系统路径、显示名、源哈希、机器本地构建数据、加载上下文，或源枚举顺序。

IRIS-V1-META-C005: 运行时 MUST 为每个 `(package_id, api_major)` 最多激活一个包实现修订。同主版本的 minor 或 patch 修订共享 nominal identity，不能作为隐藏加载上下文身份共存。不兼容共存需要不同的 `api_major`。

IRIS-V1-META-C006: 依赖解析 MUST 在初始化前选择精确包版本和制品 digest，并把它们记录到精确 lockfile 中。构建和运行时加载 MUST 使用选定的 lock 结果，并且 MUST NOT 隐式获取、解析或偏好最新版本。

IRIS-V1-META-C007: 包依赖约束使用受包 API major 身份约束的 SemVer 兼容范围。解析失败、不兼容 API major 合一、缺少必需制品，或 digest 不匹配 MUST 在任何依赖 Module 体执行前中止包链接。

IRIS-V1-META-C008: Manifest `permissions` 条目是带命名空间的请求，不是授权。包 MAY 请求 `reflection.inspect`、`reflection.mutate`、`filesystem.read`、`network.connect` 或 `native.load` 等名称，并附带 required 或 optional 状态与 scope 元数据。Host 或 CLI MUST 显式授予一个子集。未指定权限默认 deny，未获授权的 required 请求 MUST 阻止包加载。

IRIS-V1-META-C009: 运行时范围的受信任反射策略不能由普通包 manifest 请求。它只通过受信任工具的 Host 配置或专用 host 包身份存在。包、依赖、原生制品或动态导入 MUST NOT 自行授权权限。

IRIS-V1-META-C010: 以下包表是规范性的:

| 表面 | 来源 | 身份规则 | 失败规则 | 反射暴露 |
| --- | --- | --- | --- | --- |
| 可发布包 | 带 `package_id` 和 `api_major` 的 `iris.toml` | 在兼容 minor 和 patch 发布间稳定 | 缺少或无效 manifest 中止加载 | 包元数据视图 |
| 无 manifest 脚本 | Host 选择的本地入口 | 仅运行时本地 | 不能声明稳定公共 Type 身份 | 本地包元数据视图 |
| 依赖 | SemVer 范围加 API major | lockfile 中一个选定精确版本 | 解析或 digest 失败中止链接 | 选定包和 digest 元数据 |
| 权限请求 | Manifest `permissions` | 仅请求，不授权 | Required 未授权请求中止加载 | 请求和授权来源元数据 |
| 热升级 | `Package.upgrade(...)` | 相同 `(package_id, api_major)` | 候选失败会让旧包保持活动 | 旧修订和新修订审计元数据 |

## Module 源模型、导入、导出与初始化

IRIS-V1-META-C011: 一个源文件 MAY 包含多个显式 `module` 和 `open module` 块。在 Module 块外，源 MAY 只包含包或文件导入、导出、再导出，以及包级声明式元数据。普通可执行语句和源 `fun` 声明 MUST 属于 Module 体。

IRIS-V1-META-C012: 在一个包内，每个限定 Module 名恰好有一个写作 `module Name ... { ... }` 的 origin 声明。对该 Module 的额外同包或跨文件扩展 MUST 使用 `open module Name ... { ... }`。一个限定 Module 的多个隐式 origin MUST 被拒绝。

IRIS-V1-META-C013: 导入是静态、显式、可别名、可选择的。支持的形式是 `import pkg::Module`、`import pkg::Module as Alias` 和 `from pkg::Module import Name, Other as Alias`。通配导入和运行时字符串导入不是 Iris v1 源的一部分。

IRIS-V1-META-C014: 静态导入在可执行 Module 体语句前解析。它们只引入源请求的命名 Modules 或条目，并且 MUST NOT 隐式再导出名称、通过间接导入激活扩展成员，或把运行时加载的名称加入已经编译的词法命名空间。

IRIS-V1-META-C015: `export` 标记命名 Classes、Modules、Contracts、Type aliases、常量、globals、`main` 上的 Module 体 Methods，以及导出的 open extension artifacts 为公共包 API。再导出通过 `export import` 或 `export from` facade 声明显式完成。

IRIS-V1-META-C016: 再导出暴露名称和 Contracts，但 MUST NOT 算作激活另一个 Module 导出的静态 open extension 所需的直接导入。想要静态扩展成员的消费者 MUST 直接导入导出该 open extension 的 Module。

IRIS-V1-META-C017: Module 初始化形成无环确定性 DAG。静态依赖先于依赖方初始化。Module 执行其 origin 体，然后按照 manifest 声明的源和块顺序执行适用的 open 块。依赖循环或初始化循环 MUST 是编译或链接错误。

IRIS-V1-META-C018: 符合要求的运行时 MUST NOT 暴露部分初始化或惰性循环 Modules。如果 Module 初始化失败，该 Module 及其依赖方在该次加载尝试中标记为 failed。运行时 MUST NOT 自动重试。需要显式包 reload 或 upgrade。

IRIS-V1-META-C019: 在每个依赖初始化之后，直接导入的源顺序决定兼容扩展覆盖顺序。文件系统枚举、并发调度、Module 名排序、hash-map 迭代，或任意拓扑选择 MUST NOT 决定静态扩展替换行为。

IRIS-V1-META-C020: 语言导入从不接受运行时字符串或值。受权限控制的 `Package.load(id, version...)` MAY 动态加载并初始化包或 Module，并返回有界 `Dynamic<Module>` 句柄或反射元数据。它 MUST NOT 追溯性地向已经编译的命名空间加入名称、Types 或静态扩展。

IRIS-V1-META-C021: 以下 Module 初始化表是规范性的:

| 状态 | 进入方式 | 对依赖方可见 | 失败行为 | 重试行为 |
| --- | --- | --- | --- | --- |
| Unlinked | Resolver 尚未选择精确依赖 | 无 | 解析失败在 Module 执行前中止 | 显式重新运行包解析 |
| Linked | Lockfile 和导入已解析 | 仅静态名称，没有已执行体状态 | 链接循环或缺少制品中止 | 输入变更后重新运行 |
| Initializing | Origin 或 open 体事务正在运行 | 对依赖方无部分体状态 | 失败标记 Module failed | 不自动重试 |
| Initialized | Origin 和适用 opens 已提交 | 导出的 API 和运行时状态可见 | 之后的修订事件失败不能撤销它 | 包 reload 或 upgrade 可替换它 |
| Failed | 初始化 raised 或验证失败 | 没有可用的已初始化 Module | 依赖方失败 | 仅显式 reload 或 upgrade |

## 可执行声明体

IRIS-V1-META-C022: Class 和 Module origin 体是在候选 Class 或 Module 上执行的构造事务。它们可以运行普通同步控制流、声明成员、调用元编程 API，并使用词法 locals。成功会验证完整候选并原子发布。失败不会发布该候选的任何内容。

IRIS-V1-META-C023: 在 Class 或 Module origin 或 open 体执行期间，`self` 是稳定的逻辑 Class 或 Module 对象。发送给 `self` 的结构性 meta messages，比如 `include`、`define_method` 或 `superclass=`，以当前事务候选为目标，而不是直接改变已发布的活动修订。

IRIS-V1-META-C024: 可执行 Class 或 Module 体内的非限定调用先解析词法 locals、parameters、captures、constants、Types 和 imports。如果没有找到可调用绑定或声明，它们等价于在可调用章节规则下对当前 `self` 或 Module `main` 接收者的特权发送。

IRIS-V1-META-C025: Instance Methods 只能通过显式声明语法或显式 meta-operation API 进入候选实例 Method 表。普通体调用不会隐式定义 instance Methods、properties、storage slots 或 Contract requirements。

IRIS-V1-META-C026: 可执行 Class 或 Module 体内的 Method 声明不会闭包捕获该体执行创建的词法 locals。它的词法环境包含定义或 Module 作用域、类型参数、owner 元数据和 Method 参数，而不是事务临时 locals。

IRIS-V1-META-C027: Class 和 Module 体 locals 是普通词法 locals。嵌套 Closures 可以捕获它们，逃逸 Closures 可以延长它们的生命周期。这类 locals 不会仅因为体事务提交就变成 properties、class state、Module state 或持久 storage。

IRIS-V1-META-C028: origin 或编译器可解析声明式 open 体中的顶层无条件声明 MAY 按本章静态可见性规则贡献编译器可见的静态 API。在 `if`、loops、nested blocks 或其他运行时控制流中条件执行的成员或组合，只在该路径执行时影响候选动态表面。

IRIS-V1-META-C029: Contract 体不是可执行声明事务。Contract 体 MAY 只包含类型章节允许的静态要求和元数据。它 MUST NOT 包含 Method bodies、stored state、initializers、raw ivars、private requirements、可执行语句或 open operations。

## Open 事务与候选隔离

IRIS-V1-META-C030: `class Name ... { ... }` 和 `module Name ... { ... }` 是 origin 声明。之后的结构元编程 MUST 使用显式 `open class Name { ... }`、`open module Name { ... }`，或程序式 `Class#open` 和 `Module#open` 事务 API。

IRIS-V1-META-C031: `contract` 声明是静态的，不能被 opened。任何源或程序式打开 Contract 的尝试 MUST 在发布前被拒绝，并且 MUST NOT 创建候选。

IRIS-V1-META-C032: 在一个编译单元内，`open class` 或 `open module` MAY 出现在 origin 声明之前，因为声明收集和静态脊柱解析先于 open 调度。跨 Modules 或包时，opener MUST 显式依赖定义 origin 的 Module，并且 origin 链接与初始化 MUST 在 open 事务执行前完成。

IRIS-V1-META-C033: 声明式 `open class` 和 `open module` 是与 `Class#open` 和 `Module#open` 使用相同事务模型的静态可解析语法。程序式 open 接受别名、反射结果，或动态选择的 Class 和 Module 对象，但它 MUST NOT 以 Contracts 或 closed generic Classes 或 Modules 为目标。

IRIS-V1-META-C034: open 事务体接收其目标的稳定逻辑身份，同时结构操作在当前事务组中为每个目标改变一个隐式候选。正常完成会验证并提交。Exception、无效控制转移、验证错误、capability denial、permission denial 或 conflict 会回滚组内每个候选。

IRIS-V1-META-C035: open 块及其同步动态调用链在写入后读取自身候选结构元数据。该事务上下文外的代码、其他 Tasks 和普通 instance sends 继续观察已发布的活动修订，直到成功原子发布。

IRIS-V1-META-C036: open 事务内的普通 instance sends MUST 使用已发布的活动修订。Candidate Methods、MRO、Modules 和 properties 只通过事务感知的 Class 或 Module 元数据与反射读取可见。Iris v1 没有候选实例预览。

IRIS-V1-META-C037: Open 和 revision 事务体是同步、线程受限、不挂起且不逃逸的。它们 MUST NOT `await`、用可恢复事务状态让调度器 yield、转移到另一线程、与子线程共享候选 mutation，或让候选或事务 authority 逃逸。静态违规是编译错误。动态违规 raise `MetaTransactionError`。

IRIS-V1-META-C038: 同线程嵌套 open 调用加入最外层事务组。内部 opens 从不独立提交。重新打开组内已有目标会复用该目标候选。组内所有候选一起验证并一起发布，或全部回滚。

IRIS-V1-META-C039: 候选构造 MUST NOT 持有长期独占 Class 或 Module 锁。每个候选记录每个目标的 base active revision。提交时，如果任何重叠目标自 base revision 起已改变，事务 MUST raise `MetaTransactionConflictError`，回滚每个候选，并且不发布任何内容。

IRIS-V1-META-C040: Iris v1 在 `MetaTransactionConflictError` 之后不提供自动 open retry、rebase、merge 或隐藏 block 重新执行。用户代码 MAY 捕获冲突并显式重试，并拥有 external effects、compensation、backoff、retry limits 和 idempotency。

IRIS-V1-META-C041: 结构 open 提交使用全局运行时 safepoint。验证后，提交会在 safepoints 停止参与的运行时执行，原子发布所有活动 Class 和 Module 修订以及受影响状态和失效元数据，然后恢复执行。线程 MUST NOT 观察到部分结构混合。

IRIS-V1-META-C042: origin、open、package upgrade、decorator application、closed generic materialization、migration hook 和 rollback reconstruction 的运行时回滚只覆盖运行时拥有的候选状态。文件系统、网络、数据库、外部进程、native-internal、logging、对已经发布对象的 mutation，或其他 external effects 不会自动反转。

IRIS-V1-META-C043: 以下事务状态表是规范性的:

| 事务状态 | 普通 instance sends | 候选元数据读取 | 退出路径 | 已发布状态 |
| --- | --- | --- | --- | --- |
| No transaction | 已发布活动修订 | 仅已发布元数据 | 普通控制 | 当前活动修订 |
| Candidate executing | 已发布活动修订 | 事务读取自身候选 | 体可以完成或 raise | 没有候选状态全局可见 |
| Nested candidate executing | 已发布活动修订 | 相同外层事务组 | 内部完成返回外层 | 没有独立内部提交 |
| Validation | 已发布活动修订 | 冻结候选快照 | 成功进入 safepoint，失败回滚 | 无部分状态 |
| Safepoint commit | 暂停，或旧的 entered frames 继续所选 bodies | 提交拥有的不可变数据 | 原子发布 | resume 后新修订可见 |
| Rollback | 已发布活动修订保持不变 | 候选丢弃 | 失败或 conflict 传播 | 不发布候选内容 |

## 静态成员可见性与动态成员可见性

IRIS-V1-META-C044: Iris v1 为元编程提供三层成员可见性: origin static API、编译器可解析声明式 open static API，以及 dynamic-only runtime API。origin static API 按导出规则对声明包和导入方通用。

IRIS-V1-META-C045: 声明式 open 中的顶层无条件成员 MAY 在定义 Module 内扩展 static API。跨 Module 静态可见性需要 `export open ...`，且每个消费者 MUST 直接导入扩展 Module。传递导入和再导出 MUST NOT 激活静态扩展成员。

IRIS-V1-META-C046: 程序式 open additions 和条件可执行体 additions 是 dynamic-only。它们可通过获准反射发现，可通过普通运行时派发使用，并且只能通过 `Dynamic<T>` 静态使用，或在编译并直接导入声明或导出它们的新静态制品后静态使用。

IRIS-V1-META-C047: 运行时不能把 dynamic-only 成员提升进已经编译的静态 Type API。Dynamic-only 元数据 MUST 记录 origin package、source 或 artifact identity、revision、commit、visibility，以及 dynamic-only static visibility status。

IRIS-V1-META-C048: 当多个直接导入的声明式扩展为一个 Class 或 Module 贡献相同成员时，只有在后一个直接导入的完整静态 signature、visibility 和 Contract obligations 兼容时，它才按源顺序替换早前实现。不会形成 overload set。

IRIS-V1-META-C049: 直接导入如果会替换已合并的静态扩展成员，需要使用语言接受的 `override` import marker 在导入点显式授权替换。如果当前 grammar profile 中没有这种接受的 marker，替换 MUST 被拒绝，而不是静默接受。本规则保留语义要求，但不授予隐藏语法。

IRIS-V1-META-C050: 任何同 Module origin 或 open 声明如果替换已解析成员，MUST 带有成员级 `override`。缺少 `override`、`override` 没有目标，以及不兼容替换都是错误。这适用于 instance Methods、class-object Methods、getters、setters、qualified Contract implementations、Module Methods，以及条件动态替换路径。

IRIS-V1-META-C051: 意图满足声明 Contract requirement 的成员 MUST 按类型章节规定使用 `impl`。如果同一声明还替换 ancestor 或 Module 成员，在适用处同时需要 `override` 和 `impl`。

IRIS-V1-META-C052: 动态和静态成员边界 MUST 保持普通 selector identity。静态类型、期望返回 Type、generic argument、union branch 或 extension import order MUST NOT 创建 overload dispatch 或在不兼容实现间选择。

## Module 组合与 Private 授权

IRIS-V1-META-C053: Class 和 Module 声明中的声明式 header `mixin` 贡献静态组合。origin 或 open 事务内的程序式 `include`、removal 和 re-inclusion 影响候选运行时动态表面，除非该组合也是编译器可见静态声明的一部分。

IRIS-V1-META-C054: Class 和 Module 组合操作 MUST 在发布前验证 Module `Self` constraints、generic Module arguments、static spine promises、Contract implementation obligations、MRO validity、MetaCapabilities，以及 native 或 layout safety。

IRIS-V1-META-C055: 一个 `modules` MetaCapability 控制 Module include、mixin、removal、replacement、ordering 和 recomposition。Iris v1 不拆分 add、remove、order 或 per-Module composition capability flags。

IRIS-V1-META-C056: Module 组合默认不授予 private Method 访问。从组合 Module Methods 访问 host Class 的 private Method 需要在静态或动态组合边上显式授权，比如 grammar 或 API 接受的 `private` composition option。

IRIS-V1-META-C057: Module private 授权限定于 `(host logical Class, closed Module identity, composition edge, edge revision)`。一个 host、package、generic instantiation 或旧 edge 中的授权 MUST NOT 授予另一个中的访问。移除 Module edge 会原子撤销该 edge 的 private authorization。

IRIS-V1-META-C058: 组合的 Module Methods 可以按 raw ivar 规则读取和写入当前 receiver 上非限定 `@name` raw ivars，无论 edge 是否授予 private Method access。Private authorization 只影响 private selectors。

IRIS-V1-META-C059: host Class 声明的 Protected Methods 可以作为该 host 实现层的一部分从组合 Module Methods 调用，受运行时章节的 protected receiver rule 约束。这不会授予 private access，也不会改变 raw ivar current-receiver semantics。

IRIS-V1-META-C060: v1 中的 Module reordering 使用同一事务内显式 remove 加 include。重新 include 已存在的 closed Module 是幂等的，且 MUST NOT 移动它。Removal 后 re-inclusion 创建新 edge，并要求再次说明 edge metadata，包括 private authorization。

## 修订、审计、回滚与热升级

IRIS-V1-META-C061: 每个成功结构事务组获得一个运行时范围单调 `commit_id`。每个受影响 Class 或 Module 获得自己的 per-owner 单调 revision number。失败或回滚的候选 MUST NOT 消耗 per-owner revision number 或运行时 `commit_id`。

IRIS-V1-META-C062: 被取代的可执行修订在被 entered frames、保留的 Method 或 BoundMethod 对象、Closures 或 code artifacts、native handles 或 pins、revision reflection objects、JIT dependencies、debugger 或 profiler pins，或其他显式旧修订引用保留时继续加载。普通实例本身不会 pin 旧 dispatch revisions。

IRIS-V1-META-C063: 轻量审计历史 MAY 在重型可执行修订状态被回收后保留。审计记录标识 owner、revision number、commit ID、package、source 或 artifact、initiator、timestamps 或 context、immutable artifact locator、digest 和 change summary。审计记录不可执行，除非取回精确 artifacts。

IRIS-V1-META-C064: `Class.rollback(target)` 及对应 Module 或 package rollback 入口点是新的结构事务。它们从当前活动状态加精确历史 artifacts 重建候选，验证当前 static spine、Contracts、MRO、visibility、native obligations 和 MetaCapabilities，然后在成功时发布新 revision 和 commit ID。

IRIS-V1-META-C065: Rollback MUST NOT 直接重新激活历史修订。同主版本 rollback MUST NOT 降级当前 static spine。缺少之后必需成员、Contracts、generic arity、visibility obligations、native obligations 或 superclass bounds MUST 导致 rollback 失败且不发布任何内容。

IRIS-V1-META-C066: 审计 rollback artifact retrieval MUST 通过活动包存储或显式用户 resolver 解析精确 source、bytecode、native artifact 或 manifest，并验证存储的 digest。缺失、不可访问或不匹配 artifacts MUST raise `RevisionArtifactUnavailableError` 且不发布任何内容。

IRIS-V1-META-C067: Revision artifact integrity 使用完整 BLAKE3-256 canonical manifest，覆盖影响语义的 Iris source blobs、bytecode 或 IR、native binaries、dependency package IDs、API majors、exact revisions、Iris language、compiler 和 runtime ABI versions，以及 build options 或 path-to-content mappings。Locator bytes 排除在 digest 外。

IRIS-V1-META-C068: 通过 `Package.upgrade(...)` 的同主版本包升级是显式且事务性的。运行时为受影响 Modules、generic definitions 和 interned closed constructions、Classes、Methods、properties、native ABI obligations 与 extension effects 构建候选，把它们作为一个事务验证，并在成功时原子切换 active revisions。

IRIS-V1-META-C069: 同主版本包升级复用 static Contracts 保持兼容的现有 Module、Class、shared 和 per-closed storage，并且不重新运行它们的普通 initializers。新引入的 storage 在候选事务内运行 initializer。显式 `upgrade(from_version, state_or_context)` migration hook MAY 转换或验证候选状态。

IRIS-V1-META-C070: Package upgrade hook failure、validation failure、native binding failure，或 capability 或 permission denial MUST 让旧包和状态完全保持 active。Upgrade hooks 的 external side effects 由包作者按 IRIS-V1-META-C042 负责。

IRIS-V1-META-C071: After-commit revision events 是只读观察，其调度和交付由 [07-async-resources-diagnostics.md](07-async-resources-diagnostics.md) 拥有。本章要求每个结构提交生成这些事件需要的不可变 commit metadata，并禁止任何 event subscriber veto、mutate、retry 或 rollback 已完成提交。

## MetaCapabilities 元能力

IRIS-V1-META-C072: 每个 Class static spine，以及 grammar 接受 `meta deny` 的每个 Module 或 Contract 声明，都携带不可变、可反射的 `MetaCapabilities` policy data。该 policy 描述允许哪些结构操作。每个 syntax、reflection、Dynamic、package、native 和 Host meta path MUST 调用相同的 operation-level capability checks。

IRIS-V1-META-C073: v1 MetaCapabilities 词汇表恰好是 `method_set`、`method_body`、`property_set`、`property_body`、`modules`、`superclass`、`subclass`、`shape`、`class_state_set`、`class_state_write`、`instance_state` 和 `native`。`meta deny` 中的未知名称 MUST 是编译错误。

IRIS-V1-META-C074: 源 `meta deny` 只是否定。没有源 `allow` 形式可以授予不可用运行时能力、恢复 ancestor denial、放宽 Module 或 Contract denial，或创建 runtime-internal capability。Open 块和条件可执行体代码 MUST NOT 改变 `MetaCapabilities` policy。

IRIS-V1-META-C075: `meta` 是每种接纳它的声明 kind 的最终 header clause。Class 顺序是 `extends`、`for`、`mixin`、`where`、`meta`。Module 顺序是 `mixin`、`where`、`meta`。Contract 顺序是 `extends`、`where`、`meta`。`meta deny` 只是静态 header syntax，MUST NOT 出现在可执行或条件体代码中。

IRIS-V1-META-C076: Class 的有效 capabilities 是语言和运行时默认值，减去 declared superclass chain 的每个 denial，减去 local origin denies，减去当前 runtime-superclass 或 MRO Class chain 的每个 denial，减去 active Module-sourced denies，减去 Contract-required denies。候选验证在 operations commit 前计算这个完整集合。

IRIS-V1-META-C077: 子类可以进一步收窄有效 capabilities，但 MUST NOT 重新启用 ancestor static chain 否定的 capability。Runtime superclass changes 可以通过 active runtime chain 收窄 capabilities，但 MUST NOT 恢复 denied static-chain capability。

IRIS-V1-META-C078: Module 可以声明不可变 `meta deny` restrictions。Static mixin 和 dynamic include 会把这些 denies 以 Module identity 作为 policy origin 加入 host Class revision 的 effective capabilities。移除 Module 只移除该 Module-sourced restriction，并且只在没有剩余来源否定它时移除。

IRIS-V1-META-C079: Contract 可以声明不可变 `meta deny` requirements 作为其静态承诺的一部分。声明 `for Contract` 的 Class MUST 有至少同等严格的 effective policy，且 conformance 会把这些 denies 吸收到 Class static spine。Contract inheritance 会合并 deny requirements。

IRIS-V1-META-C080: MetaCapabilities 彼此正交。否定一个 capability MUST NOT 隐式否定另一个。每个 meta operation 声明所有 required capabilities，缺少任何 required capability 都会 raise `MetaCapabilityError`，并带有 target、operation、policy origin 和 reason。整个事务组回滚。

IRIS-V1-META-C081: 以下 capability matrix 对 v1 core meta operations 是规范且完整的:

| 操作 | Required MetaCapabilities | Additional checks | Static API effect |
| --- | --- | --- | --- |
| 添加、移除、alias、undef 或改变 Method slot 或 signature | `method_set` | Static spine、visibility、`override`、`impl`、callable compatibility | 仅 origin 或 exported direct declarative open 为静态 |
| 替换兼容 Method body | `method_body` | 相同 slot signature 和 Contract compatibility | 没有新静态成员 |
| 添加、移除或改变 property getter 或 setter slot 或 signature | `property_set` | Property Contract、visibility、`override`、storage rules | 仅 origin 或 exported direct declarative open 为静态 |
| 替换兼容 property accessor body | `property_body` | 相同 accessor contract | 没有新静态成员 |
| 添加、移除或改变 stored instance property 或 shape descriptor | `property_set`, `shape` | Layout、GC、Contract、native safety | 仅 compiler-visible 时静态 |
| 添加或移除 native-backed storage 或 native structural metadata | operation capabilities 加 `native` | Native ABI 和 safety validation | 取决于 operation source |
| Include、remove、reorder 或 replace Module composition edge | `modules` | Self constraints、MRO、Contract satisfaction、edge authorization | 仅 header 或 exported direct declarative open 为静态 |
| 改变 runtime superclass | `superclass` | Declared superclass bound、MRO、layout、built-in protection | 仅在静态 bounds 内为 dynamic runtime ancestry |
| 选择一个 Class 作为 declared 或 runtime superclass | target effective `subclass` | Target policy 和 subtype validity | 声明 compiler-visible 时静态 |
| 添加、移除或改变 class/shared storage slot | `property_set`, `class_state_set` | Storage Contract、initializer、backed 时 native | 仅 compiler-visible 时静态 |
| 通过 meta APIs 写入现有 class/shared storage value | `class_state_write` | Value Contract 和 visibility | 无静态 shape change |
| 创建或重新创建缺失的 undeclared raw ivar | effective `instance_state` | 反射路径的 ReflectionPolicy、identity-bearing receiver | 仅动态 |
| 删除现有 undeclared raw ivar | 无来自 `instance_state` denial 的能力要求 | 反射路径的 ReflectionPolicy、receiver owns slot | 仅动态清理 |
| Rollback reconstruction | reconstructed operations 所需 capabilities | Exact artifact、current static spine、conflict checks | 与 reconstructed operation source 相同 |
| Decorator metadata attachment 或 compatible body wrapping | generated operation 所需 capability，常为 `method_body` 或 `property_body` | Decorator Contract、target policy、package permission | 仅 deterministic static plan 为静态 |

IRIS-V1-META-C082: Nil、Bool、Integer、Float32 和 Float64 等内建受保护 Classes 使用与用户 Classes 相同的不可变 MetaCapabilities 和 operation checks。它们的 runtime superclass mutation 在统一 meta-operation 边界被拒绝，并中止完整事务组。

IRIS-V1-META-C083: `shape` 管理 Class 范围 layout、declared storage、native layout 和 GC tracing structure。`instance_state` 管理每对象创建和重新创建缺失 undeclared raw ivars。仅 `meta deny shape` 不会密封 undeclared dynamic ivars。`meta deny instance_state` 阻止新的 undeclared raw ivar expansion，同时让 declared typed properties 仍由其自身 Contracts 管理。

IRIS-V1-META-C084: 无身份不可变值不能创建、重新创建或保留 undeclared dynamic ivars。它们的 effective policy 通过同一模型内在地否定 `instance_state`。会创建这类状态的赋值 MUST 在 source、reflection、Dynamic、native 和 Host paths 中 raise `InstanceStateError`。

## 装饰器

IRIS-V1-META-C085: Decorator 语法 `@decorator(args)` 位于 Class、Module、Contract、Method 或 property 声明之前。装饰器转换 declaration candidate metadata，而不是替换 nominal identity、declaration kind、package identity 或 target static spine。

IRIS-V1-META-C086: 多个装饰器按写下的自上而下顺序执行。每个装饰器结果 MUST 保持与输入相同的带类型声明类别。Iris v1 decorators 是受约束的带类型声明转换器，不是不受限制的替换函数、compiler AST 宏或任意编译时执行。

IRIS-V1-META-C087: 声明式装饰器有两个阶段。编译器或链接器解析直接导入的 decorators，并执行纯的确定性 static planning phase，该阶段可以贡献 compiler-visible metadata 或 API transformations。运行时 origin 或 open execution 在声明的 candidate transaction 内应用 runtime transform。

IRIS-V1-META-C088: Decorator static plan MUST NOT 依赖 IO、time、randomness、ambient runtime state、untracked environment、mutable global process state、current package load race 或 unspecified ordering。输入是 decorator identity、arguments、immutable declaration metadata、direct imports、package metadata，以及显式跟踪的 configuration。

IRIS-V1-META-C089: 运行时依赖或条件程序式 decoration 是 dynamic-only。它 MUST NOT 改变已经编译的 static API，且任何 discoverability 都通过 reflection metadata 或 Dynamic dispatch，直到新的 static artifact 被编译并直接导入。

IRIS-V1-META-C090: 装饰器没有内在结构 authority。每个 generated Method、property、storage、Module、metadata 或 body-wrapping operation MUST 声明并通过与等价手写操作相同的 MetaCapabilities、static-spine checks、package permissions、适用处的 ReflectionPolicy checks、`override` rules 和 Contract compatibility。

IRIS-V1-META-C091: Decorators MUST NOT 改变 declaration kind、nominal identity、package identity、immutable superclass bound、declared Contract set、Contract requirements、generic arity、generic invariance、native layout obligation，或任何其他 forbidden static-spine fact。Contract decorators MUST NOT 在显式 Contract source 或 deterministic static API 外隐藏或添加 required obligations。

IRIS-V1-META-C092: 内建 typed generic Decorator Contracts 接收 immutable declaration metadata 加受控 transform context，并返回 same-kind candidate transformations。Runtime transform phases 是 synchronous 且 non-awaiting。Decorator runtime code 的 external side effects 不能被 Iris 回滚，仍由 decorator author 负责。

IRIS-V1-META-C093: Origin construction、declarative open、hot upgrade、rollback reconstruction 和 closed generic materialization MUST 按源顺序把 declaration decorators 重新应用到新候选。Transform 或 validation failure 会中止整个 candidate transaction 且不发布任何内容。

IRIS-V1-META-C094: Reflection 暴露有序 decorator identity、arguments、package 和 source identity、static 和 runtime phase participation，以及经权限过滤的 generated diff metadata。它 MUST NOT 暴露 mutable transform internals 或 candidate mutation handles。

## ReflectionPolicy 与反射视图

IRIS-V1-META-C095: 反射 API 返回按权限过滤的不可变元数据快照，或返回带身份的元数据对象及其惰性不可变视图。它们 MUST NOT 暴露可变内部映射、可变内部表、候选变更句柄、作为通用 API 的 compiler AST，或 eval-string primary mutation。被阻止的访问会返回结构化权限诊断。

IRIS-V1-META-C096: 默认 reflection views 只包含对 caller package 和 visibility context 可见的成员与元数据。特权 `all_*` 或 raw metadata views 需要 ReflectionPolicy inspect scope。结构 mutation 总是使用 open 或 meta transactions，并且 MUST NOT 通过改变返回的 reflection view 发生。

IRIS-V1-META-C097: 固定最小 v1 reflection API 至少包含这些表面:

| 对象 | 必需反射视图成员 |
| --- | --- |
| Class 类 | `name`, `package`, `type`, `static_spine`, `active_revision`, `runtime_superclass`, `mro`, `contracts`, `modules`, `methods`, `properties`, `meta_capabilities`, `open` |
| Module 模块 | `name`, `package`, `type`, `modules`, `methods`, `properties`, `meta_capabilities`, `open` |
| Contract 契约 | `name`, `package`, `type`, `parents`, `requirements`, `meta_capabilities` |
| Method 方法 | `selector`, `owner`, `visibility`, `signature`, `source`, `package`, `bind`, `call` |
| Type 类型 | `kind`, `arguments`, `members`, `subtype?`, `assignable?` |
| Revision | owner、revision number、commit ID、static spine reference、MRO、Module edges、member and property metadata、layout descriptor、package 或 source revision metadata、audit data |

IRIS-V1-META-C098: `respond_to?(selector, include_non_public: false)` 报告 receiver 当前活动普通 MRO 上实际可见的普通 slots。它 MUST NOT 调用或查询 `method_missing`。可见性 denial 按 API contract 指定返回 false 或 raise，但它 MUST NOT 使用 `method_missing` 作为 probe。

IRIS-V1-META-C099: 可选 `respond_to_missing?` 是独立动态 hint，且 MUST NOT 改变 `respond_to?` 真值。Qualified Contract slots 使用 Contract-view-specific `respond_to_contract?`。Ordinary 和 qualified namespaces MUST NOT 合并。

IRIS-V1-META-C100: Raw ivar reflection 使用普通 API `Reflection::Object.list_ivars(obj)`、`Reflection::Object.get_ivar(obj, name)`、`Reflection::Object.set_ivar(obj, name, value)` 和 `Reflection::Object.remove_ivar(obj, name)`。Authorization ambient 于当前执行 package 或 Module context，受 Host-configured ReflectionPolicy 和 manifest grants 控制。

IRIS-V1-META-C101: Iris v1 没有一等 reflection token system。Reflection calls MUST NOT 接受、创建、派生、委派、撤销或传递历史 `ReflectionCapability` token。Method visibility、Dynamic typing、Module private authorization、MetaCapabilities、Host code execution 和 native extension execution 本身不授予 reflection。

IRIS-V1-META-C102: ReflectionPolicy 独立授予 `inspect` 和 `mutate`。`inspect` 覆盖 list 和 get operations。`mutate` 覆盖 set 和 remove operations。Mutate 不暗含 inspect，inspect 也不暗含 mutate。

IRIS-V1-META-C103: ReflectionPolicy scope MAY 覆盖 own package、选定 package IDs 或 Classes，或 Host policy 下运行时范围 trusted tooling。每个 reflection operation 检查 caller package identity、granted operation、granted scope、target identity、请求创建时的 target Class `instance_state`、identity-less restrictions，以及 host 或 native safety rules。

IRIS-V1-META-C104: Reflection grants 不通过 callers、dependency importers、receiver owners、stack frames 或 helper call chains 流动。Authorization 由拥有当前执行 callable 或执行 reflection call 的 call site 的 package 决定。

IRIS-V1-META-C105: Method reflection calls 使用定义精确 Method body 的 package。Closure reflection calls 使用创建 Closure 的代码所在 package，并在 escape 或 cross-package invocation 后保留它。BoundMethod 继承其 Method package。Extension-open Methods 属于 extension package，而不是 target Class origin。

IRIS-V1-META-C106: Origin Class 和 Module body code 在 origin source package 的 ReflectionPolicy 下执行。Declarative extension 或 open body code 在 extension source package 下执行。程序式 `Class#open` 或 `Module#open` 在所提供 block 的 Closure lexical definition package 下运行。

IRIS-V1-META-C107: Native callables 使用 native artifact 的 manifest package identity 进行 reflection。Host built-ins 使用带显式 policy 的专用 host package identity。Manifestless native artifacts 默认没有 reflection，除非 Host 在显式 trusted package identity 下加载它们。

IRIS-V1-META-C108: `ReflectionAccessError` MUST 报告 caller package、operation、target scope 和 denial origin，且不泄露不可访问数据。Reflection denials、capability denials、type failures 和 transaction failures 是普通 Iris failures，且 MUST NOT 暴露 host-only error channels。

IRIS-V1-META-C109: 获授权的 `get_ivar(obj, name)` 对缺失 raw ivar 返回 `nil`，并且不创建任何东西。`set_ivar` 返回 assigned value。创建或重新创建缺失 slot 还需要 target effective `instance_state`。`remove_ivar` 返回被移除的旧值，并在缺失时 raise `InstanceVariableNotFoundError`。`list_ivars` 按未指定顺序返回现有名称的 immutable `Array<Symbol>`。

IRIS-V1-META-C110: Raw ivar reflection names MUST 是 Symbols，其 canonical content 以恰好一个 `@` 开头，后跟有效 ordinary identifier，且没有 selector suffix。Strings 从不隐式转换。Empty names、repeated `@`、selector suffixes 和 source-inexpressible identifiers MUST raise `InvalidInstanceVariableNameError`。

IRIS-V1-META-C111: Raw ivar reflection 不能创建 hidden-only ivars。源 `@name`、Symbol `:@name` 和 reflection name `:@name` 表示同一个 current-receiver slot name。外部源语法 `other.@name` 仍无效，reflection 是唯一外部 raw-ivar access path。

IRIS-V1-META-C112: Raw ivar 的 reflection mutation MUST 仍尊重 identity-less value restrictions、receiver state policy、runtime/native safety，以及拥有该操作的 transaction 或 ordinary assignment boundary。ReflectionPolicy alone 不是 transaction bypass，也不是 `instance_state` bypass。

## 示例与一致性向量

IRIS-V1-META-EX001: Informative example: 显式 Module 源和导入:

```iris
import com.example.core::Text
from com.example.extra::TraceExtension import Traceable as TraceableExt

export module App {
  fun run() -> Nil {
    log("ready")
  }
}

export open class Text::Buffer {
  public fun traced() -> String { inspect() }
}
```

IRIS-V1-META-EX002: Informative example: 可执行 Class 体与仅动态可见的条件成员:

```iris
class Tool meta deny superclass, native {
  let enabled = load_config().enabled?

  if enabled {
    self.define_method(:debug) { |self: Tool| -> String; "debug" }
  }

  public fun run() -> String { "run" }
}
```

IRIS-V1-META-EX003: Informative example: 程序式 open 事务:

```iris
Tool.open() { |target: Class| -> Nil
  target.define_method(:status) { |self: Tool| -> Symbol; :ready }
}
```

IRIS-V1-META-EX004: Informative example: 带显式权限的 raw ivar 反射:

```iris
let names = Reflection.list_ivars(object)
let old = Reflection.get_ivar(object, :@cache)
Reflection.set_ivar(object, :@cache, compute())
```

IRIS-V1-META-EX005: Informative example: 装饰器顺序:

```iris
@logged(level: :info)
@memoized()
public fun total() -> Integer {
  compute_total()
}
```

IRIS-V1-META-C113: 一致性章节 MUST 保留下面 audit-exact 表中的每个 concrete vector ID，或把它映射到具有相同 observable outcome 的 machine-readable record。早前 summary-only vector rows 已退休，因为它们不满足六列一致性定义 schema。

## 可追踪性说明

IRIS-V1-META-C114: 本章拥有或锚定 D-221 到 D-232、D-243 到 D-274、D-284 到 D-336、D-474 到 D-486、D-511 到 D-514，以及 revised D-294 到 D-300 中关于 package、Module、metaprogramming、transaction、capability、decorator、reflection 和 static-extension 的部分。D-207 到 D-220 与 D-233 到 D-242 中的类型特定语义仍由 [05-types-contracts-generics.md](05-types-contracts-generics.md) 拥有。Runtime dispatch、ClassRevision identity、raw ivar source behavior、MRO lookup、visibility、construction 和 retained Method behavior 仍由 [03-runtime-object-model.md](03-runtime-object-model.md) 拥有。Revision-event delivery、GapEvent 和 audit-history iteration cleanup 仍由 [07-async-resources-diagnostics.md](07-async-resources-diagnostics.md) 拥有。

IRIS-V1-META-C115: 本章拥有或锚定的 decision IDs 是 `D-221`, `D-222`, `D-223`, `D-224`, `D-225`, `D-226`, `D-227`, `D-228`, `D-229`, `D-230`, `D-231`, `D-232`, `D-243`, `D-244`, `D-245`, `D-246`, `D-247`, `D-248`, `D-249`, `D-250`, `D-251`, `D-252`, `D-253`, `D-254`, `D-255`, `D-256`, `D-257`, `D-258`, `D-259`, `D-260`, `D-261`, `D-262`, `D-263`, `D-264`, `D-265`, `D-266`, `D-267`, `D-268`, `D-269`, `D-270`, `D-271`, `D-272`, `D-273`, `D-274`, `D-284`, `D-285`, `D-286`, `D-287`, `D-288`, `D-289`, `D-290`, `D-291`, `D-292`, `D-293`, `D-294`, `D-295`, `D-296`, `D-297`, `D-298`, `D-299`, `D-300`, `D-301`, `D-302`, `D-303`, `D-304`, `D-305`, `D-306`, `D-307`, `D-308`, `D-309`, `D-310`, `D-311`, `D-312`, `D-313`, `D-314`, `D-315`, `D-316`, `D-317`, `D-318`, `D-319`, `D-320`, `D-321`, `D-328`, `D-329`, `D-330`, `D-331`, `D-332`, `D-333`, `D-334`, `D-335`, `D-336`, `D-474`, `D-475`, `D-476`, `D-477`, `D-478`, `D-479`, `D-480`, `D-481`, `D-482`, `D-483`, `D-484`, `D-485`, `D-486`, `D-511`, `D-512`, `D-513`, and `D-514`。

IRIS-V1-META-C116: 引用但不由本章拥有的 decision IDs 是 `D-173`, `D-174`, `D-175`, `D-176`, `D-177`, `D-178`, `D-179`, `D-180`, `D-181`, `D-207`, `D-208`, `D-209`, `D-210`, `D-211`, `D-212`, `D-213`, `D-214`, `D-215`, `D-216`, `D-217`, `D-218`, `D-219`, `D-220`, `D-233`, `D-234`, `D-235`, `D-236`, `D-237`, `D-238`, `D-239`, `D-240`, `D-241`, `D-242`, `D-322`, `D-323`, `D-324`, `D-325`, `D-326`, `D-327`, `D-470`, `D-471`, `D-472`, `D-487`, `D-488`, `D-489`, and `D-490`。

IRIS-V1-META-N001: Informative note: IRIS-V1-META-C049 中的 import-site replacement authorization 保留来自 D-230 的冻结语义要求，同时避免在本章加入新的 parser production。Grammar 章节拥有 source token 和 production updates。

## Audit-Exact 一致性向量

IRIS-V1-META-C117: 以下规范性 audit-exact vectors 各自命名具体 package、source、manifest 或 transaction fixture、category、backend applicability、精确 observable result，以及直接覆盖的 decision metadata。

IRIS-V1-META-C118: Reflective Method surface 是 `Reflection::Class.method(target, selector)`，它返回 unbound Method reflective object，缺失时返回 `nil`；`Reflection::Class.invoke(method, receiver, args)`；`Reflection::Class.remove_module(target, module)`；以及 Module-side counterpart `Reflection::Module.method(target, selector)` 和 `Reflection::Module.invoke(method, receiver, args)`。Entry validation 与 `MethodBindingError` 由 `IRIS-V1-RUNTIME-C015` 和 `D-104` 拥有；retained Method 的 `super` 与 `InvalidSuperError` 由 `IRIS-V1-RUNTIME-C014` 和 `D-103` 拥有；package identity 由 `IRIS-V1-META-C105` 拥有；缺失 lookup 按 `IRIS-V1-META-C109` 返回 `nil`；`method` 在 `IRIS-V1-META-C102` 下需要 `inspect`，`invoke` 和 `remove_module` 需要 `mutate`；`remove_module` transaction semantics 由 `IRIS-V1-META-C060` 和 `IRIS-V1-META-C078` 拥有，其 capability gating 由 `IRIS-V1-META-C055` 的 `modules` 拥有，token passing 仍被 `IRIS-V1-META-C101` 禁止。

IRIS-V1-META-C119: Reflection operations 在对应的 `Reflection::*` sub-Modules 中只实现一次，`Class` 和 `Module` 通过 mixin 获得对应成员，因此 `A.remove_module(M)` 与 `Reflection::Class.remove_module(A, M)` 是同一实现的两个入口。该 weaving 以 `IRIS-V1-RUNTIME-C043` 和 `IRIS-V1-RUNTIME-C044` 为基础。不需要 grammar change：既有 `qualified_type_name` 和 `mixin_entry` productions 已经能推导 `mixin Reflection::Class`。

IRIS-V1-META-C120: 运行时超类反射的拼写是 `Reflection::Class.set_superclass(target, new_superclass)` 和 `Reflection::Class.ancestors(target)`。提交后的可观察性和永久声明边界由 `D-262` 拥有；受保护的内建 Class 按 `IRIS-V1-RUNTIME-C150` 抛出 meta-operation exception、中止整个 transaction group 且不发布 candidate；capability gating 由 `IRIS-V1-META-C073` 的 `superclass` 拥有；结果 lookup order 由 `IRIS-V1-RUNTIME-C046` 拥有；`ancestors` 在 `IRIS-V1-META-C102` 下需要 `inspect`，`set_superclass` 需要 `mutate`；以及 `A.set_superclass(B)` 与 `Reflection::Class.set_superclass(A, B)` 两个入口按 `IRIS-V1-META-C119` 是同一实现。`set_superclass` 返回 `nil` 是与 `IRIS-V1-COLLECTIONS-C024` scalar-write 规则一致的作者选择，而不是单独的语义强制要求。

IRIS-V1-META-C121：v1.25 勘误确定了读取体局部的方法声明如何被报告。IRIS-V1-META-C026 使此类方法不闭包捕获该局部，而 IRIS-V1-CONTROL-C011 使未解析的裸名抛出**或**诊断 `NameError`；两个分支均合规。因此，合规实现**可以**接受该声明并在方法运行时抛出 `NameError`，且**不得**被要求静态拒绝该声明。所属 Class 正常发布，因为 IRIS-V1-META-C022 仅对**失败**的候选保留发布，而不捕获是 C026 规定的结果而非失败。本条就地取代 IRIS-V1-META-V343 中「定义被拒绝且不提交修订」的预期。

IRIS-V1-META-C122：v1.26 勘误命名了 IRIS-V1-META-C092 所要求的装饰器 Contract。它们是 `ClassDecorator`、`ModuleDecorator`、`ContractDecorator`、`MethodDecorator` 与 `PropertyDecorator`，与 IRIS-V1-META-C085 固定的装饰目标一一对应。每一个都是普通的具名 Contract：装饰器是依据 IRIS-V1-TYPES-C044 以 `for` 声明遵从其中之一的 Class，其遵从性与任何其他已声明遵从一样被检查。每个 Contract 要求恰好两个成员，合规装饰器**必须**声明二者。`plan(declaration, arguments) -> Plan` 是 IRIS-V1-META-C087 的纯静态规划阶段，受 C088 约束，其中 `declaration` 是 IRIS-V1-META-C097 固定的该目标的不可变反射视图。`transform(declaration, arguments) -> Transformation` 是 C087 的运行时阶段，在该声明的候选事务内应用；它按 C092 的要求**返回**同类候选变换，而非改变候选句柄，从而保持 IRIS-V1-META-C095 对暴露候选变更句柄的禁止不变。装饰器即使只参与一个阶段，仍需声明两个成员；未参与的阶段返回空的 Plan 或 Transformation。返回的变换其种类与目标不同，违反 IRIS-V1-META-C086 与 C091，报告为 `IRIS-DECORATOR-KIND`。

IRIS-V1-META-C123：v1.26 勘误为 IRIS-V1-META-C097 反射表增加 `property` 行，因为 IRIS-V1-META-C085 允许属性作为装饰目标，而 C092 使目标的不可变视图成为装饰器的声明元数据。属性视图暴露 `name`、`owner`、`visibility`、`type`、`package` 与 `slot`。本条通过增加一行就地取代 C097 表；现有各行均不改变。

IRIS-V1-META-C124：v1.27 勘误修正 IRIS-V1-META-C122 所述的阶段签名。IRIS-V1-META-C092 与 D-514 给予装饰器 Contract「不可变声明元数据**加上**受控转换上下文」，这是两个不同的输入；C122 仅命名了 `declaration` 与 `arguments`，而 `arguments` 是 IRIS-V1-META-C085 中 `@decorator(args)` 的应用点实参列表，**并非**该上下文。受控转换上下文仅提供给**运行时**阶段，因为 IRIS-V1-META-C088 使静态规划阶段保持纯粹与确定，候选事务的句柄在该阶段没有意义。因此成员签名为 `plan(declaration, arguments) -> Plan` 与 `transform(declaration, arguments, context) -> Transformation`，其中 `declaration` 是 IRIS-V1-META-C097 与 C123 固定的该目标不可变反射视图，`arguments` 是应用点实参列表，`context` 是 C092 所称的受控转换上下文。本条就地取代 C122 中的 `transform` 签名；C122 的其余陈述均不变。

IRIS-V1-META-C125：v1.28 勘误确定了 IRIS-V1-META-C122 所命名的 `Plan` 与 `Transformation` 类型的最小成员。`Plan` 承载 IRIS-V1-META-C087 所述的编译器可见元数据；其最小表面是 `Plan.empty`，即不贡献静态元数据的装饰器所返回的计划。若静态计划读取了 IRIS-V1-META-C088 所定白名单之外的输入，则在运行时阶段执行前即被拒绝，诊断为 `IRIS-DECORATOR-NONDETERMINISTIC`，阶段为 static，且不发生任何运行时变换或目标发布。`Transformation` 承载同类候选变换；其最小表面是 `Transformation.empty`，即不做改变的装饰器所返回的变换；`Transformation.add_method(selector, body)`，它向目标候选暂存一个方法，并要求与 IRIS-V1-META-C090 对手写声明所要求的相同的 `method_set` 能力；以及 `kind`，即该变换所针对的声明类别。若 `Transformation` 的 `kind` 与被装饰目标的类别不同，则以 `IRIS-DECORATOR-KIND` 拒绝，阶段为 static，且目标不保留任何候选或修订，这是 IRIS-V1-META-C086 与 C091 已有的要求。对已应用装饰器的反射依据 IRIS-V1-META-C094 与 C095 将生成的差异暴露为不可变的、经权限过滤的视图，绝不暴露为可变的变换句柄。本表面是**最小**的：IRIS-V1-META-C090 还列出了生成属性、存储、Module、元数据与方法体包裹等操作，后续修订**可以**在不改变此处所定成员的前提下增补。

| Vector ID | Category | 适用性 | 来源/输入 | 预期可观察结果 | Decisions |
| --------------------- | ------------ | ---------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| `IRIS-V1-META-V340` | positive | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v340/iris.toml` 声明 `package_id="org.iris.v340"`、`api_major=1`；`src/main.ir` 声明 `module A`、`module B` 和 `class Host mixin A, B`；其 open transaction 移除并 include `A`。 | Module edge list 从 `[A, B]` 变为 `[B, A]`；Host MRO 把 `A` 放在 `B` 前；提交一个 Host revision。 | `D-221`, `D-321` |
| `IRIS-V1-META-V341` | negative | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v341/iris.toml` 声明 `package_id="org.iris.v341"`、`api_major=1`；`src/main.ir` 声明 `class Box { self.define_method(:ok) { 1 }; raise :stop }`。 | Runtime error `:stop` 在 origin transaction 期间发生；`Box` 不发布，且不提交 `ok` Method 或 revision。 | `D-222` |
| `IRIS-V1-META-V342` | positive | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v342/iris.toml` 声明 `package_id="org.iris.v342"`、`api_major=1`；`src/main.ir` 声明 `module M { let value = 7; self.define_method(:answer) { value }; fun value() -> Integer { 8 } }`。 | Candidate metadata 包含 `answer`；`M.main.answer()` 返回 integer `8`；declared Method capture 排除 transaction local `value`。 | `D-223`, `D-224` |
| `IRIS-V1-META-V343` | positive | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v343/iris.toml` 声明 `package_id="org.iris.v343"`、`api_major=1`；`src/main.ir` 声明 `class Box { let local = 1; fun value() -> Integer { local } }`。 | 如果该定义捕获 `local`，则被拒绝；不提交 Box revision。 | `D-224` |
| `IRIS-V1-META-V344` | positive | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v344/iris.toml` 声明 `package_id="org.iris.v344"`、`api_major=1`；`src/main.ir` 存储一个捕获 origin local `7` 的 closure。 | Escaped closure 返回 integer `7`；Box reflection 中没有名为 `local` 的 property、class state 或 storage slot。 | `D-225` |
| `IRIS-V1-META-V345` | positive | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v345/iris.toml` 声明 `package_id="org.iris.v345"`、`api_major=1`；`class Box` 的 true branch 调用 `self.define_method(:extra) { :extra }`。 | Runtime call 返回 `:extra`；reflection 把 `extra` 标记为 dynamic-only，consumer static compilation 拒绝它。 | `D-226` |
| `IRIS-V1-META-V346` | positive | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v346/iris.toml` 声明 `package_id="org.iris.v346"`、`api_major=1`；extension package 导出 `open class Base { fun tag() { :tag } }`；consumer 直接导入它。 | `Base.new().tag()` 返回 `:tag`；移除直接导入会产生 static member diagnostic `IRIS-STATIC-MEMBER-NOT-FOUND`。 | `D-227` |
| `IRIS-V1-META-V347` | positive | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v347/iris.toml` 声明 `package_id="org.iris.v347"`、`api_major=1`；两个直接导入的 extension Modules 在显式 replacement authorization 下兼容替换 `Base#tag`。 | 后导入的实现返回 `:second`；不兼容 signature 变体在 link 时失败，且不形成 overload set。 | `D-228` |
| `IRIS-V1-META-V348` | positive | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v348/iris.toml` 声明 `package_id="org.iris.v348"`、`api_major=1`；consumer source 先导入 `First` 再导入 `Second`，二者都是已授权的 `Base#tag` extensions。 | `Base.new().tag()` 返回 `:second`；反转两个 import 行则返回 `:first`。 | `D-229` |
| `IRIS-V1-META-V349` | diagnostic | compiler required；interpreter 不适用；JIT 不适用；native 不适用 | `fixtures/meta/v349/iris.toml` 声明 `package_id="org.iris.v349"`、`api_major=1`；consumer 导入两个兼容的 `Base#tag` extensions，但没有 replacement marker。 | Diagnostic `IRIS-IMPORT-REPLACEMENT-AUTHORIZATION` 的 severity 为 error，phase 为 link。 | `D-230` |
| `IRIS-V1-META-V350` | diagnostic | compiler required；interpreter 不适用；JIT 不适用；native 不适用 | `fixtures/meta/v350/iris.toml` 声明 `package_id="org.iris.v350"`、`api_major=1`；`open class Box { fun tag() { :new } }` 替换已有 `tag`，但没有 `override`。 | Diagnostic `IRIS-MEMBER-OVERRIDE-REQUIRED` 的 severity 为 error；Box active revision 保持 `1`。 | `D-231` |
| `IRIS-V1-META-V351` | diagnostic | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v351/iris.toml` 声明 `package_id="org.iris.v351"`、`api_major=1`；`Child` 替换 inherited 和 Module-provided `tag`，然后尝试在没有 `impl` 的情况下替换 Contract slot。 | 缺少 `override` 或 `impl` 会产生 `IRIS-MEMBER-OVERRIDE-REQUIRED` 或 `IRIS-CONTRACT-IMPL-REQUIRED`；不提交 candidate。 | `D-232` |
| `IRIS-V1-META-V352` | positive | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v352/iris.toml` 声明 `package_id="org.iris.v352"`、`api_major=1`、version `1.2.0`；companion manifests 在 API major 1 使用 version `1.2.1`，在 API major 2 使用 `2.0.0`；每个 source 都声明 `class Token`。 | 1.2.0 和 1.2.1 的 Token Type identities 相等；API-major-2 identity 不相等。选择两个 API-major-1 implementations 的 lock 以 status `PackageVersionUnificationError` 使 package link 失败；不执行 Module body，也不创建 hidden identity。 | `D-243`, `D-244`, `D-245` |
| `IRIS-V1-META-V353` | differential | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v353/iris.toml` 把 `org.iris.v353` 从 1.0.0 升级到 1.0.1；`src/main.ir` 把 `A#value` 和 `B#value` 从 `:old` 改为 `:new`；初始 revisions 为 A=1、B=1，commit 为 40；upgrade 期间保留一个 entered old frame 和 retained Method。 | Safepoint publication 原子产生 A=2、B=2、commit 41；没有 observer 看到混合值，entered frame 返回 `:old`，post-resume sends 返回 `:new`。Heavy revision-1 state 在被保留期间继续存在，release 后可回收，而其 audit row 仍可查询。 | `D-246`, `D-247`, `D-250`, `D-267`, `D-269` |
| `IRIS-V1-META-V354` | negative | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v354/iris.toml` upgrade hook 写入精确 external log line `migration-start`，把 candidate counter 设为 9，然后 raise `MigrationStop`；active version 是 1.0.0，counter 为 4，owner revision 为 1，commit 为 40。 | Error `MigrationStop` 传播；version 1.0.0、counter 4、revision 1 和 commit 40 保持 active；不发布 candidate diff，而 external log 恰好包含 `migration-start`。 | `D-248`, `D-249` |
| `IRIS-V1-META-V355` | differential | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v355/iris.toml` 以 A=1、B=1、commit 40 开始；外层 open 暂存 `A#a2`，嵌套 open 暂存 `B#b2`，竞争事务在 validation 前提交 A；block 递增 external run counter；companion open 包含 `await ready()`。 | Candidate reflection 列出 `a2` 和 `b2`，ordinary sends 仍使用 revision 1，且不存在 candidate-instance invocation operation。Conflict raises `MetaTransactionConflictError`，使组保持 unpublished，不消耗 commit ID，且 run counter 恰好为 1。没有 competitor 时，一个 safepoint 在 commit 41 发布 A=2 和 B=2，且无 mixed observation。Await 发出 static transaction-suspension diagnostic，且不启动 candidate。 | `D-251`, `D-252`, `D-253`, `D-254`, `D-255`, `D-256`, `D-257`, `D-268` |
| `IRIS-V1-META-V356` | negative | compiler required；interpreter required；JIT required；native optional | `fixtures/meta/v356/iris.toml` 打开 `Account`、添加 `fee`，然后尝试 native-layout-unsafe storage；已有 Account 缺少 `fee` 预期的 state。 | Method-only candidate 可以发布，无需 constructor replay 或 instance scanning；unsafe candidate 因 native/layout validation 失败且 diff 为空。Iris 不迁移 instance state、不修复 resources，也不恢复 business invariants。 | `D-263` |
| `IRIS-V1-META-V357` | diagnostic | compiler required；interpreter required；JIT required；native optional | `fixtures/meta/v357/artifact.json` 具有精确 BLAKE3-256 `6e7c0d24c82a8dc03b320c790a7c8c2d4f2f9b77b9635c315d21ea34e92f4601`；变体只改变 locator、改变 source bytes、使 digest 不匹配，或省略当前 required Method `status`；active owner revision 为 2，commit 为 41。 | Locator-only 变体保留 digest；source change 改变完整 32-byte digest。有效 rollback 在 commit 42 发布 revision 3，绝不重新激活 revision 1。Mismatch raises `RevisionArtifactUnavailableError`；缺少 `status` 导致 static-spine validation 失败；两种失败都让 revision 2 保持 active，且不消耗 commit ID。 | `D-270`, `D-271`, `D-272`, `D-273`, `D-274` |
| `IRIS-V1-META-V358` | positive | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v358/iris.toml` 声明没有 `extends` 的 Contract Plain、没有 `mixin` 或 `where Self` 的 Module Empty，并在 load 后反射二者。 | `Plain.parents` 是 `[]`，`Empty.modules` 是 `[]`，normalized `Empty::Self` upper bound 是 `Object`；不出现隐式 Contract parent、Module edge 或 NonNil bound。 | `D-284` |
| `IRIS-V1-META-V359` | differential | compiler required；interpreter required；JIT required；native optional | `fixtures/meta/v359/iris.toml` 把 Module Marker 组合进 Nil，并打开 Bool 和 Integer 以添加兼容 Method `tag`；随后三者都通过 source、reflection、Dynamic、native 和 Host paths 尝试 `superclass=`。 | Nil 保持 singleton identity 并获得 Marker behavior；Bool 和 Integer 保留 primitive identity/immutability，且 `tag()` 返回 `:ok`。每次 superclass attempt 都 raises `MetaCapabilityError`，每个 transaction diff 都为空，所有路径报告相同 protected operation 和 policy origin。 | `D-285`, `D-286`, `D-287`, `D-288` |
| `IRIS-V1-META-V360` | diagnostic | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v360/iris.toml` 定义 Base `meta deny method_set`、RuntimeBase `meta deny property_set`、Module NoShape `meta deny shape` 和 Contract Fixed `meta deny class_state_set`；subclass Target include NoShape 并声明 `for Fixed`。Companion source 在 body 中放置 `meta deny future_power` 并尝试放宽 reflected policy。 | Target effective deny origins 恰好是 Base/method_set、RuntimeBase/property_set、NoShape/shape 和 Fixed/class_state_set。Policy view 不可变；subclass/open 不能恢复 deny；移除 NoShape 只移除 shape。Body use 和 unknown name 发出 static diagnostics，且后续 header clause 前的 `meta` 被拒绝。 | `D-289`, `D-290`, `D-294`, `D-295`, `D-296`, `D-297`, `D-298` |
| `IRIS-V1-META-V361` | diagnostic | compiler required；interpreter required；JIT required；native optional | `fixtures/meta/v361/iris.toml` 先用全部 requirements 尝试每个 C081 operation，再在一个 required capability 被 denied 时重复；不同 owners deny `method_set`、`method_body`、`property_set`、`property_body`、`class_state_set`、`class_state_write`、`modules`、`shape` 或 `instance_state`；一个 object 初始拥有 undeclared `@old`。 | Allowed lane 提交精确 requested diff。每个 denied lane 都 raises `MetaCapabilityError`，命名 target、operation、missing capability 和 origin，并发布 empty diff。Denying shape 仍允许 absent `@new = 1`；denying instance_state 拒绝创建，允许移除 `@old` 并返回其值，且拒绝重新创建。全部 12 个 vocabulary names 可解析，`future_power` 发出 unknown-capability diagnostic。 | `D-291`, `D-292`, `D-293`, `D-299`, `D-300`, `D-301`, `D-302`, `D-303`, `D-304`, `D-305`, `D-306` |
| `IRIS-V1-META-V362` | diagnostic | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v362/iris.toml` 只向 app package 授予对 `Box` 的 inspect/mutate；未获授权的 helper、escaped Closure、BoundMethod、extension-open Method、origin/open bodies 和 programmatic-open Closure 执行 raw-ivar reflection；inputs 包括 `:@missing`、`:@x`、`:"@@x"` 和 `:ready?`。 | Missing get 返回 `nil`；authorized set 返回 assigned value；remove 返回 old value，absent remove raises `InstanceVariableNotFoundError`；invalid names raise `InvalidInstanceVariableNameError`。未授权 package contexts raise `ReflectionAccessError`；callable checks 在 escape 后使用 lexical source package；不存在 token-accepting overload。 | `D-328`, `D-329`, `D-331`, `D-332`, `D-333`, `D-335`, `D-336` |
| `IRIS-V1-META-V363` | negative | compiler required；interpreter required；JIT required；native required | `fixtures/meta/v363/iris.toml` 加载属于未获授权 package `org.iris.native.helper` 的 native artifact；其 callable 调用 `Reflection.list_ivars`。Companion manifestless artifact 在没有 trusted Host identity 的情况下加载。 | 两次调用都 raise `ReflectionAccessError`，caller package 等于 artifact identity 或 untrusted-manifestless identity；caller stack 和 Host process privilege 不借出 inspect scope。 | `D-334` |
| `IRIS-V1-META-V415` | positive | compiler required；interpreter required；JIT 不适用；native 不适用 | `fixtures/meta/v415/iris.toml`: `package_id="org.iris.v415"`、`api_major=1`；`src/main.ir`: `module First { } module Second { }`；transaction fixture 先加载 `First` 再加载 `Second`。 | Package load status 是 success；initialized Module order 恰好是 `[First, Second]`；不存在 top-level executable statement。 | `D-474` |
| `IRIS-V1-META-V416` | positive | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v416/iris.toml`: `package_id="org.iris.v416"`、`api_major=1`；`src/a.ir`: `module P::M { fun one() -> Integer { 1 } }`；`src/b.ir`: `open module P::M { fun two() -> Integer { 2 } }`；transaction fixture 加载两个文件。 | Package load status 是 success；`P::M.main.one()` 是 integer `1`，`P::M.main.two()` 是 integer `2`，reflection 报告一个 origin 加一个 open revision。 | `D-475` |
| `IRIS-V1-META-V417` | diagnostic | compiler required；interpreter 不适用；JIT 不适用；native 不适用 | `fixtures/meta/v417/iris.toml`: `package_id="org.iris.v417"`、`api_major=1`；`src/main.ir`: `import org.dep::Core as C; from org.dep::Names import One, Two as T; import org.dep::*`。 | Diagnostic `IRIS-IMPORT-WILDCARD` 的 severity 为 error，phase 为 static；不启动 package link transaction。 | `D-476` |
| `IRIS-V1-META-V418` | positive | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v418/iris.toml`: `package_id="org.iris.v418"`、`api_major=1`；package `org.dep` 导出 `Core`；facade source 是 `export from org.dep::Core import Name`；extension source 是 `export open class Core::Thing { fun tag() -> Symbol { :tag } }`；consumer 导入 facade 但不导入 extension。 | `Name` 通过 facade 解析；consumer static type 没有 `tag`；直接导入 extension source 会让 `tag()` 返回 symbol `:tag`。 | `D-477` |
| `IRIS-V1-META-V419` | negative | compiler required；interpreter required；JIT 不适用；native 不适用 | `fixtures/meta/v419/iris.toml`: `package_id="org.iris.v419"`、`api_major=1`；`src/a.ir` imports `B`；`src/b.ir` imports `A`；transaction fixture 以 `A` 为 entry 链接。 | Error kind 为 `ModuleInitializationCycleError`，phase 为 link；package status 是 failed；`A` 和 `B` 都没有 active revision。 | `D-478` |
| `IRIS-V1-META-V420` | positive | compiler required；interpreter required；JIT 不适用；native 不适用 | `fixtures/meta/v420/iris.toml`: `package_id="org.iris.v420"`、`api_major=1`、`version="1.2.3"`、dependency `org.dep="^2.1.0"`；`iris.lock` 选择带 digest `b3:dep214` 的 `org.dep 2.1.4`；`src/main.ir` 是 `module Main { }`；transaction fixture 加载 lock result。 | Package status 是 success；reflection 报告 identity `(org.iris.v420, 1)` 和 selected dependency `(org.dep, 2, 2.1.4, b3:dep214)`；不发生 resolver fetch。 | `D-479` |
| `IRIS-V1-META-V421` | diagnostic | compiler required；interpreter required；JIT 不适用；native 不适用 | `fixtures/meta/v421/iris.toml` 声明 `package_id="org.iris.v421"`、`api_major=1`、scoped to `fixtures/data/` 的 required `filesystem.read`，以及 scoped to `org.target::Box` 的 optional `reflection.inspect`；`src/main.ir` 列出 Box ivars。第一个 Host grant fixture 只授予 inspect；第二个授予两个 requests；self-authorization companion 调用 `Package.grant("network.connect")`。 | 第一次 package-load status 是 `{kind: PermissionDenied, success: false, permission: filesystem.read}`，发生在 Main 执行前。第二次 status 是 success，且 inspect 只对 Box 生效。Self-authorization raises `ReflectionAccessError` 或 Host authorization rejection，不创建 grant，mutate call 仍被 denied。 | `D-330`, `D-480` |
| `IRIS-V1-META-V422` | positive | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v422/iris.toml`: `package_id="org.iris.v422"`、`api_major=1`、optional `package.load` grant；`src/main.ir` 是 `module Main { let m = Package.load("org.dynamic", "1.0.0"); m }`；transaction fixture 预链接 `org.dynamic::Plugin`。 | Return type 是 `Dynamic<Module>`；`Plugin` 不被引入 Main 的 compiled lexical namespace；static `import "org.dynamic"` 以 `IRIS-IMPORT-NONSTATIC` 被拒绝。 | `D-481` |
| `IRIS-V1-META-V423` | positive | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v423/iris.toml`: `package_id="org.iris.v423"`、`api_major=1`、对 `org.target::Box` 的 scoped `reflection.inspect` grant；`src/main.ir` 调用 `Box.reflect.methods`；transaction fixture 创建 public `show` 和 private `hide`。 | View 不可变；method selectors 等于 `[:show]`；尝试 `view.append(:hide)` raises `NoSuchMethodError`；target revision 不改变。 | `D-482` |
| `IRIS-V1-META-V424` | positive | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v424/iris.toml`: `package_id="org.iris.v424"`、`api_major=1`；`src/main.ir` 反射 `Box`、`Tools`、`Printable`、`Box.method(:show)`、`Box.type` 和 `Box.active_revision`；transaction fixture 打开 `Box` 一次。 | Views 分别暴露 C097 要求的 Class、Module、Contract、Method、Type 和 Revision members；`active_revision.revision` 是 integer `2`；所有 views 都是 immutable snapshots。 | `D-483` |
| `IRIS-V1-META-V425` | positive | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v425/iris.toml`: `package_id="org.iris.v425"`、`api_major=1`；`src/main.ir` 定义 `class Box { fun method_missing(s) { raise :called } }` 和带 `name` 的 Contract `Named`；transaction fixture 求值 `Box.new().respond_to?(:ghost)` 和 `Named.view(Box).respond_to_contract?(:name)`。 | Values 是 `false` 和 `true`；不发生 `:called` error；当只有 qualified Contract slot 存在时，ordinary `respond_to?(:name)` 仍为 false。 | `D-484` |
| `IRIS-V1-META-V426` | positive | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v426/iris.toml`: `package_id="org.iris.v426"`、`api_major=1`；`src/main.ir` 定义 `class Box { fun old() -> Symbol { :old } }`；transaction fixture 使用 `Box.open` 定义返回 `:new` 的 Method `new`。 | Commit status 是 success；Box revision 从 `1` 变为 `2`；`Box.new().new()` 返回 `:new`；audit diff 列出 added selector `new`。 | `D-485` |
| `IRIS-V1-META-V427` | positive | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v427/iris.toml`: `package_id="org.iris.v427"`、`api_major=1`；`src/main.ir` 在 `class Box` 中条件调用 `self.define_method(:extra) { :extra }`；transaction fixture 中条件为 true，并单独编译 consumer `Box#extra`。 | Runtime `Box.new().extra()` 返回 `:extra`；reflection 把 `extra` 标记为 dynamic-only 并带其 commit ID；consumer compilation 发出 `IRIS-STATIC-MEMBER-NOT-FOUND`，直到新的 exported static artifact 被直接导入。 | `D-486` |
| `IRIS-V1-META-V428` | diagnostic | compiler required；interpreter required；JIT 不适用；native 不适用 | `fixtures/meta/v428/iris.toml`: `package_id="org.iris.v428"`、`api_major=1`；`src/main.ir` 把 `@as_module()` 应用于 `class Box`；decorator fixture 返回 Module candidate。 | Diagnostic `IRIS-DECORATOR-KIND` 的 severity 为 error，phase 为 static；`Box` 不保留 candidate 或 revision。 | `D-511` |
| `IRIS-V1-META-V429` | diagnostic | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v429/iris.toml`: `package_id="org.iris.v429"`、`api_major=1`；导入的 `@stamp()` decorator static plan 读取 `Clock.now()`；`src/main.ir` 把 `@stamp()` 应用于 `class Box`。 | Diagnostic `IRIS-DECORATOR-NONDETERMINISTIC` 的 severity 为 error，phase 为 static；不发生 runtime transform 或 Box publication。 | `D-512` |
| `IRIS-V1-META-V430` | negative | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v430/iris.toml`: `package_id="org.iris.v430"`、`api_major=1`；`class Box meta deny method_set`；导入的 `@adds_trace()` 生成 Method `trace`；transaction fixture 应用该 decorator。 | Error kind 为 `MetaCapabilityError`，phase 为 candidate validation；target 是 `Box`，operation 是 `method_set`；Box revision 和 method list 不改变。 | `D-513` |
| `IRIS-V1-META-V431` | positive | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v431/iris.toml`: `package_id="org.iris.v431"`、`api_major=1`；`@first()` 然后 `@second()` 装饰 `class Box`；Decorator Contract fixture 接收 immutable declaration metadata 和 runtime transform context；transaction fixture 把 `1.0.0` upgrade 到 `1.0.1`，然后从 retained exact artifact rollback。 | Origin、upgrade 和 rollback 的 static 与 runtime decorator order 都是 `[first, second]`；`Box.reflect.decorators` 报告 ordered identities 和 filtered generated diff；transform 中尝试 `await` raises `MetaTransactionError` 且不发布任何内容。 | `D-514` |
| `IRIS-V1-META-V432` | differential | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v432/iris.toml` 在 revision 1 active 时构造 Box；`initialize` 记录 revision 1，一个 concurrent open 发布兼容 revision 2，然后对完成的 instance 发送 `value`；companion 在 open 前后检查 Type identity，并在 declared bound 内改变 runtime superclass。 | Allocation 和 initialize 都使用 revision 1；construction 既不重试也不协调；完成的 instance 后续 send 使用 revision 2。Nominal Box Type 和 Contract identity 在 open 前后保持相等，而 `is`、`subtype?` 和 ancestry 反映新的 active runtime superclass。 | `D-258`, `D-259`, `D-260`, `D-261`, `D-262` |
| `IRIS-V1-META-V433` | diagnostic | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v433/iris.toml` 向一个 tracked instance 显式发送普通 dynamic message `migrate_revision(old, new)`；其 Method 接收 immutable revision views。Companions 移除该 Method，并尝试 mutate 或 reactivate old view。 | Runtime commit 自动枚举或迁移零个 instances。显式 send 返回 symbol `:migrated`；缺失 Method 遵循 ordinary missing-selector behavior；revision mutation/reactivation raises immutable-view error，且不改变 active revision。 | `D-264`, `D-265`, `D-266` |
| `IRIS-V1-META-V434` | differential | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v434/iris.toml` 创建两个 Box receivers；每个都在 undeclared `@x` 中先存 Integer 后存 String；Class object 存储自己的 `@x`，hierarchy class state 存储 `@@x`；在第一个 receiver 上创建的 escaped Closure 读写 `@x`；external source 尝试 `other.@x`；private Method `secret` 从 owner code、subclass、external、Dynamic 和 reflection paths 调用。 | Raw reads 的 static type 是 `Dynamic<Object>`；两个 receivers、Class-object `@x` 和 `@@x` 保持不同。Escaped Closure 仍绑定第一个 receiver 并返回其 String。`other.@x` 发出 parse/static diagnostic。只有 declaring-Class lexical call 返回 `:secret`；所有其他 private calls raise visibility error。Interpreter 和 JIT observations 相等。 | `D-307`, `D-308`, `D-309`, `D-310`, `D-311`, `D-312`, `D-313` |
| `IRIS-V1-META-V435` | differential | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v435/iris.toml` 把 Modules A 再 B 组合进 Host；B 有显式 private-edge authorization，两个 Modules 都访问 receiver `@x`，并都定义 `rank`；嵌套 Module C 也 include A。Transactions 移除 B，然后移除并重新 include A。 | B 只在其 edge 存在期间调用 Host private Method；两个 Modules 都读写 Host `@x`；protected calls 遵守 hierarchy receiver rules，且没有 private grant。初始 `rank` 是 B，B removal 后是 A，re-inclusion 创建新的 A edge，MRO 中每个 closed Module 只出现一次。Interpreter 和 JIT 产生相等的 MRO 与 values。 | `D-314`, `D-315`, `D-316`, `D-317`, `D-318`, `D-319`, `D-320` |
| `IRIS-V1-META-V436` | diagnostic | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v436/iris.toml` 声明 `contract Drawable { fun draw(Integer) -> String }`、`class Canvas for Drawable { impl fun draw(Integer) -> String { "old" } }`，以及带不兼容 `draw(String) -> String` 的 Module Bad。不同 open transactions 移除 declared conformance metadata、把 runtime superclass 设到 declared bound 外、include Bad、替换 Contract-visible signature，并执行一次兼容 body-only replacement。 | 前四个 candidates 各自 raise `TypeContractError`，保留 Canvas revision 1、declared Contract set、superclass bound、MRO 和 Method signature，并发布 empty diffs。兼容 body replacement 发布 revision 2，且 `draw(1)` 返回 `"new"`；nominal identity 和 declared conformance 保持不变。 | `D-173`, `D-174`, `D-175`, `D-176` |
| `IRIS-V1-META-V437` | diagnostic | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v437/iris.toml` 把 `open class A { fun marker() -> String { "open" } }` 放在 origin `class A { fun marker() -> String { "origin" } }` 之前，然后以程序方式打开 alias `Alias = A`，并在 raise 前读取 staged property `x`。Cross-package companion 省略对 A origin 的 dependency；另一个 source 尝试 `open contract C`。 | Declaration collection 在调度 declarative open 前解析 A，已发布的 `marker()` 返回 `"open"`。Programmatic open 读取 staged `x`，external reflection 仍看到旧 property set，raise 让 A 的 prior revision 保持 active 且没有 `x`。缺失 dependency 发出 unresolved-origin link diagnostic；Contract open 发出 `OPEN_CONTRACT_FORBIDDEN` 且不创建 candidate。 | `D-177`, `D-178`, `D-179`, `D-180` |
| `IRIS-V1-META-V438` | differential | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v438/iris.toml` 导出顶层无条件 declarative open Method `static_extra`，并在一个 consumer 中直接导入它；第二个 consumer 只看到 facade re-export。Programmatic open 添加 `dynamic_extra`；static 和 `Dynamic<A>` callers 在 commit 前编译。 | Direct importer 以 static type `String` 调用 `static_extra()`；facade-only consumer 发出 `IRIS-STATIC-MEMBER-NOT-FOUND`。Commit 后，`Dynamic<A>` 调用 `dynamic_extra()` 并返回 `"dynamic"`，而已编译的 static A surface 保持不变。Interpreter 和 JIT 一致。 | `D-181` |
| `IRIS-V1-META-V439` | diagnostic | compiler required；interpreter required；JIT required；native optional | `fixtures/meta/v439/iris.toml` materializes `Box<String>` 和 `Box<Integer>`，然后打开 generic definition `Box<T>` 添加 Method `tag(T) -> T`；companion body 违反 Integer substitution，不同 source 尝试 declarative `open class Box<String>` 和 programmatic `Box<String>.open`。 | 有效 open 在一个 commit 中原子发布 definition 和两个 closed logical Classes，每个都有一个新 revision 和稳定 identity；calls 返回其 argument。违规 candidate raises `TypeContractError`，且不改变 definition 或 closed revision。两个 closed-open attempts 都 raise `CLOSED_GENERIC_OPEN_FORBIDDEN` 且不发布任何内容。 | `D-207`, `D-208` |
| `IRIS-V1-META-V440` | diagnostic | compiler required；interpreter required；JIT required；native 不适用 | `fixtures/meta/v440/iris.toml` 声明兼容 Contracts A 和 B，二者都 require `m(Object) -> String`；声明 `class Host for A, B mixin Provider { impl fun m(value: Object) -> String { "host" } }`；Provider 带有兼容 body。Open variants 移除 Provider、加回 Provider、在没有 `impl` 时替换 `m`、在没有 `override` 时替换 inherited Provider Method，并声明 `module Wrong for A`。 | 只有在 Host 自身 `impl` 仍满足两个 immutable conformance obligations 时，removing Provider 才允许；adding Provider 不转移 nominal conformance。缺少 `impl` 发出 `CONTRACT_IMPLEMENTATION_REQUIRES_IMPL`；inherited replacement missing `override` 发出 `IRIS-MEMBER-OVERRIDE-REQUIRED`；`module Wrong for A` 发出 `CONTRACT_FOR_CLASS_ONLY`。失败 variants 保留 Host revision 和 conformance set。 | `D-233`, `D-234`, `D-278`, `D-279` |
| `IRIS-V1-META-V441` | diagnostic | compiler required；interpreter required；JIT 不适用；native 不适用 | `fixtures/meta/v441/iris.toml` 声明 `contract C { fun m() -> Nil { nil } }` 和 companion `open contract C { fun n() -> Nil }`；没有 Class 或 Module source 依赖任一声明。 | Method body 发出 `CONTRACT_METHOD_BODY_FORBIDDEN`；open 发出 `OPEN_CONTRACT_FORBIDDEN`；两个输入都不发布 Contract C、不创建 Contract candidate，也不贡献 MRO Method。 | `D-275` |
