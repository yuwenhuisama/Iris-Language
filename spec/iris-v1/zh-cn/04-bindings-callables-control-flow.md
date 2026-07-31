# Iris v1 绑定、可调用体与控制流

状态：Iris v1.15，冻结语义并有所有者批准的勘误。

IRIS-V1-CONTROL-C001: 本章定义 Iris v1 的绑定、作用域、名称查找、可调用运行时种类、参数绑定、Closure 捕获与返回、调用与尾随块、赋值、条件、循环、match、Iterator 降低、异常，以及控制转移结果。它 MUST 在 [README.md](README.md)、[02-lexical-grammar.md](02-lexical-grammar.md) 和 [03-runtime-object-model.md](03-runtime-object-model.md) 之后阅读。

IRIS-V1-CONTROL-C002: 本章 MUST NOT 定义新 token、替代解析器产生式、对象布局、类型代数细节、async 调度、资源辅助 API、包加载，或一致性运行器格式。它细化 [02-lexical-grammar.md](02-lexical-grammar.md) 已命名的语法形式，以及 [03-runtime-object-model.md](03-runtime-object-model.md) 中的运行时可调用体、派发、属性、真值性和存储锚点。

## 绑定声明与确定赋值

IRIS-V1-CONTROL-C003: 局部绑定使用 `let` 表示不可变绑定，使用 `mut` 表示可变绑定。`let name[: T] = expr` MUST 声明一个已初始化的不可变词法绑定。`mut name[: T] = expr` MUST 声明一个已初始化的可变词法绑定。`mut name: T` MAY 声明一个延迟初始化的可变绑定。裸 `name = expr` MUST 赋值给已有的可变绑定，并且 MUST NOT 声明新的局部绑定。

IRIS-V1-CONTROL-C004: `let` MUST 有初始化器。无类型且未初始化的 `mut name` MUST 以 `BINDING_MISSING_TYPE_FOR_DEFERRED_INIT` 拒绝。在确定赋值之前读取延迟的可变绑定，MUST 按编译期知识引发或诊断 `DefiniteAssignmentError`。绑定已有声明或推断的局部类型之后，后续赋值 MUST 满足该固定类型，并且 MUST NOT 隐式拓宽它。

IRIS-V1-CONTROL-C005: 绑定注解是绑定单元的静态和运行时 Contract。没有注解时，初始化器的精确静态类型成为该绑定的固定局部类型。需要更宽目标类型的程序 MUST 显式写出它，例如 `mut value: String | Integer = "ready"` 或 `mut value: Dynamic<Object> = source`。

IRIS-V1-CONTROL-C006: 参数绑定、catch 绑定和逐迭代 `for` 绑定默认不可变，除非它们自己的声明形式显式使用 `mut`。`_` 是丢弃绑定。它接受一个值但不创建可读绑定，在绑定模式允许的位置 MAY 重复出现，MUST NOT 被捕获，并且作为表达式读取时 MUST 是编译期错误。

IRIS-V1-CONTROL-C007: 下列绑定表是规范性的：

| 源形式 | 绑定类别 | 初始化规则 | 重新赋值规则 | 初始化前读取 | 赋值结果 |
| --- | --- | --- | --- | --- | --- |
| `let x = expr` | 不可变局部 | 必需 | 禁止 | 若被接受则不可能 | 不适用 |
| `let x: T = expr` | 不可变类型化局部 | 必需且按`T`检查 | 禁止 | 若被接受则不可能 | 不适用 |
| `mut x = expr` | 可变推断局部 | 必需 | 在推断类型内允许 | 若被接受则不可能 | 存储的值 |
| `mut x: T = expr` | 可变类型化局部 | 必需且按`T`检查 | 在`T`内允许 | 若被接受则不可能 | 存储的值 |
| `mut x: T` | 可变类型化延迟局部 | 延迟 | 第一次赋值初始化，后续赋值更新 | `DefiniteAssignmentError` | 存储的值 |
| Parameter or catch binding | 不可变词法绑定 | 由调用或 catch 绑定 | 禁止 | 若被接受则不可能 | 不适用 |
| `_` | 丢弃 | 接受并忽略值 | 不可寻址 | 读取是编译期错误 | 不适用 |

IRIS-V1-CONTROL-EX001: Informative example，本地绑定和确定赋值：

```iris
let name: String = "Iris"
mut count: Integer
count = 1

mut value: String | Integer = "ready"
value = 42
```

## 存储标记、作用域与查找

IRIS-V1-CONTROL-C008: `@name` 表示 [03-runtime-object-model.md](03-runtime-object-model.md) 下当前接收者的原始存储。它 MUST NOT 用作 `other.@name`。`@@name` 表示锚定到逻辑 Class 的已声明层级 Class 变量绑定单元。`$name` 表示由 `global let` 或 `global mut` 声明的包限定运行时全局。通过 `@@name` 或 `$name` 读写 MUST 要求匹配的声明。

IRIS-V1-CONTROL-C009: 标记语法 MUST NOT 仅因使用而创建绑定。对不存在的未声明 `@name` 赋值，MAY 仅在接收者的 `instance_state` 策略下创建原始当前接收者槽。对不存在的 `@@name` 或 `$name` 赋值 MUST 以缺少已声明存储失败。对不存在的普通局部 `name` 赋值 MUST 以 `NameError` 或静态未解析绑定诊断失败。

IRIS-V1-CONTROL-C010: Module 可执行体、Method 体、Closure 体、大括号块、条件分支、循环体、match 分支、catch 体和 `finally` 体会创建词法作用域。同一个词法作用域 MUST NOT 重新声明同一个普通绑定名。嵌套作用域 MAY 遮蔽外层名称，并且新声明的初始化器 MUST 解析到外层绑定，因为新绑定只在其初始化器完成后才进入作用域。

IRIS-V1-CONTROL-C011: 非限定名称解析 MUST 从最内层向外搜索局部词法绑定，然后搜索可见常量、Types、Module 声明、包声明和显式导入。没有可调用词法绑定或声明绑定的语法调用 `name(...)` MUST 成为对当前 `self` 或模块 `main` 的隐式特权发送。非调用的未解析裸 `name` MUST 引发或诊断 `NameError`；它 MUST NOT 隐式读取属性、Method、全局或运行时添加的成员。

IRIS-V1-CONTROL-C012: 顶层可执行代码只存在于 Module 体内。每个可执行 Module 拥有一个带身份的 `main` 接收者。该 Module 内的顶层 `fun f(...)` 会在 `main` 上创建 Method，默认私有。同一 Module 体内的顶层 `f(...)` 是对 `main` 的特权隐式发送。导入者只有在顶层辅助项被导出或通过 Module API 规则以其他方式公开时，才能调用它。

IRIS-V1-CONTROL-C013: 下列查找表是规范性的：

| 源形式 | 查找目标 | 缺失行为 | 创建行为 |
| --- | --- | --- | --- |
| `name` | 词法绑定，然后是可见声明和导入 | `NameError` | 从不创建 |
| `name(args...)` | 词法或声明可调用体，否则是当前`self`或`main`普通发送 | 普通派发失败或`method_missing` | 从不创建局部 |
| `@name` | 当前接收者原始槽 | 只有在原始 ivar 规则下缺失读取才返回`nil` | 仅当原始 ivar 扩展被授权时可创建 |
| `@@name` | 已声明层级 Class 变量单元 | 缺少声明错误 | 只能声明创建 |
| `$name` | 已声明包限定运行时全局 | 缺少声明错误 | 只能用 `global let` 或 `global mut` |
| `obj.name` | 普通属性 getter 选择器`name` | 只有真正不存在时才 `method_missing` | 从不创建存储 |

IRIS-V1-CONTROL-EX002: Informative example，遮蔽和隐式 self 调用：

```iris
module Demo {
  fun helper() -> String { "module" }

  fun run() -> String {
    let helper_value = helper()
    {
      let helper_value = "inner"
      helper_value
    }
    helper_value
  }
}
```

## 可调用运行时种类与声明

IRIS-V1-CONTROL-C014: Iris v1 恰好有三种普通可调用运行时种类：Method、BoundMethod 和 Closure。它 MUST NOT 暴露单独的 Function 运行时种类。每个具名 `fun` 声明都在其词法拥有者上定义一个 Method。读取或绑定实例 Method 会按运行时规则创建 BoundMethod。Closure 是匿名词法代码和捕获。

IRIS-V1-CONTROL-C015: 具名 Method 语法是 `fun selector(parameter_list) -> ReturnType { body }`，可带可见性、`override`、`impl`、`async`、`class`、泛型参数、`where` 和属性修饰符，均与语法定义完全一致。Method 所有权由放置位置决定。实例和 Module Methods 附着到包围的 Class 或 Module。`class fun` 在 Class 对象上安装单例 Method。Module 体内的 `fun` 在该 Module 的 `main` 接收者上安装 Method。

IRIS-V1-CONTROL-C016: Closure 语法是 `{ |parameters| -> ReturnType body }`。对于单行体，头部 MUST 以 `;` 结束，例如 `{ |x: String| -> Nil; print(x) }`。对于多行体，主体在头部终止符之后开始。空参数使用 `{ || -> R body }`。显式返回类型形式是 `|parameters| -> ReturnType`；只有当预期可调用上下文唯一提供一个闭合函数类型时，返回注解 MAY 省略。

IRIS-V1-CONTROL-C017: Method 省略返回注解按全局类型默认规则处理，在没有显式注解时 MUST 视为 `Dynamic<Object>`。Closure 省略返回注解在没有唯一预期可调用类型时 MUST NOT 默认其公共类型；它 MUST 被诊断为 `CALLABLE_MISSING_CLOSURE_RETURN_TYPE`。

IRIS-V1-CONTROL-C018: 已被 v1.11 勘误中的 IRIS-V1-TYPES-C094 与 IRIS-V1-TYPES-C096 取代；可调用 Type 现在须指明其种类，写作 `Closure<S>` 或 `BoundMethod<S>`，且签名兼容性在调用处检查而非通过变型。被取代的原文为：BoundMethod 和 Closure 值在绑定后共享普通可调用函数类型，写作 `(P1, P2, ...) -> R`。可赋值性 MUST 使用函数兼容性：参数位置逆变，返回位置协变，并且元数、参数类别、关键字名称、块通道和运行时 Contracts MUST 兼容。未绑定 Method 值仍是反射性 Method 对象，需要显式绑定或接收者调用。

IRIS-V1-CONTROL-C019: 正常 Method 和 Closure 穿透返回最终表达式值。没有产出值语句的主体返回 `nil`。`return expr` 以 `expr` 退出当前 Method 或当前 Closure 调用；裸 `return` 以 `nil` 退出。每条正常返回路径 MUST 满足可调用体的显式返回 Contract，并且声明为 `-> Never` 的可调用体 MUST NOT 正常完成。

IRIS-V1-CONTROL-C020: Closure 内的 `return` 只以该 Closure 调用为目标。Method 内的 `return` 只以该 Method 帧为目标。Closure 内的 `break` 和 `continue` MUST NOT 以 Closure 调用边界外的循环为目标。这种目标跨越在静态可见时 MUST 是编译期错误，否则在执行无效转移之前成为控制目标错误。

IRIS-V1-CONTROL-C021: 下列可调用种类表是规范性的：

