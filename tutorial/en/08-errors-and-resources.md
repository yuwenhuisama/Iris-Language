# Errors and Resources

This chapter covers error handling, resource cleanup, and asynchronous operations. Iris allows programs to raise any object rather than restricting exceptions to a narrow class hierarchy. The runtime maintains propagation details inside an `ExceptionContext`. The `try`, `catch`, and `finally` forms produce expressions, deterministic cleanup centers on the `Closeable` contract, and cooperative concurrency uses single-threaded `Task<T>` instances with `await`.

<!-- iris-example: {"id":"08-raising-and-catching","mode":"vm","stdout":"failed operation\nfailed operation\n"} -->
```iris
try {
  raise "failed operation"
} catch val, ctx {
  print(val)
  print(ctx.value)
}
```

Expected terminal output:

```text
failed operation
failed operation
```

The raised object is the string `"failed operation"`. Stack traces, origin locations, and cause links are stored in the runtime-owned `ExceptionContext` rather than polluting the raised payload.

## Raising any object

The expression `raise value` accepts any Iris value or object. Every raise creates a fresh `ExceptionContext`. When handling an exception, the `catch` clause receives the original object and an optional `ExceptionContext`.

Iris supports structured causal chaining using `raise value from context`:

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

Expected terminal output:

```text
outer failure
inner failure
```

A typed catch clause narrows candidates before the catch block executes. Catches evaluate in source order. Bare `raise` re-raises the currently active propagation during the synchronous dynamic extent of a catch.

## Try, catch, and finally produce values

In Iris, `try` is a value-producing expression. Its value resolves to the final expression of the `try` block or the selected `catch` arm. A normally completing `finally` block executes unconditionally and discards its evaluation result.

<!-- iris-example: {"id":"08-try-finally-values","mode":"vm","stdout":"working\n"} -->
```iris
let result = try {
  "working"
} finally {
  "cleanup"
}

print(result)
```

Expected terminal output:

```text
working
```

Exception interaction in `finally` follows precise rules under `IRIS-V1-CONTROL-C064`: if `finally` raises a new value while a pending exception context exists, the new propagation becomes primary and the pending context becomes its causal context (`cause`), unless an explicit `from` specifies otherwise. The pending context is not discarded or placed on a suppressed list in this case. In contrast, bare `raise` in `finally` simply continues propagating the existing pending exception.

Structured records inside `ExceptionContext` provide precise diagnostic information:
- `SourceLocation` provides `path`, 1-based `line`, and 1-based `column`.
- `StackFrame` records `callable_name` and `location`.
- `RaiseSite` tracks re-raise locations.

## Closeable and using

The normative cleanup protocol centers on the `Closeable` contract (`IRIS-V1-ASYNC-C030`), which requires an ordinary method `close() -> Nil`. Under full normative conformance, the standard helper signature is `using(resource: Closeable, &block)` (`IRIS-V1-ASYNC-C032`). Conforming classes declare explicit nominal conformance via `class ... for Closeable` and implement the dedicated slot via `public impl fun Closeable::close() -> Nil` (refer to chapter 06 for the exact syntax and view dispatch rules).

In the current implementation environment, the built-in `Closeable` contract is not yet bound in the global namespace, so referencing `Closeable` directly raises a NameError. The engine instead provides the helper as a name-based runtime cleanup convenience, dispatching to any receiver that exposes a matching `close()` method. The runnable snippet below demonstrates this convenience behavior, but it does not represent standard nominal `Closeable` contract conformance. Note that `using` remains an ordinary method or helper identifier, not a language keyword or special syntax.

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

Expected terminal output:

```text
inside block
resource closed
done
```

Per `IRIS-V1-ASYNC-C033`, if the protected block raises an error and `close()` also raises, the block's exception remains primary while the close failure context is appended to the primary context's suppressed list. If the block completes normally and `close()` raises, the close failure becomes primary.

**Specification-only (not executed):** This I/O illustration requires an external `File` implementation and a `data.txt` fixture, neither supplied here. It shows how the ordinary `using` helper pairs with a resource:

<!-- iris-example: {"id":"08-using-pattern","mode":"spec-only","reason":"Specification example illustrating high-level using resource block syntax"} -->
```iris
let text = using(File.open("data.txt")) { |file: File| -> String
  file.read_all()
}
```

## Single-threaded async

Asynchronous execution in Iris is cooperative, non-preemptive, and hosted on a single-runtime thread scheduler (`IRIS-V1-ASYNC-C011`). Declaring `async fun` marks a method that returns a `Task<T>`.

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

Expected terminal output:

```text
42
```

An async method that does not suspend completes synchronously (`IRIS-V1-ASYNC-C012`). In an asynchronous script context, applying `await` unwinds a task to its completion value or repropagates its failure.

**Specification-only (not executed):** This illustration of cooperative `await` chaining (`IRIS-V1-ASYNC-C003`) requires an external asynchronous `File.read_text` implementation, a `user.ir` fixture, and an async context for the final `await`; these are not supplied here:

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

The `await` operator binds tighter than binary operators and looser than postfix calls. Completed tasks become immutable and can be safely awaited multiple times without re-executing work. Iris v1 contains no implicit blocking waits in script source.

## Static promise, dynamic freedom

Iris ensures reliable cleanup while accommodating runtime failures.

**Dynamic freedom**
- Any object or value can be raised as an exception payload.
- Catch clauses match dynamically against runtime classes or contract conformances.
- Async tasks suspend execution cooperatively and complete later via the scheduler.

**Static promises**
- `ExceptionContext` maintains immutable propagation records and causal history.
- `finally` cleanup guarantees deterministic execution order.
- `Task<T>` signatures enforce the type of the yielded value upon completion.
- Open metadata transactions cannot suspend across asynchronous await points.

**Hands-on Exercise**

Write a function `safe_divide(a: Integer, b: Integer) -> Integer` that raises `"zero_division"` if `b == 0`, or returns `a / b`. Wrap the call in a `try/catch` block that catches the error string and prints `"caught: zero_division"`. Verify the output with `./target/debug/iris --vm divide.iris`.

Expected terminal output:

```text
caught: zero_division
```

## Read the spec

For formal execution semantics and normative grammar rules, refer to [04-bindings-callables-control-flow.md](../../spec/iris-v1/04-bindings-callables-control-flow.md) and [07-async-resources-diagnostics.md](../../spec/iris-v1/07-async-resources-diagnostics.md):

| Clause | Topic |
| --- | --- |
| `IRIS-V1-CONTROL-C055` through `IRIS-V1-CONTROL-C067` | `raise`, `catch`, `finally`, value rules, and `ExceptionContext`. |
| `IRIS-V1-CONTROL-C079` | `SourceLocation`, `StackFrame`, and `RaiseSite` records. |
| `IRIS-V1-GRAMMAR-C071` | `await` as a unary operator and its precedence. |
| `IRIS-V1-ASYNC-C003` through `IRIS-V1-ASYNC-C010` | Async callable surface and `Task<T>` result typing. |
| `IRIS-V1-ASYNC-C011` through `IRIS-V1-ASYNC-C019` | Single scheduler, suspension, and absence of implicit blocking waits. |
| `IRIS-V1-ASYNC-C020` through `IRIS-V1-ASYNC-C029` | Task completion, failure propagation, and unobserved failure diagnostics. |
| `IRIS-V1-ASYNC-C030` through `IRIS-V1-ASYNC-C038` | `Closeable`, `using`, idempotent close, and cleanup across async suspension. |
