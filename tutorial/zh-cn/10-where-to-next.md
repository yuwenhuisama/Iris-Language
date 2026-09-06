# 下一步

本教程为你提供了掌握 Iris 的入门指引。形式化规范才是语言体系的最终权威。作为教程的收尾章节，本章全面梳理 14 个规范产物，提供针对不同目标的阅读路径建议，并解析宿主嵌入、外部函数接口（FFI）与测试一致性的核心准则。

我们可以通过一个综合性的示例，将模块、契约、渐进类型与运行时对象完整串联起来：

<!-- iris-example: {"id":"10-capstone-conformance","mode":"vm","stdout":"Record::active\n2026-v1\ntrue\n"} -->
```iris
contract Describable {
  fun describe() -> String
}

module Timestamped {
  public fun stamp() -> String {
    "2026-v1"
  }
}

class Record for Describable mixin Timestamped {
  public impl fun describe() -> String {
    "Record::active"
  }
}

let rec = Record.new()
let desc = rec as Describable

print(desc..describe())
print(rec.stamp())
print(rec is Record)
```

预期终端输出：

```text
Record::active
2026-v1
true
```

## 14 个规范产物

Iris v1 规范包含 14 个结构清晰的正式文档。每个章节均带有明确的状态标识，表明其具备冻结的语义并已吸纳经所有者批准的勘误条款：

| 序号 | 规范产物 | 核心内容 |
| --- | --- | --- |
| 1 | [README.md](../../spec/iris-v1/README.md) | 产物目录、编辑准则、条款编号命名规范与依赖关系图。 |
| 2 | [01-language-identity.md](../../spec/iris-v1/01-language-identity.md) | 动态与静态的划分准则、实现独立性原则、包标识以及暂缓特性。 |
| 3 | [02-lexical-grammar.md](../../spec/iris-v1/02-lexical-grammar.md) | 上下文词法标记、字面量精度要求、运算符优先级、结合性与语法。 |
| 4 | [03-runtime-object-model.md](../../spec/iris-v1/03-runtime-object-model.md) | 对象模型、对象身份、方法分派、类与模块、MRO 线性化及修订版本。 |
| 5 | [04-bindings-callables-control-flow.md](../../spec/iris-v1/04-bindings-callables-control-flow.md) | 词法绑定、作用域、可调用体、闭包、迭代循环、模式匹配与 `ExceptionContext`。 |
| 6 | [05-types-contracts-generics.md](../../spec/iris-v1/05-types-contracts-generics.md) | 渐进类型检查、联合类型、可空性、转换、契约、具体化且不变的泛型与 `Never`。 |
| 7 | [06-collections-text-regex.md](../../spec/iris-v1/06-collections-text-regex.md) | 数组、元组、哈希表、区间、字符串编码、符号、字节序列、正则与稳定哈希。 |
| 8 | [07-async-resources-diagnostics.md](../../spec/iris-v1/07-async-resources-diagnostics.md) | 单线程 `Task`、异步调度、`Closeable` 资源清理、诊断与生命周期跟踪。 |
| 9 | [08-modules-metaprogramming.md](../../spec/iris-v1/08-modules-metaprogramming.md) | 包命名空间、开放事务、成员可见性、装饰器、`meta deny` 与 `ReflectionPolicy`。 |
| 10 | [09-native-host-ffi.md](../../spec/iris-v1/09-native-host-ffi.md) | C Host ABI、不透明句柄、原生载荷、线程亲和性与脚本级 FFI 规范。 |
| 11 | [10-serialization-standard-library.md](../../spec/iris-v1/10-serialization-standard-library.md) | JSON 序列化、IrisValue 编码、核心包边界与标准库范围。 |
| 12 | [11-migration-divergence.md](../../spec/iris-v1/11-migration-divergence.md) | 记录自早期设计以来的全部架构演进与差异化台账。 |
| 13 | [12-conformance.md](../../spec/iris-v1/12-conformance.md) | 测试向量模式、验证类别、适用性准则与冻结门禁。 |
| 14 | [traceability-matrix.md](../../spec/iris-v1/traceability-matrix.md) | 设计决策与规范条款、测试用例之间的双向追溯矩阵。 |

## 建议阅读路径

不同背景和需求的读者可以采用差异化的阅读路径：

- **深入语言语义**：阅读 `README.md`、`01-language-identity.md`、`02-lexical-grammar.md`、`03-runtime-object-model.md`、`04-bindings-callables-control-flow.md`、`05-types-contracts-generics.md`、`07-async-resources-diagnostics.md` 与 `08-modules-metaprogramming.md`。
- **日常应用开发**：聚焦于 `03-runtime-object-model.md`、`04-bindings-callables-control-flow.md`、`05-types-contracts-generics.md`、`06-collections-text-regex.md` 与 `07-async-resources-diagnostics.md`。
- **工具链与引擎开发**：首先研读 `README.md`、`01-language-identity.md`、`02-lexical-grammar.md`、`12-conformance.md` 与 `traceability-matrix.md`。
- **原生扩展与宿主嵌入**：按序阅读 `01-language-identity.md`、`05-types-contracts-generics.md`、`07-async-resources-diagnostics.md`、`08-modules-metaprogramming.md` 与 `09-native-host-ffi.md`。