| 种类 | 创建方式 | 身份 | 接收者或捕获 | 调用结果规则 |
| --- | --- | --- | --- | --- |
| Method | 具名`fun`、`property fun`、生成的访问器，或兼容元操作 | 带身份的定义对象 | 词法拥有者，不绑定接收者 | 除非通过普通派发调用，否则需要接收者或显式绑定 |
| BoundMethod | 在接收者上读取或绑定 Method | 带身份的可调用对象 | 捕获接收者关系和确切 Method 身份 | 重新验证当前拥有者成员资格并返回 Method 体结果 |
| Closure | 求值 Closure 字面量或尾随块 | 带身份的可调用对象 | 捕获词法绑定单元和当前接收者 | 运行 Closure 体并只返回局部结果 |

IRIS-V1-CONTROL-EX003: Informative example，Method、BoundMethod 和 Closure 形式：

```iris
class Counter {
  fun initialize() -> Nil {
    @value = 0
  }

  fun add(delta: Integer) -> Integer {
    @value += delta
  }
}

let counter = Counter.new()
let bound: BoundMethod<(Integer) -> Integer> = counter.add
let closure: Closure<(Integer) -> Integer> = { |delta: Integer| -> Integer; counter.add(delta) }
```

## 参数、默认值与关键参数

IRIS-V1-CONTROL-C022: 参数声明顺序 MUST 是必需位置参数、可选位置参数、至多一个位置 rest、必需 keyword-only、可选 keyword-only、至多一个 keyword rest，然后一个尾随块绑定。把较晚类别放在较早类别之前、重复 rest 类别，或声明多于一个块绑定的声明，MUST 被诊断为 `PARSE_BAD_PARAMETER_ORDER` 或更严格的可调用诊断。

IRIS-V1-CONTROL-C023: 必需位置参数使用 `name: T`。可选位置参数使用 `name: T = default`。位置 rest 使用 `*args: T`，并绑定一个新的 `Array<T>`，其中含有额外位置实参。必需 keyword-only 参数使用 `key name: T`。可选 keyword-only 参数使用 `key name: T = default`。Keyword rest 使用 `**kwargs: V`，并绑定一个新的 `Hash<Symbol,V>`，其中含有未匹配的关键字实参。尾随块绑定使用 `&block: Block<(P...) -> R>`，并且 MAY 使用 `= nil` 将省略标记为可接受。

IRIS-V1-CONTROL-C024: 参数默认表达式在每次调用时从左到右求值，在更早参数已绑定之后，在更晚参数存在之前。默认值 MAY 引用当前接收者、Module 作用域、全局和更早的参数绑定。默认值 MUST NOT 引用更晚参数。引发异常的默认值会在进入主体之前中止调用并传播异常。

IRIS-V1-CONTROL-C025: 参数绑定是不可变绑定单元。Rest 和 keyword-rest 容器每次调用都是新的，但容器值保留其普通可变性。省略的可选块绑定为 `nil`。元数、重复关键字、没有 `**kwargs` 的未知关键字、缺少必需关键字、缺少必需位置参数、意外块以及参数 Contract 失败，MUST 在选择器解析之后引发 `ArgumentError` 或 `TypeError`，并且 MUST NOT 调用 `method_missing`。

IRIS-V1-CONTROL-C026: 调用把关键字实参写作 `name: value`。位置实参 MUST 先绑定位置参数。关键字实参 MUST 在检测重复后按名称绑定关键字参数，不依赖源顺序。`*expr` 按集合规则从 iterable 或 Array 展开位置实参。`**expr` 按集合和类型章节指定的以 Symbol 或可接受关键字名称表示为键的 Hash 展开关键字实参。`&expr` 提供块通道。

IRIS-V1-CONTROL-C027: 下列参数表是规范性的：

| 类别 | 声明示例 | 调用示例 | 绑定结果 | 失败示例 |
| --- | --- | --- | --- | --- |
| Required positional | `value: String` | `write("x")` | 不可变`value` | 缺少实参引发`ArgumentError` |
| Optional positional | `limit: Integer = 10` | `take()` or `take(5)` | 不可变`limit` | 默认值失败中止调用 |
| Positional rest | `*items: String` | `join("a", "b")` | 新`Array<String>` | 元素 Contract 失败引发`TypeError` |
| Required keyword | `key path: String` | `open(path: "a.ir")` | 不可变`path` | 缺少`path:`引发 `ArgumentError` |
| Optional keyword | `key mode: Symbol = :read` | `open(path: "a", mode: :write)` | 不可变`mode` | 重复`mode:`引发 `ArgumentError` |
| Keyword rest | `**options: Object` | `build(debug: true)` | 新`Hash<Symbol,Object>` | 非键展开失败引发`TypeError` |
| Block binding | `&block: Block<(String) -> Nil>` | `each() { block body }` | 可调用块值 | 意外块引发`ArgumentError` |
| Optional block binding | `&block: Block<(String) -> Nil> = nil` | `each()` | 省略时为`nil` | 非可调用块值引发`TypeError` |

IRIS-V1-CONTROL-EX004: Informative example，所有参数类别：

```iris
fun render(
  title: String,
  count: Integer = 1,
  *items: String,
  key path: String,
  key mode: Symbol = :read,
  **options: Object,
  &block: Block<(String) -> Nil> = nil
) -> Nil {
  if block != nil { block.call(title) }
}

render("report", "a", "b", path: "out.txt", verbose: true) { |line: String| -> Nil; print(line) }
```

## Closure 捕获与尾随块

IRIS-V1-CONTROL-C028: Closure 按引用捕获词法绑定单元，而不是按值快照捕获。被捕获的不可变绑定只读。被捕获的可变绑定共享，所以外层作用域的写入和 Closure 内的写入观察同一个单元。后续嵌套遮蔽 MUST NOT 重定向已有 Closure 捕获。

IRIS-V1-CONTROL-C029: 在实例、Module、Class 对象或 main Method 中创建的 Closure 捕获当前接收者关系和接收者名称存储访问。Source、reflection、Dynamic、native 和 Host API MUST NOT 重新绑定该已捕获接收者。Closure 内的原始 `@name` 指向在拥有接收者的代码中创建 Closure 时捕获的接收者。

IRIS-V1-CONTROL-C030: 尾随 Closure 只通过专用 `&block` 调用通道传递。它不是最后一个位置实参。调用 MUST 至多提供一个块，要么通过实参列表中的 `&expr`，要么通过一个尾随 Closure。跟在已完成后缀表达式之后的尾随 Closure 绑定到该调用目标最终声明的块参数。如果没有兼容块参数，调用 MUST 引发 `ArgumentError`。

IRIS-V1-CONTROL-C031: Closure 调用 MUST 按与其他绑定可调用体相同的类别规则绑定参数。Closure 结果是其自己的 `return` 或最终表达式的值。Closure 内引发的异常按普通 ExceptionContext 规则通过调用点传播。如果 Closure 通过无效循环转移退出，运行时 MUST 报告控制目标错误，并且 MUST NOT 意外继续外层循环。

IRIS-V1-CONTROL-EX005: Informative example，按引用捕获和本地 Closure 返回：

```iris
mut total: Integer = 0
let add: Closure<(Integer) -> Integer> = { |value: Integer| -> Integer
  total += value
  return total
}

add(2)  // returns 2
add(3)  // returns 5
```

## 调用、属性与赋值

IRIS-V1-CONTROL-C032: 普通零实参和非零实参调用 MUST 使用括号。属性 getter 语法是唯一省略括号的普通类调用语法。具名中缀调用是在其语法优先级上的普通一实参 Method 发送。Contract 视图调用使用运行时章节定义的限定 `..` 命名空间。

IRIS-V1-CONTROL-C033: 调用求值顺序是接收者表达式、callee 或选择器解析输入、位置实参从左到右、关键字实参值按源顺序、splat 操作数按其源位置、块操作数或尾随 Closure 创建，然后调用。如果任何较早求值引发异常，后续实参求值和调用 MUST NOT 发生。选择器解析和元数或类型验证发生在该调用形状所需的接收者和调用操作数求值之后。

IRIS-V1-CONTROL-C034: 属性读取 `obj.name` MUST 发送 getter 选择器 `name`。属性赋值 `obj.name = value` MUST 发送 setter 选择器 `name=`，带一个实参。Setter 选择器可在语法允许的声明和属性赋值上下文中，在 `=` 前包含后缀标点，例如 `ready?=` 和 `value!=`。表达式 `a != b` 仍是不等式。

IRIS-V1-CONTROL-C035: 赋值是右结合的。绑定、Class 变量、全局和原始 ivar 赋值 MUST 产出实际存储的值。属性和索引赋值 MUST 产出 setter Method 结果。在 `a.x = b.y = v` 中，内层 setter 先执行，外层 setter 接收内层 setter 的实际结果。

IRIS-V1-CONTROL-C036: 复合赋值 `target OP= rhs` MUST 对目标位置和 `rhs` 各求值恰好一次，读取目标恰好一次，向读取值发送普通运算符 `OP`，并以 `rhs` 为实参，通过目标的正常写路径写入运算符结果恰好一次，并产出该写操作的结果。支持的符号复合赋值是 `+=`、`-=`、`*=`、`/=`、`**=`、`&=`、`|=`、`^=`、`<<=` 和 `>>=`。

IRIS-V1-CONTROL-C037: 逻辑赋值 `target &&= rhs` 和 `target ||= rhs` 是控制流赋值形式。目标位置 MUST 被求值并读取恰好一次。`&&=` MUST 只在当前目标值为 truthy 时求值并写入 `rhs`。`||=` MUST 只在当前目标值为 falsy 时求值并写入 `rhs`。无写路径产出当前值。写路径产出正常写回结果。

IRIS-V1-CONTROL-C038: 下列赋值表是规范性的：

| 形式 | 读取次数 | 写入次数 | 使用的派发 | 结果值 | 错误规则 |
| --- | --- | --- | --- | --- | --- |
| `x = rhs` | 绑定解析后无 | 一次 | 绑定写入 | 存储的值 | 不可变或缺失绑定失败 |
| `@x = rhs` | 无 | 一次 | 原始存储写入 | 存储的值 | 无身份或拒绝扩展引发`InstanceStateError` |
| `@@x = rhs` | 声明查找后无 | 一次 | Class 变量单元写入 | 存储的值 | 缺少声明失败 |
| `$x = rhs` | 声明查找后无 | 一次 | 全局单元写入 | 存储的值 | 缺少声明失败 |
| `obj.name = rhs` | 赋值本身不要求 getter | 一次 setter 调用 | `name=` Method | Setter 结果 | 缺少 setter 遵循普通派发失败 |
| `obj[key] = rhs` | 索引接收者和键一次 | 一次 setter 调用 | `[]=` Method | Setter 结果 | Setter 失败传播 |
| `target += rhs` | 一次 | 一次 | `+` 后正常写入 | 写回结果 | 运算符或写入失败阻止后续步骤 |
| `target &&= rhs` | 一个真值测试值 | 零次或一次 | `to_bool`，然后可选写入 | 当前值或写回结果 | `to_bool` 失败阻止 RHS 和写入 |

IRIS-V1-CONTROL-EX006: Informative example，赋值结果：

```iris
mut x: Integer = 1
x += 2                    // yields 3

object.status = :ready     // yields status= setter result
target &&= compute()       // compute runs only when target is truthy
```

## 真值性与条件表达式

