# Iris 复活计划：项目审查与语言设计讨论纪要

> 状态：讨论基线文档  
> 整理日期：2026-07-23  
> 当前代码基线：`iris_dev` / `40b6559`  
> 原始项目开发期：2016-03 至 2017-04  
> 适用范围：旧项目质量评估、语言设计取舍、Rust 重写、运行时/JIT 路线、渐进类型系统

## 1. 文档目的

本文整理目前围绕 Iris-Language 已完成的关键讨论，作为后续逐项设计新版 Iris 的统一上下文。

本文刻意区分三类内容：

- **已确认事实**：可从当前仓库、Git 历史、旧语言介绍或测试脚本直接验证。
- **当前建议**：截至目前讨论形成的推荐方向，尚未等价于正式语言规范。
- **待决策事项**：后续必须逐项讨论并冻结的语义或工程选择。

本文不是新版 Iris 的最终 specification，也不是详细实施计划。任何语法示例均用于表达设计方向，最终语法仍需独立评审。

---

## 2. 项目背景与原始定位

### 2.1 已确认事实

`Document/Iris Programming Language Intro.pdf` 将 Iris v0.0.1.0 定位为：

- 动态解释型语言；
- 主要借鉴 Ruby，同时受到 Java、C#、Python、Perl、PHP 等语言影响；
- 完全面向对象，字面量、方法、类等均具有对象身份；
- 完全动态，允许运行期修改类与方法；
- 面向宿主应用脚本化，特别是跨平台游戏引擎；
- 强调与 C++ 等宿主语言交互以及 native extension；
- 以编码表达力和优雅性优先，不以极致执行效率为第一目标。

Git 历史共 43 个提交，开发期大约为 2016-03 至 2017-04，主要由单人完成。`iris_dev` 是最新代码；`iris_stable` 实际落后六个提交，并非经过额外稳定化的发布分支。

### 2.2 当前评价

Iris 不是简单的表达式解释器，而是完成了以下端到端链路：

```text
source
  -> lexer/parser
  -> AST
  -> semantic validation
  -> bytecode generation
  -> bytecode VM
  -> dynamic object model
  -> GC / threads / native extension API
```

作为 2016-2017 年的单人玩具语言，其完成度和技术覆盖面明显高于普通练习项目；作为今天可维护、可嵌入、可发布的运行时，则尚未进入稳定化阶段。

---

## 3. 旧项目整体质量结论

### 3.1 综合评分

| 维度 | 评分 | 说明 |
|---|---:|---|
| 玩具项目完成度 | 7/10 | 功能面与实现深度突出 |
| 架构方向 | 6/10 | AST → bytecode → VM、对象模型、扩展机制方向合理 |
| 当前正确性 | 3/10 | 存在静态可证明的 GC、线程、ABI、VM 缺陷 |
| 可维护性 | 3/10 | 全局单例、God Object、裸指针、grammar 缺失 |
| 测试质量 | 3/10 | 行为样例较多，但没有自动断言与聚合 runner |
| 构建可复现性 | 5.5/10 | Windows 工具链明确，但无 CI 且配置已漂移 |
| API 设计 | 5/10 | 扩展思路清楚，但 ABI、错误处理、生命周期不稳定 |
| 安全与健壮性 | 3.5/10 | 二进制输入与 native 边界缺少防御 |
| 生产就绪度 | 2/10 | 不宜嵌入长期运行或处理不可信输入 |
| 综合专业质量 | 4/10 | 有完整闭环，但尚未进入稳定化与产品化阶段 |

### 3.2 值得肯定的实现设计

1. **真正的多阶段语言实现**  
   `IrisCompiler.cpp`、`IrisInstructorMaker.cpp`、`IrisInterpreter.cpp` 分别承担编译、指令生成和 VM 执行。

2. **AST Validate / Generate 分离**  
   AST 节点先进行合法性检查，再生成 bytecode；对玩具语言而言边界清晰且实用。

3. **Native wrapper + Tag 模型一致**  
   语言可见类负责方法注册、分配、释放、标记，`*Tag` 持有 C++ payload。

4. **对象模型覆盖广**  
   支持 class、inheritance、module、interface、getter/setter、closure、variadic parameters、method authority、异常与 iterator。

5. **行为样例覆盖较广**  
   `Iris Library Test/test script/` 下的 expression / statement 脚本覆盖了大量语言表面。

### 3.3 当前明确的发布阻断问题

以下问题不是风格争议，而是静态可证明或高置信度的正确性问题。