## FFI 与 Host 嵌入

Iris 将宿主嵌入与脚本层面的外部函数调用进行了清晰的边界隔离。

Host ABI 为宿主进程嵌入 Iris 执行引擎提供了极其稳定的 C 接口。该接口使用不透明的、根植于运行时的句柄，并强制执行托管堆对象的运行时线程亲和性。Rust 和 C++ 的封装层仅作为 C ABI 之上的易用性绑定，绝非稳定的二进制身份依据。

**仅规范（不执行）：** 脚本级 `FFI` 子系统规定如何调用外部共享库。此示意需要 `mathlib` 及其 `mathlib.ffi` 声明文件，本例均未提供；它不展示当前 CLI 的 FFI 支持情况：

<!-- iris-example: {"id":"10-ffi-specification","mode":"spec-only","reason":"Specification example illustrating script-level FFI library loading and invocation syntax"} -->
```iris
let library = FFI.open("mathlib", declarations: "mathlib.ffi")
let value = library.hypot(3.0f64, 4.0f64)
library.close()
```

脚本代码绝不会直接操作裸内存地址或非托管指针。原生异步操作会将完成事件安全投递回 Iris 运行时的主线程中执行。

## 一致性

Iris 规范定义一致性要求和向量格式，与仓库中的可执行 `iris-conformance` 运行器是不同的产物。JSON 向量位于 [`conformance/iris-v1/`](../../conformance/iris-v1/)。运行器报告实现证据，包括通过的用例、现有缺口以及需要未完成子系统或诊断预期的用例。运行一次并不代表已实现完整一致性。

使用 Cargo 运行语法章节的一致性测试：

```bash
cargo run -p iris-conformance -- --chapter GRAMMAR
```

规范规定以下一致性规则；这些是要求，并不表示当前每条用例均已通过：

| 规则 | 含义 |
| --- | --- |
| 稳定的向量标识符 | 向量统一采用形如 `IRIS-V1-<chapter>-V<nnn>` 的不可变编号。 |
| 严密的分类观测 | 测试细分为 `positive`（正向）、`negative`（负向）、`diagnostic`（诊断）与 `differential`（差异对比）。 |
| 基于语义的断言比对 | 测试校验语言值、异常和副作用的语义结果，不比对内存地址或调试文本。 |
| 跨后端一致性 | 适用的实现必须产生一致的语义结果。仓库提供 VM 和参考求值器；这不意味着已有可用的原生编译引擎。 |
| 明确标注暂缓特性 | 暂缓实现的功能配备显式的拒绝向量，绝不伪装功能已实现。 |

## 静态承诺，动态自由

Iris 在坚实的确定性基础之上开放广阔的扩展自由。

**动态自由**
- 开发者可自由实现替代运行时、虚拟机、JIT 编译器或 AOT 原生引擎。
- 脚本充分享受动态消息分派与基于事务的安全元编程。
- 宿主系统可通过句柄抽象安全、动态地绑定原生扩展。

**静态承诺**
- 14 个规范产物及条款标识符是永久稳定的权威引用。
- 测试向量陈述语义预期；运行器结果展示实现对这些预期的满足程度。
- C Host ABI 保证跨工具链升级时的二进制接口稳定性。
- 不变的类型法则与契约义务在任何执行环境中始终有效。

**动手练习**

查阅 `conformance/iris-v1/vectors/runtime/` 中的测试向量。选取简单用例，阅读其来源条款与输入断言，运行 `cargo run -p iris-conformance -- --chapter RUNTIME`。将报告与向量预期对照，记录缺口或不支持的用例，不要假设全部用例均会通过。

## 阅读规范

如需开始深入研读正式规范，建议从 [README.md](../../spec/iris-v1/README.md) 起步，并逐一查阅以下基础条款：

| 条款 | 主题 |
| --- | --- |
| `IRIS-V1-TRACE-C014` | 固定的 14 篇规范产物清单。 |
| `IRIS-V1-TRACE-C009` | 规范依赖顺序与阅读建议。 |
| `IRIS-V1-TRACE-C016` | 规范术语及其形式化定义。 |
| `IRIS-V1-IDENTITY-C019` 至 `IRIS-V1-IDENTITY-C023` | 实现独立性与 C Host ABI 核心准则。 |
| `IRIS-V1-FFI-C003` 至 `IRIS-V1-FFI-C006` | 宿主嵌入与脚本 FFI 的边界隔离。 |
| `IRIS-V1-FFI-C043` 至 `IRIS-V1-FFI-C050` | 脚本级 FFI 库与声明规范。 |
| `IRIS-V1-CONFORMANCE-C003` 至 `IRIS-V1-CONFORMANCE-C018` | 向量模式、类别划分与验证准则。 |
| `IRIS-V1-CONFORMANCE-C046` 至 `IRIS-V1-CONFORMANCE-C049` | 各章节必备的向量覆盖要求。 |
