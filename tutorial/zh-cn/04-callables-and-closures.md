# 函数、闭包与块

本章解释 Iris 的可调用体。具名 `fun` 声明创建 Method，而不是独立函数对象。从接收者读取 Method 会创建 BoundMethod。Closure 字面量创建带有词法捕获的匿名可调用体。尾随块是通过专用 `&block` 通道传递的 Closure 字面量。

```iris
class Counter {
  fun initialize() -> Nil { @value = 0 }
  fun add(delta: Integer) -> Integer { @value += delta }
}

let counter = Counter.new()
let bound: (Integer) -> Integer = counter.add
let closure: (Integer) -> Integer = { |delta: Integer| -> Integer; counter.add(delta) }
```

这个片段改编自 `IRIS-V1-CONTROL-EX003`。

## 具名 fun 声明创建 Method

Method 声明以 `fun` 开始，前面可以有可见性、`override`、`impl`、`async` 或 `class` 等修饰符，位置必须符合语法允许处。放置位置决定所有权。在 Class 中，`fun` 创建实例 Method。`class fun` 在 Class 对象上创建单例 Method。在 Module 体内，顶层 `fun` 附着到该 Module 的 `main` 接收者。

```iris
class Greeter {
  fun initialize(name: String) -> Nil { @name = name }
  fun label() -> String { @name }
  fun rename(name: String) -> Nil { @name = name }
}
```

## 参数和块有显式通道

参数语法是显式的。必需位置参数写作 `name: Type`。可选位置参数添加 `= default`。位置 rest 参数使用 `*items: Type`。Keyword-only 参数以 `key` 开头。Keyword rest 使用 `**options: Type`。块通道使用 `&block: (P) -> R`，并可通过 `= nil` 变为可选。

```iris
fun render(
  title: String,
  count: Integer = 1,
  *items: String,
  key path: String,
  &block: (String) -> Nil = nil
) -> Nil {
  if block != nil { block(title) }
}
```

调用使用括号传递普通实参。关键字实参写作 `name: value`。调用后面的尾随 Closure 不是最后一个位置实参；它通过目标的 `&block` 参数提供。

```iris
render("report", path: "out.txt") { |line: String| -> Nil
  line.to_string()
}
```

## Closure 字面量是匿名可调用体

Closure 字面量使用 `{ |parameters| -> ReturnType body }`。单行主体在分号之后开始。多行主体在头部终止符之后开始。空参数使用 `||`。只有当预期可调用类型提供唯一闭合函数类型时，Closure 返回注解才可以省略，所以教程示例会显式写出它。

```iris
let identity: (String) -> String = { |value: String| -> String; value }
let answer: () -> Integer = { || -> Integer; 42 }
```

## Closure 捕获词法单元

Closure 按引用捕获词法绑定单元。如果被捕获的绑定可变，那么 Closure 内的写入和外层作用域的写入会作用在同一个单元上。在拥有接收者的代码中创建的 Closure 还会捕获当前接收者关系，所以原始 `@name` 会继续指向那个接收者。

```iris
mut total: Integer = 0
let add: (Integer) -> Integer = { |value: Integer| -> Integer
  total += value
  return total
}
```

这个片段改编自 `IRIS-V1-CONTROL-EX005`。

## Return 留在当前可调用体内

`return` 只作用于当前可调用体。在 Method 中，它退出该 Method 帧。在 Closure 中，它只退出该 Closure 调用。Closure 不能用 `break` 或 `continue` 跨过自己的调用边界跳进外层循环。

```iris
fun choose(value: Integer) -> Integer {
  let normalize: (Integer) -> Integer = { |item: Integer| -> Integer
    if item < 0 { return 0 }
    item
  }
  normalize(value)
}
```

## 静态承诺，动态自由

可调用类型在绑定后写作 `(P1, P2) -> R`。该类型描述调用形状和结果承诺。可调用值可以是 BoundMethod 或 Closure，但实参检查、返回检查、块形状、关键字名称和元数仍遵循声明契约。

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
- [`IRIS-V1-TYPES-C036`](../../spec/iris-v1/05-types-contracts-generics.md)：可调用 Type。
