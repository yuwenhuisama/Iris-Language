# Iris v1 一致性框架

状态：Iris v1 草案，冻结一致性契约。

IRIS-V1-CONFORMANCE-C001：本章定义了稳定的Iris v1可执行示例和一致性向量模式。它指定如何表示 normative 示例、章节向量表、diagnostic 案例、格式错误的输入、differential 后端检查、legacy 迁移标签、未来语料库记录和冻结门槛。它 MUST NOT 实现运行程序、添加测试、定义产品 source 代码或更改语言语义。

IRIS-V1-CONFORMANCE-C002：一致性语料库是一个规范工件契约，而不是本文档浪潮中的已交付运行程序。未来的实现工作 MAY 在此处指定的位置创建文件，但此任务仅冻结记录形状、required 类别、覆盖规则和验证门。

## 一致性条款

IRIS-V1-CONFORMANCE-C003：一致性向量是一种稳定的、机器可读的记录，它描述输入、适用性、预期观察、required 子句锚点、required 决策锚点，并允许一种 Iris v1 行为的实现自由。

IRIS-V1-CONFORMANCE-C004：可执行示例是 normative 章节中的 source 文本，具有稳定的 `IRIS-V1-<chapter>-EX<nnn>` ID。如果一个可执行示例旨在影响一致性，则它 MUST 可以由具有相同可观察结果的向量表示，也可以由解释其为何仅用于文档的可追溯行表示。

IRIS-V1-CONFORMANCE-C005：格式错误的输入向量是 negative 或 diagnostic 向量，其输入故意不是有效的完整 Iris 程序、package、metadata 记录、native 工件、serialized 值或 FFI sidecar。它 MUST 仍然具有稳定的预期诊断或拒绝状态。

IRIS-V1-CONFORMANCE-C006：后端 differential 向量检查解释器、JIT、native 边界或主机集成路径是否在多个后端应用的情况下产生相同的 required Iris v1 观察结果。 differential 向量 MUST NOT 允许这些后端之间出现语义上不同的结果。

IRIS-V1-CONFORMANCE-C007：legacy 标记的向量携带来自 `11-migration-divergence.md` 的迁移上下文。 legacy 标记记录该案例是否保留、故意偏离、删除或推迟 Legacy Iris 行为，但预期结果仍然仅由 Iris v1 normative 子句锚点控制。

## 稳定的ID和记录身份

IRIS-V1-CONFORMANCE-C008：每个向量 ID MUST 使用 [README.md](README.md) 中的固定章节代码匹配 `IRIS-V1-<chapter>-V<nnn>`。章节拥有的向量 ID MUST 一旦发布就保持稳定。

IRIS-V1-CONFORMANCE-C009：语料库记录 MUST 包含人类可读的 `name`，但工具 MUST NOT 将 `name` 视为稳定身份。只有 `id` 是稳定的向量标识。当 `id`、锚点、输入和预期观察值不变时，`name` MAY 为清晰性而改变。

IRIS-V1-CONFORMANCE-C010：使用种类 `Failure` 的章节向量表行会映射到规范类别 `negative`，除非预期结果只是 diagnostic 流义务；在这种情况下，它会映射到 `diagnostic`。已发布的向量 ID 和可观察结果 MUST 被保留。

IRIS-V1-CONFORMANCE-C011：稳定记录标识包括 `id`、`schema_version`、`source.chapter`、`source.clauses`、`source.decisions`、`category`、`input.kind`、`applicability` 和 `expect`。对这些字段中任何一个的更改都是语义语料库更改，并且 MUST 被作为规范更改进行审查。

IRIS-V1-CONFORMANCE-C012：初始架构版本是 `iris-v1-vector-schema-1`。未来不兼容的模式版本 MUST 需要未来的 Iris 语言主要版本或保留所有 Iris v1 向量标识和观察结果的迁移文档。

## 规范类别

IRIS-V1-CONFORMANCE-C013：`category` 字段 MUST 是 `positive`、`negative`、`diagnostic` 或 `differential` 之一。

