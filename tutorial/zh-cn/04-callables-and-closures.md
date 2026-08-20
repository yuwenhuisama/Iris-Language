# 函数、闭包与块

本章解释 Iris 的可调用体。具名 `fun` 声明创建 Method，而不是独立函数对象。从接收者读取 Method 会创建 BoundMethod。Closure 字面量创建带有词法捕获的匿名可调用体。尾随块是通过专用 `&block` 通道传递的 Closure 字面量。每个可调用值都通过 `.call(...)` 调用。

```iris
class Counter {
  property value: Integer = 0
  fun add(delta: Integer) -> Integer { @value += delta }
}

let counter = Counter.new()
let bound: BoundMethod<(Integer) -> Integer> = counter.add
let closure: Closure<(Integer) -> Integer> = { |delta: Integer| -> Integer; counter.add(delta) }

bound.call(1)
closure.call(2)
```

这个片段改编自 `IRIS-V1-CONTROL-EX003`。

## Callable Types 命名自己的种类

可调用 Type 从不是裸签名。签名 `(P1, P2) -> R` 是一个组成部分，Type 会把它包在所描述的可调用种类中：

| Type | 持有 |
| --- | --- |
| `Closure<(P) -> R>` | 一个 Closure 值 |
| `BoundMethod<(P) -> R>` | 一个 BoundMethod 值 |
| `Block<(P) -> R>` | 两者之一，用于块通道 |

`Block<S>` 是语言核心 Type alias，声明为 `type Block<S> = BoundMethod<S> | Closure<S>`。它位于 `Kernel` Module 中，而 `Kernel` 被组合进 `Object`，所以无需 import 就处处可见。

Callable Type 实参是不变的，和所有其他泛型实参完全一样。`Closure<(Integer) -> Object>` 不能赋给 `Closure<(Integer) -> Symbol>`，反向也不行。兼容性在调用点根据被调用可调用体的声明签名检查，而不是通过 callable Types 之间的 variance 检查。

## 具名 fun 声明创建 Method

Method 声明以 `fun` 开始，前面可以有可见性、`override`、`impl`、`async`、`class` 或 `module` 等修饰符，位置必须符合语法允许处。放置位置决定所有权。在 Class 中，`fun` 创建实例 Method。`class fun` 在 Class 对象上创建单例 Method。在 Module 声明中，`module fun` 会在 Module 对象自身上安装 Method，而未修饰的 `fun` 是提供给组合该 Module 的对象的实例 Method。

```iris
module Config {
  module fun default_path() -> String { "iris.toml" }
  fun describe() -> String { "config" }
}

Config.default_path()
```

`class` 和 `module` 在同一个声明上互斥。

## 参数和块有显式通道

参数语法是显式的。必需位置参数写作 `name: Type`。可选位置参数添加 `= default`。位置 rest 参数使用 `*items: Type`。Keyword-only 参数以 `key` 开头。Keyword rest 使用 `**options: Type`。块通道使用 `&block: Block<(P) -> R>`，并可通过 `= nil` 变为可选。

```iris
fun render(
  title: String,
  count: Integer = 1,
  *items: String,
  key path: String,
  &block: Block<(String) -> Nil> = nil
) -> Nil {
  if block != nil { block.call(title) }
}
```

省略可选块会绑定 `nil`，所以 `block != nil` 是 `block.call(...)` 前的存在性测试。

调用使用括号传递普通实参。关键字实参写作 `name: value`。调用后面的尾随 Closure 不是最后一个位置实参；它通过目标的 `&block` 参数提供。在一次调用中同时通过 `&saved` 和尾随块传递 Closure 是 `ArgumentError`。

```iris
render("report", path: "out.txt") { |line: String| -> Nil
  line.to_string()
}
```

## Closure 字面量是匿名可调用体

Closure 字面量使用 `{ |parameters| -> ReturnType body }`。单行主体在分号之后开始。多行主体在头部终止符之后开始。空参数使用 `||`。只有当预期可调用类型提供唯一闭合函数类型时，Closure 返回注解才可以省略，所以教程示例会显式写出它。

```iris
let identity: Closure<(String) -> String> = { |value: String| -> String; value }
let answer: Closure<() -> Integer> = { || -> Integer; 42 }

answer.call()
```

## 调用始终是 call