#### P0：Pointer extension callback ABI 与核心不兼容

核心 `IrisNativeFunction` 已包含第五个 `IIrisThreadInfo*` 参数：

- `IrisLangLibrary/include/IrisInterfaces/IIrisClass.h`
- `IrisLangLibrary/include/IrisExportAPIs/IrisExportForCpp.h`

但 `Iris Pointer Extension/IrisPointer.h` 中四个 callback 仍然只有四个参数。Git 提交 `c38cf86` 修改 callback signature 时未完整同步 extension。当前四项目 solution 很可能无法完整构建。

#### P0：GC shutdown 存在 use-after-free 风险

`IrisLangLibrary/src/IrisInterpreter/IrisNativeModules/IrisGC.cpp` 的 shutdown 顺序是：

1. 删除 thread GC data；
2. detach GC thread；
3. 设置 shutdown flag；
4. 删除 `std::thread` 对象。

被 detach 的线程仍可能访问已释放数据。正确顺序应为设置关闭状态、唤醒、join、确认退出，再释放数据。

#### P0：thread detach 条件写反

以下位置在 `!joinable()` 时调用 `detach()`：

- `IrisLangLibrary/src/IrisThread/IrisThreadManager.cpp`
- `IrisLangLibrary/src/IrisThread/IrisThreadTag.cpp`

这可能抛出 `std::system_error`；对仍 joinable 的 thread 对象进行析构则可能触发 `std::terminate`。

#### P0：Stop-The-World 状态存在数据竞争

GC 跨线程共享状态使用普通 `bool`，且共享容器的遍历依赖一个没有建立可靠内存序与锁不变量的 STW 协议。若 mutator 未完全停止，mark/sweep 与 heap、thread map、environment map 的并发访问可能导致 iterator invalidation、use-after-free 或容器损坏。

#### P0：`.irc` 反序列化信任文件内容

`IrisLangLibrary/src/IrisUnil/IrisVirtualCodeFile.cpp`：

- 不验证 magic / version；
- 不限制字段数量和字符串长度；
- 不验证剩余文件大小；
- 不检查 `read()` 成功；
- 直接根据文件长度分配内存。

畸形 `.irc` 可导致 OOM、未初始化内存读取、越界及进程终止。

#### P0：VM 取指无边界检查

`IrisInterpreter::GetOneAM()` 及 opcode loop 使用 `vector::operator[]` 直接读取 operand，没有 verifier 或边界检查。损坏的 bytecode 可导致未定义行为。

#### P1：`SPR` opcode fallthrough

`IrisLangLibrary/src/IrisInterpreter.cpp` 中 `case SPR` 后没有 `break`，会继续执行 `LOAD_CAST` 并覆盖结果。

#### P1：thread cleanup API 忽略形参

`IrisGC::RemoveThreadGCData(nId)` 与 `IrisThreadManager::DeleteThreadInfo(nThreadID)` 删除时使用 `this_thread::get_id()`，并通过 `operator[]` 查询 map，可能删除错误对象或插入空项。

#### P1：Pointer native buffer 不安全

`Iris Pointer Extension/IrisPointerTag.cpp`：

- 负长度会转换为巨大 `size_t`；
- `offset + length` 可能有符号溢出并绕过 bounds check；
- `Get()` 忽略 offset，总是从 buffer 开头读取。

#### P1：Host/native API 边界检查不足

- `IrisValues::GetValue()` / `SetValue()` 使用未检查的 `operator[]`；
- `IrisStack::Pop()` 和 `IrisArrayTag::Pop()` 对空容器调用 `back()`；
- `IR_Initialize()` 不验证 init struct 和 callback；
- C ABI 大量依赖裸句柄与强制转换。

### 3.4 系统性架构债务

#### 全局单例

以下服务全部通过 `CurrentX()` 暴露进程级状态：

- Compiler
- Interpreter
- GC
- ThreadManager
- FatalErrorHandler

后果包括无法支持多个独立 runtime、测试隔离困难、生命周期不清晰、并发与重复初始化难以证明。

#### `IrisInterpreter.cpp` 是 God Object

约 3000 行文件同时负责：

- built-in initialization；
- class/module/interface registry；
- globals/constants；
- heap；
- extension loader；
- VM loop；
- opcode handlers；
- thread coordination；
- shutdown。

#### Opcode 缺少单一事实源

opcode 信息分散在：

- `IrisVirtualCodeNumber.h`
- `IrisVirtualCodeStructures.h`
- `IrisInstructorMaker.cpp`
- `IrisInterpreter.cpp`
- `.irc` serializer

