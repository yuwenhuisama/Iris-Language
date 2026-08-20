# 模块与 Contract

本章说明 Iris 如何把行为复用和承诺分开。`module` 提供 Methods，供 Class 或另一个 Module 混入。`contract` 声明一个义务表面，Class 用 `for` 显式承诺它，再用 `impl` 满足它。两者会配合使用，但不能互换：Modules 影响查找顺序，Contracts 定义会被检查的 slots 和视图。

```iris
module A { fun trace() -> Symbol { :A } }
module B mixin A { override fun trace() -> Symbol { :B } }
class C extends Object mixin A, B {}

C.new().trace()           // selects B before A
```

这段代码来自 `IRIS-V1-RUNTIME-C047` 和 `IRIS-V1-RUNTIME-C053`。一个 Class 有一条 superclass 链，然后是组合进来的 Modules。头部 `mixin A, B` 按从左到右应用，但查找会先检查最近的 Module，所以有效顺序是 `C`，然后 `B`，然后 `A`，最后是 superclass MRO。

## Modules 添加行为

Module 是可以持有 Methods 的命名对象。Class 可以在声明中组合 Modules。另一个 Module 也可以组合 Modules，这让公共行为可以收集成可复用的层。

```iris
module Trace {
  fun trace() -> Symbol { :trace }
}

class Job extends Object mixin Trace {}
```

这不是多重继承。Class 仍然只有一个 superclass。Modules 会插入 Method 查找顺序，并按 Module 身份去重。再次包含同一个 Module 不会创建第二份副本，也不会移动旧副本。

Module Methods 使用当前接收者运行。如果某个 Module Method 读取 `@state`，它读取的是接收者上名为 `@state` 的 slot，不是 Module 自己拥有的存储。Private Method 访问则不同：除非组合边显式请求，Module 组合不会授予 private 访问权，写法是在 mixin 列表中把该条目标记为 `private`。

```iris
class Job extends Object mixin Trace private, Audit {}
```

该标记位于单个组合边上，所以这里 `Trace` 获得 private 授权，而 `Audit` 没有。当前接收者上的原始 `@x` 访问是另一条规则，不依赖这个标记。

## Contracts 声明承诺

Contract 是 Iris 的显式义务表面。Contract 体包含 requirements，不包含可执行 Method body 或存储。requirement 是有签名但没有主体的 Method 声明；在 Contract 内写主体会以 `CONTRACT_METHOD_BODY_FORBIDDEN` 被拒绝。Class 用 `for` 选择一个 Contract，再用 `impl` 标记满足它的成员。

```iris
contract Printable<T> where T: Object {
  fun print(value: T) -> String
}

class User for Printable<User> {
  impl fun print(value: User) -> String {
    value.name
  }
}

let view = User.new() as Printable<User>
view..print(User.new())
```

这个示例复用自 `IRIS-V1-TYPES-EX006`。`for Printable<User>` 声明是 `User` 的静态脊柱事实。之后的动态变更可以替换兼容的主体，但不能抹掉 `User` 已承诺 `Printable<User>` 这一事实。

`impl` 标记很重要。它表示这个 Method 意在满足某个 Contract requirement。如果该 Method 同时替换继承或混入来的 Method，Iris 会写出两个标记，例如 `override impl fun draw() -> Nil { ... }`。

## 限定 Contract slots

普通派发不会按静态类型 overload。如果两个 Contracts 要求同一个 selector，但签名不兼容，Iris 不会猜测。你要写显式限定实现，并通过带 `..` 的 Contract view 调用它们。

```iris
contract Parser {
  fun process(input: String) -> Object
}

contract Validator {
  fun process(input: Object) -> Bool
}

class Tool for Parser, Validator {
  impl Parser::process(input: String) -> Object { input }
  impl Validator::process(input: Object) -> Bool { true }
}

let parser = Tool.new() as Parser
parser..process("source")
```

这复用自 `IRIS-V1-TYPES-EX007`。声明形式用 `Parser::process` 命名一个 Contract slot。表达式形式用 `..process` 调用限定 slot。单个点保留普通派发。

```iris
let view = parser as ParserContract
parser.process(input)     // ordinary selector process
view.process(input)       // still ordinary selector process on the receiver
view..process(input)      // qualified Contract slot ParserContract::process
```

这段代码复用自运行时示例。要记住的区别是：`as ParserContract` 检查或构造 Contract view，但 `view.process(input)` 仍然是在接收者上做普通 selector 查找。只有 `view..process(input)` 会选择 Contract 限定的命名空间。

## 静态承诺，动态自由

动态自由：Class 可以混入 Modules，替换兼容的 Method bodies，并在运行时形成 Contract views。Module 顺序可以通过已验证的 open 事务改变。

静态承诺：Class 有一个稳定的 superclass 承诺，一个已声明的 Contract 集，一个不 overload 的普通 selector 命名空间，以及一个显式 Contract 限定命名空间。之后的动态变更必须先保留静态脊柱，才能发布。

这种分割正是 Contract 派发显式化的原因。Iris 允许运行时行为移动，但拒绝把按类型决定的 overload 选择藏在普通调用里。

已声明的 Contract 集是 Class 拥有的最强静态事实之一。运行时尝试移除其中一个 Contract，拼写为 `remove_contract(contract)` 或 `Reflection::Class.remove_contract(target, contract)`，只为让拒绝可以被观察到而存在：它会在发布前以 `TypeContractError` 被拒绝，Class 保留自己的 Contracts 和活动修订。当提议的祖先关系会丢弃目标依赖的、携带静态事实的祖先时，运行时 superclass 变更同样会被拒绝。

## Kernel 始终在作用域内

`Kernel` 是组合进 `Object` 的语言核心 Module。它携带必须无需 import 就处处可见的声明，例如第 04 章中的 `Block<S>` Type alias。它是普通 Module，拥有普通组合和查找规则，并且不是第二个根 Class；它不能提供或 shadow `Object` 的默认比较、真值性、缺失消息或零参数 `initialize`。

## 阅读规范

精确规则请阅读 [03-runtime-object-model.md](../../spec/iris-v1/03-runtime-object-model.md) 和 [05-types-contracts-generics.md](../../spec/iris-v1/05-types-contracts-generics.md)：

| 条款 | 主题 |
| --- | --- |
| `IRIS-V1-RUNTIME-C046` through `IRIS-V1-RUNTIME-C053` | Module 组合与 MRO 顺序。 |
| `IRIS-V1-RUNTIME-C030` through `IRIS-V1-RUNTIME-C033` | Contract view 派发与缺失的限定 slots。 |
| `IRIS-V1-RUNTIME-C163` | `Kernel` 作为始终可见的语言核心 Module。 |
| `IRIS-V1-TYPES-C041` through `IRIS-V1-TYPES-C053` | Contract 声明、`for`、`impl`、限定实现、views、相等性和哈希。 |
| `IRIS-V1-TYPES-C098` and `IRIS-V1-TYPES-C099` | 受保护的祖先关系与被拒绝的 `remove_contract`。 |
| `IRIS-V1-GRAMMAR-C061` | mixin 条目上的 `private` 标记。 |
| `IRIS-V1-GRAMMAR-C062` | 作为 Contract requirements 的无主体 Method 声明。 |
| `IRIS-V1-IDENTITY-C009` and `IRIS-V1-IDENTITY-C010` | 没有静态类型 overload 派发。 |