| Category | Required meaning | Required expected fields |
| --- | --- | --- |
| `positive` | 有效输入完成并生成 required 观察结果 | 至少 `stdout`、`value`、`type`、`diagnostics`、`status` 或 `artifact` 之一 |
| `negative` | 输入被拒绝或执行引发 required 普通 Iris 错误 | `error` 或 `status`，当所属子句指定 diagnostic 代码时加上 `diagnostics` |
| `diagnostic` | 主要义务是 diagnostic 排放、警告策略或结构化 diagnostic 有效负载 | `diagnostics` |
| `differential` | 多个适用的执行路径必须就相同的语义观察达成一致 | `backends`、`equivalence` 和至少一个普通预期观察结果 |

IRIS-V1-CONFORMANCE-C014: `positive` 向量 MUST NOT 接受错误结果，除非向量显式测试将错误对象表示为成功值的普通 Iris 值。

IRIS-V1-CONFORMANCE-C015: `negative` 向量 MUST 区分解析器、静态验证、运行时异常、ABI 状态、package 加载、metadata 验证、serialized 输入拒绝和主机边界拒绝（当拥有子句进行区分时）。

IRIS-V1-CONFORMANCE-C016: `diagnostic` 向量 MUST 记录稳定的主 diagnostic 代码、严重性、阶段和锚定子句。当这些详细信息为 normative 时，它们 MAY 记录辅助注释、跨度、警告策略行为和结构化负载密钥。

IRIS-V1-CONFORMANCE-C017: `differential` 向量 MUST 命名每个比较的后端或路径。如果后端不适用，记录 MUST 将其标记为 `not_applicable` 并带有子句支持的原因，而不是默默地省略它。

## 记录模式

IRIS-V1-CONFORMANCE-C018：未来的机器可读向量记录 MUST 是 UTF-8 JSON 对象，仅使用本章中的稳定字段名称。 JSON 对象成员顺序不是语义的。

IRIS-V1-CONFORMANCE-C019：向量记录 MUST 包含以下顶级字段：`schema_version`、`id`、`name`、`category`、`source`、`input`、`applicability`、 `expect` 和 `tags`。 `name` 字段 MUST 是一个非空的人类可读标签，MUST NOT 用于身份、查找稳定性、重复数据删除、替换匹配或可追溯性连接。

IRIS-V1-CONFORMANCE-C020：`source` 对象 MUST 包含 `chapter`、`artifact`、`clauses` 和 `decisions`。 `clauses` MUST 包含至少一个 `IRIS-V1-<chapter>-C<nnn>` 锚点。 `decisions` MUST 包含解释预期结果所需的每个已知的冻结 `D-<nnn>` 决策，或者仅当向量纯粹是编辑且不存在决策锚时才是空数组。

IRIS-V1-CONFORMANCE-C021：`input` 对象 MUST 包含 `kind` 以及 `source_text`、`files`、`package`、`metadata`、`native_artifact`、`serialized_value` 之一， `host_calls` 或 `fixture_ref`。 `kind` MUST 是 `source`、`package`、`metadata`、`native`、`ffi`、`serialized`、`host`、`legacy` 之一，或者`fixture`。

IRIS-V1-CONFORMANCE-C022：`input` 对象 MAY 仅针对 negative 或 diagnostic 向量包含 `malformed: true`。当 `malformed` 为 true 时，记录 MUST 仍然标识 `kind` 中的畸形表面，并且 MUST 在 `expect.diagnostics` 或 `expect.error.phase` 中命名拒绝阶段。

IRIS-V1-CONFORMANCE-C023：`applicability` 对象 MUST 包含 `interpreter`、`jit` 和 `native`。每个值 MUST 是 `required`、`optional`、`not_applicable` 或 `prohibited` 之一。

IRIS-V1-CONFORMANCE-C024：`applicability.reason` MUST 在任何适用性值为 `optional`、`not_applicable` 或 `prohibited` 时存在。该 reason MUST 引用至少一个 source 子句，或说明该输入 kind 不存在对应后端。

IRIS-V1-CONFORMANCE-C025：`expect` 对象 MUST 仅使用这些稳定的观察字段：`stdout`、`stderr`、`value`、`type`、`error`、`diagnostics`、`status`、 `artifact`、`backends`、`equivalence` 和 `side_effects`。

