# Iris v1 运行时对象模型

状态：Iris v1.2，冻结语义并有所有者批准的勘误。

IRIS-V1-RUNTIME-C001：本章定义 Iris v1 的运行时值、对象性、身份、派发、逻辑 Class 与活动修订语义、Module MRO、Method 与 BoundMethod 身份、构造、属性、原始 ivar、类变量、真值性、缺失消息处理、内置数值行为、哈希以及内置开放性。本章 MUST 在 [README.md](README.md)、[01-language-identity.md](01-language-identity.md) 和 [02-lexical-grammar.md](02-lexical-grammar.md) 之后阅读。

IRIS-V1-RUNTIME-C002：本章 MUST NOT 定义解析器产生式、渐进类型代数、包初始化、open 事务调度、反射 API、序列化格式、FFI 句柄、异步调度，或超出后续章节所需运行时契约的集合操作细节。后续章节在细化自身表面时 MUST 保留本章中的锚点。

## 运行时值基础

IRIS-V1-RUNTIME-C003：每个 Iris 运行时值 MUST 都是一个对象。对象性意味着该值具有运行时 Class 或值-Class 关系，根据本章接受消息协议，并参与其类别指定的相等、哈希、真值性和异常引发。

IRIS-V1-RUNTIME-C004：对象性并不意味着可观察身份。符合要求的实现 MUST 根据 IRIS-V1-RUNTIME-C006 将每个内置值类别分类为带身份或无身份。`same?` MUST 只接受带身份操作数，并且 MUST 对无身份操作数引发 `IdentityError`。

IRIS-V1-RUNTIME-C005：`Object` 是普通对象行为的根运行时 Class。`Object` 提供如下指定的默认比较、真值性、缺失消息和零参数 `initialize` 行为。不得存在第二个根 Class（No second root Class MAY exist）。

IRIS-V1-RUNTIME-C006：下表是内置身份、默认哈希、可变性和开放性分类的规范：

| 内置类别     | 身份分类                                                                  | 默认哈希契约                                                                                                                           | 值状态可变性                                | 动态开放性                                                   |
| --------------------- | ---------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------- | ------------------------------------------------------------------ |
|                       |                                                                                          |                                                                                                                                                 |                                                          |                                                                    |
| `nil`               | 带身份单例                                                               | 规范稳定的单例哈希                                                                                                             | 单例状态固定                                 | 其 Class 可以组合 Modules 并替换兼容行为      |
| `false`, `true`   | 带身份单例                                                              | 规范稳定的单例哈希                                                                                                             | 单例状态固定                                 | `Bool` Class 可以组合 Modules 并替换兼容行为 |
| `Integer` 值    | 无身份不可变值                                                           | 规范稳定的数值哈希；无效 NaN 情况不适用                                                                            | 数学值固定                              | `Integer` Class 在安全边界内行为开放        |
| `Float32` 值    | 无身份不可变值                                                           | 规范稳定的数值哈希；NaN 会引发 `InvalidKeyError`                                                                         | IEEE-754 交换位固定                      | `Float32` Class 在安全边界内行为开放        |
| `Float64` 值    | 无身份不可变值                                                           | 规范稳定的数值哈希；NaN 会引发 `InvalidKeyError`                                                                         | IEEE-754 交换位固定                      | `Float64` Class 在安全边界内行为开放        |
| Class 对象         | 带身份定义对象                                                      | 运行时本地身份哈希，除非 Method 替换它                                                                                         | Object 身份在兼容 open 期间保持不变         | 逻辑 Class 以活动修订保持同一身份             |
| Module 对象        | 带身份定义对象                                                      | 运行时本地身份哈希，除非 Method 替换它                                                                                         | Object 身份在兼容 open 期间保持不变         | Module 以活动修订保持同一身份                    |
| Contract 对象      | 带身份定义对象                                                      | 运行时本地身份哈希，除非 Method 替换它                                                                                         | 静态要求在声明后固定          | Contract 不能被 open                                         |
| Contract 视图        | 无身份不可变能力值                                                | 派生自接收者哈希和 Contract Type 身份                                                                                           | 接收者关系和 Contract 身份固定        | 视图派发通过 `..` 显式进行                            |
| Method 对象        | 带身份可调用定义对象                                             | 运行时本地身份哈希，除非 Method 替换它                                                                                         | Method 身份和主体保留                    | 槽位替换会创建新的 Method                              |
| BoundMethod 对象   | 带身份可调用对象                                                        | 运行时本地身份哈希，除非 Method 替换它                                                                                         | 捕获的接收者关系和 Method 身份固定 | 调用会重新验证当前所有者成员关系                    |
| Closure 对象       | 带身份可调用对象                                                        | 运行时本地身份哈希，除非 Method 替换它                                                                                         | 捕获的环境绑定按 Closure 规则固定   | 每次求值都会创建不同身份                        |
| ClassRevision 对象 | 带身份、运行时拥有的元数据对象                                          | 运行时本地身份哈希                                                                                                                     | 只读元数据和代码引用固定          | 被取代的修订在被引用时可以保持 pinned                |
| String                | 无身份不可变文本值                                                       | 由集合章节定义的稳定文本哈希                                                                                             | 标量序列固定                                 | Class 行为可在安全边界内开放                    |
| MutableString         | 带身份可变文本对象                                                     | 运行时本地身份哈希，除非 Method 替换它                                                                                         | 文本缓冲区可通过 Methods 变更                   | Class 行为可在安全边界内开放                    |
| Symbol                | 无身份不可变驻留名称值                                              | 由集合章节定义的稳定 Symbol 哈希                                                                                           | Symbol 文本固定                                     | Class 行为可在安全边界内开放                    |
| Bytes                 | 无身份不可变二进制值                                                     | 由集合章节定义的稳定字节哈希                                                                                            | 字节序列固定                                   | Class 行为可在安全边界内开放                    |
| ByteArray             | 带身份可变二进制对象                                                   | 运行时本地身份哈希，除非 Method 替换它                                                                                         | 字节序列可通过 Methods 变更                 | Class 行为可在安全边界内开放                    |
| Tuple                 | 无身份不可变有序值                                                    | 所有元素均可哈希时使用稳定元素哈希                                                                                              | 元素序列固定                                | Class 行为可在安全边界内开放                    |
| Array                 | 带身份可变有序对象                                                  | 运行时本地身份哈希，除非 Method 替换它                                                                                         | 元素可通过 Methods 变更                      | Class 行为可在安全边界内开放                    |
| Hash                  | 带身份可变映射对象                                                  | 运行时本地身份哈希，除非 Method 替换它                                                                                         | 条目可通过 Methods 变更                       | Class 行为可在安全边界内开放                    |
| Range                 | 无身份不可变区间值                                                   | 端点可哈希时使用稳定端点哈希                                                                                                | 端点和包含性固定                      | Class 行为可在安全边界内开放                    |
| Regex                 | 无身份不可变模式值                                                    | 由集合章节定义的稳定模式哈希                                                                                          | 模式和标志固定                              | Class 行为可在安全边界内开放                    |
| Type 对象          | 不同于 Class、Module 和 Contract 对象的驻留带身份 Type 对象 | 对可发布的命名和复合 Type 稳定；对局部匿名身份为运行时本地，更细派生由类型章节负责 | 规范 Type 身份固定                         | 反射可见性遵循后续章节                       |

IRIS-V1-RUNTIME-C007：无身份不可变值 MUST NOT 获得可观察的接收者特定状态。它们 MUST NOT 创建、重新创建或保留未声明的动态 ivar；任何会创建这种状态的赋值 MUST 引发 `InstanceStateError`。

IRIS-V1-RUNTIME-C008：承载身份的普通对象 MAY 根据其 Class 策略、声明的存储和有效的 `MetaCapabilities` 携带接收者特定的状态。垃圾收集、对象移动、分配策略、实习、拆箱或重新装箱 MUST NOT 更改任何可观察的身份或无身份分类。

## 逻辑 Class 与活动修订

IRIS-V1-RUNTIME-C009：逻辑 Class 是 Iris 程序可见的稳定身份。逻辑 Class 在任一观察点恰好有一个当前活动修订，并且普通实例发送、Contract 限定查找、MRO 查找和运行时祖先检查 MUST 查询该当前活动修订，除非本章说明已进入帧或保留的可调用值保留捕获的 Method 主体。

IRIS-V1-RUNTIME-C010：重新打开现有 Class MUST 通过验证后发布新的活动修订来改变同一个逻辑 Class 身份。重新打开前后捕获的引用 MUST 满足 `same?`，因为逻辑 Class 对象未改变。将名称重新绑定到不同 Class 对象不是 reopen，也不是命名 Class 声明的 v1 操作。

IRIS-V1-RUNTIME-C011：每个实例都引用其稳定的逻辑 Class。实例 MUST NOT 为普通的未来发送永久选择旧活动修订。结构提交成功后，现有实例和未来实例会在后续普通发送和祖先检查中使用该逻辑 Class 的新活动修订。

IRIS-V1-RUNTIME-C012：已进入的 Method 帧执行调用进入时选定的 Method 身份和主体。进入后替换或移除槽位 MUST NOT 改变已经运行帧的主体。未来普通查找使用新的活动修订和选定槽位状态。

IRIS-V1-RUNTIME-C013：Method 替换 MUST 创建新的带身份 Method 对象并将其安装到槽位中。先前捕获的 Method 对象在被引用期间保留其身份和旧主体，并且在接收者绑定验证约束下，可继续通过反射显式调用。

IRIS-V1-RUNTIME-C014：Method 对象 MUST 保留其词法所有者 Class 或 Module 的身份。当保留的旧 Method 执行 `super` 时，查找 MUST 在调用时使用接收者当前版本化 MRO，并从该词法所有者之后继续。如果当前 MRO 中不存在该词法所有者，则执行 MUST 引发 `InvalidSuperError`。

IRIS-V1-RUNTIME-C015：反射式 Method 调用 MUST 在进入时验证接收者的当前 MRO 包含该 Method 的词法所有者。Class 所有者要求实例的当前 MRO 包含该 Class。Module 所有者要求接收者 MRO 中包含该 Module。即使主体路径不会访问状态或执行 `super`，验证失败也 MUST 引发 `MethodBindingError`。

IRIS-V1-RUNTIME-C016：`ClassRevision` 是只读的承载身份的运行时元数据对象。它 MUST 至少公开所有者 Class、每个 Class 修订号、静态骨架引用、运行时超类、MRO、模块、成员和属性元数据、布局或形状描述符、包或源修订元数据，并通过后续反射规则提交元数据。用户代码 MUST NOT 变更或重新激活 `ClassRevision`。

IRIS-V1-RUNTIME-C017：起源 Class 修订号为 `1`。影响该逻辑 Class 的每次成功结构发布 MUST 获得下一个每 Class 整数。失败或回滚的候选 MUST NOT 消耗每 Class 修订号。

IRIS-V1-RUNTIME-C018：每个成功的结构事务组 MUST 获得一个运行时范围内严格递增的 `commit_id`，由该组中所有 Class 和 Module 修订共享。`commit_id` 是运行时本地的诊断和审计排序元数据。它 MUST NOT 参与名义 Type 身份或跨运行时稳定哈希。

IRIS-V1-RUNTIME-C019：由 Method 帧、保留的 Method 对象、原生引用、工具或用户持有的 `ClassRevision` 引用 pinned 的被取代修订，MUST 保留这些引用所需的可执行元数据。一旦不再被引用且非活动，重型修订状态 MAY 被回收，而轻量审计记录 MAY 保留。

IRIS-V1-RUNTIME-C020：回滚操作 MUST 发布一个新的已验证修订，而不是直接重新激活历史修订。回滚 MUST 在发布前验证当前静态骨架、Contract 要求、MRO、可见性、原生义务和安全约束。缺失或不匹配的历史产物 MUST 引发 `RevisionArtifactUnavailableError`，且不发布任何内容。

IRIS-V1-RUNTIME-C021：同主版本包或 Class 演化 MUST NOT 降级当前静态骨架。新增的兼容普通 API MAY 保留，但移除所需当前成员、Contracts、泛型元数、可见性义务、原生义务或超类边界 MUST 验证失败。

IRIS-V1-RUNTIME-C022：以下修订转换表是规范性的：

| 事件                                | 事件期间的全局实例发送                       | 成功后的新发送                  | 已进入帧                                | 保留的 Method 或 BoundMethod                          |
| ------------------------------------ | -------------------------------------------------------- | ---------------------------------------- | --------------------------------------------- | ------------------------------------------------------- |
| open 候选开始                | 使用已发布活动修订                            | 提交前不变                   | 继续选定主体                        | 继续捕获的 Method 身份                       |
| 候选主体读取元数据        | 事务只通过 meta/reflection 读取候选 | 提交前不变                   | 继续选定主体                        | 继续捕获的 Method 身份                       |
| 候选验证失败或引发异常 | 已发布活动修订保持不变                        | 已发布活动修订保持不变        | 继续选定主体                        | 继续捕获的 Method 身份                       |
| 候选提交                    | 不暴露部分候选派发                 | 使用新的活动修订                  | 如果已进入则继续旧选定主体 | 当前所有者重新验证后调用捕获的 Method |
| 槽位替换提交             | 已进入帧继续使用旧槽位                      | 未来查找选择新的 Method         | 继续旧主体                             | 捕获的旧 Method 仍可显式调用        |
|Module MRO 更改                   | 当前活动 MRO 直到提交                          | 未来查找使用新的 MRO               | 当前帧保留选定的主体             | `super` 使用调用时当前 MRO              |
| 回滚提交                     | 不会原地重新激活历史修订           | 未来查找使用新的回滚修订 | 继续选定主体                        | 捕获的 Method 规则仍然适用                       |

