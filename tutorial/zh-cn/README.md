# 教程索引

本教程面向有经验的程序员，提供面向实践的 Iris v1 入门指导。它通过具体代码、在 Iris 虚拟机上的验证执行，以及指向冻结规范的精确引用来讲解语言模型。

这里的语言语法是稳定的：v1 规范已经冻结，涵盖直到 v1.33 的所有者批准勘误，塑造了可调用 Type、存储属性、`shared` 类状态、`typeof`、生成器和装饰器的当前形式。这些章节通篇采用当前拼写。

位于 [`crates/`](../../crates/) 的参考实现提供了 `iris` 可执行文件，包含字节码寄存器机（`--vm`）与树遍历参考引擎。虽然编译器与寄存器机目前已经支持核心语法结构，但 Iris 尚未形成完整的生产级工具链发布版。

**快速起步与环境准备**

构建 Iris 需要稳定的 Rust 工具链（参见 [`rust-toolchain.toml`](../../rust-toolchain.toml)）。从工作区根目录克隆代码库并构建 CLI 二进制文件：

```bash
git clone https://github.com/yuwenhuisama/Iris-Language.git
cd Iris-Language
cargo build -p iris-cli
```

在 Unix 系统上，这会生成 `./target/debug/iris`。在 Windows 环境下，会生成 `.\target\debug\iris.exe`。

运行以下任一 `hello.iris` 命令之前，请先用[第 01 章](01-getting-started.md)中的程序创建该文件。`-e` 命令不需要脚本文件。

```bash
./target/debug/iris --vm hello.iris    # 在寄存器机上执行脚本
./target/debug/iris hello.iris         # 在参考引擎上执行脚本（默认）
./target/debug/iris --vm -e 'print(1 + 2)' # 在 VM 上直接执行表达式
./target/debug/iris -e 'print(1 + 2)'  # 在参考引擎上执行命令行表达式
./target/debug/iris                    # 进入交互式参考 REPL 会话，输入 :quit 退出
```

**理解示例元数据与验证契约**

本教程中的每个代码示例在其代码块上方均紧跟精确的验证契约注解：

- `vm`：直接在 `./target/debug/iris --vm` 上运行，预期退出码为 0 且标准输出完全匹配。
- `reference`：在 `./target/debug/iris` 参考引擎上运行，预期退出码为 0 且标准输出完全匹配。
- `expected-error`：在指定引擎（`vm` 或 `reference`）上执行，预期产生非零退出码并匹配特定的标准错误诊断子串。
- `spec-only`：展示语言标准所规定的规范性语法或语义，并伴随明确的可见文字说明。

这些课程中展示的预置 `print(...)` 函数与集合便捷方法仅用于经过验证的教学演示，并不代表已经冻结的标准库。

贯穿整套教程的核心思想记录在 `IRIS-V1-IDENTITY-C008` 中：静态承诺约束动态行为。每个运行时值都是对象，方法派发发生在运行时，而类型、Contract、泛型约束、Class 身份、包身份和元编程策略始终是动态行为必须遵守的静态承诺。

## 章节

本教程分为三个递进阶段：

**起步 (Start)**

- [01. Iris 是什么](01-getting-started.md)：核心心智模型、通过 `--vm` 运行脚本，以及动静契约。
- [02. 值与绑定](02-values-and-bindings.md)：局部绑定（`let`、`mut`）、共享存储、字面量形式与真值性。

**核心 (Core)**

- [03. 控制流](03-control-flow.md)：产出值的 `if` 表达式、带 `break` 的 `while` 循环、`for` 迭代与模式匹配。
- [04. 函数、闭包与块](04-callables-and-closures.md)：具名 Method、匿名 Closure、尾随块通道与 `.call` 调用。
- [05. 类与对象](05-classes-and-objects.md)：Class、实例、存储属性、`@ivar` 存储、继承以及身份与相等性的区分。
- [06. 模块与 Contract](06-modules-and-contracts.md)：Mixin 组合、方法解析顺序 (MRO) 以及限定派发。

**进阶 (Advanced)**

- [07. 渐进类型](07-types-and-generics.md)：类型注解、联合类型、可空性、不变性与泛型约束。
- [08. 错误与资源](08-errors-and-resources.md)：`try`/`catch` 异常处理、确定性资源清理与任务。
- [09. 动态与静态的交汇](09-dynamic-and-static.md)：类重开、原子修订事务与反射策略。
- [10. 下一步](10-where-to-next.md)：导航 14 个规范产物、C ABI 宿主嵌入以及一致性测试向量。

## 如何阅读本教程

建议先阅读第 01 章至第 06 章以掌握核心对象模型与组合机制。第 07 章至第 09 章阐明渐进类型、错误处理与元编程如何在有界动态机制下紧密配合。第 10 章将你的实践知识与形式化规范产物、一致性套件及嵌入接口无缝衔接。

第 01 章至第 05 章中的每个可运行示例都附带经过验证的预期输出，并通过元数据标签（`vm` 或 `reference`）指定执行引擎。每章结尾都包含“阅读规范”一节，列出定义语言行为的规范性条款。如果教程表述与规范有差异，请以规范为准。
