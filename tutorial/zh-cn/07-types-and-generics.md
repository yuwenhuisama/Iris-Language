# 渐进类型

本章介绍 Iris 中的渐进类型系统。它是一套具有执行力的承诺机制，而非割裂执行模型的静态检查器。你可以编写无注解的动态代码，也可以编写清晰的类型注解，让 Iris 在编译期尽可能验证，并在跨越运行时边界时严格执行守护。本章涵盖 `Object`、可 nil 类型、类型测试与转换、`typeof`、作为一等公民的类型值、具体化且不变的泛型以及底部类型 `Never`。

<!-- iris-example: {"id":"07-gradual-types","mode":"vm","stdout":"30\n"} -->
```iris
module Adder {
  public module fun add(a: Integer, b: Integer) -> Integer {
    a + b
  }
}

let total: Integer = Adder.add(10, 20)
print(total)
```

预期终端输出：

```text
30
```

`Adder.add` 对参数和返回值设定了明确的书面契约。根据 `IRIS-V1-TYPES-C003`，未标注类型的方法参数与返回值默认具有 `Dynamic<Object>` 契约，在边界上接受任意值，并在方法内部执行常规的动态消息分派。

局部变量绑定遵循 `IRIS-V1-CONTROL-C005` 的不同规则：当局部绑定省略类型注解时，初始化表达式的精确静态类型直接成为该绑定的固定局部静态类型。例如未加注解并以 `"hello"` 初始化的绑定，其类型是 `String` 而非 `Dynamic<Object>`。若程序需要更宽或动态的绑定单元，必须显式书写类型契约，如 `mut x: Dynamic<Object> = source` 或 `mut x: String | Integer = "ready"`。

## 可选注解仍然有意义

Iris 的类型注解是可选的，但一旦写出，便会形成不可逾越的边界守护。变量绑定注解、参数类型、返回值类型、属性声明和泛型实参均遵循统一的规则：在能够静态证明类型不兼容时立即报错，无法静态证明时则在运行时边界进行严格检查。

<!-- iris-example: {"id":"07-union-bindings","mode":"vm","stdout":"ready\n42\n"} -->
```iris
mut value: String | Integer = "ready"
print(value)

value = 42
print(value)
```

预期终端输出：

```text
ready
42
```

联合类型声明确保 `value` 只能持有 `String` 或 `Integer` 类型的值。赋入其他类型会直接破坏边界。变量的声明类型在赋值后不会发生动态拓宽。

## Object、Nil 与可 nil 类型

`Object` 是 Iris 的顶层类型。运行时中的所有值都是 `Object` 的实例，包括 `nil`。但这并不意味着所有类型都可以容纳 `nil`。像 `String` 这样的具体类型严格排除了 `nil`，除非显式标注为 `String?` 或 `String | Nil`。

<!-- iris-example: {"id":"07-nilable-types","mode":"vm","stdout":"value is present\n"} -->
```iris
let text: String? = "value is present"

if text != nil {
  let unwrapped: String = text as String
  print(unwrapped)
}
```

预期终端输出：

```text
value is present
```

语法 `T?` 是 `T | Nil` 的精确语法糖，两者在内部规范化为完全相同的类型身份。通过非空检查可以安全地收窄联合类型。

## 测试与 casts

Iris 提供了三组核心类型运算符：`is`、`as` 以及 `as?`。运算符 `is` 执行运行时查询并返回 `Bool`。运算符 `as` 验证目标类型，成功时返回原值，失败时抛出运行时异常。运算符 `as?` 进行安全类型检查，失败时返回 `nil`。

<!-- iris-example: {"id":"07-tests-and-casts","mode":"vm","stdout":"hello\n"} -->
```iris
let value: Object = "hello"

if value is String {
  let text = value as String
  print(text)
}
```

预期终端输出：

```text
hello
```

这些操作直接检查已有对象的运行时身份，不会执行隐式数值类型转换、深拷贝集合数据或变更方法分派表。

当显式类型转换失败时，系统会立即抛出异常：

