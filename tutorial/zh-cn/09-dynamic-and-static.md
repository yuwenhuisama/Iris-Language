# 动态与静态的交汇

本章展示 Iris 如何将灵活的动态元编程与稳定的静态边界统一起来。Iris 允许程序重新打开类与模块、发布活动修订版本、检查元数据、定义装饰器以及挂载动态成员。然而，这些能力受到严格约束。候选变更在发布生效前，必须通过静态脊柱、MetaCapabilities、反射策略、契约承诺及事务隔离规则的全面验证。

<!-- iris-example: {"id":"09-open-class-reopen","mode":"vm","stdout":"2\n"} -->
```iris
class Counter {
  public fun value() -> Integer { 1 }
}

let c = Counter.new()

open class Counter {
  public override fun value() -> Integer { 2 }
}

print(c.value())
```

预期终端输出：

```text
2
```

重新打开类绝不会创建新的类对象。它完整保留了类的逻辑身份，并在验证通过后原子提交新的活动修订版本。

## Open 事务

声明式的 `open class` 和 `open module` 与编程式的 `Class#open` 及 `Module#open` 共享相同的事务模型。事务代码块在隔离环境中构建候选变更集。处于事务外部的代码继续观察当前已发布的修订版本，直到安全点原子发布。

**仅规范（不执行）：** 此编程式开放事务示意假设已经存在 `Tool` 类，本例省略其定义。

<!-- iris-example: {"id":"09-programmatic-open","mode":"spec-only","reason":"Specification example illustrating programmatic Class#open dynamic method definition"} -->
```iris
Tool.open() { |target: Class| -> Nil
  target.define_method(:status) { |self: Tool| -> Symbol; :ready }
}
```

编程式开放事务针对动态选定的类和模块对象。它们不可作用于契约、封闭泛型类或封闭泛型模块（`IRIS-V1-META-C033`）。事务具有同步执行、线程封闭、不可挂起以及不可逃逸的特征。若出现异常或冲突，候选集整体回滚，不发布任何部分变更。

## Active revisions 与保留的 Methods

普通消息分派始终基于当前的活动修订版本。已进入执行的方法栈帧保留其在调用发起时确定的方法实现体。已捕获的方法引用或绑定方法完整保留原有的方法身份，但在被调用时仍会重新核验接收者当前的 MRO 链是否仍然包含该方法的拥有者。

<!-- iris-example: {"id":"09-retained-methods","mode":"vm","stdout":"1\n2\n"} -->
```iris
class Counter {
  public fun value() -> Integer { 1 }
}

let c = Counter.new()
let retained = c.value

open class Counter {
  public override fun value() -> Integer { 2 }
}

print(retained.call())
print(c.value())
```

预期终端输出：

```text
1
2
```

已绑定的方法引用 `retained` 保留了原先的方法实现，而实例上的新调用 `c.value()` 则自动分派至新发布的活动修订版本。

## MetaCapabilities

头部子句 `meta deny` 用于静态收窄类或模块的动态演化能力。一旦某个类显式禁用了某项能力，后续的动态代码无法通过任何手段重新获取该权限。

**仅规范（不执行）：** 此能力限制示意依赖应用定义的 `load_config()` 及其结果的 `enabled?` 成员，本例均未提供。

<!-- iris-example: {"id":"09-meta-capabilities","mode":"spec-only","reason":"Specification example illustrating static meta deny capability restrictions"} -->
```iris
class Tool meta deny superclass, native {
  let enabled = load_config().enabled?

  if enabled {
    self.define_method(:debug) { |self: Tool| -> String; "debug" }
  }

  public fun run() -> String { "run" }
}
```

标准能力集合包括 `method_set`、`method_body`、`property_set`、`property_body`、`modules`、`superclass`、`subclass`、`shape`、`class_state_set`、`class_state_write`、`instance_state` 以及 `native`。未定义的能力标识符会触发编译错误。禁用某项具体能力绝不会隐式波及其他未声明的能力项。

## 导入显式命名包

包导入语句要么引用当前包内的模块，要么使用 `pkg::Module` 跨越包边界。包标识符采用全局唯一的反向域名格式。Iris 明确拒绝通配符导入与基于动态字符串的导入。

**仅规范（不执行）：** 这些导入形式假设外部 `org.dep` 包导出 `Codec`、`encode` 和 `decode`；本例未提供该依赖。

<!-- iris-example: {"id":"09-import-paths","mode":"spec-only","reason":"Specification example illustrating package-qualified imports and override import modifiers"} -->
```iris
import org.dep::Codec
from org.dep::Codec import encode, decode

override import org.dep::Codec
```

导入语句上的 `override` 修饰符用于显式授权目标依赖包引入的兼容扩展，既避免了意外的方法遮盖，又维护了静态契约的不可侵犯性。

## 装饰器转换候选

装饰器附着于声明。根据 `IRIS-V1-META-C087`，编译器或链接器执行纯粹、确定性的静态规划，运行时转换则在声明的初始构建或开放候选事务内执行。多个装饰器严格按书写的从上到下顺序执行（`IRIS-V1-META-C086`）。它们不能改变类的标称身份、修改包标识符或规避 `meta deny` 策略。

**仅规范（不执行）：** 此应用顺序示意省略了 `logged`、`memoized` 装饰器及应用方法 `compute_total` 的定义。

<!-- iris-example: {"id":"09-decorator-syntax","mode":"spec-only","reason":"Specification example illustrating method decorator application"} -->
```iris
@logged(level: :info)
@memoized()
public fun total() -> Integer {
  compute_total()
}
```

