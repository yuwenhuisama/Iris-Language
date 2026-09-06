# 错误与资源

本章介绍错误处理、资源清理与异步并发。Iris 允许程序抛出任意对象，而不局限于特定的异常基类。运行时系统在 `ExceptionContext` 中统一维护异常传播细节。`try`、`catch` 与 `finally` 均作为产生值的表达式参与控制流，确定性清理围绕 `Closeable` 契约构建，而协作式并发则由单运行时线程的 `Task<T>` 与 `await` 驱动。

<!-- iris-example: {"id":"08-raising-and-catching","mode":"vm","stdout":"failed operation\nfailed operation\n"} -->
```iris
try {
  raise "failed operation"
} catch val, ctx {
  print(val)
  print(ctx.value)
}
```

预期终端输出：

```text
failed operation
failed operation
```

被抛出的对象是普通字符串 `"failed operation"`。调用栈、发生位置和成因追溯均由运行时所有的 `ExceptionContext` 独立持有，不污染被抛出的数据对象本身。

## 抛出任意对象

表达式 `raise value` 支持抛出任何 Iris 值或对象。每次抛出都会创建全新的 `ExceptionContext`。在捕获异常时，`catch` 子句不仅可以接收原始对象，还可以选用关联的上下文对象。

Iris 支持使用 `raise value from context` 进行结构化的因果链接：

<!-- iris-example: {"id":"08-chained-cause","mode":"vm","stdout":"outer failure\ninner failure\n"} -->
```iris
try {
  try {
    raise "inner failure"
  } catch err, first_ctx {
    raise "outer failure" from first_ctx
  }
} catch err, second_ctx {
  print(second_ctx.value)
  print(second_ctx.cause.value)
}
```

预期终端输出：

```text
outer failure
inner failure
```

带类型注解的 catch 分支会在进入主体前先行匹配。各个 catch 分支按源码顺序依序检查。单独书写的 `raise` 语句用于在 catch 的同步动态作用域内重新传播当前活跃的异常。

## Try、catch 与 finally 产生值

在 Iris 中，`try` 是一个产生值的完整表达式。其求值结果来自 try 主体的最终表达式，或者被选中的 catch 分支。正常执行完毕的 `finally` 块无条件运行，并自动丢弃其自身的最终计算值。

<!-- iris-example: {"id":"08-try-finally-values","mode":"vm","stdout":"working\n"} -->
```iris
let result = try {
  "working"
} finally {
  "cleanup"
}

print(result)
```

预期终端输出：

```text
working
```

`finally` 的异常交互遵循 `IRIS-V1-CONTROL-C064`：若存在待传播的异常上下文，而 `finally` 抛出新值，则新传播成为主异常，原上下文成为其成因上下文（`cause`），除非显式 `from` 另有指定。此时原上下文既不会丢失，也不会进入抑制列表。相反，`finally` 中单独书写的 `raise` 会继续传播原有的待传播异常。

`ExceptionContext` 内部提供精确的结构化记录：
- `SourceLocation` 包含文件路径 `path`、从 1 起始的行号 `line` 与列号 `column`。
- `StackFrame` 记录可调用体名称 `callable_name` 与位置 `location`。
- `RaiseSite` 追踪每一次重新抛出的代码位置。

## Closeable 与 using

规范定义的清理协议以 `Closeable` 契约（`IRIS-V1-ASYNC-C030`）为核心，要求普通方法 `close() -> Nil`。在完整的规范一致性下，标准辅助函数的签名要求为 `using(resource: Closeable, &block)`（`IRIS-V1-ASYNC-C032`）。符合规范的类通过 `class ... for Closeable` 声明显式标称一致性，并通过 `public impl fun Closeable::close() -> Nil` 实现专属契约槽位（具体语法形式与视图派发规则请参考第 06 章）。

在当前运行环境中，内置的 `Closeable` 契约尚未绑定到全局命名空间，直接引用 `Closeable` 会触发 NameError。当前引擎仅将 `using` 作为基于方法名的运行时清理便利机制提供，直接派发至任何包含 `close()` 方法的接收者。下方的可运行示例仅展示此便捷清理机制，并不代表标准的标称 `Closeable` 契约一致性。注意，`using` 仍是普通方法或辅助函数的标识符，并非语言关键字或特殊语法。

<!-- iris-example: {"id":"08-closeable-using","mode":"vm","stdout":"inside block\nresource closed\ndone\n"} -->
```iris
class ManagedResource {
  mut closed = false
  public fun close() -> Nil {
    if !@closed {
      @closed = true
      print("resource closed")
    }
    nil
  }
}

let res = ManagedResource.new()
let outcome = using(res) { |r|
  print("inside block")
  "done"
}
print(outcome)
```