IRIS-V1-CONFORMANCE-C026: `expect.stdout` 和 `expect.stderr` MUST 是精确的 UTF-8 行数组，不带行终止符。字节流观察 MUST 将 `expect.artifact` 与显式媒体类型和字节编码结合使用。

IRIS-V1-CONFORMANCE-C027: `expect.value` MUST 使用以下一种 kind 描述可观察的 Iris 值：`nil`、`bool`、`integer`、`float32_bits`、`float64_bits`、`string`、`symbol`、`bytes_hex`、`array`、`tuple`、`hash_entries`、`range`、`regex`、`object_identity`、`class_name`、`module_name`、`contract_name`、`task`、`exception_context` 或 `opaque_handle_status`。

IRIS-V1-CONFORMANCE-C028: `expect.type` MUST 使用规范化后的 Iris v1 规范 Type 拼写。来自 [05-types-contracts-generics.md](05-types-contracts-generics.md) 的类型代数向量 MUST 比较规范化的 Type 身份，而不是只比较显示拼写。

IRIS-V1-CONFORMANCE-C029: `expect.error` MUST 包含 `kind`、`phase` 和 `message_policy`。 `kind` MUST 是普通 Iris 错误类、ABI 状态名称、package 拒绝类型、metadata 拒绝类型、serialized 输入拒绝类型或所属子句的解析器/静态 diagnostic 类别 required。

IRIS-V1-CONFORMANCE-C030: `expect.diagnostics` MUST 是一个数组。每个 diagnostic 条目 MUST 包含 `code`、`severity`、`phase` 和 `clause`。只有当 `span`、`notes`、`payload` 和 `policy` 为 normative 时，条目 MAY 包含这些字段。

IRIS-V1-CONFORMANCE-C031: `expect.status` MUST 用于 Host ABI、native 扩展、package 加载、metadata 验证、FFI 绑定，以及不是普通 Iris 值的 serialized 值解码结果。它 MUST 包含 `kind` 和 `success`。

IRIS-V1-CONFORMANCE-C032: `expect.artifact` MUST 仅在所属章节冻结的语义级别描述生成或验证的工件。它 MUST NOT 需要逐字节实现输出，除非所属章节将该字节序列设为 normative。

IRIS-V1-CONFORMANCE-C033: 当向量依赖于无部分突变、一次性清理、无 native 调用、无 package 发布、无堆突变、无候选提交、无 Hash 突变或特定 diagnostic 流发射时，`expect.side_effects` MUST 出现。

IRIS-V1-CONFORMANCE-C034: `tags` MUST 是小写字符串数组。保留的标记前缀为 `legacy:`、`phase:`、`surface:`、`backend:`、`hash:`、`unicode:`、`ffi:`、`package:`、`diagnostic:` 和 `differential:`。

IRIS-V1-CONFORMANCE-C035：旧标记 MUST 使用 `legacy:preserve`、`legacy:intentional-divergence`、`legacy:removed`、`legacy:deferred` 或 `legacy:not-applicable` 之一。一旦 `11-migration-divergence.md` 完成，legacy 标记的向量 SHOULD 也会引用迁移行。

## 适用规则

IRIS-V1-CONFORMANCE-C036：对于每个 source、package、运行时、类型、集合、异步、metadata 和 diagnostic 向量，解释器适用性为 `required`，除非该向量只执行 Host ABI 或 native 工件加载，且不执行 Iris source。

IRIS-V1-CONFORMANCE-C037：当向量涵盖优化代码可能执行的可观察语言语义时，JIT 适用性为 `required`，包括数字舍入、稳定哈希结果、分派、通用防护、异常选择、迭代、异步恢复、开放事务失效和后端 differential 行为。

IRIS-V1-CONFORMANCE-C038：JIT 适用性是 `not_applicable`，用于纯词法拒绝、解析拒绝、可执行代码之前的静态 metadata 验证以及 Host ABI 表协商，除非章节明确要求 JIT 参与。

