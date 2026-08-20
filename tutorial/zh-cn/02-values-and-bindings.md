# 值与绑定

本章讲解可以直接写在 Iris 源码中的基础数据，以及保存它们的绑定形式：`let`、`mut`、`const` 和 `shared`。你会看到注解如何成为静态和运行时契约，为什么 `nil` 是真正的对象，以及真值性如何通过 `to_bool` 工作，而不是通过硬编码的条件规则。

```iris
let name: String = "Iris"
mut count: Integer
count = 1

mut value: String | Integer = "ready"
value = 42
```

这个片段复用自 `IRIS-V1-CONTROL-EX001`。

## 绑定决定局部名称的形状

绑定形式刻意保持很小。`let` 声明不可变局部，并且必须有初始化器。`mut` 声明可变局部。`mut` 绑定只有在写出类型时才能延迟初始化。`const` 使用和 `let`、`mut` 相同的语法产生式，但它命名的是不可变声明级绑定，而不是可变局部单元。

```iris
let language = "Iris"
let answer: Integer = 42
mut state: Symbol = :ready
const Version: Integer = 1
```

## 共享和全局存储必须声明，不会凭空出现

局部单元不是唯一的单元。Class 或 Module 主体可以在 `@@name` 上用 `shared let` 或 `shared mut` 声明锚定在层级上的存储，包可以在 `$name` 上用 `global let` 或 `global mut` 声明全局存储。两种形式都必须先声明后使用：如果没有声明创建过 `@@count` 或 `$count`，给它赋值会产生 `MISSING_DECLARED_STORAGE` 诊断，而不是隐式定义。

```iris
class Registry {
  shared mut @@count: Integer = 0
  shared let @@limit: Integer = 16

  class fun record() -> Integer { @@count += 1 }
}
```

`shared let` 创建不可变单元，`shared mut` 创建可赋值单元。该单元锚定到声明它的 Class 或 Module，所以子类不能遮蔽或重新声明它；重新声明会产生 `CLASS_VARIABLE_REDECLARATION` 诊断，祖先单元保持不变。

## 注解固定局部契约

如果局部绑定省略类型注解，初始化器会固定局部类型。之后对 `mut` 绑定的赋值仍必须符合这个固定类型。如果你想要更宽的单元，要在一开始写出更宽类型。

```iris
mut exact = "ready"
mut flexible: String | Integer = "ready"
flexible = 42
```

## 字面量形式创建对象

常见标量字面量都是创建对象的源码形式。Integer 字面量在语言层面是任意精度。没有后缀的浮点字面量是 `Float64`；后缀选择 `Float32` 或 `Float64`。String 是不可变文本值。`m"..."` 创建一个新的 `MutableString` 身份。Symbol 以 `:` 开头，表示不可变的驻留名称。

```iris
let whole = 1_000
let ratio = 0.5f64
let title = "report"
let buffer = m"draft"
let tag = :ready
```

## Nil 和集合都是普通值

`nil`、`true` 和 `false` 不是特殊的非对象。它们是具有指定运行时行为的单例对象。`nil` 的 Type 是 `Nil`。除非你用 `String?` 或 `String | Nil` 明确说明，否则 `String` 这样的类型不包含 `nil`。

```iris
let missing: String? = nil
let present: String | Nil = "name"
```

集合也有字面量形式，但其详细行为属于规范后续章节。现阶段，可以把它们理解为会产出对象的表达式：Array 使用 `[...]`，Hash 字面量使用 `%{ ... }`，Range 使用 `..=` 或 `..<`。

```iris
let items = [1, 2, 3]
let table = %{ :name: "Iris", :version: 1 }
let closed = 1 ..= 3
let half_open = 1 ..< 3
```

## 真值性通过 to_bool

条件使用动态的 `to_bool() -> Bool` 协议。根 `Object` 默认为真。`nil` 为假。`false` 为假，`true` 为真，因为 Bool 返回自身。逻辑运算符 `&&` 和 `||` 返回其中一个操作数，而不是转换后的 Bool，并且只在需要时才求值右侧。

```iris
let chosen = if config.ready? { "ready" } else { nil }
let cached = value || compute_default()
let both = label && label.length()
```

这个片段复用自 `IRIS-V1-CONTROL-EX007`。

这个真值性规则是动态的，但它本身不是类型谓词。真值测试不会自动把 `String?` 变成 `String`。当你需要流类型收窄时，使用显式 nil 比较、`is`、`as`、`as?`、类型化 `catch` 或 match 类型模式。

```iris
let name: String? = load_name()

if name != nil {
  let strong: String = name
}
```

这个片段改编自 `IRIS-V1-TYPES-EX003`。

## 静态承诺，动态自由

绑定注解既是静态承诺，也是运行时边界守卫。它不会冻结对象行为，不会选择重载，也不会复制值。它承诺任何存入该单元的内容在跨越边界时都满足写出的类型。

在需要重新赋值前，优先使用 `let`。当单元会随时间变化时，使用 `mut`。对于应该参与限定声明命名空间的不可变具名声明，使用 `const`。

## 阅读规范

本章简化了以下规范性条款：

- [`IRIS-V1-GRAMMAR-C024`](../../spec/iris-v1/02-lexical-grammar.md)：Integer 字面量形式。
- [`IRIS-V1-GRAMMAR-C029`](../../spec/iris-v1/02-lexical-grammar.md)：float 后缀。
- [`IRIS-V1-GRAMMAR-C037`](../../spec/iris-v1/02-lexical-grammar.md)：`MutableString` 字面量创建。
- [`IRIS-V1-GRAMMAR-C040`](../../spec/iris-v1/02-lexical-grammar.md)：Array、Hash、Tuple 和 Range 字面量语法。
- [`IRIS-V1-CONTROL-C003`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：`let` 和 `mut` 绑定声明。
- [`IRIS-V1-CONTROL-C004`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：确定赋值和延迟 `mut` 规则。
- [`IRIS-V1-CONTROL-C005`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：绑定注解作为固定局部契约。
- [`IRIS-V1-CONTROL-C009`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：对未声明 shared 或 global 存储的赋值会失败。
- [`IRIS-V1-GRAMMAR-C059`](../../spec/iris-v1/02-lexical-grammar.md)：`shared let` 和 `shared mut` 声明形式。
- [`IRIS-V1-RUNTIME-C162`](../../spec/iris-v1/03-runtime-object-model.md)：shared 单元创建、不可变性和重复拒绝。
- [`IRIS-V1-CONTROL-C039`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：通过 `to_bool` 的真值性。
- [`IRIS-V1-TYPES-C011`](../../spec/iris-v1/05-types-contracts-generics.md)：`Nil` 和可 nil 性。
- [`IRIS-V1-TYPES-C012`](../../spec/iris-v1/05-types-contracts-generics.md)：`T?` 即 `T | Nil`。
