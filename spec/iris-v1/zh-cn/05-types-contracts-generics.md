# Iris v1 类型、Contracts 与泛型

状态：Iris v1 草案，语义已冻结。

IRIS-V1-TYPES-C001: 本章定义 Iris v1 的渐进类型 Contracts、`Dynamic<T>`、运行时类型测试与 cast、Type 对象、顶层和底层类型、nilability、`NonNil`、union 和 intersection 代数、callable 子类型、Contract 声明与视图、泛型约束与物化、Type aliases，以及 `Never` 流。它 MUST 在 [README.md](README.md)、[01-language-identity.md](01-language-identity.md)、[02-lexical-grammar.md](02-lexical-grammar.md)、[03-runtime-object-model.md](03-runtime-object-model.md) 和 [04-bindings-callables-control-flow.md](04-bindings-callables-control-flow.md) 之后阅读。

IRIS-V1-TYPES-C002: 本章 MUST NOT 定义 overload dispatch、声明处 variance、使用处 projection、raw generic instance types、隐式 generic conversion、递归 Type aliases、higher-kinded types、specialization as source semantics、conditional types、mapped types、任意编译期执行，或 structural auto-conformance。后续章节在细化 packages、reflection、conformance、migration、native binding 和 library API 时 MUST 保留这些排除项。

## 渐进 Contracts 与边界强制

IRIS-V1-TYPES-C003: 类型注解可选择书写，但一旦书写就必须遵守。省略的 Method 参数或返回注解具有静态和运行时 Contract `Dynamic<Object>`。实现 MAY 在主体内部推断局部事实以供诊断或优化，但它 MUST NOT 把推断出的主体事实导出为签名元数据。

IRIS-V1-TYPES-C004: 绑定、属性、参数、返回、泛型实参、Contract requirement、native metadata entry 或 Type alias target 上书写的类型注解，既是静态 Contract，也是运行时边界 guard。可证明的违反 MUST 在执行前诊断。未被证明的边界 MUST 在该值跨越边界发布或传递前在运行时检查。

IRIS-V1-TYPES-C005: 运行时类型 guards MUST 保留动态派发语义。它们验证值、callable 结果、属性写入、泛型物化、native metadata、reflection construction 和 Contract views，但它们 MUST NOT 选择不同的普通选择器、定义 overload dispatch、复制值、转换数据，或改变运行时接收者 Class。

IRIS-V1-TYPES-C006: 边界 Contract 失败 MUST 引发或报告 `TypeError`、`TypeContractError`，或拥有章节命名的更具体普通 Iris 诊断。JIT、native、Host、reflection、Dynamic 和 package 路径 MUST 保留相同边界检查，除非对仍有效的静态骨架的证明允许移除重复物理 guard。

IRIS-V1-TYPES-C007: 下列渐进边界表是规范性的：

| Boundary | Syntax or source | Static rule | Runtime guard | Reflection metadata |
| --- | --- | --- | --- | --- |
| Omitted Method parameter | `fun f(value)` | `Dynamic<Object>` | Dynamic bound check 后接受任何 Object | Parameter Type 是`Dynamic<Object>` |
| Omitted Method return | `fun f() { body }` | `Dynamic<Object>` | 返回值按 Dynamic bound 检查 | Return Type 是`Dynamic<Object>` |
| Binding annotation | `let x: T = expr` | `expr` 可赋给 `T` | 存储值必须满足`T` | Binding 或 debug metadata 记录`T` |
| Property write | `property name: T` | 写入值可赋给`T` | Setter 或 storage 检查`T` | Property Type 是`T` |
| Call argument | `target(arg)` | Actual 可赋给 parameter Type | Dynamic actual 在主体进入前检查 | Method signature 记录 parameter Types |
| Callable return | `fun f() -> T` | 每条正常路径可赋给`T` | 调用者观察前检查返回值 | Method signature 记录 return Type |
| Generic materialization | `Box<T>` | 实参满足约束 | Reflection/Dynamic/native 路径重新检查约束 | Closed Type 记录规范化实参 |
| Contract view | `value as C` | 值可能符合`C` | View construction 检查名义符合性 | View 记录 receiver relation 和 Contract Type |

IRIS-V1-TYPES-EX001: Informative example，可选注解仍强制已书写 Contracts：

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

## 类型表达式构造子

IRIS-V1-TYPES-C008: 本章使用的解析器可见类型表达式构造子是名义 Type 名称、闭合泛型应用 `G<T, U>`、callable Types `(P1, P2, ...) -> R`、unions `A | B`、intersections `A & B`、nilability 后缀 `T?`、`Dynamic<T>`、`Dynamic`、`NonNil`、`Never`、`Nil`、`Object`、括号化 Type expressions，以及透明 Type aliases。类型表达式中 `&` 比 `|` 绑定更紧。任何想要的不同分组 MUST 使用括号。

IRIS-V1-TYPES-C009: `Object` 是顶层 Type。每个 Iris 值，包括 `nil`、singleton values、带身份值、无身份值、Class objects、Module objects、Contract objects、Type objects、Methods、BoundMethods、Closures 和未来 core values，都是 `Object` 的子类型，除非后续章节显式把某一值类别标记在普通 Iris values 之外。

IRIS-V1-TYPES-C010: `Never` 是底层 Type。`Never` 没有可正常构造的运行时值，可赋给每个 Type，并由 raise 表达式、裸 re-raise、静态证明不终止路径、不可达路径，以及声明返回 Type 为 `Never` 的调用产生。

IRIS-V1-TYPES-C011: `Nil` 是唯一带身份 `nil` singleton 的 Type，且是 `Object` 的子类型。`String` 和 `User` 等具体 Types 不隐式包含 `nil`。接受 `nil` 与另一 Type 的程序 MUST 写 `T?` 或 `T | Nil`，除非 `T` 已经是包含 `Nil` 的超类型。

IRIS-V1-TYPES-C012: `T?` 是 `T | Nil` 的精确语法糖。规范化后它不是单独的 Type 构造子。`T?` 的 Type identity、reflection、hashing、assignability、runtime guards、generic arguments、narrowing 和 JIT 语义 MUST 与 `T | Nil` 相同。

IRIS-V1-TYPES-C013: `NonNil` 是一个 intrinsic marker Type，由除 `nil` 外的每个值满足。它不声明 Methods，不增加运行时 superclass，且 `Nil` 不能通过 metaprogramming 获得它。`Object & NonNil` 是规范 non-nil top Type。

IRIS-V1-TYPES-C014: `Dynamic<T>` 是有界动态消息发送。裸 `Dynamic` 规范化为 `Dynamic<Object>`。进入 `Dynamic<T>` 会检查该值满足 reified `T`；在 Dynamic 边界内，可发送任意普通选择器而无需静态成员验证。Qualified Contract dispatch 仍要求显式 Contract view 和 `..` 调用。

IRIS-V1-TYPES-C015: Type aliases 使用 `type Name<T...> = TypeExpr where ...`，其中 `where` 子句可选并遵循泛型约束语法。Aliases 是透明的，不创建名义运行时 wrapper。直接或间接递归 aliases，包括只通过容器递归的 aliases，在 v1 中 MUST 是编译期错误。

IRIS-V1-TYPES-C016: 运行时 Type objects 是 interned 带身份对象，区别于 Class、Module 和 Contract objects。每个规范化 reified Type expression 在适用运行时和包身份作用域内恰好有一个 interned Type identity。Type objects 上的 `same?` 和默认 `==` 比较规范 Type identity。

IRIS-V1-TYPES-C017: 下列构造子覆盖表是规范性的：