IRIS-V1-CONFORMANCE-C039：对于 Host ABI、native 扩展、FFI、native metadata、native payload、native async bridge 和 native boundary Contract 向量，native 适用性为 `required`。对于没有 native 交互的纯 Iris source 语法向量，native 适用性为 `not_applicable`。

IRIS-V1-CONFORMANCE-C040：没有向量 MAY 允许解释器和 JIT 为相同的适用语义情况生成不同的 Iris 值、类型、错误、诊断、副作用或稳定的哈希结果。

IRIS-V1-CONFORMANCE-C041：D-049 和 D-050 要求 source 浮点 literal 向量跨编译器 host、解释器、JIT 和目标平台比较精确发出位和 diagnostics。host locale、host rounding mode 和不合适的 host 浮点解析器 MUST NOT 影响预期结果。

IRIS-V1-CONFORMANCE-C042：D-077 到 D-086 和 D-113 需要哈希向量来比较精确的公共 64 位 Integer 结果、精确的派生密钥上下文字符串以及精确的摘要提取规则（这些详细信息位于所属章节中）。

IRIS-V1-CONFORMANCE-C043：D-272 要求 revision artifact integrity 向量在测试 artifact recovery 行为时比较完整的 BLAKE3-256 manifest digest。语料库记录 MUST NOT 为了一致性便利而缩短该摘要。

IRIS-V1-CONFORMANCE-C044: D-503 需要 serialized `IrisValue` 向量来标识单独指定的格式版本和严格的解码限制义务。本章 MUST NOT 定义该字节格式，但一旦格式工件存在，冻结门槛 MUST 要求提供向量。

IRIS-V1-CONFORMANCE-C045：D-507 到 D-510 要求语法向量涵盖完整的优先级、关联性、不可链接运算符、封闭保留关键字以及与 positive 和格式错误的记录的上下文最长匹配冲突。

## 按章节所需的向量类

IRIS-V1-CONFORMANCE-C046：语料库 MUST 保留任何 Iris v1 章节发布的所有 normative 向量 ID，包括 `LIBRARY` 向量以及普通向量表之前的稳定哈希表、代数表和样本形状表。未发布向量 ID 的章节仍需要另一章中的映射向量义务或显式 metadata-only 例外。

| Chapter | Required vector classes before freeze |
| --- | --- |
| `IDENTITY` | 没有章节拥有的向量 ID 是 required，因为章节定义了身份、版本控制、延迟和非目标 metadata。冻结仍然要求将每个可执行身份义务映射到相关章节向量或可追溯性矩阵 metadata-only 异常。 |
| `GRAMMAR` | 关键字清单、运算符清单、优先级、关联性、上下文最长匹配、格式错误的词法输入、格式错误的解析输入和 legacy 非关键字拒绝 |
| `RUNTIME` | 对象标识、动态调度、Method 和 BoundMethod 标识、MRO、构造、原始 ivars、真值、数值相等和算术、稳定数值和单例哈希、突变失效和运行时故障 |
| `CONTROL` | 绑定、Closure 捕获、可调用参数、赋值求值顺序、条件、循环、遍历清理、标签、匹配、raise/catch/finally、ExceptionContext 和无效控制传输 |
| `TYPES` | 每个类型构造函数、代数规范化、Contract 限定调度、Contract 视图相等性和散列、通用不变性和具体化、Dynamic 边界、类型别名、`Never`、反射和故障诊断 |
| `COLLECTIONS` | String、MutableString、Bytes、ByteArray、Tuple、Array、Hash、Range、迭代、Regex、Unicode 17.0.0 行为、稳定的集合哈希、格式错误的 Regex 和文本案例以及并发修改失败 |
| `ASYNC` | Task、Awaitable、等待放置、FIFO 调度程序行为、失败的 Task 观察、未观察到的 Task 诊断、`using`、Closeable 清理、跨挂起的异步清理和 diagnostic 流有效负载 |
| `META` | 包、导入、导出、初始化 DAG、声明体、开放事务、回滚、修订事件、MetaCapabilities、装饰器、ReflectionPolicy、原始 ivars、静态扩展边界、热升级和故障回滚 |
| `FFI` | Host ABI 协商、不透明句柄、GC 根、线程关联、post queue、native 错误、metadata 绑定、native 有效负载、Task 完成令牌、FFI.open、库绑定和不安全调用拒绝 |
| `LIBRARY` | 已发布向量 `IRIS-V1-LIBRARY-V001` 至 `IRIS-V1-LIBRARY-V014`，涵盖 JSON 范围、`IrisValue` 格式边界、编码 API、核心 package 成员资格、标准 package 类别、延迟 package 区域、安全解码限制和非核心库拒绝 |
| `MIGRATION` | 迁移分类帐行是 `IRIS-V1-MIG-<nnn>` 行，而不是向量 ID。冻结需要与迁移相关的一致性向量，或语法、运行时、控制、类型、集合、元、库或一致性语料库中的显式映射向量义务，对于每个分类帐标签，更改 legacy 语法，删除 legacy 关键字，替换示例，有意分歧，保留案例和延迟 legacy 功能。 |

