# 渐进类型

本章把 Iris 的类型层解释为一组承诺，而不是另一套执行模型。你可以不写注解，得到动态边界；也可以写注解，让 Iris 在能静态强制时静态强制，在必须运行时检查时运行时检查。你还会看到 `Object`、union、可 nil 的 `?`、类型测试和 casts、泛型、不变性，以及 `Never`。

```iris
fun add(a: Integer, b: Integer) -> Integer {
  a + b
}

fun dynamic_add(a, b) {
  a + b
}

let total: Integer = add(1, 2)
let loose = dynamic_add("a", 3)
```

这个示例复用自 `IRIS-V1-TYPES-EX001`。`add` 在参数和结果上写出了 Contracts。`dynamic_add` 省略了它们，所以这些位置的静态和运行时 Contract 都是 `Dynamic<Object>`。

## 可选注解仍然有意义

Iris 是渐进的。你不必给每个局部或 Method 写注解，但一旦写出注解，它同时就是静态 Contract 和运行时边界 guard。绑定注解、参数注解、返回类型、属性类型、泛型实参和 Contract requirement 都有同一种基本形态：如果违反可被证明，就在执行前诊断；如果无法证明，边界会在运行时检查。

```iris
let name: String = "Iris"
mut count: Integer
count = 1

mut value: String | Integer = "ready"
value = 42
```

这段代码复用自绑定示例。union 类型表示 `value` 可以持有 `String` 或 `Integer`。它不表示该绑定会在赋值后拓宽。声明类型对该绑定是固定的。

## Object、Nil 与可 nil 类型

`Object` 是顶层类型。每个 Iris 值都是 `Object`，包括 `nil`。但这不表示每个类型都可 nil。像 `String` 这样的具体类型默认排除 `nil`，除非你写 `String?` 或 `String | Nil`。

```iris
let name: String? = load_name()

if name != nil {
  let strong: String = name
}

let value: String | MutableString = load_text()
value.length()
```

这复用自 `IRIS-V1-TYPES-EX003`。`T?` 是 `T | Nil` 的精确语法糖，Iris 会把二者规范化成同一个 Type 身份。nil 检查可以收窄 true 路径。只有当每个 branch 都有安全的公共 member 和调用形状时，才能访问 union member。

## 测试与 casts

Iris 有三个核心类型操作。`is` 提问并返回 `Bool`。`as` 检查或证明类型，失败时 raise。`as?` 安全检查，失败时返回 `nil`。

```iris
let object: Object = load_value()

if object is String {
  object.length()
}

let maybe_user: User? = object as? User
let printable = object as Printable
printable..print()
```

这个示例复用自 `IRIS-V1-TYPES-EX004`。这些操作保留同一个值。它们不转换数字，不克隆集合，不改变泛型实参，也不选择另一个普通 Method。Contract casts 会创建已检查的 Contract view，而限定 Contract 调用仍然要求 `..`。

## 泛型是 reified 且不变的

泛型 Classes、Contracts、Modules 和 Methods 都携带运行时保留的实参和元数据。泛型声明可以用 `where` 子句约束类型参数。

```iris
contract Comparable<T> {
  fun compare(other: T) -> Integer
}

class SortedBox<T> where T: Comparable<T> & NonNil {
  property value: T
}

let names: SortedBox<String> = SortedBox<String>.new()
let objects: SortedBox<Object> = names  // rejected because generic Classes are invariant
```

这个示例复用自 `IRIS-V1-TYPES-EX008`。不变性是关键规则：`SortedBox<String>` 不会仅仅因为 `String` 是 `Object` 就成为 `SortedBox<Object>` 的 subtype。Casts 也不会跨越不变泛型实参。如果想要转换后的容器，就要自己写转换代码。

裸泛型 Class 名称是元数据，不是实例类型。对于 `class Box<T>`，`Box` 命名的是泛型定义对象。实例注解和普通构造需要闭合类型，比如 `Box<String>`，或 construction-site placeholder，比如 `Box<_>.new(value)`。

```iris
fun make<T>() -> T where T: Object {
  load_value() as T
}

let user: User = make()
let box = Box<_>.new(user)
```

这复用自 `IRIS-V1-TYPES-EX009`。推断是局部且有界的。Iris 可以使用 actual arguments 和 immediate expected result type。它不会检查 Method bodies、后续 uses，或 whole-program state。

## Never 标记没有值的路径

`Never` 是底层类型。它没有正常运行时值，并且可赋给每个类型。Raising、裸 re-raise、静态不可达路径，以及声明为 `-> Never` 的调用都会产生 `Never`。

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

这个示例复用自 `IRIS-V1-TYPES-EX011`。`else` branch 不会拓宽 `if` 结果，因为它不会产生正常值。

## 静态承诺，动态自由

动态自由：未注解位置会在自己的 `Dynamic<Object>` 边界内接受动态 sends，`is` 和 `as?` 可以基于运行时值收窄，泛型物化会在需要闭合实参时发生。

静态承诺：已写出的注解、Contract views、泛型约束、不变实参、nilability 和 `Never` flow 都仍是可强制的边界。它们都不会创建 overload 派发或隐式转换。

## 阅读规范

精确规则请阅读 [05-types-contracts-generics.md](../../spec/iris-v1/05-types-contracts-generics.md)：

| 条款 | 主题 |
| --- | --- |
| `IRIS-V1-TYPES-C003` through `IRIS-V1-TYPES-C007` | 渐进注解边界和 `Dynamic<Object>` 默认值。 |
| `IRIS-V1-TYPES-C008` through `IRIS-V1-TYPES-C017` | Type 构造子、`Object`、`Nil`、`T?`、`Dynamic<T>`、aliases 和 Type objects。 |
| `IRIS-V1-TYPES-C018` through `IRIS-V1-TYPES-C027` | Union、intersection、nilability、`NonNil` 和同名义务规则。 |
| `IRIS-V1-TYPES-C028` through `IRIS-V1-TYPES-C035` | `is`、`as`、`as?`、Contract views 和 narrowing。 |
| `IRIS-V1-TYPES-C054` through `IRIS-V1-TYPES-C074` | 泛型声明、约束、推断、物化和不变性。 |
| `IRIS-V1-TYPES-C080` through `IRIS-V1-TYPES-C083` | `Never` 和不可达 flow。 |
