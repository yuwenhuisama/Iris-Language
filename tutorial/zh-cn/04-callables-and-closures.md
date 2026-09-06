# 函数、闭包与块

本章解释 Iris 中可调用代码的组织与调用机制。具名 `fun` 声明在类或模块上定义 Method，而不是孤立的全局函数。引用方法会生成 BoundMethod。闭包字面量创建匿名可调用对象并通过引用捕获词法绑定。在所有情况下，可调用值均通过 `.call(...)` 进行调用。

## Callable Types 命名自己的种类

在 Iris 中，可调用类型显式指明其具体的运行时类别：

| 类型 | 代表含义 |
| --- | --- |
| `Closure<(P) -> R>` | 匿名闭包对象 |
| `BoundMethod<(P) -> R>` | 绑定到接收者对象的方法 |
| `Block<(P) -> R>` | 用于块参数的联合别名（`BoundMethod<S> | Closure<S>`） |

与其他泛型类型一样，可调用类型的参数类型是严格不变的（invariant）。

## 具名 fun 声明创建 Method

具名函数使用 `fun` 关键字声明。Class 中的普通 `fun` 定义实例 Method，而 `class fun` 安装到 Class 对象上。在 Module 声明中，`module fun` 安装到 Module 对象上，普通 `fun` 则向组合该模块的宿主提供 mixin Method（`IRIS-V1-CONTROL-C074`）。这不同于可执行 Module 体中的顶层 `fun`，后者安装到该 Module 的 `main` 接收者上（`IRIS-V1-CONTROL-C012`）。

<!-- iris-example: {"id":"04-fun-declaration","mode":"vm","stdout":"36\n"} -->
```iris
module Math {
  public module fun square(n: Integer) -> Integer {
    n * n
  }
}
print(Math.square(6))
```

预期输出：

```text
36
```

由于具名函数是归属于模块或类的方法，它们完全参与常规的动态方法派发。

## 参数和块有显式通道

Iris 中的参数具有清晰的语法通道：

- 必需位置参数：`name: Type`
- 可选位置参数：`name: Type = default`
- 剩余位置参数：`*items: Type`
- 尾随块通道：`&block: Block<(P) -> R>`

块通道为向方法传递代码执行块提供了专门的语法途径。

**仅参考引擎示例。** 将此程序保存为 `blocks.iris`，运行 `./target/debug/iris blocks.iris`。当前 VM 对这个带类型注解的块示例报 `NameError`，参考引擎可以执行。尾随闭包提供的是 `&block`，而不是额外的位置参数；方法通过 `.call(n)` 调用它。

<!-- iris-example: {"id":"04-trailing-block","mode":"reference","stdout":"6\n"} -->
```iris
module Helpers {
  public module fun apply(n: Integer, &block: Block<(Integer) -> Integer>) -> Integer {
    block.call(n)
  }
}
print(Helpers.apply(3) { |x: Integer| -> Integer; x * 2 })
```

预期输出：

```text
6
```

## Closure 字面量是匿名可调用体

闭包字面量使用花括号与竖线语法：`{ |params| -> ReturnType; body }`。

<!-- iris-example: {"id":"04-closure-literal","mode":"vm","stdout":"21\n"} -->
```iris
let mult = { |x: Integer| -> Integer; x * 3 }
print(mult.call(7))
```

预期输出：

```text
21
```

闭包是完整的运行时对象，可以存入变量、作为参数传递给其他方法，并按需执行。

## 调用始终是 call

可调用值——无论是闭包、绑定方法还是块参数——始终通过显式的 `.call(...)` 消息进行调用。

对存储在变量中的可调用对象使用裸括号（例如 `mult(7)`）是非法的；不带显式选择器的圆括号仅保留给接收者上的直接方法发送。

<!-- iris-example: {"id":"04-invocation-call","mode":"vm","stdout":"10\n"} -->
```iris
class Multiplier {
  public fun factor() -> Integer { 10 }
}
let m = Multiplier.new()
let bound = m.factor
print(bound.call())
```

预期输出：

```text
10
```

提取 `m.factor` 会创建一个保留了 `m` 作为接收者目标的 `BoundMethod` 对象。

## Closure 捕获词法单元

闭包通过引用捕获外层作用域中的变量，而不是复制它们的值。在闭包内部所作的修改会反映在原始变量单元中，外层的改变也同样对闭包可见。

<!-- iris-example: {"id":"04-lexical-capture","mode":"vm","stdout":"15\n15\n"} -->
```iris
mut total = 10
let add = { |amount: Integer| -> Integer; total = total + amount; total }
print(add.call(5))
print(total)
```

预期输出：

```text
15
15
```

## Return 留在当前可调用体内

在 Iris 中，`return` 的范围严格限制在直接包围它的可调用体内。

在闭包内部执行 `return` 仅会终止当前闭包的调用，并将值返回给 `.call(...)` 的调用方，不会跳出外层方法栈帧。

<!-- iris-example: {"id":"04-return-local","mode":"vm","stdout":"0\n8\n"} -->
```iris
module Checker {
  public module fun test_val(n: Integer) -> Integer {
    let check = { |x: Integer| -> Integer; if x < 0 { return 0 }; x }
    check.call(n)
  }
}
print(Checker.test_val(-5))
print(Checker.test_val(8))
```

预期输出：

```text
0
8
```

## 静态承诺，动态自由

可调用对象的签名在参数数量与类型上建立了静态边界。然而，因为可调用体本身是一等对象，它们可以在满足接口契约的前提下在运行时动态替换与传递。

**实战练习**

在 `blocks.iris` 中，将尾随闭包从乘以 `2` 改成加上 `4`，保留 `Integer` 参数和返回类型注解。运行 `./target/debug/iris blocks.iris`，确认输出 `7`。

## 阅读规范

本章内容简化并对应于以下规范性条款：

- [`IRIS-V1-CONTROL-C014`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：可调用运行时类别（Method、BoundMethod、Closure）。
- [`IRIS-V1-CONTROL-C015`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：具名 Method 语法与归属。
- [`IRIS-V1-CONTROL-C016`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：Closure 字面量语法。
- [`IRIS-V1-CONTROL-C017`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：省略返回注解规则。
- [`IRIS-V1-CONTROL-C019`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：正常返回与末尾表达式求值。
- [`IRIS-V1-CONTROL-C020`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：局部于可调用体的 return 语义。
- [`IRIS-V1-CONTROL-C022`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：参数声明顺序。
- [`IRIS-V1-CONTROL-C026`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：调用实参绑定。
- [`IRIS-V1-CONTROL-C028`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：通过引用捕获 Closure 外部变量。
- [`IRIS-V1-CONTROL-C030`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：尾随 Closure 块通道。
- [`IRIS-V1-CONTROL-C074`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：`module fun` 安装规则。
- [`IRIS-V1-CONTROL-C075`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：省略返回类型默认为 `Dynamic<Object>`。
- [`IRIS-V1-CONTROL-C076`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：`.call` 作为唯一的调用形式。
- [`IRIS-V1-TYPES-C094`](../../spec/iris-v1/05-types-contracts-generics.md)：`Closure<S>` 与 `BoundMethod<S>` 可调用类型。
- [`IRIS-V1-TYPES-C095`](../../spec/iris-v1/05-types-contracts-generics.md)：`Block<S>` 类型别名。
- [`IRIS-V1-TYPES-C096`](../../spec/iris-v1/05-types-contracts-generics.md)：可调用类型参数的不变性。