IRIS-V1-CONFORMANCE-C047：只有当每个涵盖的子句都列在 `source.clauses` 中时，章节才可以通过覆盖多个子句的一个向量来满足 required 向量类，并且如果任何涵盖的子句错误，则预期的观察将失败。

IRIS-V1-CONFORMANCE-C048：冻结门槛 MUST 拒绝具有 normative 子句但没有向量类覆盖的章节，除了可追溯性矩阵标记为文档结构而不是可执行行为的仅可追溯性子句。

IRIS-V1-CONFORMANCE-C049：章节标记为 required 的向量类 MUST 至少有一个 positive 向量和一个 negative 或 diagnostic 向量，除非所属子句只定义 prohibited 行为。仅禁止区域 MUST 至少有一个 negative 或 diagnostic 向量。

## 未来语料库的位置和格式

IRIS-V1-CONFORMANCE-C050：未来的机器可读语料库文件 SHOULD 位于 14 文件规范清单之外的 `conformance/iris-v1/` 下。首选布局是 `conformance/iris-v1/vectors/<chapter>/<id>.json`、`conformance/iris-v1/fixtures/` 和 `conformance/iris-v1/schema/iris-v1-vector-schema-1.json`。

IRIS-V1-CONFORMANCE-C051：语料库文件 MUST NOT 将添加到 `spec/iris-v1/` 下，除非 [README.md](README.md) 中的产品工件清单被批准的计划更改。

IRIS-V1-CONFORMANCE-C052：语料库包 MUST 包含名为 `conformance/iris-v1/manifest.json` 的清单，用于记录语料库模式版本、Iris 语言主版本、Unicode 版本、哈希模式版本、source 规范提交或发布标识符、向量计数、fixture 摘要列表，以及任何单独版本化的 `IrisValue` 格式引用。

IRIS-V1-CONFORMANCE-C053：夹具文件 MUST 由 BLAKE3-256 进行内容寻址，并记录 MUST 引用 fixture 摘要、逻辑路径、媒体类型和角色。夹具路径和镜像 MUST NOT 影响预期的语义结果。

IRIS-V1-CONFORMANCE-C054：语料库验证器 MUST 拒绝重复的向量 ID、格式错误的 ID、未知的章节代码、未解析的 source 子句、未解析的 D-ID、不支持的类别值、缺失预期观察值、格式错误的适用性值以及保留词汇表之外的 legacy 标签。

## 有效和无效样本记录

IRIS-V1-CONFORMANCE-C055：以下有效示例记录对 schema 形状是 normative。除了所引用的语法子句和决策之外，它不是新的语言测试。

```json iris-v1-vector-valid
{
  "schema_version": "iris-v1-vector-schema-1",
  "id": "IRIS-V1-GRAMMAR-V003",
  "name": "Exponent precedence parses right-associative exponent before unary negation",
  "category": "positive",
  "source": {
    "chapter": "GRAMMAR",
    "artifact": "spec/iris-v1/02-lexical-grammar.md",
    "clauses": ["IRIS-V1-GRAMMAR-C041", "IRIS-V1-GRAMMAR-C042"],
    "decisions": ["D-033", "D-507", "D-508"]
  },
  "input": {
    "kind": "source",
    "source_text": "2 ** 3 ** 2\n-2 ** 2\n2 ** -3"
  },
  "applicability": {
    "interpreter": "required",
    "jit": "required",
    "native": "not_applicable",
    "reason": "Pure source expression parsing has no native boundary."
  },
  "expect": {
    "artifact": {
      "parse_shapes": ["2 ** (3 ** 2)", "-(2 ** 2)", "2 ** (-3)"]
    }
  },
  "tags": ["phase:parse", "surface:operator", "differential:interpreter-jit"]
}
```