IRIS-V1-CONTROL-C039: 条件使用 [03-runtime-object-model.md](03-runtime-object-model.md) 中的动态 `to_bool() -> Bool` 协议。`if`、`while`、语法允许位置的 match guard、`!`、`&&`、`||`、`&&=` 和 `||=` MUST 对每次测试的操作数求值一次，调用 `to_bool` 一次，要求实际 `Bool`，并按该 Bool 结果分支。`to_bool` 异常 MUST 原样传播。

IRIS-V1-CONTROL-C040: `!x` 返回 `x.to_bool()` 的 Bool 否定。`a && b` 测试 `a`；若为 false，则返回原始 `a`，不求值 `b`；若为 true，则求值并返回 `b`。`a || b` 测试 `a`；若为 true，则返回原始 `a`，不求值 `b`；若为 false，则求值并返回 `b`。由 `to_bool` 产生的 Bool 不是 `&&` 或 `||` 的值。

IRIS-V1-CONTROL-C041: `if condition { then } else if other { middle } else { last }` 会产出值。每个分支有自己的词法作用域。结果是被选分支的最终表达式，或对没有值的被选主体为 `nil`。缺少 `else` 会贡献一个隐式 `nil` 分支。结果类型是可达分支结果类型的规范化 union。

IRIS-V1-CONTROL-C042: 条件测试本身 MUST NOT 重新类型化被测试操作数。任意 `to_bool` 不是类型谓词。显式谓词如 `is`、nil 比较和后续类型规则可收窄流类型，但真值性协议本身让源绑定类型保持不变。

IRIS-V1-CONTROL-EX007: Informative example，条件值和返回操作数的逻辑运算符：

```iris
let chosen = if config.ready? { "ready" } else { nil }
let cached = value || compute_default()
let both = label && label.length()
```

## 循环、标签与 Iterator 降低

IRIS-V1-CONTROL-C043: `while condition { body }` MUST 在每次迭代前测试 `condition`，并使用 `to_bool`。自然完成，包括零次迭代，产出 `nil`。`break expr` 以 `expr` 作为循环结果退出目标循环。裸 `break` 以 `nil` 退出。`continue` 开始目标循环的下一次迭代且没有值。

IRIS-V1-CONTROL-C044: `for pattern in iterable { body }` MUST 对 `iterable` 求值一次，取得 `Iterator<T>`，通过规范 `Iterable<T>` 协议，重复调用 `next() -> Iteration<T>`，在 `Iteration.done` 时正常退出，并在每次迭代的新鲜逐迭代词法作用域中把每个 yield 值绑定到 `pattern`。自然完成产出 `nil`。`break` 像 `while` 一样提供循环结果。

IRIS-V1-CONTROL-C045: `for` 解构 MUST 使用由 match 和集合规则指定的绑定、Tuple、Array、rest 和 `_` 模式子集。解构不匹配 MUST 引发 `PatternMatchError`。Iterator MUST 在该错误传播前关闭。每次迭代创建新鲜绑定单元，所以逃逸 Closure 捕获该次迭代的值，而不是共享的最终循环变量。

IRIS-V1-CONTROL-C046: 语言遍历使用的 Iterators MUST 在每条退出路径上关闭：自然耗尽、`break`、以外层循环为目标的 `continue`、`return`、主体异常、Iterator 异常、模式错误和外层展开。自然耗尽通过 `Iteration.done` 释放。提前退出 MUST 在待定转移提交前调用幂等 `close()`。

IRIS-V1-CONTROL-C047: 如果遍历清理期间 Iterator `close()` 失败且存在 primary ExceptionContext，则关闭失败的 ExceptionContext MUST 按发生顺序追加到 primary context 的受保护 suppressed 列表。如果没有 primary exception，关闭失败成为 primary propagated exception，并阻止待定的正常、`break`、`continue` 或 `return` 转移完成。

IRIS-V1-CONTROL-C048: 当解析器接受 `outer: while condition { ... }` 或 `outer: for pattern in iterable { ... }` 等形式时，循环标签显式指向循环。无标签 `break` 和 `continue` 指向最近的包围循环。`break outer: expr` 和 `continue outer` 指向具名词法循环。不受限的无标签 `break expr` 仍然有效，并指向最近的包围循环。标签 MUST NOT 复制重叠作用域中另一个活动标签、标记非循环构造，或跨越 Closure 边界。有标签 break 值会贡献给目标循环的结果类型。

IRIS-V1-CONTROL-N001: Informative note：语法章节目前包含 `loop_label`、带标签 `break`、`continue` 和 `return` 产生式。本章拥有标签目标有效性、重复标签错误、Closure 边界限制、清理和结果类型化。

IRIS-V1-CONTROL-C049: 下列循环控制表是规范性的：

| 转移 | 有效目标 | 目标处结果 | 提交前清理 | 错误规则 |
| --- | --- | --- | --- | --- |
| `break` | 最近循环 | `nil` | 关闭转移跨过的活动遍历 iterators | 没有循环目标会引发控制目标错误 |
| `break expr` | 最近循环 | `expr` | 关闭转移跨过的活动遍历 iterators | 没有循环目标会引发控制目标错误 |
| `break label: expr` | 具名循环 | `expr` | 关闭转移跨过的活动遍历 iterators | 跨 Closure 边界的目标无效 |
| `continue` | 最近循环或具名循环 | 无值 | 关闭转移跨过的内层遍历 iterators | 不能指向已完成或不存在的循环 |
| Natural`while` completion | 循环自身 | `nil` | 除主体作用域外无 | 条件失败传播 |
| Natural`for` completion | 循环自身 | `nil` | Iterator 已 done 并释放 | Iterator 故障传播 |
| Pattern mismatch in`for` | 无正常目标 | 无循环结果 | 传播前关闭 Iterator | 引发`PatternMatchError` |

IRIS-V1-CONTROL-EX008: Informative example，`for` 降低和 Closure 捕获：

```iris
mut callbacks: Array<() -> Integer> = []
for value in 1 ..= 3 {
  callbacks.append({ || -> Integer; value })
}

callbacks[0]()  // returns 1
callbacks[1]()  // returns 2
```

## Match 表达式与模式

IRIS-V1-CONTROL-C050: `match value { arms }` MUST 对 scrutinee 求值一次，然后按源顺序测试 arms。第一个匹配 arm 执行并产生 match 结果。不存在 fallthrough。如果没有 arm 匹配，源 MUST 被拒绝，除非 arm 集可证明穷尽，或在语法允许处包含 `else`。Dynamic 或开放域要求显式 fallback arm。

IRIS-V1-CONTROL-C051: v1 模式词汇是字面量模式、`nil`、Bool 字面量、带绑定的名义类型测试、union alternatives、Tuple 解构、固定或 rest Array 解构、绑定模式和 `_`。V1 match MUST NOT 使用任意对象解构、Regex 匹配、用户定义模式协议或 fallthrough。模式绑定不可变，并限定在被选 arm 作用域内。

IRIS-V1-CONTROL-C052: 语法允许 guard 的位置，guard MUST 在模式结构成功后并在临时模式绑定存在后运行。Guard 条件 MUST 使用 `to_bool`。如果 guard 返回 false，临时绑定被丢弃，匹配继续到下一个 arm。如果 guard 引发异常，匹配停止且异常传播。

IRIS-V1-CONTROL-C053: Union alternatives MUST 绑定同一名称且合并类型兼容。`_` 丢弃且不创建绑定。仅绑定解构上下文中的解构不匹配 MUST 引发 `PatternMatchError`；普通 match arm 测试期间的不匹配意味着该 arm 不匹配，除非该模式形式已经通过 guard 或后续类型规则指定的绑定上下文提交。

IRIS-V1-CONTROL-N002: Informative note：语法章节目前包含 match guards、fallback arms、块或表达式主体，以及更丰富的 match patterns。本章拥有 match 选择、guard 真值性、绑定生命周期、穷尽性和模式失败行为。

IRIS-V1-CONTROL-C054: 下列 match 表是规范性的：

| 模式形式 | 匹配规则 | 绑定 | 失败行为 |
| --- | --- | --- | --- |
| Literal,`nil`, `true`, `false` | 按字面量的相等规则比较 | 无 | Arm 不匹配 |
| `is T` with binding | 运行时检查类型或 Contract 匹配 | 不可变收窄绑定 | 检查失败时 arm 不匹配 |
| Binding name | 总是匹配 | 到 scrutinee 或子值的不可变绑定 | 不会失败 |
| `_` | 总是接受 | 无 | 不会失败 |
| Tuple pattern | 元数和元素模式必须匹配 | 元素绑定 | 按上下文为 arm 不匹配或解构错误 |
| Array pattern | 长度、固定元素和 rest 规则必须匹配 | 元素和 rest 绑定 | 按上下文为 arm 不匹配或解构错误 |
| Union alternative | 该 arm 内第一个匹配 alternative 胜出 | 同一名称且合并类型兼容 | 如果所有 alternatives 失败则 arm 不匹配 |
| Guard | `to_bool` 必须返回 `Bool` | 仅 true 时保留临时绑定 | False 继续，异常传播 |

IRIS-V1-CONTROL-EX009: Informative example，match 结果：

```iris
let label = match value {
  nil => "none"
  true => "yes"
  _ => "other"
}
```

## Raise、Catch、Finally 与 ExceptionContext

IRIS-V1-CONTROL-C055: `try { ... }` MUST 后接至少一个 `catch` 子句或一个 `finally` 子句。零个或多个 catches 位于可选最终 `finally` 之前。执行顺序是 try 体、如有则选中的 first-match catch、如有则 finally，然后向外提交所得正常值、`return`、循环转移或异常。

IRIS-V1-CONTROL-C056: `raise value` 接受任何 Iris 对象或值，并创建一个新的传播事件，带有独立的、带身份的、运行时拥有的 `ExceptionContext`。被捕获对象是原始值且不改变。Stack、cause、suppressed cleanup failures、re-raise sites 和 source location 属于 ExceptionContext，不属于被 raise 的对象。

IRIS-V1-CONTROL-C057: `raise value from context` MUST 接受带身份的 `ExceptionContext` 作为显式 cause。`raise value from nil` MUST 抑制 cause chaining。如果在处理活动传播时发生 `raise value`，活动 ExceptionContext 成为自动 cause，除非显式 `from` 形式替换或抑制它。把普通 raised object 作为 `from` 传递 MUST 失败，因为它不标识传播事件。

IRIS-V1-CONTROL-C058: 裸 `raise` 在 catch 的同步动态范围内，以及该 catch 退出前所调用且仍在栈上的 helper 调用期间，继续当前正在处理的传播记录。它追加一个 re-raise site，且不创建新 context。在该动态范围外，包括稍后调用的存储 Closure，裸 `raise` MUST 引发 `NoActiveExceptionError`。

IRIS-V1-CONTROL-C059: 类型化 catch 匹配 MAY 命名名义 Class 或显式 Contract。Class 匹配接受该 Class 及其子类。Contract 匹配接受具有显式名义符合性的 Classes，并且 MUST NOT 使用结构性自动符合。Catch 子句按书写顺序测试，只有第一个匹配 catch 执行。静态不可达的后续 catches MUST 是编译期错误。动态匹配在选择开始时使用当前 Class 层级和 Contract 符合性。

IRIS-V1-CONTROL-C060: Catch 形式绑定不可变词法值。`catch error: Type {}` 绑定被 raise 对象并收窄为 `Type`。`catch error: Type, context {}` 同时绑定被 raise 对象及其 ExceptionContext。`catch error {}` 捕获每个被 raise 对象并将其绑定为 `Object`。`catch error, context {}` 捕获每个被 raise 对象并绑定对象加 context。`catch {}` 捕获所有但不绑定。`catch _, context {}` 和 `catch _: Type, context {}` 丢弃对象，只绑定 context。Catch-all MUST 位于最后。

