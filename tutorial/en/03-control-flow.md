# Control Flow

This chapter shows how Iris chooses among paths. `if`, loops, and `match` are value-producing forms governed by the same object model you saw earlier. Conditions call `to_bool`, loops can produce values through `break`, `for` binds fresh per-iteration cells, and `match` uses source-order pattern arms with optional guards.

```iris
let label = match value {
  nil => "none"
  true => "yes"
  _ => "other"
}
```

This snippet is reused from `IRIS-V1-CONTROL-EX009`.

## If expressions produce values

`if` is a real expression, not a statement with a value bolted on. It appears wherever a primary expression is allowed, including as a call argument or the right-hand side of an assignment. Each branch has its own lexical scope. The selected branch contributes its final expression, or `nil` if the selected body has no value-producing statement. If there is no `else` and the condition is false, the missing branch contributes `nil`.

```iris
let status = if user.ready? {
  :ready
} else {
  :waiting
}

print(if status == :ready { 1 } else { 0 })
```

An `else if` chain is just an `else` followed by another `if`. Conditions evaluate once per test and must return an actual Bool from `to_bool`. Arbitrary truthiness doesn't narrow a type by itself, so use explicit checks when the following code depends on a narrower type.

```iris
let name: String? = load_name()

let label = if name != nil {
  name
} else {
  "anonymous"
}
```

## While loops can return through break

`while` tests before each iteration. Natural completion, including zero iterations, yields `nil`. `break expr` exits the loop and makes `expr` the loop result. `continue` starts the next iteration and has no value.

```iris
mut index: Integer = 0
let found = while index < limit {
  if index == target { break index }
  index += 1
}
```

Labels let `break` and `continue` target an outer loop. A label appears immediately before `while` or `for`. Labeled `break` uses `break label: value`; labeled `continue` uses `continue label`.

```iris
outer: while keep_running {
  while ready {
    if done { break outer: :done }
    continue
  }
}
```

## For loops bind fresh iteration cells

`for pattern in iterable` evaluates the iterable once and traverses it through the language iteration protocol. The loop body receives fresh immutable bindings for each iteration. That matters for Closures: an escaped Closure captures the cell for its own iteration, not one shared loop variable.

```iris
mut callbacks: Array<Closure<() -> Integer>> = []
for value in 1 ..= 3 {
  callbacks.append({ || -> Integer; value })
}
```

This snippet is adapted from `IRIS-V1-CONTROL-EX008`.

## Generators yield a lazy iterator

A callable whose body contains `yield` is a generator. Invoking it does not run the body: it returns an `Iterator<T>`. Each `next()` resumes the body until the next `yield`, answering `Iteration.yield(value)`, and answers `Iteration.done` once the body completes. That is the same protocol `for` traverses, so a generator can drive a `for` loop directly.

```iris
fun counting(limit: Integer) -> Iterator<Integer> {
  mut index = 0
  while index < limit {
    yield index
    index += 1
  }
}

for value in counting(3) {
  print(value)
}
```

`yield` sits at the same precedence as `await`, and like `await` it is forbidden inside an open or revision transaction body, because those bodies must not suspend.

## Match arms run in source order

`match` evaluates the scrutinee once, then tries arms in source order. There is no fallthrough. Dynamic or open-ended domains need an explicit `else` fallback unless the arm set is provably exhaustive.

```iris
let result = match item {
  nil => :missing
  is String text if text.length() > 0 => :text
  _ => :other
}
```

Patterns can match literals, `nil`, Bool values, nominal type tests with optional binding, alternatives, Tuple patterns, Array patterns, binding names, and `_`. Arms are separated by a newline or a comma, so a compact `match` fits on one line. A guard runs after the pattern shape succeeds and after provisional bindings exist. If the guard is false, those provisional bindings are discarded and matching continues.

```iris
let kind = match pair {
  (:ok, value) => value
  (:error, _) => nil
  else => nil
}
```

## Static promise, dynamic freedom

Control flow is dynamic because conditions, guards, and logical operators call `to_bool` at runtime. It is statically promised because branch result types, loop result types, definite assignment, match exhaustiveness, pattern bindings, and invalid control targets are still checked by the rules in the spec.

## Treat control forms as expressions

The important habit is to treat control forms as expressions whose value and type matter. A branch that raises or calls a `Never` returning Method contributes no normal result. A `break value` contributes to the loop's result type. A `match` arm binding lives only in the selected arm.

## Read the spec

This chapter simplifies these normative clauses:

- [`IRIS-V1-CONTROL-C039`](../../spec/iris-v1/04-bindings-callables-control-flow.md): condition truthiness.
- [`IRIS-V1-CONTROL-C040`](../../spec/iris-v1/04-bindings-callables-control-flow.md): logical operator behavior.
- [`IRIS-V1-CONTROL-C041`](../../spec/iris-v1/04-bindings-callables-control-flow.md): `if` as a value-producing form.
- [`IRIS-V1-CONTROL-C043`](../../spec/iris-v1/04-bindings-callables-control-flow.md): `while`, `break`, and `continue`.
- [`IRIS-V1-CONTROL-C044`](../../spec/iris-v1/04-bindings-callables-control-flow.md): `for` traversal.
- [`IRIS-V1-CONTROL-C045`](../../spec/iris-v1/04-bindings-callables-control-flow.md): `for` destructuring and per-iteration bindings.
- [`IRIS-V1-CONTROL-C048`](../../spec/iris-v1/04-bindings-callables-control-flow.md): loop labels.
- [`IRIS-V1-CONTROL-C050`](../../spec/iris-v1/04-bindings-callables-control-flow.md): `match` selection and fallback.
- [`IRIS-V1-CONTROL-C051`](../../spec/iris-v1/04-bindings-callables-control-flow.md): v1 pattern vocabulary.
- [`IRIS-V1-CONTROL-C052`](../../spec/iris-v1/04-bindings-callables-control-flow.md): match guards.
- [`IRIS-V1-GRAMMAR-C060`](../../spec/iris-v1/02-lexical-grammar.md): `if` in expression position.
- [`IRIS-V1-GRAMMAR-C072`](../../spec/iris-v1/02-lexical-grammar.md): `yield` and generator callables.
