# Iris v1 序列化与标准库边界

状态：Iris v1 草案，语义冻结。

IRIS-V1-LIBRARY-C001：本章定义 Iris v1 的序列化契约边界、JSON 职责、单独版本化的 IrisValue 职责、安全解码限制、Encoding 和 Unicode 标准表面、核心包边界、官方标准包类别、延后库领域，以及 Regex 包拆分。本章 MUST 在阅读 [README.md](README.md)、[01-language-identity.md](01-language-identity.md)、[05-types-contracts-generics.md](05-types-contracts-generics.md)、[06-collections-text-regex.md](06-collections-text-regex.md)、[08-modules-metaprogramming.md](08-modules-metaprogramming.md) 和 [09-native-host-ffi.md](09-native-host-ffi.md) 之后阅读。

IRIS-V1-LIBRARY-C002：本章 MUST NOT 定义解析器产生式、集合存储布局、早前章节以外的稳定哈希规范字节、确切的 IrisValue 字节标签、JSON 解析器实现策略、Regex 引擎内部、加密 API、HTTP API、数据库 API、GUI API、数据库，或 Host ABI 函数名称。本章只固定所有权、资格、版本化和安全规则。

## 序列化契约范围

IRIS-V1-LIBRARY-C003：Iris v1 序列化是显式且由 Contract 驱动的。语言核心和标准库 MUST NOT 仅因为这些状态存在，就通过反射序列化任意对象、以接收者名称标识的原始 ivar、Class 对象 ivar、层级 Class 变量、私有字段、反射元数据表、Method 主体、Closure 捕获、Task 状态、FFI 句柄、原生载荷或外部资源。

IRIS-V1-LIBRARY-C004：标准 `Serializable` Contract 是一个选择加入的承诺，表示 Class 提供显式的序列化表示。只有当 Class 声明的静态脊柱列出 `for Serializable`，或列出对它进行细化的更具体标准序列化 Contract 时，该 Class 才参与序列化。鸭子类型、反射可见性、`to_string`、`inspect`、原始 ivar 访问和公共属性存在，MUST NOT 暗示具备序列化资格。

IRIS-V1-LIBRARY-C005：`Serializable` 表示是 Class 实现自行选择的普通 Iris 数据。它 MUST 由所选目标格式接受的值构成，并在该值意图按名义 Class 实例往返时包含显式类型身份和 schema 元数据。该表示 MUST NOT 包含隐藏运行时指针、对象 ID、原始 ivar 名称或值，除非 Class 有意把这些精确数据作为普通表示内容发布；也 MUST NOT 包含原生句柄、打开的文件描述符、调度器状态或绕过 ReflectionPolicy 的数据。

IRIS-V1-LIBRARY-C006：反序列化为名义 Class 时，MUST 在发布结果前验证包身份、包 API 主版本、Iris 语言主版本、声明的 schema 版本、声明的 `Serializable` Contract 符合性、泛型约束、构造器或工厂 Contract，以及表示形状。失败会引发普通 Iris 解码错误或 Type Contract 错误，并且 MUST NOT 发布部分初始化的对象。

IRIS-V1-LIBRARY-C007：`Serializable` 不会覆盖可见性、MetaCapabilities、包权限或 ReflectionPolicy。私有数据只有在 Class 自己的序列化实现用其普通权限显式发出时，才可以出现在序列化表示中。外部调用者不能通过选择 JSON、IrisValue、反射或 FFI API 来取得私有状态。

IRIS-V1-LIBRARY-C008：下列序列化资格矩阵具有规范性：