IRIS-V1-RUNTIME-N001：实现说明：内联缓存、原始数字路径和编译代码可能会记住修订、Method 标识、查找版本和表示假设，仅作为受保护的依赖项。在稍后受影响的发送观察到陈旧行为之前，更改的依赖项必须失效、回退或取消优化。

## 派发与选择器命名空间

IRIS-V1-RUNTIME-C023：普通消息标识是完整的选择器名称加上其普通调用形式。静态注释、推断类型、联合分支、泛型参数、预期返回类型或声明顺序 MUST NOT 选择不同的普通选择器或重载。

IRIS-V1-RUNTIME-C024：Iris v1 MUST NOT 支持 Method 重载。一个 Class 修订版或 MRO 槽命名空间中的一个完整选择器最多映射到一个 Method。重新定义相同的完整选择器属于替换，需要适用的`override`授权；它绝不是一个过载集。

IRIS-V1-RUNTIME-C025：普通选择器查找 MUST 按语法和控制流章节对接收者和实参求值，解析接收者的当前活动修订和 MRO，应用可见性以及元数/类型检查，并调用选定的 Method。已有选择器的形状不匹配 MUST 因元数不匹配引发 `ArgumentError`，或因运行时实参或块 Contract 失败引发 `TypeError`。它们 MUST NOT 调用 `method_missing`。

IRIS-V1-RUNTIME-C026：命名中缀调用 `receiver selector argument` MUST 与单参数方法的 `receiver.selector(argument)` 相同的可观察普通发送。解析从不查阅 Method 表。 Method 声明 MUST NOT 更改命名的中缀优先级、关联性或解析有效性。

IRIS-V1-RUNTIME-C027：语法中可重载的符号运算符 MUST 是普通 Method 发送。对于 `a OP b`，运行时 MUST 先求值 `a`，再求值 `b`，然后在 `a` 的运行时 Class 和活动修订上解析并调用选择器 `OP`，并以 `b` 作为实参。决定可接受操作数和结果类型的是内置运算符 Method 契约，而不是全局提升表。

IRIS-V1-RUNTIME-C028：`!`、`&&`、`||`、`&&=` 和 `||=` MUST 是不可重载的核心控制流形式，并按 IRIS-V1-RUNTIME-C093 至 IRIS-V1-RUNTIME-C098 的规定使用 `to_bool`。它们 MUST NOT 安装为 Class 或 Module Method 选择器。`&`、`|`、`^` 和 `~` 仍然是可重载的运算符消息。

IRIS-V1-RUNTIME-C029：`same?` 是不可重载的原始身份比较。它 MUST 对左操作数求值一次、对右操作数求值一次，只接受带身份操作数，绕过 Method 查找、`<=>`、`==`、代理和用户替换，并返回 `Bool`。如果任一操作数无身份，则 MUST 引发 `IdentityError`。

IRIS-V1-RUNTIME-C030：Contract 限定的派发具有与普通派发不同的选择器命名空间。源表单 `(value as ContractType)..member(args...)` 或存储的 Contract 视图后跟 `..member(args...)` MUST 选择包含 Contract 身份和成员选择器的槽身份。普通`value.member(args...)`和`view.member(args...)` MUST仍然是不合格的普通发送。

IRIS-V1-RUNTIME-C031：Contract 视图是不可变的无身份能力值，承载底层接收者关系和 Contract 身份或一致性。重复视图创建 MAY 不分配任何内容。 `same?` 在 Contract 视图上 MUST 引发 `IdentityError`。

IRIS-V1-RUNTIME-C032：内置 Contract 视图对身份承载接收方的平等性 MUST 需要相同的接收方身份和相同的 Contract 身份。内置 Contract 视图对无身份接收方的平等性 MUST 要求在当前平等行为下接收方相等，并且 Contract 身份相同。

IRIS-V1-RUNTIME-C033：缺少普通选择器 MUST 调用 IRIS-V1-RUNTIME-C099 和 IRIS-V1-RUNTIME-C100 中指定的 `method_missing`。缺少合格的 Contract 槽位 MUST 引发 `ContractDispatchError`，并且 MUST NOT 调用普通 `method_missing`，回退到非合格查找，搜索另一个 Contract，或使用动态Contract-缺少钩子。

IRIS-V1-RUNTIME-C034：可见性拒绝 MUST 绕过 `method_missing`。如果查找找到 Method 或属性选择器，但调用者缺乏私有或受保护的访问权限，则直接派发 MUST 引发 `MethodVisibilityError`。

IRIS-V1-RUNTIME-C035：以下派发命名空间表是规范的：

| 源形式                       | 选择器命名空间                                  | 使用的动态所有者                                     | 缺失行为                             | 允许重载 |
| --------------------------------- | --------------------------------------------------- | ------------------------------------------------------ | -------------------------------------------- | ---------------- |
|`value.member(args...)`         | 普通选择器`member`                         | 接收者逻辑 Class 当前有效版本和 MRO | `method_missing(:member, args, block)`     | 否               |
| `value selector arg`            | 普通选择器`selector`                       | 接收者逻辑 Class 当前有效版本和 MRO | `method_missing(:selector, [arg], nil)`    | 否               |
| `value + arg`                   | 普通选择器`+`                              | 接收者逻辑 Class 当前有效版本和 MRO | `method_missing(:+, [arg], nil)`| 否               |
|`value.name`                    | 普通属性 getter 选择器`name`           | 接收者逻辑 Class 当前有效版本和 MRO | `method_missing(:name, [], nil)`           | 否               |
| `value.name = rhs`              | 普通属性设置选择器`name=`          | 接收者逻辑 Class 当前有效版本和 MRO | `method_missing(:name=, [rhs], nil)`       | 否               |
| `(value as C)..member(args...)` | 合格 Contract 槽位`C::member`                | 该 Contract 槽位的接收者 Class 实现    | `ContractDispatchError`| 否               |
|`view..member(args...)`         | 从 Contract 身份来看合格的 Contract 槽位 | 该 Contract 槽位的接收者 Class 实现    | `ContractDispatchError`                    | 否               |
| `view.member(args...)`          | 普通选择器`member`                         | 底层接收者普通派发                  | `method_missing(:member, args, block)`     | 否               |
| `same?` 或 `.same?`           | 原始身份比较                       | 无                                                   | `IdentityError` 用于无身份操作数| 否               |

## Method、BoundMethod、Closure 与 Class 对象

IRIS-V1-RUNTIME-C036：Method 是安装在 Class、Module、Contract 限定槽实现、类对象单例 Method 表或相关接收者表面中的带有身份的可调用定义对象。 Method 默认相等 MUST 通过根行为仅识别身份。

IRIS-V1-RUNTIME-C037：Method 别名 MUST 创建引用相同 Method 标识的另一个槽位。删除槽位 MUST 只删除当前所有者本地槽位，并且可能暴露祖先实现。取消定义槽位 MUST 安装一个阻止祖先查找的墓碑；普通调用将被视为缺席，并可能调用 `method_missing`。

IRIS-V1-RUNTIME-C038：读取或绑定实例 Method MUST 创建一个 BoundMethod，该 BoundMethod 捕获调用接收者关系以及在绑定时解析的确切 Method 身份。它 MUST NOT 捕获历史 MRO 快照。稍后更换槽位 MUST NOT 重定向 BoundMethod。

IRIS-V1-RUNTIME-C039：每次 BoundMethod 调用 MUST 重新验证接收者的当前 Class 和 MRO 包含所捕获 Method 的词法所有者。如果不存在，调用 MUST 引发 `MethodBindingError`。如果主体执行 `super`，`super` MUST 按 IRIS-V1-RUNTIME-C014 使用调用时的当前 MRO。

IRIS-V1-RUNTIME-C040：每次对 `obj.method` 进行读取或绑定求值，MUST 创建一个不同的带身份 BoundMethod，即使接收者和解析出的 Method 未改变。因此，`obj.method same? obj.method` 是 `false`，因为两个操作数是两次求值得到的绑定。保存的 BoundMethod 引用与自身是 `same?`。

IRIS-V1-RUNTIME-C041：BoundMethod 默认相等 MUST 仅身份。默认情况下，MUST NOT 在结构上比较捕获的接收者和 Method 组件。反射 MAY 公开这些组件，以便在以后的反射规则下进行显式比较。

IRIS-V1-RUNTIME-C042：每次对 Closure 表达式求值，MUST 创建一个新的带身份 Closure 对象，并带有自己的捕获环境。Closure 默认相等性 MUST 仅按身份。Iris MUST NOT 对可执行代码或捕获环境进行结构比较。

IRIS-V1-RUNTIME-C043：Class 对象本身就是一个身份承载对象，也是 `Class` 的实例。 `class fun` 声明 MUST 在该特定 Class 对象上安装单例 Method。

IRIS-V1-RUNTIME-C044：向 Class 对象发送消息 MUST 首先搜索该对象的单例方法，然后按顺序搜索相应的逻辑运行时超类 Class 对象，然后是 `Class` 提供的普通实例方法。 `super(...)` 位于 Class 对象单例 Method MUST 中，继续沿着该 Class 对象链。 V1 MUST NOT 公开单独的元类声明语法。

IRIS-V1-RUNTIME-C045：Class-对象原始`@x`状态属于Class对象接收者本身。子类 Class 对象具有其自己的 Class 对象 `@x` 状态。此存储不同于以 `@@` 命名的层次结构类变量。

## Module 组合与 MRO

IRIS-V1-RUNTIME-C046：Class 具有单个 Class 继承加上 Module 组合。查找MUST从逻辑Class自己的活动成员开始，然后以相反的组合顺序组合模块，然后递归运行时超类MRO。

IRIS-V1-RUNTIME-C047：标头 `mixin A, B` MUST 从左到右应用并在嵌套扩展之前生成查找顺序 `Class, B, A, Super...`。如果两个边都存在，则一个已提交事务 MUST 中的顺序动态 `include(A); include(B)` 会产生相同的相对顺序。

IRIS-V1-RUNTIME-C048：嵌套的 Module 组合 MUST 通过最接近 Class 出现处的封闭 Module 标识进行确定性扩展和重复数据删除。重新包含已存在的封闭 Module MUST 是幂等的：它不会创建重复项，不会移动或重新排序现有的事件，并且不会导致结构修订，除非其他边缘元数据发生更改。

IRIS-V1-RUNTIME-C049：V1 不提供专用的 Module 重新排序 API。程序 MAY 仅通过删除和包含一个open 事务中的模块来重新排序组合。候选验证、原子提交、`modules` 功能、Module `Self` 约束、MRO 有效性、Contract 实现和组合边缘授权 MUST 全部重新应用。

IRIS-V1-RUNTIME-C050：默认情况下，Module 组合不授予私有 Method 访问权限。从 Module 方法进行私有访问托管 Class 私有方法需要在静态或动态组合边缘进行显式授权，并且范围仅限于该主机逻辑 Class、封闭的 Module 身份和边缘修订版。

IRIS-V1-RUNTIME-C051：在主机接收者 MAY 上执行的组合 Module Method 根据原始 ivar 规则在当前接收者上读取和写入不合格的原始 ivar，无论组合边缘是否授予私有 Method 访问权限。私有授权仅影响私有选择器。

IRIS-V1-RUNTIME-C052：删除 Module 边缘 MUST 以原子方式撤销该边缘的私有授权并重新计算 MRO、有效 `MetaCapabilities`、Contract 满意度和查找版本。 Module 全局状态 MUST NOT 保留主机私有访问权限。

IRIS-V1-RUNTIME-C053：以下MRO表是规范的：

| 组合状态                                | 进入超类递归前的查找顺序   | 修订影响                                        |
| ------------------------------------------------ | ---------------------------------------------------- | ------------------------------------------------------ |
| `class C extends S {}`                         | `C, S...`                                          | 原始版本不包含 Module 边缘               |
| `class C extends S mixin A, B {}`              | `C, B, A, S...`                                    | 静态 mixin 边缘位于原始版本中|
|`include(A); include(B)` 来自之前没有的模块 | `C, B, A, S...`                                    | 如果事务成功，则提交一个动态修订 |
| `B mixin A`； `C mixin A, B`                  | `C, B, A, S...`                                    | Nested`A` 在最接近的出现位置进行重复数据删除      |
| 重新包含已存在的`A`                  | 现有订单不变                             | 除非边缘元数据发生变化，否则不会进行结构修改    |
| 删除`A`，然后包含`A private`          | `A` 出现在带有新元数据的新边缘位置 | 如果交易成功则进行新修订|

## 构造生命周期

IRIS-V1-RUNTIME-C054：标准结构是 Class 对象 Method `new`。 `A.new(args...)` MUST 分配一个完整的内存安全实例，执行 IRIS-V1-RUNTIME-C057 中指定的存储属性初始值设定项，调用最终的动态 `initialize(args...)`，根据可调用/类型章节验证或忽略其 `Nil` 结果，并在成功时返回实例。

IRIS-V1-RUNTIME-C055：`Object` MUST 提供默认的零参数 `initialize`。 `initialize` 默认是私有的。 Iris v1 MUST NOT 提供 Class 命名的构造函数语法或构造函数重载。