IRIS-V1-CONTROL-C061: 正常 catch 完成会处理异常。其最终表达式成为临时 `try` 结果。原始传播不会继续，除非 catch 执行裸 `raise`。执行 `raise value`，即使 `value` 是被捕获对象，也会创建新的 ExceptionContext，并把活动 context 链接为 cause，除非显式 `from` 改变该关系。

IRIS-V1-CONTROL-C062: `try/catch/finally` 会产出值。正常 try 完成时，try 体最终表达式是临时结果。当 catch 正常处理时，该 catch 体最终表达式是临时结果。无值主体产出 `nil`。正常完成的 `finally` 为效果求值其主体，丢弃其普通最终值，并且 MUST NOT 替换临时结果。

IRIS-V1-CONTROL-C063: `finally` 总是在 try 和任何选中 catch 之后运行。`return`、`break` 或 `continue` 来自 `finally` 时会覆盖任何待定正常结果或异常。如果它丢弃待定 ExceptionContext，该被丢弃 context 只记录在受保护运行时诊断通道中，而不作为 cause 或 suppressed 元数据。普通调用者只观察新的控制转移。

IRIS-V1-CONTROL-C064: 如果 `finally` 在存在待定 exception context 时 raise 一个新值，新的传播成为 primary，待定 context 成为其 cause，除非显式 `from` 替换或抑制它。此情况下待定 context MUST NOT 放入 discarded diagnostics 或 suppressed cleanup failures。裸 `raise` 在 `finally` 中且存在待定传播时继续它；若不存在则引发 `NoActiveExceptionError`。

IRIS-V1-CONTROL-C065: `ExceptionContext` 暴露只读公共属性 `value: Object`、`cause: ExceptionContext?`、`suppressed: ReadonlyArray<ExceptionContext>`、`original_stack: ReadonlyArray<StackFrame>`、`re_raise_sites: ReadonlyArray<RaiseSite>` 和 `raise_location: SourceLocation`。公共 getter 可被动态替换以供普通属性读取，但运行时展开、诊断、native bridges 和 uncaught formatting MUST 使用受保护内部记录。

IRIS-V1-CONTROL-C066: Suppressed cleanup failures 是运行时拥有的有序 `ExceptionContext` 条目。用户代码 MUST NOT 插入、删除、替换、重排或伪造 suppressed 条目。Raised objects 自身 MUST NOT 因被 raise 而获得 `suppressed`、`cause`、stack 或 re-raise 元数据。运行时拥有的 cause 和 suppressed 边 MUST 保持无环；尝试形成环 MUST 引发 `ExceptionChainError` 并保持现有图不变。

IRIS-V1-CONTROL-C067: `ExceptionContext` 相等和哈希默认使用身份。不同传播事件不相等，即使它们携带相同 raised object、stack、cause 和 suppressed graph。保留的 ExceptionContext 仍可读取且有效，可作为未来显式 `from` cause，并强保留其可达传播图和被 raise 值。保留 context 不会让 catch 动态范围结束后的裸 `raise` 变有效。

IRIS-V1-CONTROL-C068: 下列异常和可调用返回控制表是规范性的：

| 控制事件 | 目标 | 结果值 | ExceptionContext 规则 | 错误规则 |
| --- | --- | --- | --- | --- |
| `return expr` | 当前可调用体 | `expr` | 无新 context；返回 Contract 检查前关闭跨过的遍历 iterators | 在可调用体外、跨帧边界或 Contract 失败时无效 |
| Bare`return` | 当前可调用体 | `nil` | 无新 context；返回 Contract 检查前关闭跨过的遍历 iterators | 在可调用体外、跨帧边界或 Contract 失败时无效 |
| `raise value` | 最近匹配 catch 或调用者 | 无，raise 路径上类型为`Never` | 新 context，如有则自动活动 cause | 可 raise 任何对象 |
| `raise value from context` | 最近匹配 catch 或调用者 | 无 | 带显式 cause 的新 context | 非 context cause 引发类型错误 |
| `raise value from nil` | 最近匹配 catch 或调用者 | 无 | 无 cause 的新 context | 除值求值外无 |
| Bare`raise` in active catch extent | 当前处理的传播 | 无 | 同一 context，追加 re-raise site | 无活动 context 引发`NoActiveExceptionError` |
| Normal catch completion | `try` 表达式 | Catch 最终表达式或`nil` | 活动异常成为已处理 | 无 |
| Normal finally completion | 待定外层目标 | 临时 try 或 catch 结果 | Finally 值被丢弃 | Finally 异常或转移覆盖 |
| `return` in finally | 当前可调用体 | 返回表达式或`nil` | 待定 context 记录为 discarded diagnostic | 仍检查可调用体返回 Contract |
| `break` in finally | 目标循环 | Break 表达式或`nil` | 待定 context 记录为 discarded diagnostic | 无效循环目标失败 |
| Exception in finally | 最近匹配 catch 或调用者 | 无 | 新 primary，待定异常成为 cause，除非`from`另有说明 | 应用无环性验证 |
| Iterator close failure during unwinding | 现有 primary propagation | 无 | 追加为 suppressed context | 没有 primary 时，close failure 成为 primary |

IRIS-V1-CONTROL-EX010: Informative example，对象和 context 不同：

```iris
try {
  raise :failed
} catch value: Symbol, context {
  context.value          // :failed
  context.suppressed     // runtime-owned ExceptionContext list
  raise value from context
}
```

IRIS-V1-CONTROL-EX011: Informative example，finally 值被丢弃：

```iris
let result = try {
  "try value"
} finally {
  "ignored final value"
}

result  // "try value"
```

## 控制转移覆盖与向量

IRIS-V1-CONTROL-C069: 本章中的每个控制转移在 IRIS-V1-CONTROL-C049 或 IRIS-V1-CONTROL-C068 中都有目标、结果和错误规则。如果未来章节添加控制转移，它 MUST 添加等价的目标、结果和错误规则，并交叉链接本章。

IRIS-V1-CONTROL-C070: 下列控制流向量表是规范性的。一致性章节 MUST 保留这些向量 ID，或把它们映射到具有相同可观察结果的机器可读记录：

