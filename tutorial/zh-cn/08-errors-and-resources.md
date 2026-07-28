# 错误与资源

本章介绍失败和清理。Iris 可以抛出任意对象，不限于某种特殊错误基类。运行时把传播元数据保存在 `ExceptionContext` 中。`try`、`catch` 和 `finally` 是会产生值的控制形式，`using` 是围绕 `Closeable` 构建的普通辅助函数，async 代码使用单一运行时的 `Task<T>` 值和 `await`。

```iris
try {
  raise :failed
} catch value: Symbol, context {
  context.value          // :failed
  context.suppressed     // runtime-owned ExceptionContext list
  raise value from context
}
```

这段代码复用自 `IRIS-V1-CONTROL-EX010`。被抛出的对象是 `Symbol` `:failed`。栈、原因、重新抛出点和被抑制的清理失败属于 `ExceptionContext`，不属于这个 symbol。

## 抛出任意对象

`raise value` 接受任何 Iris 对象或值。每次抛出都会创建一个新的 `ExceptionContext`。捕获会给你原始的被抛出对象，保持不变，并且可以选择同时给出上下文。

```iris
try {
  raise :tag
} catch value: Symbol, context {
  value
}
```

带类型的 `catch` 会在 `catch` 主体运行前收窄。Class 捕获接受子类。Contract 捕获要求显式名义一致性。全捕获必须放在最后，因为捕获会按源码顺序测试。

裸 `raise` 不同于 `raise value`。裸 `raise` 会在 `catch` 的同步动态范围内继续当前传播。在该范围外调用它会抛出 `NoActiveExceptionError`。

## Try、catch 与 finally 产生值

`try` 表达式的暂定值来自 `try` 主体或被选中的 `catch`。正常完成的 `finally` 会在之后运行，并丢弃自己的最终值。

```iris
let result = try {
  "try value"
} finally {
  "ignored final value"
}

result  // "try value"
```

这复用自 `IRIS-V1-CONTROL-EX011`。如果 `finally` 抛出、返回、跳出或继续，它会覆盖挂起中的结果或异常。当清理失败，而另一个异常已经是主异常时，清理上下文会追加到主异常上下文由运行时拥有的被抑制列表中。

## Closeable 与 using

标准资源 Contract 是 `Closeable`，带有普通 `close() -> Nil` Method。它不是魔法语法。辅助函数 `using(resource) { ... }` 会运行代码块，然后通过等价于 `try/finally` 的控制关闭资源。

```iris
let text = using(File.open("data.txt")) { |file: File| -> String
  file.read_all()
}
```

这个示例复用自 `IRIS-V1-ASYNC-EX003`。它只说明形状。因为 Iris 没有实现或标准库发布，所以这里的 `File` 要当作规范中的示例表面，而不是今天能打开的东西。

如果代码块完成且 `close()` 完成，辅助函数返回代码块的值。如果代码块抛出且 `close()` 也抛出，代码块的上下文保持为主异常，关闭上下文变成被抑制的上下文。如果代码块完成但 `close()` 抛出，关闭失败会变成主异常。

## 单线程 async

Iris 中的 async 是协作式且单运行时的。调用 `async fun` 会返回 `Task<T>`。等待该 Task 会得到 `T`，或在该 Task 失败时重新传播捕获的 `ExceptionContext`。

```iris
async fun read_name(path: String) -> String {
  return await File.read_text(path)
}

async fun log_name(path: String) -> Nil {
  print(await read_name(path))
}

let task: Task<String> = read_name("user.ir")
let name: String = await task
```

这段代码复用自 `IRIS-V1-ASYNC-EX001`。无结果 async Method 返回 `Task<Nil>`，不是某种特殊空结果形式。Iris v1 没有取消语义，没有即发即忘签名，也没有 Iris 源码中的隐式阻塞等待。

```iris
async fun value() -> Integer { 7 }

let task = value()
let first = await task
let second = await task
first == second  // true
```

这复用自 `IRIS-V1-ASYNC-EX002`。已完成的 Task 不可变，并且可以被 `await` 多次。失败的 Task 会保留一个捕获的失败上下文；`await` 它会观察到同一个根上下文，并带有 async 诊断链接。

## 静态承诺，动态自由

动态自由：任意对象都可以被抛出，`catch` 选择取决于运行时 Class 或 Contract 一致性，async Tasks 之后通过调度器完成。

静态承诺：`ExceptionContext` 拥有传播元数据，`finally` 和 `using` 有固定清理优先级，`Task<T>` 携带被等待结果的类型，async 交错只发生在已定义的 `await` 或调度器边界。Open 事务不能挂起。

## 阅读规范

精确规则请阅读 [04-bindings-callables-control-flow.md](../../spec/iris-v1/04-bindings-callables-control-flow.md) 和 [07-async-resources-diagnostics.md](../../spec/iris-v1/07-async-resources-diagnostics.md)：

| 条款 | 主题 |
| --- | --- |
| `IRIS-V1-CONTROL-C055` through `IRIS-V1-CONTROL-C067` | `raise`、`catch`、`finally`、值规则和 `ExceptionContext`。 |
| `IRIS-V1-ASYNC-C003` through `IRIS-V1-ASYNC-C010` | Async 可调用表面和 `Task<T>` 结果类型。 |
| `IRIS-V1-ASYNC-C011` through `IRIS-V1-ASYNC-C019` | 单一调度器、挂起，以及没有隐式阻塞等待。 |
| `IRIS-V1-ASYNC-C020` through `IRIS-V1-ASYNC-C029` | Task 完成、失败 Task 传播和未观察失败诊断。 |
| `IRIS-V1-ASYNC-C030` through `IRIS-V1-ASYNC-C038` | `Closeable`、`using`、幂等关闭和跨 async 挂起的清理。 |
