# 类与对象

本章详细讲解 Iris v1 的对象模型。你将创建 Class、附加方法、定义存储属性、检视实例存储、理解继承机制，并区分引用身份与结构相等性。

## 已存储属性声明类型化槽位

`property` 简写形式用于定义类型化的属性槽位并自动生成对应的访问器。当需要定制逻辑时，可使用 `property fun` 定义显式属性访问器。

<!-- iris-example: {"id":"05-stored-properties","mode":"vm","stdout":"100\n"} -->
```iris
class Account {
  public property balance: Integer = 100
}
let a = Account.new()
print(a.balance)
```

预期输出：

```text
100
```

属性的读取通过 getter 派发，无需书写显式的圆括号。

## Class 是带有 new 消息的对象

在 Iris 中，Class 本身也是第一类运行时对象。通过向 Class 对象发送 `new` 消息来实例化类。Iris 不支持构造器重载或与类同名的特殊构造器语法。

<!-- iris-example: {"id":"05-classes-new","mode":"vm","stdout":"42\n"} -->
```iris
class Counter {
  public fun value() -> Integer { 42 }
}
let c = Counter.new()
print(c.value())
```

预期输出：

```text
42
```

调用 `Counter.new()` 会在堆上分配新的实例并在返回实例前派发初始化流程。

## 原始实例变量位于当前接收者

原始实例变量以 `@` 开头（例如 `@value`）。它们代表与当前接收者实例绑定的内部槽位存储。外部代码不能直接突破边界读写私有 ivar 状态，从而保证了封装性。

<!-- iris-example: {"id":"05-instance-state","mode":"vm","stdout":"0\n7\n"} -->
```iris
class Entity {
  public fun initialize() -> Nil {
    @val = 0
    nil
  }
  public fun set_val(n: Integer) -> Integer {
    @val = n
  }
  public fun get_val() -> Integer {
    @val
  }
}
let e = Entity.new()
print(e.get_val())
e.set_val(7)
print(e.get_val())
```

预期输出：

```text
0
7
```

相比之下，类级别的共享状态必须使用 `shared mut @@name` 或 `shared let @@name` 显式声明。

## Self 和 super 让接收者角色显式

关键字 `self` 在方法体内引用当前活跃的接收者实例。

类通过 `extends` 支持单继承。子类中的方法可以重写继承的行为并参与方法解析顺序。

<!-- iris-example: {"id":"05-inheritance-super","mode":"vm","stdout":"base derived\ntrue\n"} -->
```iris
class Base {
  public fun name() -> String { "base" }
}
class Derived extends Base {
  public override fun name() -> String { super() + " derived" }
}
let d = Derived.new()
print(d.name())
print(d is Base)
```

预期输出：

```text
base derived
true
```

调用 `super(args...)` 会在接收者的方法解析顺序（MRO）中调用下一个实现。

## 相等性不同于身份

Iris 对值相等性与对象身份保持明确的区分：

- 相等性（`==`）派发到对象所属类上定义的比较协议。
- 对象身份（`same?`）检查两个具有身份的值是否代表同一个可观察对象身份，而不是比较内存地址。任一操作数不具有身份时，都会抛出 `IdentityError`（`IRIS-V1-RUNTIME-C029`）。

<!-- iris-example: {"id":"05-equality-identity","mode":"vm","stdout":"true\nfalse\n"} -->
```iris
let first = Object.new()
let second = first
let third = Object.new()
print(first same? second)
print(first same? third)
```

预期输出：

```text
true
false
```

`first same? second` 结果为 `true`，因为两个标识符指向同一实例。`first same? third` 结果为 `false`，因为它们属于不同的分配。

## Open Class 保持逻辑身份

Iris 支持使用 `open class` 重开现有类。这会在原子发布新类修订版本的同时，严格保持既有的逻辑身份与静态契约保证。

<!-- iris-example: {"id":"05-open-class","mode":"vm","stdout":"updated\n"} -->
```iris
class Service {
  public fun status() -> String { "initial" }
}
open class Service {
  public override fun status() -> String { "updated" }
}
let s = Service.new()
print(s.status())
```

预期输出：

```text
updated
```

重开类会以原子方式发布活动修订版，从而在不使现有对象身份失效的前提下实现动态更新。

## 静态承诺，动态自由

类定义了静态骨架（static spine）：其超类、属性契约与成员签名构成了不可改变的承诺。动态操作（如重开类或替换方法实现）必须严格符合静态骨架的要求，修订才被允许发布。

**实战练习**

将 `Account` 示例保存为 `account.iris`。在第一次打印之后添加 `a.balance = 125` 和 `print(a.balance)`。运行 `./target/debug/iris --vm account.iris`，确认依次输出 `100`、`125`：生成的 setter 改变存储槽位的值，同时保持其 `Integer` 契约。

## 阅读规范

本章内容简化并对应于以下规范性条款：

- [`IRIS-V1-RUNTIME-C003`](../../spec/iris-v1/03-runtime-object-model.md)：运行时对象性。
- [`IRIS-V1-RUNTIME-C009`](../../spec/iris-v1/03-runtime-object-model.md)：逻辑类与活动修订。
- [`IRIS-V1-RUNTIME-C010`](../../spec/iris-v1/03-runtime-object-model.md)：重开类保留逻辑 Class 身份。
- [`IRIS-V1-RUNTIME-C023`](../../spec/iris-v1/03-runtime-object-model.md)：普通消息身份。
- [`IRIS-V1-RUNTIME-C029`](../../spec/iris-v1/03-runtime-object-model.md)：通过 `same?` 进行原始身份比较。
- [`IRIS-V1-RUNTIME-C038`](../../spec/iris-v1/03-runtime-object-model.md)：Method 绑定产生 BoundMethod。
- [`IRIS-V1-RUNTIME-C040`](../../spec/iris-v1/03-runtime-object-model.md)：BoundMethod 实例的独立身份。
- [`IRIS-V1-RUNTIME-C046`](../../spec/iris-v1/03-runtime-object-model.md)：单类继承与 MRO 查找。
- [`IRIS-V1-RUNTIME-C054`](../../spec/iris-v1/03-runtime-object-model.md)：通过 `new` 进行对象构造。
- [`IRIS-V1-RUNTIME-C055`](../../spec/iris-v1/03-runtime-object-model.md)：不存在同名构造器重载。
- [`IRIS-V1-RUNTIME-C066`](../../spec/iris-v1/03-runtime-object-model.md)：通过 `@ivar` 访问接收者实例存储。
- [`IRIS-V1-RUNTIME-C081`](../../spec/iris-v1/03-runtime-object-model.md)：显式调用 `super(args...)`。
- [`IRIS-V1-RUNTIME-C086`](../../spec/iris-v1/03-runtime-object-model.md)：默认对象相等性语义。
- [`IRIS-V1-RUNTIME-C161`](../../spec/iris-v1/03-runtime-object-model.md)：存储属性的单一支持槽位。
- [`IRIS-V1-RUNTIME-C162`](../../spec/iris-v1/03-runtime-object-model.md)：共享类级存储。
- [`IRIS-V1-RUNTIME-C164`](../../spec/iris-v1/03-runtime-object-model.md)：修订迁移约定。
- [`IRIS-V1-GRAMMAR-C058`](../../spec/iris-v1/02-lexical-grammar.md)：属性简写与访问器定义。
- [`IRIS-V1-GRAMMAR-C064`](../../spec/iris-v1/02-lexical-grammar.md)：属性作用域修饰符。
