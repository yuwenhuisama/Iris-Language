# What Iris Is

This chapter introduces the core execution model of Iris v1. Every runtime value is an object, behavior is message sending, and dynamic behavior is bounded by static promises. The language specification is frozen, including owner-approved errata through v1.33.

The reference implementation under [`crates/`](../../crates/) provides a command-line binary `iris` that includes both a register machine VM and a reference engine.

**Prerequisites and Obtaining the Repository**

Building Iris requires a stable Rust toolchain (see [`rust-toolchain.toml`](../../rust-toolchain.toml)). Clone the official repository and change into the project root directory:

```bash
git clone https://github.com/yuwenhuisama/Iris-Language.git
cd Iris-Language
```

**Building the toolchain**

Build the command-line executable from source using Cargo:

```bash
cargo build -p iris-cli
```

On Unix systems this produces `./target/debug/iris`. On Windows environments it creates `.\target\debug\iris.exe`.

**Running your first script**

Create a file named `hello.iris`:

<!-- iris-example: {"id":"01-hello","mode":"vm","stdout":"Hello, Iris!\n"} -->
```iris
print("Hello, Iris!")
```

Run this script explicitly using the register machine:

```bash
./target/debug/iris --vm hello.iris
```

On Windows:

```cmd
.\target\debug\iris.exe --vm hello.iris
```

The exact terminal output is:

```text
Hello, Iris!
```

To run using the default tree-walking reference engine instead, omit the `--vm` flag:

```bash
./target/debug/iris hello.iris
```

**Running short expressions and the REPL**

You can evaluate quick expressions directly from the command line using the `-e` flag with either engine:

```bash
./target/debug/iris --vm -e 'print(1 + 2)'  # run on bytecode VM
./target/debug/iris -e 'print(1 + 2)'       # run on reference engine
```

When invoked without arguments, `./target/debug/iris` starts an interactive REPL session on the reference engine. The interactive REPL automatically prints the value of each entered expression after evaluation. In contrast, script execution in both the VM and the reference runner requires explicit `print(...)` calls to emit output. Type `:quit` to leave the REPL.

## Every value is an object

Iris is object-oriented at the root. Every runtime value is conceptually an object, and `Object` is the top type. As specified in `IRIS-V1-RUNTIME-C008`, this objecthood model permits runtime implementations to use unboxed representations or immediate values for performance, provided that observable identity behavior and type classification remain strictly compliant.

When you write `1 + 2`, the integer `1` receives the message `+` with argument `2`. Expressions do not call bare external functions; they dispatch through the receiver's type.

## Operators are message sends

Arithmetic and comparison operators correspond to message sends on the receiver object. The expression `total << 1` invokes the `<<` method on `total`. Similarly, `==` dispatches an equality check.

<!-- iris-example: {"id":"01-operators","mode":"vm","stdout":"6\nfalse\n"} -->
```iris
let total = 1 + 2
let shifted = total << 1
print(shifted)
print(total == shifted)
```

Expected output:

```text
6
false
```

Iris reserves a small set of short-circuiting control forms that are not message sends: `!`, `&&`, `||`, `&&=`, and `||=`.

## Identity is explicit

Equality (`==`) and identity (`same?`) are distinct operations. `==` checks whether two objects have equivalent value according to their equality protocol.

The primitive operator `same?` tests observable object identity rather than raw memory addresses (`IRIS-V1-RUNTIME-C029`). It accepts only identity-bearing operands (such as mutable collections, classes, and instances). If either operand is identity-less (such as an Integer or Float value), `same?` immediately raises an `IdentityError`.

<!-- iris-example: {"id":"01-identity","mode":"vm","stdout":"true\nfalse\n"} -->
```iris
let first = [1, 2]
let second = first
let third = [1, 2]
print(first same? second)
print(first same? third)
```

Expected output:

```text
true
false
```

`first` and `second` reference the same identity-bearing array object, so `first same? second` evaluates to `true`. While `third` contains identical elements, it is a distinct array instance, so `first same? third` evaluates to `false`.

## Static promises bound dynamic behavior

Iris combines dynamic dispatch with static boundaries. A method signature establishes a static promise for caller contracts, but dispatch remains dynamic at runtime. Static types do not trigger compile-time method overloading.

<!-- iris-example: {"id":"01-static-promise","mode":"vm","stdout":"30\n"} -->
```iris
module Calculator {
  public module fun add(a: Integer, b: Integer) -> Integer {
    a + b
  }
}
print(Calculator.add(10, 20))
```

Expected output:

```text
30
```

The type annotations `a: Integer` and `b: Integer` validate inputs at boundary crossings. However, the evaluation of `a + b` still sends the message `+` to the object referenced by `a`.

## Static promise, dynamic freedom

Static guarantees prevent programs from silently drifting into invalid states. Dynamic dispatch allows runtime flexibility without complex static overload resolution rules. A type annotation never chooses a hidden alternate method implementation.

## Where the next chapters go

The next chapters expand on this execution model:

- [Values and Bindings](02-values-and-bindings.md) covers immutable `let`, mutable `mut`, literal forms, and truthiness.
- [Control Flow](03-control-flow.md) explores `if`, `while`, `for`, and `match` expressions.
- [Functions, Closures, Blocks](04-callables-and-closures.md) examines method definitions, anonymous closures, and trailing blocks.
- [Classes and Objects](05-classes-and-objects.md) explains object instantiation, instance state, and inheritance.

## Try it

**Hands-on Exercise**

Write a script named `math_test.iris` that binds an integer `x = 40`, adds `2`, shifts the result left by 1 bit (`<< 1`), and prints the output. Run it with `./target/debug/iris --vm math_test.iris`.

Expected terminal output:

```text
84
```

## Read the spec

This chapter simplifies and aligns with the following normative clauses:

- [`IRIS-V1-TRACE-C011`](../../spec/iris-v1/README.md): the specification set does not implement Iris.
- [`IRIS-V1-IDENTITY-C006`](../../spec/iris-v1/01-language-identity.md): every runtime value is an object.
- [`IRIS-V1-IDENTITY-C007`](../../spec/iris-v1/01-language-identity.md): behavior is message sending.
- [`IRIS-V1-IDENTITY-C008`](../../spec/iris-v1/01-language-identity.md): static promises bound dynamic behavior.
- [`IRIS-V1-IDENTITY-C009`](../../spec/iris-v1/01-language-identity.md): static type choice does not change ordinary selector identity.
- [`IRIS-V1-IDENTITY-C013`](../../spec/iris-v1/01-language-identity.md): compact dynamic and static model.
- [`IRIS-V1-RUNTIME-C003`](../../spec/iris-v1/03-runtime-object-model.md): runtime objecthood.
- [`IRIS-V1-RUNTIME-C027`](../../spec/iris-v1/03-runtime-object-model.md): overloadable symbolic operators are ordinary Method sends.
- [`IRIS-V1-RUNTIME-C029`](../../spec/iris-v1/03-runtime-object-model.md): `same?` is primitive identity comparison.