新增或修改指令必须人工同步多处。

#### Grammar 源缺失

仓库仅保留生成的：

- `lex.yy.cpp`
- `y.tab.cpp`
- `y.tab.h`

缺失 `iris.l` / `iris.y`，语言语法无法安全演进。

#### 内存所有权没有统一模型

项目同时存在 interpreter heap、environment heap、thread-local native heap、Tag payload、method-owned function、extension object、singleton 与 optional memory pool，但所有权由裸指针和分散约定表达。

---

## 4. 旧语言设计评估

### 4.1 语言的核心身份

旧 Iris 最清晰的设计主线是：

> 以完全对象化和消息发送作为核心，用 module、interface、block、动态类修改增强脚本表达能力，同时保持与 native host 的高黏合度。

这条主线值得保留。新版不应退化为 Rust、Kotlin 或 TypeScript 的表面模仿。

### 4.2 值得保留的语言机制

1. **完全对象模型**  
   Integer、Float、String 等字面量都是对象；类、方法、模块、接口具有运行时身份。

2. **运算符即方法 / 消息发送**  
   一元、二元、索引和赋值索引均可解释为方法调用，例如 `+`、`[]`、`[]=`。

3. **单继承 + module mixin + interface contract**  
   分别解决实现继承、行为复用和显式规范，而非依赖多继承。

4. **只有 `false` 与 `nil` 为假**  
   truthiness 简单且可预测。

5. **实例变量默认私有**  
   外部属性访问通过 accessor，符合“字段不直接成为 public layout”的对象哲学。

6. **任意对象可作为异常**  
   `groan` 抛出对象，`order/serve/ignore` 承担 try/catch/finally 类语义。

7. **尾随 block 与 variadic block parameters**  
   适合 iterator、resource scope、DSL 和 callback。

8. **Range 开闭区间**  
   数学区间表达力强，支持整数、字符、正向和反向范围。

9. **`switch/when` 无 fallthrough**  
   分支可预测，也容易演进为 pattern matching。

10. **动态修改类与方法**  
    元编程从“类和方法也是对象”的模型自然推导出来。

11. **class method 与 instance method 明确区分**。

### 4.3 需要重新设计的语言机制

#### 前置分号

旧 Iris 使用：

```iris
;a = 10
;print(a)
```

该语法具有辨识度，但几乎没有语义收益，会增加视觉噪音、学习成本和工具链负担。当前建议是新版删除前置分号，使用换行或常规尾部分号。

#### 循环 `if`

旧设计复用 `if(condition, count, counter)` 表达循环，并以 `if(true, 0)` 表示无限循环。它通过减少 `while` 关键字增加了语义复杂度和意外程度。

当前建议：

```iris
while condition {
}

repeat 10 as i {
}
```

或优先通过 Range + `for` 表达计数循环。

#### `block / with / without / cast`

旧语法将 block presence 分支写成方法体外的 `with` / `without`，并用 `cast` 或后来的 `cast.call` 调用 block，作用域与语义不直观。

当前建议将 block 变成显式的特殊参数或普通 closure value，例如：

```iris
fun process(&block) {
    if block != nil {
        block.call(value)
    }
}
```

尾随 block 调用语法仍保留。

#### 方法权限

PDF 使用 `everyone/native/personal`，最终实现改为 `everyone/relative/personal`，且权限必须在方法定义后单独声明。

当前建议把权限放到方法声明处：

```iris
public fun draw() { }
protected fun update() { }
private fun reset() { }
```

若保留 Iris 词汇，也应写成 `everyone fun` / `relative fun` / `personal fun`，而不是后置列表。

#### `involves` / `joints`

两者英文语义不自然，`joints` 作为动词形式也不正确。

当前建议优先可读性：

```iris
class Sprite extends Node
             mixes Drawable
             implements Serializable {
}
```

最终关键字仍待决策。

#### Interface 兼容规则

旧文档声称要求“参数个数和类型一致”，但语言没有参数类型声明，实际只能检查固定参数数量和 variadic 形态。

当前建议将 interface 升级为 nominal protocol contract，并以“调用兼容性”而非 AST 形式完全一致作为规则。

#### 数组读越界自动扩容

旧设计中读取不存在的 index 会返回 `nil`，同时扩展数组并填充 `nil`。读取产生结构性副作用，可能意外分配大量内存并改变后续迭代结果。

当前建议：读取越界返回 `nil` 或抛 `IndexError`，但不扩容；只有写入才可能扩容。

#### Range 箭头与 Hash 冲突

