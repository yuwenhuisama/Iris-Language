# 类与对象

本章把对象模型放进代码。你会定义 Class，用 `fun` 附着 Method，使用 `@name` 原始实例变量，用 `self` 指代当前接收者，通过 `Type.new()` 构造实例，并区分身份和相等性。本章最后会指向 Module 和 Contract，它们是单一 Class 继承之后的下一组组合工具。

```iris
class Counter {
  fun initialize() -> Nil { @value = 0 }
  fun add(delta: Integer) -> Integer { @value += delta }
  fun value() -> Integer { @value }
}

let counter = Counter.new()
```

这个片段改编自 `IRIS-V1-CONTROL-EX003`。

## Class 是带有 new 消息的对象

Class 声明创建逻辑 Class 对象。实例通过向 Class 对象发送 `new` 创建。Iris v1 没有用 Class 命名的构造函数语法，也没有构造函数重载。构造会分配实例，在存在已存储属性初始化时运行这些初始化，然后调用最终动态的 `initialize(...)`，并在初始化成功时返回该实例。

```iris
class Point {
  fun initialize(x: Integer, y: Integer) -> Nil {
    @x = x
    @y = y
  }
}

let point = Point.new(1, 2)
```

## 原始实例变量位于当前接收者

原始实例变量使用 `@name`。它们命名当前接收者上的存储。源代码不能写 `other.@name`；外部代码应该使用 Method 或属性。在原始 ivar 规则下，如果原始 ivar 读取找不到槽位，会返回 `nil`。只有当接收者的策略允许实例状态扩展时，首次赋值才可以创建接收者状态。

```iris
class Named {
  fun initialize(name: String) -> Nil { @name = name }
  fun name() -> String { @name }
  fun rename(name: String) -> Nil { @name = name }
}
```

## Self 和 super 让接收者角色显式

`self` 命名当前接收者。当你想把接收者作为对象传递，或让接收者角色更明确时，它很有用。接收者代码内的非限定 Method 调用也可以解析为对当前接收者的特权发送。

```iris
class Box {
  fun initialize(value: Object) -> Nil { @value = value }
  fun value() -> Object { @value }
  fun copy_value_to(other: Box) -> Object { other.replace(value()) }
  fun replace(value: Object) -> Object { @value = value }
  fun receiver() -> Box { self }
}
```

继承使用 `extends`。一个 Class 有单个运行时超类。`super(args...)` 在接收者当前 Method 查找顺序中，从当前 Method 的词法所有者之后继续调用同一个选择器。没有裸 `super`，也没有隐式参数转发。

```iris
class NamedCounter extends Counter {
  fun initialize(name: String) -> Nil {
    super()
    @name = name
  }
}
```

## 相等性不同于身份

相等性和身份不同。`==` 是普通比较 Method 槽位。对带身份对象，根相等性先检查两个引用是否表示同一个对象，然后再遵循比较协议。`same?` 是原始身份测试。它只接受带身份操作数，并会对无身份值引发 `IdentityError`。

```iris
let first = Object.new()
let second = first
let third = Object.new()

let same_reference = first same? second
let equal_by_protocol = first == third
```

Method 和 BoundMethod 也有身份。每次代码读取 `obj.method` 时，Iris 都会创建一个新的 BoundMethod 对象，捕获绑定时选定的接收者关系和精确 Method 身份。之后的普通发送会使用当前活动 Class 修订，但保存下来的 BoundMethod 保留其捕获的 Method 身份，并在调用时重新验证接收者成员关系。

```iris
class Counter {
  fun value() -> Integer { @value }
}

let counter = Counter.new()
let before = counter.value
let again = counter.value
```

这个片段改编自 `spec-snippets.json` 中来自 `03-runtime-object-model.md` 的运行时示例。

## Open Class 保持逻辑身份

Class 可以通过 `open class` 重新打开，但必须遵守静态骨架和验证规则。重新打开会发布同一逻辑 Class 身份的新活动修订。这是动态行为，但它不能移除或削弱已经承诺给现有类型化代码的静态事实。

```iris
let klass = Counter

open class Counter {
  override fun value() -> Integer { 2 }
}

let still_same_class = klass same? Counter
```

## 静态承诺，动态自由

Class 是动态的，因为它的活动修订可以通过授权的 open 操作改变。它也有静态承诺，因为已声明超类、已声明 Contract、可见成员名称和签名、属性契约、泛型元数以及其他静态骨架事实，都必须在新修订发布前保持兼容。

单继承只是组合的第一层。第 06 章由另一位译者处理，会介绍用于复用行为的 Module，以及用于命名义务表面的 Contract。当普通选择器身份不足以表达意图时，Contract 也需要显式 `..` 限定派发。

## 阅读规范

本章简化了以下规范性条款：

- [`IRIS-V1-RUNTIME-C003`](../../spec/iris-v1/03-runtime-object-model.md)：每个运行时值都是对象。
- [`IRIS-V1-RUNTIME-C009`](../../spec/iris-v1/03-runtime-object-model.md)：逻辑 Class 和活动修订。
- [`IRIS-V1-RUNTIME-C010`](../../spec/iris-v1/03-runtime-object-model.md)：重新打开保持逻辑 Class 身份。
- [`IRIS-V1-RUNTIME-C023`](../../spec/iris-v1/03-runtime-object-model.md)：普通消息身份。
- [`IRIS-V1-RUNTIME-C029`](../../spec/iris-v1/03-runtime-object-model.md)：原始 `same?`。
- [`IRIS-V1-RUNTIME-C038`](../../spec/iris-v1/03-runtime-object-model.md)：Method 绑定创建 BoundMethod。
- [`IRIS-V1-RUNTIME-C040`](../../spec/iris-v1/03-runtime-object-model.md)：每次 Method 读取创建不同的 BoundMethod 身份。
- [`IRIS-V1-RUNTIME-C046`](../../spec/iris-v1/03-runtime-object-model.md)：单 Class 继承加 Module 组合查找。
- [`IRIS-V1-RUNTIME-C054`](../../spec/iris-v1/03-runtime-object-model.md)：通过 `new` 标准构造。
- [`IRIS-V1-RUNTIME-C055`](../../spec/iris-v1/03-runtime-object-model.md)：没有 Class 命名构造函数语法或构造函数重载。
- [`IRIS-V1-RUNTIME-C066`](../../spec/iris-v1/03-runtime-object-model.md)：当前接收者上的原始 `@x` 存储。
- [`IRIS-V1-RUNTIME-C081`](../../spec/iris-v1/03-runtime-object-model.md)：显式 `super(args...)`。
- [`IRIS-V1-RUNTIME-C086`](../../spec/iris-v1/03-runtime-object-model.md)：带身份对象的默认相等性。