装饰器本身是实现了标准契约的普通类，具体契约包括 `ClassDecorator`、`ModuleDecorator`、`ContractDecorator`、`MethodDecorator` 与 `PropertyDecorator`。

**仅规范（不执行）：** 此无操作示意展示 `IRIS-V1-META-C124` 规定的当前契约签名，参数不加注解，以免虚构元数据类型名。运行时向 `transform` 提供第三个 `context` 参数；静态 `plan` 不接收事务上下文。此片段未将装饰器应用于任何声明。

<!-- iris-example: {"id":"09-decorator-contract-impl","mode":"spec-only","reason":"Current C124 decorator contract sketch without an application fixture"} -->
```iris
class Stamp for MethodDecorator {
  impl fun plan(declaration, arguments) -> Plan { Plan.empty }
  impl fun transform(declaration, arguments, context) -> Transformation { Transformation.empty }
}
```

`plan` 阶段是纯粹、静态且确定性的。运行时 `transform` 阶段在候选事务内执行，返回受控的 `Transformation`。即使某个阶段不作贡献，也必须声明两个成员；`Plan.empty` 与 `Transformation.empty` 表示相应的无操作结果。

## ReflectionPolicy，不是 reflection tokens

Iris 的反射系统输出经过严格权限过滤的不可变元数据视图，绝不向外暴露裸露的可变内部表，也不提供基于动态字符串的代码求值接口。反射相关操作集中在以目标类别划分的 `Reflection::*` 子模块中。

<!-- iris-example: {"id":"09-reflection-inspection","mode":"vm","stdout":"status\n"} -->
```iris
class Tool {
  public fun status() -> String { "ready" }
}

let method = Reflection::Class.method(Tool, :status)
print(method.selector)
```

预期终端输出：

```text
status
```

针对实例变量的反射检查与写入通过 `Reflection::Object` 进行：

<!-- iris-example: {"id":"09-reflection-ivars","mode":"vm","stdout":"99\n"} -->
```iris
class Box {}

let b = Box.new()
Reflection::Object.set_ivar(b, :@token, 99)
print(Reflection::Object.get_ivar(b, :@token))
```

预期终端输出：

```text
99
```

反射权限完全受控于 `ReflectionPolicy`，系统依据调用方所在的包、目标对象、申请的操作以及可见性范围进行权限判定，杜绝无边界的特权凭据传递。

## 为什么有界动态重要

Iris 明确划定边界：运行时变更可以替换兼容行为，但必须保留调用方依赖的声明。有界动态描述的是 Iris 自身的规则，并不意味着其他动态或静态语言缺少安全保障或适应能力。

调用方能够坚定信赖静态承诺：方法签名、契约一致性、泛型不变性与包命名始终不可撼动。与此同时，元编程依然保有发布兼容修订版本或挂载动态辅助方法的自由。中间的验证机制确保了任何破坏静态承诺的不完整更新绝不可能发布到活跃执行环境中。

## 静态承诺，动态自由

Iris 依靠坚实清晰的边界构建了可控的元编程体系。

**动态自由**
- 运行时可动态定义并发布兼容的类与模块活动修订版本。
- 支持动态扩展方法、调整混入并重新绑定调用。
- 装饰器将静态规划与运行时候选转换结合起来。

**静态承诺**
- 类的逻辑身份在版本迭代中始终唯一不变。
- 契约义务、泛型不变性与超类骨架不可被破坏。
- `meta deny` 永久锁定已声明禁用的结构变更能力。
- 开放事务保证原子化的安全点发布或彻底回滚。

**动手练习**

声明一个类 `Configuration`，包含方法 `mode() -> String { "debug" }`。使用 `open class Configuration` 重新打开该类并覆盖定义为 `mode() -> String { "release" }`。在已有实例上调用 `mode()`，打印输出以验证活动修订版本的动态分派。使用 `./target/debug/iris --vm test_open.iris` 验证。

预期终端输出：

```text
release
```

## 阅读规范

如需查阅元编程事务、权限判定与反射策略的形式化规范，请参考 [01-language-identity.md](../../spec/iris-v1/01-language-identity.md)、[03-runtime-object-model.md](../../spec/iris-v1/03-runtime-object-model.md) 与 [08-modules-metaprogramming.md](../../spec/iris-v1/08-modules-metaprogramming.md)：

| 条款 | 主题 |
| --- | --- |
| `IRIS-V1-IDENTITY-C008` 至 `IRIS-V1-IDENTITY-C014` | 静态承诺约束动态行为的基本准则。 |
| `IRIS-V1-RUNTIME-C009` 至 `IRIS-V1-RUNTIME-C022` | 逻辑类、活动修订版本、方法替换与原子回滚。 |
| `IRIS-V1-META-C030` 至 `IRIS-V1-META-C043` | 开放事务、候选隔离、冲突检测与安全点提交。 |
| `IRIS-V1-META-C044` 至 `IRIS-V1-META-C052` | 静态与动态成员可见性，禁止重载分派。 |
| `IRIS-V1-META-C072` 至 `IRIS-V1-META-C084` | MetaCapabilities 及其操作权限检查。 |
| `IRIS-V1-META-C085` 至 `IRIS-V1-META-C094` | 装饰器计算阶段与安全约束。 |
| `IRIS-V1-META-C118` 至 `IRIS-V1-META-C126` | `Reflection::*` 接口、装饰器契约、`Plan` 与 `Transformation`。 |
| `IRIS-V1-GRAMMAR-C068` 与 `IRIS-V1-GRAMMAR-C069` | 包限定导入语法与 `override` 导入修饰符。 |
| `IRIS-V1-META-C095` 至 `IRIS-V1-META-C112` | ReflectionPolicy 与权限过滤视图规范。 |