IRIS-V1-CONFORMANCE-C056：以下无效示例记录对检查器失败形状是 normative。模式检查器 MUST 拒绝它，因为 negative 向量同时缺少 `expect.error` 和 `source.decisions`。

```json iris-v1-vector-invalid
{
  "schema_version": "iris-v1-vector-schema-1",
  "id": "IRIS-V1-FFI-V018",
  "name": "Signature-less FFI native call is rejected",
  "category": "negative",
  "source": {
    "chapter": "FFI",
    "artifact": "spec/iris-v1/09-native-host-ffi.md",
    "clauses": ["IRIS-V1-FFI-C046"],
    "decisions": []
  },
  "input": {
    "kind": "ffi",
    "source_text": "FFI.open(path).call(:danger)"
  },
  "applicability": {
    "interpreter": "required",
    "jit": "not_applicable",
    "native": "required",
    "reason": "FFI binding crosses the native boundary."
  },
  "expect": {},
  "tags": ["ffi:binding", "phase:runtime"]
}
```

## 冻结门槛

IRIS-V1-CONFORMANCE-C057: Iris v1 规范冻结 MUST 拒绝任何带有未覆盖的 normative 子句的语料库或可追溯性集，除非该子句明确仅可追溯性、提供信息或标记为 `DEFERRED V1` 且没有可执行义务。

IRIS-V1-CONFORMANCE-C058：冻结 MUST 拒绝未解析的链接、未解析的 Markdown 锚点、未解析的 D-ID、未解析的向量 ID、未解析的示例 ID、重复的 ID、格式错误的 ID 前缀以及对不在固定清单中的章节的引用。

IRIS-V1-CONFORMANCE-C059：冻结 MUST 拒绝任何被测试或描述为 normative 行为的延迟项目。只有当预期结果为拒绝、无功能或显式延迟状态报告时，延迟项目 MAY 具有 diagnostic 或迁移向量。

IRIS-V1-CONFORMANCE-C060：冻结 MUST 拒绝依赖于后端的行为。任何影响 Iris 值、类型、错误、稳定哈希、工件完整性、副作用或排序语义的解释器、JIT、native、主机或平台差异都是不合格的，除非 source 子句明确将后端标记为不适用。

IRIS-V1-CONFORMANCE-C061：冻结 MUST 拒绝省略其类别的 required 预期观察值的向量记录，省略 required 错误或 diagnostic（对于格式错误的输入），省略适用性，在 IRIS-V1-CONFORMANCE-C035 之外使用 legacy 标记，或引用一个子句而不保留该子句子句的可观察结果。

IRIS-V1-CONFORMANCE-C062：冻结 MUST 拒绝其向量 ID 未在未来语料库清单中表示的章节表，除非清单包含具有相同向量 ID 和相同预期观察的显式替换记录。

IRIS-V1-CONFORMANCE-C063：仅当 `IRIS-V1-LIBRARY-V001` 到 `IRIS-V1-LIBRARY-V014` 被保留或使用相同的预期观察值进行映射时，冻结 MUST 将 `LIBRARY` 视为完整的章节拥有的向量发布。冻结 MUST 继续将 `MIGRATION` 视为不完整，直到每个迁移分类帐行都有与迁移相关的一致性向量或显式映射向量义务。仅当所有可执行标识义务都由依赖章节向量覆盖并且所有剩余标识子句在可追溯性中标记为 metadata-only 时，冻结 MUST 才将 `IDENTITY` 视为完整的，没有章节拥有的向量 ID。

