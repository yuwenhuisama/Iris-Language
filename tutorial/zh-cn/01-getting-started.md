# Iris 是什么

本章介绍 Iris v1 的核心执行模型：每个运行时值都是对象，行为是消息发送，动态行为受静态承诺约束。语言规范已经冻结，包含直到 v1.33 的所有者批准勘误。

位于 [`crates/`](../../crates/) 的参考实现提供了命令行程序 `iris`，包含字节码虚拟机（`--vm`）与参考引擎。

**环境准备与获取代码库**

构建 Iris 需要稳定的 Rust 工具链（参见 [`rust-toolchain.toml`](../../rust-toolchain.toml)）。克隆官方代码仓库并进入工作区根目录：

```bash
git clone https://github.com/yuwenhuisama/Iris-Language.git
cd Iris-Language
```

**构建工具链**

使用 Cargo 从源码编译命令行可执行文件：

```bash
cargo build -p iris-cli
```

在 Unix 系统上，这会生成 `./target/debug/iris`。在 Windows 环境下，会生成 `.\target\debug\iris.exe`。

**运行你的第一个脚本**

创建一个名为 `hello.iris` 的文件：

<!-- iris-example: {"id":"01-hello","mode":"vm","stdout":"Hello, Iris!\n"} -->
```iris
print("Hello, Iris!")
```

使用寄存器机明确执行该脚本：

```bash
./target/debug/iris --vm hello.iris
```

在 Windows 上：

```cmd
.\target\debug\iris.exe --vm hello.iris
```

终端标准输出为：

```text
Hello, Iris!
```

若要使用默认的树遍历参考引擎运行该脚本，省略 `--vm` 参数即可：

```bash
./target/debug/iris hello.iris
```

**运行短表达式与 REPL**

你可以使用 `-e` 参数在命令行通过任一引擎直接对表达式求值：

```bash
./target/debug/iris --vm -e 'print(1 + 2)'  # 在字节码 VM 上执行
./target/debug/iris -e 'print(1 + 2)'       # 在参考引擎上执行
```

若不带参数运行 `./target/debug/iris`，将启动参考引擎上的交互式 REPL 会话。交互式 REPL 会自动在每行表达式求值后打印其值。相比之下，无论在 VM 还是参考引擎的脚本执行模式下，都需要使用显式的 `print(...)` 调用来输出内容。在 REPL 中输入 `:quit` 可退出会话。

## 每个值都是对象

Iris 从根基上就是面向对象的。每个运行时值在概念上都是对象，`Object` 是所有类型的顶层根类型。正如 `IRIS-V1-RUNTIME-C008` 所明确规定的，这一对象性模型允许运行时实现出于性能优化目的采用未经装箱（unboxed）的原始表示或立即数，只要可观察的对象身份分类与类型行为保持完全一致即可。

当你写下 `1 + 2` 时，整数 `1` 接收到了带有参数 `2` 的消息 `+`。表达式不会调用独立的外部裸函数，而是通过接收者的类型进行派发。

## 运算符是消息发送

算术与比较运算符对应于接收者对象上的消息发送。表达式 `total << 1` 调用 `total` 上的 `<<` 方法。同理，`==` 派发相等性检查方法。

<!-- iris-example: {"id":"01-operators","mode":"vm","stdout":"6\nfalse\n"} -->
```iris
let total = 1 + 2
let shifted = total << 1
print(shifted)
print(total == shifted)
```

预期输出：

```text
6
false
```

Iris 保留了极少数不作为消息发送的短路控制形式：`!`、`&&`、`||`、`&&=` 与 `||=`。

## 身份是显式的

相等性（`==`）与对象身份（`same?`）是两种不同的操作。`==` 按照相等性协议检查两个对象是否具有等价的值。

原语运算符 `same?` 测试的是可观察的对象身份，而不是底层的内存指针地址（`IRIS-V1-RUNTIME-C029`）。它只接受带有身份的操作数（例如可变集合、类以及实例）。如果任一操作数是不带身份的值（例如 Integer 或 Float），`same?` 会立即抛出 `IdentityError`。

<!-- iris-example: {"id":"01-identity","mode":"vm","stdout":"true\nfalse\n"} -->
```iris
let first = [1, 2]
let second = first
let third = [1, 2]
print(first same? second)
print(first same? third)
```

预期输出：

```text
true
false
```

`first` 与 `second` 引用同一个带身份的数组对象，因此 `first same? second` 得到 `true`。虽然 `third` 包含相同的元素，但它是独立的数组实例，因此 `first same? third` 得到 `false`。

## 静态承诺约束动态行为

Iris 将动态派发与静态边界结合。方法签名确立了调用方契约的静态承诺，但运行时的派发依然保持动态。静态类型不会在编译期触发方法重载选择。

<!-- iris-example: {"id":"01-static-promise","mode":"vm","stdout":"30\n"} -->
```iris
module Calculator {
  public module fun add(a: Integer, b: Integer) -> Integer {
    a + b
  }
}
print(Calculator.add(10, 20))
```

预期输出：

```text
30
```

类型注解 `a: Integer` 与 `b: Integer` 在跨越边界时校验输入。但在底层，`a + b` 的求值仍然是向 `a` 所指向的对象发送消息 `+`。

## 静态承诺，动态自由

静态保证能够防止程序无声无息地滑入非法状态。动态派发则赋予了运行时灵活性，且无需复杂的静态重载解析规则。类型注解绝不会用于挑选隐藏的备用方法实现。

## 后续章节会讲什么

接下来的章节将逐步展开这一执行模型：

- [值与绑定](02-values-and-bindings.md) 介绍不可变 `let`、可变 `mut`、字面量形式与真值性。
- [控制流](03-control-flow.md) 探讨 `if`、`while`、`for` 与 `match` 表达式。
- [函数、闭包与块](04-callables-and-closures.md) 解析方法定义、匿名闭包与尾随块。
- [类与对象](05-classes-and-objects.md) 说明对象实例化、实例状态与继承机制。

## 试试看

**实战练习**

编写一个名为 `math_test.iris` 的脚本：绑定整数 `x = 40`，加 `2`，将结果左移 1 位（`<< 1`），并打印输出。使用 `./target/debug/iris --vm math_test.iris` 运行。

预期终端输出：

```text
84
```

## 阅读规范

本章内容简化并对应于以下规范性条款：

- [`IRIS-V1-TRACE-C011`](../../spec/iris-v1/README.md)：规范集不实现 Iris。
- [`IRIS-V1-IDENTITY-C006`](../../spec/iris-v1/01-language-identity.md)：每个运行时值都是对象。
- [`IRIS-V1-IDENTITY-C007`](../../spec/iris-v1/01-language-identity.md)：行为是消息发送。
- [`IRIS-V1-IDENTITY-C008`](../../spec/iris-v1/01-language-identity.md)：静态承诺约束动态行为。
- [`IRIS-V1-IDENTITY-C009`](../../spec/iris-v1/01-language-identity.md)：静态类型选择不改变普通选择器身份。
- [`IRIS-V1-IDENTITY-C013`](../../spec/iris-v1/01-language-identity.md)：紧凑的动态与静态模型。
- [`IRIS-V1-RUNTIME-C003`](../../spec/iris-v1/03-runtime-object-model.md)：运行时对象性。
- [`IRIS-V1-RUNTIME-C027`](../../spec/iris-v1/03-runtime-object-model.md)：可重载符号运算符是普通 Method 发送。
- [`IRIS-V1-RUNTIME-C029`](../../spec/iris-v1/03-runtime-object-model.md)：`same?` 是原始身份比较。