PDF 使用 `->`，最终测试改成 `=>`，与 Hash key/value 箭头冲突。

当前建议保留开闭区间，但使用独立语法，例如：

```iris
[1 .. 10)
("a" .. "z"]
```

#### `super` 与 MRO

旧文档只定义 `super` 调用父类同名实例方法，没有说明 module mixin 的查找顺序。

新版必须正式定义 method resolution order；`super` 应调用 MRO 中的下一个实现，而不只是直接 superclass。

#### 完全动态的边界

旧文档承诺几乎所有运行期类修改能力，但未定义修改后的 method cache、interface conformance、对象 shape 和 JIT 失效规则。

当前建议通过版本化约束动态性：

```text
method_table_version
shape_version
hierarchy_version
contract_version
```

动态修改必须验证契约并让相关 inline cache / JIT code 失效。

### 4.4 旧文档与最终实现的语法漂移

旧 PDF 是早期设计介绍，不是最终 specification。已确认的漂移包括：

| 机制 | PDF | 最终脚本/高亮 |
|---|---|---|
| Range | `->` | `=>` |
| protected 权限 | `native` | `relative` |
| block 调用 | `cast(...)` | `cast.call(...)` |
| mixin 关键字 | `involve` / `involves` 混用 | `involves` |

文档自身还包含：

- 称有 16 种语句，但编号到 18；
- Range 遍历一处标记为“暂未添加”；
- `display()` / `show()` 不一致；
- `super` 示例漏写 `extends A`；
- `i/I`、`integer/interger` 等笔误；
- 使用十六进制字面量但未在字面量章节定义；
- Block 闭包与元编程只有章节标题，没有正式语义。

因此新版不能把 PDF 直接转换为 parser grammar，应先建立语义决策清单。

### 4.5 必须补充定义的语言语义

1. 固定整数和浮点模型，不再依赖宿主 C++ `int` / `double` ABI；当前倾向 `i64` 与 IEEE-754 `f64`。
2. 明确 block 内 `return` 是 local return 还是 non-local return。
3. 区分类绑定是否可重新赋值、类对象是否可 reopen、superclass/module composition 是否可修改。
4. 正式定义 class/module/interface 的 MRO 与冲突处理。
5. 定义外部资源管理；GC 只能回收内存，不能替代文件、socket、锁、GPU resource 的确定性释放。新版需 `defer`、scope guard 或 close protocol。
6. 定义动态修改与 type contract、inline cache、JIT invalidation 的关系。

---

## 5. Rust 重写与后端/JIT 路线

### 5.1 已形成的核心判断

以下是三项独立决策，不应绑定为一次重写：

1. 使用 Rust 作为实现语言；
2. 使用成熟代码生成后端；
3. 实现 JIT。

成熟后端可以替代机器码生成与部分优化基础设施，但不会自动解决动态语言真正困难的部分：

- value representation；
- object layout / shape；
- dynamic dispatch；
- inline cache；
- GC roots / safepoints；
- exception unwinding；
- closure environment；
- code invalidation；
- deoptimization；
- stack trace / debugging。

### 5.2 当前推荐路线

```text
Rust frontend and runtime
  + Iris-owned HIR
  + Iris-owned register bytecode / MIR
  + reference interpreter
  + Cranelift baseline JIT
```

总体架构：

```text
Iris source
    -> Lexer / Parser
    -> AST
    -> HIR: name resolution, scope, class/module semantics
    -> Iris Bytecode / MIR
         -> Reference Interpreter
         -> Cranelift Baseline JIT
              -> Rust Runtime Helpers
```

应重写旧 VM 的实现，但不应删除语言自己的中间层。

### 5.3 LLVM 评价

优点：

- 优化上限高；
- ORC JIT 与 target support 成熟；
- IR、optimization pass、debug info、stack map 设施完整。

代价：

- Rust 需要 `llvm-sys` / Inkwell 等 FFI；
- LLVM 主版本和本地安装绑定；
- CI 与分发复杂；
- moving GC 仍需 statepoint、`gc.relocate`、stack map 与 runtime 协议；
- LLVM 不替语言实现 object model、dispatch、deopt 和 GC policy。

当前结论：LLVM 更适合作为未来 optimizing JIT backend，而不是复活项目的第一个后端。

### 5.4 CLR/.NET 评价

CLR 同时提供：

- managed object model；
- GC；
- JIT；
- exception；
- thread；
- reflection / metadata；
- debugging / profiling；
- .NET ecosystem。