| 值族 | JSON 默认 | IrisValue 默认 | Serializable 选择加入 | 必须拒绝或限制 |
| --- | --- | --- | --- | --- |
| `nil`, Bool, Integer, finite Float | 满足 JSON 数值限制时支持 | 支持，包括任意 Integer 和声明的 Float 宽度或位 | 不需要 | JSON 拒绝非有限 Float，除非显式包选项把它映射为数据 |
| String and Symbol | 支持 String，Symbol 只通过显式表示支持 | 支持 | Symbol 只能在格式声明的位置表示为内容 | String 和 Symbol 不可能有无效 Unicode |
| Bytes | 不是 JSON 标量，需要显式表示，例如编码文本 | 支持 | JSON 可选表示 | 解码器在分配前验证声明的编码和长度 |
| Tuple and Array | 当每个元素都受支持时按数组支持 | 支持 | 容器形状不需要 | 元素数量和嵌套深度有界 |
| Hash | 只支持 String 键，或显式键表示 | 每个键和值受支持时支持 | JSON 可选表示 | 重复解码键和无效键 Contract 是错误 |
| Regex and Match | Regex 只通过显式表示支持，Match 不是默认 JSON | 只有 IrisValue 格式版本声明 Regex 时才支持 Regex，Match 不是默认 | 可选表示可以命名 Regex pattern 和 flags | 核心 Regex 语义仍归 [06-collections-text-regex.md](06-collections-text-regex.md) 所有 |
| Ordinary Class instance | 不自动支持 | 不自动支持 | 名义往返需要 | 不自动转储原始 ivar、属性或反射表 |
| Class, Module, Contract, Type, Method, BoundMethod, Closure | 不自动支持 | 不自动支持，除非格式版本显式声明 Type 元数据 | 只允许显式元数据表示，不允许可执行身份 | 不复活代码、修订、Closure 环境或 Method 主体 |
| Task, Awaitable, Iterator, ExceptionContext, diagnostic event | 不自动支持 | 不自动支持 | Diagnostics 包可以定义显式记录 | 不序列化实时调度器、游标、清理和失败观察状态 |
| FFI Library, Host handle, native payload, File, socket, database connection, external resource | 不自动支持 | 不自动支持 | 只允许包选择的显式资源描述符 | 拒绝实时句柄和原生指针 |

## JSON 职责

IRIS-V1-LIBRARY-C009：`JSON` 是用于解析和发出 JSON 兼容数据的稳定运行时包。它拥有 JSON 文本语法符合性、JSON 值映射、规范错误报告和安全解析器限制。它不拥有任意 Iris 对象持久化、二进制 IrisValue 兼容性、加密、HTTP 传输、文件系统策略或 schema 语言设计。

IRIS-V1-LIBRARY-C010：JSON 解码 MUST 只生成 JSON 兼容 Iris 值，除非调用者显式提供 `Serializable` 目标、工厂或 schema。默认 JSON 值集合是 `nil`、Bool、String、处于实现所文档化 JSON 数值模式范围内的有限数值、Array，以及 String 键的 Hash。JSON 解码默认 MUST NOT 从输入中的类型名称实例化任意 Class。

IRIS-V1-LIBRARY-C011：JSON 编码默认 MUST 只接受 JSON 兼容值。Class 实例只能通过它的显式 `Serializable` 表示或调用者提供的编码器来编码。编码器 MUST 拒绝不受支持的值，而不是退回到 `to_string`、`inspect`、对象身份、原始 ivar 扫描、Method 枚举，或绕过 ReflectionPolicy 的元数据。

IRIS-V1-LIBRARY-C012：JSON 解析和生成在显式文本或二进制边界使用 Unicode 标量 String 和 UTF-8 Bytes。无效 UTF-8 字节会在解释 JSON token 前引发 `EncodingError`。JSON API MUST NOT 隐式选择 OS 区域设置、进程代码页、终端编码、文件系统编码或 Host 默认编码。

IRIS-V1-LIBRARY-C013：JSON 安全解码限制 MUST 显式，并且可通过带安全默认值的普通选项配置。符合实现至少 MUST 对输入字节长度、解码文本长度、嵌套深度、对象成员数量、数组元素数量、字符串标量长度、数值 token 长度和总输出分配量设定上界。解码器 MUST 在分配目标缓冲区或容器前验证大小。