| Vector ID | Category | Applicability | Source/Input | Expected observable | Decisions |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-CONTROL-V001` | positive | 需要 interpreter；需要 JIT；native 不适用 | 带初始化器和固定推断类型的 `let` | 后续兼容读取返回初始化器值。 | `D-426`, `D-427` |
| `IRIS-V1-CONTROL-V002` | diagnostic | 需要 compiler；JIT 不适用；native 不适用 | 没有初始化器的 `let x: Integer` | `BINDING_LET_REQUIRES_INITIALIZER`. | `D-426`, `D-427` |
| `IRIS-V1-CONTROL-V003` | diagnostic | 需要 compiler；JIT 不适用；native 不适用 | 无类型延迟 `mut x` | `BINDING_MISSING_TYPE_FOR_DEFERRED_INIT`. | `D-427` |
| `IRIS-V1-CONTROL-V004` | negative | 需要 interpreter；需要 JIT；native 不适用 | 赋值前读取类型化延迟 `mut` | `DefiniteAssignmentError`. | `D-427` |
| `IRIS-V1-CONTROL-V005` | negative | 需要 interpreter；需要 JIT；native 不适用 | 没有绑定的裸 `missing = 1` | `NameError` 或静态未解析绑定诊断；不创建绑定。 | `D-426` |
| `IRIS-V1-CONTROL-V006` | positive | 需要 interpreter；需要 JIT；native 不适用 | Closure 捕获可变绑定和当前接收者 | 后续 Closure 调用观察共享单元和接收者。 | `D-420` |
| `IRIS-V1-CONTROL-V007` | diagnostic | 需要 compiler；JIT 不适用；native 不适用 | Closure 在无预期可调用类型时省略返回注解 | `CALLABLE_MISSING_CLOSURE_RETURN_TYPE`. | `D-417` |
| `IRIS-V1-CONTROL-V008` | positive | 需要 interpreter；需要 JIT；native 不适用 | Method 带必需、可选、rest、keyword、keyword rest 和可选 block | 参数按声明绑定。 | `D-418` |
| `IRIS-V1-CONTROL-V009` | negative | 需要 interpreter；需要 JIT；native 不适用 | 重复关键字实参 | 选择器解析后 `ArgumentError`。 | `D-357` |
| `IRIS-V1-CONTROL-V010` | negative | 需要 interpreter；需要 JIT；native 不适用 | 意外尾随块 | `ArgumentError`. | `D-423` |
| `IRIS-V1-CONTROL-V011` | positive | 需要 interpreter；需要 JIT；native 不适用 | 属性赋值 setter 返回 marker | 赋值表达式产出 marker。 | `D-344` |
| `IRIS-V1-CONTROL-V012` | positive | 需要 interpreter；需要 JIT；native 不适用 | 复合赋值目标带有副作用接收者和索引 | 接收者、索引和 RHS 各求值一次。 | `D-347` |
| `IRIS-V1-CONTROL-V013` | positive | 需要 interpreter；需要 JIT；native 不适用 | `target &&= rhs` 且当前值 falsy | RHS 不求值，结果是当前值。 | `D-349` |
| `IRIS-V1-CONTROL-V014` | positive | 需要 interpreter；需要 JIT；native 不适用 | 当前值 falsy 的逻辑或赋值 | RHS 求值一次并返回写回结果。 | `D-349` |
| `IRIS-V1-CONTROL-V015` | negative | 需要 interpreter；需要 JIT；native 不适用 | `to_bool` 在 `if` 中返回非 Bool | `TypeContractError`；主体不运行。 | `D-350`, `D-352` |
| `IRIS-V1-CONTROL-V016` | positive | 需要 interpreter；需要 JIT；native 不适用 | `if` 没有 `else` 且条件 false | 结果是 `nil`。 | `D-437` |
| `IRIS-V1-CONTROL-V017` | positive | 需要 interpreter；需要 JIT；native 不适用 | `while` 自然完成 | 结果是 `nil`。 | `D-438` |
| `IRIS-V1-CONTROL-V018` | positive | 需要 interpreter；需要 JIT；native 不适用 | 从循环 `break 7` | 循环结果是 `Integer(7)`。 | `D-438` |
| `IRIS-V1-CONTROL-V019` | negative | 需要 interpreter；需要 JIT；native 不适用 | 循环外 `break` | 控制目标错误 `CONTROL_TRANSFER_WITHOUT_TARGET`（IRIS-V1-CONTROL-C077）。 | `D-438` |
| `IRIS-V1-CONTROL-V020` | positive | 需要 interpreter；需要 JIT；native 不适用 | `for` 遍历先 yield `Iteration.yield(nil)` 再 done 的 Iterator | 主体收到合法 `nil`，然后循环完成。 | `D-439` |
| `IRIS-V1-CONTROL-V021` | negative | 需要 interpreter；需要 JIT；native 不适用 | `for` 解构不匹配 | Iterator 关闭，然后 `PatternMatchError` 传播。 | `D-139`, `D-439` |
| `IRIS-V1-CONTROL-V022` | positive | 需要 interpreter；需要 JIT；native 不适用 | `for` 中的 Closure 捕获逐迭代绑定 | 逃逸 Closures 返回不同迭代值。 | `D-434` |
| `IRIS-V1-CONTROL-V023` | positive | 需要 interpreter；需要 JIT；native 不适用 | `break outer: 7` 到外层循环 | 外层循环结果是 break 表达式。 | `D-440` |
| `IRIS-V1-CONTROL-V024` | diagnostic | 需要 compiler；JIT 不适用；native 不适用 | 跨 Closure 边界的标签目标 | 控制目标诊断。 | `D-421`, `D-440` |
| `IRIS-V1-CONTROL-V025` | positive | 需要 interpreter；需要 JIT；native 不适用 | 带 fallback arm 的穷尽 match | 第一个匹配 arm 结果。 | `D-441` |
| `IRIS-V1-CONTROL-V026` | diagnostic | 需要 compiler；JIT 不适用；native 不适用 | 对开放类型的非穷尽 match | 编译期穷尽性诊断。 | `D-441` |
| `IRIS-V1-CONTROL-V027` | positive | 需要 interpreter；需要 JIT；native 不适用 | `raise :x` 被带 context 的 catch-all 捕获 | Catch 收到 `:x`；`context.value` 是 `:x`。 | `D-150`, `D-157` |
| `IRIS-V1-CONTROL-V028` | positive | 需要 interpreter；需要 JIT；native 不适用 | `raise value from nil` | Context cause 是 `nil`。 | `D-144`, `D-156` |
| `IRIS-V1-CONTROL-V029` | negative | 需要 interpreter；需要 JIT；native 不适用 | `raise value from value` 使用普通对象 cause | 非 ExceptionContext cause 的类型错误。 | `D-156` |
| `IRIS-V1-CONTROL-V030` | positive | 需要 interpreter；需要 JIT；native 不适用 | Catch 内裸 `raise` | 同一 context 继续并追加 re-raise site。 | `D-154`, `D-155` |
| `IRIS-V1-CONTROL-V031` | negative | 需要 interpreter；需要 JIT；native 不适用 | Catch 动态范围之后裸 `raise` | `NoActiveExceptionError`. | `D-154` |
| `IRIS-V1-CONTROL-V032` | positive | 需要 interpreter；需要 JIT；native 不适用 | 正常 catch 完成 | Try 表达式结果是 catch 最终表达式。 | `D-168`, `D-170` |
| `IRIS-V1-CONTROL-V033` | positive | 需要 interpreter；需要 JIT；native 不适用 | 正常 finally 最终表达式 | 值被丢弃，临时结果被保留。 | `D-168`, `D-169` |
| `IRIS-V1-CONTROL-V034` | positive | 需要 interpreter；需要 JIT；native 不适用 | 存在待定异常时 finally raise | 新 context 是 primary，旧 context 是 cause。 | `D-165` |
| `IRIS-V1-CONTROL-V035` | positive | 需要 interpreter；需要 JIT；native 不适用 | 存在待定异常时清理失败 | Cleanup context 出现在 primary `suppressed` 中。 | `D-140`, `D-160` |
| `IRIS-V1-CONTROL-V036` | negative | 需要 interpreter；需要 JIT；native 不适用 | ExceptionContext cause 环尝试 | `ExceptionChainError`；图不变。 | `D-161` |
| `IRIS-V1-CONTROL-V037` | positive | 需要 interpreter；需要 JIT；native 不适用 | 活动遍历清理后从当前可调用体 `return 3` | 当前可调用体在清理后返回 `Integer(3)`。 | `D-139`, `D-421` |
| `IRIS-V1-CONTROL-V038` | diagnostic | 需要 compiler；JIT 不适用；native 不适用 | 任何可调用体外的 `return` | 无效返回位置诊断 `CONTROL_RETURN_OUTSIDE_CALLABLE`（IRIS-V1-CONTROL-C077）。 | `D-421` |

IRIS-V1-CONTROL-C078：v1.15 勘误为那些冻结文本以散文描述拒绝但未命名的行命名稳定诊断码。本条款仅命名诊断码：它不改变任何拒绝条件，也不改动任何决策。在需要 v1 产生式的位置写出遗留的异常、循环或分支形式，必须诊断为 `PARSE_LEGACY_FORM`；这涵盖 IRIS-V1-CONTROL-V288A 与 V289A 的 `groan`、`rescue`、`ensure`、`throw` 形式，V355B 的 `repeat` 形式，以及 V357A 的 `switch`/`when` 形式。这些拼写在 D-509 下仍是普通标识符，因此本条款仅在 D-509 已允许的上下文特定位置拒绝它们。不带括号书写的普通调用必须诊断为 `PARSE_CALL_REQUIRES_PARENTHESES`，即 IRIS-V1-CONTROL-V342A 的对应码。`const` 声明重新绑定同一作用域中已声明的名称，必须诊断为 `DECLARATION_REBINDING`，即 V358 的对应码，且原声明保持绑定。子类重新声明锚定于祖先的类变量，必须诊断为 `CLASS_VARIABLE_REDECLARATION`，即 V348 的对应码，且祖先单元保持不变。会拓宽绑定固定局部类型的赋值，必须诊断为 `BINDING_FIXED_LOCAL_TYPE`，即 V345 中“固定局部类型诊断”的对应码。引用后声明参数的参数默认值，必须诊断为 `PARAMETER_DEFAULT_FORWARD_REFERENCE`，这是 IRIS-V1-CONTROL-V337A 要求的第二个码；其第一个码是 IRIS-V1-CONTROL-C077 的 `BINDING_ASSIGN_TO_IMMUTABLE`。向不存在的 `$name` 或 `@@name` 存储赋值，必须诊断为 `MISSING_DECLARED_STORAGE`，即 V347A 中缺失存储诊断的对应码，且不创建任何存储。

IRIS-V1-CONTROL-C079：v1.15 勘误定义三个记录 Type，它们是 IRIS-V1-CONTROL-C065 与 `D-473` 已要求 `ExceptionContext` 暴露、却从未规定其成员的类型。本条款仅定义 Type：它不改变任何传播、清理或链接行为。`SourceLocation` 是不可变、无标识的值，具有只读的 `path: String`、`line: Integer`、`column: Integer`，其中 `line` 与 `column` 从 1 开始计数。`StackFrame` 是不可变、无标识的值，具有只读的 `callable_name: Symbol` 与 `location: SourceLocation`。`RaiseSite` 是不可变、无标识的值，具有只读的 `location: SourceLocation`，记录一次裸 `raise` 在 `D-155` 下继续传播的位置。三者均按结构比较与哈希，这与 IRIS-V1-CONTROL-C067 保持基于标识的 `ExceptionContext` 不同：命名同一可调用体且位于同一位置的两个栈帧相等。`original_stack` 按原始 raise 处由内向外排序，`re_raise_sites` 保持 `D-155` 已要求的出现顺序。两个集合均不可由用户构造，这与 IRIS-V1-CONTROL-C066 禁止伪造传播元数据一致。

## 控制覆盖向量

IRIS-V1-CONTROL-C073: 下列向量是带有具体源输入和预期控制观察的规范性可追溯向量。

IRIS-V1-CONTROL-C074：在 Module 声明内，`module fun` 将 Method 安装到该 Module 对象自身。该声明内未修饰的 `fun` 仍是 Module instance Method，在 Module 被组合时提供给 host；它不同于顶层可执行 Module body 的 `fun`，后者按 IRIS-V1-CONTROL-C012 安装在该 Module 的 `main` 接收者上。`module` 修饰符与 `class` 互斥；这与 `class fun` 安装到 Class 对象及其在 IRIS-V1-RUNTIME-C043 和 IRIS-V1-RUNTIME-C044 中的单例查找表面相对应。

IRIS-V1-CONTROL-C075：当具名 Method 省略 `-> ReturnType` 时，其声明的静态和运行时返回 Contract 是 `Dynamic<Object>`，不受推断出的最终表达式或显式 return 的主体事实影响。实现 MAY 在本地使用这些主体事实进行诊断或优化，但 MUST NOT 将它们发布为 Method 签名元数据、用它们选择不同 Method 或 overload，或推断更窄的返回 Contract。若最终表达式的静态 Type 不已知，它在主体分析中是 `Dynamic<Object>`，且该 Method 声明的返回 Contract 仍是 `Dynamic<Object>`。

IRIS-V1-CONTROL-C076：v1.10 勘误使 `call` 成为 IRIS-V1-CONTROL-C021 所述普通可调用类别的唯一调用拼写。`closure.call(args...)`、`bound.call(args...)` 和 `block.call(args...)` 调用 Closure 或 BoundMethod，且可调用值不得通过直接对其应用实参列表来调用。本次修订取代了 IRIS-V1-MIG-004 先前指定为遗留 `cast.call(...)` 替代形式的直接应用拼写，并取代 `D-454` 中移除 `call` 作为调用面的那一部分。`call` 是可调用对象上的普通选择子，因此 Closure 与 BoundMethod 在 IRIS-V1-RUNTIME-C042 和 IRIS-V1-RUNTIME-C040 下仍是带标识的可调用对象，按 IRIS-V1-CONTROL-C022 至 IRIS-V1-CONTROL-C026 的参数规则接收实参，并通过普通派发应答 `call`。尾随 Closure 仍通过 IRIS-V1-GRAMMAR-C050 的专用 `&block` 通道绑定，省略的可选 block 仍按 IRIS-V1-CONTROL-C025 绑定 `nil`，因此 `block != nil` 仍是 `block.call(...)` 之前的存在性测试。

IRIS-V1-CONTROL-C077：v1.14 勘误为三行命名稳定诊断码，这些行的冻结文本以散文描述诊断但未命名。目标循环在所属可调用体内不存在的 `break` 或 `continue` 必须诊断为 `CONTROL_TRANSFER_WITHOUT_TARGET`，即 IRIS-V1-CONTROL-V019 中“控制目标错误”的对应码。出现在任何可调用体之外的 `return` 必须诊断为 `CONTROL_RETURN_OUTSIDE_CALLABLE`，即 IRIS-V1-CONTROL-V038 中“无效 return 位置诊断”的对应码。目标为不可变绑定的赋值必须诊断为 `BINDING_ASSIGN_TO_IMMUTABLE`，即 IRIS-V1-CONTROL-V040 中“静态不可变绑定诊断”的对应码。本条款仅命名诊断码：它不改变哪些源代码被拒绝，不改动 `D-421`、`D-426` 与 `D-438` 所述的条件，也不影响 IRIS-V1-CONTROL-V024——后者的诊断已由 IRIS-V1-CONTROL-V339A 在同一 `D-421` 下命名为 `CONTROL_TARGET_CROSSES_CLOSURE`。`CONTROL_TRANSFER_WITHOUT_TARGET` 与 `CONTROL_TARGET_CROSSES_CLOSURE` 保持相互区别：前者报告所属可调用体内不存在目标循环，后者报告目标存在但位于 Closure 调用边界之外。

| Vector ID | Category | Applicability | Source/Input | Expected observable | Decisions |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-CONTROL-V039` | positive | 需要 interpreter；需要 JIT；native 不适用 | `mut x: Integer = 1; x = 2; x` | `Integer(2)`；绑定可变且固定为 `Integer`。 | `D-426`, `D-427` |
| `IRIS-V1-CONTROL-V040` | diagnostic | 需要 compiler；JIT 不适用；native 不适用 | `let x: Integer = 1; x = 2` | 赋值处出现静态不可变绑定诊断 `BINDING_ASSIGN_TO_IMMUTABLE`（IRIS-V1-CONTROL-C077）；不发生写入。 | `D-426` |
| `IRIS-V1-CONTROL-V041` | diagnostic | 需要 compiler；JIT 不适用；native 不适用 | `mut x = 1; x = "s"` | 静态固定推断类型诊断；`x` 不拓宽。 | `D-427` |

