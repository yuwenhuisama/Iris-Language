# 下一步

本教程给你一条入门路径。规范才是权威。本章会映射 14 个 Iris v1 规范产物，为不同目标建议阅读顺序，并指出与 FFI、Host 嵌入和一致性相关的章节。

```iris
let library = FFI.open("mathlib", declarations: "mathlib.ffi")
let value = library.hypot(3.0f64, 4.0f64)
library.close()
```

这段代码复用自 `IRIS-V1-FFI-EX001`。它展示脚本 FFI 的形状：通过标准 `FFI` 子系统打开一个库，只调用已声明的签名，然后关闭资源。参考实现还未提供它。

## 14 个规范产物

规范目录有固定清单。文件名很重要，因为条款 ID 和可追溯性都引用它们。章节带有状态行：`frozen semantics with owner-approved errata` 表示原始条款未被更改，而后续勘误条款会就地取代特定条款。勘误条款总会说明它取代了哪个条款，所以被取代的条款永远不会被静默删除。

| 顺序 | 规范产物 | 阅读目的 |
| --- | --- | --- |
| 1 | [README.md](../../spec/iris-v1/README.md) | 清单、编辑规则、条款 ID 规则、术语和依赖顺序。 |
| 2 | [01-language-identity.md](../../spec/iris-v1/01-language-identity.md) | 动态与静态契约、实现独立性、包身份和 v1 延后项。 |
| 3 | [02-lexical-grammar.md](../../spec/iris-v1/02-lexical-grammar.md) | Tokens、literals、优先级、结合性、保留关键字和语法。 |
| 4 | [03-runtime-object-model.md](../../spec/iris-v1/03-runtime-object-model.md) | Objects、身份、派发、Classes、Modules、MRO、revisions、properties、truthiness 和内建项。 |
| 5 | [04-bindings-callables-control-flow.md](../../spec/iris-v1/04-bindings-callables-control-flow.md) | 绑定、作用域、函数、Closures、调用、循环、match、raise、catch、finally 和 `ExceptionContext`。 |
| 6 | [05-types-contracts-generics.md](../../spec/iris-v1/05-types-contracts-generics.md) | 渐进类型、`Dynamic`、unions、intersections、nilability、casts、Contracts、泛型和 `Never`。 |
| 7 | [06-collections-text-regex.md](../../spec/iris-v1/06-collections-text-regex.md) | Tuple、Array、Hash、Range、iteration、String、MutableString、Symbol、Bytes、Regex 和稳定哈希。 |
| 8 | [07-async-resources-diagnostics.md](../../spec/iris-v1/07-async-resources-diagnostics.md) | `Task`、`Awaitable`、调度器行为、`using`、Closeable 清理、diagnostics 和 revision events。 |
| 9 | [08-modules-metaprogramming.md](../../spec/iris-v1/08-modules-metaprogramming.md) | Packages、imports、exports、open 事务、静态与动态成员、装饰器、MetaCapabilities 和 ReflectionPolicy。 |
| 10 | [09-native-host-ffi.md](../../spec/iris-v1/09-native-host-ffi.md) | 稳定 C Host ABI、不透明 handles、native metadata、native payloads、async completion、script FFI 和 wrapper 指引。 |
| 11 | [10-serialization-standard-library.md](../../spec/iris-v1/10-serialization-standard-library.md) | JSON 和 IrisValue 范围、Encoding API、core package 边界和标准库延后项。 |
| 12 | [11-migration-divergence.md](../../spec/iris-v1/11-migration-divergence.md) | 每个有意差异和迁移台账行。 |
| 13 | [12-conformance.md](../../spec/iris-v1/12-conformance.md) | Vector schema、categories、适用性、预期观察、覆盖规则和冻结门槛。 |
| 14 | [traceability-matrix.md](../../spec/iris-v1/traceability-matrix.md) | 从冻结决策到条款、示例、vectors 和延后项的映射。 |

## 建议阅读路径

如果你想了解语言语义，请阅读 README、identity、grammar、runtime、control、types、async 和 metaprogramming。这条路径说明程序的含义从何而来。

