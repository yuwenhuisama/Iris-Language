# 模块与 Contract

本章说明 Iris 如何将行为复用与契约承诺明确分离。`module` 提供可复用的方法实现，供类或其他模块混入使用。`contract` 声明明确的义务接口，类通过 `for` 显式承诺该契约，并通过 `impl` 提供具体实现。两者相互配合，但绝不可互换混淆：模块决定运行时的查找顺序，而契约定义经过检查的槽位、视图以及限定分派。

<!-- iris-example: {"id":"06-composition","mode":"vm","stdout":"B\n"} -->
```iris
module A {
  public fun trace() -> String { "A" }
}

module B mixin A {
  public override fun trace() -> String { "B" }
}

class C mixin A, B {}

print(C.new().trace())
```

预期终端输出：

```text
B
```

类拥有单一超类继承链，随后是组合进来的各模块。头部 `mixin A, B` 声明按从左到右应用，但在方法查找时会优先检查距离当前类最近的模块。因此，实际的查找顺序依次为 `C`、`B`、`A`，最后进入超类继承链。

## Modules 添加行为

模块是容纳方法的具名对象。类可以在声明头部混入模块。模块自身也可以组合其他模块，从而将公共行为组织为结构清晰的复用层。

<!-- iris-example: {"id":"06-modules-add-behavior","mode":"vm","stdout":"Hello, Iris\n"} -->
```iris
module Greeter {
  public fun hello(name: String) -> String {
    "Hello, " + name
  }
}

class User mixin Greeter {}

let u = User.new()
print(u.hello("Iris"))
```

预期终端输出：

```text
Hello, Iris
```

模块混入不同于多重继承。类始终只有一个直接超类。组合的模块会被插入到方法解析顺序（MRO）中，并依据模块身份完成去重。重复混入相同的模块不会创建多个副本，也不会移动已有的查找位次。

模块方法在当前接收者的上下文中执行。当模块方法访问 `@state` 时，它读写的是接收者实例上的 `@state` 变量，而非模块私有的独立存储。私有方法访问受到严格约束：模块组合默认不授予私有方法访问权限，除非在混入列表中显式使用 `private` 标记。

**仅规范（不执行）：** 下面仅展示私有混入授权的声明语法（`IRIS-V1-GRAMMAR-C061`），不实际调用私有方法。根据 `IRIS-V1-RUNTIME-C050`，该边授权模块方法访问宿主类的私有方法，而非反向授权。

<!-- iris-example: {"id":"06-mixin-private-access","mode":"spec-only","reason":"Declaration-only illustration of Module access to host Class private methods; no private call is exercised"} -->
```iris
module Trace {
  fun internal_log() -> Nil { nil }
}

class Job mixin Trace private {}
```

`private` 关键字直接声明在具体的组合边上。在本例中，`Trace` 获得调用 `Job` 私有方法的授权，而未标记的混入边不具备该权限。授权范围限定于宿主逻辑类、封闭模块身份及组合边修订版本。

## Contracts 声明承诺

契约是 Iris 中声明明确义务的规范接口。契约主体只包含方法签名要求，不包含具体实现体或存储字段。在契约主体内编写方法实现会触发语法错误。类使用 `for` 关键字承诺遵循契约，并使用 `impl` 标记实现方法。

<!-- iris-example: {"id":"06-contracts-declare-promises","mode":"vm","stdout":"Beep boop\n"} -->
```iris
contract Speaker {
  fun speak() -> String
}

class Robot for Speaker {
  public impl fun speak() -> String {
    "Beep boop"
  }
}

let bot = Robot.new() as Speaker
print(bot..speak())
```

预期终端输出：

```text
Beep boop
```

声明 `for Speaker` 构成了 `Robot` 类不可动摇的静态脊柱事实。后续的动态变更可以替换兼容的方法体，但无法抹除 `Robot` 已承诺 `Speaker` 的既定事实。

`impl` 标记明确表示该方法旨在满足契约要求。如果一个方法在满足契约的同时，还重写了继承或混入的方法，两个修饰符需一并书写，例如 `override impl fun draw() -> Nil { ... }`。

