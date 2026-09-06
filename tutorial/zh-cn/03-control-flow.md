# 控制流

本章展示 Iris 如何处理分支、循环和迭代。语言定义了产出值的控制形式，但当前解析器只接受独立的 `match`，不接受将它用于变量初始化。条件通过 `to_bool` 协议计算真假，循环可通过 `break` 提供返回值，`for` 循环为每次迭代绑定新的局部单元，而 `match` 按照自上而下的源码顺序依次匹配各个分支。

## If 表达式产出值

在 Iris 中，`if` 是能产出值的表达式，而不仅仅是语句。它可以出现在变量初始化、返回值或方法调用的实参中。每个分支拥有独立的词法作用域。

<!-- iris-example: {"id":"03-if","mode":"vm","stdout":"1\n"} -->
```iris
let status = if true {
  :ready
} else {
  :waiting
}
print(if status == :ready { 1 } else { 0 })
```

预期输出：

```text
1
```

若条件为 `false` 且省略了 `else` 块，整个表达式求值结果为 `nil`。

## While 循环可以通过 break 返回

`while` 循环在执行每轮迭代之前检查条件。若循环自然结束而未遇到显式的 `break`，其结果值为 `nil`。使用 `break expr` 会立即终止循环，并将 `expr` 作为整个 `while` 表达式的最终求值结果。

<!-- iris-example: {"id":"03-while-break","mode":"vm","stdout":"3\n"} -->
```iris
mut index: Integer = 0
let found = while index < 10 {
  if index == 3 { break index }
  index = index + 1
}
print(found)
```

预期输出：

```text
3
```

关键字 `continue` 直接开启下一轮迭代，不产生值。带标签的循环允许使用 `break label: value` 干净利落地跳出多层嵌套循环。

## For 循环绑定新鲜迭代单元

`for` 循环遍历满足迭代协议的任意对象。对于迭代的每一步，循环变量都会获得一个全新的不可变局部绑定。

<!-- iris-example: {"id":"03-for-loop","mode":"vm","stdout":"6\n"} -->
```iris
mut sum = 0
for value in [1, 2, 3] {
  sum = sum + value
}
print(sum)
```

预期输出：

```text
6
```

由于每次迭代都有独立的变量绑定，在循环体内创建的闭包能够各自捕获独立的变量实例。

## 生成器 yield 惰性迭代器

根据 `IRIS-V1-GRAMMAR-C072`，包含 `yield` 语句的可调用体属于生成器：调用它不会立即执行方法体，而是返回一个 `Iterator<T>`。调用 `.next()` 会推进执行到下一个 `yield`，返回 `Iteration.yield(value)`；在执行完毕后返回 `Iteration.done`。树遍历参考引擎能够直接执行生成器可调用体。

**仅参考引擎示例。** 将以下程序保存为 `generator.iris`，运行时不要添加 `--vm`：

```bash
./target/debug/iris generator.iris
```

<!-- iris-example: {"id":"03-generator-yield","mode":"reference","stdout":"10\n20\n"} -->
```iris
class CounterGenerator {
  public fun steps() -> Nil {
    yield 10
    yield 20
  }
}
let gen = CounterGenerator.new().steps()
print(gen.next().value)
print(gen.next().value)
```

预期输出：

```text
10
20
```

寄存器机 VM 支持集合迭代，但不支持这个生成器挂起示例。

## Match 分支按源码顺序运行

当前解析器和 VM 支持以下独立的 `match`。它对被匹配对象求值一次，然后自上而下严格按源码顺序依次匹配各个分支，没有隐式穿透（`IRIS-V1-CONTROL-C050`）。`else =>` 分支在前面的模式均未命中时作为默认回退分支执行。解析器尚不接受 `let label = match ...` 这样的初始化语法，因此本例改为给已有的可变绑定赋值。

<!-- iris-example: {"id":"03-match","mode":"vm","stdout":"two\n"} -->
```iris
mut label = "none"
match 2 {
  1 => label = "one"
  2 => label = "two"
  else => label = "other"
}
print(label)
```

预期输出：

```text
two
```

每个分支可以执行表达式或代码块，并且模式守卫（`if 条件`）允许在选定分支前施加额外的条件过滤。

## 静态承诺，动态自由

分支条件与循环守卫在运行时通过 `to_bool` 动态求值。然而，分支结果类型的汇聚、模式绑定的作用域以及跳转目标均受到静态检验与约束。

## 把控制形式当作表达式

在实现支持的地方，产出值的控制形式可以消除不必要的可变临时变量：如第一个示例所示，将 `if` 的结果直接赋值给不可变的 `let`。对于 `match`，请保留上面展示的独立形式，因为当前解析器不接受 `match` 初始化表达式。

**实战练习**

编写脚本 `control_test.iris`，使用 `while` 循环计算 `5` 的阶乘：声明 `mut n = 5`，`mut acc = 1`，在 `n > 1` 的条件下循环执行 `acc = acc * n` 与 `n = n - 1`，最后打印 `acc`。使用 `./target/debug/iris --vm control_test.iris` 运行以确认输出 `120`。

## 阅读规范

本章内容简化并对应于以下规范性条款：

- [`IRIS-V1-CONTROL-C039`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：条件真值性求值。
- [`IRIS-V1-CONTROL-C040`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：逻辑运算符短路特性。
- [`IRIS-V1-CONTROL-C041`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：作为值产生形式的 `if`。
- [`IRIS-V1-CONTROL-C043`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：`while`、`break` 与 `continue`。
- [`IRIS-V1-CONTROL-C044`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：`for` 遍历协议。
- [`IRIS-V1-CONTROL-C045`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：`for` 迭代绑定作用域。
- [`IRIS-V1-CONTROL-C048`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：循环标签与定向跳出。
- [`IRIS-V1-CONTROL-C050`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：`match` 分支求值语义。
- [`IRIS-V1-CONTROL-C051`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：模式匹配语法词汇表。
- [`IRIS-V1-CONTROL-C052`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：模式守卫。
- [`IRIS-V1-GRAMMAR-C060`](../../spec/iris-v1/02-lexical-grammar.md)：文法中的 `if` 表达式。
- [`IRIS-V1-GRAMMAR-C072`](../../spec/iris-v1/02-lexical-grammar.md)：`yield` 与生成器语义。
