# 动态与静态的交汇

本章是前面章节的落点。Iris 允许程序重新打开 Classes 和 Modules，发布新的 active revisions，添加动态成员，应用装饰器，并检查元数据。这些能力有边界。候选变更必须先通过静态脊柱、MetaCapabilities、包与 ReflectionPolicy、Contract obligations 和事务规则的验证，之后才能可见。

```iris
let klass = Counter
let first = Counter.new()

open class Counter {
  override fun value() -> Integer { 2 }
}

klass same? Counter        // true, reopen kept the logical Class identity
first.value()             // later send uses the current active revision
```

这段代码复用自运行时示例。重新打开 Class 不会创建第二个 Class object。它保留逻辑 Class 身份，并在验证后发布新的 active revision。

## Open 事务

`open class Name { ... }` 和 `open module Name { ... }` 使用与程序式 `Class#open` 和 `Module#open` 相同的事务模型。主体构建一个候选。事务外部的代码在提交前仍然看到已发布的 active revision。

```iris
Tool.open() { |target: Class| -> Nil
  target.define_method(:status) { |self: Tool| -> Symbol; :ready }
}
```

这复用自 `IRIS-V1-META-EX003`。程序式 opens 可以面向动态选择的 Class 和 Module objects。它们仍然不能面向 Contracts 或闭合泛型 Classes。它们是同步的、线程受限的、不可挂起的、不可逃逸的。

如果主体正常完成，候选会被验证。如果验证成功，Iris 会在 safepoint 发布。如果主体 raises、capability check 失败、permission check 失败，或检测到冲突，整个候选组都会回滚，什么也不发布。

## Active revisions 与保留的 Methods

普通 sends 使用当前 active revision。已经进入的 Method frames 保留调用进入时选中的主体。Captured Method 或 BoundMethod objects 保留它们的旧 Method 身份，但调用时会重新验证接收者当前 MRO 仍然包含该 Method 的 owner。

```iris
let before = counter.value
let again = counter.value
before same? again        // false, each read creates a BoundMethod identity

open class Counter {
  override fun value() -> Integer { 3 }
}

before()                  // invokes the captured old Method after revalidation
counter.value()           // ordinary lookup uses the new Method
```

这段代码复用自运行时示例。这个区别对工具和优化器很重要。它们可以缓存，但只能用尊重 revision 和 Method 身份的 guards。

## MetaCapabilities

`meta deny` 是静态头部子句。它收窄之后可发生的结构操作。源码不能写 `allow` 形式来取回已否定的能力。

```iris
class Tool meta deny superclass, native {
  let enabled = load_config().enabled?

  if enabled {
    self.define_method(:debug) { |self: Tool| -> String; "debug" }
  }

  public fun run() -> String { "run" }
}
```

这个示例复用自 `IRIS-V1-META-EX002`。条件中的 `define_method` 只是动态可见，因为它依赖可执行主体控制流。它可以通过获准反射或动态派发找到，但不会成为已经编译的静态 API 的一部分。

v1 capability 名称包括 `method_set`、`method_body`、`property_set`、`property_body`、`modules`、`superclass`、`subclass`、`shape`、`class_state_set`、`class_state_write`、`instance_state` 和 `native`。未知的 `meta deny` 名称是错误。否定一个 capability 不会静默否定所有其他 capabilities。

## 导入显式命名包

导入路径要么命名当前包内的 Module，要么用 `pkg::Module` 跨越包边界，其中 package 段是反向域名形式的 `package_id`。没有通配导入，也没有运行时字符串导入。

```iris
import org.dep::Codec
from org.dep::Codec import encode, decode

override import org.dep::Codec
```

`import` 或 `from` 前可选的 `override` marker 会授权被导入扩展贡献的兼容替换。它绝不授权签名或静态 Contract 不兼容，并且未标记的替换仍会被拒绝。该 marker 位于整个 import 上，不位于单个名称上。

## 装饰器转换候选

装饰器附加到声明，并转换声明候选元数据。多个装饰器按从上到下运行。它们不会把 Class 改成 Module，不会替换名义身份，不会重写包身份，也不会绕过 MetaCapabilities。

```iris
@logged(level: :info)
@memoized()
public fun total() -> Integer {
  compute_total()
}
```

这复用自 `IRIS-V1-META-EX005`。装饰器用 `@Name(arguments)` 应用于 Class、Module、Contract、Method 或 property 声明之前，名称也可以限定，例如 `@D::Stamp()`，以到达另一个 Module 中声明的装饰器。

装饰器是普通 Class，它为五个 Decorator Contracts 之一声明 `for`，每种目标 kind 一个：`ClassDecorator`、`ModuleDecorator`、`ContractDecorator`、`MethodDecorator` 和 `PropertyDecorator`。每个都恰好要求两个 members，符合的装饰器即使只参与一个阶段，也要同时声明二者。

```iris
class Stamp for MethodDecorator {
  impl fun plan(declaration: MethodDeclaration, arguments: Array<Object>) -> Plan {
    Plan.empty
  }

  impl fun transform(
    declaration: MethodDeclaration,
    arguments: Array<Object>,
    context: TransformContext
  ) -> Transformation {
    Transformation.add_method(:stamped) { |self: Object| -> Symbol; :stamped }
  }
}
```