如果你想理解普通编程风格，请阅读 runtime、control、types、collections、async，然后阅读 metaprogramming。Grammar 可以先略读，直到你需要精确的优先级或 literal 规则。

如果你想做工具，请先阅读 README、identity、grammar、conformance 和 traceability。工具工作在需要 prose examples 之前，先需要精确 IDs、产物边界、vector categories 和冻结门槛。

如果你想做原生集成，请按顺序阅读 identity、types、async、metaprogramming 和 FFI。原生章节依赖与源码相同的 Contract、ReflectionPolicy、MetaCapabilities 和 Task 规则。

## FFI 与 Host 嵌入

Iris v1 有两个接近原生的表面，而且它们刻意分开。

Host ABI 是用于嵌入或扩展 Iris runtime 的稳定 C 边界。它使用不透明的 runtime-rooted handles、status 加 result records，以及触及堆调用的 runtime-thread affinity。Rust 和 C++ wrappers 可以存在，但它们只是 C ABI 之上的便利层，不是稳定二进制身份。

Script FFI 是 Iris 源码层外部二进制调用路径。它经过 `FFI.open`、已验证签名和 `FFI::Library` objects。Scripts 不会直接调用 Host ABI tables、extension tables、raw loader APIs 或任意 symbols。

Native async work 返回 `Task<T>`，并通过 runtime-owned completion token 完成，该 token 会 post 回 runtime thread。Worker threads 不读取 Iris handles，也不触碰 managed heap。

## 一致性

一致性章节定义 record shape 和覆盖规则。Conformance vector 是稳定 JSON 记录，包含 ID、source clauses、input、适用性、预期观察和 tags。从冻结表派生的 vectors 位于 [`conformance/iris-v1/`](../../conformance/iris-v1/)，`iris-conformance` 按章节运行它们。

```bash
cargo run -p iris-conformance -- --chapter GRAMMAR
```

读者需要注意的要点：

| 规则 | 含义 |
| --- | --- |
| Vector IDs 是稳定的 | `IRIS-V1-<chapter>-V<nnn>` 是身份，不是人类可读名称。 |
| Categories 是固定的 | Vectors 是 `positive`、`negative`、`diagnostic` 或 `differential`。 |
| Expected observations 是语义性的 | 工具比较 Iris values、types、diagnostics、statuses、artifacts 和 side effects，不比较 heap addresses 或 debug strings。 |
| Backend differences 不允许 | Interpreter、JIT、native 和 host paths 在适用处必须一致。 |
| Deferred features 不是行为 | 延后项可以有 rejection 或 diagnostic vector，不能有假装该特性存在的 vector。 |

## 静态承诺，动态自由

动态自由：规范允许实现选择 interpreter、JIT、AOT、allocation、GC、caches 和 host wrappers，只要可观察的 Iris 行为不变。

静态承诺：规范清单、条款 IDs、包身份、Type 身份、一致性 schema、Host ABI 角色，以及后端无关语义都足够稳定，可供未来工具和实现作为目标。

## 阅读规范

精确规则请从 [README.md](../../spec/iris-v1/README.md) 开始，然后阅读这些条款：

| 条款 | 主题 |
| --- | --- |
| `IRIS-V1-TRACE-C014` | 固定的 14 产物清单。 |
| `IRIS-V1-TRACE-C009` | 规范依赖阅读顺序。 |
| `IRIS-V1-TRACE-C016` | 规范术语。 |
| `IRIS-V1-IDENTITY-C019` through `IRIS-V1-IDENTITY-C023` | 实现独立性和 C Host ABI 身份。 |
| `IRIS-V1-FFI-C003` through `IRIS-V1-FFI-C006` | 稳定边界角色。 |
| `IRIS-V1-FFI-C043` through `IRIS-V1-FFI-C050` | Script FFI library 表面。 |
| `IRIS-V1-CONFORMANCE-C003` through `IRIS-V1-CONFORMANCE-C018` | 一致性术语、IDs、categories 和 schema shape。 |
| `IRIS-V1-CONFORMANCE-C046` through `IRIS-V1-CONFORMANCE-C049` | 各章节必需的 vector classes。 |
