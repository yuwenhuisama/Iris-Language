# 值与绑定

本章讲解可以在 Iris 源码中直接编写的基础数据字面量，以及承载它们的各种绑定形式：`let`、`mut`、`const` 与 `shared`。你将学习类型注解如何确立固定的局部契约、集合与 `nil` 如何作为普通对象运作，以及真值性如何通过 `to_bool` 协议进行求值。

## 绑定决定局部名称的形状

Iris 提供了明确的绑定关键字：

- `let` 声明不可变的局部变量，必须提供初始表达式。
- `mut` 声明可变的局部变量，后续可以重新赋值。
- `const` 在声明级别声明不可变常量。

<!-- iris-example: {"id":"02-bindings","mode":"vm","stdout":"Iris\n2\n"} -->
```iris
let language = "Iris"
mut count = 1
count = count + 1
print(language)
print(count)
```

预期输出：

```text
Iris
2
```

向不可变的 `let` 变量赋值会在编译阶段被拒绝。

## 共享和全局存储必须声明，不会凭空出现

局部变量存在于当前的执行栈帧中。当多个实例或方法需要共享状态时，必须显式声明存储空间后才能使用。

在 Class 或 Module 内部，`shared mut @@name` 或 `shared let @@name` 声明依附于继承层级中该类型的共享变量。类似地，`global let $name` 或 `global mut $name` 声明包级全局变量。访问或赋值未声明的 `@@name` 或 `$name` 会触发 `MISSING_DECLARED_STORAGE` 编译错误。

<!-- iris-example: {"id":"02-shared","mode":"vm","stdout":"1\n2\n"} -->
```iris
class Counter {
  shared mut @@count: Integer = 0
  public fun bump() -> Integer {
    @@count = @@count + 1
  }
}
let c = Counter.new()
print(c.bump())
print(c.bump())
```

预期输出：

```text
1
2
```

## 注解固定局部契约

在局部变量上书写类型注解时，该类型将作为该槽位所有后续赋值的固定契约。

<!-- iris-example: {"id":"02-annotations","mode":"vm","stdout":"15\n"} -->
```iris
mut count: Integer = 10
count = count + 5
print(count)
```

预期输出：

```text
15
```

若省略类型注解，变量的类型将由初始表达式推断得出，并在该绑定的生命周期内保持固定。

## 字面量形式创建对象

每种字面量形式都会创建具体的对象：

- **整数**：任意精度的整数值（例如 `1000`）。
- **浮点数**：默认为 `Float64`；后缀如 `f32` 与 `f64` 显式指定位宽（例如 `0.5f64`）。
- **字符串**：由双引号括起来的不可变文本值（例如 `"report"`）。
- **符号**：以冒号开头的驻留不可变标识符（例如 `:ready`）。

<!-- iris-example: {"id":"02-literals","mode":"vm","stdout":"1000\n0.5\nreport\nready\n"} -->
```iris
let count = 1000
let ratio = 0.5f64
let title = "report"
let tag = :ready
print(count)
print(ratio)
print(title)
print(tag)
```

预期输出：

```text
1000
0.5
report
ready
```

## Nil 和集合都是普通值

`nil`、`true` 与 `false` 是单例对象，而不是底层的特殊标记值。`nil` 是 `Nil` 类型的实例。

集合类型同样生成标准对象：

- 数组使用方括号语法 `[...]`。
- 字典映射使用散列表语法 `%{ ... }`。

<!-- iris-example: {"id":"02-nil-collections","mode":"vm","stdout":"2\nIris\n"} -->
```iris
let items = [1, 2]
let table = %{ :lang: "Iris" }
print(items.length())
print(table[:lang])
```

预期输出：

```text
2
Iris
```

## 真值性通过 to_bool

Iris 中的条件表达式通过 `to_bool() -> Bool` 协议计算真假：

- `Object` 默认在条件判断中为真。
- `nil` 为假。
- `false` 为假，而 `true` 为真。
- 逻辑运算符 `||` 与 `&&` 具备短路特性，并直接返回操作数本身，而不是强制转换成布尔值。

<!-- iris-example: {"id":"02-truthiness","mode":"vm","stdout":"active\nyes\n"} -->
```iris
let primary = nil
let fallback = "active"
let chosen = primary || fallback
print(chosen)
print(if "text" { "yes" } else { "no" })
```

预期输出：

```text
active
yes
```

因为 `primary` 为 `nil`（假），`||` 运算符返回右侧计算出的操作数 `"active"`。

## 静态承诺，动态自由

绑定注解充当系统的边界保证。它们不会更改对象的底层表示或派发机制，但能确保非法类型的值无法越过边界注入到类型化槽位中。

**实战练习**

创建脚本 `bindings_test.iris`，建立指向可变哈希表的不可变绑定 `let config = %{ :port: 8080, :host: "localhost" }`（注意 `let` 仅锁定变量名绑定，并不冻结容器内部内容），声明可变变量 `mut status: String = "starting"`，随后将 `status` 重新赋值为 `"running"`，并打印字典中的 `:port` 与 `status`。使用 `./target/debug/iris --vm bindings_test.iris` 运行以确认输出 `8080` 与 `running`。

## 阅读规范

本章内容简化并对应于以下规范性条款：

- [`IRIS-V1-GRAMMAR-C024`](../../spec/iris-v1/02-lexical-grammar.md)：整数字面量形式。
- [`IRIS-V1-GRAMMAR-C029`](../../spec/iris-v1/02-lexical-grammar.md)：浮点数字面量格式与后缀。
- [`IRIS-V1-GRAMMAR-C037`](../../spec/iris-v1/02-lexical-grammar.md)：`MutableString` 字面量创建。
- [`IRIS-V1-GRAMMAR-C040`](../../spec/iris-v1/02-lexical-grammar.md)：数组、哈希、元组与范围字面量语法。
- [`IRIS-V1-CONTROL-C003`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：`let` 与 `mut` 绑定声明。
- [`IRIS-V1-CONTROL-C004`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：明确赋值规则。
- [`IRIS-V1-CONTROL-C005`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：绑定注解作为固定的局部契约。
- [`IRIS-V1-CONTROL-C009`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：未声明的共享或全局存储访问失败。
- [`IRIS-V1-GRAMMAR-C059`](../../spec/iris-v1/02-lexical-grammar.md)：`shared let` 与 `shared mut` 声明语法。
- [`IRIS-V1-RUNTIME-C162`](../../spec/iris-v1/03-runtime-object-model.md)：共享单元创建与不可变性。
- [`IRIS-V1-CONTROL-C039`](../../spec/iris-v1/04-bindings-callables-control-flow.md)：通过 `to_bool` 求值真值性。
- [`IRIS-V1-TYPES-C011`](../../spec/iris-v1/05-types-contracts-generics.md)：`Nil` 与可空性。
- [`IRIS-V1-TYPES-C012`](../../spec/iris-v1/05-types-contracts-generics.md)：作为 `T | Nil` 简写的 `T?`。
