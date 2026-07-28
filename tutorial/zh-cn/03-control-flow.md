# 控制流

本章展示 Iris 如何在路径之间做选择。`if`、循环和 `match` 都是产出值的形式，并受你前面见过的同一对象模型管辖。条件调用 `to_bool`，循环可以通过 `break` 产出值，`for` 会绑定每轮迭代的新鲜单元，`match` 使用按源码顺序排列的模式分支，并可带可选 guard。

```iris
let label = match value {
  nil => "none"
  true => "yes"
  _ => "other"
}
```

这个片段复用自 `IRIS-V1-CONTROL-EX009`。

## If 表达式产出值

`if` 会产出值。每个分支都有自己的词法作用域。被选分支贡献它的最终表达式；如果被选主体没有产出值的语句，则贡献 `nil`。如果没有 `else` 且条件为 false，缺失分支贡献 `nil`。

```iris
let status = if user.ready? {
  :ready
} else {
  :waiting
}
```

`else if` 链只是一个 `else` 后面跟另一个 `if`。每个测试只求值一次条件，并且 `to_bool` 必须返回实际 Bool。任意真值性本身不会收窄类型，所以当后续代码依赖更窄类型时，请使用显式检查。

```iris
let name: String? = load_name()

let label = if name != nil {
  name
} else {
  "anonymous"
}
```

## While 循环可以通过 break 返回

`while` 在每轮迭代前测试条件。自然完成，包括零次迭代，产出 `nil`。`break expr` 退出循环，并让 `expr` 成为循环结果。`continue` 开始下一轮迭代且没有值。

```iris
mut index: Integer = 0
let found = while index < limit {
  if index == target { break index }
  index += 1
}
```

标签让 `break` 和 `continue` 可以指向外层循环。标签紧挨着出现在 `while` 或 `for` 前面。带标签的 `break` 使用 `break label: value`；带标签的 `continue` 使用 `continue label`。

```iris
outer: while keep_running {
  while ready {
    if done { break outer: :done }
    continue
  }
}
```

## For 循环绑定新鲜迭代单元

`for pattern in iterable` 对 iterable 求值一次，并通过语言迭代协议遍历它。循环体在每轮迭代中接收新鲜的不可变绑定。这对 Closure 很重要：逃逸的 Closure 捕获它自己那轮迭代的单元，而不是一个共享循环变量。

```iris
mut callbacks: Array<() -> Integer> = []
for value in 1 ..= 3 {
  callbacks.append({ || -> Integer; value })
}
```

这个片段改编自 `IRIS-V1-CONTROL-EX008`。

## Match 分支按源码顺序运行

`match` 对 scrutinee 求值一次，然后按源码顺序尝试各个 arm。没有 fallthrough。动态或开放域需要显式 `else` fallback，除非 arm 集可证明穷尽。

```iris
let result = match item {
  nil => :missing
  is String text if text.length() > 0 => :text
  _ => :other
}
```

模式可以匹配字面量、`nil`、Bool 值、带可选绑定的名义类型测试、alternatives、Tuple 模式、Array 模式、绑定名和 `_`。Guard 会在模式形状成功后、临时绑定存在后运行。如果 guard 为 false，这些临时绑定会被丢弃，匹配继续。

```iris
let kind = match pair {
  (:ok, value) => value
  (:error, _) => nil
  else => nil
}
```

## 静态承诺，动态自由

控制流是动态的，因为条件、guard 和逻辑运算符会在运行时调用 `to_bool`。它也有静态承诺，因为分支结果类型、循环结果类型、确定赋值、match 穷尽性、模式绑定和无效控制目标仍按规范规则检查。

## 把控制形式当作表达式

关键习惯是把控制形式当成具有值和类型的表达式。会 raise 或调用返回 `Never` 的 Method 的分支不贡献正常结果。`break value` 会贡献到循环的结果类型。`match` arm 的绑定只在被选 arm 中存在。

## 阅读规范

本章简化了以下规范性条款：

- [`IRIS-V1-CONTROL-C039`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：条件真值性。
- [`IRIS-V1-CONTROL-C040`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：逻辑运算符行为。
- [`IRIS-V1-CONTROL-C041`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：`if` 作为产出值的形式。
- [`IRIS-V1-CONTROL-C043`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：`while`、`break` 和 `continue`。
- [`IRIS-V1-CONTROL-C044`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：`for` 遍历。
- [`IRIS-V1-CONTROL-C045`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：`for` 解构和逐迭代绑定。
- [`IRIS-V1-CONTROL-C048`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：循环标签。
- [`IRIS-V1-CONTROL-C050`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：`match` 选择和 fallback。
- [`IRIS-V1-CONTROL-C051`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：v1 模式词汇。
- [`IRIS-V1-CONTROL-C052`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：match guards。
