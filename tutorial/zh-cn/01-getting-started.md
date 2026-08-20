# Iris 是什么

本章给出 Iris v1 的心智模型：每个运行时值都是对象，行为是消息发送，动态行为受静态承诺约束。语言本身已经冻结，包括这些章节遵循的所有者批准勘误直到 v1.33。工具链还没有冻结：[`crates/`](../../crates/) 下的参考实现运行语言的一个子集，所以这里有些代码块今天可以执行，另一些则是教学材料。

```bash
cargo build -p iris-cli
./target/debug/iris -e 'print(1 + 2)'
```

```iris
class Counter {
  property value: Integer = 0
  fun add(delta: Integer) -> Integer { @value += delta }
}

let counter = Counter.new()
let bound: BoundMethod<(Integer) -> Integer> = counter.add
```

这个片段改编自 `IRIS-V1-CONTROL-EX003`。注意这个可调用注解：自 v1.11 勘误起，可调用 Type 总是命名它的种类，所以它是 `BoundMethod<(Integer) -> Integer>`，而不是裸的 `(Integer) -> Integer`。

## 每个值都是对象

Iris 从根上就是面向对象的。`Counter` 是一个 Class 对象。`Counter.new()` 向这个 Class 对象发送 `new` 消息。`counter.add` 读取一个实例 Method，并产生一个 BoundMethod，它可以存入可调用绑定并通过 `bound.call(1)` 调用。具名 `fun` 声明没有一个名为 Function 的独立运行时类别。

这是它与许多语言的第一个不同点。Iris 的语法可能看起来熟悉，但语义基于消息。Method 调用、属性读取、具名中缀调用，以及大多数运算符，都是发给某个接收者的发送。

```iris
let total = 1 + 2
let shifted = total << 1
let equal = total == shifted
```

## 运算符是消息发送

`+`、`<<` 和 `==` 形式不是全局函数。它们通过接收者的运行时 Class 和当前活动修订来选择，并受运行时章节规则约束。Iris 也有少量不是可重载消息的核心控制形式，包括 `!`、`&&`、`||`、`&&=` 和 `||=`。

## 身份是显式的

身份是显式的。原语 `same?` 测试两个带身份的操作数是否是同一个对象。它绕过普通 Method 查找、`==`、比较方法和用户替换。并非每个值都有可观察身份，所以规范区分带身份对象与无身份不可变值。Class 对象、Closure 对象和可变容器属于前者，Integer 和 String 值属于后者。

```iris
let first = Object.new()
let second = first
let identity = first same? second
```

当你要表达普通相等协议时，使用 `==`。只有在你要表达对象身份，且操作数带身份时，才使用 `same?`。

## 静态承诺约束动态行为

Iris 也围绕静态与动态的分工来设计。静态注解、已声明超类、可见成员签名、已声明 Contract、类型化属性和泛型约束都是承诺。运行时派发仍然是动态的，但这些承诺不能被动态变更、包升级、反射、原生绑定或优化静默削弱。

```iris
fun add(a: Integer, b: Integer) -> Integer {
  a + b
}

fun dynamic_add(a, b) {
  a + b
}
```

这个片段改编自 `IRIS-V1-TYPES-EX001`。

第一个 Method 在参数和返回值上写出了契约。第二个省略了这些注解，所以它的公共 Method 签名在 `Object` 边界内是动态的。在两种情况下，`a + b` 仍然是动态消息发送。静态类型绝不会选择一个隐藏的重载。

## 静态承诺，动态自由

静态事实约束哪些内容必须保持为真。动态行为决定运行时当前哪个 Method 主体接收消息。Iris 不用静态重载解析、预期返回类型、泛型实参或 union 分支来选择不同的普通选择器。如果你需要 Contract 专属槽位，后续章节会使用显式 Contract 限定派发 `..`。

## 后续章节会讲什么

本教程会一直沿用这种分工。[值与绑定](02-values-and-bindings.md) 从局部名称和字面量值讲起。[控制流](03-control-flow.md) 展示分支、循环和匹配都是产出值的形式。[函数、闭包与块](04-callables-and-closures.md) 解释可调用模型。[类与对象](05-classes-and-objects.md) 回到对象身份、实例状态、继承和构造。

## 试试看

`iris` 二进制可以运行文件、`-e` 字符串或交互式会话。脚本运行器有意不打印脚本的最终值；程序通过它做的事来沟通。交互式会话会打印值，因为那就是它的目的。

```bash
./target/debug/iris hello.iris
./target/debug/iris -e 'print("hello")'
./target/debug/iris                    # then :quit to leave
```

语言的整章内容还没有实现，所以如果某个示例抛出 `unsupported construct`，那是实现中的缺口，不是语言中的缺口。

## 阅读规范

本章简化了以下规范性条款：

- [`IRIS-V1-TRACE-C011`](../../spec/iris-v1/README.md)：规范集不实现 Iris。
- [`IRIS-V1-IDENTITY-C006`](../../spec/iris-v1/01-language-identity.md)：每个运行时值都是对象。
- [`IRIS-V1-IDENTITY-C007`](../../spec/iris-v1/01-language-identity.md)：行为是消息发送。
- [`IRIS-V1-IDENTITY-C008`](../../spec/iris-v1/01-language-identity.md)：静态承诺约束动态行为。
- [`IRIS-V1-IDENTITY-C009`](../../spec/iris-v1/01-language-identity.md)：静态类型选择不会改变普通选择器身份。
- [`IRIS-V1-IDENTITY-C013`](../../spec/iris-v1/01-language-identity.md)：紧凑的动态与静态模型。
- [`IRIS-V1-RUNTIME-C003`](../../spec/iris-v1/03-runtime-object-model.md)：运行时对象性。
- [`IRIS-V1-RUNTIME-C027`](../../spec/iris-v1/03-runtime-object-model.md)：可重载符号运算符是普通 Method 发送。
- [`IRIS-V1-RUNTIME-C029`](../../spec/iris-v1/03-runtime-object-model.md)：`same?` 是原始身份比较。