`plan` 是静态阶段：纯、确定，并且受限于输入白名单。读取任何白名单之外的内容都是 phase static 的 `IRIS-DECORATOR-NONDETERMINISTIC`，不会发生运行时 transform 或目标发布。`transform` 是运行时阶段，在声明的候选事务内运行；它接收受控 transform context，并返回同 kind 的 `Transformation`，而不是改变 candidate handle。`kind` 不同于被装饰目标的 `Transformation` 是 `IRIS-DECORATOR-KIND`，目标不保留任何候选或 revision。

`Plan.empty` 和 `Transformation.empty` 是无贡献值。`Transformation.add_method(selector, body)` 暂存一个 Method，并需要与手写声明相同的 `method_set` capability。对已应用装饰器的反射会把生成的 diff 显示为不可变的按权限过滤视图，绝不是可变 handle。

## ReflectionPolicy，不是 reflection tokens

Reflection 返回按权限过滤的不可变元数据视图。它不会交出原始可变表，也不会提供 eval-string mutation。结构性变更仍然要经过 open 或 meta 事务。操作按目标 kind 位于 `Reflection::*` sub-Modules 中。

```iris
let names = Reflection::Object.list_ivars(object)
let old = Reflection::Object.get_ivar(object, :@cache)
Reflection::Object.set_ivar(object, :@cache, compute())

let reader = Reflection::Class.method(Tool, :status)
let ancestry = Reflection::Class.ancestors(Tool)
```

这段代码改编自 `IRIS-V1-META-EX004`。`Reflection::Class` 还携带 `invoke`、`remove_module`、`set_superclass` 和总是被拒绝的 `remove_contract`；`Reflection::Module` 携带 Module 侧的 `method` 和 `invoke`。找不到内容的 lookup 返回 `nil`。检查操作需要 `inspect`，变更操作需要 `mutate`。

这些通过 mixin 编织进 `Class` 和 `Module`，所以 `Tool.remove_module(M)` 和 `Reflection::Class.remove_module(Tool, M)` 是同一实现的两个入口点。

Reflection 授权来自 `ReflectionPolicy`：caller package、operation、granted scope、target identity 和 target policies。Iris v1 没有一等 reflection capability token。


## 为什么有界动态重要

许多动态语言允许你改变 class。许多静态语言给调用者稳定的类型承诺。Iris 试图同时保留两者，但它拒绝让不稳定的部分变得不可见。

普通调用者可以依赖已声明的 Method signature、Contract 集、泛型不变性和包身份。元程序仍然可以替换兼容的 Method body，或添加只动态可见的 helper。Validator 站在这两个世界之间。它拒绝会降级静态脊柱、隐藏部分候选，或发布半变更运行时的改动。

这让工具成为可能，同时不把语言变成静态语言。编译器、反射用户、Host 嵌入层或一致性套件都可以命名那些会在运行时变更后继续存在的承诺。

## 静态承诺，动态自由

动态自由：兼容的 Class 和 Module revisions 可以在运行时构建，Methods 可以被替换，Modules 可以重新组合，dynamic-only members 可以出现，装饰器可以包装声明。

静态承诺：逻辑 Class 身份、静态脊柱事实、Contract obligations、泛型元数和不变性、MetaCapabilities、ReflectionPolicy、包身份，以及原子发布或回滚都会持久保留。没有普通 send 会观察到部分候选。

## 阅读规范

精确规则请阅读 [01-language-identity.md](../../spec/iris-v1/01-language-identity.md)、[03-runtime-object-model.md](../../spec/iris-v1/03-runtime-object-model.md) 和 [08-modules-metaprogramming.md](../../spec/iris-v1/08-modules-metaprogramming.md)：

| 条款 | 主题 |
| --- | --- |
| `IRIS-V1-IDENTITY-C008` through `IRIS-V1-IDENTITY-C014` | 约束动态行为的静态承诺。 |
| `IRIS-V1-RUNTIME-C009` through `IRIS-V1-RUNTIME-C022` | 逻辑 Classes、active revisions、Method replacement 和 rollback 基础。 |
| `IRIS-V1-META-C030` through `IRIS-V1-META-C043` | Open 事务、候选隔离、冲突、safepoint commit 和 rollback。 |
| `IRIS-V1-META-C044` through `IRIS-V1-META-C052` | 静态与动态成员可见性，以及没有 overload dispatch。 |
| `IRIS-V1-META-C072` through `IRIS-V1-META-C084` | MetaCapabilities 和 operation checks。 |
| `IRIS-V1-META-C085` through `IRIS-V1-META-C094` | 装饰器阶段和限制。 |
| `IRIS-V1-META-C118` through `IRIS-V1-META-C126` | `Reflection::*` surfaces、Decorator Contracts、`Plan` 和 `Transformation`。 |
| `IRIS-V1-GRAMMAR-C068` and `IRIS-V1-GRAMMAR-C069` | 包限定导入路径和 `override` import marker。 |
| `IRIS-V1-META-C095` through `IRIS-V1-META-C112` | ReflectionPolicy 和 reflection views。 |