## 可追溯性说明

IRIS-V1-CONTROL-C071: 本章拥有 IRIS-V1-CONTROL-C072 中列出的可调用和控制流决策。它本地引用 D-127 作为 `Iterator.next() -> Iteration<T>` 遍历边界，引用 D-138 作为 `close()` 清理表面，但 D-128 到 D-137 的 Iterator/Iteration 身份、相等、哈希、检查、比较、保留、释放和具体容器遍历细节委托给运行时和集合章节。D-353 是运行时拥有的真值性派发；本章只通过条件表达式直接观察其 fallback。D-443 是运行时拥有的构造失败和逃逸实例行为，没有章内向量。D-444、D-447 和 D-451 作为可见性、`super` 和 Class 对象调用锚点在本地引用，但其运行时对象模型所有权仍被委托。D-470、D-471 和 D-472 委托给 async/resources 章节。

IRIS-V1-CONTROL-C072: 本章拥有的决策 ID 是 `D-139`, `D-140`, `D-141`, `D-142`, `D-143`, `D-144`, `D-145`, `D-146`, `D-147`, `D-148`, `D-149`, `D-150`, `D-151`, `D-152`, `D-153`, `D-154`, `D-155`, `D-156`, `D-157`, `D-158`, `D-159`, `D-160`, `D-161`, `D-162`, `D-163`, `D-164`, `D-165`, `D-166`, `D-167`, `D-168`, `D-169`, `D-170`, `D-171`, `D-172`, `D-343`, `D-344`, `D-345`, `D-346`, `D-347`, `D-348`, `D-349`, `D-350`, `D-351`, `D-352`, `D-354`, `D-355`, `D-356`, `D-357`, `D-358`, `D-359`, `D-360`, `D-361`, `D-415`, `D-416`, `D-417`, `D-418`, `D-419`, `D-420`, `D-421`, `D-422`, `D-423`, `D-424`, `D-425`, `D-426`, `D-427`, `D-428`, `D-429`, `D-430`, `D-431`, `D-432`, `D-433`, `D-434`, `D-435`, `D-436`, `D-437`, `D-438`, `D-439`, `D-440`, `D-441`, `D-469`, and `D-473`.

IRIS-V1-CONTROL-N003: Informative note：Hash rehash、Hash 遍历、构造生命周期、存储属性初始化、元操作、Closeable helper、幂等资源关闭和取消决策可能影响调用到本章的示例，但其规范性所有权仍在 collections、runtime、meta 和 async/resource 章节。

## 审计精确一致性向量

这些行是规范性审计精确向量。每一行都是最小 fixture；用分号分隔的用例是具名向量中的独立 fixture 条目。