| Constructor | Syntax | Normalization | Assignability | Runtime guard | Reflection |
| --- | --- | --- | --- | --- | --- |
| Nominal Class | `User` | 规范命名 Type identity | 子类和自身可赋给它 | 当前运行时祖先检查 | `kind: class`, name, package, Class object |
| Nominal Contract | `Readable` | 规范命名 Type identity | 显式符合 Classes 可赋给它 | 名义 Contract 符合性检查 | `kind: contract`, name, package, Contract object |
| Closed generic | `Box<String>` | 按定义和规范化实参 intern | 不变精确构造加 bounds 内子类型规则 | 实参约束和名义成员资格 | `kind: generic`, definition, arguments |
| Callable | `(String) -> Object` | 规范参数类别和返回 Type | 参数逆变、返回协变、相同调用形状 | Callable 值加边界检查 | `kind: callable`, parameters, return |
| Union | `A \| B` | Flatten、sort、dedup、吸收子类型、bottom identity | Source 可赋值到至少一个 branch 时可赋值 | 任一 branch guard 成功 | `kind: union`, normalized members |
| Intersection | `A & B` | Flatten、sort、dedup、吸收超类型、top identity、不可能时为 Never | Source 满足每个 member 时可赋值 | 所有 member guards 成功 | `kind: intersection`, normalized members |
| Nilable | `T?` | `T \| Nil` | 同规范化 union | 同规范化 union | 反射为规范化 union，显示可偏好`?` |
| Dynamic | `Dynamic<T>` | Bare`Dynamic` 到 `Dynamic<Object>` | `T` 到 `Dynamic<T>`，有界 Dynamic widening | 进入时检查 bound，离开到静态 Type 时检查 | `kind: dynamic`, bound |
| NonNil | `NonNil` | Intrinsic marker，`Nil & NonNil` 到 `Never` | 任意 non-nil 值可赋值 | 只拒绝`nil` | `kind: marker`, name `NonNil` |
| Object | `Object` | Top，空 intersection identity | 每个值可赋值 | 对 Iris values 总是成功 | `kind: class`, root Class object |
| Never | `Never` | Bottom，空 union identity | 可赋给每个 Type | 无正常值可满足构造 | `kind: never` |
| Alias | `Name<T>` | 展开后规范化 target | 同展开 target | 同展开 target | 规范 target Type identity |

IRIS-V1-TYPES-EX002: Informative example，类型表达式构造子：

```iris
type Name = String & NonNil
type MaybeUser<T> = User<T>?

let printer: (Object) -> Nil = { |value: Object| -> Nil; print(value) }
let item: Dynamic<Object> = load_external_value()
let checked = item as? Name
```

## Union、Intersection、Nil 与 NonNil 代数

IRIS-V1-TYPES-C018: Union 和 intersection Types 是可交换、可结合、幂等、flattened 的，按稳定 Type identity 顺序排序，并在规范化后 intern。重复 members MUST 折叠。重排或重组的等价表达式 MUST 产生同一个 Type object identity。

IRIS-V1-TYPES-C019: 名义子类型吸收在规范化期间应用。如果 `Dog <: Animal`，则 `Dog | Animal` 规范化为 `Animal`，`Dog & Animal` 规范化为 `Dog`。吸收 MUST 使用不可变名义 superclass、已声明 Contract 和 closed generic identity facts，而不是结构性成员形状。

IRIS-V1-TYPES-C020: `T | Never` 规范化为 `T`，`T & Never` 规范化为 `Never`，空 union identity 是 `Never`，且 `T & Object` 规范化为 `T`。`Object | T` 规范化为 `Object`，空 intersection identity 是 `Object`。

IRIS-V1-TYPES-C021: `Object | Nil` 和 `Object?` 规范化为 `Object`。`Nil & Object` 规范化为 `Nil`。`Never?` 规范化为 `Nil`，因为 `Never | Nil` 恰好有 `Nil` inhabitant。

IRIS-V1-TYPES-C022: 将 union 与 `NonNil` 相交会在语义上移除其 `Nil` member。`(A | Nil) & NonNil` 规范化为 `A & NonNil`，如果 `A` 已知 non-nil，则规范化为 `A`。`Nil & NonNil` 规范化为 `Never`。对于无约束类型参数 `T`，`T & NonNil` 保持显式，并且 MUST NOT 简化为 `T`。

IRIS-V1-TYPES-C023: 规范 Type identity MUST NOT 把 intersections over unions 或 unions over intersections 展开为规范 DNF 或 CNF。Assignability、overlap 和 narrowing 算法 MAY 按需使用分配等价推理，但它们 MUST NOT 把指数展开 Type graphs intern 或暴露为规范形式。

IRIS-V1-TYPES-C024: 当名义 finality、Contract requirements、generic invariance 或其他冻结事实证明没有运行时值能满足所有 constituents 时，静态不可能的 intersection 规范化为 `Never`。如果开放性阻止证明，该 Type MAY 保持为 guarded intersection，由运行时符合性验证决定每个 candidate。

IRIS-V1-TYPES-C025: Union member access 只允许访问每个 branch 都存在且有一个安全调用 contract 的 members。对 union branches 上每个同名 member，静态检查 MUST 建模每个 branch 接受的 argument-count set，包括必需参数、可选参数、positional rest、keyword-only 参数、keyword rest 和可选 block 形状。只有调用元数属于每个 branch 的 accepted set 时才允许。一个 branch 中的 variadic 或 rest 接受 MUST NOT 覆盖另一个 branch 的有限限制。对被接受元数所提供的实际 positions、keywords 和 block channel，允许输入域是已接受 branch domains 的 intersection，结果 Type 是 branch return Types 的规范化 union。公共 API 不必可表示为一个简化声明。如果 argument-count sets、parameter modes、visibility、effects、Contracts 或 Types 无法产生一个 all-branches-safe call，则该 member 在 flow narrowing 前不可用。

IRIS-V1-TYPES-C026: Intersection same-name obligations 要求一个兼容实现。Iris MUST NOT 创建 overload sets、按静态 argument Type 选择、偏好声明顺序，或任意选择。不兼容 same-name requirements 要么使类型不可能、使声明不可满足，要么要求显式 qualified Contract slots。

IRIS-V1-TYPES-C027: 下列代数向量表是规范性的。一致性章节 MUST 保留这些向量 ID，或把它们映射到具有相同预期规范化 Type identity 的机器可读记录：

| Vector ID | Law | Expression | Expected normalized Type |
| --- | --- | --- | --- |
| `IRIS-V1-TYPES-V001` | Commutativity, union | `String \| Integer` and `Integer \| String` | 同一 interned union Type |
| `IRIS-V1-TYPES-V002` | Commutativity, intersection | `Readable & Closeable` and `Closeable & Readable` | 同一 interned intersection Type |
| `IRIS-V1-TYPES-V003` | Idempotence, union | `String \| String` | `String` |
| `IRIS-V1-TYPES-V004` | Idempotence, intersection | `String & String` | `String` |
| `IRIS-V1-TYPES-V005` | Absorption, union | `Dog \| Animal` where `Dog <: Animal` | `Animal` |
| `IRIS-V1-TYPES-V006` | Absorption, intersection | `Dog & Animal` where `Dog <: Animal` | `Dog` |
| `IRIS-V1-TYPES-V007` | Bottom union identity | `String \| Never` | `String` |
| `IRIS-V1-TYPES-V008` | Bottom intersection annihilator | `String & Never` | `Never` |
| `IRIS-V1-TYPES-V009` | Top intersection identity | `String & Object` | `String` |
| `IRIS-V1-TYPES-V010` | Top union absorber | `String \| Object` | `Object` |
| `IRIS-V1-TYPES-V011` | Nilability sugar | `String?` | `String \| Nil` normalized optional display |
| `IRIS-V1-TYPES-V012` | Object optional collapse | `Object?` | `Object` |
| `IRIS-V1-TYPES-V013` | Never optional collapse | `Never?` | `Nil` |
| `IRIS-V1-TYPES-V014` | NonNil nil removal | `(String \| Nil) & NonNil` | `String` |
| `IRIS-V1-TYPES-V015` | NonNil nil impossibility | `Nil & NonNil` | `Never` |
| `IRIS-V1-TYPES-V016` | No canonical distribution | `A & (B \| C)` | 包含 union member 的紧凑 intersection |

