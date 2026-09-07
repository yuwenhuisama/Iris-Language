(function () {
  "use strict";

  const STORAGE_LANG = "iris-site-lang";
  const STORAGE_THEME = "iris-site-theme";
  const REPO_URL = "https://github.com/yuwenhuisama/Iris-Language";
  const chapters = [
    ["README", "Specification Index", "规范索引"],
    ["01-language-identity", "Language Identity", "语言身份"],
    ["02-lexical-grammar", "Lexical Grammar", "词法语法"],
    ["03-runtime-object-model", "Runtime Object Model", "运行时对象模型"],
    ["04-bindings-callables-control-flow", "Bindings, Callables, And Control Flow", "绑定、可调用与控制流"],
    ["05-types-contracts-generics", "Types, Contracts, And Generics", "类型、Contracts 与泛型"],
    ["06-collections-text-regex", "Collections, Text, Binary, Regex, And Stable Hashing", "集合、文本、二进制、Regex 与稳定哈希"],
    ["07-async-resources-diagnostics", "Async, Resources, And Diagnostics", "Async、Resources 与 Diagnostics"],
    ["08-modules-metaprogramming", "Modules And Metaprogramming", "模块与元编程"],
    ["09-native-host-ffi", "Native Host And FFI", "原生 Host 与 FFI"],
    ["10-serialization-standard-library", "Serialization And Standard Library Boundary", "序列化与标准库边界"],
    ["11-migration-divergence", "Migration And Divergence Ledger", "迁移与差异台账"],
    ["12-conformance", "Conformance Framework", "一致性框架"],
    ["traceability-matrix", "Traceability Matrix", "可追溯性矩阵"]
  ].map(([stem, en, zh]) => ({ stem, file: `${stem}.md`, en, zh }));
  const tutorialChapters = [
    ["README", "Tutorial index", "教程索引"],
    ["01-getting-started", "Getting Started", "开始使用"],
    ["02-values-and-bindings", "Values and Bindings", "值与绑定"],
    ["03-control-flow", "Control Flow", "控制流"],
    ["04-callables-and-closures", "Functions, Closures, Blocks", "函数、闭包与块"],
    ["05-classes-and-objects", "Classes and Objects", "类与对象"],
    ["06-modules-and-contracts", "Modules and Contracts", "模块与 Contract"],
    ["07-types-and-generics", "Gradual Types", "渐进类型"],
    ["08-errors-and-resources", "Errors and Resources", "错误与资源"],
    ["09-dynamic-and-static", "Dynamic Meets Static", "动态与静态的交汇"],
    ["10-where-to-next", "Where To Next", "下一步"]
  ].map(([stem, en, zh]) => ({ stem, file: `${stem}.md`, en, zh }));
  const featureExamples = {
    "01-operators": {
      backend: "--vm", stdout: "6\nfalse\n",
      source: `let total = 1 + 2
let shifted = total << 1
print(shifted)
print(total == shifted)
`
    },
    "07-union-bindings": {
      backend: "--vm", stdout: "ready\n42\n",
      source: `mut value: String | Integer = "ready"
print(value)

value = 42
print(value)
`
    },
    "06-composition": {
      backend: "--vm", stdout: "B\n",
      source: `module A {
  public fun trace() -> String { "A" }
}

module B mixin A {
  public override fun trace() -> String { "B" }
}

class C mixin A, B {}

print(C.new().trace())
`
    },
    "06-contracts-declare-promises": {
      backend: "--vm", stdout: "Beep boop\n",
      source: `contract Speaker {
  fun speak() -> String
}

class Robot for Speaker {
  public impl fun speak() -> String {
    "Beep boop"
  }
}

let bot = Robot.new() as Speaker
print(bot..speak())
`
    },
    "09-open-class-reopen": {
      backend: "--vm", stdout: "2\n",
      source: `class Counter {
  public fun value() -> Integer { 1 }
}

let c = Counter.new()

open class Counter {
  public override fun value() -> Integer { 2 }
}

print(c.value())
`
    }
  };
  const homepageFeatures = [
    { key: "objects", stem: "01-getting-started", examples: ["01-operators"] },
    { key: "promises", stem: "07-types-and-generics", examples: ["07-union-bindings"] },
    { key: "composition", stem: "06-modules-and-contracts", examples: ["06-composition", "06-contracts-declare-promises"] },
    { key: "revisions", stem: "09-dynamic-and-static", examples: ["09-open-class-reopen"] }
  ];
  const strings = {
    en: {
      nav: { tutorial: "Learn", spec: "Specification", status: "Status", chapters: "Chapters" },
      skipContent: "Skip to content", home: "Home", themeLight: "Light", themeDark: "Dark",
      switchLanguage: "切换到中文", switchLight: "Switch to light theme", switchDark: "Switch to dark theme",
      heroKicker: "The Iris programming language",
      heroTitle: "Object-oriented.<br>Dynamic, within bounds.",
      heroLede: "Iris is a scripting language built on objects and messages. Static promises keep dynamic behavior within clear boundaries, from changing values to evolving classes.",
      readTutorial: "Build your first program", browseLessons: "Explore the learning path",
      exampleLabel: "Your first Iris program", sourceLabel: "Source", outputLabel: "Output",
      exampleCaption: "One function call. One line of output. Run it locally on the bytecode VM.",
      featureKicker: "01 / Meet the language", featureTitle: "Four ideas. Real code.",
      featureLede: "Objects, promises, composition, and change. Explore each through complete tutorial examples and their VM output.",
      backendLabel: "Backend",
      features: {
        objects: { title: "Objects all the way down.", body: "Every runtime value is an object. Arithmetic and comparison express behavior through message sends: +, <<, and == invoke methods on their receivers.", detail: "Here, adding makes 3, shifting makes 6, and equality compares the two values.", link: "Explore objects and messages" },
        promises: { title: "Changing values. Fixed promises.", body: "Dispatch stays runtime-dynamic. Type annotations constrain what is allowed; they never select a different selector or overload by static type.", detail: "This union example shows a binding changing from String to Integer while its allowed types stay fixed. It demonstrates a type boundary, not the full dispatch model.", link: "Explore types and promises" },
        composition: { title: "Reuse behavior. Declare obligations.", body: "Iris combines single Class inheritance with Modules and Contracts. A Module supplies reusable methods; a Contract declares requirements a Class explicitly promises to satisfy.", detail: "Module lookup reaches B before A here. Separately, Robot promises Speaker with for, implements speak with impl, and calls its Contract slot through a view. Reuse and promises play different roles.", link: "Explore Modules and Contracts" },
        revisions: { title: "Evolve a class. Keep its identity.", body: "A compatible open class revision can replace a method body without replacing the logical Class. The instance created before the update sees the new method here.", detail: "The language rule is transactional: validate a candidate, then publish atomically or roll back. This example shows a compatible method update, not full rollback or package-upgrade support.", link: "Explore bounded runtime change" }
      },
      featureCaptions: {
        "01-operators": "Arithmetic and comparison as messages.",
        "07-union-bindings": "The value changes; String | Integer remains the boundary.",
        "06-composition": "Module reuse: B overrides the trace method from A.",
        "06-contracts-declare-promises": "Contract promises: Robot satisfies Speaker explicitly.",
        "09-open-class-reopen": "An existing instance observes the compatible method update."
      },
      startKicker: "02 / Build & run", startTitle: "From source to hello.",
      startLede: "With a stable Rust toolchain installed, open the Iris repository in your terminal.",
      build: "Build the CLI", run: "Run on the VM", setupLink: "Setup, script files, and the REPL",
      startNote: "These commands use a Unix-style shell. The getting-started chapter covers setup and platform details.",
      learnKicker: "03 / Learn the language", learnTitle: "Follow the thread.",
      learnLede: "Ten chapters, one reading path. Begin with runnable examples; continue into the language model with explicit backend boundaries.",
      tutorialIndex: "Read the tutorial index", groupStart: "Start", groupCore: "Core", groupAdvanced: "Advanced",
      groupStartBody: "Build, run, and get comfortable with values.",
      groupCoreBody: "Shape control flow, functions, and objects.",
      groupAdvancedBody: "Understand promises, resources, and the boundaries.",
      runtimeKicker: "Know what you are running", runtimeTitle: "Two backends. Explicit choices.",
      vmTitle: "Bytecode VM", vmBody: "Use --vm for the learning path. The VM implements a supported subset and rejects constructs it cannot compile; it does not silently fall back.",
      referenceTitle: "Reference evaluator", referenceBody: "Without --vm, the CLI uses the tree-walking reference evaluator. It is a separate execution path, not evidence that a feature works on the VM.",
      statusTitle: "Running code, not a toolchain release.",
      statusBody: "Iris is a work in progress. The implementation is partial, with no production-readiness or compatibility promise. Check the repository for current limitations.",
      statusCta: "Current implementation status",
      specKicker: "04 / Keep the reference close", specTitle: "The language, precisely.",
      specLede: "The frozen v1 specification defines the language, not the implementation's current feature set. English is authoritative; Simplified Chinese is a reference translation.",
      specBrowse: "Browse all 14 specification artifacts", readSpec: "Open the specification index",
      footer: "Iris / A language to explore.", repo: "Source on GitHub",
      readerToc: "On this page", readerChapters: "Chapters", loading: "Loading document...",
      specLabel: "Frozen v1 specification", tutorialLabel: "Iris handbook", previous: "Previous", next: "Next",
      sourceLink: "View Markdown source", sectionLink: "Link to section", tableLabel: "Scrollable table", codeLabel: "Code example",
      translationNoticeTitle: "Chinese tutorial translation pending",
      translationNoticeBody: "The English source is shown until this chapter's translation is available.",
      errorTitle: "This document could not be loaded", errorHelp: "Check your connection and the document path. Local previews must serve the repository over HTTP.",
      recover: "Back to the collection index"
    },
    zh: {
      nav: { tutorial: "学习", spec: "语言规范", status: "项目状态", chapters: "章节" },
      skipContent: "跳转到正文", home: "首页", themeLight: "亮色", themeDark: "暗色",
      switchLanguage: "Switch to English", switchLight: "切换到亮色主题", switchDark: "切换到暗色主题",
      heroKicker: "Iris 编程语言",
      heroTitle: "面向对象，<br>动态有界。",
      heroLede: "Iris 是一门以对象与消息为基础的脚本语言。从值的变化到类的演进，静态承诺为动态行为划定清晰的边界。",
      readTutorial: "构建你的第一个程序", browseLessons: "探索学习路线",
      exampleLabel: "你的第一个 Iris 程序", sourceLabel: "源代码", outputLabel: "输出",
      exampleCaption: "一次函数调用，一行输出。在本地的字节码 VM 上运行它。",
      featureKicker: "01 / 认识语言", featureTitle: "四个理念，真实代码。",
      featureLede: "对象、承诺、组合与变化。通过完整的教程示例及其 VM 输出，认识 Iris 的语言模型。",
      backendLabel: "执行后端",
      features: {
        objects: { title: "每一个值，都是对象。", body: "所有运行时值都是对象。算术与比较通过消息发送表达行为：+、<< 和 == 都会调用接收者上的方法。", detail: "这个例子先相加得到 3，再移位得到 6，最后比较两个值是否相等。", link: "了解对象与消息" },
        promises: { title: "值可以变，承诺不变。", body: "分派始终在运行时动态进行。类型注解约束允许的行为，不会根据静态类型选择不同的选择子或重载。", detail: "联合类型示例中，绑定的值从 String 变为 Integer，允许的类型却始终固定。它展示的是类型边界，而非完整的分派模型。", link: "了解类型与承诺" },
        composition: { title: "复用行为，明确义务。", body: "Iris 将 Class 单继承与 Module、Contract 结合。Module 提供可复用的方法，Contract 声明由 Class 显式承诺满足的要求。", detail: "第一个例子的方法查找先到 B，再到 A。第二个例子中，Robot 用 for 承诺 Speaker，用 impl 实现 speak，再通过视图调用 Contract 槽位。行为复用与契约承诺各司其职。", link: "了解 Module 与 Contract" },
        revisions: { title: "类可以演进，身份保持不变。", body: "兼容的 open class 修订可以替换方法体，而不替换逻辑上的 Class。这个例子中，更新前创建的实例也能看到新方法。", detail: "语言规定变更必须遵循事务规则：验证候选变更，再原子发布或回滚。本例展示兼容的方法更新，并非完整回滚或包升级支持的证明。", link: "了解有边界的运行时变化" }
      },
      featureCaptions: {
        "01-operators": "算术与比较，以消息表达。",
        "07-union-bindings": "值发生变化，String | Integer 的边界保持不变。",
        "06-composition": "Module 复用：B 重写来自 A 的 trace 方法。",
        "06-contracts-declare-promises": "Contract 承诺：Robot 显式满足 Speaker。",
        "09-open-class-reopen": "已有实例观察到兼容的方法更新。"
      },
      startKicker: "02 / 构建与运行", startTitle: "从源码到第一声问候。",
      startLede: "安装稳定版 Rust 工具链后，在终端中打开 Iris 仓库目录。",
      build: "构建 CLI", run: "在 VM 上运行", setupLink: "环境配置、脚本文件与 REPL",
      startNote: "以下命令使用 Unix 风格的 shell。环境配置与平台差异请参阅开始使用章节。",
      learnKicker: "03 / 学习语言", learnTitle: "循着代码，逐步深入。",
      learnLede: "十个章节，一条学习路线。从可运行示例开始，再深入语言模型；每一步都明确区分后端能力边界。",
      tutorialIndex: "阅读教程索引", groupStart: "起步", groupCore: "核心", groupAdvanced: "进阶",
      groupStartBody: "构建、运行，熟悉值与绑定。", groupCoreBody: "掌握控制流、函数与对象。", groupAdvancedBody: "理解承诺、资源与能力边界。",
      runtimeKicker: "明确你正在运行什么", runtimeTitle: "两个后端，明确选择。",
      vmTitle: "字节码 VM", vmBody: "学习路线使用 --vm。VM 实现了受支持的语言子集，无法编译的结构会明确拒绝，不会悄悄回退到其他后端。",
      referenceTitle: "参考求值器", referenceBody: "不带 --vm 时，CLI 默认使用树遍历参考求值器。这是另一条执行路径，不能据此认定同一特性已获 VM 支持。",
      statusTitle: "代码已经能运行，但尚未发布工具链。",
      statusBody: "Iris 仍在开发中。目前只是部分实现，不承诺生产可用性或兼容性。请在仓库中查看当前限制。",
      statusCta: "查看当前实现状态",
      specKicker: "04 / 随时查阅规范", specTitle: "准确理解这门语言。",
      specLede: "冻结的 v1 规范定义语言语义，并不代表当前实现的功能范围。英文为权威版本，简体中文为参考译文。",
      specBrowse: "浏览全部 14 份规范文档", readSpec: "打开规范索引",
      footer: "Iris / 一门值得探索的语言。", repo: "GitHub 源代码",
      readerToc: "本页目录", readerChapters: "章节", loading: "正在加载文档...",
      specLabel: "冻结的 v1 规范", tutorialLabel: "Iris 学习手册", previous: "上一章", next: "下一章",
      sourceLink: "查看 Markdown 源文件", sectionLink: "链接到此节", tableLabel: "可横向滚动的表格", codeLabel: "代码示例",
      translationNoticeTitle: "中文教程翻译待完成", translationNoticeBody: "本章译文尚未提供，暂时显示英文源文。",
      errorTitle: "无法加载这份文档", errorHelp: "请检查网络连接及文档路径。本地预览需要通过 HTTP 服务仓库根目录。", recover: "返回文档索引"
    }
  };
  const collections = {
    spec: { view: "spec", labelKey: "specLabel", chapters, getPath: (chapter, lang) => `spec/iris-v1/${lang === "zh" ? "zh-cn/" : ""}${chapter.file}` },
    tutorial: { view: "tutorial", labelKey: "tutorialLabel", chapters: tutorialChapters, getPath: (chapter, lang) => `tutorial/${lang === "zh" ? "zh-cn" : "en"}/${chapter.file}` }
  };
  let currentLang = normalizeLang(localStorage.getItem(STORAGE_LANG) || "en");
  let currentTheme = localStorage.getItem(STORAGE_THEME) || "light";
  let activeSectionObserver = null;
  let navigationVersion = 0;
  let renderedDocument = "";
  const main = document.getElementById("main");
  const langToggle = document.getElementById("lang-toggle");
  const themeToggle = document.getElementById("theme-toggle");
  const sidebarToggle = document.getElementById("sidebar-toggle");

  function normalizeLang(value) { return /^(zh|zh-cn)$/i.test(value) ? "zh" : "en"; }
  function t(path) { return path.split(".").reduce((value, key) => value[key], strings[currentLang]); }
  function title(chapter) { return chapter[currentLang]; }
  function escapeHtml(value) { return String(value).replace(/[&<>"]/g, (char) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[char]); }
  function readerHref(view, stem, anchor = "") { return `#/${view}/${currentLang}/${stem}${anchor ? `?h=${encodeURIComponent(anchor)}` : ""}`; }

  function updateStaticLabels() {
    document.documentElement.lang = currentLang === "zh" ? "zh-CN" : "en";
    document.querySelectorAll("[data-i18n]").forEach((node) => { node.textContent = t(node.dataset.i18n); });
    document.querySelectorAll("[data-nav]").forEach((link) => {
      link.href = readerHref(link.dataset.nav, "README");
      if (link.dataset.nav === parseRoute().view) link.setAttribute("aria-current", "page");
      else link.removeAttribute("aria-current");
    });
    langToggle.textContent = currentLang === "zh" ? "EN" : "中文";
    langToggle.setAttribute("aria-label", t("switchLanguage"));
    themeToggle.textContent = currentTheme === "dark" ? t("themeLight") : t("themeDark");
    themeToggle.setAttribute("aria-label", currentTheme === "dark" ? t("switchLight") : t("switchDark"));
  }

  function setTheme(theme) {
    currentTheme = theme === "dark" ? "dark" : "light";
    localStorage.setItem(STORAGE_THEME, currentTheme);
    document.documentElement.dataset.theme = currentTheme;
    document.querySelector('meta[name="theme-color"]').content = currentTheme === "dark" ? "#171b22" : "#f7f5ef";
    updateStaticLabels();
  }

  function parseRoute() {
    const [path, query = ""] = (location.hash || "#/").slice(1).split("?");
    const [view, lang, stem = "README"] = path.split("/").filter(Boolean);
    return Object.hasOwn(collections, view) ? { view, lang: normalizeLang(lang), stem, anchor: new URLSearchParams(query).get("h") || "" } : { view: "landing" };
  }

  function highlightIris(source) {
    const tokens = /(\/\/[^\n\r]*|"(?:\\.|[^"\\])*"|:[A-Za-z_][\w?!=]*|\b(?:class|module|contract|fun|let|mut|global|import|from|export|as|type|where|open|override|extends|for|mixin|if|else|match|while|loop|return|raise|try|catch|finally|using|async|await|do|end|nil|true|false|self|super|new|Dynamic|Never)\b|\b\d[\w.]*|\b[A-Z][A-Za-z0-9_]*\b)/g;
    return source.split(tokens).map((token, index) => {
      if (index % 2 === 0) return escapeHtml(token);
      const kind = token.startsWith("//") ? "comment" : token.startsWith('"') ? "string" : token.startsWith(":") ? "symbol" : /^\d/.test(token) ? "number" : /^[A-Z]/.test(token) ? "type" : "keyword";
      return `<span class="sh-${kind}">${escapeHtml(token)}</span>`;
    }).join("");
  }

  function chapterLink(chapter, view) {
    const number = chapter.stem === "README" ? "00" : chapter.stem === "traceability-matrix" ? "TM" : chapter.stem.slice(0, 2);
    return `<a class="chapter-link" href="${readerHref(view, chapter.stem)}"><span class="chapter-number">${number}</span><span>${escapeHtml(title(chapter))}</span><span class="link-arrow" aria-hidden="true">↗</span></a>`;
  }

  function renderLanding() {
    document.title = "Iris Programming Language | " + (currentLang === "zh" ? "面向对象，动态有界。" : "Object-oriented. Dynamic, within bounds.");
    main.className = "landing";
    main.innerHTML = `
      <section class="hero" aria-labelledby="hero-title">
        <div class="hero-copy"><p class="eyebrow">${t("heroKicker")}</p><h1 id="hero-title">${t("heroTitle")}</h1><p class="hero-lede">${t("heroLede")}</p>
          <div class="hero-actions"><a class="primary-button" href="${readerHref("tutorial", "01-getting-started")}">${t("readTutorial")} <span aria-hidden="true">↗</span></a><a class="text-link" href="#learning-path" data-home-anchor="learning-path">${t("browseLessons")}</a></div>
        </div>
        <figure class="teaching-sheet" aria-label="${t("exampleLabel")}"><div class="sheet-heading"><span class="meta-chip">hello.iris</span><span class="sheet-backend">--vm</span></div>
          <div class="sheet-source"><span class="code-label">${t("sourceLabel")}</span><pre tabindex="0" aria-label="${t("sourceLabel")}"><code>${highlightIris('print("Hello, Iris!")')}</code></pre></div>
          <div class="sheet-output"><span class="code-label">${t("outputLabel")}</span><pre><code>Hello, Iris!</code></pre></div><figcaption>${t("exampleCaption")}</figcaption>
        </figure>
      </section>
      <section class="landing-section feature-section" aria-labelledby="feature-title"><div class="section-heading"><div><p class="section-kicker">${t("featureKicker")}</p><h2 id="feature-title">${t("featureTitle")}</h2></div><p class="section-lede">${t("featureLede")}</p></div>
        ${homepageFeatures.map(({ key, stem, examples }) => `<article class="feature-row" aria-labelledby="feature-${key}"><div class="feature-copy"><h3 id="feature-${key}">${t(`features.${key}.title`)}</h3><p>${escapeHtml(t(`features.${key}.body`))}</p><p>${escapeHtml(t(`features.${key}.detail`))}</p><a class="text-link" href="${readerHref("tutorial", stem)}">${t(`features.${key}.link`)} <span aria-hidden="true">↗</span></a></div><div class="feature-examples">${examples.map((id) => {
          const example = featureExamples[id];
          return `<figure class="feature-sheet" data-example-id="${id}" aria-labelledby="caption-${id}">
            <div class="sheet-heading"><span class="code-label">${t("backendLabel")}</span> <span class="sheet-backend">${example.backend}</span></div>
            <div class="sheet-source feature-source"><span class="code-label">${t("sourceLabel")}</span><pre tabindex="0" aria-label="${t("sourceLabel")}"><code>${highlightIris(example.source)}</code></pre></div>
            <div class="sheet-output feature-output"><span class="code-label">${t("outputLabel")}</span><pre tabindex="0" aria-label="${t("outputLabel")}"><code>${escapeHtml(example.stdout)}</code></pre></div>
            <figcaption id="caption-${id}">${escapeHtml(t(`featureCaptions.${id}`))}</figcaption></figure>`;
        }).join("")}</div></article>`).join("")}
      </section>
      <section class="landing-section start-section" aria-labelledby="start-title"><div class="section-heading"><div><p class="section-kicker">${t("startKicker")}</p><h2 id="start-title">${t("startTitle")}</h2></div><p class="section-lede">${t("startLede")}</p></div>
        <div class="command-steps"><article class="command-step"><h3><span class="chapter-number">01</span>${t("build")}</h3><pre tabindex="0" aria-label="${t("build")}"><code>cargo build -p iris-cli</code></pre></article>
        <article class="command-step"><h3><span class="chapter-number">02</span>${t("run")}</h3><pre tabindex="0" aria-label="${t("run")}"><code>./target/debug/iris --vm -e 'print("Hello, Iris!")'</code></pre></article></div>
        <div class="section-footnote"><p>${t("startNote")}</p><a class="text-link" href="${readerHref("tutorial", "01-getting-started")}">${t("setupLink")} <span aria-hidden="true">↗</span></a></div>
      </section>
      <section class="landing-section learning-section" id="learning-path" aria-labelledby="learn-title"><div class="section-heading"><div><p class="section-kicker">${t("learnKicker")}</p><h2 id="learn-title">${t("learnTitle")}</h2></div><p class="section-lede">${t("learnLede")}</p></div>
        <div class="curriculum">${[["Start", 1, 3], ["Core", 3, 7], ["Advanced", 7, 11]].map(([group, start, end]) => `<section class="curriculum-group"><div class="group-intro"><h3>${t(`group${group}`)}</h3><p>${t(`group${group}Body`)}</p></div><nav aria-label="${t(`group${group}`)}">${tutorialChapters.slice(start, end).map((chapter) => chapterLink(chapter, "tutorial")).join("")}</nav></section>`).join("")}</div>
        <a class="text-link section-end-link" href="${readerHref("tutorial", "README")}">${t("tutorialIndex")} <span aria-hidden="true">↗</span></a>
      </section>
      <section class="landing-section runtime-section" aria-labelledby="runtime-title"><div class="section-heading"><div><p class="section-kicker">${t("runtimeKicker")}</p><h2 id="runtime-title">${t("runtimeTitle")}</h2></div></div><div class="backend-grid"><article><h3>${t("vmTitle")} <code>--vm</code></h3><p>${t("vmBody")}</p></article><article><h3>${t("referenceTitle")}</h3><p>${t("referenceBody")}</p></article></div>
        <aside class="status-note"><h3>${t("statusTitle")}</h3><p>${t("statusBody")}</p><a class="text-link" href="${REPO_URL}#runtime-architecture" target="_blank" rel="noopener">${t("statusCta")} <span aria-hidden="true">↗</span></a></aside>
      </section>
      <section class="landing-section spec-section" aria-labelledby="spec-title"><div class="section-heading"><div><p class="section-kicker">${t("specKicker")}</p><h2 id="spec-title">${t("specTitle")}</h2></div><p class="section-lede">${t("specLede")}</p></div><a class="text-link" href="${readerHref("spec", "README")}">${t("readSpec")} <span aria-hidden="true">↗</span></a><details class="spec-directory"><summary>${t("specBrowse")}</summary><nav class="chapter-grid" aria-label="${t("specLabel")}">${chapters.map((chapter) => chapterLink(chapter, "spec")).join("")}</nav></details></section>
      <footer class="site-footer"><span>${t("footer")}</span><a href="${REPO_URL}" target="_blank" rel="noopener">${t("repo")} <span aria-hidden="true">↗</span></a></footer>`;
    main.querySelectorAll("[data-home-anchor]").forEach((link) => link.addEventListener("click", (event) => {
      event.preventDefault();
      document.getElementById(link.dataset.homeAnchor).scrollIntoView({ block: "start" });
    }));
  }

  function chapterNav(collection, stem) {
    return `<nav class="chapter-nav" aria-label="${t("readerChapters")}">${collection.chapters.map((chapter) => `<a href="${readerHref(collection.view, chapter.stem)}" ${chapter.stem === stem ? 'aria-current="page"' : ""}><span class="meta-chip">${chapter.stem === "README" ? "00" : chapter.stem === "traceability-matrix" ? "TM" : chapter.stem.slice(0, 2)}</span><span>${escapeHtml(title(chapter))}</span></a>`).join("")}</nav>`;
  }

  function errorPanel(path, error, view) {
    return `<div class="error-panel" role="alert"><h2>${t("errorTitle")}</h2><p>${t("errorHelp")}</p><p><code>${escapeHtml(path)}</code></p><p>${escapeHtml(error.message)}</p><a href="${readerHref(view, "README")}">${t("recover")}</a></div>`;
  }

  async function renderReader(collection, route, version) {
    const chapter = collection.chapters.find((item) => item.stem === route.stem);
    const path = chapter ? collection.getPath(chapter, currentLang) : collection.getPath({ file: `${route.stem}.md` }, currentLang);
    document.title = `${chapter ? title(chapter) : t("errorTitle")} - Iris Programming Language`;
    main.className = "reader-page";
    main.innerHTML = `<div class="reader-shell"><aside class="reader-sidebar" id="reader-sidebar"><h2>${t("readerChapters")}</h2>${chapterNav(collection, route.stem)}</aside>
      <div class="reader-main"><header class="reader-header"><nav class="reader-breadcrumb" aria-label="${t(collection.labelKey)}"><a href="#/">${t("home")}</a><span aria-hidden="true">/</span><a href="${readerHref(collection.view, "README")}">${t(collection.labelKey)}</a></nav><a class="reader-source" href="${escapeHtml(path)}" target="_blank" rel="noopener">${t("sourceLink")} <span aria-hidden="true">↗</span></a></header>
      <article class="reader-panel" aria-busy="true"><div class="markdown-body" id="reader-content"><p role="status">${t("loading")}</p></div></article></div>
      <aside class="reader-toc"><details ${matchMedia("(min-width: 1101px)").matches ? "open" : ""}><summary>${t("readerToc")}</summary><nav class="toc-list" id="page-toc" aria-label="${t("readerToc")}"></nav></details></aside></div>`;
    setSidebar(false);
    const content = document.getElementById("reader-content");
    try {
      if (!chapter) throw new Error("404 File not found");
      if (!window.marked) throw new Error("Markdown renderer unavailable (CDN)");
      const result = await fetchMarkdown(collection, chapter, currentLang);
      if (version !== navigationVersion) return;
      document.querySelector(".reader-source").href = result.path;
      content.innerHTML = renderMarkdown(result.markdown);
      if (result.fallback) content.insertAdjacentHTML("afterbegin", `<aside class="translation-notice" role="note"><strong>${t("translationNoticeTitle")}</strong><p>${t("translationNoticeBody")}</p></aside>`);
      postProcessMarkdown(content, collection, chapter);
      buildToc(content);
      const index = collection.chapters.indexOf(chapter);
      content.insertAdjacentHTML("afterend", `<nav class="reader-pagination" aria-label="${t("readerChapters")}">${[[index - 1, "previous"], [index + 1, "next"]].map(([position, label]) => collection.chapters[position] ? `<a href="${readerHref(collection.view, collection.chapters[position].stem)}"><span>${t(label)}</span><strong>${escapeHtml(title(collection.chapters[position]))}</strong></a>` : "<span></span>").join("")}</nav>`);
      renderedDocument = `${route.view}/${route.lang}/${route.stem}`;
      observeSections();
      if (route.anchor) scrollToHeading(route.anchor);
    } catch (error) {
      if (version !== navigationVersion) return;
      content.innerHTML = `<h1>${t("errorTitle")}</h1>${errorPanel(path, error, collection.view)}`;
    }
    if (version === navigationVersion) document.querySelector(".reader-panel").setAttribute("aria-busy", "false");
  }

  async function fetchMarkdown(collection, chapter, lang) {
    const path = collection.getPath(chapter, lang);
    const response = await fetch(path, { cache: "no-cache" });
    if (response.ok) return { markdown: await response.text(), path, fallback: false };
    if (collection.view === "tutorial" && lang === "zh" && response.status === 404) {
      const fallbackPath = collection.getPath(chapter, "en");
      const fallback = await fetch(fallbackPath, { cache: "no-cache" });
      if (fallback.ok) return { markdown: await fallback.text(), path: fallbackPath, fallback: true };
    }
    throw new Error(`${response.status} ${response.statusText}`);
  }

  function slugify(value) {
    return String(value).trim().toLowerCase().replace(/<[^>]*>/g, "").replace(/[`*_~]/g, "").replace(/[\s]+/g, "-").replace(/[!"#$%&'()*+,./:;<=>?@[\]\\^`{|}~]/g, "").replace(/-+/g, "-").replace(/^-|-$/g, "");
  }

  function renderMarkdown(markdown) {
    const renderer = new marked.Renderer();
    const seen = new Map();
    let headingIndex = 0;
    renderer.heading = function (text, level) {
      const base = slugify(text) || "section";
      const count = seen.get(base) || 0;
      seen.set(base, count + 1);
      headingIndex += 1;
      return `<h${level} id="section-${headingIndex}" data-slug="${escapeHtml(count ? `${base}-${count}` : base)}" tabindex="-1">${text}</h${level}>`;
    };
    renderer.code = function (code, language) {
      const lang = String(language || "").split(/\s+/)[0].toLowerCase();
      return `<pre tabindex="0" aria-label="${t("codeLabel")}"><code class="language-${escapeHtml(lang)}">${lang === "iris" ? highlightIris(code) : escapeHtml(code)}</code></pre>`;
    };
    return marked.parse(markdown, { renderer, gfm: true, breaks: false });
  }

  function postProcessMarkdown(content, collection, chapter) {
    const sourceUrl = new URL(collection.getPath(chapter, currentLang), document.baseURI);
    content.querySelectorAll("a[href]").forEach((link) => {
      const href = link.getAttribute("href");
      if (!href || href.startsWith("#/")) return;
      if (href.startsWith("#")) {
        link.href = readerHref(collection.view, chapter.stem, href.slice(1));
        return;
      }
      if (/^https?:\/\//i.test(href)) { link.target = "_blank"; link.rel = "noopener"; return; }
      const resolved = new URL(href, sourceUrl);
      const relative = resolved.pathname.slice(new URL(".", document.baseURI).pathname.length);
      const file = relative.split("/").pop();
      const targetCollection = relative.startsWith("spec/iris-v1/") ? collections.spec : /^tutorial\/(en|zh-cn)\//.test(relative) ? collections.tutorial : null;
      const targetChapter = targetCollection?.chapters.find((item) => item.file === file);
      if (targetChapter) link.href = readerHref(targetCollection.view, targetChapter.stem, decodeURIComponent(resolved.hash.slice(1)));
      else link.href = `${REPO_URL}/blob/iris_dev/${relative}${resolved.hash}`;
    });
    content.querySelectorAll("table").forEach((table) => {
      const wrapper = document.createElement("div");
      wrapper.className = "table-wrap";
      wrapper.tabIndex = 0;
      wrapper.setAttribute("role", "region");
      wrapper.setAttribute("aria-label", t("tableLabel"));
      table.parentNode.insertBefore(wrapper, table);
      wrapper.appendChild(table);
    });
    content.querySelectorAll("h2, h3").forEach((heading) => heading.insertAdjacentHTML("beforeend", ` <a class="heading-anchor" href="${readerHref(collection.view, chapter.stem, heading.id)}" aria-label="${t("sectionLink")}">#</a>`));
  }

  function buildToc(content) {
    const toc = document.getElementById("page-toc");
    toc.innerHTML = Array.from(content.querySelectorAll("h2, h3")).map((heading) => `<a href="${location.hash.split("?")[0]}?h=${heading.id}" data-heading="${heading.id}" class="${heading.tagName === "H3" ? "toc-nested" : ""}">${escapeHtml(heading.textContent.replace(/#$/, "").trim())}</a>`).join("");
  }

  function observeSections() {
    activeSectionObserver = new IntersectionObserver((entries) => {
      const visible = entries.filter((entry) => entry.isIntersecting).sort((first, second) => second.intersectionRatio - first.intersectionRatio)[0];
      if (visible) document.querySelectorAll(".toc-list a").forEach((link) => {
        if (link.dataset.heading === visible.target.id) link.setAttribute("aria-current", "location");
        else link.removeAttribute("aria-current");
      });
    }, { rootMargin: "-18% 0px -60% 0px", threshold: [0, 0.2, 0.6, 1] });
    document.querySelectorAll(".markdown-body h2, .markdown-body h3").forEach((heading) => activeSectionObserver.observe(heading));
  }

  function scrollToHeading(id) {
    const target = document.getElementById(id) || Array.from(main.querySelectorAll("[data-slug]")).find((heading) => heading.dataset.slug === id);
    if (target) { target.scrollIntoView({ block: "start" }); target.focus({ preventScroll: true }); }
  }

  function setSidebar(open) {
    document.getElementById("reader-sidebar")?.classList.toggle("is-open", open);
    sidebarToggle.setAttribute("aria-expanded", String(open));
  }

  function route() {
    const parsed = parseRoute();
    if (parsed.view !== "landing" && renderedDocument === `${parsed.view}/${parsed.lang}/${parsed.stem}`) {
      setSidebar(false);
      if (parsed.anchor) scrollToHeading(parsed.anchor);
      else window.scrollTo({ top: 0, behavior: "instant" });
      return;
    }
    navigationVersion += 1;
    renderedDocument = "";
    activeSectionObserver?.disconnect();
    sidebarToggle.classList.toggle("is-hidden", parsed.view === "landing");
    if (parsed.view !== "landing") {
      currentLang = parsed.lang;
      localStorage.setItem(STORAGE_LANG, currentLang);
      renderReader(collections[parsed.view], parsed, navigationVersion);
    } else renderLanding();
    updateStaticLabels();
    window.scrollTo({ top: 0, behavior: "instant" });
  }

  langToggle.addEventListener("click", () => {
    const parsed = parseRoute();
    currentLang = currentLang === "en" ? "zh" : "en";
    localStorage.setItem(STORAGE_LANG, currentLang);
    if (parsed.view === "landing") route();
    else {
      const headings = Array.from(main.querySelectorAll(".markdown-body [data-slug]"));
      const readingEdge = headings[0] ? parseFloat(getComputedStyle(headings[0]).scrollMarginBlockStart) : 0;
      const visible = headings.filter((heading) => heading.getBoundingClientRect().top <= readingEdge + 1).pop() || headings[0];
      location.replace(readerHref(parsed.view, parsed.stem, visible?.id || parsed.anchor || ""));
    }
  });
  themeToggle.addEventListener("click", () => setTheme(currentTheme === "dark" ? "light" : "dark"));
  sidebarToggle.addEventListener("click", () => {
    const open = sidebarToggle.getAttribute("aria-expanded") !== "true";
    setSidebar(open);
    if (open) document.querySelector('.chapter-nav a[aria-current="page"]')?.focus();
  });
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && sidebarToggle.getAttribute("aria-expanded") === "true") { setSidebar(false); sidebarToggle.focus(); }
  });
  document.querySelector(".skip-link").addEventListener("click", (event) => { event.preventDefault(); main.focus(); main.scrollIntoView(); });
  main.addEventListener("click", (event) => {
    const link = event.target.closest("a[href]");
    if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey || link?.getAttribute("href") !== location.hash) return;
    const parsed = parseRoute();
    if (parsed.view !== "landing") {
      event.preventDefault();
      route();
    }
  });
  window.addEventListener("hashchange", route);
  setTheme(currentTheme);
  route();
})();