IRIS-V1-LIBRARY-C014：JSON 对象重复名称行为 MUST 是文档化选项。安全默认值 MUST 在解码到 Hash 或类型化表示时拒绝重复名称，除非调用者显式选择确定性的后者胜、前者胜，或全部收集策略。所选策略 MUST 在诊断中可见，并且 MUST NOT 依赖哈希迭代顺序。

## IrisValue 格式职责

IRIS-V1-LIBRARY-C015：`IrisValue` 是官方 Iris 二进制值格式族，用于受支持的原语、不可变值、集合和显式 `Serializable` 表示。Iris v1 要求存在这个单独指定的版本化格式，但本语言规范不定义它的字节标签、字段顺序、压缩、校验和或完整字节 schema。

IRIS-V1-LIBRARY-C016：每个 IrisValue 流 MUST 携带魔数、格式版本、兼容性元数据，以及足够的声明上下文，以便在解码任何依赖这些事实的载荷前，验证 Iris 语言主版本、Unicode 数据版本、相关的公共哈希语义版本、schema 或表示版本、名义值的包身份和 API 主版本，以及特性要求。

IRIS-V1-LIBRARY-C017：IrisValue 解码 MUST 施加严格有界的长度和计数。格式规范 MUST 为流长度、嵌套深度、元素数量、字节长度、标量长度、符号长度、包名长度、类型实参数量、schema 元数据长度和总分配量定义最大值或调用者可配置限制。解码器 MUST NOT 信任未验证的输入大小，不得在没有溢出检查时相乘大小，不得在限制验证前按声明长度分配，也不得在结构检查失败后继续读取。

IRIS-V1-LIBRARY-C018：IrisValue 支持任意 Integer、声明的 Float 宽度和位、`nil`、Bool、String、Symbol、Bytes、Tuple、Array、Hash，以及显式 `Serializable` 表示。只有当格式版本声明额外稳定且无身份的值族，并且拥有该语义的章节定义其值相等和版本约束时，它 MAY 支持这些值族。它 MUST NOT 静默编码实时 Class、Module、Method、Closure、Task、FFI、原生资源、迭代器游标，或运行时句柄身份。

IRIS-V1-LIBRARY-C019：IrisValue Hash 解码 MUST 在插入前根据当前 Type 和哈希 Contract 验证每个解码键。解码后的重复逻辑键 MUST 引发确定性解码错误，除非格式版本显式声明确定性合并规则。合并规则 MUST NOT 依赖运行时哈希桶顺序。

IRIS-V1-LIBRARY-C020：IrisValue 名义反序列化 MUST 只调用与相关 `Serializable` 表示对应的已声明标准反序列化工厂或构造器。它 MUST 验证 [05-types-contracts-generics.md](05-types-contracts-generics.md) 要求的泛型约束、[08-modules-metaprogramming.md](08-modules-metaprogramming.md) 要求的包身份，以及 [09-native-host-ffi.md](09-native-host-ffi.md) 要求的原生载荷限制。

## Encoding 与 Unicode 分工

IRIS-V1-LIBRARY-C021：Iris 语言核心保证 UTF-8 源文本、Unicode 标量 String 内容，以及 [06-collections-text-regex.md](06-collections-text-regex.md) 定义的严格 UTF-8 文本到二进制便捷转换。`String#to_bytes`、`MutableString#to_bytes`、`Bytes#to_string` 和 ByteArray 快照解码默认使用严格 UTF-8。

IRIS-V1-LIBRARY-C022：标准 `Encoding` 包拥有显式 Encoding 对象。它 MUST 至少提供 UTF-8、UTF-16LE、UTF-16BE 和 Latin-1。每个 Encoding 对象默认使用严格错误处理。替换、忽略或其他有损行为需要调用点的显式选项，并且默认 MUST NOT 选择这些行为。