IRIS-V1-TYPES-EX003: Informative example，nil narrowing 和 union calls：

```iris
let name: String? = load_name()

if name != nil {
  let strong: String = name
}

let value: String | MutableString = load_text()
value.length()
```

## 可赋值性、测试、Casts 与流收窄

IRIS-V1-TYPES-C028: `value is T` 对 `value` 求值一次并返回 `Bool`。它检查 reified Types，包括名义 Classes、当前运行时祖先、Contracts、规范化 unions 和 intersections、闭合不变泛型实参、callable Types、value Types、meta Types、`NonNil`、`Never` 和 `Dynamic<T>` bounds。它 MUST NOT 对值调用 `to_bool`，并且 MUST NOT 执行数据转换。

IRIS-V1-TYPES-C029: `value as T` 对 `value` 求值一次，证明或检查该值满足 reified `T`，成功时返回同一底层对象或值，失败时引发 `TypeError`。已证明的 casts MAY 消除运行时 guards。静态不可能 casts MUST 是编译期错误。

IRIS-V1-TYPES-C030: `value as? T` 对 `value` 求值一次并返回 `T?`。运行时检查成功时返回同一底层对象或值，失败时返回 `nil`。静态不可能的 safe casts MUST 被诊断，而不是用作隐藏 fallback 惯用法。

IRIS-V1-TYPES-C031: Casts 从不执行数据转换、复制、替换对象分配、集合元素转换、数值转换、文本转换或泛型实参转换。`as` 不能跨越不变泛型实参。需要转换后数据的程序 MUST 调用显式 API。

IRIS-V1-TYPES-C032: `as ContractType` 构造一个已检查 Contract view。可存储 Contract view 携带 receiver relation 加 Contract identity 或 conformance。普通 `view.member()` 仍是转发给接收者的非限定消息。Qualified invocation MUST 使用 `view..member()` 或 `(value as ContractType)..member()`。

IRIS-V1-TYPES-C033: 来自 `is`、`as?`、nil 比较、typed catch 和 match Type patterns 的 flow narrowing，在 true 路径上使用 intersection，并在 Type algebra 可表示时排除已证明 branch。Iris v1 不引入一般 type subtraction，除本章显式 `NonNil` 和 nil-removal 规则外。

IRIS-V1-TYPES-C034: 命名 Class 或 Contract 的 typed catch 在 catch 主体运行前执行一次已检查 narrowing。Catch 绑定不可变，并具有该静态收窄 Type。后续 Class revision 不能移除或削弱支撑该 narrowing 的声明事实。

IRIS-V1-TYPES-C035: 下列运行时类型操作表是规范性的：

| Operation | Syntax | Success result | Failure result | Flow effect | Boundary preservation |
| --- | --- | --- | --- | --- | --- |
| Type test | `value is T` | `true` | `false` | True 路径按`T`收窄；false 路径在可表示时排除 | 无转换或选择器改变 |
| Strict cast | `value as T` | 同一值，类型为`T` | 引发`TypeError` | 后续表达式具有`T` | 无转换或不变性跨越 |
| Safe cast | `value as? T` | 同一值，类型为`T` | `nil` | 结果 Type 是`T?` | 无转换或不变性跨越 |
| Contract view | `value as C` | View 或同一可存储 view value | 引发`TypeError` | 结果可用`..`访问 qualified slots | 普通 dot 仍非限定 |
| Dynamic entry | `value as Dynamic<T>` or assignment | 同一值，带 Dynamic bound | 引发`TypeError` | Bound 内静态成员检查禁用 | 后续静态边界重新检查 |

IRIS-V1-TYPES-EX004: Informative example，casts 保留值：

```iris
let object: Object = load_value()

if object is String {
  object.length()
}

let maybe_user: User? = object as? User
let printable = object as Printable
printable..print()
```

## Callable Types 与 Method Contract 兼容性

IRIS-V1-TYPES-C036: BoundMethod 和 Closure 值共享写作 `(P1, P2, ...) -> R` 的普通 callable Types。完整 callable Type 包含元数、参数类别、关键字名称、rest 和 keyword-rest 通道、可选 block 通道、参数 Types、返回 Type 和必需运行时 Contracts。

IRIS-V1-TYPES-C037: Callable assignability 使用标准函数子类型。实现或源 callable 只有在接受 promise 允许的每个调用形状，并返回可赋给 promise return Type 的值时，才可赋给 promised callable Type。参数位置逆变，返回位置协变，参数或返回位置中的 callable Types 递归应用同一规则。

IRIS-V1-TYPES-C038: 用于 superclass、Contract、Module、intersection、open、package upgrade、native metadata 和 dynamic replacement 的 Method implementation compatibility 使用 IRIS-V1-TYPES-C037 中的 callable subtyping 规则。兼容主体 replacement 可改变实现细节，但 MUST 在原子发布前保留静态 promise。

IRIS-V1-TYPES-C039: Unbound Method objects 是反射性定义对象，不是普通 callable values。它们使用独立 reified Method metadata Type，并要求显式 receiver binding 或 reflective invocation validation 才能执行。把 unbound Method object 赋给 `(P...) -> R` MUST 失败，除非显式绑定操作产生 BoundMethod。

IRIS-V1-TYPES-C040: Callable compatibility 不创建 overload。同一 selector、同一 qualified Contract slot 或同一 property accessor location 仍只有一个被选择的 Method identity。静态注解、expected return Type、union branch 或 generic argument MUST NOT 在多个 implementations 中选择。

IRIS-V1-TYPES-EX005: Informative example，callable subtyping：

```iris
fun accepts_object(value: Object) -> String {
  value.to_string()
}

let string_to_object: (String) -> Object = accepts_object
```

## Contract 声明、继承、实现与视图

IRIS-V1-TYPES-C041: Contract 是 Iris v1 中显式静态和动态 obligation surface 的唯一术语。Contract declarations 使用 `contract Name<T...> extends ParentA, ParentB where T: ConstraintExpr meta deny capability { ... }`，每个允许子句至多出现一次且按语法顺序出现。没有 `extends` 的 Contract 没有 parent Contracts。

IRIS-V1-TYPES-C042: Contract bodies 可声明 public 或 protected instance Method requirements、Class-object Method requirements、property requirements，以及带 `where` 约束的 generic Method requirements。Contract bodies MUST NOT 包含 Method bodies、default implementations、stored state、initializers、raw ivars、private requirements、executable statements 或 open operations。

IRIS-V1-TYPES-C043: Contract 可继承多个具名 Contracts。继承形成静态 requirements 的 union，不带 implementation MRO、`super`、stored state 或 Method bodies。兼容 same-name requirements 按 callable compatibility 合并成一个 obligation。不兼容 same-name requirements 使 Contract declaration 成为编译期错误，除非它们通过显式 qualified Contract slots 表示为独立 identities。