IRIS-V1-CONFORMANCE-C064：冻结 MUST 拒绝使用 Legacy Iris 脚本、旧生成的解析器文件、旧 PDF 文本、当前实现怪癖或 native 扩展示例作为 normative 权限，除非向量引用明确采用该行为的冻结 Iris v1 子句。

IRIS-V1-CONFORMANCE-C065：如果 IRIS-V1-CONFORMANCE-C055 中的有效样本失败或 IRIS-V1-CONFORMANCE-C056 中的无效样本通过，则冻结 MUST 拒绝样本模式验证。

## 验证清单

IRIS-V1-CONFORMANCE-C066：本章的文档验证器 MUST 检查本地子句序列、重复 ID、Markdown 表形状、本地链接、示例记录验证、required 类别词汇、required 预期字段、保留 legacy 标记、引用的 D-ID 格式以及此任务中缺少运行程序或产品实现文件。

IRIS-V1-CONFORMANCE-C067：语料库覆盖率检查器 MUST 提取 normative 章节子句、示例、向量表和可追溯行，然后按章节、类别、向量类、后端适用性、格式错误输入覆盖率、legacy 标签覆盖率和 D-ID 覆盖率报告覆盖率。

IRIS-V1-CONFORMANCE-C068：differential 检查器 MUST 仅比较 `expect` 定义的语义观察结果。它 MUST NOT 比较实现日志、堆地址、对象指针值、计时、私有字节码布局或非 normative 调试字符串。

IRIS-V1-CONFORMANCE-C069：diagnostic 检查器 MUST 比较稳定的 diagnostic 代码、严重性、阶段、子句锚点和 required 有效负载密钥。它 MUST NOT 需要精确的英语 diagnostic 措辞，除非拥有子句明确冻结该措辞。

IRIS-V1-CONFORMANCE-C070：格式错误输入检查器 MUST 证明格式错误的词法、解析、metadata、FFI、Host、native 工件、package 和 serialized-value 输入会在指定阶段失败，并且当副作用属于预期观察的一部分时不会产生部分语义效果。

IRIS-V1-CONFORMANCE-C071：legacy 覆盖检查器 MUST 将迁移分类帐标签与向量标签进行比较，并拒绝任何已删除或故意偏离的缺少 v1 替换、拒绝或延迟状态向量的 Legacy Iris 行为。

## 可追溯性注释

IRIS-V1-CONFORMANCE-C072：本章锚定了已批准的验证策略，示例将其编译到未来的机器可读一致性语料库中。它不拥有新的语言语义。

IRIS-V1-CONFORMANCE-C073：本章引用了决策 ID `D-049`、`D-050`、`D-077`、`D-078`、`D-079`、`D-080`、`D-081`、`D-082`、 `D-083`、`D-084`、`D-085`、`D-086`、`D-113`、`D-272`、`D-503`、`D-507`、`D-508`、`D-509` 和`D-510` 用于一致性敏感的文字、散列、工件、序列化、优先级、关联性、关键字和上下文令牌义务。

IRIS-V1-CONFORMANCE-C074：本章还依赖于 [README.md](README.md) 子句 IRIS-V1-TRACE-C001 到 IRIS-V1-TRACE-C018 用于库存、ID、编辑、非目标、冻结语义和后端独立规则。

## 审计精确一致性向量

IRIS-V1-CONFORMANCE-C075：以下一致性向量是 normative 元级向量定义。每个向量使用具体的 JSON 记录 fixture 作为输入，并定义准确的验证器接受、拒绝、状态、诊断或工件观察。