预期终端输出：

```text
inside block
resource closed
done
```

根据 `IRIS-V1-ASYNC-C033`，若受保护的代码块与 `close()` 均抛出异常，代码块的异常保持主异常地位，关闭失败的上下文追加到主异常上下文的抑制列表。若代码块正常结束而 `close()` 抛出异常，则关闭失败成为主异常。

**仅规范（不执行）：** 此 I/O 示意需要外部 `File` 实现与 `data.txt` 测试文件，本例均未提供。它展示普通 `using` 辅助方法如何与资源配合：

<!-- iris-example: {"id":"08-using-pattern","mode":"spec-only","reason":"Specification example illustrating high-level using resource block syntax"} -->
```iris
let text = using(File.open("data.txt")) { |file: File| -> String
  file.read_all()
}
```

## 单线程 async

Iris 中的异步执行是协作式、非抢占式的，由单运行时线程的调度器管理（`IRIS-V1-ASYNC-C011`）。`async fun` 声明标记返回 `Task<T>` 的方法。

<!-- iris-example: {"id":"08-async-task-host","mode":"vm","stdout":"42\n"} -->
```iris
class Worker {
  public async fun compute() -> Integer {
    42
  }
}

let task = Worker.new().compute()
let result = Host.run(task)
print(result)
```

预期终端输出：

```text
42
```

不发生挂起的异步方法会同步完成（`IRIS-V1-ASYNC-C012`）。在异步脚本上下文中，`await` 取出任务的完成值，或重新传播其失败。

**仅规范（不执行）：** 此协作式 `await` 链式调用示意（`IRIS-V1-ASYNC-C003`）需要外部异步 `File.read_text` 实现、`user.ir` 测试文件以及最后一个 `await` 所在的异步上下文；本例均未提供：

<!-- iris-example: {"id":"08-async-await","mode":"spec-only","reason":"Specification example illustrating async fun signature, await expression, and Task result handling"} -->
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

运算符 `await` 的优先级高于二元操作符，低于后缀方法调用。已经完成的 Task 具有不可变性，可以被安全地多次 `await`，而不会触发重复执行。Iris v1 脚本层面不提供任何隐式阻塞等待。

## 静态承诺，动态自由

Iris 确保资源确定性清理的同时包容运行时的多样失败场景。

**动态自由**
- 任何对象或值均可作为异常载荷被抛出。
- catch 分支支持在运行时动态匹配类的类型层次或契约一致性。
- 异步 Task 在挂起点协作式让出控制权，由调度器择机恢复执行。

**静态承诺**
- `ExceptionContext` 始终保有不可变的传播记录与因果历史。
- `finally` 与清理协议保证严格确定的执行次序。
- `Task<T>` 签名显式固定任务完成时交付的数据类型。
- 元编程开放事务绝不允许跨越异步 `await` 挂起点。

**动手练习**

编写一个函数 `safe_divide(a: Integer, b: Integer) -> Integer`。当 `b == 0` 时抛出 `"zero_division"`，否则返回 `a / b`。在外层通过 `try/catch` 结构捕获该错误字符串并打印 `"caught: zero_division"`。使用 `./target/debug/iris --vm divide.iris` 进行验证。

预期终端输出：

```text
caught: zero_division
```

## 阅读规范

如需查阅形式化语义与规范细节，请参阅 [04-bindings-callables-control-flow.md](../../spec/iris-v1/04-bindings-callables-control-flow.md) 与 [07-async-resources-diagnostics.md](../../spec/iris-v1/07-async-resources-diagnostics.md)：

| 条款 | 主题 |
| --- | --- |
| `IRIS-V1-CONTROL-C055` 至 `IRIS-V1-CONTROL-C067` | `raise`、`catch`、`finally`、求值规则与 `ExceptionContext`。 |
| `IRIS-V1-CONTROL-C079` | `SourceLocation`、`StackFrame` 与 `RaiseSite` 结构记录。 |
| `IRIS-V1-GRAMMAR-C071` | 作为一元运算符的 `await` 及其优先级。 |
| `IRIS-V1-ASYNC-C003` 至 `IRIS-V1-ASYNC-C010` | 异步可调用接口与 `Task<T>` 结果类型。 |
| `IRIS-V1-ASYNC-C011` 至 `IRIS-V1-ASYNC-C019` | 单一调度器、任务挂起与禁止隐式阻塞等待。 |
| `IRIS-V1-ASYNC-C020` 至 `IRIS-V1-ASYNC-C029` | 任务完成、失败传播与未观测失败诊断。 |
| `IRIS-V1-ASYNC-C030` 至 `IRIS-V1-ASYNC-C038` | `Closeable`、`using`、幂等关闭与跨挂起点的清理保障。 |
