# Control Flow

This chapter demonstrates how Iris handles branching, looping, and iteration. The language defines value-producing control forms, but the current parser accepts `match` as a standalone form, not in a variable initializer. Conditions evaluate through the `to_bool` protocol, loops can yield return values through `break`, `for` loops bind fresh iteration variables, and `match` evaluates arms in top-to-bottom source order.

## If expressions produce values

In Iris, `if` is an expression that yields a value rather than a bare statement. It can appear in variable initializers, return values, or method arguments. Each branch establishes its own lexical block.

<!-- iris-example: {"id":"03-if","mode":"vm","stdout":"1\n"} -->
```iris
let status = if true {
  :ready
} else {
  :waiting
}
print(if status == :ready { 1 } else { 0 })
```

Expected output:

```text
1
```

If the condition evaluates to `false` and there is no `else` block, the expression produces `nil`.

## While loops can return through break

A `while` loop tests its condition before executing each iteration. Natural completion of a loop without an explicit `break` produces `nil`. Using `break expr` immediately terminates the loop and provides the overall result value of the `while` expression.

<!-- iris-example: {"id":"03-while-break","mode":"vm","stdout":"3\n"} -->
```iris
mut index: Integer = 0
let found = while index < 10 {
  if index == 3 { break index }
  index = index + 1
}
print(found)
```

Expected output:

```text
3
```

The `continue` keyword advances execution to the next iteration without returning a value. Labeled loops allow `break label: value` to exit nested loops cleanly.

## For loops bind fresh iteration cells

The `for` loop iterates over any object satisfying the iteration protocol. For each step of iteration, a fresh immutable binding is introduced for the loop variable.

<!-- iris-example: {"id":"03-for-loop","mode":"vm","stdout":"6\n"} -->
```iris
mut sum = 0
for value in [1, 2, 3] {
  sum = sum + value
}
print(sum)
```

Expected output:

```text
6
```

Because each iteration receives a fresh binding, closures created inside the loop capture independent variable instances.

## Generators yield a lazy iterator

Under `IRIS-V1-GRAMMAR-C072`, a callable containing `yield` statements is a generator: calling it does not execute the body immediately, but returns an `Iterator<T>`. Calling `.next()` advances to each `yield`, returning `Iteration.yield(value)`, and returns `Iteration.done` upon completion. The tree-walking reference engine executes generator callables directly.

**Reference-only example.** Save the following program as `generator.iris` and run it without `--vm`:

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

Expected output:

```text
10
20
```

The register machine VM supports collection iteration, but does not support this generator suspension example.

## Match arms run in source order

The standalone `match` below is supported by the current parser and VM. It evaluates its subject once and tests pattern arms sequentially in source order without fallthrough (`IRIS-V1-CONTROL-C050`). An `else =>` arm acts as the default fallback when no preceding pattern matches. Initializer syntax such as `let label = match ...` is not yet accepted by the parser; this example assigns to an existing mutable binding instead.

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

Expected output:

```text
two
```

Each arm can execute an expression or block, and pattern guards (`if condition`) allow additional filtering before selecting an arm.

## Static promise, dynamic freedom

Branch conditions and loop guards evaluate dynamically using `to_bool`. However, branch convergence types, pattern binding scopes, and jump destinations remain statically verified.

## Treat control forms as expressions

Where supported, value-producing control forms eliminate unneeded mutable temporary variables: assign the result of `if` directly to an immutable `let`, as in the first example. For `match`, retain the standalone form shown above because the current parser does not accept a `match` initializer.

**Hands-on Exercise**

Write a script `control_test.iris` that computes the factorial of `5` using a `while` loop: declare `mut n = 5`, `mut acc = 1`, loop while `n > 1`, multiplying `acc = acc * n` and decrementing `n = n - 1`, then print `acc`. Run with `./target/debug/iris --vm control_test.iris` to confirm output `120`.

## Read the spec

This chapter simplifies the following normative clauses:

- [`IRIS-V1-CONTROL-C039`](../../spec/iris-v1/04-bindings-callables-control-flow.md): condition truthiness evaluation.
- [`IRIS-V1-CONTROL-C040`](../../spec/iris-v1/04-bindings-callables-control-flow.md): logical operator short-circuiting.
- [`IRIS-V1-CONTROL-C041`](../../spec/iris-v1/04-bindings-callables-control-flow.md): `if` as a value-producing form.
- [`IRIS-V1-CONTROL-C043`](../../spec/iris-v1/04-bindings-callables-control-flow.md): `while`, `break`, and `continue`.
- [`IRIS-V1-CONTROL-C044`](../../spec/iris-v1/04-bindings-callables-control-flow.md): `for` traversal protocol.
- [`IRIS-V1-CONTROL-C045`](../../spec/iris-v1/04-bindings-callables-control-flow.md): `for` iteration binding scoping.
- [`IRIS-V1-CONTROL-C048`](../../spec/iris-v1/04-bindings-callables-control-flow.md): loop labels and targeted breaks.
- [`IRIS-V1-CONTROL-C050`](../../spec/iris-v1/04-bindings-callables-control-flow.md): `match` arm evaluation semantics.
- [`IRIS-V1-CONTROL-C051`](../../spec/iris-v1/04-bindings-callables-control-flow.md): pattern matching syntax vocabulary.
- [`IRIS-V1-CONTROL-C052`](../../spec/iris-v1/04-bindings-callables-control-flow.md): pattern guards.
- [`IRIS-V1-GRAMMAR-C060`](../../spec/iris-v1/02-lexical-grammar.md): `if` expressions in the grammar.
- [`IRIS-V1-GRAMMAR-C072`](../../spec/iris-v1/02-lexical-grammar.md): `yield` and generator semantics.