如果完全接受 CLR runtime model，它能替项目承担大量基础工作。但代价是 Iris 会被深度塑造成 .NET language，并且 Rust + CLR 组合会增加两套 runtime 与 FFI 边界。

当前结论：

- 若目标是最快获得成熟 GC/JIT 并接入 .NET 生态，可选择 CLR，且 runtime 更适合使用 C#。
- 若目标是保持独立语言身份、以 Rust 为核心并研究现代动态语言 runtime，则不推荐 CLR 作为首选。

### 5.5 Cranelift 评价

优点：

- Rust-native integration；
- `JITBuilder` / `JITModule` 直接支持 JIT function/data/module；
- 编译速度快，适合 baseline JIT；
- backend 复杂度低于 LLVM；
- runtime helper symbol 绑定直接；
- 更容易阅读与调试 generated IR。

限制：

- 优化上限低于 LLVM；
- `cranelift-jit` 仍应视作需要版本固定和 backend abstraction 的依赖。

当前结论：Cranelift 是复活 Iris 的第一 JIT 后端首选。

### 5.6 为什么保留解释器

即使目标明确包含 JIT，也应保留 reference interpreter：

1. 作为语言语义基准；
2. 支持 interpreter / JIT differential testing；
3. 执行冷代码，避免全部 JIT；
4. 支持 REPL、debugger、single-step 和 fallback；
5. 帮助定位 frontend、runtime、lowering 或 JIT 错误。

合理分层：

```text
first calls / cold code  -> interpreter
hot threshold reached    -> baseline JIT
future sustained hot code -> optional optimizing JIT
```

### 5.7 新 IR 建议

不应逐行移植旧 bytecode。建议：

- AST 只保存语法结构；
- HIR 完成名称解析、变量种类、scope、closure capture、class/module/interface resolution、source span 和 desugaring；
- execution IR 使用寄存器式 bytecode/MIR，而非旧式隐式 stack/register 混合模型。

寄存器式 IR 的优势：

- 更容易 lower 到 Cranelift IR；
- stack effect 少；
- verifier 简单；
- disassembler/debugger 更清楚；
- 不易发生 stack underflow；
- 更适合未来 SSA lowering。

### 5.8 JIT 分阶段建议

#### 第一阶段：Baseline JIT

- 不做 speculative optimization；
- dynamic send 统一调用 runtime helper；
- primitive operation 可先调用 `iris_add` 等 helper；
- 目标是正确、编译快、易调试。

首版明确不做：

- unboxing；
- speculative inlining；
- polymorphic inline cache；
- deoptimization；
- escape analysis；
- moving GC；
- OSR。

#### 第二阶段：Inline Cache

动态 OO 语言的第一项高收益优化通常是 call-site inline cache，而不是通用 `-O3`。

每个 send site 可记录：

```text
expected_class
method_handle
class_version
```

命中时 direct call，失败时回到 generic lookup 并更新 cache。

#### 第三阶段：Primitive Specialization

对 SmallInt / Float / String 等热点操作增加 guard + fast path，失败时回到 generic message send。

#### 第四阶段：Optimizing JIT

只有 baseline JIT、profiling 和 inline cache 稳定后，再评估 speculative inlining、deoptimization、OSR、escape analysis 等；此时 LLVM 的价值才更明显。

### 5.9 GC 建议

第一版采用：

```text
single-threaded
precise
non-moving
mark-sweep
explicit roots / shadow stack
```

理由：

- 对象地址稳定；
- 不需要第一版处理 `gc.relocate`；
- 仍可做到精确 GC；
- 容易建立 interpreter/JIT root contract。

不建议第一版使用 reference counting，因为 object graph、class/module graph、closure capture、array/hash 天然形成环。

Rust 不会自动解决动态语言 GC。允许少量 `unsafe`，但应集中在 `gc`、`jit`、`ffi` 边界，每个 unsafe block 明确不变量，其余 frontend/runtime 尽量 safe Rust。

### 5.10 Rust workspace 方向

候选结构：

```text
iris/
├── crates/
│   ├── iris-syntax/
│   ├── iris-hir/
│   ├── iris-bytecode/
│   ├── iris-runtime/
│   ├── iris-interpreter/
│   ├── iris-jit/
│   ├── iris-jit-cranelift/
│   ├── iris-cli/
│   └── iris-test-runner/
└── tests/
```

但第一阶段不要过度拆 crate；HIR、bytecode、interpreter 边界尚未稳定时可先合并，稳定后再拆。

### 5.11 第一垂直切片里程碑