IRIS-V1-TYPES-C044: Class Contract conformance 在 Class 头中用 `for` 声明，例如 `class Sprite extends Node for Drawable, Serializable { ... }`。只有 Classes 用 `for` 声明 instance Contract conformance。Module composition 本身从不转移名义 conformance，且 final Class static spine 必须显式列出每个声称的 Contract。

IRIS-V1-TYPES-C045: 对给定名义 Class declaration 和 revision static spine，已声明 Contract conformance 是不可变的。Metaprogramming 不能就地添加、移除、重命名、削弱或不兼容地替换已声明 Contract facts。Candidate revisions、opens、package upgrades、native metadata 和 reflection construction MUST 在发布前验证完整 static spine。

IRIS-V1-TYPES-C046: 旨在满足已声明 Contract requirement 的 member MUST 使用 `impl`。当没有已声明 Contract member 被满足时，编译器拒绝 `impl`；当要求显式 implementation 时，也拒绝未标记的 Class-provided implementation。如果声明还替换 inherited 或 Module Method，则两个 modifiers 都写出，例如 `override impl fun draw() -> Nil`。

IRIS-V1-TYPES-C047: 一个 `impl` member 自动满足每个已声明 same-name Contract requirement，只要其 slot 与该单一动态 Method signature 兼容。它不需要列出目标。如果 same-name Contract requirements 不兼容，Iris MUST NOT 按 Contract 顺序选择且 MUST NOT 形成 overloads。声明必须对不兼容 slots 使用显式 qualified Contract implementations。

IRIS-V1-TYPES-C048: Qualified Contract implementation 声明为 `impl ContractType::member(...) -> R { ... }`，在 inherited 或 Module replacement 规则要求时加 `override`。`::` 是声明和名称限定。它不选择普通表达式派发。

IRIS-V1-TYPES-C049: 显式 qualified Contract call 写作 `(value as ContractType)..member(args...)` 或 `view..member(args...)`。`as ContractType` 部分证明或检查名义 Contract view。`..member` token 选择 Contract-qualified slot identity。普通 `value.member(args...)` 始终发送非限定 selector。

IRIS-V1-TYPES-C050: Contract views 是不可变无身份 capability values。Contract view 上的 `same?` MUST 引发 `IdentityError`。重复 checked view construction MAY 免分配。对带身份 receivers，内置 view equality 要求相同 receiver identity 和相同 Contract identity。对无身份 receivers，内置 view equality 要求当前 equality 下的 receiver equality 加相同 Contract identity。

IRIS-V1-TYPES-C051: Contract view public hash 派生自 receiver public hash 和 Contract Type public hash。派生使用 BLAKE3 derive-key mode，精确 ASCII context 为 `Iris Language v1 contract view hash`，输入为 `receiver_public_hash_u64_le || contract_type_hash_u64_le`；digest reduction 使用前八个 digest bytes 作为 unsigned little-endian `Integer`。Receiver hash failure 会传播。

IRIS-V1-TYPES-C052: Named Contract Type public hash 基于 canonical package identity、fully qualified Contract name、Iris language major、type kind、generic arity、API major，以及适用时的 closed argument identities。它 MUST NOT 依赖 structural member shape、runtime allocation、filesystem path、display name、source hash 或 machine-local build data。

IRIS-V1-TYPES-C053: 下列 Contract 表是规范性的：

| Surface | Syntax | Normalization or identity | Assignability | Runtime guard | Reflection |
| --- | --- | --- | --- | --- | --- |
| Contract declaration | `contract C<T> extends P where T: Object { ... }` | 稳定名义 Contract identity | 显式`for C<T>`的 Classes 满足它 | Static spine 和 candidate validation | Contract object plus`.type`, parents, requirements |
| Class conformance | `class A for C { ... }` | 不可变 static spine fact | `A` instances 可赋给 `C` | Construction/open/upgrade validates requirements | Class metadata lists Contracts |
| Implementation | `impl fun m() -> R` | 一个 Method slot 满足兼容 obligations | 兼容 Method promise | Publication validates Method contract | Method metadata records impl relation |
| Qualified implementation | `impl C::m() -> R` | 独立 Contract slot identity | 满足该 qualified slot | Qualified dispatch validates receiver conformance | Requirement and Method metadata link slot |
| Contract view | `value as C` | Receiver relation 加 Contract identity | View 可赋给`C`和 Contract view Type | Checked nominal conformance | View Type records Contract and receiver relation |
| Qualified call | `view..m()` | 使用 Contract slot namespace | 要求 view 或 checked`as C` | 缺少 slot 引发 Contract dispatch error | Call metadata names qualified slot |

IRIS-V1-TYPES-EX006: Informative example，Contract 声明和实现：

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

IRIS-V1-TYPES-EX007: Informative example，不兼容 same-name Contract slots：

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

## 泛型、约束、推断与物化

IRIS-V1-TYPES-C054: Iris v1 支持用户定义的泛型 Class、Contract、Module 和 Method declarations，并保留运行时实参和元数据。泛型语法在声明和 Type grammar 上下文中使用 angle brackets，例如 `class Box<T>`、`contract Iterable<T>`、`module Helpers<T>`、`fun map<T, U>(value: T) -> U` 和 `Box<String>`。

IRIS-V1-TYPES-C055: 泛型约束使用 `where` 子句。语法用逗号分隔 constraint assignments，例如 `where T: A & B, U: C | D`。逗号开始不同参数或 `Self` 约束。`&`、`|`、括号和 Type aliases 组合一个受约束参数的 Type expression。逗号 MUST NOT 表示 intersection。

IRIS-V1-TYPES-C056: 无约束类型参数具有隐式上界 `Object`，不是 `Dynamic`、不是 `Object & NonNil`，也不是隐藏 non-nil bound。`Nil` 是无约束类型参数的有效实参。排除 `nil` 需要显式约束，例如 `where T: NonNil` 或 `where T: Object & NonNil`。

IRIS-V1-TYPES-C057: 泛型 declarations 可用同一声明中的 peer parameters 约束参数。声明顺序不限制引用。编译器和运行时用 union 和 intersection normalization 求解依赖图。不可满足或不能唯一解析的循环关系是编译期声明错误。

IRIS-V1-TYPES-C058: V1 允许受限名义 F-bounded constraints，例如 `where T: Comparable<T>`。具体实参只有在替换后通过显式名义 Class 或 Contract conformance 时才满足此 bound。匹配 member shape 不足够。Higher-kinded、任意递归类型函数、fixed-point programs 和无效递归约束会被拒绝。

IRIS-V1-TYPES-C059: Generic Class 和 Contract instantiations 严格不变。`G<A>` 和 `G<B>` 不会仅因 `A <: B` 而具有 subtype、assignment 或 cast 关系。`as` 和 `as?` 不能跨越不变泛型实参。Collection 或 container conversion 是显式用户代码。

IRIS-V1-TYPES-C060: 每个 generic Class、Contract、Module 和 Method application 都有固定完整元数。V1 没有默认 generic type arguments。缺少尾部实参是元数错误，绝不表示 `Object`、`Dynamic<Object>` 或推断默认值。

IRIS-V1-TYPES-C061: 裸 generic Class names 只表示 definition metadata。对 `class Box<T>`，裸 `Box` 是用于 reflection 和 opening definition 的带身份 generic definition object。它不是 instance Type、raw generic Type，或 `Box<Object>` 的简写。实例注解和普通构造需要闭合 `Box<Type>` 或 construction-site `Box<_>`。