IRIS-V1-RUNTIME-C056：`A.new(args...)` MUST 快照 `A` 在构造开始时对分配、布局、存储属性初始化和初始 `initialize` 派发的活动修订。如果在构造完成之前提交了新的活动修订，则构造将使用捕获的构造修订继续进行。成功后，实例将引用逻辑 Class `A`，之后的普通发送将使用 `A` 的当前活动修订版。

IRIS-V1-RUNTIME-C057：存储属性初始值设定项 MUST 在显式 `initialize` Method 之前执行超类到子类和声明顺序。父`initialize`逻辑仅通过显式`super(...)`运行；运行时 MUST NOT 自动调用每个祖先初始化程序。

IRIS-V1-RUNTIME-C058：如果 `initialize` 或存储的属性初始值设定项引发，则 `new` MUST 传播异常并且不返回任何实例。如果 `self` 转义，任何分配的对象仍然是内存安全的和普通的。运行时 MUST NOT 中毒、撤销、扫描、自动关闭、运行特殊终结、撤消状态或回滚外部效果。

IRIS-V1-RUNTIME-C059：Iris v1 不提供自动构建和修订协调。它 MUST NOT 提供 `new_current`、`new_checked`、构造函数重试、修订锁定、自动构造后重新初始化、自动实例状态迁移或隐藏构造函数副作用补偿。

IRIS-V1-RUNTIME-C060：传统的 `migrate_revision(from: ClassRevision, to: ClassRevision) -> Nil` Method MAY 由应用程序在其跟踪的对象上实现和显式调用。运行时打开或提交 MUST NOT 枚举实时实例，自动调用此 Method，或提供 Class 范围的实时实例枚举。

## 属性、原始 ivar 与类变量

IRIS-V1-RUNTIME-C061：属性是显式 Method 选择器。 getter 声明 `property fun name() -> T` 定义选择器 `name`。 setter 声明 `property fun name=(value: T) -> R` 定义选择器 `name=`。属性读取 `obj.name` MUST 发送零参数 getter 选择器，赋值 `obj.name = value` MUST 发送一参数 setter 选择器。

IRIS-V1-RUNTIME-C062：属性设置器 MAY 返回其声明和运行时合约允许的任何值。 `obj.name = value` MUST 产生设置器 Method 结果，不一定是右侧值或 `nil`。

IRIS-V1-RUNTIME-C063：赋值 MUST 是右关联的。在`a.x = b.y = v`中，内部setter首先执行，外部setter接收内部setter的实际结果。静态和动态检查 MUST 使用实际的表达式结果。

IRIS-V1-RUNTIME-C064：绑定、类变量、共享和原始 ivar 赋值 MUST 生成实际存储值。属性和索引赋值 MUST 产生 setter Method 结果。复合赋值 MUST 对目标位置和 RHS 求值一次，读取目标一次，发送普通运算符 Method，通过目标的正常写入路径写入一次，并产生写入操作的结果。

IRIS-V1-RUNTIME-C065：存储属性简写 MUST 创建修订级别类型化实例存储以及显式访问器 Method 选择器。生成的访问器主体 MAY 通过授权变更可以兼容地替换为 `property fun` 声明。 Contract 暴露是通过属性方法，从不原始存储。

IRIS-V1-RUNTIME-C066：原始 `@x` 语法 MUST 访问当前接收者上名为 `@x` 的单个槽位。槽位标识为 `(receiver, name)`，未声明 Class 或 Module。父类、子类和组合 Module 在同一接收者上执行的方法会看到并写入相同的槽。

IRIS-V1-RUNTIME-C067：源语法 MUST NOT 允许 `other.@x` 或 `obj.@@x` 或 `A.@@x`。对实现状态 MUST 的跨实例和外部访问使用后面章节定义的属性、方法或权限检查反射。

IRIS-V1-RUNTIME-C068：读取未声明的动态原始 ivar MUST 具有静态类型 `Dynamic<Object>`。如果接收者缺少槽位，则读取 MUST 返回 `nil`，并且 MUST NOT 创建实例状态。

IRIS-V1-RUNTIME-C069：当有效 `instance_state` 存在时，对带身份对象上不存在的未声明原始 ivar 赋值 MAY 创建该槽位。当有效 `instance_state` 被拒绝时，对不存在的未声明原始 ivar 的首次赋值（包括删除后的重新创建）MUST 引发 `InstanceStateError`。现有动态 ivar 保持可读可写，除非另一条规则禁止写入。

IRIS-V1-RUNTIME-C070：被拒绝 `instance_state` MUST 仍然允许删除现有的未声明的动态 ivar 进行清理。删除 MUST NOT 删除其他实例的状态，并且 MUST NOT 更改 Class 范围的形状。后来对缺席名称的分配是新的扩展，并且 MUST 服从 IRIS-V1-RUNTIME-C069。

IRIS-V1-RUNTIME-C071：未声明的动态 ivar MUST 没有固定的值类型 Contract。现有未声明动态 ivar MAY 保存任何 `Object` 值，包括 `nil`，后续写入 MAY 改变运行时值类型。需要稳定类型的程序 MUST 使用已声明的类型化属性或存储。

IRIS-V1-RUNTIME-C072：在实例 Method 中创建的 Closure MUST 捕获其当前接收者，并且 MAY 在逃逸后继续读取和写入该接收者的原始 ivar。捕获的接收者和私有能力 MUST NOT 通过源代码、反射、Dynamic、原生或 Host API 重新绑定；它们只属于该 Closure 的词法主体。

IRIS-V1-RUNTIME-C073：`@@name` 表示锚定到逻辑 Class 的已声明层次结构绑定单元。它不是 Class 对象的 ivar 或属性。声明 Class 及其子类层次结构根据词法 Class 变量规则共享该单元。

IRIS-V1-RUNTIME-C074：Class 变量必须 (MUST) 可从声明 Class 或其子类拥有的实例 Methods 和 Class Methods 中按词法访问。查找遵循该 Method 不可变的静态词法 Class 层次结构，并且 MUST NOT 随运行时超类修订变化而变化。公共访问 MUST 使用 Methods 或属性。

IRIS-V1-RUNTIME-C075：Class-对象原始ivars和层次结构类变量是不同的存储类别。 Class 对象 `A` MAY 具有独立于子类 Class 对象 `@x` 的 `B` 的普通接收者名称 `@x` 状态，而锚定到 `@@y` 的 `A` 通过层次结构共享，并且不能由 `B` 隐藏或重新声明。

IRIS-V1-RUNTIME-C076：以下存储表是规范的：

| 语法或声明      | 存储所有者                                               | 可见性表面                              | 创建权限                                                           | 赋值结果                     |
| -------------------------- | ----------------------------------------------------------- | ----------------------------------------------- | ---------------------------------------------------------------------------- | ------------------------------------- |
| `@x`|当前接收者槽位名为`@x`                           | 仅当前接收者实现代码       | `instance_state` 用于未声明的扩展或声明的存储规则       | 储值                          |
| `property fun x()`       | Method 选择器`x`                                        | 消息/属性 API                            | `property_set` 用于槽位，`property_body` 用于兼容主体更换 | getter 结果                         |
| `property fun x=(v)`     | Method 选择器`x=`                                       | 消息/属性 API|`property_set` 用于槽位，`property_body` 用于兼容主体更换 | setter 结果                         |
| 已存储`property x: T`    | 修订级类型化实例存储以及访问器        | 仅访问器方法                           | `property_set` 加 `shape`；原生存储还需要 `native`     | 属性赋值的 setter 结果 |
| Class-对象`@x`         | Class 对象接收者                                   | Class-对象实现代码                | Object 该 Class 对象的实例状态策略                           | 储值|
|`@@x`                    | 声明的层次结构绑定单元锚定到逻辑 Class | 词法授权的 Class 和子类方法 | 声明的存储规则和类状态功能                          | 储值                          |
| 共享/类属性槽 | Class/共享存储元数据                               | 方法或元 APIs                            | `property_set & class_state_set`；原生存储还需要 `native`  | setter 或元写入结果           |

## 可见性与 super

IRIS-V1-RUNTIME-C077：Class 实例方法、Class 对象单例方法、Module 方法、属性和模块主方法默认为私有，除非另有声明。可见性前缀是声明本地的，并且 MUST NOT 创建有状态的可见性部分。

IRIS-V1-RUNTIME-C078：私有 Method 选择器 MAY 仅从由声明逻辑 Class 和任何单独授权的模块进行词法授权的实现代码发送。子类代码、外部调用者、`Dynamic<T>` 和普通反射 MUST NOT 仅仅因为选择器存在而调用私有 Method。

IRIS-V1-RUNTIME-C079：由 Class `A` MAY 声明的受保护的 Method 从词法上属于 `A` 或名义子类的实现代码调用，接收者限制为当前 `self`或该允许的层次结构中的实例。外部呼叫者和 `Dynamic<T>` MUST NOT 绕过受保护的可见性。

IRIS-V1-RUNTIME-C080：受保护的属性 MUST 遵循受保护的 Method 可见性。原始 ivar 仍然是当前接收者槽位，并且不是继承的受保护字段。

IRIS-V1-RUNTIME-C081：`super(args..., key..., &block)` MUST 在接收者的当前 MRO 或 Class 对象单例 Method 链中的当前 Method 词汇所有者之后调用相同的完整选择器。裸 `super` 和隐式参数转发 MUST 是非法的。如果不存在后继者，则派发 MUST 引发 `NoSuperMethodError`，并且 MUST NOT 调用 `method_missing`。

IRIS-V1-RUNTIME-C082：合格的 Contract Method `super` MUST 在显式合格槽位和该 Contract 槽位的当前 MRO 规则内继续。它 MUST NOT 回退到普通选择器查找。

## 相等性、排序、哈希与身份

IRIS-V1-RUNTIME-C083：根 `Object` MUST 提供 `<=> (other: Object) -> Integer?`，其默认实现对每个操作数返回 `nil`。默认情况下，不得暴露全局地址顺序或对象 ID 顺序。

IRIS-V1-RUNTIME-C084：默认比较 Methods `==`、`!=`、`<`、`<=`、`>` 和 `>=` MUST 是普通的一参数 Method 槽位。它们未经修改的默认实现源自当前可见的 `<=>` 响应。`<=>` 结果 `Integer(-1)`、`Integer(0)`、`Integer(1)` 和 `nil` 映射到比较结果如下：零表示相等，负一表示小于，正一表示大于，`nil` 表示无序且 `== false`、`!= true`，四个有序比较全部为 false。

IRIS-V1-RUNTIME-C085：默认委托比较方法 MUST 仅从 `<=>` 接受精确的 `Integer(-1)`、`Integer(0)`、`Integer(1)` 或 `nil`。任何其他整数或类型 MUST 根据是否违反动态比较协议或声明的返回类型来引发 `ComparisonContractError` 或 `TypeError`。

IRIS-V1-RUNTIME-C086：对于 `==` 槽位仍使用根/默认实现的带身份对象，相等性 MUST 首先测试两个引用是否表示同一对象。如果是，则不查询 `<=>` 而返回 `true`。否则，它调用当前可见的 `<=>`，并且仅当结果是精确的 `Integer(0)` 时返回 `true`。当默认 `!=` 槽位仍保持安装时，它与相等槽位互补。

IRIS-V1-RUNTIME-C087：用户MAY通过授权动态变异独立替换`<=>`或任何比较槽位。替换 `<=>` 会影响仍然默认的委托比较方法，但 MUST NOT 会覆盖单独替换的比较槽。用户承担最终的平等、排序和哈希一致性的责任。

IRIS-V1-RUNTIME-C088：普通身份承载对象和身份承载运行时或元对象 MAY 是 Hash 密钥，并在 `0..2^64-1` 中使用运行时稳定的身份哈希。同一对象 MUST 在其整个生命周期和 GC 移动过程中保留该哈希值。身份哈希 MUST NOT 在进程间保持稳定或暴露原始内存地址。

IRIS-V1-RUNTIME-C089：`nil`、`false` 和 `true` 是有效的 Hash 密钥，MUST 使用规范稳定的单例哈希覆盖通用身份哈希。数值 MUST 使用规范稳定的数字哈希值。 Hash 表 MAY 秘密地将公共哈希与运行时/容器种子重新混合，但公共 `hash` 结果 MUST 保持指定，而内置 Method 保持选中状态。

IRIS-V1-RUNTIME-C090：Contract-查看公共哈希 MUST 使用 BLAKE3 派生密钥模式以及精确的 ASCII 上下文 `Iris Language v1 contract view hash` 和输入 `receiver_public_hash_u64_le || contract_type_hash_u64_le`。摘要缩减 MUST 使用前八个摘要字节作为无符号小端字节序。接收者哈希故障 MUST 传播。

IRIS-V1-RUNTIME-C091：`Bool#<=>(other: Object) -> Integer?` MUST 定义总计 Bool 顺序：相等的单例值返回 `Integer(0)`，`false <=> true` 返回 `Integer(-1)`、`true <=> false`返回 `Integer(1)`，非 Bool 操作数返回 `nil`。 Bool MUST 仍然与数字类型不同，并且 MUST NOT 与 `Integer(0)` 或 `Integer(1)` 相等或顺序。