<!-- iris-example: {"id":"07-cast-failure","mode":"expected-error","engine":"vm","exit":1,"stderr":"Type","stdout":""} -->
```iris
let raw: Object = "text"
let num = raw as Integer
```

## Typeof 复制静态类型

运算符 `typeof(expression)` 是静态类型构造器，而不是运行时求值查询（`IRIS-V1-TYPES-C093`）。它提取操作数表达式在当前程序点的规范化静态类型，包括当前生效的流敏感收窄结果。操作数表达式仅参与类型检查，绝不会在运行时被执行。

<!-- iris-example: {"id":"07-typeof-expression","mode":"vm","stdout":"25\n"} -->
```iris
let base: Integer = 10
let copy: typeof(base) = 25
print(copy)
```

预期终端输出：

```text
25
```

如果无法从代码上下文推导出静态类型（例如来自省略的方法返回值注解），`typeof(expression)` 则回退为 `Dynamic<Object>`。

## Types 是值

在 Iris 中，类型对象是具有唯一身份的一等公民运行时值。表达式 `(T).type` 可以将静态类型表达式实例化为具体的可执行对象。

<!-- iris-example: {"id":"07-types-as-values","mode":"vm","stdout":"nominal\ntrue\n"} -->
```iris
let int_type = Integer.type
print(int_type.kind())
print(int_type.subtype?(Object.type))
```

预期终端输出：

```text
nominal
true
```

封闭的泛型类以及联合类型同样支持这种形式。书写类似 `(String | Nil).type` 时必须加上圆括号，以此区分联合类型语法与按位或操作符。

## 泛型是 reified 且不变的

泛型类、模块、契约及可调用体在运行时完整保留其实参元数据。泛型类可以直接实例化并基于具体化类型实参执行：

<!-- iris-example: {"id":"07-generic-box","mode":"vm","stdout":"42\n"} -->
```iris
class Box<T> {
  property value: T = 0
  public fun set_val(v: T) -> Nil {
    @value = v
    nil
  }
  public fun get_val() -> T {
    @value
  }
}

let b = Box<Integer>.new()
b.set_val(42)
print(b.get_val())
```

预期终端输出：

```text
42
```

泛型声明还可以通过 `where` 子句施加类型约束。

**仅规范（不执行）：** 此示意展示复合 `where` 约束和不合法的不变泛型赋值，并非完整可运行程序：它省略了对本地 `Comparable<T>` 契约的实现以及 `value` 的初始化。泛型类严格不变（`IRIS-V1-TYPES-C068`），因此即使补齐这些前提，`SortedBox<String>` 仍不可赋值给 `SortedBox<Object>`。

<!-- iris-example: {"id":"07-generic-invariance","mode":"spec-only","reason":"Specification example demonstrating invariant generic class parameterization and rejection of covariance"} -->
```iris
contract Comparable<T> {
  fun compare(other: T) -> Integer
}

class SortedBox<T> where T: Comparable<T> & NonNil {
  property value: T
}

let names: SortedBox<String> = SortedBox<String>.new()
let objects: SortedBox<Object> = names
```

Iris 严格执行泛型不变性规则。尽管 `String` 是 `Object` 的子类型，但 `SortedBox<String>` 绝不是 `SortedBox<Object>` 的子类型。类型转换无法跨越不变泛型边界。

可调用体类型同样遵循完全一致的不变性约束。`Closure<(Integer) -> Object>` 绝不能直接赋值给 `Closure<(Integer) -> Symbol>`，从而保证调用签名始终稳定可信。

当局部类型推导存在歧义时，调用方可以在调用点使用类型实参列表显式指定类型：

**仅规范（不执行）：** 此语法示意省略了 `choose` 和 `value` 的定义，展示显式调用类型实参以及调用点允许使用的通配占位符 `_`（`IRIS-V1-GRAMMAR-C067`）：

<!-- iris-example: {"id":"07-explicit-type-arguments","mode":"spec-only","reason":"Specification example illustrating explicit type argument bracket syntax and wildcards"} -->
```iris
let chosen = choose<String, Integer>(value)
let partial = choose<String, _>(value)
```