IRIS-V1-TYPES-C062: 闭合 generic constructions 按 generic definition identity 和规范化有序 Type argument identities intern。同一运行时中重复 `Box<String>` references 产生同一个 closed Type 和 closed logical Class identity。不同 definitions 或不同规范化 arguments 产生不同 identities。

IRIS-V1-TYPES-C063: Opening generic Class 或 Module definition 会构建 candidate definition，并为每个已 interned closed construction 构建替换后的 candidates。所有 static spines、constraints、Contracts、MRO、Modules、native obligations 和 layout obligations MUST 作为一个 transaction 验证。`open class Box<String>` 和 programmatic `Box<String>.open` 在 v1 中是错误。

IRIS-V1-TYPES-C064: 普通 generic class-level storage 按 closed construction 独立。声明在 `Generic<T>` 上的 class-level property 在 `Generic<String>` 和 `Generic<User>` 中有独立存储。`shared class property` 属于未应用 generic definition，且 MUST NOT 直接或间接引用 definition 的 type parameters。

IRIS-V1-TYPES-C065: Definition-wide shared generic properties 只能通过未应用 generic definition object 读写，例如 `Cache.count`。`Cache<String>` 等 closed Classes 不继承或转发该 accessor。Reflection MUST 标记 definition ownership 与 per-closed ownership。

IRIS-V1-TYPES-C066: Shared generic property initializers 在 origin module initialization 期间按源声明顺序运行一次。Per-closed class property initializers 在 closed Class 第一次物化时，在 closed-Class creation transaction 内运行一次。失败的 closed-Class materialization MUST 丢弃 candidate Class、property、Method、MRO、Contract、interning 和 JIT-cache state，并且 MUST 对该 closed construction 不发布也不 intern 任何内容。它 MUST NOT 自动撤销 filesystem、network、database、native、logging、对已发布对象的 mutation 或其他外部副作用。后续请求 MAY 重试 materialization 并重新运行 initializers，initializer 作者拥有 cleanup、compensation、retry safety 和 duplicate external side effects 的责任。

IRIS-V1-TYPES-C067: Closed generic materialization 在 interning 或 publishing 之前验证每个规范化 `where` 约束。Reflection、Dynamic、plugin、deserialization、native construction 和 package-loading 路径 MUST 执行运行时验证，除非由仍有效静态事实证明为重复。失败引发 `TypeContractError` 且不发布 closed Class。

IRIS-V1-TYPES-C068: Generic Method type inference 是局部且有界的。省略的 Method type arguments 只从 actual arguments 的 static Types 和显式 immediate expected result Type 推断。Inference MUST NOT 检查 Method bodies、runtime values、任意后续 uses 或 whole-program state。

IRIS-V1-TYPES-C069: Inference 把一个 type parameter 的多个 lower-bound candidates 合并为其规范化 union，把多个 upper-bound candidates 合并为其规范化 intersection。结果必须是一个唯一 most-specific substitution，满足每个 bound 和 `where` expression。否则调用要求显式 type arguments 或被拒绝。

IRIS-V1-TYPES-C070: 来自 annotated assignment、return Contract、argument position 或等价 immediate context 的显式 expected result MAY 推断 Method type arguments，包括无实参 factories，例如 `let user: User = make()`。独立无约束调用 MUST NOT 把 type parameter 默认成 `Object`。

IRIS-V1-TYPES-C071: 部分显式 Method type arguments 使用带 `_` placeholders 的完整元数 angle brackets，例如 `choose<String, _>(value)`。每个 `_` 请求该位置的局部 inference。省略尾部位置是元数错误。此位置中的 `_` 是 type-argument placeholder，不是绑定，也不是新的用户命名 type variable。

IRIS-V1-TYPES-C072: Generic Class construction 只可在完整元数 construction-site type argument list 中使用 `_`，例如 `Box<_>.new(value)`。它从 constructor arguments 和 immediate expected result Type 推断。`_` 禁止出现在 persistent Type positions 中，例如 variable、property、parameter、return annotations、Contract declarations、base Types、stored reflection metadata 和 generic constraints。

IRIS-V1-TYPES-C073: Generic Modules 像其他 generic definitions 一样 reified 和 interned。Generic Module arguments 在 mix 或 include site 显式提供，不从 host Class 推断。Intrinsic `Self` 可出现在 Module `where` constraints 中，表示最终 receiver 或 host。V1 没有 generic Module type arguments 的 `_` inference。

IRIS-V1-TYPES-C074: 下列泛型特性表是规范性的：

| Feature | Syntax | Normalization or materialization | Assignability | Runtime guard | Reflection |
| --- | --- | --- | --- | --- | --- |
| Generic declaration | `class Box<T> where T: Object {}` | Definition identity 加 parameter list | 不是 instance Type | Declaration constraints validated | Generic definition metadata |
| Closed application | `Box<String>` | 按 definition 和规范化 arguments intern | 不变精确构造 | Materialization validates constraints | Closed Type and logical Class metadata |
| Method inference | `map<T, _>(value)` | 唯一局部 substitution | Instantiated Method signature must be callable | Call boundary checks substituted Types | Inferred arguments appear in call metadata if exposed |
| Class construction inference | `Box<_>.new(value)` | 带局部 placeholders 的完整元数 | 唯一时创建 closed Class | Constructor and constraints checked | Closed construction metadata |
| F-bound | `where T: Comparable<T>` | 替换后名义 conformance | Argument accepted only by explicit conformance | Materialization revalidates | Constraint graph records self reference |
| Shared property | `shared class property count: Integer` | generic definition 上的一个 slot | 不按 closed Type | Initializer checked once | Reflection marks definition storage |
| Per-closed property | `class property value: T` | 每个 closed construction 一个替换后 slot | T-specific value Contract | Materialization initializer checked | Reflection marks closed storage |
| Generic Module | `mixin Helpers<User>` | 按显式 arguments intern 的 closed Module | Host must satisfy`Self` constraints | Composition validates transactionally | Module metadata records arguments and Self constraints |

IRIS-V1-TYPES-EX008: Informative example，generic constraints 和 invariance：

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

IRIS-V1-TYPES-EX009: Informative example，local generic inference：

```iris
fun make<T>() -> T where T: Object {
  load_value() as T
}

let user: User = make()
let box = Box<_>.new(user)
```

## Type Objects、Reflection 与稳定 Type Identity

IRIS-V1-TYPES-C075: Type reflection 至少暴露 `kind`、`arguments`、`members`、`subtype?` 和 `assignable?` 给 Type objects。Reflection MUST 报告规范化 canonical Type identity，而不是源拼写，除非 display APIs 选择偏好的表面形式，例如 `T?`。

IRIS-V1-TYPES-C076: Class、Module 和 Contract objects 暴露相关 `.type` metadata，但 Type object 本身不是 Class、Module 或 Contract object。该区别允许 union、intersection、`Dynamic<T>`、callable、`Never`、`NonNil`、nilable 和 closed generic constructions 共享一个 reflection API。

IRIS-V1-TYPES-C077: 对组件具有稳定 package identities 的可发布 named 和 composite Types，Type public hash 是稳定的。对本地匿名或无 manifest identities，它是运行时局部的。Stable Type hash derivations MUST 包含 Type kind 和规范化 component Type identities，使 `A | B` 和 `B | A` 在规范化后 hash 相同。

IRIS-V1-TYPES-C078: Type reflection 和 hash identity MUST NOT 只依赖 display name、member structural shape、runtime allocation order、filesystem path、source hash 或 machine-local build data。可发布名义 identity 遵循 package ID、package API major、Iris language major、fully qualified name、Type kind、generic arity 和 closed argument identities。