IRIS-V1-RUNTIME-C092：`nil#<=>(other: Object) -> Integer?` MUST 仅当 `other` 是同一个 `nil` 单例时返回 `Integer(0)`，并且对每个非 nil 值返回 `nil`。`nil` MUST NOT 是全局最小值或最大值。

## 真值性与缺失消息

IRIS-V1-RUNTIME-C093：真值性是动态`to_bool() -> Bool`协议。条件、逻辑非、逻辑运算符和逻辑赋值 MUST 对操作数求值一次，发送 `to_bool` 一次，需要实际的 `Bool`，并且 MUST NOT 递归地转换结果。

IRIS-V1-RUNTIME-C094：根 `Object` 最初提供 `to_bool() -> Bool` 返回 `true`。 `Nil` 返回 `false`。 `Bool` 返回自身，因此 `false.to_bool()` 是 `false`，`true.to_bool()` 是 `true`。

IRIS-V1-RUNTIME-C095：如果 `to_bool` 引发异常，该异常 MUST 原样传播。如果它正常返回非 Bool，运行时 MUST 引发 `TypeContractError`。依赖真值测试的条件体、RHS 表达式和逻辑赋值写回 MUST NOT 在真值测试失败后运行。

IRIS-V1-RUNTIME-C096：如果允许移除 Method 后 `to_bool` 缺失，真值测试 MUST 调用 `method_missing(:to_bool, [], nil)` 一次。其结果 MUST 是 `Bool`，否则引发 `TypeContractError`。缺失或递归 fallback MUST 以普通消息缺失失败终止，并且 MUST NOT 对 fallback 结果递归执行真值测试。

IRIS-V1-RUNTIME-C097： `!x` MUST 发送 `x.to_bool()` 一次并返回 Bool 否定。 `a && b` 必须 (MUST) 测试 `a`；当 false 时，它返回原始 `a` 而不评估 `b`，当 true 时，它评估并返回 `b`。 `a || b` 必须 (MUST) 测试 `a`；当 true 时，它返回原始 `a`，当 false 时，它评估并返回 `b`。

IRIS-V1-RUNTIME-C098：`target &&= rhs` 和 `target ||= rhs` 是控制流分配形式，而不是 Method 选择器。目标位置 MUST 被评估并读取一次。仅当当前值为真时，`&&=` MUST 评估并写入 RHS。仅当当前值为假时，`||=` MUST 评估并写入 RHS。无写路径产生当前值；写路径产生正常的写回结果。

IRIS-V1-RUNTIME-C099：根 `Object` MUST 声明普通动态可替换 `method_missing(selector: Symbol, arguments: Array<Object>, block: Closure?) -> Object`。运行时 MUST 在普通选择器查找真正失败后调用它，传递精确选择器 Symbol、不可变或快照位置参数 Array 以及尾部块或 `nil`。

IRIS-V1-RUNTIME-C100：默认的 `method_missing` 实现 MUST 使用接收者 Class 或活动版本、选择器、元数、源位置和可见性上下文提升 `MessageNotFoundError`。运行时 MUST NOT 递归地重新输入 `method_missing` 以查找缺失的 `method_missing` 本身。

## 内置数值模型

IRIS-V1-RUNTIME-C101：`Integer` 在语义上是任意精度。隐藏的立即或固定宽度表示和 BigInt 提升 MUST NOT 可观察为不同的 Iris 类型、身份、派发行为、相等行为或哈希行为。 Integer 算术 MUST 没有固定宽度溢出或换行语义。

IRIS-V1-RUNTIME-C102：`Float32` 和 `Float64` 是不同的内置语言类型，具有 IEEE-754 二进制 32 和二进制 64 交换语义。无后缀的源浮点文字生成 `Float64`； `f32`和`f64`后缀选择语法章节下相应的宽度。

IRIS-V1-RUNTIME-C103：内置 `Integer`、`Float32` 和 `Float64` 算术方法 MAY 接受来自其他内置数值类的操作数。每个接收者 Class 的 Method 合约定义转换、精度、异常值和结果类型。不存在全局数值提升规则、对称派发规则或用户定义的数值强制表。

IRIS-V1-RUNTIME-C104：对于接收者为 `Integer` 并且参数为 `Float32` 或 `Float64` 的内置算术，接收者 MUST 使用 IEEE-754 舍入转换为参数的浮动宽度，在该宽度执行，并返回相同的浮动类型。可以观察到精度损失。

IRIS-V1-RUNTIME-C105：对于接收者为 `Float32` 或 `Float64` 且参数为 `Integer` 的内置算术，参数 MUST 使用 IEEE-754 舍入转换为接收者的宽度，以该宽度执行，并且返回接收者的浮动类型。可以观察到精度损失。

IRIS-V1-RUNTIME-C106：`Float32` 与 `Float64` 之间的内置算术中，`Float32` 操作数 MUST 拓宽为 `Float64`，在 `Float64` 宽度执行，并返回 `Float64`，与操作数顺序无关。同宽浮点算术 MUST 返回相同宽度。

IRIS-V1-RUNTIME-C107：每个原始内置 `Float32` 和 `Float64` `+`、`-`、`*` 和 `/` 步骤 MUST 轮使用 IEEE-754 `roundTiesToEven` 确定的结果宽度。 Iris 不公开更改这些运算符的可变全局或线程本地舍入模式。

IRIS-V1-RUNTIME-C108：源级单独的乘法和加法运算 MUST 根据表达式树独立舍入。实现 MUST NOT 将 `a * b + c` 契约为融合乘加，如果这样做可能会改变可观察位或特殊值行为。显式 `mul_add` Method 是唯一的标准融合操作。

IRIS-V1-RUNTIME-C109：每个基元 `Float32` 或 `Float64` 结果 MUST 在成为后续操作的操作数之前，在语义上舍入到其声明的宽度。仅当后续每个观察结果都与使用已舍入目标宽度位完全等价时，MAY 使用更宽的物理寄存器。

IRIS-V1-RUNTIME-C110：浮点除以零 MUST 遵循非陷阱 IEEE-754 语义。非零有限除以正零或负零返回相应的有符号无穷大。浮点零除以浮点零返回 NaN。这些结果 MUST NOT 提高了 `DivisionByZeroError`。

IRIS-V1-RUNTIME-C111：有限算术或 Integer 到浮点转换超出有限范围的浮点溢出 MUST 在 IEEE-754 舍入下产生有符号无穷大。MUST NOT 引发 `FloatOverflowError` 或 `NumericConversionError`，并且 MUST NOT 饱和至最大有限值。

IRIS-V1-RUNTIME-C112：传播或创建 NaN MUST 的普通浮点操作仅保证在既定结果宽度上安静的 NaN。它们 MUST NOT 指定 NaN 有效负载、输入有效负载选择或符号位。需要精确位 MUST 的代码使用 `to_bits` 和 `from_bits`。

IRIS-V1-RUNTIME-C113：`Float32#to_bits()` MUST 返回 `0..2^32-1` 范围内的非负 `Integer`，`Float64#to_bits()` MUST 返回 `0..2^64-1` 范围内的非负 `Integer`。`Float32.from_bits(bits)` 和 `Float64.from_bits(bits)` MUST 只接受相应宽度的范围，并且 MUST 对负值或过大值引发 `RangeError`。

IRIS-V1-RUNTIME-C114：`from_bits(bits).to_bits() == bits` MUST 保留每个有效的 32 位或 64 位 IEEE 交换位模式，包括信令 NaN 编码。构造、存储、参数传递、返回、复制和 `to_bits` MUST NOT 自动静默或规范化信令 NaN。

IRIS-V1-RUNTIME-C115：`is_nan`、`is_signaling_nan`、`is_infinite`、`is_finite`、`is_normal`、`is_subnormal`、`is_zero` 和`sign_bit` MUST 是纯位分类方法。它们 MUST NOT 执行普通浮点算术、静默信令 NaN、更改有效负载或符号、引发浮点异常或更改 `to_bits()`。

IRIS-V1-RUNTIME-C116：`Float32.nan`、`Float32.infinity`、`Float64.nan` 和 `Float64.infinity` MUST 最初是相应 Class 对象上的仅获取属性。 getter 返回该宽度的规范值，而内置 getter 保持选中状态。负无穷大是由普通的一元否定产生的。不存在全局 `nan` 或 `inf` 文字。

IRIS-V1-RUNTIME-C117：内置特殊值 getter 是普通的动态可变 Class 对象属性 getter 方法。授权变更 MAY 替换或删除它们，并且替换后表达式不需要返回规范的 IEEE 值。授权变更MAY添加setter；添加 setter 仅创建分配消息行为，不会自动分配后备存储或与 getter 同步。

IRIS-V1-RUNTIME-C118：内置 `Float32.mul_add` 和 `Float64.mul_add` MUST 接受 `Integer`、`Float32` 和 `Float64` 作为乘数和加数。如果接收者、乘法器或加数为 `Float64`，则通用结果宽度为 `Float64`；否则它是接收者的宽度。 Integer 操作数作为精确数学整数参与融合表达式。精确的乘积加加值将四舍五入一次至公共结果宽度。

IRIS-V1-RUNTIME-C119：当融合被乘数之一为正无穷大或负无穷大且另一个为数字零时，内置 `mul_add` MUST 以公共结果宽度返回 NaN。当无限乘积和无限加数具有相反符号时，它也会返回 MUST NaN。这些情况 MUST NOT 引发 Iris 异常。

IRIS-V1-RUNTIME-C120: `Integer / Integer` MUST 执行实数除法并返回 `Float64`，包括可整除的操作数。两个操作数均根据 `Float64` 转换和 IEEE-754 规则进行解释，因此允许精度损失和有符号无穷大。

IRIS-V1-RUNTIME-C121：内置 `Integer.div` MUST 为每个非零整数除数计算向下取整除法。`/`、`div`、`mod` 或任何整数除法或余数操作中的除零 MUST 引发 `DivisionByZeroError`。

IRIS-V1-RUNTIME-C122：内置 `Integer.mod` MUST 对非零 `b` 定义为 `a - (a div b) * b`。它 MUST 满足 `a == (a div b) * b + (a mod b)` 和 `abs(a mod b) < abs(b)`。每个非零结果与除数 `b` 具有相同符号。

IRIS-V1-RUNTIME-C123：内置 `Integer ** Integer` MUST 针对每个非负指数返回任意精度 `Integer`。负整数指数 MUST 使用 `Float64` 倒数幂语义并返回 `Float64`。

IRIS-V1-RUNTIME-C124： `Integer(0) ** Integer(0)` MUST 提高 `DomainError`。 `Integer(0)` 为正整数指数，返回 `Integer(0)`。 `Integer(0)` 到负整数指数返回 `Float64(+Infinity)`，并且 MUST NOT 提高 `DivisionByZeroError` 或 `DomainError`。

IRIS-V1-RUNTIME-C125：对于 `Float32` 和 `Float64`，正或负零基数和正或负浮点零指数 MUST 的每种组合都会提高 `DomainError`。对于具有负非零 `Integer` 指数的零浮点基数，正零返回正无穷大，具有奇数指数的负零返回负无穷大，具有偶数指数的负零返回正无穷大。

IRIS-V1-RUNTIME-C126：负有限 `Float32` 或 `Float64` 基数提升为非整数浮点指数且没有实数结果时，MUST 按既定结果宽度返回 NaN，并且 MUST NOT 提高 `DomainError`。浮点求幂宽度 MUST 遵循普通浮点宽度规则： `Float32` 与 `Integer` 或 `Float32` 返回 `Float32`、`Float64` 与 `Integer` 或`Float64` 返回 `Float64`，混合 `Float32`/`Float64` 返回 `Float64`。

IRIS-V1-RUNTIME-C127：内置 `Integer` 按位运算符 `&`、`|`、`^`，前缀 `~`、`<<` 和`>>` MUST 使用抽象的无限符号扩展二进制补码模型。 `~x == -x - 1`、`~0 == -1`、右移为算术运算。

IRIS-V1-RUNTIME-C128：Integer 移位 MUST 接受任何符号的计数。负计数以精确的任意精度绝对值反转方向：`x << -n` 相当于 `x >> n`，对于正 `x >> -n`，`x << n` 相当于 `n`。零计数就是身份。负计数 MUST NOT 提高 `RangeError`。

IRIS-V1-RUNTIME-C129：每个非负右移计数都有一个数学结果。在无限符号扩展下，足够大的非负值右移稳定在 `0`，足够大的负值右移稳定在 `-1`。数学上有效的巨大分配 MAY 引发 `ResourceError` 或 `MemoryLimitError`；主机限制 MUST NOT 改变成功的数学结果。

IRIS-V1-RUNTIME-C130：内置数值 `==` 和 `!=` MUST 比较 `Integer`、`Float32` 和 `Float64` 之间的精确数学值，除了NaN。任意精度的 `Integer` MUST NOT 首先被舍入为浮点数以确保相等。正浮零和负浮零比较相等。每个 NaN 与每个值进行比较都不相等，包括其自身和另一个 NaN。

IRIS-V1-RUNTIME-C131：有限值上的内置数值 `<`、`<=`、`>`、`>=` 和 `<=>` MUST 使用精确的数学值并在操作数顺序上保持对称。无穷大通过扩展实数位置进行比较。如果任一操作数为 NaN，则 `<=>` 返回 `nil`，并且四个有序比较返回 `false` 而不引发。