## Never 标记没有值的路径

`Never` 是 Iris 的底部类型。它表示永远无法产生正常运行时值的执行路径。抛出异常的表达式、死循环以及声明为 `-> Never` 的方法均归属于 `Never` 类型。

**仅规范（不执行）：** 此控制流示意省略了应用定义的条件 `ready?`，展示返回 `Never` 的函数（`IRIS-V1-TYPES-C080`）如何参与控制流，而不会拓宽 `if` 表达式的静态返回类型：

<!-- iris-example: {"id":"07-never-paths","mode":"spec-only","reason":"Specification example demonstrating bottom type Never in branching and early exit handling"} -->
```iris
fun fail(message: String) -> Never {
  raise message
}

let value: String = if ready? {
  "ready"
} else {
  fail("not ready")
}
```

由于 `Never` 代表不可达的求值结果，它可以安全赋值给任意其他目标类型，而不会拓宽外层分支的推导结果。

## 静态承诺，动态自由

Iris 渐进类型体系统一了开发安全与表达自由。

**动态自由**
- 未标注类型的方法参数默认为 `Dynamic<Object>`；未标注类型的局部绑定保留其初始化表达式的推导类型。
- 运行时类型测试 `is` 与条件转换 `as?` 允许代码灵活响应不同输入。
- 泛型实参在执行期按需具体化实例化。

**静态承诺**
- 显式类型注解确立硬性边界，杜绝隐式类型漂移。
- 泛型系统在所有参数化形式下保持严格不变性。
- 可 nil 规则有效防止意外的空值扩散。
- 类型系统从不改变方法选择器，也从不触发静态重载选择。

**动手练习**

声明一个函数 `safe_length(item: Object) -> Integer`，检查传入的 `item` 是否为 `String`。若是，将其转换为 `String` 并返回长度；若不是，返回 `0`。使用字符串和整数作为实参进行测试，并通过 `./target/debug/iris --vm test_type.iris` 执行验证。

预期终端输出：

```text
5
0
```

## 阅读规范

如需查阅形式化规范与确切类型法则，请参阅 [05-types-contracts-generics.md](../../spec/iris-v1/05-types-contracts-generics.md)：

| 条款 | 主题 |
| --- | --- |
| `IRIS-V1-TYPES-C003` 至 `IRIS-V1-TYPES-C007` | 渐进注解边界与 `Dynamic<Object>` 默认规则。 |
| `IRIS-V1-TYPES-C008` 至 `IRIS-V1-TYPES-C017` | 类型构造器、`Object`、`Nil`、`T?`、`Dynamic<T>`、别名与类型对象。 |
| `IRIS-V1-TYPES-C018` 至 `IRIS-V1-TYPES-C027` | 联合类型、交叉类型、可空性、`NonNil` 与同名义务规则。 |
| `IRIS-V1-TYPES-C028` 至 `IRIS-V1-TYPES-C035` | `is`、`as`、`as?`、契约视图与类型收窄。 |
| `IRIS-V1-TYPES-C054` 至 `IRIS-V1-TYPES-C074` | 泛型声明、约束条件、类型推导、具体化与不变性。 |
| `IRIS-V1-TYPES-C080` 至 `IRIS-V1-TYPES-C083` | `Never` 与不可达代码流分析。 |
| `IRIS-V1-TYPES-C093` | 作为静态类型复制的 `typeof(expression)`。 |
| `IRIS-V1-TYPES-C094` 至 `IRIS-V1-TYPES-C096` | `Closure<S>`、`BoundMethod<S>`、`Block<S>` 与可调用体不变性。 |
| `IRIS-V1-TYPES-C097` | 封闭泛型具体化失败时的 `TypeContractError`。 |
| `IRIS-V1-GRAMMAR-C063`, `C065`, `C066`, `C067` | 封闭泛型名称、`(T).type` 与显式调用类型实参语法。 |