`call` 是 Closure、BoundMethod 和块值的唯一调用拼写。不能把实参列表直接应用到可调用值上：`closure(1)` 不是调用。只有 Method 发送，例如 `f(1)` 或 `obj.m(1)`，才使用裸调用语法，并且普通发送始终要求括号，`f 1` 会以 `PARSE_CALL_REQUIRES_PARENTHESES` 被拒绝。

```iris
fun apply(value: Integer, &block: Block<(Integer) -> Integer>) -> Integer {
  block.call(value)
}

let doubled = apply(21) { |value: Integer| -> Integer; value * 2 }
```

因为 `call` 是身份承载可调用对象上的普通选择器，它像任何其他消息一样派发，并按普通参数规则绑定实参。

## Closure 捕获词法单元

Closure 按引用捕获词法绑定单元。如果被捕获的绑定可变，那么 Closure 内的写入和外层作用域的写入会作用在同一个单元上。在拥有接收者的代码中创建的 Closure 还会捕获当前接收者关系，所以原始 `@name` 会继续指向那个接收者。

```iris
mut total: Integer = 0
let add: Closure<(Integer) -> Integer> = { |value: Integer| -> Integer
  total += value
  return total
}
```

这个片段改编自 `IRIS-V1-CONTROL-EX005`。

Method 声明不是 Closure。如果一个 Method 读取只存在于外围主体局部中的名称，它不会捕获该名称；声明会被接受，而读取会在 Method 运行时引发 `NameError`。

## Return 留在当前可调用体内

`return` 只作用于当前可调用体。在 Method 中，它退出该 Method 帧。在 Closure 中，它只退出该 Closure 调用。Closure 不能用 `break` 或 `continue` 跨过自己的调用边界跳进外层循环；那就是 `CONTROL_TARGET_CROSSES_CLOSURE`。

```iris
fun choose(value: Integer) -> Integer {
  let normalize: Closure<(Integer) -> Integer> = { |item: Integer| -> Integer
    if item < 0 { return 0 }
    item
  }
  normalize.call(value)
}
```

当 Method 省略 `-> ReturnType` 时，它声明的返回 Contract 是 `Dynamic<Object>`。实现可以使用主体的最终表达式来做局部诊断，但绝不会发布推断出的更窄返回类型。

## 静态承诺，动态自由

可调用 Type 描述调用形状、结果承诺，以及现在的可调用种类。背后的值只有在 Type 允许该种类时才可以是 BoundMethod 或 Closure，而实参检查、返回检查、块形状、关键字名称和元数仍遵循声明契约。

这个模型会引出[类与对象](05-classes-and-objects.md)：Class 安装 Method，实例产生 BoundMethod，Closure 让行为可以移动，同时让词法捕获保持显式。

## 阅读规范

本章简化了以下规范性条款：

- [`IRIS-V1-CONTROL-C014`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：Method、BoundMethod 和 Closure 是可调用运行时种类。
- [`IRIS-V1-CONTROL-C015`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：具名 Method 语法和所有权。
- [`IRIS-V1-CONTROL-C016`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：Closure 语法。
- [`IRIS-V1-CONTROL-C017`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：省略 Method 和 Closure 返回注解的规则。
- [`IRIS-V1-CONTROL-C019`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：正常返回和最终表达式值。
- [`IRIS-V1-CONTROL-C020`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：可调用体本地 return 和跨 Closure 的无效循环转移。
- [`IRIS-V1-CONTROL-C022`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：参数声明顺序。
- [`IRIS-V1-CONTROL-C026`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：调用实参绑定。
- [`IRIS-V1-CONTROL-C028`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：Closure 按引用捕获。
- [`IRIS-V1-CONTROL-C030`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：尾随 Closure 块通道。
- [`IRIS-V1-CONTROL-C074`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：`module fun` installation。
- [`IRIS-V1-CONTROL-C075`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：省略返回注解是 `Dynamic<Object>`。
- [`IRIS-V1-CONTROL-C076`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：`call` 作为唯一调用拼写。
- [`IRIS-V1-TYPES-C094`](../../spec/iris-v1/05-types-contracts-generics.md)：`Closure<S>` 和 `BoundMethod<S>` callable Types。
- [`IRIS-V1-TYPES-C095`](../../spec/iris-v1/05-types-contracts-generics.md)：`Block<S>` alias 和块通道。
- [`IRIS-V1-TYPES-C096`](../../spec/iris-v1/05-types-contracts-generics.md)：callable Type 实参是不变的。