IRIS-V1-LIBRARY-C023：String 保持 Unicode 标量内容，绝不把源编码、已解码文件的编码、终端编码或包编码作为隐藏状态保留。把 String 转换为 Bytes 总是需要显式目标 Encoding，核心 UTF-8 便捷 Method 除外。把 Bytes 或 ByteArray 转换为 String 总是需要严格验证，除非调用者显式选择非严格错误选项。

IRIS-V1-LIBRARY-C024：Iris 语言主版本 1 的默认 Unicode 数据是 [06-collections-text-regex.md](06-collections-text-regex.md) 固定的 Unicode 17.0.0。使用默认 Unicode 语义的 Encoding、Unicode 属性、规范化、大小写映射、casefold、JSON、IrisValue 和 Regex API MUST 使用该版本，除非它们是 Iris v1 语言默认值之外的显式版本化 API。

IRIS-V1-LIBRARY-C025：Encoding API MUST NOT 隐式选择 OS 区域设置、当前进程代码页、文件系统代码页、控制台代码页、环境变量或 Host 默认值。Host 或标准包可以把这些设置暴露为显式值，但选择它们用于解码或编码需要调用者选择。

IRIS-V1-LIBRARY-C026：下列 Encoding 分工表具有规范性：

| 表面 | 核心或标准 | 必需行为 | 显式不包含 |
| --- | --- | --- | --- |
| Source file decoding | Core language | 只允许 UTF-8 源文本 | Locale-selected source decoding |
| `String#to_bytes` and `MutableString#to_bytes` | Core text convenience | 不可变 UTF-8 Bytes 快照 | Hidden source encoding retention |
| `Bytes#to_string` and ByteArray snapshot decode | Core text convenience | 严格 UTF-8 或 `EncodingError` | Lossy default replacement |
| `Encoding::UTF_8`, `UTF_16LE`, `UTF_16BE`, `LATIN_1` | Stable standard package | 显式对象，默认严格错误 | Locale-selected fallback |
| Additional encodings | Separately versioned packages | 显式包身份和版本 | Language-core ABI promise |
| Unicode properties and normalization | Core plus standard Unicode API | 默认 Unicode 17.0.0 | Host Unicode table drift |

## 核心运行时包

IRIS-V1-LIBRARY-C027：Iris v1 核心和稳定运行时包属于语言主版本兼容性表面的一部分，但并非全都是解析器或 VM 原语。它们的公共 Contracts、值映射、诊断和语义版本化 MUST 保持 [01-language-identity.md](01-language-identity.md) 中的 Iris v1 兼容性规则。

IRIS-V1-LIBRARY-C028：下列核心与标准对照表具有规范性：

| 领域 | 核心或稳定运行时包 | 单独版本化的官方标准包 | V1 边界规则 |
| --- | --- | --- | --- |
| Object, Class, Module, Contract, Type, Method, Closure | Core | Reflection convenience packages may provide queries | 核心拥有身份和分派，便捷 API 不能改变语义 |
| ReflectionPolicy, MetaCapabilities, package metadata | Core | Tooling packages may present views | 变更仍使用核心事务和权限 |
| Collections, text, bytes, Symbol, Range, iteration | Core | Algorithms and adapters may be standard packages | 核心拥有值语义和稳定哈希 |
| Regex and Match | Core safe Regex subset | Advanced PCRE-style Regex engines | 标准引擎使用不同类型，不替换核心 Regex |
| Async Task, Awaitable, event loop, Closeable, diagnostics | Core | Extra schedulers or observability sinks | 标准包不能向核心 v1 引入取消语义 |
| IO, File, Path | Stable runtime package | Filesystem watchers, archives, globbing, shell integration | Host 权限仍支配效果 |
| Encoding and Unicode | Stable runtime package | Extra encodings and locale adapters | 核心默认值保持 UTF-8 和 Unicode 17.0.0 |
| JSON | Stable runtime package | Schema, JSON Lines, canonicalization profiles | JSON 不暗示任意对象转储 |
| IrisValue | Stable runtime package plus separate format specification | Format tooling and migration helpers | 语言规范要求版本化和限制，不要求字节标签 |
| FFI | Stable runtime package | Binding generators and platform libraries | 脚本二进制调用仍使用唯一 FFI 路径 |
| Package, manifest, lock, permissions | Core | Registries, publishing tools, update advisors | 运行时绝不隐式解析 latest |
| Testing and conformance helpers | Stable runtime package | Test frameworks and reporters | 符合性向量仍归规范所有 |
| Cryptography | Not core ABI | Official crypto packages | 内部 BLAKE3 使用不是公共加密套件 |
| HTTP and general networking protocols | Not core ABI | Official network and protocol packages | 权限名称可以存在，协议 API 是单独的 |
| Databases | Not core ABI | Official database packages | 连接是外部资源，不是可序列化实时状态 |
| GUI and platform UI | Not core ABI | Official GUI packages | 平台事件循环不能改变 Iris task 语义 |

