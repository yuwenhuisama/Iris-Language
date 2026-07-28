(function () {
  "use strict";

  const STORAGE_LANG = "iris-site-lang";
  const STORAGE_THEME = "iris-site-theme";
  const SPEC_BASE = "spec/iris-v1";
  const TUTORIAL_BASE = "tutorial";
  const REPO_URL = "https://github.com/yuwenhuisama/Iris-Language";
  // Deep section links use #/<collection>/<lang>/<file-stem>?h=<heading-id>. Rendered headings receive stable ordinal IDs so EN/ZH toggles preserve position; text slugs remain as fallbacks for incoming Markdown/GitHub-style anchors.

  const chapters = [
    { file: "README.md", stem: "README", en: "Specification Index", zh: "规范索引", purposeEn: "Normative document index, editorial contract, terminology, status, and reading order.", purposeZh: "规范性文档索引、编辑契约、术语、状态和阅读顺序。" },
    { file: "01-language-identity.md", stem: "01-language-identity", en: "Language Identity", zh: "语言身份", purposeEn: "Goals, compatibility identity, normative conventions, versioning, and v1 deferrals.", purposeZh: "目标、兼容性身份、规范性约定、版本控制和 v1 延后项。" },
    { file: "02-lexical-grammar.md", stem: "02-lexical-grammar", en: "Lexical Grammar", zh: "词法语法", purposeEn: "Source encoding, tokens, literals, keywords, precedence, associativity, and consolidated grammar.", purposeZh: "源编码、词元、字面量、关键字、优先级、结合性和合并语法。" },
    { file: "03-runtime-object-model.md", stem: "03-runtime-object-model", en: "Runtime Object Model", zh: "运行时对象模型", purposeEn: "Values, identity, equality/hash, Class/Module/Contract, MRO, revisions, Methods, properties, and built-ins.", purposeZh: "值、身份、相等性/哈希、Class/Module/Contract、MRO、修订、Method、属性和内建项。" },
    { file: "04-bindings-callables-control-flow.md", stem: "04-bindings-callables-control-flow", en: "Bindings, Callables, And Control Flow", zh: "绑定、可调用与控制流", purposeEn: "Bindings, scopes, callable forms, parameters, Closure capture, calls, assignments, conditionals, loops, match, and exceptions.", purposeZh: "绑定、作用域、可调用形式、参数、Closure 捕获、调用、赋值、条件、循环、match 和异常。" },
    { file: "05-types-contracts-generics.md", stem: "05-types-contracts-generics", en: "Types, Contracts, And Generics", zh: "类型、Contracts 与泛型", purposeEn: "Gradual contracts, Dynamic, union and intersection, nilability, casts, inference, generics, Contract views, and Never.", purposeZh: "渐进 Contract、Dynamic、并集与交集、可 nil 性、转换、推断、泛型、Contract 视图和 Never。" },
    { file: "06-collections-text-regex.md", stem: "06-collections-text-regex", en: "Collections, Text, Binary, Regex, And Stable Hashing", zh: "集合、文本、二进制、Regex 与稳定哈希", purposeEn: "Tuple, Array, Hash, Range, iteration, String, MutableString, Symbol, Bytes, ByteArray, Regex, and hashing boundaries.", purposeZh: "Tuple、Array、Hash、Range、迭代、String、MutableString、Symbol、Bytes、ByteArray、Regex 和哈希边界。" },
    { file: "07-async-resources-diagnostics.md", stem: "07-async-resources-diagnostics", en: "Async, Resources, And Diagnostics", zh: "Async、Resources 与 Diagnostics", purposeEn: "Task/Awaitable, single-thread scheduler, Closeable/using, ExceptionContext, stack/cause/suppressed, and diagnostic streams.", purposeZh: "Task/Awaitable、单线程调度器、Closeable/using、ExceptionContext、stack/cause/suppressed 和诊断流。" },
    { file: "08-modules-metaprogramming.md", stem: "08-modules-metaprogramming", en: "Modules And Metaprogramming", zh: "模块与元编程", purposeEn: "Packages, modules, import/export, executable declaration bodies, open transactions, revisions, decorators, MetaCapabilities, and ReflectionPolicy.", purposeZh: "包、模块、import/export、可执行声明体、open 事务、修订、装饰器、MetaCapabilities 和 ReflectionPolicy。" },
    { file: "09-native-host-ffi.md", stem: "09-native-host-ffi", en: "Native Host And FFI", zh: "原生 Host 与 FFI", purposeEn: "Stable C Host/extension ABI boundary, handles, GC/native payload, async completion, FFI script API, and Rust/C++ integration rules.", purposeZh: "稳定 C Host/扩展 ABI 边界、句柄、GC/native 载荷、异步完成、FFI 脚本 API 和 Rust/C++ 集成规则。" },
    { file: "10-serialization-standard-library.md", stem: "10-serialization-standard-library", en: "Serialization And Standard Library Boundary", zh: "序列化与标准库边界", purposeEn: "JSON/IrisValue scope, Encoding API, core versus standard packages, and deferred library areas.", purposeZh: "JSON/IrisValue 范围、Encoding API、核心包与标准包，以及延后的库领域。" },
    { file: "11-migration-divergence.md", stem: "11-migration-divergence", en: "Migration And Divergence Ledger", zh: "迁移与差异台账", purposeEn: "Neutral ledger of intentional divergences and migration examples recorded by the specification.", purposeZh: "规范记录的有意差异与迁移示例的中性台账。" },
    { file: "12-conformance.md", stem: "12-conformance", en: "Conformance Framework", zh: "一致性框架", purposeEn: "Example/vector ID schema, positive/negative/diagnostic/differential coverage, and freeze criteria.", purposeZh: "示例/向量 ID 模式、正向/负向/诊断/差异覆盖和冻结标准。" },
    { file: "traceability-matrix.md", stem: "traceability-matrix", en: "Traceability Matrix", zh: "可追溯性矩阵", purposeEn: "Every frozen D-ID mapped to exact normative clause(s), examples/vectors, divergence entry, or explicit deferred status.", purposeZh: "每个冻结 D-ID 映射到精确的规范性条款、示例/向量、差异条目或明确延后状态。" }
  ];

  const tutorialChapters = [
    { file: "README.md", stem: "README", en: "Tutorial index", zh: "教程索引", purposeEn: "Start here for the guided reading path before the normative specification.", purposeZh: "从这里开始，先按引导路径学习，再进入规范性文档。" },
    { file: "01-getting-started.md", stem: "01-getting-started", en: "What Iris Is", zh: "Iris 是什么", purposeEn: "Meet the language goals, current release status, and the shape of a first Iris program.", purposeZh: "了解语言目标、当前发布状态，以及第一个 Iris 程序的样貌。" },
    { file: "02-values-and-bindings.md", stem: "02-values-and-bindings", en: "Values and Bindings", zh: "值与绑定", purposeEn: "Learn object values, names, mutability, and the basic expression model.", purposeZh: "学习对象值、名称、可变性和基本表达式模型。" },
    { file: "03-control-flow.md", stem: "03-control-flow", en: "Control Flow", zh: "控制流", purposeEn: "Walk through conditionals, loops, matching, and the way Iris code branches.", purposeZh: "了解条件、循环、match，以及 Iris 代码如何分支。" },
    { file: "04-callables-and-closures.md", stem: "04-callables-and-closures", en: "Functions, Closures, Blocks", zh: "函数、闭包与块", purposeEn: "Use callable forms, captured state, parameters, and trailing blocks.", purposeZh: "使用可调用形式、捕获状态、参数和 trailing block。" },
    { file: "05-classes-and-objects.md", stem: "05-classes-and-objects", en: "Classes and Objects", zh: "类与对象", purposeEn: "Build objects with Classes, Methods, properties, and dynamic revision in mind.", purposeZh: "围绕 Class、Method、属性和动态修订来构建对象。" },
    { file: "06-modules-and-contracts.md", stem: "06-modules-and-contracts", en: "Modules and Contracts", zh: "模块与 Contract", purposeEn: "Compose behavior with Modules and express promises through Contracts.", purposeZh: "用 Module 组合行为，并用 Contract 表达承诺。" },
    { file: "07-types-and-generics.md", stem: "07-types-and-generics", en: "Gradual Types", zh: "渐进类型", purposeEn: "Understand annotations, Dynamic, generic constraints, and runtime-checked views.", purposeZh: "理解标注、Dynamic、泛型约束和运行时检查视图。" },
    { file: "08-errors-and-resources.md", stem: "08-errors-and-resources", en: "Errors and Resources", zh: "错误与资源", purposeEn: "Handle failures, cleanup, diagnostics, and resource lifetime patterns.", purposeZh: "处理失败、清理、诊断和资源生命周期模式。" },
    { file: "09-dynamic-and-static.md", stem: "09-dynamic-and-static", en: "Dynamic Meets Static", zh: "动态与静态的交汇", purposeEn: "Tie dynamic behavior back to the promises static consumers can rely on.", purposeZh: "把动态行为重新连接到静态消费者可以依赖的承诺。" },
    { file: "10-where-to-next.md", stem: "10-where-to-next", en: "Where To Next", zh: "下一步", purposeEn: "Move from tutorial knowledge into the specification, conformance, and implementation work.", purposeZh: "从教程知识进入规范、一致性和实现工作。" }
  ];

  const strings = {
    en: {
      nav: { tutorial: "Tutorial", spec: "Specification", github: "GitHub", chapters: "Chapters" },
      themeLight: "Light", themeDark: "Dark",
      heroKicker: "Frozen semantics preview",
      heroTitleA: "Iris",
      heroTitleB: "dynamic where it moves, promised where it matters.",
      heroLede: "Iris Programming Language is a new object-oriented scripting language with first-class dynamic behavior bounded by durable static promises. Every value is an object, operators are runtime-dispatched messages, composition blends single Class inheritance with Modules and Contracts, and host embedding is framed by a stable C ABI.",
      readTutorial: "Start the tutorial",
      readSpec: "Read the spec",
      viewGithub: "View GitHub",
      statFiles: "spec artifacts",
      statLangs: "languages",
      statImpl: "reference implementation",
      statusTitle: "Frozen draft, not a toolchain release",
      statusBody: "The language semantics are frozen for specification review, but no reference implementation, compiler, package manager, playground, or downloadable runtime exists yet.",
      statusCta: "Implementation status",
      featuresTitle: "Language Surface",
      positioningKicker: "Design intent, not a spec guarantee",
      positioningTitle: "What Iris Is For",
      positioningLede: "This section describes project direction, not shipped behavior: Iris is intended to fit both host embedding and standalone application work, while keeping capabilities outside the language core in versioned extensions.",
      dualityKicker: "IRIS-V1-IDENTITY-C013",
      dualityTitle: "Dynamic Behavior, Static Promises",
      dualityLede: "The defining Iris Programming Language design contract is not static versus dynamic. It is dynamic evolution held inside promises that compiled code, reflection, packages, native hosts, and conformance can trust.",
      dualityDynamic: "May change dynamically",
      dualityPromised: "Remains promised",
      dualityExample: "Example",
      dualitySource: "Read the normative clause",
      featuresLede: "With the static/dynamic contract established, the remaining surface shows how Iris code sends messages, composes behavior, handles control flow, and crosses host boundaries.",
      tutorialTitle: "Tutorial",
      tutorialKicker: "Recommended start",
      tutorialLede: "The complete guided tutorial is live: an index plus ten chapters, fully written in English and fully translated into Simplified Chinese, from first concepts through the path into the frozen specification.",
      specTitle: "Specification Index",
      specLede: "All chapters are rendered from the checked-in Markdown at runtime. English and Simplified Chinese files share the same filenames, so links preserve chapter position across languages.",
      footer: "Iris Programming Language draft specification preview. Source repository:",
      readerToc: "On this page",
      readerChapters: "Chapters",
      loading: "Loading document...",
      specLabel: "Specification",
      tutorialLabel: "Tutorial",
      translationNoticeTitle: "Chinese tutorial translation pending",
      translationNoticeBody: "This tutorial chapter is not translated yet, so the English source is shown for now. The language toggle and route stay on Chinese and will use the translation automatically when it lands.",
      errorTitle: "Document fetch failed",
      errorHelp: "Serve the repository root over HTTP and check that the Markdown file exists at the relative path shown below."
    },
    zh: {
      nav: { tutorial: "教程", spec: "规范", github: "GitHub", chapters: "章节" },
      themeLight: "亮色", themeDark: "暗色",
      heroKicker: "冻结语义预览",
      heroTitleA: "Iris",
      heroTitleB: "让动态能力落在静态边界之内。",
      heroLede: "Iris Programming Language 是一门新的面向对象脚本语言：动态行为是一等能力，但始终受持久静态承诺约束。每个值都是对象，运算符是运行时动态派发的消息，组合使用单一 Class 继承加 Modules 与 Contracts，并以稳定 C ABI 定义 Host 嵌入边界。",
      readTutorial: "开始教程",
      readSpec: "阅读规范",
      viewGithub: "查看 GitHub",
      statFiles: "规范产物",
      statLangs: "语言版本",
      statImpl: "参考实现",
      statusTitle: "冻结草案，不是工具链发布",
      statusBody: "语言语义已冻结用于规范审阅，但尚无参考实现、编译器、包管理器、playground 或可下载运行时。",
      statusCta: "实现状态",
      featuresTitle: "语言表面",
      positioningKicker: "设计意图，不是规范保证",
      positioningTitle: "Iris 适合做什么",
      positioningLede: "本节说明项目方向，而不是已经交付的行为：Iris 的目标是在 Host 嵌入和独立应用编写之间取得平衡，同时把语言核心之外的能力放进独立版本化的扩展。",
      dualityKicker: "IRIS-V1-IDENTITY-C013",
      dualityTitle: "动态行为，静态承诺",
      dualityLede: "Iris 的关键设计契约不是静态对动态，而是把真实的动态演进放进可被编译代码、反射、包、原生 Host 和一致性信任的承诺边界中。",
      dualityDynamic: "可以动态变化",
      dualityPromised: "仍保持承诺",
      dualityExample: "示例",
      dualitySource: "阅读规范条款",
      featuresLede: "在静态/动态契约之外，这些特性展示 Iris 代码如何发送消息、组合行为、表达控制流并跨越 Host 边界。",
      tutorialTitle: "教程",
      tutorialKicker: "推荐起点",
      tutorialLede: "完整的引导式教程已经上线：包含索引和十章正文，英文已完整写成，简体中文也已完整翻译，覆盖从初始概念到进入冻结规范的路径。",
      specTitle: "规范索引",
      specLede: "所有章节都在运行时从仓库中的 Markdown 渲染。英文与简体中文目录使用相同文件名，因此切换语言会保留章节位置。",
      footer: "Iris Programming Language 草案规范预览。源码仓库：",
      readerToc: "本页目录",
      readerChapters: "章节",
      loading: "正在加载文档...",
      specLabel: "规范",
      tutorialLabel: "教程",
      translationNoticeTitle: "中文教程翻译待完成",
      translationNoticeBody: "本教程章节尚未翻译，因此暂时显示英文源文。语言切换和路由会保留中文；翻译文件发布后会自动使用中文版本。",
      errorTitle: "文档获取失败",
      errorHelp: "请通过 HTTP 服务仓库根目录，并检查下方相对路径中的 Markdown 文件是否存在。"
    }
  };

  const features = [
    ["Every Value Is An Object", "每个值都是对象", "Objecthood governs message protocols, equality, hashing, truthiness, and exception raising.", "对象性统领消息协议、相等性、哈希、truthiness 与异常抛出。"],
    ["Operators Are Messages", "运算符是消息", "Symbolic operators dispatch as ordinary Methods on the receiver, not through a global promotion table.", "符号运算符作为接收者上的普通 Method 派发，而不是通过全局提升表。"],
    ["Classes, Modules, Contracts", "Classes、Modules、Contracts", "Composition uses one Class superclass, Module behavior mixins, and explicit nominal Contract conformance.", "组合使用一个 Class 超类、Module 行为 mixin，以及显式名义 Contract 符合。"],
    ["Contract-Qualified Dispatch", "Contract 限定派发", "Explicit `..` calls choose Contract slot identity without creating overload dispatch by static type.", "显式 `..` 调用选择 Contract 槽位身份，而不会按静态类型创建重载派发。"],
    ["Closures And Trailing Blocks", "Closure 与 trailing block", "Lexical Closures capture binding cells and receiver relation; trailing blocks use a dedicated block channel.", "词法 Closure 捕获绑定 cell 与接收者关系；trailing block 使用专用 block 通道。"],
    ["Single-Thread Async And Resources", "单线程 Async 与资源", "Task/Awaitable scheduling, ordinary using/Closeable cleanup, and structured diagnostics share the object model.", "Task/Awaitable 调度、普通 using/Closeable 清理与结构化诊断共享对象模型。"],
    ["Stable C Host ABI", "稳定 C Host ABI", "Embedding and native extension are specified around opaque handles, explicit signatures, and runtime-thread affinity.", "嵌入与原生扩展围绕不透明句柄、显式签名和运行时线程亲和性来定义。"]
  ];

  const positioningCards = [
    {
      en: "Embeddable by direction",
      zh: "面向嵌入的方向",
      bodyEn: "The intent is for Iris to drop into a native application as its scripting layer. The frozen FFI chapter backs the boundary shape with a stable C ABI, opaque value handles, runtime-thread affinity, thread-safe posting, and no foreign unwinding across the boundary.",
      bodyZh: "Iris 的设计方向，是能放进已有原生应用中作为脚本层。冻结的 FFI 章节已经支撑了边界形状：稳定 C ABI、不透明值句柄、运行时线程亲和、线程安全投递，以及禁止 foreign unwinding 跨过边界。",
      links: [
        { id: "IRIS-V1-FFI-C003", stem: "09-native-host-ffi" },
        { id: "IRIS-V1-FFI-C007", stem: "09-native-host-ffi" },
        { id: "IRIS-V1-FFI-C012", stem: "09-native-host-ffi" },
        { id: "IRIS-V1-FFI-C013", stem: "09-native-host-ffi" },
        { id: "IRIS-V1-FFI-C019", stem: "09-native-host-ffi" }
      ]
    },
    {
      en: "Standalone by intent",
      zh: "也面向独立使用",
      bodyEn: "Iris is not positioned only as a guest language. The project direction keeps room for writing applications directly in Iris once an implementation and supporting libraries exist.",
      bodyZh: "Iris 并不只被定位为宿主语言中的客体。项目方向也为未来在实现和支撑库存在之后，直接用 Iris 编写应用保留空间。",
      links: []
    },
    {
      en: "Small core, versioned extensions",
      zh: "小核心，版本化扩展",
      bodyEn: "The language core stays narrow. Standard-library growth, networking, databases, GUI, cryptography, advanced Regex, and HTTP are deferred to standard packages, using package identity, API-major identity, and SemVer range machinery.",
      bodyZh: "语言核心保持收窄。标准库增长、网络、数据库、GUI、密码学、高级 Regex 与 HTTP 都延后到标准包，并使用 package_id、api_major 与 SemVer 范围机制。",
      links: [
        { id: "chapter 10 deferred packages", stem: "10-serialization-standard-library" },
        { id: "package_id", stem: "08-modules-metaprogramming" },
        { id: "api_major", stem: "08-modules-metaprogramming" },
        { id: "SemVer ranges", stem: "08-modules-metaprogramming" }
      ]
    }
  ];

  const dualityLayers = [
    {
      en: "Objects and Methods",
      zh: "对象和 Method",
      links: ["03-runtime-object-model", "04-bindings-callables-control-flow"],
      dynamicEn: "Compatible Method bodies, non-contract members, property accessors, Module composition, class-object state, policy-allowed metadata.",
      dynamicZh: "兼容 Method 体、非 Contract 成员、属性访问器、Module 组合、类对象状态、策略允许的元数据。",
      promisedEn: "Value objecthood, selector identity, Method identity rules, declared Class or Contract promises, identity classification.",
      promisedZh: "值对象性、选择器身份、Method 身份规则、声明的 Class 或 Contract 承诺、身份分类。",
      exampleStem: "03-runtime-object-model",
      exampleFile: "spec/iris-v1/03-runtime-object-model.md",
      exampleLines: [
        "let view = parser as ParserContract",
        "parser.process(input)     // ordinary selector process",
        "view.process(input)       // still ordinary selector process on the receiver",
        "view..process(input)      // qualified Contract slot ParserContract::process",
        ""
      ]
    },
    {
      en: "Types and Contracts",
      zh: "类型和 Contract",
      links: ["05-types-contracts-generics"],
      dynamicEn: "Runtime-checked views, Dynamic sends inside bounds, compatible implementations, reified generic materialization.",
      dynamicZh: "运行时检查视图、边界内的 Dynamic 发送、兼容实现、具象化泛型实例化。",
      promisedEn: "No overload, invariant generics, Contract slot identity, declared requirements, `Dynamic<T>` boundary checks, `Never` flow meaning.",
      promisedZh: "无重载、不变泛型、Contract 槽位身份、声明要求、`Dynamic<T>` 边界检查、`Never` 流意义。",
      exampleStem: "05-types-contracts-generics",
      exampleFile: "spec/iris-v1/05-types-contracts-generics.md",
      exampleLines: [
        "contract Printable<T> where T: Object {",
        "  fun print(value: T) -> String",
        "}",
        "",
        "class User for Printable<User> {",
        "  impl fun print(value: User) -> String {",
        "    value.name",
        "  }",
        "}",
        "",
        "let view = User.new() as Printable<User>",
        "view..print(User.new())",
        ""
      ]
    },
    {
      en: "Metaprogramming",
      zh: "元编程",
      links: ["08-modules-metaprogramming"],
      dynamicEn: "Candidate Class or Module revisions, open transactions, decorators, reflection-visible dynamic members.",
      dynamicZh: "候选 Class 或 Module 修订、open 事务、装饰器、反射可见动态成员。",
      promisedEn: "Static spine, MetaCapabilities, ReflectionPolicy, synchronous non-escaping transaction boundaries, atomic publish or rollback.",
      promisedZh: "静态脊柱、MetaCapabilities、ReflectionPolicy、同步非逃逸事务边界、原子发布或回滚。",
      exampleStem: "08-modules-metaprogramming",
      exampleFile: "spec/iris-v1/08-modules-metaprogramming.md",
      exampleLines: [
        "class Tool meta deny superclass, native {",
        "  let enabled = load_config().enabled?",
        "",
        "  if enabled {",
        "    self.define_method(:debug) { |self: Tool| -> String; \"debug\" }",
        "  }",
        "",
        "  public fun run() -> String { \"run\" }",
        "}",
        ""
      ]
    },
    {
      en: "Packages",
      zh: "包",
      links: ["08-modules-metaprogramming"],
      dynamicEn: "Same-major compatible hot upgrade through explicit transaction.",
      dynamicZh: "通过显式事务进行同 major 兼容热升级。",
      promisedEn: "Package ID, API major identity, one active implementation per API major, stable nominal Type identity, old-revision retention while referenced.",
      promisedZh: "包 ID、API major 身份、每个 API major 一个活动实现、稳定名义 Type 身份、被引用时保留旧修订。",
      exampleStem: "08-modules-metaprogramming",
      exampleFile: "spec/iris-v1/08-modules-metaprogramming.md",
      exampleLines: [
        "import com.example.core::Text",
        "from com.example.extra::TraceExtension import Traceable as TraceableExt",
        "",
        "export module App {",
        "  fun run() -> Nil {",
        "    log(\"ready\")",
        "  }",
        "}",
        "",
        "export open class Text::Buffer {",
        "  public fun traced() -> String { inspect() }",
        "}",
        ""
      ]
    },
    {
      en: "Native and FFI",
      zh: "原生和 FFI",
      links: ["09-native-host-ffi"],
      dynamicEn: "Extension implementation behind metadata, script-bound FFI library Methods, async completion through runtime posts.",
      dynamicZh: "元数据背后的扩展实现、脚本绑定的 FFI 库 Method、通过运行时投递完成异步。",
      promisedEn: "Stable C ABI boundary, opaque handles, runtime-thread affinity, explicit signatures, no raw managed pointer escape.",
      promisedZh: "稳定 C ABI 边界、不透明句柄、运行时线程亲和性、显式签名、不泄漏原始托管指针。",
      exampleStem: "09-native-host-ffi",
      exampleFile: "spec/iris-v1/09-native-host-ffi.md",
      exampleLines: [
        "let library = FFI.open(\"mathlib\", declarations: \"mathlib.ffi\")",
        "let value = library.hypot(3.0f64, 4.0f64)",
        "library.close()",
        ""
      ]
    }
  ];

  const collections = {
    spec: {
      view: "spec",
      base: SPEC_BASE,
      labelKey: "specLabel",
      chapters,
      contentId: "reader-content",
      sidebarId: "reader-sidebar",
      getPath(chapter, lang) {
        return `${SPEC_BASE}/${lang === "zh" ? "zh-cn/" : ""}${chapter.file}`;
      }
    },
    tutorial: {
      view: "tutorial",
      base: TUTORIAL_BASE,
      labelKey: "tutorialLabel",
      chapters: tutorialChapters,
      contentId: "reader-content",
      sidebarId: "reader-sidebar",
      getPath(chapter, lang) {
        return `${TUTORIAL_BASE}/${lang === "zh" ? "zh-cn" : "en"}/${chapter.file}`;
      },
      getFallbackPath(chapter, lang) {
        return lang === "zh" ? `${TUTORIAL_BASE}/en/${chapter.file}` : "";
      }
    }
  };

  const heroCode = [
    "let klass = Counter",
    "let first = Counter.new()",
    "",
    "open class Counter {",
    "  override fun value() -> Integer { 2 }",
    "}",
    "",
    "klass same? Counter        // true, reopen kept the logical Class identity",
    "first.value()             // later send uses the current active revision",
    ""
  ].join("\r\n");

  let currentLang = normalizeLang(localStorage.getItem(STORAGE_LANG) || "en");
  let currentTheme = localStorage.getItem(STORAGE_THEME) || "dark";
  let activeSectionObserver = null;

  const main = document.getElementById("main");
  const langToggle = document.getElementById("lang-toggle");
  const themeToggle = document.getElementById("theme-toggle");
  const sidebarToggle = document.getElementById("sidebar-toggle");

  function normalizeLang(value) {
    return value === "zh" || value === "zh-cn" || value === "zh-CN" ? "zh" : "en";
  }

  function t(path) {
    return path.split(".").reduce((value, key) => value && value[key], strings[currentLang]) || path;
  }

  function setLanguage(lang, options = {}) {
    currentLang = normalizeLang(lang);
    localStorage.setItem(STORAGE_LANG, currentLang);
    document.documentElement.lang = currentLang === "zh" ? "zh-CN" : "en";
    langToggle.textContent = currentLang === "zh" ? "中文" : "EN";
    updateStaticLabels();
    if (!options.silent) {
      const route = parseRoute();
      if (collections[route.view]) {
        const anchor = visibleHeadingId() || route.anchor || "";
        navigateReader(route.view, currentLang, route.stem, anchor, true);
      } else {
        renderLanding();
      }
    }
  }

  function setTheme(theme) {
    currentTheme = theme === "light" ? "light" : "dark";
    localStorage.setItem(STORAGE_THEME, currentTheme);
    document.documentElement.dataset.theme = currentTheme;
    themeToggle.textContent = currentTheme === "dark" ? t("themeLight") : t("themeDark");
    themeToggle.setAttribute("aria-label", currentTheme === "dark" ? "Switch to light theme" : "Switch to dark theme");
  }

  function updateStaticLabels() {
    document.querySelectorAll("[data-i18n]").forEach((node) => {
      node.textContent = t(node.dataset.i18n);
    });
    document.querySelector('[data-nav="tutorial"]')?.setAttribute("href", `#/tutorial/${currentLang}/README`);
    document.querySelector('[data-nav="spec"]')?.setAttribute("href", `#/spec/${currentLang}/README`);
    themeToggle.textContent = currentTheme === "dark" ? t("themeLight") : t("themeDark");
  }

  function parseRoute() {
    const rawHash = window.location.hash || "#/";
    const [pathPart, queryString = ""] = rawHash.slice(1).split("?");
    const params = new URLSearchParams(queryString);
    const parts = pathPart.split("/").filter(Boolean);
    if (collections[parts[0]]) {
      return { view: parts[0], lang: normalizeLang(parts[1]), stem: parts[2] || "README", anchor: params.get("h") || "" };
    }
    return { view: "landing" };
  }

  function navigateSpec(lang, stem, anchor, replace) {
    navigateReader("spec", lang, stem, anchor, replace);
  }

  function navigateReader(view, lang, stem, anchor, replace) {
    const collection = collections[view] || collections.spec;
    const target = `#/${collection.view}/${normalizeLang(lang)}/${stem}${anchor ? `?h=${encodeURIComponent(anchor)}` : ""}`;
    if (replace) {
      window.location.replace(target);
    } else {
      window.location.hash = target;
    }
  }

  function escapeHtml(value) {
    return String(value).replace(/[&<>"]/g, (char) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", "\"": "&quot;" })[char]);
  }

  function slugify(value) {
    return String(value).trim().toLowerCase()
      .replace(/<[^>]*>/g, "")
      .replace(/[`*_~]/g, "")
      .replace(/[\s]+/g, "-")
      .replace(/[!"#$%&'()*+,./:;<=>?@[\]\\^`{|}~]/g, "")
      .replace(/-+/g, "-")
      .replace(/^-|-$/g, "");
  }

  function uniqueSlug(base, seen) {
    const clean = base || "section";
    const count = seen.get(clean) || 0;
    seen.set(clean, count + 1);
    return count ? `${clean}-${count}` : clean;
  }

  function highlightIris(source) {
    const escaped = escapeHtml(source);
    const pattern = /(\/\/.*$)|("(?:\\.|[^"\\])*")|(:[A-Za-z_][\w?!=]*)|\b(0[xX][\da-fA-F_]+|0[bB][01_]+|0[oO][0-7_]+|\d(?:[\d_]*\d)?(?:\.\d(?:[\d_]*\d)?)?(?:[eEpP][+-]?\d(?:[\d_]*\d)?)?(?:f32|f64)?)\b|(^|[^A-Za-z0-9_?])(class|module|contract|fun|let|mut|global|import|from|export|as|type|where|open|override|extends|for|mixin|if|else|match|while|loop|return|raise|try|catch|finally|using|async|await|do|end|nil|true|false|self|super|same\?|new|Dynamic|Never)(?=$|[^A-Za-z0-9_?])|\b([A-Z][A-Za-z0-9_]*)\b/gm;
    return escaped.replace(pattern, (match, comment, string, symbol, number, prefix, keyword, type) => {
      if (comment) return `<span class="sh-comment">${comment}</span>`;
      if (string) return `<span class="sh-string">${string}</span>`;
      if (symbol) return `<span class="sh-symbol">${symbol}</span>`;
      if (number) return `<span class="sh-number">${number}</span>`;
      if (keyword) return `${prefix}<span class="sh-keyword">${keyword}</span>`;
      if (type) return `<span class="sh-type">${type}</span>`;
      return match;
    });
  }

  function highlightEbnf(source) {
    return escapeHtml(source).split("\n").map((line) => {
      let out = line.replace(/^([A-Za-z_][\w-]*)(\s*::=)/, '<span class="sh-rule">$1</span><span class="sh-operator">$2</span>');
      out = out.replace(/(&quot;(?:\\.|[^&])*?&quot;|'(?:\\.|[^'])*')/g, '<span class="sh-terminal">$1</span>');
      out = out.replace(/(::=)/g, '<span class="sh-operator">$1</span>');
      return out;
    }).join("\n");
  }

  function highlightCode(source, lang) {
    const normalized = String(lang || "").toLowerCase();
    if (normalized === "iris") return highlightIris(source);
    if (normalized === "ebnf") return highlightEbnf(source);
    return escapeHtml(source);
  }

  function renderLanding() {
    document.title = "Iris Programming Language";
    main.className = "landing";
    main.innerHTML = `
      <section class="hero" aria-labelledby="hero-title">
        <div>
          <div class="eyebrow">${t("heroKicker")}</div>
          <h1 id="hero-title">${t("heroTitleA")}<span>${t("heroTitleB")}</span></h1>
          <p class="hero-lede">${t("heroLede")}</p>
          <div class="hero-actions">
            <a class="primary-button" href="#/tutorial/${currentLang}/README">${t("readTutorial")}</a>
            <a class="ghost-button" href="#/spec/${currentLang}/README">${t("readSpec")}</a>
            <a class="ghost-button" href="${REPO_URL}" target="_blank" rel="noopener">${t("viewGithub")}</a>
          </div>
          <dl class="hero-facts" aria-label="Specification facts"><div><dt>14</dt><dd>${t("statFiles")}</dd></div><div><dt>EN / ZH</dt><dd>${t("statLangs")}</dd></div><div><dt>0</dt><dd>${t("statImpl")}</dd></div></dl>
        </div>
        <aside class="code-window" aria-label="Iris Programming Language code sample from the specification">
          <div class="window-top"><span class="window-dots" aria-hidden="true"><span></span><span></span><span></span></span><span class="code-window-label">spec/03 :: open class + revision</span></div>
          <pre><code class="language-iris">${highlightIris(heroCode)}</code></pre>
        </aside>
      </section>
      <section class="landing-section duality-section" aria-labelledby="duality-title">
        <div class="section-heading duality-heading">
          <div><a class="section-kicker source-link" href="#/spec/${currentLang}/01-language-identity?h=dynamic-behavior-static-promises">${t("dualityKicker")}</a><h2 id="duality-title">${t("dualityTitle")}</h2></div>
          <p class="section-lede">${t("dualityLede")}</p>
        </div>
        <div class="duality-grid">${dualityLayers.map((layer, index) => dualityCard(layer, index)).join("")}</div>
        <a class="duality-source" href="#/spec/${currentLang}/01-language-identity?h=dynamic-behavior-static-promises">${t("dualitySource")} <span>IRIS-V1-IDENTITY-C013</span></a>
      </section>
      <section class="landing-section positioning-section" aria-labelledby="positioning-title">
        <div class="section-heading positioning-heading">
          <div><div class="section-kicker">${t("positioningKicker")}</div><h2 id="positioning-title">${t("positioningTitle")}</h2></div>
          <p class="section-lede">${t("positioningLede")}</p>
        </div>
        <div class="positioning-grid">${positioningCards.map((card, index) => positioningCard(card, index)).join("")}</div>
      </section>
      <section class="hero status-banner" aria-labelledby="status-title">
        <div class="status-icon" aria-hidden="true">v1</div>
        <div>
          <h2 id="status-title">${t("statusTitle")}</h2>
          <p>${t("statusBody")}</p>
          <div class="status-actions"><a class="ghost-button" href="#/spec/${currentLang}/README?h=implementation-and-non-goals">${t("statusCta")}</a></div>
        </div>
      </section>
      <section class="landing-section" aria-labelledby="features-title">
        <div class="section-heading">
          <div><div class="section-kicker">Iris identity</div><h2 id="features-title">${t("featuresTitle")}</h2></div>
          <p class="section-lede">${t("featuresLede")}</p>
        </div>
        <div class="feature-grid">${features.map((feature) => `<article class="feature-card"><h3>${currentLang === "zh" ? feature[1] : feature[0]}</h3><p>${currentLang === "zh" ? feature[3] : feature[2]}</p></article>`).join("")}</div>
      </section>
      <section class="landing-section tutorial-section" aria-labelledby="tutorial-index-title">
        <div class="section-heading">
          <div><div class="section-kicker">${t("tutorialKicker")}</div><h2 id="tutorial-index-title">${t("tutorialTitle")}</h2></div>
          <p class="section-lede">${t("tutorialLede")}</p>
        </div>
        <div class="tutorial-strip">
          <a class="chapter-card tutorial-card is-featured" href="#/tutorial/${currentLang}/README"><span class="chapter-number">00</span><span><h3>${currentLang === "zh" ? tutorialChapters[0].zh : tutorialChapters[0].en}</h3><p>${currentLang === "zh" ? tutorialChapters[0].purposeZh : tutorialChapters[0].purposeEn}</p></span></a>
          <div class="tutorial-list" aria-label="${t("tutorialTitle")}">${tutorialChapters.slice(1).map((chapter, index) => tutorialLink(chapter, index)).join("")}</div>
        </div>
      </section>
      <section class="landing-section" aria-labelledby="spec-index-title">
        <div class="section-heading">
          <div><div class="section-kicker">14 artifacts</div><h2 id="spec-index-title">${t("specTitle")}</h2></div>
          <p class="section-lede">${t("specLede")}</p>
        </div>
        <div class="chapter-grid">${chapters.map((chapter, index) => chapterCard(chapter, index)).join("")}</div>
      </section>
      <footer class="site-footer"><span>${t("footer")}</span><a href="${REPO_URL}" target="_blank" rel="noopener">github.com/yuwenhuisama/Iris-Language</a></footer>
    `;
  }

  function chapterCard(chapter, index) {
    const title = currentLang === "zh" ? chapter.zh : chapter.en;
    const purpose = currentLang === "zh" ? chapter.purposeZh : chapter.purposeEn;
    return `<a class="chapter-card" href="#/spec/${currentLang}/${chapter.stem}"><span class="chapter-number">${String(index + 1).padStart(2, "0")}</span><span><h3>${title}</h3><p>${purpose}</p></span></a>`;
  }

  function tutorialLink(chapter, index) {
    const title = currentLang === "zh" ? chapter.zh : chapter.en;
    return `<a href="#/tutorial/${currentLang}/${chapter.stem}"><span>${String(index + 1).padStart(2, "0")}</span>${title}</a>`;
  }

  function positioningCard(card, index) {
    const title = currentLang === "zh" ? card.zh : card.en;
    const body = currentLang === "zh" ? card.bodyZh : card.bodyEn;
    const links = card.links.map((link) => `<a href="#/spec/${currentLang}/${link.stem}">${link.id}</a>`).join("");
    return `<article class="positioning-card" style="--index: ${index}">
      <span class="chapter-number">${String(index + 1).padStart(2, "0")}</span>
      <div>
        <h3>${title}</h3>
        <p>${inlineCode(body)}</p>
        ${links ? `<nav class="positioning-links" aria-label="${title}">${links}</nav>` : ""}
      </div>
    </article>`;
  }

  function dualityCard(layer, index) {
    const title = currentLang === "zh" ? layer.zh : layer.en;
    const dynamicText = currentLang === "zh" ? layer.dynamicZh : layer.dynamicEn;
    const promisedText = currentLang === "zh" ? layer.promisedZh : layer.promisedEn;
    const chapterLinks = layer.links.map((stem) => {
      const chapter = chapters.find((item) => item.stem === stem);
      return `<a href="#/spec/${currentLang}/${stem}">${chapter ? (currentLang === "zh" ? chapter.zh : chapter.en) : stem}</a>`;
    }).join("");
    const exampleCode = layer.exampleLines.join("\r\n");
    const exampleChapter = chapters.find((item) => item.stem === layer.exampleStem);
    const exampleTitle = exampleChapter ? (currentLang === "zh" ? exampleChapter.zh : exampleChapter.en) : layer.exampleStem;
    return `<article class="duality-card" style="--index: ${index}">
      <header><span class="chapter-number">${String(index + 1).padStart(2, "0")}</span><h3>${title}</h3><nav class="duality-chapter-links" aria-label="${title}">${chapterLinks}</nav></header>
      <div class="duality-split">
        <div class="duality-cell is-dynamic"><span>${t("dualityDynamic")}</span><p>${inlineCode(dynamicText)}</p></div>
        <div class="duality-cell is-promised"><span>${t("dualityPromised")}</span><p>${inlineCode(promisedText)}</p></div>
      </div>
      <details class="duality-example">
        <summary><span class="summary-indicator" aria-hidden="true"></span><span>${t("dualityExample")}</span><a href="#/spec/${currentLang}/${layer.exampleStem}">spec/${layer.exampleStem.slice(0, 2)} · ${exampleTitle}</a></summary>
        <pre><code class="language-iris">${highlightIris(exampleCode)}</code></pre>
      </details>
    </article>`;
  }

  function inlineCode(value) {
    return escapeHtml(value).replace(/`([^`]+)`/g, "<code>$1</code>");
  }

  function chapterNav(collection, activeStem) {
    return `<nav class="chapter-nav" aria-label="${t("readerChapters")}">${collection.chapters.map((chapter, index) => {
      const title = currentLang === "zh" ? chapter.zh : chapter.en;
      return `<a href="#/${collection.view}/${currentLang}/${chapter.stem}" ${chapter.stem === activeStem ? 'aria-current="page"' : ""}><span class="meta-chip">${String(index + 1).padStart(2, "0")}</span><br>${title}</a>`;
    }).join("")}</nav>`;
  }

  async function renderSpec(route) {
    return renderReader(collections.spec, route);
  }

  async function renderReader(collection, route) {
    const lang = normalizeLang(route.lang || currentLang);
    if (lang !== currentLang) setLanguage(lang, { silent: true });
    const chapter = collection.chapters.find((item) => item.stem === (route.stem || "README"));
    if (!chapter) {
      renderMissingChapter(collection, route.stem || "README");
      return;
    }
    const path = collection.getPath(chapter, currentLang);
    document.title = `${currentLang === "zh" ? chapter.zh : chapter.en} - Iris Programming Language`;
    main.className = "reader-page";
    main.innerHTML = `
      <div class="reader-shell">
        <aside class="reader-sidebar" id="reader-sidebar"><h2>${t("readerChapters")}</h2>${chapterNav(collection, chapter.stem)}</aside>
        <article class="reader-panel" aria-live="polite">
          <header class="reader-header"><div><span class="meta-chip">${t(collection.labelKey)}</span><h1>${currentLang === "zh" ? chapter.zh : chapter.en}</h1></div><span class="reader-source">${path}</span></header>
          <div class="markdown-body" id="reader-content"><p>${t("loading")}</p></div>
        </article>
        <aside class="reader-toc" aria-label="${t("readerToc")}"><h2>${t("readerToc")}</h2><nav class="toc-list" id="page-toc"></nav></aside>
      </div>
    `;
    sidebarToggle?.setAttribute("aria-expanded", "false");
    try {
      if (!window.marked) throw new Error("marked failed to load from CDN");
      const result = await fetchMarkdown(collection, chapter, currentLang);
      const content = document.getElementById(collection.contentId);
      document.querySelector(".reader-source").textContent = result.path;
      content.innerHTML = renderMarkdown(result.markdown);
      if (result.notice) content.insertAdjacentHTML("afterbegin", result.notice);
      postProcessMarkdown(content, collection, chapter);
      buildToc(content);
      observeSections();
      if (route.anchor) scrollToHeading(route.anchor);
    } catch (error) {
      const content = document.getElementById(collection.contentId);
      content.innerHTML = `<div class="error-panel" role="alert"><h2>${t("errorTitle")}</h2><p>${t("errorHelp")}</p><p><code>${escapeHtml(path)}</code></p><p><code>${escapeHtml(error.message)}</code></p></div>`;
    }
  }

  function renderMissingChapter(collection, stem) {
    const chapter = {
      file: `${stem}.md`,
      stem,
      en: stem.replace(/-/g, " "),
      zh: stem.replace(/-/g, " ")
    };
    const path = collection.getPath(chapter, currentLang);
    document.title = `${currentLang === "zh" ? chapter.zh : chapter.en} - Iris Programming Language`;
    main.className = "reader-page";
    main.innerHTML = `
      <div class="reader-shell">
        <aside class="reader-sidebar" id="reader-sidebar"><h2>${t("readerChapters")}</h2>${chapterNav(collection, "")}</aside>
        <article class="reader-panel" aria-live="polite">
          <header class="reader-header"><div><span class="meta-chip">${t(collection.labelKey)}</span><h1>${currentLang === "zh" ? chapter.zh : chapter.en}</h1></div><span class="reader-source">${path}</span></header>
          <div class="markdown-body" id="reader-content"><div class="error-panel" role="alert"><h2>${t("errorTitle")}</h2><p>${t("errorHelp")}</p><p><code>${escapeHtml(path)}</code></p><p><code>404 File not found</code></p></div></div>
        </article>
        <aside class="reader-toc" aria-label="${t("readerToc")}"><h2>${t("readerToc")}</h2><nav class="toc-list" id="page-toc"></nav></aside>
      </div>
    `;
    sidebarToggle?.setAttribute("aria-expanded", "false");
  }

  async function fetchMarkdown(collection, chapter, lang) {
    const path = collection.getPath(chapter, lang);
    const fallbackPath = collection.getFallbackPath?.(chapter, lang) || "";
    const response = await fetch(path, { cache: "no-cache" });
    if (response.ok) return { markdown: await response.text(), path, notice: "" };
    if (fallbackPath) {
      const fallbackResponse = await fetch(fallbackPath, { cache: "no-cache" });
      if (fallbackResponse.ok) {
        return {
          markdown: await fallbackResponse.text(),
          path: fallbackPath,
          notice: `<aside class="translation-notice" role="note"><h2>${t("translationNoticeTitle")}</h2><p>${t("translationNoticeBody")}</p><p><code>${escapeHtml(fallbackPath)}</code></p></aside>`
        };
      }
    }
    throw new Error(`${response.status} ${response.statusText}`);
  }

  function renderMarkdown(markdown) {
    const renderer = new marked.Renderer();
    const seen = new Map();
    let headingIndex = 0;
    renderer.heading = function (text, level) {
      headingIndex += 1;
      const textSlug = uniqueSlug(slugify(text), seen);
      const id = `section-${headingIndex}`;
      return `<h${level} id="${id}" data-slug="${textSlug}">${text}</h${level}>`;
    };
    renderer.code = function (code, language) {
      const lang = String(language || "").split(/\s+/)[0].toLowerCase();
      return `<pre><code class="language-${escapeHtml(lang)}">${highlightCode(code, lang)}</code></pre>`;
    };
    marked.setOptions({ gfm: true, breaks: false, mangle: false, headerIds: false });
    return marked.parse(markdown, { renderer });
  }

  function postProcessMarkdown(content, collection, chapter) {
    content.querySelectorAll("a[href]").forEach((link) => {
      const href = link.getAttribute("href");
      if (!href || href.startsWith("#")) return;
      if (/^https?:\/\//i.test(href)) {
        link.target = "_blank";
        link.rel = "noopener";
        return;
      }
      const match = href.match(/^([^#?]+\.md)(#[^?]+)?$/);
      if (match) {
        const target = resolveMarkdownLink(match[1], collection);
        if (target) {
          const anchor = match[2] ? `?h=${encodeURIComponent(match[2].slice(1))}` : "";
          link.href = `#/${target.collection.view}/${currentLang}/${target.chapter.stem}${anchor}`;
        }
      }
    });
    content.querySelectorAll('a[href^="#"]').forEach((link) => {
      const href = link.getAttribute("href");
      if (href.startsWith("#/")) return;
      const hash = href.slice(1);
      if (hash) link.href = `#/${collection.view}/${currentLang}/${chapter.stem}?h=${encodeURIComponent(hash)}`;
    });
    content.querySelectorAll("table").forEach((table) => {
      const wrapper = document.createElement("div");
      wrapper.className = "table-wrap";
      table.parentNode.insertBefore(wrapper, table);
      wrapper.appendChild(table);
    });
    content.querySelectorAll("h2, h3").forEach((heading) => {
      heading.tabIndex = -1;
      heading.insertAdjacentHTML("beforeend", ` <a class="heading-anchor" href="#/${collection.view}/${currentLang}/${chapter.stem}?h=${encodeURIComponent(heading.id)}" aria-label="Link to section">#</a>`);
    });
  }

  function resolveMarkdownLink(path, currentCollection) {
    const normalized = path.replace(/\\/g, "/");
    const file = normalized.split("/").pop();
    if (/(^|\/)spec\/iris-v1\//.test(normalized)) {
      const chapter = chapters.find((item) => item.file === file);
      return chapter ? { collection: collections.spec, chapter } : null;
    }
    if (/(^|\/)tutorial\/(en|zh-cn)\//.test(normalized)) {
      const chapter = tutorialChapters.find((item) => item.file === file);
      return chapter ? { collection: collections.tutorial, chapter } : null;
    }
    const chapter = currentCollection.chapters.find((item) => item.file === file);
    return chapter ? { collection: currentCollection, chapter } : null;
  }

  function buildToc(content) {
    const toc = document.getElementById("page-toc");
    const headings = Array.from(content.querySelectorAll("h2, h3"));
    toc.innerHTML = headings.map((heading) => `<a href="#${heading.id}" data-heading="${heading.id}" style="padding-left: ${heading.tagName === "H3" ? "1.25rem" : "0.75rem"}">${heading.textContent.replace(/#$/, "").trim()}</a>`).join("");
    toc.querySelectorAll("a").forEach((link) => {
      link.addEventListener("click", (event) => {
        event.preventDefault();
        const id = link.dataset.heading;
        history.replaceState(null, "", `${location.hash.split("?")[0]}?h=${encodeURIComponent(id)}`);
        scrollToHeading(id);
      });
    });
  }

  function observeSections() {
    if (activeSectionObserver) activeSectionObserver.disconnect();
    const headings = Array.from(document.querySelectorAll(".markdown-body h2, .markdown-body h3"));
    activeSectionObserver = new IntersectionObserver((entries) => {
      const visible = entries.filter((entry) => entry.isIntersecting).sort((a, b) => b.intersectionRatio - a.intersectionRatio)[0];
      if (!visible) return;
      document.querySelectorAll(".toc-list a").forEach((link) => link.classList.toggle("is-active", link.dataset.heading === visible.target.id));
    }, { rootMargin: "-18% 0px -68% 0px", threshold: [0, 0.2, 0.6, 1] });
    headings.forEach((heading) => activeSectionObserver.observe(heading));
  }

  function scrollToHeading(id) {
    const safeId = window.CSS && CSS.escape ? CSS.escape(id) : id.replace(/"/g, "\\\"");
    const target = document.getElementById(id) || document.querySelector(`[data-slug="${safeId}"]`);
    if (target) {
      target.scrollIntoView({ block: "start" });
      target.focus({ preventScroll: true });
    }
  }

  function visibleHeadingId() {
    const headings = Array.from(document.querySelectorAll(".markdown-body h2, .markdown-body h3"));
    return headings.find((heading) => heading.getBoundingClientRect().top >= 0)?.id || "";
  }

  function route() {
    const parsed = parseRoute();
    if (collections[parsed.view]) {
      renderReader(collections[parsed.view], parsed);
    } else {
      setLanguage(currentLang, { silent: true });
      renderLanding();
    }
    updateStaticLabels();
  }

  langToggle.addEventListener("click", () => setLanguage(currentLang === "en" ? "zh" : "en"));
  themeToggle.addEventListener("click", () => setTheme(currentTheme === "dark" ? "light" : "dark"));
  sidebarToggle.addEventListener("click", () => {
    const sidebar = document.getElementById("reader-sidebar");
    if (!sidebar) return;
    const open = !sidebar.classList.contains("is-open");
    sidebar.classList.toggle("is-open", open);
    sidebarToggle.setAttribute("aria-expanded", String(open));
  });
  window.addEventListener("hashchange", route);

  setTheme(currentTheme);
  setLanguage(currentLang, { silent: true });
  route();
})();