| Vector ID | Category | Applicability | Source/Input | Expected observable | Decisions |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-CONFORMANCE-V010` | diagnostic | 解释器 required; JIT 不适用； native 不适用；语料库验证器 required | JSON 记录 fixture `{"schema_version":"iris-v1-vector-schema-1","id":"IRIS-V1-LIBRARY-V005","name":"Reject unsupported IrisValue format version","category":"negative","source":{"chapter":"LIBRARY","artifact":"spec/iris-v1/10-serialization-standard-library.md","clauses":["IRIS-V1-LIBRARY-C007","IRIS-V1-LIBRARY-C008"],"decisions":["D-503"]},"input":{"kind":"serialized","serialized_value":{"format":"IrisValue","bytes_hex":"4952563102","declared_version":2,"limit_profile":"v1-default"},"malformed":true},"applicability":{"interpreter":"required","jit":"not_applicable","native":"not_applicable","reason":"Serialized IrisValue validation is a decode-surface check before executable code."},"expect":{"status":{"kind":"IrisValueUnsupportedVersion","success":false},"diagnostics":[{"code":"IRISVALUE_UNSUPPORTED_VERSION","severity":"error","phase":"decode","clause":"IRIS-V1-LIBRARY-C008"}],"side_effects":{"published_value":false,"allocation_from_unvalidated_size":false}},"tags":["phase:decode","surface:serialized"]}` | 验证者接受记录； `expect.status.success` 是 `false`； diagnostic 代码正是 `IRISVALUE_UNSUPPORTED_VERSION`； `phase` 正是 `decode`； `side_effects.published_value` 是 `false`； `side_effects.allocation_from_unvalidated_size` 是 `false`。 | `D-503` |
| `IRIS-V1-CONFORMANCE-V011` | positive | 解释器 required; JIT required; native 不适用；语料库验证器 required | JSON 记录 fixture `{"schema_version":"iris-v1-vector-schema-1","id":"IRIS-V1-GRAMMAR-V008","name":"Full precedence source forms parse to canonical shapes","category":"positive","source":{"chapter":"GRAMMAR","artifact":"spec/iris-v1/02-lexical-grammar.md","clauses":["IRIS-V1-GRAMMAR-C041","IRIS-V1-CONFORMANCE-C045"],"decisions":["D-507"]},"input":{"kind":"source","source_text":"a.b(c)[d] ** -e * f + g << h & i ^ j \u007c k ..< l < m == n named o && p \u007c\u007c q = r"},"applicability":{"interpreter":"required","jit":"required","native":"not_applicable","reason":"Pure source parsing has no native boundary."},"expect":{"artifact":{"parse_shape":"assign(q_or_chain(logical_or(logical_and(named_infix(equality(relational(range(bit_or(bit_xor(bit_and(shift(add(mul(pow(primary_chain(a.b(c)[d]), unary(-e)), f), g), h), i), j), k), l), m), n), named, o), p)), r)","precedence_rows_covered":17},"diagnostics":[]},"tags":["phase:parse","surface:operator","differential:interpreter-jit"]}` | 验证者接受记录； `expect.artifact.precedence_rows_covered` 正是 `17`； `expect.diagnostics` 为空；解释器和 JIT 适用性都是 `required`； native 适用性为 `not_applicable`。 | `D-507` |
| `IRIS-V1-CONFORMANCE-V012` | diagnostic | 解释器 required; JIT required; native 不适用；语料库验证器 required | JSON 记录 fixture `{"schema_version":"iris-v1-vector-schema-1","id":"IRIS-V1-GRAMMAR-V007","name":"Reject non-associative operator chains","category":"diagnostic","source":{"chapter":"GRAMMAR","artifact":"spec/iris-v1/02-lexical-grammar.md","clauses":["IRIS-V1-GRAMMAR-C043","IRIS-V1-CONFORMANCE-C045"],"decisions":["D-508"]},"input":{"kind":"source","source_text":"a < b < c\nx as T as U","malformed":true},"applicability":{"interpreter":"required","jit":"required","native":"not_applicable","reason":"Pure parse diagnostics have no native boundary."},"expect":{"diagnostics":[{"code":"PARSE_NONASSOCIATIVE_CHAIN","severity":"error","phase":"parse","clause":"IRIS-V1-GRAMMAR-C043","payload":{"forms":["a < b < c","x as T as U"]}}],"side_effects":{"program_accepted":false,"bytecode_emitted":false}},"tags":["phase:parse","surface:operator","diagnostic:nonassociative-chain"]}` | 验证者接受记录； diagnostic 数组长度为 `1`； diagnostic 代码正是 `PARSE_NONASSOCIATIVE_CHAIN`； `phase` 正是 `parse`； `side_effects.program_accepted` 是 `false`； `side_effects.bytecode_emitted` 是 `false`。 | `D-508` |