IRIS-V1-LIBRARY-C029：必需的核心 Modules 或命名空间包括 `Object`、`Type`、`Contract`、`Reflection`、`Collections`、`Text`、`Bytes`、`Regex`、`Async`、`IO`、`File`、`Path`、`Encoding`、`Unicode`、`JSON`、`IrisValue`、`FFI`、`Package`、`Diagnostics` 和 `Testing`。符合实现 MAY 以不同方式组织文件或内部模块，但这些公共包表面 MUST 存在，或由标准包映射提供别名。

IRIS-V1-LIBRARY-C030：内部 BLAKE3 用于稳定哈希、修订制品完整性、包制品摘要或 IrisValue 完整性元数据时，MUST NOT 暗示核心运行时中存在公共加密 API。公共加密只属于单独版本化的官方标准包。

## 官方标准包与延后领域

IRIS-V1-LIBRARY-C031：核心 ABI 之外的官方标准包是单独版本化的包，带有来自 [08-modules-metaprogramming.md](08-modules-metaprogramming.md) 的包身份、API 主版本、清单、权限和兼容性规则。它们可以随实现分发，但除非本章把相关领域列为核心或稳定运行时，否则它们的 API 不是 Iris v1 语言核心 ABI。

IRIS-V1-LIBRARY-C032：下列官方标准包类别因 v1 边界目的而命名：

| 类别 | 状态 | 边界规则 |
| --- | --- | --- |
| Advanced Regex engines | `DEFERRED V1 to standard packages` | 必须使用不同于核心 Regex 的类型，并且不能改变 `/.../flags` 语义 |
| Cryptography | `DEFERRED V1 to standard packages` | BLAKE3 内部使用不产生公共核心加密套件 |
| HTTP clients and servers | `DEFERRED V1 to standard packages` | 协议 API 不是语言核心 ABI |
| General networking protocols | `DEFERRED V1 to standard packages` | 权限可以限制访问，但 API 是单独的包 |
| Databases and query clients | `DEFERRED V1 to standard packages` | 连接是 Closeable 外部资源，不是可序列化实时值 |
| GUI, graphics, and platform UI | `DEFERRED V1 to standard packages` | 平台循环必须保持 Iris 调度器语义 |
| Compression, archives, and checksums | `DEFERRED V1 to standard packages` | 不是本语言规范中 IrisValue 字节 schema 的一部分 |
| Schema languages and validation profiles | `DEFERRED V1 to standard packages` | JSON 和 IrisValue 只定义基础映射 |
| Locale, collation, and culture adapters | `DEFERRED V1 to standard packages` | 默认值不能覆盖 Unicode 17.0.0 或无 OS 依赖的 Encoding 规则 |
| Filesystem watchers and shell/process integration | `DEFERRED V1 to standard packages` | IO/File/Path 核心仍是稳定基础表面 |
| Time zones, calendars, and clocks beyond minimal diagnostics | `DEFERRED V1 to standard packages` | 序列化时间戳没有隐藏依赖 |
| Image, audio, video, and binary media codecs | `DEFERRED V1 to standard packages` | 按核心语义，Bytes 仍是无类型字节内容 |