IRIS-V1-TYPES-C079: `Type#subtype?(other)` 和 `Type#assignable?(other)` MUST 使用与编译器和运行时 guards 相同的名义、generic invariance、callable subtyping、Dynamic-boundary、union 和 intersection、nilability 和 `NonNil` 规则。它们是对当前有效 Type facts 的查询，不是绕过边界检查的许可。

IRIS-V1-TYPES-EX010: Informative example，Type reflection identity：

```iris
let a = (String | Nil).type
let b = String?.type

a same? b  // true
```

## Never 流、诊断与不可达代码

IRIS-V1-TYPES-C080: `raise`、裸 re-raise、静态证明不终止循环、声明为 `-> Never` 的调用和静态不可达路径具有 Type `Never`。`Never` 路径不会拓宽 `if`、`match`、loops、`try`、callable returns 或 generic inference 的结果 unions。

IRIS-V1-TYPES-C081: 声明为 `-> Never` 的 callable MUST NOT 正常完成。如果正常完成可被静态证明，则是编译期错误。如果 Dynamic、reflection、native 或 Host 路径让正常完成到达边界，运行时 MUST 在调用者观察到正常结果前引发 `TypeError`。

IRIS-V1-TYPES-C082: 静态不可达 statements 收到默认编译器警告。它们仍会为诊断、名称解析、畸形语法和无效 Type use 接受类型检查。Tool policy MAY 把该警告提升为错误，且不改变语言语义。

IRIS-V1-TYPES-C083: Exhaustive `match`、definite return、definite assignment、typed catch 和 branch result analysis MUST 把 `Never` 当作 bottom。只能 raise 或调用 `Never` callable 的 branch 不向包围表达式结果贡献 value Type。

IRIS-V1-TYPES-EX011: Informative example，Never flow：

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

## 排除项与可追溯性说明

IRIS-V1-TYPES-C084: Iris v1 没有 overload sets、没有 implicit type-directed dispatch、没有 declaration-site variance、没有 use-site projection、没有 raw generic instance Type、没有 implicit generic conversion、没有 default generic type arguments、没有 recursive Type aliases、没有 higher-kinded Types、没有 dependent Types、没有 non-type generic parameters、没有 variadic type parameters、没有 conditional Types、没有 mapped Types、没有 source-level specialization semantics，且没有 structural auto-conformance。

IRIS-V1-TYPES-C085: `Dynamic<T>` 从不擦除边界检查。进入 Dynamic bound 的值按 `T` 检查；离开 Dynamic 到静态边界的值按该边界检查；parameter、return、property、generic、native、reflection 和 Contract-view guards 保持生效。

IRIS-V1-TYPES-C086: 本章中的 Contract 术语是独占且规范的。此概念的历史或非规范名称 MUST NOT 出现在规范性 Iris v1 type、reflection、migration、native、library 或 conformance artifacts 中，除非作为本章外清楚标记的历史或 migration replacement text。

IRIS-V1-TYPES-C087: Foundation conflict report：`Document/Iris Revival Design Review.md:831` through `:837` 中的历史 design review 示例使用 `Array[Integer]`，而冻结 grammar 和 generic decisions 要求 `Array<Integer>`。同一 review section 使用了 Contract 的旧 protocol 词。本章遵循批准草案和当前 foundations：angle-bracket generics 和规范 Contract terminology。

IRIS-V1-TYPES-C088: 本章整合 D-172 through D-220、D-233 through D-241，以及 D-452 through D-458。批准草案中的后续修订措辞优先于更早历史 review phrasing。

IRIS-V1-TYPES-C089: 本章拥有的决策 ID 是 `D-172`, `D-173`, `D-174`, `D-175`, `D-176`, `D-177`, `D-178`, `D-179`, `D-180`, `D-181`, `D-182`, `D-183`, `D-184`, `D-185`, `D-186`, `D-187`, `D-188`, `D-189`, `D-190`, `D-191`, `D-192`, `D-193`, `D-194`, `D-195`, `D-196`, `D-197`, `D-198`, `D-199`, `D-200`, `D-201`, `D-202`, `D-203`, `D-204`, `D-205`, `D-206`, `D-207`, `D-208`, `D-209`, `D-210`, `D-211`, `D-212`, `D-213`, `D-214`, `D-215`, `D-216`, `D-217`, `D-218`, `D-219`, `D-220`, `D-233`, `D-234`, `D-235`, `D-236`, `D-237`, `D-238`, `D-239`, `D-240`, `D-241`, `D-452`, `D-453`, `D-454`, `D-455`, `D-456`, `D-457`, and `D-458`.

IRIS-V1-TYPES-C090: 本地引用但委托的 syntax、runtime、package、reflection、collection 和 meta-header 决策 ID 是 `D-077`, `D-242`, `D-243`, `D-244`, `D-245`, `D-246`, `D-247`, `D-248`, `D-249`, `D-250`, `D-258`, `D-259`, `D-260`, `D-261`, `D-262`, `D-275`, `D-276`, `D-277`, `D-278`, `D-279`, `D-280`, `D-281`, `D-282`, `D-283`, `D-298`, `D-466`, `D-495`, `D-507`, `D-508`, `D-509`, and `D-510`。IRIS-V1-TYPES-C089 中列出的本章拥有 IDs 仍可能在其他章节有非类型 syntax、runtime dispatch、package identity、reflection 或 metaprogramming anchors；这不会使它们对本章类型语义成为 delegated。其他章节仍对本章类型语义之外的细节拥有权威性。

IRIS-V1-TYPES-C091: 一致性章节 MUST 为 IRIS-V1-TYPES-C017 中的每个 Type constructor、IRIS-V1-TYPES-C027 中的每个 algebra vector、Contract qualified dispatch 和 view hash behavior、generic invariance and materialization、Dynamic boundary preservation、Type alias transparency，以及 `Never` flow 包含 positive、failure、diagnostic 和 reflection vectors。

## 类型覆盖向量

IRIS-V1-TYPES-C092: 下列向量是带有具体类型检查输入和预期观察的规范性可追溯向量。