第一个里程碑不是“JIT 跑起来”，而是：

> 用 Rust 复现一个可验证的语义垂直切片，让解释器与 JIT 对同一程序产生完全一致的结果。

首个切片建议支持：

- Integer；
- Bool / Nil；
- local variable；
- arithmetic / comparison；
- function definition / call；
- `if`；
- recursion；
- `return`；
- `print`；
- source diagnostics。

同时完成：

1. lexer/parser；
2. AST → HIR；
3. HIR → bytecode；
4. bytecode verifier；
5. reference interpreter；
6. Cranelift baseline JIT；
7. interpreter/JIT differential test；
8. expected-output test；
9. malformed-bytecode test；
10. Windows/Linux/macOS CI。

后续语义依赖顺序：

```text
functions
  -> closures
  -> objects/classes
  -> method send + inline cache
  -> arrays/hashes
  -> GC
  -> modules/interfaces
  -> extension ABI
  -> concurrency
```

---

## 6. 渐进类型系统方向

### 6.1 核心定位

新版 Iris 的 type hint 当前建议定义为：

> 可省略、但一旦写出就具有强制语义的渐进类型契约。  
> Optional to write, mandatory to obey.

它不同于：

- TypeScript 式复杂、编译后擦除的独立类型层；
- Python 式默认只作为工具提示、运行时不强制的注解。

推荐分类：

```text
Gradual Typing with Enforced Runtime Contracts
```

### 6.2 四条类型系统原则

1. **Type annotations are optional to write, mandatory to obey.**
2. **Provable violations are compile errors; uncertain boundaries get runtime guards.**
3. **Types constrain values and messages, but never change dynamic dispatch.**
4. **Type metadata survives at runtime and participates in reflection, plugins, and JIT invalidation.**

### 6.3 基本示例

完全动态方法：

```iris
fun add(a, b) {
    return a + b
}
```

带强制契约：

```iris
fun add(a: Integer, b: Integer) -> Integer {
    return a + b
}
```

静态已知错误：

```iris
add("1", "2")
```

应在编译期报错。

动态来源：

```iris
value = load_external_value()
add(value, 2)
```

无法静态证明时，在调用边界生成运行时 guard；失败时抛 Iris `TypeError`。

### 6.4 类型是语义契约，不是优化提示

无论 interpreter、Debug、Release 或 JIT：

- 参数必须满足标注；
- 返回值必须满足标注；
- property/variable 写入必须满足标注；
- 违反契约必须产生确定的 Iris TypeError；
- 不允许 JIT 因错误信任注解而进入未定义行为。

JIT 只可在已证明的调用路径消除重复检查，不能改变语义保证。

### 6.5 静态事实模型

编译器内部可区分：

```text
Known(T)  : 已知是 T
Unknown   : 类型信息不足
Dynamic   : 显式放弃静态成员检查
```

用户首版不一定直接看到 `Unknown`，但编译器需区分“尚未证明”和“显式 dynamic”。

### 6.6 类型元数据保留到运行时

类型信息不擦除，应成为 Method/Class 等运行时对象的元数据：

```iris
parse.parameters
parse.return_type
User.properties
User.methods
```

它参与：

- runtime contract；
- reflection；
- native/plugin boundary；
- interface conformance；
- JIT guard 与 invalidation。

### 6.7 首版语法方向

变量：

```iris
count: Integer = 0
name: String = "Iris"
```

方法：

```iris
fun greet(name: String) -> String {
    return "Hello, " + name
}
```

无返回值使用 `Nil`，不另设 `void`：

```iris
fun print_name(name: String) -> Nil {
    print(name)
}
```

实例属性：

```iris
class User {
    property name: String
    property age: Integer
}
```

可变参数：

```iris
fun sum(*values: Integer) -> Integer
```

其中方法体内 `values` 的类型为 `Array[Integer]`。

Block/function type：

```iris
fun each(block: (Value) -> Nil) -> Nil
```

具体 block syntax 仍待讨论。

### 6.8 Nilability

首版必须严格区分：

```text
T   : 不包含 nil
T?  : T | Nil
```

例如：

```iris
name: String?

if name != nil {
    print(name.length)
}
```

分支内类型缩窄为 `String`。

不建议默认所有引用类型都允许 nil。

### 6.9 类型关系

#### Class 使用 nominal typing

```iris
class Dog extends Animal
```

则 `Dog <: Animal`。不会因为两个无关类拥有同名方法而自动互相赋值。

#### Interface 作为显式 protocol contract