IRIS-V1-LIBRARY-C033：延后类别中的包 MAY 定义自己的可序列化记录、权限名称、资源记录和 Encoding 适配器。它 MUST NOT 声称具有语言核心 ABI 状态，不得绕过 Host 权限授予，不得改变核心值语义，不得为无关 Class 授予隐式 JSON 或 IrisValue 资格，也不得改变本章定义的安全默认值。

IRIS-V1-LIBRARY-C034：高级 Regex 包 MUST 交叉链接到核心 Regex 规则，而不是替换它们。它们可以通过不同包 Types 和显式构造 API 提供 PCRE 风格特性、回溯控制，或 Host 引擎绑定。核心 Regex 字面量、`=~`、`!~`、标志、Match 不可变性和安全子集限制仍归 [06-collections-text-regex.md](06-collections-text-regex.md) 所有。

## 安全解码与诊断

IRIS-V1-LIBRARY-C035：每个用于 JSON、IrisValue、Encoding、包元数据或标准包记录的标准解码器 MUST 在信任大小、计数、类型身份、schema 版本或分配请求前验证完整结构。格式错误的输入失败 MUST 是普通 Iris 解码异常或结构化诊断，而不是断言失败、Host panic、进程中止、未检查分配或未定义状态。

IRIS-V1-LIBRARY-C036：安全解码诊断 MUST 标识解码器、格式或包名称、已知格式版本、可用的字节或标量偏移、可用的结构路径、违反的限制或期望 Contract，以及是否丢弃了任何部分结果。诊断 MUST NOT 包含秘密 Host 路径、原始原生指针、对象地址，或超出按策略报告失败所需输入片段的私有数据。

IRIS-V1-LIBRARY-C037：历史 `.irc` 危险模式是负面证据，其中二进制输入在没有魔数、版本、大小、读取或分配检查时被信任。Iris v1 序列化和包格式 MUST 通过要求版本元数据、有界大小、成功读取，以及分配或发布前验证来拒绝该模式。

## 示例与符合性向量

IRIS-V1-LIBRARY-EX001：资料性示例，显式 Serializable 表示：

```iris
class User for Serializable {
  property name: String { get; }
  property age: Integer { get; }

  impl fun to_serializable() -> Hash<String,Object> {
    %{ "schema": 1, "name": name, "age": age }
  }
}
```

IRIS-V1-LIBRARY-EX002：资料性示例，JSON 编码拒绝隐式对象转储：

```iris
let task = read_async()
JSON.encode(task)        // error, Task is not a JSON value
JSON.encode(task.to_string())
```

IRIS-V1-LIBRARY-C038：下列向量表具有规范性。符合性章节 MUST 保留这些向量 ID，或把它们映射到具有相同可观察结果的机器可读记录：