| Vector ID | Category | Applicability | Source/Input | Expected observable | Decisions |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-CONTROL-V282` | positive | 需要 interpreter；需要 JIT；native 不适用 | Fixture`ExitProbeIterator(exit)` 追加 `:close`，来自 `close()`；独立 Iris 用例离开 `for` 的方式是 `break`、带标签外层 `continue`、可调用体 `return`、主体 `raise :body`、iterator `next()` raise `:next`，以及包围展开 `raise :outer`。 | 每个用例恰好记录一次`:close`，发生在其待定转移或原始 `ExceptionContext` 被观察之前；自然 `Iteration.done` 释放遍历状态且没有重复 close 效果。 | `D-139` |
| `IRIS-V1-CONTROL-V283` | negative | 需要 interpreter；需要 JIT；native 不适用 | 独立 fixtures 使用`FailingCloseIterator.new(:close)`，此时待定退出为自然完成、`break 7`、外层 `continue` 或 `return 9`。 | 每个用例都传播 primary`ExceptionContext.value == :close`；没有待定正常值或转移提交。 | `D-140` |
| `IRIS-V1-CONTROL-V284` | positive | 需要 interpreter；需要 JIT；native 不适用 | 嵌套失败 iterators raise`:body`；外层 catch 把每个 suppressed context 映射到其 value。 | 清理发生顺序为 `[:inner, :outer]`；`context.value == :body`。 | `D-141` |
| `IRIS-V1-CONTROL-V285` | negative | 需要 interpreter；需要 JIT；native 不适用 | 从主体失败加清理失败捕获`context`；独立用例调用 `append`、`delete`、索引替换和 `reverse!` 于 `context.suppressed`。 | 每次 mutation 都引发`ReadonlyMutationError`；同一有序运行时拥有条目随后仍可见。 | `D-142` |
| `IRIS-V1-CONTROL-V286` | diagnostic | 需要 interpreter；需要 JIT；native 不适用 | 把公共`ExceptionContext.suppressed` getter 替换为返回 `[]`；然后 raise `:body`，同时 `FailingCloseIterator.close()` raise `:close`。 | 普通`context.suppressed == []`；uncaught diagnostic payload 仍包含受保护 suppressed context，其 `value == :close`。 | `D-143` |
| `IRIS-V1-CONTROL-V287` | positive | 需要 interpreter；需要 JIT；native 不适用 | `try { raise :first } catch value, first { try { raise :second } catch _, second { [second.value, second.cause.same?(first)] } }` | `[:second, true]`；处理期间普通 `raise value` 创建新的 context，自动 cause 为活动 context。 | `D-144` |
| `IRIS-V1-CONTROL-V288` | positive | 需要 interpreter；需要 JIT；native 不适用 | 独立源 fixtures 在 catches 中执行`raise :x`、`raise :x from context` 和 `raise :x from nil`。 | 捕获值是`:x`；causes 分别是自动、显式和 `nil`。 | `D-145` |
| `IRIS-V1-CONTROL-V288A` | diagnostic | 需要 compiler；JIT 不适用；native 不适用 | `groan :x` | 旧 throw 词汇在执行前被解析拒绝。 `PARSE_LEGACY_FORM`（IRIS-V1-CONTROL-C078） | `D-145` |
| `IRIS-V1-CONTROL-V289` | positive | 需要 interpreter；需要 JIT；native 不适用 | `try { raise :x } catch error: Symbol { error } finally { nil }` | 结果是`:x`；类型化 catch 在 `finally` 前执行。 | `D-146` |
| `IRIS-V1-CONTROL-V289A` | diagnostic | 需要 compiler；JIT 不适用；native 不适用 | 解析旧 `groan`、`rescue`、`ensure` 和 `throw` 异常形式。 | 每个旧形式都在执行前被解析拒绝。 `PARSE_LEGACY_FORM`（IRIS-V1-CONTROL-C078） | `D-146` |
| `IRIS-V1-CONTROL-V290` | positive | 需要 interpreter；需要 JIT；native 不适用 | 把`Child.new()` raise 到 `catch error: Parent`；把 `Nominal.new()` raise 到 `catch error: C`；把 `StructuralOnly.new()` raise 到 `catch error: C` 后接 catch-all，其中只有 `Nominal` 显式符合 `C`。 | 结果`["parent", "contract", "fallback"]`；Class catches 接受子类，Contract catches 要求名义符合。 | `D-147` |
| `IRIS-V1-CONTROL-V291` | positive | 需要 interpreter；需要 JIT；native 不适用 | `try { raise Child.new() } catch _: Parent { :parent } catch _ { :fallback }` | `:parent`；选择第一个匹配 catch。 | `D-148` |
| `IRIS-V1-CONTROL-V292` | positive | 需要 interpreter；需要 JIT；native 不适用 | 提交有效 revision，使`CurrentChild` 继承 `Parent`；然后把 `CurrentChild.new()` raise 到 `catch _: Parent`。 | 类型化 catch 在提交后执行；提交前未匹配 fixture 仍未匹配。 | `D-149` |
| `IRIS-V1-CONTROL-V293` | positive | 需要 interpreter；需要 JIT；native 不适用 | `try { raise :x } catch _ { :handled }` | `:handled`；catch-all 接受 raised value 且不绑定。 | `D-150` |
| `IRIS-V1-CONTROL-V294` | diagnostic | 需要 compiler；JIT 不适用；native 不适用 | `let error = :outer; try { raise :x } catch error: Symbol, context { let read = [error, context.value]; error = :y; context = nil }; [error, context]` | 两次赋值处发生不可变绑定诊断；catch 局部读取有效，外层`error` 保持 `:outer`，最终 `context` 读取是未解析绑定诊断。 | `D-151` |
| `IRIS-V1-CONTROL-V295` | positive | 需要 interpreter；需要 JIT；native 不适用 | 独立 fixtures`raise nil`、`raise 1`、`raise :tag` 和 `raise Object.new()` 各自捕获 `value, context` 并比较 `context.value == value`。 | 四个`true` 结果；每个原始 raised value 原样交付。 | `D-152` |
| `IRIS-V1-CONTROL-V296` | positive | 需要 interpreter；需要 JIT；native 不适用 | 存储`value = Object.new()`；从两次独立 `raise value` 执行捕获 contexts。 | 两个 contexts 都有`value.same?(value) == true` 且 `first.same?(second) == false`。 | `D-153` |
| `IRIS-V1-CONTROL-V297` | negative | 需要 interpreter；需要 JIT；native 不适用 | 存储一个零实参 Closure，它在 catch 期间包含裸`raise`，然后在 catch 返回后调用它。 | `NoActiveExceptionError`. | `D-154` |
| `IRIS-V1-CONTROL-V298` | positive | 需要 interpreter；需要 JIT；native 不适用 | `try { raise :x } catch _, context { raise }`，在外部作为 `continued` 捕获。 | `continued.same?(context) == true` 且其 `re_raise_sites.length == 1`。 | `D-155` |
| `IRIS-V1-CONTROL-V299` | positive | 需要 interpreter；需要 JIT；native 不适用 | 捕获`first`，来自 `raise :first`；执行 `raise :second from first` 并捕获 `second`。 | `second.cause.same?(first) == true`。 | `D-156` |
| `IRIS-V1-CONTROL-V300` | positive | 需要 interpreter；需要 JIT；native 不适用 | `try { raise :x } catch error: Symbol, context { [error, context.value, context.class_name] }` | `[:x, :x, "ExceptionContext"]`. | `D-157` |
| `IRIS-V1-CONTROL-V301` | diagnostic | 需要 compiler；JIT 不适用；native 不适用 | `try { raise :x } catch _, context { _ }` | `DISCARD_BINDING_READ` 诊断位于 `_`；`context` 仍是有效 `ExceptionContext` 绑定。 | `D-158` |
| `IRIS-V1-CONTROL-V302` | positive | 需要 interpreter；需要 JIT；native 不适用 | 在`raise :body`、裸 re-raise 和一次失败清理之后捕获 context；读取 `value`、`cause`、`suppressed`、`original_stack`、`re_raise_sites` 和 `raise_location`。 | 类型是`Object`、`ExceptionContext?`、`ReadonlyArray<ExceptionContext>`、`ReadonlyArray<StackFrame>`、`ReadonlyArray<RaiseSite>` 和 `SourceLocation`。 | `D-159` |
| `IRIS-V1-CONTROL-V302A` | negative | 需要 interpreter；需要 JIT；native 不适用 | 捕获`context`，然后赋值 `context.value = :other`。 | `ReadonlyPropertyError`；受保护和公共 value 仍为原始 raised object。 | `D-159` |
| `IRIS-V1-CONTROL-V303` | positive | 需要 interpreter；需要 JIT；native 不适用 | Raise`:body`，同时 `FailingCloseIterator.close()` raises `:close`；catch `_, context` 并检查 `context.suppressed[0]`。 | 条目类型为`ExceptionContext`、`value == :close`，有自己的 stack，且身份不同于 primary context。 | `D-160` |
| `IRIS-V1-CONTROL-V304` | negative | 需要 interpreter；需要 JIT；native 不适用 | 捕获 contexts`first` 和 `second`，且 `second.cause == first`，然后尝试支持的 context-link 操作，将 `first.cause` 设为 `second`。 | `ExceptionChainError`；`first.cause` 仍是 `nil` 且 `second.cause.same?(first)` 仍为 true。 | `D-161` |
| `IRIS-V1-CONTROL-V305` | positive | 需要 interpreter；需要 JIT；native 不适用 | 捕获`first` 和 `second`，来自两次独立 `raise :same` 执行；把二者用作 Hash 键。 | `first == second` 为 false，`first.same?(second)` 为 false，且 Hash 有两个条目。 | `D-162` |
| `IRIS-V1-CONTROL-V306` | positive | 需要 interpreter；需要 JIT；native 不适用 | 在 catch 中存储`saved`，退出后读取 `saved.value`，然后 `raise :next from saved`；另行求值裸 `raise`。 | 显式 cause raise 成功且`cause.same?(saved) == true`；后续裸 raise 产生 `NoActiveExceptionError`。 | `D-163` |
| `IRIS-V1-CONTROL-V307` | diagnostic | 需要 interpreter；需要 JIT；native 不适用 | 三个 fixtures raise`:pending` 于 `try`，并使用 `return :returned`、`break :broken` 或 `continue`，来自 `finally`。 | 结果是 finally 转移；受保护诊断 payload 包含 discarded`:pending`，它不是结果的 cause 或 suppressed 条目。 | `D-164` |
| `IRIS-V1-CONTROL-V308` | positive | 需要 interpreter；需要 JIT；native 不适用 | `try { raise :old } finally { raise :new }`，在外部捕获为 `value, context`。 | `value == :new`、`context.value == :new` 且 `context.cause.value == :old`；旧 context 不是 suppressed。 | `D-165` |
| `IRIS-V1-CONTROL-V309` | negative | 需要 interpreter；需要 JIT；native 不适用 | `try { raise :pending } finally { raise }`；另行 `try { 1 } finally { raise }`。 | 第一个重新传播原始 context 身份；第二个引发`NoActiveExceptionError`。 | `D-166` |
| `IRIS-V1-CONTROL-V310` | positive | 需要 interpreter；需要 JIT；native 不适用 | `mut log: Array<Symbol> = []; try { log.append(:try); raise :x } catch _ { log.append(:catch); :handled } finally { log.append(:finally) }; log` | `[:try, :catch, :finally]`；try 表达式结果是 `:handled`。 | `D-167` |
| `IRIS-V1-CONTROL-V311` | positive | 需要 interpreter；需要 JIT；native 不适用 | `let a = try { 1 } finally { 2 }; let b = try { raise :x } catch _ { 3 } finally { 4 }; [a, b]` | `[1, 3]`；try 或选中 catch 提供临时值，正常完成的 `finally` 保留它。 | `D-168` |
| `IRIS-V1-CONTROL-V312` | positive | 需要 interpreter；需要 JIT；native 不适用 | `let result = try { :try_value } finally { :finally_value }; result` | `:try_value`；正常完成 `finally` 的最终表达式被求值并丢弃。 | `D-169` |
| `IRIS-V1-CONTROL-V313` | positive | 需要 interpreter；需要 JIT；native 不适用 | `let result = try { raise :x } catch _: Symbol { :caught }; result` | `:caught`；匹配 catch 处理传播，其最终表达式是 try 表达式值。 | `D-170` |
| `IRIS-V1-CONTROL-V314` | positive | 需要 interpreter；需要 JIT；native 不适用 | `try { raise :same } catch value, first { try { raise value } catch _, second { [second.same?(first), second.cause.same?(first)] } }` | `[false, true]`；显式重新 raise 被捕获值会创建新 context 并链接到已处理 context。 | `D-171` |
| `IRIS-V1-CONTROL-V315` | diagnostic | 需要 compiler；JIT 不适用；native 不适用 | `try { raise Child.new() } catch error: Parent { error = Child.new() }` | `error` 在 catch 中具有静态类型 `Parent`，且赋值产生不可变绑定诊断。 | `D-172` |
| `IRIS-V1-CONTROL-V318` | positive | 需要 interpreter；需要 JIT；native 不适用 | `class Flag { fun ready?() -> Bool { true }; fun save!() -> Symbol { :saved } }; [Flag.new().ready?(), Flag.new().save!()]` | `[true, :saved]`；reflection 报告不同选择器 `ready?` 和 `save!`。 | `D-342` |
| `IRIS-V1-CONTROL-V319` | positive | 需要 interpreter；需要 JIT；native 不适用 | `class Box { property fun name() -> String { "read" }; property fun name=(value: String) -> Symbol { :written } }; let b = Box.new(); [b.name, b.name = "x"]` | `["read", :written]`；派发使用选择器 `name` 和 `name=`。 | `D-343` |
| `IRIS-V1-CONTROL-V320` | positive | 需要 interpreter；需要 JIT；native 不适用 | `Box.new().name = "x"`，其中 `name=` 返回 `:written`。 | `:written`，setter Method 结果。 | `D-344` |
| `IRIS-V1-CONTROL-V321` | positive | 需要 interpreter；需要 JIT；native 不适用 | `outer.name = (inner.name = "x")`，其中 inner setter 返回 `:inner`。 | Outer setter 接收`:inner`；它在 inner setter 之后执行。 | `D-345` |
| `IRIS-V1-CONTROL-V322` | positive | 需要 interpreter；需要 JIT；native 不适用 | `mut local: Integer = 0; let local_result = (local = 1); let property_result = (Box.new().name = "x"); [local_result, property_result]` | `[1, :written]`；绑定写返回存储值，属性写返回 setter 结果。 | `D-346` |
| `IRIS-V1-CONTROL-V323` | positive | 需要 interpreter；需要 JIT；native 不适用 | `factory().items[index()] += rhs()`，其中每个 probe 把自己的名称追加到 `events`。 | `events == [:factory, :index, :rhs]`，一次读取、一次 `+`、一次写入。 | `D-347` |
| `IRIS-V1-CONTROL-V324` | diagnostic | 需要 compiler；JIT 不适用；native 不适用 | 解析使用 IRIS-V1-CONTROL-C036 列出的十个运算符的独立赋值，然后解析`x %= 2`。 | 恰好列出的形式解析；`x %= 2` 产生 `PARSE_UNSUPPORTED_COMPOUND_ASSIGNMENT`。 | `D-348` |
| `IRIS-V1-CONTROL-V325` | positive | 需要 interpreter；需要 JIT；native 不适用 | 可变 nilable`x` 初始为 `nil`；用 RHS `"set"` 应用逻辑或赋值；读取 `x`。 | `"set"`；RHS 求值一次并返回写回结果。 | `D-349` |
| `IRIS-V1-CONTROL-V326` | positive | 需要 interpreter；需要 JIT；native 不适用 | `if Probe.new() { :yes }`，其中 `Probe.to_bool()` 记录一次调用并返回 `true`。 | `:yes` 且一次 `to_bool` 调用。 | `D-350` |
| `IRIS-V1-CONTROL-V327` | positive | 需要 interpreter；需要 JIT；native 不适用 | probe 对象计数`to_bool`；独立用例在尝试安装同 token Method 后求值逻辑取反、合取、析取、逻辑与赋值和逻辑或赋值。 | 每个被测试 LHS 调用`to_bool()` 一次，RHS 遵循短路，且未调用任何尝试安装的 token Method。 | `D-351` |
| `IRIS-V1-CONTROL-V328` | negative | 需要 interpreter；需要 JIT；native 不适用 | `if Explosive.new() { :body }`，其中 `to_bool()` raise `:sentinel`。 | 传播的 context value 为`:sentinel`；主体不执行。 | `D-352` |
| `IRIS-V1-CONTROL-V329` | negative | 需要 interpreter；需要 JIT；native 不适用 | `class C { fun one(x: Integer) -> Nil { nil } }; C.new().one()` | 选择器解析后 `ArgumentError`；不调用 `method_missing`。 | `D-357` |
| `IRIS-V1-CONTROL-V330` | positive | 需要 interpreter；需要 JIT；native 不适用 | Nilable`left` 是 `nil`；求值 `left or "fallback"` 并做静态 Type reflection。 | 值`"fallback"`；规范化操作数 union Type，而不是 `Bool`。 | `D-359` |
| `IRIS-V1-CONTROL-V331` | positive | 需要 interpreter；需要 JIT；native 不适用 | Nilable`value` 在 `if` 中测试；然后检查其静态 Type。 | Type 保持原始 nilable union；真值性不收窄它。 | `D-360` |
| `IRIS-V1-CONTROL-V332` | positive | 需要 interpreter；需要 JIT；native 不适用 | 对 nilable 可变绑定应用逻辑取反和逻辑或赋值；检查结果 Types。 | 取反 Type 是`Bool`；赋值结果 Type 是原始 nilable union。 | `D-361` |
| `IRIS-V1-CONTROL-V333` | positive | 需要 interpreter；需要 JIT；native 不适用 | Fixture 声明一个 Method，从实例读取它，求值一个带注解 Closure，并反射 callable kinds。 | 观察到的 kinds 是`Method`、`BoundMethod` 和 `Closure`；`Function` 查找不存在。 | `D-415` |
| `IRIS-V1-CONTROL-V334` | negative | 需要 interpreter；需要 JIT；native 不适用 | Module`M` 定义顶层 `fun hidden() -> Integer { 1 }` 并在内部调用 `hidden()`；导入者调用 `M.hidden()`。 | 内部调用返回`1`；导入者收到私有成员派发失败。 | `D-416` |
| `IRIS-V1-CONTROL-V335` | positive | 需要 interpreter；需要 JIT；native 不适用 | 声明并调用一个规范一实参 Method 和一个规范带注解一实参 Closure。 | 结果是`[1, 2]`；两种声明形式都执行。 | `D-417` |
| `IRIS-V1-CONTROL-V335A` | diagnostic | 需要 compiler；JIT 不适用；native 不适用 | 独立畸形 fixtures 省略 Method 参数闭合和 Closure 头部分隔符。 | 每个 fixture 都在解析期间被拒绝且不发布 callable。 | `D-417` |
| `IRIS-V1-CONTROL-V336` | positive | 需要 interpreter；需要 JIT；native 不适用 | `fun f(required: Integer, optional: Integer = 2, *rest: Integer, key named: Integer, **options: Object, &block: Block<() -> Nil> = nil) -> Integer { required }; f(1, 3, 4, named: 5)` | `Integer(1)`；参数类别按声明顺序绑定。 | `D-418` |
| `IRIS-V1-CONTROL-V337` | positive | 需要 interpreter；需要 JIT；native 不适用 | 调用`pair(a: Integer, b: Integer = a + 1)` 为 `pair(2)` 和 `pair(4)`。 | 结果是`[[2, 3], [4, 5]]`；默认值每次调用在更早参数绑定后求值。 | `D-419` |
| `IRIS-V1-CONTROL-V337A` | diagnostic | 需要 compiler；JIT 不适用；native 不适用 | 独立 fixtures 对参数`b` 赋值，并定义默认值 `a = later`，位于 `later` 之前。 | 不可变参数和后续参数引用静态诊断在调用前发生。 | `D-419` |
| `IRIS-V1-CONTROL-V338` | positive | 需要 interpreter；需要 JIT；native 不适用 | Closure 两次递增并读取外层可变整数。 | `[2, 5]`；Closure 捕获共享可变绑定单元。 | `D-420` |
| `IRIS-V1-CONTROL-V339` | positive | 需要 interpreter；需要 JIT；native 不适用 | Method 调用一个主体执行`return 4` 的 Closure，然后 Method 返回 `5`。 | 观察值是`[4, 5]`；Closure return 不会从 Method 返回。 | `D-421` |
| `IRIS-V1-CONTROL-V339A` | diagnostic | 需要 compiler；JIT 不适用；native 不适用 | 外层循环内的独立 Closures 包含`break` 和 `continue`。 | 每个都在任何外层转移前产生`CONTROL_TARGET_CROSSES_CLOSURE`。 | `D-421` |
| `IRIS-V1-CONTROL-V340` | positive | 需要 interpreter；需要 JIT；native 不适用 | 调用带整数最终表达式的 Method 和 Closure，然后调用值为空主体的 Method 和 Closure。 | 结果是`[7, 8, nil, nil]`；返回最终表达式，值为空主体返回 `nil`。 | `D-422` |
| `IRIS-V1-CONTROL-V341` | negative | 需要 interpreter；需要 JIT；native 不适用 | 对接受 block 的 Method 分别用`&saved`、尾随 Closure 和两个通道同时调用。 | 前两次调用通过专用 block 通道接收 Closure；双通道调用在调用前引发`ArgumentError`。 | `D-423` |
| `IRIS-V1-CONTROL-V342` | positive | 需要 interpreter；需要 JIT；native 不适用 | 求值`f()`、`obj.m()`、裸 callable 值 `f`、属性 getter 和具名中缀发送。 | 带括号调用执行，裸`f` 不被调用，且两个文档化的无括号例外执行。 | `D-424` |
| `IRIS-V1-CONTROL-V342A` | diagnostic | 需要 compiler；JIT 不适用；native 不适用 | 独立普通调用 fixtures 使用`f 1` 和 `obj.m 1`。 | 二者都在调用前被解析拒绝，因为普通调用需要括号。 `PARSE_CALL_REQUIRES_PARENTHESES`（IRIS-V1-CONTROL-C078） | `D-424` |
| `IRIS-V1-CONTROL-V343` | positive | 需要 interpreter；需要 JIT；native 不适用 | 把 BoundMethod 和带注解 Closure 赋给同一一实参 callable Type 并调用二者。 | `[1, 2]`；两个值都满足同一 callable Type。 | `D-425` |
| `IRIS-V1-CONTROL-V344` | positive | 需要 interpreter；需要 JIT；native 不适用 | `let fixed = 1; mut changed = 1; changed = 2; [fixed, changed]` | `[1, 2]`；不可变和可变声明具有其指定行为。 | `D-426` |
| `IRIS-V1-CONTROL-V344A` | diagnostic | 需要 compiler；JIT 不适用；native 不适用 | 独立 fixtures 使用`let missing: Integer`、给 `let` 赋值，以及赋值 `undeclared = 1`。 | 缺少初始化器、不可变绑定和未解析绑定诊断发生；无效写入不创建存储。 | `D-426` |
| `IRIS-V1-CONTROL-V345` | negative | 需要 compiler；JIT 不适用；native 不适用 | `mut value = 1; value = "text"` | 固定局部 Type 诊断；不发生拓宽。 `BINDING_FIXED_LOCAL_TYPE`（IRIS-V1-CONTROL-C078） | `D-427` |
| `IRIS-V1-CONTROL-V346` | positive | 需要 interpreter；需要 JIT；native 不适用 | Closure 捕获外层`value`；嵌套块在使用外层初始化器后遮蔽它。 | 内层块返回`2`；closure 返回原始 `1`。 | `D-428` |
| `IRIS-V1-CONTROL-V347` | positive | 需要 interpreter；需要 JIT；native 不适用 | 在授权 Method 中，赋值`@dynamic = 1`，然后读取它。 | 值是`1`；原始当前接收者 ivar 存储按其普通能力规则创建。 | `D-429` |
| `IRIS-V1-CONTROL-V347A` | diagnostic | 需要 compiler；JIT 不适用；native 不适用 | 独立 fixtures 对未声明的`$missing = 1`、`@@missing = 1` 和 `local_missing = 1` 赋值。 | 缺少存储和未解析绑定诊断发生；不创建存储。 | `D-429` |
| `IRIS-V1-CONTROL-V348` | negative | 需要 compiler；JIT 不适用；native 不适用 | `class Parent { class mut @@x: Integer = 1 }; class Child extends Parent { class mut @@x: Integer = 2 }` | Class 变量重声明诊断；祖先单元保持不变。 `CLASS_VARIABLE_REDECLARATION`（IRIS-V1-CONTROL-C078） | `D-430` |
| `IRIS-V1-CONTROL-V349` | positive | 需要 interpreter；需要 JIT；native 不适用 | `class Parent { class mut @@x: Integer = 1; fun read() -> Integer { @@x }; class fun read_class() -> Integer { @@x } }; class Child extends Parent {}; [Child.new().read(), Child.read_class()]` | `[1, 1]`；实例和 Class Methods 使用声明词法 Class 层级。 | `D-436` |
| `IRIS-V1-CONTROL-V350` | negative | 需要 interpreter；需要 JIT；native 不适用 | Packages`a` 和 `b` 各自声明 private `global mut $count: Integer`；修改 `a::$count`，通过授权包代码读取二者，然后访问未声明 `$missing`。 | 值按包区分且为运行时局部；不存在访问引发`NameError` 且不创建存储。 | `D-431` |
| `IRIS-V1-CONTROL-V351` | positive | 需要 interpreter；需要 JIT；native 不适用 | Fixture 把相同非限定名称给到词法绑定、可见 Module 常量和显式导入，然后在嵌套作用域读取它。 | 解析先选择词法作用域，然后当前 Module 或包声明，然后显式导入。 | `D-432` |
| `IRIS-V1-CONTROL-V351A` | diagnostic | 需要 compiler；JIT 不适用；native 不适用 | 在一个 Module 中声明`const Name = 1` 然后 `class Name {}`。 | `QUALIFIED_NAMESPACE_COLLISION`；没有第二个声明发布。 | `D-432` |
| `IRIS-V1-CONTROL-V352` | positive | 需要 interpreter；需要 JIT；native 不适用 | Module 体定义`fun helper() -> Integer { 1 }`，求值 `helper()`，然后求值未解析裸 `helper_missing`。 | `helper()` 返回 `1`，通过隐式 `main` 发送；裸查找引发 `NameError`。 | `D-433` |
| `IRIS-V1-CONTROL-V353` | positive | 需要 interpreter；需要 JIT；native 不适用 | 对两个循环值各存储一个逃逸 Closure，然后调用二者。 | `[1, 2]`；每次迭代拥有新鲜捕获绑定单元。 | `D-434` |
| `IRIS-V1-CONTROL-V354` | positive | 需要 interpreter；需要 JIT；native 不适用 | 求值被选`if` 分支，带局部 `inside`，然后求值一个 `if false`，没有 `else`。 | 结果是`[1, nil]`；被选分支最终表达式提供第一个值。 | `D-437` |
| `IRIS-V1-CONTROL-V354A` | negative | 需要 interpreter；需要 JIT；native 不适用 | 读取分支局部`inside`，在其 `if` 表达式完成后。 | `NameError`；分支绑定在外部不可见。 | `D-437` |
| `IRIS-V1-CONTROL-V355` | positive | 需要 interpreter；需要 JIT；native 不适用 | 求值`while false` 和 `while true { break 7 }`。 | 循环值是`[nil, 7]`。 | `D-438` |
| `IRIS-V1-CONTROL-V355B` | diagnostic | 需要 compiler；JIT 不适用；native 不适用 | `repeat { nil }` | 旧循环形式在执行前被解析拒绝。 `PARSE_LEGACY_FORM`（IRIS-V1-CONTROL-C078） | `D-438` |
| `IRIS-V1-CONTROL-V355A` | positive | 需要 interpreter；需要 JIT；native 不适用 | 脚本 iterable 返回`Iteration.yield(nil)` 然后 `Iteration.done`；`for` 追加每个绑定值，同时 probes 计数 `iterator()` 和 `next()`。 | 看到的值是`[nil]`；`iterator()` 运行一次，`next()` 运行两次，done 正常终止而不是把 yield 的 `nil` 当作完成。 | `D-439` |
| `IRIS-V1-CONTROL-V356` | positive | 需要 interpreter；需要 JIT；native 不适用 | `outer: while true { while true { break outer: 7 } }` | 来自具名外层循环的 `Integer(7)`。 | `D-440` |
| `IRIS-V1-CONTROL-V357` | positive | 需要 interpreter；需要 JIT；native 不适用 | `match 1 { 1 => :one, else => :other }` | 结果是`:one`；fallback 使 match 穷尽。 | `D-441` |
| `IRIS-V1-CONTROL-V357A` | diagnostic | 需要 compiler；JIT 不适用；native 不适用 | `switch value { when 1 { :one } }` | 旧`switch`/`when` 在解析期间被拒绝且没有 arm 执行。 `PARSE_LEGACY_FORM`（IRIS-V1-CONTROL-C078） | `D-441` |
| `IRIS-V1-CONTROL-V358` | negative | 需要 compiler；JIT 不适用；native 不适用 | `const Name = 1; const Name = 2`. | 声明重绑定诊断；原始声明保持绑定。 `DECLARATION_REBINDING`（IRIS-V1-CONTROL-C078） | `D-449` |
| `IRIS-V1-CONTROL-V359` | diagnostic | 需要 compiler；JIT 不适用；native 不适用 | `defer { cleanup() }` | `PARSE_UNSUPPORTED_DEFER`；不创建或运行 cleanup Closure。 | `D-468` |
| `IRIS-V1-CONTROL-V360` | positive | 需要 interpreter；需要 JIT；native 不适用 | `try { raise :x } catch value: Symbol, context { raise :y from context } finally { nil }` | 向外 context 有`value == :y`，且其 cause 的 `value == :x`。 | `D-469` |
| `IRIS-V1-CONTROL-V361` | diagnostic | 需要 interpreter；需要 JIT；native 不适用 | Raise`:body`，同时 cleanup raises `:close`；将捕获的 context 保留到 catch 之外，并提交给结构化诊断 sink fixture。 | Payload 报告不可变`value == :body`、一个 suppressed `ExceptionContext.value == :close`、原始 stack、raise location 和稳定保留 context identity。 | `D-473` |
| `IRIS-V1-CONTROL-V362` | positive | 需要 interpreter；需要 JIT；native 不适用 | 移除继承的`to_bool` Method，来自 fixture Class `FallbackTruth`；其 `method_missing(selector, args, block)` 记录调用并返回 `true`；求值 `if FallbackTruth.new() { :then } else { :else }`。 | 结果是`:then`；`method_missing` 以 `[:to_bool, [], nil]` 被恰好调用一次，其 Bool 结果被直接使用且不再做真值测试。 | `D-353` |