```iris
interface Printable {
    fun print() -> String
}

class User implements Printable {
    fun print() -> String { ... }
}
```

首版要求显式 `implements`，不自动 structural conformance，以获得更清晰的诊断、metadata、method slot 和 JIT 行为。

### 6.10 `Dynamic` 与 `Object` 必须区分

`Object`：

> 任意 Iris 对象，但静态上只允许使用 Object 契约声明的消息。

`Dynamic`：

> 显式关闭静态成员检查，允许发送任意消息；失败在运行时暴露。

从 `Dynamic` 赋给 `T` 必须生成 checked guard，不能像 TypeScript `any` 一样静默污染。

### 6.11 类型测试与转换

建议提供：

```iris
value is User
value as User
```

- `is` 返回 Bool 并触发 flow narrowing；
- `as` 执行 checked cast，失败抛 TypeError。

旧 `cast` 不再用于 block 调用，恢复类型转换语义或直接弃用。

### 6.12 必须检查的边界

1. typed variable initialization；
2. typed variable assignment；
3. method arguments；
4. method return；
5. property writes；
6. Array/Hash writes；
7. block parameters/returns；
8. native extension return；
9. reflection/plugin invocation；
10. deserialization/external input 进入 typed value。

编译器已证明 `expression: T` 且 target 要求 T 时，可移除重复 runtime check。

### 6.13 类型不能改变动态分派

```iris
value: Animal = Dog.new()
value.speak()
```

仍调用 Dog 的动态实现。类型只约束：

- 消息是否允许发送；
- 参数与返回值契约；
- 可建立哪些 JIT guard。

不根据静态类型改变 method selection，不支持 type-directed overload。

### 6.14 动态类修改与契约

非契约方法可动态增删；被 interface/type contract 引用的方法不能替换为不兼容签名。

动态修改流程建议为：

1. 验证新方法与现有 contract；
2. 更新 method table；
3. 增加 version；
4. invalidates related JIT code；
5. 不兼容则抛 `TypeContractError`。

### 6.15 首版类型能力边界

#### 支持

- parameter / return / local / property / class variable type；
- nominal class subtype；
- explicit interface conformance；
- `T?`；
- `is` narrowing；
- `as` checked cast；
- `Dynamic` / `Object`；
- built-in `Array[T]`；
- built-in `Hash[K, V]`；
- built-in `Range[T]`；
- block/function type；
- variadic element type；
- runtime metadata；
- runtime contract guards；
- typed native/plugin boundary。

#### 暂不支持

- arbitrary union；
- intersection；
- type-directed overload；
- user-defined generics；
- generic constraints；
- variance annotations；
- dependent types；
- conditional/mapped types；
- implicit numeric conversion；
- structural auto-conformance；
- inferred public API contracts；
- compile-time type metaprogramming。

内建 mutable containers 首版全部 invariant，例如 `Array[Dog]` 不是 `Array[Animal]`。

### 6.16 类型错误

类型契约失败应产生普通 Iris 异常对象，例如：

```text
TypeError {
    expected
    actual
    source_location
    boundary
}
```

应能通过 Iris 异常机制捕获。不得通过 Rust panic、assert、abort 或 JIT crash 暴露。

---

## 7. 复活项目的建议顺序

### 阶段 0：冻结目标

明确项目定位：

- 历史项目修复；
- 现代动态语言实验室；
- 面向产品化的独立语言。

当前讨论更接近第二项：以保留 Iris 身份为前提，用 Rust 重建可维护的动态语言与 JIT 实验平台。

### 阶段 1：冻结旧行为

1. 在 Windows 环境恢复旧项目完整构建；
2. 修复 Pointer callback ABI；
3. 排除或明确标记不完整 File Extension；
4. 将 34 个 `.ir` 示例转成 batch golden tests；
5. 保存 stdout/stderr/expected error baseline；
6. 把旧实现作为语义考古与 differential reference，而非新版模块模板。

### 阶段 2：建立新版语言决策清单

优先逐项冻结：

- 前置分号；
- loop syntax；
- block syntax；
- block return；
- exception vocabulary；
- class/module/interface syntax；
- MRO；
- accessor/property；
- Range；
- numeric model；
- dynamic class mutation；
- resource management；
- type contract defaults。

### 阶段 3：Rust 垂直切片

完成 syntax → HIR → bytecode → verifier → interpreter → baseline JIT → differential tests 的最小完整链路。

### 阶段 4：对象模型与 GC

依次加入：

```text
closures
  -> objects/classes
  -> message send
  -> inline cache
  -> arrays/hashes/range
  -> precise non-moving GC
```