| Vector ID | Category | Applicability | Source/Input | Expected observable | Decisions |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-TYPES-V018` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture: `contract Named { fun name() -> String } class User for Named { impl fun name() -> String { "iris" } } let view = User.new() as Named; view..name()`. | 值 `"iris"`；Type `String`；checked view construction 和 explicit qualified dispatch 选择 `Named::name`。 | `D-233`, `D-234`, `D-236`, `D-237`, `D-239`, `D-278` |

## 审计精确一致性向量

这些行是规范性审计精确向量。每一行命名一个具体已审计决策及其可观察结果。

| Vector ID | Category | Applicability | Source/Input | Expected observable | Decisions |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-TYPES-V200` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Metadata fixture:`Class A for C`; candidate removes `C`. | `TypeContractError` before commit；`A` 仍符合 `C`。 | `D-173` |
| `IRIS-V1-TYPES-V201` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Metadata fixture:`class Dog extends Animal`; candidate changes superclass to `Object`. | `TypeContractError` before commit；`Dog.type.subtype?(Animal.type)` 仍为 `true`。 | `D-174` |
| `IRIS-V1-TYPES-V202` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Metadata fixture: compose a Module whose`draw(String)` conflicts with declared Contract `draw(Integer)`. | `TypeContractError`；先前 Class MRO 和 Contract conformance 仍已发布。 | `D-175` |
| `IRIS-V1-TYPES-V203` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Metadata fixture: open candidate replaces Contract-visible`draw() -> String` with `draw() -> Integer`. | `TypeContractError`；不发布 candidate Method 或 revision。 | `D-176` |
| `IRIS-V1-TYPES-V204` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`open contract C { fun m() -> Nil }`. | 静态诊断`OPEN_CONTRACT_FORBIDDEN`；不存在 Contract revision。 | `D-177` |
| `IRIS-V1-TYPES-V205` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`open class A { fun marker() -> String { "open" } } class A { fun marker() -> String { "origin" } } A.new().marker()`. | 值`"open"`；Type `String`；declaration collection 在 open transaction 前解析 origin。 | `D-178` |
| `IRIS-V1-TYPES-V206` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Metadata fixture:`A.open { add valid m; add incompatible Contract method }`. | `TypeContractError`；reflection 不暴露任何 staged member。 | `D-179` |
| `IRIS-V1-TYPES-V207` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Metadata fixture: inside`A.open`, write property `x`, read `x`, then force validation failure; concurrent reader queries `A.properties`. | 内部读取返回 staged property；外部查询看到旧 property set；rollback 不发布`x`。 | `D-180` |
| `IRIS-V1-TYPES-V208` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Metadata fixture: programmatic open adds public`extra()` to `A`; static caller uses `A` while reflective caller invokes `extra`. | Static call 被拒绝；reflective/Dynamic call 只在成功发布后允许。 | `D-181` |
| `IRIS-V1-TYPES-V209` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`class Box<T> {} let raw: Box = Box<String>.new()`. | 静态诊断`RAW_GENERIC_TYPE_FORBIDDEN`；不形成 raw instance Type。 | `D-182` |
| `IRIS-V1-TYPES-V210` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`class Box<T> {} let item: Box[String] = Box<String>.new()`. | 解析器诊断`GENERIC_BRACKET_SYNTAX_FORBIDDEN`；`Box<String>` 仍是接受拼写。 | `D-183` |
| `IRIS-V1-TYPES-V211` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`let value: String \| Integer = "iris"; value is String`. | 值`true`；Type `Bool`；注解是 reified union `String \| Integer`。 | `D-184` |
| `IRIS-V1-TYPES-V212` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`(String?).type same? (String \| Nil).type`. | 值`true`；Type `Bool`；两个表达式有同一个规范化 Type identity。 | `D-185` |
| `IRIS-V1-TYPES-V213` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`(Dog \| Animal).type same? Animal.type`, where `class Dog extends Animal {}`. | 值`true`；Type `Bool`；union 吸收 `Dog`。 | `D-186` |
| `IRIS-V1-TYPES-V214` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Metadata fixture: reflect`A & (B \| C)`. | Type kind`intersection`；members 是 `A` 和规范化 union `B \| C`，不是 distributed alternatives。 | `D-187` |
| `IRIS-V1-TYPES-V215` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`(Nil & NonNil).type`. | Type`Never`；无正常值满足该 Type。 | `D-188` |
| `IRIS-V1-TYPES-V216` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`(String \| Never).type same? String.type`. | 值`true`；Type `Bool`。 | `D-189` |
| `IRIS-V1-TYPES-V217` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`(String & Object).type same? String.type`. | 值`true`；Type `Bool`。 | `D-190` |
| `IRIS-V1-TYPES-V218` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`(Object?).type same? Object.type`. | 值`true`；Type `Bool`。 | `D-191` |
| `IRIS-V1-TYPES-V219` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`class Box<T> {} let item: Box<Nil> = Box<Nil>.new()`. | Construction succeeds；Type`Box<Nil>`；implicit `T` bound 是 `Object`。 | `D-192` |
| `IRIS-V1-TYPES-V220` | negative | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`let value: NonNil = nil`. | `TypeContractError` at binding boundary；不存储值。 | `D-193` |
| `IRIS-V1-TYPES-V221` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`((String \| Nil) & NonNil).type same? String.type`. | 值`true`；Type `Bool`。 | `D-194` |
| `IRIS-V1-TYPES-V222` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`let text: String \| MutableString = load_text(); text.length(); text.append("x")`. | `length()` Type-checks with return Type `Integer`；`append` 不可用，诊断 `UNION_MEMBER_NOT_COMMON`。 | `D-195` |
| `IRIS-V1-TYPES-V223` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`let value: A \| B = load(); value.m(); value.m(1)`, where `A#m()` and `B#m(x: Integer)`. | 两个调用都以`UNION_CALL_ARITY_MISMATCH` 拒绝；accepted-arity intersection 为空。 | `D-196` |
| `IRIS-V1-TYPES-V224` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`contract A { fun m(x: String) -> String } contract B { fun m(x: Object) -> Integer } class X for A, B { impl fun m(x: Object) -> String { "x" } }`. | 静态诊断`CONTRACT_REQUIREMENT_INCOMPATIBLE`；不创建 overload。 | `D-197` |
| `IRIS-V1-TYPES-V225` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`fun accept(x: Object) -> String { "ok" }; let f: (String) -> Object = accept; f("x")`. | 值`"ok"`；Type `Object`；parameter contravariance 和 return covariance 成立。 | `D-198` |
| `IRIS-V1-TYPES-V226` | negative | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`class Box<T> {} let target: Box<Object> = Box<String>.new()`. | 静态`TypeContractError`；不变 `Box<String>` 不可赋给 `Box<Object>`。 | `D-199` |
| `IRIS-V1-TYPES-V227` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`fun id<T>(x: T) -> T { x }; let result: String = id("iris")`. | 值`"iris"`；推断 Type argument `String`；result Type `String`。 | `D-200` |
| `IRIS-V1-TYPES-V228` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`fun pair<T>(a: T, b: T) -> T { a }; pair("x", 1)`. | 推断`T` 是规范化 `String \| Integer`；若 target 要求 `String`，则静态 `TypeContractError`。 | `D-201` |
| `IRIS-V1-TYPES-V229` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`fun make<T>() -> T { load_value() as T }; let user: User = make(); make()`. | 第一次调用推断`User`；独立调用报告 `GENERIC_INFERENCE_UNCONSTRAINED`。 | `D-202` |
| `IRIS-V1-TYPES-V230` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`fun choose<T,U>(x: T, y: U) -> U { y }; choose<String, _>("x", 1); choose<String>("x", 1)`. | 第一次调用返回`Integer(1)` 且 `U = Integer`；第二次报告 `GENERIC_ARGUMENT_ARITY`。 | `D-203` |
| `IRIS-V1-TYPES-V231` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`class Box<T> { fun initialize(value: T) -> Nil {} } let b: Box<String> = Box<_>.new("x"); let bad: Box<_> = b`. | Construction infers`Box<String>`；persistent annotation 报告 `GENERIC_PLACEHOLDER_FORBIDDEN`。 | `D-204` |
| `IRIS-V1-TYPES-V232` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`class Box<T> {} Box.new()`. | 静态诊断`GENERIC_ARGUMENT_ARITY`；裸 `Box` 是 definition metadata，不是 `Box<Object>`。 | `D-205` |
| `IRIS-V1-TYPES-V233` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`class Box<T> {} Box<String>.type same? Box<String>.type; Box<String>.type same? Box<Integer>.type`. | 值`true`，然后 `false`；closed identities 按 definition 和规范化 arguments intern。 | `D-206` |
| `IRIS-V1-TYPES-V234` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Metadata fixture: materialize`Box<String>` and `Box<Integer>`; open `Box<T>` with a member violating one substituted constraint. | `TypeContractError`；definition 和两个 closed revision 都不改变。 | `D-207` |
| `IRIS-V1-TYPES-V235` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`open class Box<String> { fun m() -> Nil {} }`. | 静态诊断`CLOSED_GENERIC_OPEN_FORBIDDEN`。 | `D-208` |
| `IRIS-V1-TYPES-V236` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`class Cache<T> { class property value: T } Cache<String>.value = "s"; Cache<Integer>.value = 1`. | 读取分别为`"s"` 和 `Integer(1)`；每个 closed Class 的 storage 独立。 | `D-209` |
| `IRIS-V1-TYPES-V237` | negative | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`class Cache<T> { shared class property bad: T }`. | 静态诊断`GENERIC_SHARED_PROPERTY_REFERENCES_TYPE_PARAMETER`。 | `D-210` |
| `IRIS-V1-TYPES-V238` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`class Cache<T> { shared class property count: Integer = 0 } Cache.count; Cache<String>.count`. | `Cache.count` 返回 `Integer(0)`；closed access 报告 `MESSAGE_NOT_FOUND`。 | `D-211` |
| `IRIS-V1-TYPES-V239` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Metadata fixture: module declares`shared class property first: Integer = 1` then initializer `raise :stop`. | Module initialization fails；status`not_published`；两个 shared property 都不可观察。 | `D-212` |
| `IRIS-V1-TYPES-V240` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Metadata fixture:`Box<T>` per-closed initializer appends its closed Type to a log; request `Box<String>`, `Box<String>`, `Box<Integer>`. | Log 是`[Box<String>, Box<Integer>]`；每个 closed initializer 运行一次。 | `D-213` |
| `IRIS-V1-TYPES-V241` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Metadata fixture:`Box<T>` initializer records external `"attempt"` then raises during `Box<String>` materialization; request it twice. | 每次 request 引发`TypeContractError`；不存在 `Box<String>` intern entry；external log 是 `["attempt", "attempt"]`。 | `D-214` |
| `IRIS-V1-TYPES-V242` | negative | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Metadata fixture: reflection constructs`Box<Nil>` for `class Box<T> where T: NonNil {}`. | `TypeContractError`；status `not_published`；不 intern closed Type identity。 | `D-215` |
| `IRIS-V1-TYPES-V243` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`class Pair<T,U> where U: T {} class Bad<T,U> where T: U, U: T {}`. | `Pair<Object,String>` 因 bound 被拒绝；`Bad` 报告 `GENERIC_CONSTRAINT_CYCLE`。 | `D-216` |
| `IRIS-V1-TYPES-V244` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`contract Comparable<T> {} class Box<T> where T: Comparable<T> {} Box<String>.new()`. | `TypeContractError`，除非 `String` 显式符合 `Comparable<String>`；structural members 不满足它。 | `D-217` |
| `IRIS-V1-TYPES-V245` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`class Pair<T,U> {} let p: Pair<String> = Pair<String, Integer>.new()`. | 静态诊断`GENERIC_ARGUMENT_ARITY`；不提供默认 `U`。 | `D-218` |
| `IRIS-V1-TYPES-V246` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`module Helpers<T> {} mixin Helpers<String>`. | Closed Module Type`Helpers<String>` 被 reified 和 interned。 | `D-219` |
| `IRIS-V1-TYPES-V247` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`module Helpers<T> where Self: T {} class Host { mixin Helpers<_> }`. | 静态诊断`GENERIC_MODULE_ARGUMENTS_EXPLICIT`；不使用 host inference。 | `D-220` |
| `IRIS-V1-TYPES-V248` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`contract C { fun m() -> Nil } class A for C { fun m() -> Nil {} }`. | 静态诊断`CONTRACT_IMPLEMENTATION_REQUIRES_IMPL`。 | `D-233` |
| `IRIS-V1-TYPES-V249` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture: compatible`contract A` and `contract B` both require `m(Object) -> String`; `class X for A, B { impl fun m(x: Object) -> String { "x" } }`. | 一个 Method 满足两个 requirements；`X.new() as A` 和 `X.new() as B` 各自把 `m` qualify 到 `"x"`。 | `D-234`, `D-276` |
| `IRIS-V1-TYPES-V250` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`contract C { fun m() -> String } class X for C { impl fun m() -> String { "c" } } let view = X.new() as C; view..m()`. | 值`"c"`；Type `String`；只有显式 `..m()` 选择 Contract slot。 | `D-237` |
| `IRIS-V1-TYPES-V251` | negative | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`let a = X.new() as C; a same? a`. | `IdentityError`；Contract views 没有独立 identity。 | `D-239` |
| `IRIS-V1-TYPES-V252` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture: two`Integer(1) as NumericContract` views with the same Contract identity, compared with `==`. | 值`true`；Type `Bool`；equality 对无身份 receivers 使用 value equality。 | `D-240` |
| `IRIS-V1-TYPES-V253` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`fun f(value) { value }; f.method.parameters[0].type; f.method.return_type`. | 两个 reflected Types 都是`Dynamic<Object>`；签名元数据中没有 body-local inference。 | `D-452` |
| `IRIS-V1-TYPES-V254` | negative | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`let value: Dynamic<String> = 1`. | `TypeError` at Dynamic entry；任意 selector send 不开始。 | `D-453` |
| `IRIS-V1-TYPES-V255` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`let value: Object = "iris"; [value is String, value as? Integer]`. | 值`[true, nil]`；Type `Array<Bool \| Nil>`；值不转换。 | `D-454` |
| `IRIS-V1-TYPES-V256` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`type Name<T> = Array<T>; Name<String>.type same? Array<String>.type; type Loop = Array<Loop>`. | 第一个表达式返回`true`；第二个声明报告 `RECURSIVE_TYPE_ALIAS`。 | `D-455` |
| `IRIS-V1-TYPES-V257` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`class Box<T> {} let t = Box<String>.type; [t same? Box<String>.type, t same? Box<String>]`. | 值`[true, false]`；`t` 是 interned Type object，不同于 Class object。 | `D-456` |
| `IRIS-V1-TYPES-V258` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`contract C { fun m() -> Nil { nil } }`. | 静态诊断`CONTRACT_METHOD_BODY_FORBIDDEN`；不发布 Contract declaration。 | `D-275`, `D-457` |
| `IRIS-V1-TYPES-V259` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture:`fun fail() -> Never { nil }`. | 静态`TypeContractError`；如果通过 Dynamic 边界到达，运行时在观察正常返回前引发 `TypeError`。 | `D-458` |
| `IRIS-V1-TYPES-V260` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Metadata fixture: construct named Contract Type `pkg@1::C` twice with different source paths and allocation order, then construct `pkg@2::C`. | 前两个 public hashes 相等；API-major-changed Contract Type public hash 不同。 | `D-242` |
| `IRIS-V1-TYPES-V261` | diagnostic | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Iris source fixture: `contract C { fun m() -> Nil } module M for C {} class A for C { impl fun m() -> Nil { nil } }`. | `module M for C` 报告 `CONTRACT_FOR_CLASS_ONLY`；`A` 被接受并名义符合 `C`。 | `D-279` |
| `IRIS-V1-TYPES-V262` | positive | 需要 compiler；需要 interpreter；需要 JIT；native 不适用 | Metadata fixture: Contract view receiver public hash is `1`, Contract Type public hash is `2`, and the view is hashed. | Artifact 使用 BLAKE3 derive-key context `Iris Language v1 contract view hash`、输入 `01000000000000000200000000000000`，且 public Integer 来自 digest bytes `0..7` as unsigned little-endian。 | `D-241` |