IRIS-V1-RUNTIME-C132：内置数值 `<=>` 方法 MUST 接受 `other: Object` 并返回 `Integer?`。对于 `Integer`、`Float32` 或 `Float64`，它们返回精确的 `Integer(-1)`、`Integer(0)`、`Integer(1)` 或 `nil` NaN。非数字对象返回 `nil`。

IRIS-V1-RUNTIME-C133：内置数值比较方法是普通的动态可替换方法。冻结的数字比较规则仅指定其未修改的标准行为。替换它们的用户承担任何由此产生的对称性、传递性、排序或相等/哈希不一致的责任。

IRIS-V1-RUNTIME-C134：任一浮点宽度的 quiet 或 signaling NaN 都不是有效的 Hash 键。Hash 构造、插入、键更新，或对 NaN 直接调用内置公共 `hash`，MUST 引发 `InvalidKeyError`。

IRIS-V1-RUNTIME-C135：以下内置数值运算表是规范的：

| 操作族                   | 接收者和操作数                            | 结果和异常行为                                                            |
| ---------------------------------- | ------------------------------------------------ | ------------------------------------------------------------------------------------------ |
| `Integer + - *` 与 `Integer` | 精确的任意精度整数算术     | `Integer`；分配失败引发 `ResourceError` 或 `MemoryLimitError`|
|`Integer / Integer`              | 实数除法通过`Float64`转换      | `Float64`；零除数提高 `DivisionByZeroError`                                   |
| `Integer div Integer`            | 底商                                   | `Integer`；零除数提高 `DivisionByZeroError`                                   |
| `Integer mod Integer`            | `a - (a div b) * b`                            | `Integer`；零除数提高 `DivisionByZeroError`                                   |
| `Integer ** Integer`             | 非负指数                             | `Integer`； `0 ** 0` 筹集资金 `DomainError`|
|`Integer ** Integer`             | 负指数                                | `Float64`； `0 ** negative` 返回 `+Infinity`                                       |
| `Integer OP Float32`             | 接收者转换为`Float32`                 | `Float32`                                                                                |
| `Integer OP Float64`             | 接收者转换为`Float64`                 | `Float64`                                                                                |
| `Float32 OP Integer`             | 参数转换为`Float32`                 | `Float32`|
|`Float64 OP Integer`             | 参数转换为`Float64`                 | `Float64`                                                                                |
| `Float32 OP Float32`             | IEEE-754 二进制32                                | `Float32`                                                                                |
| `Float64 OP Float64`             | IEEE-754 二进制64                                | `Float64`                                                                                |
| `Float32 OP Float64` 或相反  | 加宽`Float32` 至 `Float64`                  | `Float64`|
|浮点数除以零             | IEEE-754非陷印                            | 签名无穷大或 NaN，no`DivisionByZeroError`                                          |
| 浮点溢出                     | IEEE-754溢出                                | 有符号无穷大，无溢出异常                                                     |
| `FloatN ** Integer`              | 保留基本宽度，除非与`Float64`混合 | `FloatN` 按宽度规则，零负指数使用有符号零奇偶校验                  |
| `FloatN ** FloatN`               | 浮点求幂                          | 宽度相同；零到零加注`DomainError`；负有限到非整数返回 NaN|
|混合浮点求幂         | Any`Float64` 操作数                           | `Float64`                                                                                |
| `mul_add`                        | 精确熔合产品加添加                     | 通用宽度，最后一次舍入，无效无穷大情况返回NaN                        |
| `to_bits` 和 `from_bits`      | 精确位重新解释                       | 往返每个位模式；范围违规引发`RangeError`                        |
| 按位`& \| ^ ~ << >>`           | `Integer` 仅内置标准运算符|无限符号扩展补码；巨额分配可能会筹集资金`ResourceError`        |

## 稳定哈希契约与向量

IRIS-V1-RUNTIME-C136：内置公共数值 `hash` MUST 返回 `Integer` 中的非负 `0..2^64-1`。对于普通 `==` 下相同的内置数值，MUST 在具体数值类型和浮点宽度上返回相同的值，包括正零和负零。不相等的值 MAY 发生碰撞。

IRIS-V1-RUNTIME-C137：公共数字 `hash` MUST 在 Iris 语言主要版本 1 内跨进程、平台、解释器/JIT 模式、指针宽度和一致实现保持稳定，同时内置 Method 保持选中状态。 Hash 容器 MUST NOT 直接使用公共值作为桶索引，无需秘密随机内部混合。

IRIS-V1-RUNTIME-C138：数字哈希计算 MUST 使用具有精确 ASCII 上下文 `Iris Language v1 stable numeric hash` 的 BLAKE3 派生密钥模式。它 MUST 通过规范数字编码计算标准 32 字节摘要，获取摘要字节 `0..8`，将这些字节解释为无符号小端字节序，然后返回该 `Integer`。

IRIS-V1-RUNTIME-C139：规范数字编码 MUST 以一个顶级字节开头：`0x00` 表示有限数学值，`0x01` 表示正无穷大，`0x02` 表示负无穷大。具体数值类型和浮点宽度MUST省略。 NaN MUST 没有规范的哈希编码。

IRIS-V1-RUNTIME-C140：有限零数字编码 MUST 正是 `[0x00, 0x00]`。非零有限值 MUST 使用精确范式 `sign * odd_significand * 2^exponent`，从正奇数有效数中删除 2 的所有因数。在顶级有限标签`0x00`之后，子标签`0x01`表示正，`0x02`表示负。

IRIS-V1-RUNTIME-C141：标准化正奇数有效数 MUST 编码为 `ULEB128(byte_length) || big_endian_magnitude`。 `byte_length` MUST 使用最短规范的 ULEB128 至少是一个。幅度 MUST 是最短的无符号大端字节序列，没有前导零和奇数正值。

IRIS-V1-RUNTIME-C142：有符号任意精度指数 MUST 使用数学 ZigZag 映射到非负任意精度整数：当 `e >= 0` 时为 `2*e`，当 `e < 0` 时为 `-2*e - 1`。映射值 MUST 编码为最短规范 ULEB128。Host 宽度移位或 XOR 公式 MAY 仅在完全等效且没有溢出或宽度依赖时使用。

IRIS-V1-RUNTIME-C143：正无穷大 MUST 编码与 `[0x01]` 完全相同，负无穷大与 `[0x02]` 完全相同。不存在宽度、符号字段、有效负载、长度或尾随字节。

IRIS-V1-RUNTIME-C144：规范数字哈希字节是规范内部的。一致的实现 MUST NOT 将它们公开为标准 Iris Method，例如 `canonical_numeric_bytes`。用户程序仅观察最终的公共哈希，而一致性工具 MAY 检查规范字节。

IRIS-V1-RUNTIME-C145：`nil`、`false` 和 `true` 的单例哈希 MUST 使用具有精确 ASCII 上下文的 BLAKE3 派生密钥模式分别为 `Iris Language v1 stable singleton hash` 和一字节输入 `[0x00]`、`[0x01]` 和 `[0x02]`。摘要缩减 MUST 使用前八个字节作为无符号小端字节序。

IRIS-V1-RUNTIME-C146：以下数字和单例哈希向量表是规范的。摘要前缀和 `hash` 值是使用 IRIS-V1-RUNTIME-C138 和 IRIS-V1-RUNTIME-C145 中命名的 BLAKE3 派生密钥上下文计算的。

| 向量 ID                | 值                                                             | 规范输入十六进制 | 上下文                                    | 摘要字节`0..8` 十六进制 | 公开哈希 Integer      |
| ------------------------ | ----------------------------------------------------------------- | ------------------- | ------------------------------------------ | ------------------------ | ------------------------ |
|`IRIS-V1-RUNTIME-V001` | `Integer(0)`, `Float32(+0.0)`, `Float64(-0.0)`              | `0000`            | `Iris Language v1 stable numeric hash`   | `c030472a1b58c53c`     | `4379003086384345280`  |
| `IRIS-V1-RUNTIME-V002` | `Integer(1)`，精确 `Float32(1.0)`，精确 `Float64(1.0)`    | `0001010100`      | `Iris Language v1 stable numeric hash`   | `384e0f3cb1fc5bf7`     | `17824117788395916856`|
|`IRIS-V1-RUNTIME-V003` | `Integer(-1)`，精确 `Float32(-1.0)`，精确 `Float64(-1.0)` | `0002010100`      | `Iris Language v1 stable numeric hash`   | `0cb716ef77f96039`     | `4134578751433783052`  |
| `IRIS-V1-RUNTIME-V004` | `Integer(2)` 和确切的 `Float64(2.0)`                         | `0001010102`      | `Iris Language v1 stable numeric hash`   | `15e928f2541b81c4`     | `14159628755083520277`|
|`IRIS-V1-RUNTIME-V005` | 精确的数学 `3/2` 为 `Float32(1.5)` 或 `Float64(1.5)` | `0001010301`      | `Iris Language v1 stable numeric hash`   | `6fd5e66f00ac66c4`     | `14152187996935738735` |
| `IRIS-V1-RUNTIME-V006` | 正无穷大，或者浮动宽度                             | `01`              | `Iris Language v1 stable numeric hash`   | `60fe3108fddb52bd`     | `13642208101069356640`|
|`IRIS-V1-RUNTIME-V007` | 负无穷大，或者浮动宽度                             | `02`              | `Iris Language v1 stable numeric hash`   | `e99dd5adce797260`     | `6949751103572778473`  |
| `IRIS-V1-RUNTIME-V008` | `nil`                                                           | `00`              | `Iris Language v1 stable singleton hash` | `d37c681abf4074a4`     | `11850167709044604115`|
|`IRIS-V1-RUNTIME-V009` | `false`                                                         | `01`              | `Iris Language v1 stable singleton hash` | `24d2464b3697b5f8`     | `17921396551637717540` |
| `IRIS-V1-RUNTIME-V010` | `true`                                                          | `02`              | `Iris Language v1 stable singleton hash` | `40168bee0b35dfc4`     | `14186115676603356736`|

IRIS-V1-RUNTIME-C147：以下 Contract-view 哈希示例向量对推导形状是规范性的。给定接收者公开哈希 `nil.hash == 11850167709044604115` 和 Contract Type 公开哈希 `0x0123456789abcdef`，视图哈希输入 MUST 为 `d37c681abf4074a4efcdab8967452301`，上下文 MUST 为 `Iris Language v1 contract view hash`，摘要字节 `0..8` MUST 为 `2a88da355f670591`，公开哈希 MUST 为 `10449872169006172202`。

## 内置开放性与元安全

IRIS-V1-RUNTIME-C148：`Nil`、`Bool`、`Integer`、`Float32` 和 `Float64` 稳定类 MAY 被打开以获取或删除动态成员、组成模块、替换兼容的Method 实现，并在其有效 `MetaCapabilities` 和静态脊柱允许时更改属性协议。

IRIS-V1-RUNTIME-C149：内置值 Class 开放性 MUST NOT 破坏单例身份、数字不变性、浮点位不变性、隐藏表示安全、GC 跟踪、原生布局、声明的合约、静态脊柱义务，或原始/JIT 防护有效性。

IRIS-V1-RUNTIME-C150：用户 meta API MUST NOT 改变 `Nil`、`Bool`、`Integer`、`Float32` 和 `Float64` 的运行时超类。任何针对这些受保护 Classes 的声明式或编程式超类变更尝试 MUST 引发 meta-operation 异常，中止完整事务组，并且不发布候选修订。

IRIS-V1-RUNTIME-C151：所有语法、反射、Dynamic、包、原生、Host、帮助程序和间接元操作路径 MUST 均低于相同操作级别 `MetaCapabilities` 和结构安全检查。用户代码 MUST NOT 获取或制造内部功能以绕过受保护的内置超类限制。

IRIS-V1-RUNTIME-C152：有效的 `MetaCapabilities` MUST 是 Class 修订版的不可变策略。源可表达的 `meta deny` MAY 仅缩小默认功能集。它 MUST NOT 授予运行时内部权力，或稍后扩大继承的、Module 提供的或 Contract 所需的拒绝。

IRIS-V1-RUNTIME-C153：Deny MUST 按后续元编程规则，通过静态超类链、运行时超类链、组合 Modules 和已声明 Contracts 累积。候选验证 MUST 在应用或提交结构操作前计算有效集合，失去授权 MUST 导致原子失败。

IRIS-V1-RUNTIME-C154：`method_set` 和 `method_body` MUST 是单独的功能。 `property_set` 和 `property_body` MUST 是独立的功能。 Class/共享存储形状和写入 MUST 使用 `class_state_set` 和 `class_state_write` 作为单独的功能。没有修补程序、反射、原生或 Host 路径 MAY 绕过这些功能。

IRIS-V1-RUNTIME-C155：以下开放表是规范的：

| 目标            | 初始允许的动态行为                                                                        | 内在受保护行为                                                                                  |
| ----------------- | --------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `Nil` Class     | 兼容的方法、属性、模块、`to_bool`、`method_missing` 行为                          | 单例身份和受保护的超类|
|`Bool` Class    | 兼容的方法、属性、模块、比较行为                                              | 两个单例身份，Bool 与数字的区别，受保护的超类                                |
| `Integer` Class | 兼容的方法、属性、模块、数字 Method 替换                                       | 无身份值语义、任意精度值不变性、无动态 ivars、受保护的超类 |
| `Float32` Class | 兼容的方法、属性、模块、特殊值 getter/setter 变更、数字 Method 替换 | 无身份值语义、32 位交换不变性、无动态 ivars、受保护的超类        |
| `Float64` Class | 兼容的方法、属性、模块、特殊值 getter/setter 变更、数字 Method 替换 | 无身份值语义、64 位交换不变性、无动态 ivars、受保护的超类|
|用户Class        | 静态脊柱和有效策略允许的功能                                                 | 静态脊柱，声明 Contract 设置，原生/布局安全，累积否认                                 |
| Module            | 兼容方法，Module组成，宣布政策收窄                                         | Module 身份，静态标头承诺，无自授予主机私有访问权限                                  |
| Contract          | 仅静态需求声明                                                                       | 没有主体，没有存储状态，没有初始化器，没有原始 ivar，没有打开操作                                  |