### 阶段 5：Module / Interface / Type Contracts

在 MRO 与 runtime metadata 稳定后加入 mixin、protocol contract 和 enforced gradual typing。

### 阶段 6：Extension ABI

以新版 runtime handle / GC root / type contract 设计 native extension SDK，不复刻旧裸指针 C++ ABI。

### 阶段 7：Concurrency 与 Optimizing JIT

最后才实现并发 runtime、并发/增量 GC、optimizing JIT、deoptimization、OSR 等高复杂度能力。

---

## 8. 当前尚待逐项讨论的决策

### 8.1 类型系统下一项关键决策

未标注参数/返回值的默认含义：

- 方案 A：默认为 `Dynamic`；
- 方案 B：编译器局部推断，但不形成 public runtime contract。

当前初步建议：

- public method parameter / return 未标注即 `Dynamic`；
- local variable 允许局部推断；
- 推断结果用于诊断与优化，但不自动成为 runtime contract；
- public contract 只能来自显式标注。

### 8.2 其他待决策项

1. 异常语汇是否保留 `groan/order/serve/ignore`；
2. class method 使用 `fun self.foo` 还是 `class fun foo`；
3. module mixin 关键字；
4. interface/protocol 关键字与 nominal/structural 边界；
5. block 参数和尾随 block 最终语法；
6. block 内 return 语义；
7. Range 最终语法；
8. 数组越界行为；
9. property/accessor 最终语法；
10. numeric overflow、integer width 与 equality semantics；
11. class reopen 与 hierarchy mutation 边界；
12. resource cleanup 模型；
13. pattern matching 是否由 `switch/when` 演进；
14. module/class/interface MRO；
15. typed block/function variance 与 optional block 表达。

---

## 9. 当前推荐的总体技术决策

| 层级 | 当前推荐 |
|---|---|
| 实现语言 | Rust |
| Parser | 手写 lexer + Pratt/recursive descent，或先恢复 grammar 再迁移 |
| Semantic IR | Iris-owned HIR |
| Execution IR | 新的寄存器式 Iris bytecode/MIR |
| 正确性基准 | Bytecode interpreter |
| 第一 JIT | Cranelift baseline JIT |
| 第一版 GC | 单线程、非移动、精确 mark-sweep + explicit roots |
| Dynamic call | generic send → monomorphic inline cache |
| Future optimizing backend | 可选 LLVM |
| CLR | 仅在明确将 Iris 定位为 .NET language 时选择 |
| Concurrency | 后期实现，不复刻旧 STW 模型 |
| Type system | Optional annotations + mandatory static/runtime contracts |
| Dispatch | 始终动态，不做 type-directed overload |

---

## 10. 当前最重要的设计原则

### 语言核心

```text
值
对象
消息发送
词法作用域
闭包
类 / 模块 / Protocol
控制流
异常
```

其余能力尽量从这些正交机制推导，而不是继续增加专用语法。

### 工程顺序

> 先冻结行为与语义，再实现 Rust frontend/runtime；先建立 reference interpreter，再做 baseline JIT；先确保单线程精确 GC 正确，再讨论并发与 optimizing JIT。

### 类型系统

> 标注可以不写，但写出后必须遵守；能证明的错误在编译期拒绝，无法证明的动态边界由运行时 contract 守卫；类型永远不改变动态分派。

---

## 11. 资料与证据入口

- 原始语言介绍：`Document/Iris Programming Language Intro.pdf`
- Notepad++ 关键字定义：`Document/IrisLangHighLight(for NP++).xml`
- 项目知识库：`AGENTS.md`
- 核心引擎说明：`IrisLangLibrary/AGENTS.md`
- 测试语义样例：`Iris Library Test/test script/`
- 完整 extension 旧样例：`Iris Pointer Extension/`
- 核心编译器：`IrisLangLibrary/src/IrisCompiler.cpp`
- Bytecode emitter：`IrisLangLibrary/src/IrisInstructorMaker.cpp`
- VM/runtime：`IrisLangLibrary/src/IrisInterpreter.cpp`
- GC：`IrisLangLibrary/src/IrisInterpreter/IrisNativeModules/IrisGC.cpp`
- Binary bytecode I/O：`IrisLangLibrary/src/IrisUnil/IrisVirtualCodeFile.cpp`
- Host/native API：`IrisLangLibrary/include/IrisExportAPIs/`

本文后续应随语言设计决策持续更新，并将已冻结项逐步迁移到独立的新版 Iris language specification 与 architecture decision records。
