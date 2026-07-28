# 教程索引

本教程面向有经验的程序员，带你入门 Iris v1。它解释语言模型，展示已通过语法检查的示例，并在你需要精确规则时指向已冻结的规范。Iris v1 目前没有可用实现。这里没有可用于执行 Iris 程序的编译器、解释器、REPL、playground、包管理器或工具链发布版，所以这些示例是教学材料，不是可运行命令。

贯穿整套教程的核心想法很简单：Iris 是动态的，但不是无边界的。每个值都是对象，派发发生在运行时，而类型、Contract、泛型约束、Class 身份、包身份和元编程策略仍是静态承诺，动态行为必须遵守它们。

## 章节

| 章节 | 主题 | 内容 |
| --- | --- | --- |
| [01. Iris 是什么](01-getting-started.md) | 语言身份 | 每个值都是对象、运算符是消息，以及明确的实现状态。 |
| [02. 值与绑定](02-values-and-bindings.md) | 值与名称 | `let`、`mut`、`const`、注解、字面量、`nil` 和真值性。 |
| [03. 控制流](03-control-flow.md) | 分支与循环 | `if`、`while`、`for`、`match`、`break` 和 `continue`。 |
| [04. 函数、闭包与块](04-callables-and-closures.md) | 可调用体 | `fun`、参数、返回类型、Closure 字面量、尾随块和捕获。 |
| [05. 类与对象](05-classes-and-objects.md) | 对象模型 | Class、`self`、`@ivar`、继承、构造、身份和相等性。 |
| [06. 模块与 Contract](06-modules-and-contracts.md) | 组合 | `module` mixin、MRO 顺序、`contract`、`for`、`impl`、Contract 视图和 `..` 派发。 |
| [07. 渐进类型](07-types-and-generics.md) | 类型承诺 | 可选注解、`Object`、union、`?`、转换、泛型、不变性和 `Never`。 |
| [08. 错误与资源](08-errors-and-resources.md) | 失败与清理 | raise 任意对象、`try` 与 `catch`、`ExceptionContext`、`using`、`Closeable`、`Task` 和 `await`。 |
| [09. 动态与静态的交汇](09-dynamic-and-static.md) | 有边界的变更 | `open class`、活动修订、原子发布或回滚、`meta deny`、装饰器和反射策略。 |
| [10. 下一步](10-where-to-next.md) | 规范交接 | 14 个规范产物的地图、建议阅读路径、FFI、Host 嵌入和一致性。 |

## 如何阅读本教程

如果你想先掌握基础语言，请先读第 01 章到第 05 章。第 06 章到第 09 章解释 Iris 为什么不同于许多面向对象语言：组合、Contract、渐进类型和元编程都通过同一组动态与静态边界相互作用。第 10 章会把你带到规范。

每章结尾都有“阅读规范”一节。那些条款 ID 是事实来源。如果本教程听起来比规范简单，请以规范为准。