## 示例

IRIS-V1-RUNTIME-EX001：信息示例，逻辑 Class 和活动修订版：

```iris
let klass = Counter
let first = Counter.new()

open class Counter {
  override fun value() -> Integer { 2 }
}

klass same? Counter        // true, reopen kept the logical Class identity
first.value()             // later send uses the current active revision
```

IRIS-V1-RUNTIME-EX002：信息示例，BoundMethod 身份和替换：

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

IRIS-V1-RUNTIME-EX003：信息示例，普通和 Contract 限定的命名空间：

```iris
let view = parser as ParserContract
parser.process(input)     // ordinary selector process
view.process(input)       // still ordinary selector process on the receiver
view..process(input)      // qualified Contract slot ParserContract::process
```

IRIS-V1-RUNTIME-EX004：信息示例，Module MRO 订单：

```iris
module A { fun trace() -> Symbol { :A } }
module B mixin A { override fun trace() -> Symbol { :B } }
class C extends Object mixin A, B {}

C.new().trace()           // selects B before A
```

IRIS-V1-RUNTIME-EX005：信息示例，数字边缘行为：

```iris
5 / 2                     // Float64(2.5)
-5 div 2                  // Integer(-3)
-5 mod 2                  // Integer(1)
0 ** -1                   // Float64.infinity
Float64.from_bits(0x8000000000000000).hash == Float64.from_bits(0x0000000000000000).hash
```

## 运行时符合性向量

IRIS-V1-RUNTIME-C156：以下运行时向量表是规范的。一致性章节 MUST 保留这些向量 ID 或将它们映射到具有相同可观察结果的机器可读记录：

| 向量 ID                | 种类     | 场景                                                                          | 预期结果                                                           |
| ------------------------ | -------- | --------------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| `IRIS-V1-RUNTIME-V011` | 正向 | 重新打开 Class 并使用 `same?` 将捕获的 Class 对象与当前名称进行比较 | `true`                                                                  |
| `IRIS-V1-RUNTIME-V012` | 正向 | 现有实例在成功兼容 Method 替换后发送            | 新的活跃 Method 结果                                                  |
| `IRIS-V1-RUNTIME-V013` | 正向 | 输入旧框架继续，同时替换提交                             | 输入的帧返回旧的主体结果|
|`IRIS-V1-RUNTIME-V014` | 失败  | 在当前 MRO 缺少词法所有者的接收者上调用保留的 Method         | `MethodBindingError` 入口处                                           |
| `IRIS-V1-RUNTIME-V015` | 失败  | 保留的 Method 在所有者从当前 MRO 中删除后执行 `super`            | 入口处的 `MethodBindingError`，依 `IRIS-V1-RUNTIME-C015`              |
| `IRIS-V1-RUNTIME-V016` | 正向 | `obj.method same? obj.method`                                                   | `false`                                                                 |
| `IRIS-V1-RUNTIME-V017` | 正向 | 保存的 BoundMethod 与其自身相比与 `same?`                                | `true`|
|`IRIS-V1-RUNTIME-V018` | 失败  | `Integer(1) same? Integer(1)`                                                   | `IdentityError`                                                         |
| `IRIS-V1-RUNTIME-V019` | 正向 | `nil same? nil`, `true same? true`, `false same? false`                     | 全部`true`                                                               |
| `IRIS-V1-RUNTIME-V020` | 正向 | `(value as C)..m()` 和 `value.m()` 安装有不同的主体            | 合格调用仅选择`C::m`；普通呼叫选择普通 `m` |
| `IRIS-V1-RUNTIME-V021` | 失败  | 缺少合格的 Contract 槽位                                                   | `ContractDispatchError`，无 `method_missing`|
|`IRIS-V1-RUNTIME-V022` | 失败  | 私人 Method 存在，但呼叫者缺乏访问权限                                     | `MethodVisibilityError`，无 `method_missing`                          |
| `IRIS-V1-RUNTIME-V023` | 正向 | `mixin A, B` 和顺序 `include(A); include(B)`                          | 在 `B` 之前查找订单位置 `A`                                     |
| `IRIS-V1-RUNTIME-V024` | 正向 | 使用 `B mixin A` 和 `C mixin A, B` 进行嵌套 Module 重复数据删除                       | MRO 在最近出现时包含一次 close`A`                       |
| `IRIS-V1-RUNTIME-V025` | 正向 | 施工修改提交旧布局，稍后打开后发送              | 施工使用捕获的修订；稍后发送使用活动修订版|
|`IRIS-V1-RUNTIME-V026` | 失败  | `initialize` 在 `self` 逃脱后加注                                      | `new` 传播；转义对象保持普通且内存安全       |
| `IRIS-V1-RUNTIME-V027` | 正向 | `a.x = b.y = v` 内部 setter 返回标记                               | 外部 setter 接收标记                                              |
| `IRIS-V1-RUNTIME-V028` | 正向 | 缺少未声明的`@x` 已读                                                     | 返回`nil`，不创建槽                                           |
| `IRIS-V1-RUNTIME-V029` | 失败  | 无身份数值会创建未声明的`@x`                              | `InstanceStateError`|
|`IRIS-V1-RUNTIME-V030` | 正向 | `false && side_effect()`                                                        | RHS 未评估；返回`false`                                       |
| `IRIS-V1-RUNTIME-V031` | 正向 | `true \|\| side_effect()`                                                         | RHS 未评估；返回`true`                                        |
| `IRIS-V1-RUNTIME-V032` | 失败  | `to_bool` 返回非 Bool                                                      | `TypeContractError`                                                     |
| `IRIS-V1-RUNTIME-V033`| 正向 |删除`to_bool`，定义`method_missing(:to_bool, [], nil)`返回Bool    | 真相测试使用回退一次                                             |
| `IRIS-V1-RUNTIME-V034` | 失败  | 现有选择器数量不匹配                                                  | `ArgumentError`，无 `method_missing`                                  |
| `IRIS-V1-RUNTIME-V035` | 正向 | `Float64.nan == Float64.nan` 和 `Float64.nan != Float64.nan`                 | `false`, `true`                                                       |
| `IRIS-V1-RUNTIME-V036` | 正向 | 正负浮点零相等和哈希                                | 相等且相同的哈希值                                                       |
| `IRIS-V1-RUNTIME-V037`| 失败  |Hash 插入任何安静或信号 NaN                                    | `InvalidKeyError`                                                       |
| `IRIS-V1-RUNTIME-V038` | 正向 | `Float64.from_bits(bits).to_bits()` 用于信令-NaN 位                      | 相同的位                                                                 |
| `IRIS-V1-RUNTIME-V039` | 正向 | `-5 div 2`、`5 div -2`、`-5 div -2`                                         | `-3`, `-3`, `2`                                                     |
| `IRIS-V1-RUNTIME-V040` | 正向 | `-5 mod 2`, `5 mod -2`, `-5 mod -2`                                         | `1`、`-1`、`-1`                                                     |
| `IRIS-V1-RUNTIME-V041`| 失败  |`0 ** 0` 用于 Integer 或零浮点指数情况                               | `DomainError`                                                           |
| `IRIS-V1-RUNTIME-V042` | 正向 | 负有限浮点到非整数浮点指数                               | NaN，无`DomainError`                                                    |
| `IRIS-V1-RUNTIME-V043` | 正向 | `~0`、`-3 >> 1`、`x << -2`                                                  | `-1`、`-2`，与`x >> 2`相同                                        |
| `IRIS-V1-RUNTIME-V044` | 失败  | 交易组中受保护的内置超类变更                       | 元操作异常且无候选发布                     |
| `IRIS-V1-RUNTIME-V045`| 正向 |稳定哈希向量 V001 到 V010                                             | IRIS-V1-RUNTIME-C146 中的精确 Integer 输出                             |
| `IRIS-V1-RUNTIME-V048` | 正向 | `Float64.infinity.mul_add(1, Float64(-Infinity))`                                 | NaN 位于 `Float64`，无 Iris 例外                                        |


## 运行时覆盖向量

IRIS-V1-RUNTIME-C160：以下记录是规范的。每个源/输入单元都是可执行的 Iris 源或完整的受控运行时夹具。每个决策都直接由其声明的可观察到的内容覆盖。

IRIS-V1-RUNTIME-C161：名为 `name` 的存储属性在其修订级类型化实例存储中创建已声明的当前接收者原始槽 `@name`。其生成的 getter MUST 读取该确切槽，其生成的 setter MUST 检查并写入该确切槽。兼容的 `property fun` 替换只改变访问器 Method 主体：它不创建第二个 backing slot，且除非其主体显式使用 `@name`，否则不隐式读写该槽；任何这种原始访问都是 IRIS-V1-RUNTIME-C066 要求的同一 `(receiver, name)` 槽。

IRIS-V1-RUNTIME-C162：按 IRIS-V1-GRAMMAR-C059 编写的 `shared_decl` 创建 IRIS-V1-RUNTIME-C073 所描述的已声明层次结构绑定单元，并将其锚定到外围逻辑 Class 或 Module。`shared let` 创建不可变单元，任何后续赋值 MUST 失败；`shared mut` 创建可赋值单元。由于 IRIS-V1-RUNTIME-C075 禁止子类隐藏或重新声明锚定单元，其名称已在声明 Class 的静态词法 Class 祖先层次结构中任何位置锚定的 `shared_decl` MUST 被拒绝为重复声明，并且事务 MUST 不发布候选修订。对从未由任何 `shared_decl` 声明的名称赋值，仍会按 IRIS-V1-CONTROL-C009 作为缺少已声明存储而失败。

IRIS-V1-RUNTIME-C163：v1.12 勘误将 `Kernel` 定义为组合进 `Object` 的语言核心 Module。它承载必须在各处无需导入即可见的语言核心 Type 别名与声明，并且是 IRIS-V1-RUNTIME-C046 至 IRIS-V1-RUNTIME-C048 下的普通 Module，因此其组合、查找顺序与去重遵循与其他任何 Module 相同的规则。`Kernel` MUST NOT 提供、替换或遮蔽 IRIS-V1-RUNTIME-C005 赋予 `Object` 的任何默认行为，即默认比较、真值、缺失消息与零参 `initialize`。由于 IRIS-V1-RUNTIME-C046 先于运行时超类搜索已组合的 Module，`Kernel` 中与这四者冲突的成员将优先命中并架空 `C005`；此类声明 MUST 被拒绝。`Kernel` 不是第二个根 Class，不影响 `C005` 的单根要求。