| 向量 ID | 类别 | 适用性 | 源/输入 | 预期可观察结果 | 决策 |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-LIBRARY-V001` | negative | 需要解释器；需要 JIT；native 不适用；JSON 编码器表面 | `JSON.encode(Plain.new())`, where `Plain` does not declare `for Serializable` and has raw ivar `@secret` plus public property `name`. | 引发 `SerializationError`；没有发出字节；观察不到原始 ivar、属性值、对象 ID、Method 或反射元数据；条款 `IRIS-V1-LIBRARY-C003`、`IRIS-V1-LIBRARY-C004` 和 `IRIS-V1-LIBRARY-C011` 适用。 | `D-502` |
| `IRIS-V1-LIBRARY-V002` | positive | 需要解释器；需要 JIT；native 不适用；JSON 编码器表面 | `JSON.encode(User.new("Ada", 37), canonical: true)`, where `User for Serializable` returns `%{ "schema": 1, "name": "Ada", "age": 37 }`. | 在规范排序下返回精确 UTF-8 String `{"age":37,"name":"Ada","schema":1}`；不发出非表示状态；条款 `IRIS-V1-LIBRARY-C004`、`IRIS-V1-LIBRARY-C005` 和 `IRIS-V1-LIBRARY-C011` 适用。 | `D-502` |
| `IRIS-V1-LIBRARY-V003` | negative | 需要解释器；需要 JIT；native 不适用；JSON 解码器表面 | `JSON.decode(nested_array_depth_4, limits: { depth: 3 })`, where the input is valid UTF-8 JSON and no caller schema is supplied. | 在分配第四层嵌套 Array 前引发 `JSONLimitError`；不发布部分值；条款 `IRIS-V1-LIBRARY-C013` 和 `IRIS-V1-LIBRARY-C035` 适用。 | `D-502` |
| `IRIS-V1-LIBRARY-V004` | negative | 需要解释器；需要 JIT；native 不适用；JSON Bytes 输入表面 | `JSON.decode(Bytes[0xc3,0x28])`. | 在 JSON token 解释前引发 `EncodingError`，并且不发布 JSON 值；条款 `IRIS-V1-LIBRARY-C012` 和 `IRIS-V1-LIBRARY-C021` 适用。 | `D-413`, `D-414` |
| `IRIS-V1-LIBRARY-V005` | negative | 需要解释器；需要 JIT；native 不适用；IrisValue 头表面 | Decode IrisValue fixture `bad-header` with invalid magic `49525630` and fixture `unsupported-version` with format version `2`. | 每个 fixture 都在载荷分配前以状态 `IRISVALUE_INCOMPATIBLE_HEADER` 失败；条款 `IRIS-V1-LIBRARY-C015`、`IRIS-V1-LIBRARY-C016` 和 `IRIS-V1-LIBRARY-C017` 适用。 | `D-503` |
| `IRIS-V1-LIBRARY-V006` | negative | 需要解释器；需要 JIT；native 不适用；IrisValue 长度表面 | Decode IrisValue fixture `array-count-over-limit`, where declared element count is one above configured limit `1024`. | 在容器分配前以状态 `IRISVALUE_LIMIT_OR_STRUCTURE` 失败；不发布部分 Array；条款 `IRIS-V1-LIBRARY-C017` 和 `IRIS-V1-LIBRARY-C035` 适用。 | `D-503` |
| `IRIS-V1-LIBRARY-V007` | positive | 需要解释器；需要 JIT；native 不适用；IrisValue 受支持值表面 | Decode then encode IrisValue fixture `roundtrip-core-values` containing arbitrary Integer `18446744073709551617`, String `"é"`, Bytes hex `00ff`, Tuple `(1,"x")`, Array `[1,2]`, and Hash entries `{(:a,1),(:b,2)}`. | 值按相等的 Iris 值语义往返；Hash 比较按无序条目；语言章节不主张字节标签或字段顺序；条款 `IRIS-V1-LIBRARY-C015` 和 `IRIS-V1-LIBRARY-C018` 适用。 | `D-503` |
| `IRIS-V1-LIBRARY-V008` | negative | 需要解释器；需要 JIT；需要 native；IrisValue 编码器表面 | Attempt default IrisValue encoding of an `FFI::Library` returned by `FFI.open`, an open `File`, and a native payload object. | 每个调用都引发 `SerializationError`；不发出指针、运行时身份、文件描述符、原生载荷字节或实时资源状态；条款 `IRIS-V1-LIBRARY-C003` 和 `IRIS-V1-LIBRARY-C018` 适用。 | `D-502`, `D-503` |
| `IRIS-V1-LIBRARY-V009` | negative | 需要解释器；需要 JIT；native 不适用；Encoding 严格 UTF-8 表面 | `Encoding::UTF_8.decode(Bytes[0xc3,0x28])`. | 在严格默认行为下引发 `EncodingError`；条款 `IRIS-V1-LIBRARY-C021` 和 `IRIS-V1-LIBRARY-C022` 适用。 | `D-413`, `D-414` |
| `IRIS-V1-LIBRARY-V010` | positive | 需要解释器；需要 JIT；native 不适用；Encoding 显式错误选项表面 | `Encoding::UTF_8.decode(Bytes[0xc3,0x28], errors: :replace)`. | 返回 String `"�("`；替换行为只因为调用选择了 `errors: :replace` 而发生；条款 `IRIS-V1-LIBRARY-C022` 和 `IRIS-V1-LIBRARY-C023` 适用。 | `D-414` |
| `IRIS-V1-LIBRARY-V011` | negative | 需要解释器；需要 JIT；native 不适用；Encoding 选择表面 | Call `File.read_text("input.txt", encoding: :host_default)` or equivalent implicit host-default Encoding selection in conforming mode. | 调用被诊断 `ENCODING_EXPLICIT_REQUIRED` 拒绝，或 API 在解码前要求显式 Encoding 对象；条款 `IRIS-V1-LIBRARY-C025` 和 `IRIS-V1-LIBRARY-C026` 适用。 | `D-414` |
| `IRIS-V1-LIBRARY-V012` | positive | 需要解释器；需要 JIT；native 不适用；包元数据和 Regex 分派表面 | Package map contains core `Regex` and separately versioned `std/regex-pcre@2` exposing Type `PcreRegex`; source evaluates `"abc" =~ /a+/` and separately constructs `PcreRegex.new("(?<=a)b")`. | 核心 `=~` 使用核心 Regex，并返回核心 Match 或 `nil`；`PcreRegex` 是不同类型，只能通过显式包 API 使用；条款 `IRIS-V1-LIBRARY-C028` 和 `IRIS-V1-LIBRARY-C034` 适用。 | `D-504`, `D-505`, `D-506` |
| `IRIS-V1-LIBRARY-V013` | negative | 需要解释器；JIT 不适用；native 不适用；包元数据和 Regex 字面量边界 | Package fixture `std/regex-pcre@2` declares `replaces_core_regex_literals: true` and attempts to bind `/.../flags` to `PcreRegex`. | 包验证以状态 `PACKAGE_CORE_ABI_CLAIM` 拒绝替换声明；核心字面量语义保持不变；条款 `IRIS-V1-LIBRARY-C028` 和 `IRIS-V1-LIBRARY-C034` 适用。 | `D-504`, `D-505`, `D-506` |
| `IRIS-V1-LIBRARY-V014` | negative | 需要解释器；JIT 不适用；native 不适用；包元数据边界 | Package fixtures `std/http@1 { core_abi: true }` and `std/crypto@1 { core_abi: true }` are validated against the v1 package map. | 每个 fixture 都以状态 `PACKAGE_CORE_ABI_CLAIM` 被拒绝；没有包作为语言核心 ABI 发布；条款 `IRIS-V1-LIBRARY-C028`、`IRIS-V1-LIBRARY-C030` 和 `IRIS-V1-LIBRARY-C032` 适用。 | `D-504` |

## 可追踪性说明

IRIS-V1-LIBRARY-C039：本章拥有序列化和库边界决策 D-502 到 D-504。它为 Encoding 和严格 UTF-8 转换细化 D-413 和 D-414，通过 [06-collections-text-regex.md](06-collections-text-regex.md) 使用修订后的 D-407 以支持 Unicode 17.0.0，并交叉链接 D-505 和 D-506，而不是重新定义 Regex 语义。

IRIS-V1-LIBRARY-C040：本章拥有的决策 ID 是 `D-502`、`D-503` 和 `D-504`。

IRIS-V1-LIBRARY-C041：引用但不拥有的决策 ID 包括 `D-407`、`D-413`、`D-414`、`D-505` 和 `D-506`。