## 限定 Contract slots

普通消息分派从不根据静态类型进行重载。如果两个契约定义了相同名称但签名不同的要求，Iris 通过限定契约槽位消除歧义。开发者为每个契约编写限定实现，并通过契约视图使用 `..` 运算符显式调用。

<!-- iris-example: {"id":"06-qualified-contract-slots","mode":"vm","stdout":"parse: text\nvalidate: text\n"} -->
```iris
contract Parser {
  fun process(input: String) -> String
}

contract Validator {
  fun process(input: String) -> String
}

class Tool for Parser, Validator {
  public impl fun Parser::process(input: String) -> String {
    "parse: " + input
  }
  public impl fun Validator::process(input: String) -> String {
    "validate: " + input
  }
}

let tool = Tool.new()
let p = tool as Parser
let v = tool as Validator
print(p..process("text"))
print(v..process("text"))
```

预期终端输出：

```text
parse: text
validate: text
```

在声明中，`Contract::selector` 将方法绑定到特定的契约限定槽位。在表达式中，`view..selector` 直接调用该限定槽位。使用单点操作符 `view.selector` 时，仍执行常规的接收者方法查找。

## 静态承诺，动态自由

Iris 在动态灵活性与静态保障之间保持平衡。

**动态自由**
- 类可以在声明期或受控事务中混入新模块。
- 方法实现体可以在运行时被动态替换为兼容版本。
- 遵循契约的实例可以在运行时自由构建契约视图。

**静态承诺**
- 类保持单一超类的继承骨架。
- 已声明的契约承诺不能被动态消除或弱化。
- 普通方法选择器与契约限定槽位属于严格隔离的命名空间。
- 试图通过反射调用 `remove_contract` 移除已声明契约的行为，会在发布前直接抛出 `TypeContractError`。

## Kernel 始终在作用域内

`Kernel` 是直接组合到 `Object` 中的核心模块。它定义了全局可见的基础标识符与类型别名，无需显式导入即可使用。

`Kernel` 是遵循标准规则的普通模块，并非第二根类。`Kernel` 不允许覆盖或遮盖 `Object` 预设的基础行为，包括默认相等性比较、真值判断、缺失消息处理以及无参构造器。

**动手练习**

编写一个契约 `Describable`，要求方法 `fun describe() -> String`。编写一个模块 `Tagged`，提供方法 `public fun tag() -> String { "[tag]" }`。接着声明类 `Item for Describable mixin Tagged` 并实现 `describe`。实例化 `Item`，将其转换为 `Describable` 视图，分别打印描述内容和标签。使用 `./target/debug/iris --vm item.iris` 运行验证。

预期终端输出：

```text
item-ready
[tag]
```

## 阅读规范

如需查阅规范原文与形式化定义，请参考 [03-runtime-object-model.md](../../spec/iris-v1/03-runtime-object-model.md) 与 [05-types-contracts-generics.md](../../spec/iris-v1/05-types-contracts-generics.md)：

| 条款 | 主题 |
| --- | --- |
| `IRIS-V1-RUNTIME-C046` 至 `IRIS-V1-RUNTIME-C053` | 模块组合与 MRO 排序规则。 |
| `IRIS-V1-RUNTIME-C030` 至 `IRIS-V1-RUNTIME-C033` | 契约视图分派与缺失限定槽位处理。 |
| `IRIS-V1-RUNTIME-C163` | `Kernel` 核心全局模块规范。 |
| `IRIS-V1-TYPES-C041` 至 `IRIS-V1-TYPES-C053` | 契约声明、`for`、`impl`、限定实现、视图、相等性与哈希规则。 |
| `IRIS-V1-TYPES-C098` 与 `IRIS-V1-TYPES-C099` | 保护继承链与禁止 `remove_contract`。 |
| `IRIS-V1-GRAMMAR-C061` | 混入项上的 `private` 标记。 |
| `IRIS-V1-GRAMMAR-C062` | 作为契约要求的无主体方法声明。 |
| `IRIS-V1-IDENTITY-C009` 与 `IRIS-V1-IDENTITY-C010` | 禁止静态类型重载分派。 |