| 向量 ID | 类别 | 适用性 | 源代码/输入 | 预期可观察结果 | 决策 |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-RUNTIME-V052` | 差异 | 需要解释器；需要 JIT；原生不适用 | `((2 ** 200) + 1) - (2 ** 200)` | `Integer(1)` 与 Type `Integer`；解释器和JIT同意；没有表示 Type 分裂。 | `D-003` |
| `IRIS-V1-RUNTIME-V053`| 正向 |需要解释器；需要 JIT；本地人不适用；原因：纯 Iris 数字源没有原生边界 | `type_of(1.0f32); type_of(1.0f64); type_of(1.0); type_of(1e10f64)` | `type`：`Float32`、`Float64`、`Float64`、`Float64`，按顺序排列。 | `D-005`, `D-006`, `D-007` |
| `IRIS-V1-RUNTIME-V054` | 负向 | 需要解释器； JIT 不适用；本地人不适用；原因：词汇拒绝 | `1.0F32` | `LEX_BAD_FLOAT_SUFFIX` 处于词汇阶段。 | `D-007` |
| `IRIS-V1-RUNTIME-V055` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：纯 Iris 派发源没有原生边界 | `fixture: {source: "Probe.left() + Probe.right()", setup: "left appends :left once and returns receiver; right appends :right once and returns argument; receiver + initially returns :old, then an authorized open replaces + with a Method returning :new", runs: 2}`|`value`：`[:old, :new]`； `side_effects`：评估日志正是`[:left, :right, :left, :right]`。 | `D-008` |
| `IRIS-V1-RUNTIME-V056` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：纯 Iris 数字源没有原生边界 | `type_and_value(16777217 + 0.0f32); type_and_value(0.0f32 + 16777217); type_and_value(1.5f32 + 2.25f64); type_and_value(2.25f64 + 1.5f32)` | 前两个值为 `Float32(16777216.0)`；最后两个是 `Float64(3.75)`。 | `D-010`, `D-011`, `D-012` |
| `IRIS-V1-RUNTIME-V057` | 正向 | 需要解释器；需要 JIT；原生不适用 | `1.0f64 / 0.0f64; 1.0f64 / -0.0f64; 0.0f64 / 0.0f64` | `+infinity`、`-infinity`、NaN；没有 `DivisionByZeroError`。 | `D-013`|
|`IRIS-V1-RUNTIME-V058` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：纯 Iris 数字源没有原生边界 | `Float32.from_bits(0x7f7fffff) * 2.0f32; Float64.from_bits(0x7fefffffffffffff) * 2.0f64` | `value`：`Float32` 处为正无穷大，然后 `Float64` 处为正无穷大；没有 Iris 错误。 | `D-014` |
| `IRIS-V1-RUNTIME-V059` | 正向 | 需要解释器；需要 JIT；原生不适用 | `Float64.nan == Float64.nan; Float64.nan < 1.0; Float64.nan <=> 1.0` | `false`、`false`、`nil`。 | `D-015`, `D-018` |
| `IRIS-V1-RUNTIME-V060` | 负向 | 需要解释器；需要 JIT；原生不适用|`1 / 0; 1 div 0; 1 mod 0` | 每个人都会筹集 `DivisionByZeroError`。 | `D-019` |
| `IRIS-V1-RUNTIME-V046` | 正向 | 需要解释器；需要 JIT；原生不适用 | `Integer(5) / Integer(2); Integer(4) / Integer(2)` | `Float64(2.5)` 和 `Float64(2.0)`。 | `D-020` |
| `IRIS-V1-RUNTIME-V061` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：纯 Iris 派发源没有原生边界 | `class A { public fun scale(value: Integer) -> Integer { value * 2 } }; let a = A.new(); a.scale(3); a scale 3` | `value`：`Integer(6)`，然后`Integer(6)`；两种形式都选择相同的 `scale` Method。|`D-021`, `D-022` |
| `IRIS-V1-RUNTIME-V062` | 正向 | 需要解释器；需要 JIT；原生不适用 | `2 ** 10; 2 ** -3; Float32.from_bits(0x80000000) ** -3` | `Integer(1024)`、`Float64(0.125)`、负 `Float32` 无穷大。 | `D-026`、`D-030`、`D-032` |
| `IRIS-V1-RUNTIME-V063` | 正向 | 需要解释器；需要 JIT；原生不适用 | `1 >> 1000000; -1 >> 1000000; -3 >> 1000000; 1 << -2` | `0`、`-1`、`-1`、`0`。 | `D-036`, `D-037` |
| `IRIS-V1-RUNTIME-V064`| 负向 |需要解释器；需要 JIT；本地人不适用；原因：受控运行时配额固定装置没有原生 API 边界 | `fixture: {memory_limit_bytes: 1048576, source: "1 << 1000000000"}` | `error`：`ResourceError`或`MemoryLimitError`，相`runtime`；无 `RangeError`、值或进程失败。 | `D-038` |
| `IRIS-V1-RUNTIME-V065` | 正向 | 需要解释器；需要 JIT；原生不适用 | `Float32.nan; Float64.infinity; -Float64.infinity` | 类型为 `Float32`、`Float64`、`Float64` 的规范值；裸 `nan` 和 `inf` 是不存在的名称。 | `D-051`, `D-052` |
| `IRIS-V1-RUNTIME-V066` | 差异 | 需要解释器；需要 JIT；本地人不适用；原因：主机舍入装置控制执行而不跨越原生 API | `fixture: {host_rounding_mode: toward_negative, source: "(Float32.from_bits(0x3f800001) * Float32.from_bits(0x3f800001)).to_bits()"}`|`backends`：解释器和JIT； `equivalence`：IEEE-754 `roundTiesToEven` 下的精确结果位，与主机舍入模式无关。 | `D-055`, `D-057` |
| `IRIS-V1-RUNTIME-V067` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：纯 Iris 融合算术没有原生边界 | `Float32.from_bits(0x3f800001).mul_add(Float32.from_bits(0x3f800001), Float32.from_bits(0xbf800000)); Float32.from_bits(0x3f800001) * Float32.from_bits(0x3f800001) + Float32.from_bits(0xbf800000); Float32.from_bits(0x3f800000).mul_add(2, 3.0)` | 第一个和第二个 Float32 结果具有相同编码；最终 Type 是 `Float64`。 | `D-056`, `D-058`, `D-059` |
| `IRIS-V1-RUNTIME-V047` | 正向 | 需要解释器；需要 JIT；原生不适用 | `Float64.infinity.mul_add(0, 1); Float32.infinity.mul_add(0, 1)` | NaN 位于通用结果宽度；没有 Iris 异常。 | `D-060`|
|`IRIS-V1-RUNTIME-V068` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：纯 Iris 浮点源没有原生边界 | `let x = Float64.from_bits(0x7ff0000000000001); [x.is_nan(), x.is_signaling_nan(), x.is_infinite(), x == x, x < 0.0, x <=> 0.0, x.to_bits()]` | `value`: `[true, true, false, false, false, nil, 0x7ff0000000000001]`;没有 Iris 错误。 | `D-063`、`D-065`、`D-066`、`D-067` |
| `IRIS-V1-RUNTIME-V069` | 负向 | 需要解释器；需要 JIT；原生不适用 | `Float64.from_bits(-1); Float32.from_bits(2 ** 32); Float64.from_bits(2 ** 64)` | 每个筹集`RangeError`；没有截断。 | `D-064` |
| `IRIS-V1-RUNTIME-V070` | 负向 | 需要解释器；需要 JIT；本地人不适用；原因：独立纯Iris源没有原生边界|`fixture: {independent_sources: ["let n = 1; n.@x = 2", "let f = 1.0f32; f.@x = 2", "1.0f64 same? 1.0f64"]}` | `error`：分别为`InstanceStateError`、`InstanceStateError`和`IdentityError`，相`runtime`。 | `D-068`, `D-069`, `D-070` |
| `IRIS-V1-RUNTIME-V071` | 正向 | 需要解释器；需要 JIT；原生不适用 | `Integer(9007199254740993) == 9007199254740993.0f64; 1 == 1.0f32` | `false`，然后 `true`；相同的数字具有相同的公共哈希值。 | `D-017`, `D-071` |
| `IRIS-V1-RUNTIME-V072` | 负向 | 需要解释器；需要 JIT；原生不适用 | `Float32.nan.hash; Float64.nan.hash; %{ Float64.nan: 1 }` | 每次评价都会引发`InvalidKeyError`。|`D-072`, `D-080` |
| `IRIS-V1-RUNTIME-V073` | 差异 | 需要解释器；需要 JIT；原生不适用 | `fixture: public_hash_inputs=[Integer(0),Float64.from_bits(0),nil,false,true]` | 精确的 C146 64 位公共哈希在后端之间一致。 | `D-073`，`D-074`，`D-075`，`D-076`，`D-077`，`D-078`，`D-079`，`D-081`， `D-082`、`D-083`、`D-084`、`D-085`、`D-086`、`D-112`、`D-113` |
| `IRIS-V1-RUNTIME-V074` | 负向 | 需要解释器；需要 JIT；原生不适用 | `Integer(1).canonical_numeric_bytes()` | `MessageNotFoundError`；规范哈希字节不是公共的 API。 | `D-087` |
| `IRIS-V1-RUNTIME-V075`| 正向 |需要解释器；需要 JIT；本地人不适用；原因：受控Iris比较夹具没有原生边界 | `fixture: {source: "let a = Probe.new(); let same = a; let b = Probe.new(); [a == same, a <=> b, a == b, a != b, a < b]", setup: "Probe#<=> returns nil and increments calls"}` | `value`: `[true, nil, false, true, false]`; `side_effects`：`<=>` 不会针对相同引用相等而调用，而是针对每个不同引用比较调用一次。 | `D-090`, `D-096`, `D-097` |
| `IRIS-V1-RUNTIME-V076` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：纯 Iris 单例比较源没有原生边界 | `[false <=> false, false <=> true, true <=> false, true == 1, false == 0, nil <=> nil, nil <=> false, nil < Object.new()]` | `value`：`[Integer(0), Integer(-1), Integer(1), false, false, Integer(0), nil, false]`。 | `D-114`、`D-115` |
| `IRIS-V1-RUNTIME-V077` | 正向 | 需要解释器；需要 JIT；原生不适用 | `obj.method == obj.method; closure.call() == closure.call(); obj.method same? obj.method`|每个独立评估的可调用对象比较不相等且不相同。 | `D-107`, `D-108` |
| `IRIS-V1-RUNTIME-V078` | 正向 | 需要解释器；需要 JIT；原生不适用 | 别名一Method；比较它，同体 Method 和 Class/Module/Contract/Type 对象 | 别名相同；默认情况下，不同的定义比较不相等。 | `D-109`, `D-110` |
| `IRIS-V1-RUNTIME-V079` | 正向 | 需要解释器；需要 JIT；原生不适用 | `fixture: object_hash_before_compact_gc; compact_gc; object_hash_after_compact_gc` | `0..2^64-1` 中相同的运行时本地哈希；固定装置未公开任何地址。 | `D-111`|
|`IRIS-V1-RUNTIME-V080` | 负向 | 需要解释器； JIT 不适用；本地人不适用；原因：声明验证先于可执行文件或原生派发 | `class A { fun f(value: Integer) -> Symbol { :integer }; fun f(value: String) -> Symbol { :string } }` | `error`：锚定到 IRIS-V1-RUNTIME-C024 的声明验证错误； `side_effects`：无过载设置或 Class 出版物。 | `D-235`, `D-358` |
| `IRIS-V1-RUNTIME-V081` | 正向 | 需要解释器；需要 JIT；原生不适用 | `(value as C)..m(); value.m()` 具有独特的主体 | `:qualified`，然后 `:ordinary`；查找命名空间保持独立。 | `D-238`, `D-355` |
| `IRIS-V1-RUNTIME-V082` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：受控Class开放没有原生边界|`fixture: {source: "let before = A; let value = A.new(); open class A { override public fun m() { :new } }; [before same? A, value.m()]", setup: "A#m initially returns :old and A conforms to Contract C"}` | `value`: `[true, :new]`;名义 Type 和 Contract 身份不变。 | `D-258`, `D-261` |
| `IRIS-V1-RUNTIME-V083` | 正向 | 需要解释器；需要 JIT；原生不适用 | `fixture: begin A.new; commit compatible A open before initialize returns; call instance.m` | 分配/初始化使用捕获的修订；稍后发送使用活动修订。 | `D-259`, `D-442` |
| `IRIS-V1-RUNTIME-V084` | 负向 | 需要解释器；需要 JIT；原生不适用 | `A.new_current(); A.new_checked()` | 两个缺席的选择器都会引发 `MessageNotFoundError`。|`D-260` |
| `IRIS-V1-RUNTIME-V085` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：受控修订夹具没有原生边界 | `fixture: {setup: "A#migrate_revision increments calls and returns nil; tracked instance a", actions: ["commit compatible open", "observe calls", "a.migrate_revision(old_revision, new_revision)", "attempt old_revision.reactivate()"]}` | `value`：显式调用`0`之前调用，显式结果`nil`；修订重新激活会引发元操作错误。 | `D-264`、`D-265`、`D-266` |
| `IRIS-V1-RUNTIME-V086` | 正向 | 需要解释器；需要 JIT；原生不适用 | `a.@x=1; a.@x="s"; b.@x` 对于不同的实例 | `a` 读取 `"s"`，`b` 读取 nil；槽位 Type 是 `Dynamic<Object>`。 | `D-307`, `D-308`, `D-309` |
| `IRIS-V1-RUNTIME-V087`| 正向 |需要解释器；需要 JIT；原生不适用 | 从实例 Closure 转义 Method 返回后增量 `@x` | 它改变了原来的接收者；重新绑定引发 `MethodBindingError`。 | `D-310`, `D-311` |
| `IRIS-V1-RUNTIME-V088` | 负向 | 需要解释器；需要 JIT；原生不适用 | 外部、子类、Dynamic 和反射发送到私有 `A#p` | `MethodVisibilityError`；声明 Class 代码成功。 | `D-313`, `D-356`, `D-444` |
| `IRIS-V1-RUNTIME-V089` | 正向 | 需要解释器；需要 JIT；原生不适用 | 组合 `M#bump` 在有或没有私有边缘权限的情况下读/写 `@x`|两者均更新`@x`；仅私有选择器访问更改。 | `D-315`, `D-317`, `D-318` |
| `IRIS-V1-RUNTIME-V090` | 正向 | 需要解释器；需要 JIT；原生不适用 | `class C mixin A,B`； `B mixin A`；每个定义 `trace` | `C.new().trace()` 是 `:B`；关闭的 `A` 在 MRO 中出现一次。 | `D-319`, `D-320` |
| `IRIS-V1-RUNTIME-V091` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：受控存储设备没有原生边界 | `fixture: {source: "A.set_shared(1); B.set_shared(2); A.set_own(:a); B.set_own(:b); [A.shared(), B.shared(), A.own(), B.own()]", setup: "A declares @@x; B extends A; set_shared/shared lexically access @@x; set_own/own access each Class object's @x"}` | `value`: `[Integer(2), Integer(2), :a, :b]`;子类 `@@x` 重新声明被拒绝。 | `D-430`、`D-435`、`D-436`|
|`IRIS-V1-RUNTIME-V092` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：受控构造夹具没有原生边界 | `fixture: {source: "Child.new().log", setup: "Base stored-property initializer appends :base; Child stored-property initializer appends :child; Child#initialize appends :initialize without super"}` | `value`: `[:base, :child, :initialize]`;反射报告两个存储属性的显式 getter 和 setter 方法。 | `D-445`, `D-446` |
| `IRIS-V1-RUNTIME-V093` | 负向 | 需要解释器； JIT 不适用；本地人不适用；原因：解析/运行时边界 | 裸`super`；明确的 `super()`，没有后继者 | 裸形式被拒绝；显式调用引发 `NoSuperMethodError`。 | `D-447` |
| `IRIS-V1-RUNTIME-V094` | 正向 | 需要解释器；需要 JIT；原生不适用|将 `f` 别名为 `g`，删除 `g`，然后 undef `f` 高于祖先 | 别名共享 Method 身份；删除暴露祖先； undef 到达缺失消息路径。 | `D-448` |
| `IRIS-V1-RUNTIME-V095` | 正向 | 需要解释器；需要 JIT；原生不适用 | `class fun build(){:class}; A.build()` 和继承的 Class-对象发送 | 选择单例Method；沿着 Class 对象链继续查找。 | `D-451` |
| `IRIS-V1-RUNTIME-V096` | 正向 | 需要解释器；需要 JIT；原生不适用 | 属性 `name` getter 返回 `:get`；设置器返回 `:set`；评估 `a.name` 和 `a.name=1` | Getter 和 Setter 选择器为 `name` 和 `name=`；结果是 `:get` 和 `:set`。|`D-343` |
| `IRIS-V1-RUNTIME-V097` | 正向 | 需要解释器；需要 JIT；原生不适用 | 自定义 `to_bool` 递增计数器然后返回 true；评估`if value { :yes }` | 计数器为 `1`；结果是`:yes`； `nil`、`false` 和 `true` 使用 false、false、true。 | `D-350` |
| `IRIS-V1-RUNTIME-V098` | 负向 | 需要解释器；需要 JIT；原生不适用 | `to_bool` 在 `:sentinel`、逻辑与和逻辑或赋值期间引发 `if` | `:sentinel` 传播不变；分支、RHS 和写回不运行。 | `D-352` |
| `IRIS-V1-RUNTIME-V099`| 正向 |需要解释器；需要 JIT；原生不适用 | 缺少 `obj.missing(1) { :block }` 和 `method_missing` 录音输入 | 使用 Symbol `:missing`、快照 `[1]` 和 Closure 调用一次；缺少 `method_missing` 会直接引发 `MessageNotFoundError`。 | `D-354` |
| `IRIS-V1-RUNTIME-V100` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：受控运行时超类固定装置没有原生边界 | `fixture: {setup: "instance is created from A; NewBase satisfies A's static superclass bound", action: "commit A runtime-superclass change to NewBase", source: "[instance is NewBase, type_of(A).subtype?(type_of(NewBase)), Reflection::Class.ancestors(A)]"}` | `value`：`[true, true, [A, NewBase, Object]]`。 | `D-262` |
| `IRIS-V1-RUNTIME-V101` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：受控内置Class开放没有原生边界 | `fixture: {actions: ["compose Module Marker into Nil", "add mark() returning :bool to Bool", "add mark() returning :integer to Integer"], source: "[nil.marker(), true.mark(), 1.mark(), nil same? nil]"}`|`value`：`[:nil, :bool, :integer, true]`；数字接收者状态创建仍然会引发 `InstanceStateError`。 | `D-285`, `D-286` |
| `IRIS-V1-RUNTIME-V102` | 正向 | 需要解释器；需要 JIT；原生不适用 | `5.mod(2); 5 mod 2; 5 % 2` | Method 和命名中缀都返回 `Integer(1)`； `%` 被语法拒绝。 | `D-024` |
| `IRIS-V1-RUNTIME-V103` | 正向 | 需要解释器；需要 JIT；原生不适用 | `Float64.infinity` getter 替换，然后添加一个记录其参数的 setter | 观察替换后的 getter 结果； setter 记录其参数而不创建隐式后备存储。 | `D-053`, `D-054`|
|`IRIS-V1-RUNTIME-V104` | 正向 | 需要解释器；需要 JIT；原生不适用 | `Float32.nan + 1.0f32; Float64.infinity - Float64.infinity` | 两者都在接收者/公共宽度处产生安静的 NaN；有效负载和符号未断言。 | `D-062` |
| `IRIS-V1-RUNTIME-V105` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：纯 Iris 数字比较没有原生边界 | `[9007199254740993 < 9007199254740992.0f64, 1.5f32 <=> 1.5f64, Float64.infinity > 10 ** 1000, Float64(-Infinity) < Integer(-10), Float64.nan <=> 0]` | `value`: `[false, Integer(0), true, true, nil]`;逆序有限比较是互补的。 | `D-088`、`D-092`、`D-095` |
| `IRIS-V1-RUNTIME-V106` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：受控数字 Method 打开没有原生边界|`fixture: {actions: ["replace Integer#<=> with a Method returning 1", "observe 1 < 2 and 1 > 2", "replace Integer#== with a Method returning true", "observe 1 == 2 and 1 > 2"]}` | `value`: `[false, true, true, true]`;单独替换的 `==` 不会替换 `<=>`。 | `D-089`, `D-091` |
| `IRIS-V1-RUNTIME-V107` | 负向 | 需要解释器；需要 JIT；本地人不适用；原因：Dynamic 运行时返回-Contract 边界没有原生 API 边界 | `fixture: {source: "Dynamic<Probe>(Probe.new()) == Object.new()", setup: "Probe#<=> is declared to return Dynamic<Object> and returns true"}` | `error`：`TypeError`，相`runtime`；没有 Bool 比较结果。 | `D-093`, `D-094` |
| `IRIS-V1-RUNTIME-V108` | 负向 | 需要解释器； JIT 不适用；本地人不适用；原因：声明验证先于可执行文件或原生派发 | `fixture: {source: "class Child extends Base {}", setup: "Base active revision denies subclass creation through MetaCapabilities"}` | `error`：元操作异常，阶段`declaration validation`； `side_effects`： `Child` 未发布。|`D-450` |
| `IRIS-V1-RUNTIME-V109` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：纯 Iris 身份源没有原生边界 | `fixture: {source: "left() same? right()", setup: "left and right each append once and return the same Object; Object#== and Object#<=> raise if called"}` | `value`: `true`; `side_effects`：日志正是 `[:left, :right]`，并且不会调用可替换的比较 Method。 | `D-098` |
| `IRIS-V1-RUNTIME-V110` | 负向 | 需要解释器； JIT 不适用；本地人不适用；原因：核心控制流拼写的声明在执行前被拒绝 | `fixture: {independent_sources: ["class A { fun !(value) { value } }", "class A { fun &&(value) { value } }", "class A { fun \|\|(value) { value } }", "class A { fun &&=(value) { value } }", "class A { fun \|\|=(value) { value } }"]}` | `error`：每个来源的声明验证拒绝； `side_effects`：未安装 Method 槽位。 | `D-351` |
| `IRIS-V1-RUNTIME-V111` | 负向 | 需要解释器； JIT 不适用；本地人不适用；原因：命名声明重新绑定是静态验证 | `fixture: {independent_sources: ["class A {}; A = class {}", "module M {}; M = module {}", "contract C {}; C = contract {}", "const K = Object.new(); K = Object.new()"]}` | `error`：每个源的静态重新绑定拒绝； `side_effects`：没有绑定变化。 | `D-449` |
| `IRIS-V1-RUNTIME-V036` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：纯 Iris 数字源没有原生边界 | `Float64.from_bits(0x0000000000000000) == Float64.from_bits(0x8000000000000000); Float64.from_bits(0x0000000000000000).hash == Float64.from_bits(0x8000000000000000).hash` | `value`：`true`，然后`true`；正零和负零具有相同的公共哈希值。|`D-016` |
| `IRIS-V1-RUNTIME-V039` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：纯 Iris 整数源没有原生边界 | `[-5 div 2, 5 div -2, -5 div -2]` | `value`：`[Integer(-3), Integer(-3), Integer(2)]`。 | `D-023` |
| `IRIS-V1-RUNTIME-V040` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：纯 Iris 整数源没有原生边界 | `[-5 mod 2, 5 mod -2, -5 mod -2]` | `value`：`[Integer(1), Integer(-1), Integer(-1)]`。 | `D-025` |
| `IRIS-V1-RUNTIME-V041`| 负向 |需要解释器；需要 JIT；本地人不适用；原因：纯 Iris 求幂源没有原生边界 | `fixture: {independent_sources: ["0 ** 0", "0.0f32 ** 0.0f32", "Float64.from_bits(0x8000000000000000) ** -0.0f64"]}` | `error`：每个来源筹集`DomainError`，相位`runtime`；不返回任何数值。 | `D-027`, `D-029` |
| `IRIS-V1-RUNTIME-V112` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：纯 Iris 整数求幂没有原生边界 | `type_and_value(0 ** -1)` | `value`：正无穷大； `type`: `Float64`;没有 `DivisionByZeroError` 或 `DomainError`。 | `D-028` |
| `IRIS-V1-RUNTIME-V042` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：纯 Iris 浮点求幂没有原生边界 | `type_and_value((-2.0f64) ** 0.5f64)`|`value`：安静的NaN； `type`: `Float64`;没有 `DomainError`。 | `D-031` |
| `IRIS-V1-RUNTIME-V043` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：纯 Iris 整数按位源没有原生边界 | `[~0, -3 >> 1, 8 << -2]` | `value`：`[Integer(-1), Integer(-2), Integer(2)]`；内置按位语义使用无限符号扩展的二进制补码。 | `D-034`, `D-035` |
| `IRIS-V1-RUNTIME-V048` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：纯 Iris 融合算术没有原生边界 | `type_and_value(Float64.infinity.mul_add(1, Float64(-Infinity)))` | `value`：安静的NaN； `type`：`Float64`；没有 Iris 异常。 | `D-061`|
|`IRIS-V1-RUNTIME-V019` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：纯 Iris 单例身份源没有原生边界 | `[nil same? nil, true same? true, false same? false]` | `value`: `[true, true, true]`;所有三个操作数都是带有身份的单例。 | `D-099` |
| `IRIS-V1-RUNTIME-V011` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：受控的Class开放没有原生边界 | `fixture: {source: "let before = A; open class A { public fun added() { :added } }; [before same? A, A.added()]", setup: "A is a named logical Class"}` | `value`: `[true, :added]`;重新打开发布新的活动修订版，而不替换 Class 身份。 | `D-100` |
| `IRIS-V1-RUNTIME-V012` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：受控 Method 替换没有原生边界|`fixture: {source: "let value = A.new(); open class A { override public fun m() { :new } }; value.m()", setup: "A#m initially returns :old"}` | `value`: `:new`;现有实例的未来发送选择替换 Method。 | `D-101` |
| `IRIS-V1-RUNTIME-V013` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：确定性 Method 帧派发器装置没有原生边界 | `fixture: {setup: "A#m enters and pauses before returning :old", schedule: ["enter value.m()", "commit replacement A#m returning :new", "resume entered frame", "call value.m() again"]}` | `value`: `[:old, :new]`;输入的帧保留其选定的正文，稍后发送选择替换内容。 | `D-101` |
| `IRIS-V1-RUNTIME-V015` | 负向 | 需要解释器；需要 JIT；本地人不适用；原因：受控 Method/MRO 夹具没有原生边界 | `fixture: {setup: "retain Module M Method m whose body executes super(); receiver current MRO initially contains M", actions: ["remove M from receiver Class MRO", "invoke retained Method on receiver"]}` | `error`：`MethodBindingError`，阶段 `runtime`，因为 `IRIS-V1-RUNTIME-C015` 的入口验证先于 `IRIS-V1-RUNTIME-C014` 中的 `super` 路径。|`D-102`, `D-103` |
| `IRIS-V1-RUNTIME-V014` | 负向 | 需要解释器；需要 JIT；本地人不适用；原因：受控反射 Method 灯具没有原生边界 | `fixture: {setup: "retain Method A#m", actions: ["change receiver Class MRO so A is absent", "reflectively invoke retained Method on receiver"]}` | `error`：调用入口处的 `MethodBindingError`； Method 主体不执行。 | `D-104` |
| `IRIS-V1-RUNTIME-V016` | 正向 | 需要解释器；需要 JIT；本地人不适用；原因：纯 Iris BoundMethod 身份源没有原生边界 | `obj.method same? obj.method` | `value`: `false`;每个 Method 读取都会创建一个不同的 BoundMethod 身份。 | `D-106` |
| `IRIS-V1-RUNTIME-V017`| 正向 |需要解释器；需要 JIT；本地人不适用；原因：受控 BoundMethod 替换没有原生边界 | `fixture: {source: "let saved = obj.method; open class A { override public fun method() { :new } }; [saved same? saved, saved(), obj.method()]", setup: "obj is an A and original A#method returns :old"}` | `value`：`[true, :old, :new]`；保存的 BoundMethod 保留其 Method 身份，而稍后查找会选择替换。 | `D-105`, `D-106` |
| `IRIS-V1-RUNTIME-V026` | 负向 | 需要解释器；需要 JIT；本地人不适用；原因：受控施工失败没有本土边界 | `fixture: {source: "let escaped = nil; class A { public fun initialize() { escaped = self; raise :sentinel } }; A.new()", observations_after_catch: ["escaped is A", "escaped.to_bool()"]}` | `error`：升高值`:sentinel`，相位`runtime`； `side_effects`：`new`不返回实例，而转义的是普通的内存安全`A`，`escaped.to_bool()`是`true`。 | `D-443`|
