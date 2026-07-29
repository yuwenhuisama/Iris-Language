# Iris v1 迁移与分歧台账

状态：Iris v1 草案，语义冻结。

IRIS-V1-MIGRATION-C001：本章是 Iris v1 对 Legacy Iris 的有意源代码和行为分歧的唯一台账。每个迁移行 MUST 记录遗留证据、v1 替代项、理由、迁移示例，以及一个处置标签：`preserve`、`intentional-divergence`、`removed` 或 `deferred`。

IRIS-V1-MIGRATION-C002：历史文件仅作为迁移证据。PDF、Notepad++ 高亮器、遗留脚本、生成的解析器输出和旧实现行为，MUST NOT 成为规范性内容，除非冻结决策和规范性 v1 章节采纳该行为。

IRIS-V1-MIGRATION-C003：只由 `legacy/Document/Iris Programming Language Intro.pdf` 支持的行为 MUST NOT 在没有冻结决策以及遗留脚本、实现行为或显式 v1 替代条款之一佐证时标记为 `preserve`。仅 PDF 证据可以支持 `removed`、`intentional-divergence` 或 `deferred`，但不能支持保留。

IRIS-V1-MIGRATION-C004：当已移除的 Legacy Iris 语法可识别时，迁移工具 SHOULD 用本章的行 ID 诊断它。命名该行的诊断仍 MUST 指向 v1 替代条款，而不是把历史语法当作可接受源代码。

IRIS-V1-MIGRATION-C005：本章不保留历史词。保留关键字和历史非关键字归 [02-lexical-grammar.md](02-lexical-grammar.md) 所有，尤其是 `IRIS-V1-GRAMMAR-C013` 和 `IRIS-V1-GRAMMAR-C014`。

IRIS-V1-MIGRATION-C006：下面的台账行 ID 是稳定迁移 ID。不要重新编号现有行。用下一个未使用的 `IRIS-V1-MIG-<nnn>` ID 添加新行。

## 处置标签

IRIS-V1-MIGRATION-C007：下列处置标签对本台账具有规范性：

| 标签 | 含义 |
| --- | --- |
| `preserve` | Iris v1 在当前 v1 表面下保留遗留意图或行为。当替代条款如此规定时，源拼写仍可能改变。 |
| `intentional-divergence` | Iris v1 保留底层语言目标，但改变源语法、语义、失败行为或静态 contract。 |
| `removed` | Iris v1 不为该遗留行为提供可接受源形式或永久别名。 |
| `deferred` | Iris v1 不定义该行为，但该领域可由未来语言版本或标准包重新考虑。 |

## 历史证据台账

IRIS-V1-MIGRATION-C008：本表每一行作为迁移处置具有规范性，作为考古说明具有资料性。证据路径命名历史来源，但这些来源本身不创建 v1 规则。

| ID | 遗留行为或词 | 证据 | 处置 | V1 替代条款 | 迁移示例 | 理由 |
| --- | --- | --- | --- | --- | --- | --- |
| IRIS-V1-MIG-001 | 语句开头分号，例如 `;print(a)` | `legacy/Document/Iris Revival Design Review.md:268`; `Iris Library Test/test script/statement/test_switch_statement.ir:1`; `Iris Library Test/test script/expression/test_binary_expression.ir:1` | `intentional-divergence` | [IRIS-V1-GRAMMAR-C008](02-lexical-grammar.md), [IRIS-V1-GRAMMAR-C047](02-lexical-grammar.md), [IRIS-V1-GRAMMAR-C054](02-lexical-grammar.md) | IRIS-V1-MIGRATION-EX001：遗留 `;print(a)` 变为单独一行的 `print(a)`，或只把 `x = 1; print(x)` 作为同一行分隔符。 | D-362 和 D-364 移除视觉噪声和空语句歧义，同时保留普通语句终止。 |
| IRIS-V1-MIG-002 | 循环 if 形式，例如 `if(true, 0)` 和 `if(cond, count, counter)` | `legacy/Document/Iris Revival Design Review.md:281`; `Iris Library Test/test script/statement/test_loopif_statement.ir:2`; `Iris Library Test/test script/statement/test_continue_statement.ir:17` | `intentional-divergence` | [IRIS-V1-CONTROL-C043](04-bindings-callables-control-flow.md), [IRIS-V1-CONTROL-C044](04-bindings-callables-control-flow.md), [IRIS-V1-COLLECTIONS-C038](06-collections-text-regex.md) | IRIS-V1-MIGRATION-EX002：遗留 `if(true, 10, i) { body }` 变为 `for i in 1 ..= 10 { body }`；遗留 `if(cond, 0) { body }` 变为 `while cond { body }`。 | D-438 保持循环显式。计数使用 Range 加 `for`，条件使用 `while`。 |
| IRIS-V1-MIG-003 | 遗留 `block` 标记以及 `with` 或 `without` 块存在分支 | `legacy/Document/Iris Revival Design Review.md:297`; `legacy/Document/IrisLangHighLight(for NP++).xml:27`; `Iris Library Test/test script/statement/test_block_statement.ir:1`; `Iris Library Test/test script/statement/test_closure_block.ir:11` | `intentional-divergence` | [IRIS-V1-CONTROL-C023](04-bindings-callables-control-flow.md), [IRIS-V1-CONTROL-C030](04-bindings-callables-control-flow.md), [IRIS-V1-GRAMMAR-C050](02-lexical-grammar.md) | IRIS-V1-MIGRATION-EX003：遗留 `block` 加 `with` 或 `without` 变为声明的 `&block: (String) -> Nil = nil` 参数，以及普通的 `if block != nil { block(value) }`。 | D-423 把块传递变成显式 callable 通道，而不是主体外部标记。 |
| IRIS-V1-MIG-004 | 遗留 `cast.call(...)` 作为尾随块调用 | `legacy/Document/Iris Revival Design Review.md:297`; `legacy/Document/Iris Revival Design Review.md:386`; `Iris Library Test/test script/statement/test_block_statement.ir:5`; `Iris Library Test/test script/expression/test_functioncall_expression.ir:26` | `intentional-divergence` | [IRIS-V1-CONTROL-C030](04-bindings-callables-control-flow.md), [IRIS-V1-TYPES-C029](05-types-contracts-generics.md), [IRIS-V1-TYPES-C030](05-types-contracts-generics.md) | IRIS-V1-MIGRATION-EX004：遗留 `cast.call(value)` 对 `&block` 参数变为 `block(value)`；类型转换使用 `value as Type` 或 `value as? Type`。 | D-454 把 `as` 和 `as?` 恢复为已检查类型操作，并移除 `cast` 的重载含义。 |
| IRIS-V1-MIG-005 | Method 权限词 `everyone`、`relative` 和 `personal`；仅 PDF 的 `native` | `legacy/Document/Iris Revival Design Review.md:313`; `legacy/Document/Iris Revival Design Review.md:386`; `legacy/Document/IrisLangHighLight(for NP++).xml:27`; `Iris Library Test/test script/statement/test_authority_statement.ir:18` | `intentional-divergence` | [IRIS-V1-GRAMMAR-C049](02-lexical-grammar.md), [IRIS-V1-RUNTIME-C077](03-runtime-object-model.md), [IRIS-V1-RUNTIME-C078](03-runtime-object-model.md), [IRIS-V1-RUNTIME-C079](03-runtime-object-model.md) | IRIS-V1-MIGRATION-EX005：遗留 `everyone [draw]`、`relative [update]` 和 `personal [reset]` 变为 `public fun draw()`、`protected fun update()` 和 `private fun reset()`。 | V1 使用常规的声明本地可见性。仅 PDF 的 `native` 不保留，因为脚本和高亮器佐证的是 `relative`，不是 `native`。 |
| IRIS-V1-MIG-006 | 异常抛出词 `groan` | `legacy/Document/Iris Revival Design Review.md:251`; `legacy/Document/Iris Revival Design Review.md:1086`; `legacy/Document/IrisLangHighLight(for NP++).xml:27`; `Iris Library Test/test script/statement/test_groan_statement.ir:1` | `removed` | [IRIS-V1-CONTROL-C056](04-bindings-callables-control-flow.md), [IRIS-V1-CONTROL-C057](04-bindings-callables-control-flow.md), [IRIS-V1-CONTROL-C058](04-bindings-callables-control-flow.md) | IRIS-V1-MIGRATION-EX006：遗留 `groan(error)` 变为 `raise error`；原因处理使用 `raise error from context` 或 `raise error from nil`。 | D-145 把 `raise` 定为唯一正式抛出语法，并且不提供永久同义词。 |
| IRIS-V1-MIG-007 | 异常处理词 `order`、`serve` 和 `ignore` | `legacy/Document/Iris Revival Design Review.md:251`; `legacy/Document/Iris Revival Design Review.md:1086`; `legacy/Document/IrisLangHighLight(for NP++).xml:27`; `Iris Library Test/test script/statement/test_order_statement.ir:8` | `removed` | [IRIS-V1-CONTROL-C055](04-bindings-callables-control-flow.md), [IRIS-V1-CONTROL-C059](04-bindings-callables-control-flow.md), [IRIS-V1-CONTROL-C063](04-bindings-callables-control-flow.md) | IRIS-V1-MIGRATION-EX007：遗留 `order { body } serve(e) { handler } ignore { cleanup }` 变为 `try { body } catch e { handler } finally { cleanup }`。 | D-146 采用常见的 `try`、`catch` 和 `finally` 词汇，并移除别名。 |
| IRIS-V1-MIG-008 | 历史 `Interface` 声明和 interface 符合性词汇 | `legacy/Document/Iris Revival Design Review.md:342`; `legacy/Document/Iris Revival Design Review.md:880`; `legacy/Document/IrisLangHighLight(for NP++).xml:27`; `Iris Library Test/test script/statement/test_inteface_intefacefunction_statement.ir:1` | `intentional-divergence` | [IRIS-V1-TYPES-C041](05-types-contracts-generics.md), [IRIS-V1-TYPES-C044](05-types-contracts-generics.md), [IRIS-V1-TYPES-C086](05-types-contracts-generics.md) | IRIS-V1-MIGRATION-EX008：遗留 `interface Printable { fun print() } class User joints Printable { ... }` 变为 `contract Printable { fun print() -> String } class User for Printable { impl fun print() -> String { ... } }`。 | V1 使用规范术语 Contract 和显式名义符合性。历史 `Interface` 只用于迁移文本。 |
| IRIS-V1-MIG-009 | `implements` 和 `satisfies` 作为可能的符合性词 | `legacy/Document/Iris Revival Design Review.md:327`; `spec/drafts/iris-language-specification.md:538` | `removed` | [IRIS-V1-GRAMMAR-C014](02-lexical-grammar.md), [IRIS-V1-TYPES-C044](05-types-contracts-generics.md) | IRIS-V1-MIGRATION-EX009：写 `class Sprite for Drawable { ... }`，不要写 `implements Drawable` 或 `satisfies Drawable`。 | D-278 冻结 `for` 作为 Class 头符合性关键字，D-509 把这两个词排除在保留状态之外。 |
| IRIS-V1-MIG-010 | Module mixin 词 `involve`、`involves` 和 `joints` | `legacy/Document/Iris Revival Design Review.md:327`; `legacy/Document/Iris Revival Design Review.md:386`; `legacy/Document/IrisLangHighLight(for NP++).xml:27`; `Iris Library Test/test script/statement/test_module_statement.ir:27` | `intentional-divergence` | [IRIS-V1-GRAMMAR-C052](02-lexical-grammar.md), [IRIS-V1-RUNTIME-C046](03-runtime-object-model.md), [IRIS-V1-META-C053](08-modules-metaprogramming.md) | IRIS-V1-MIGRATION-EX010：遗留 `class E involves A joints A::D` 变为 `class E mixin A for D { ... }`，Contract 实现由 `impl` 标记。 | V1 把通过 `mixin` 的行为组合与通过 `for` 的 Contract 符合性分开。 |
| IRIS-V1-MIG-011 | Range 箭头和带括号的开闭 Range 形式，包括 PDF `->` 和脚本 `=>` | `legacy/Document/Iris Revival Design Review.md:354`; `legacy/Document/Iris Revival Design Review.md:386`; `Iris Library Test/test script/expression/test_range_expression.ir:1`; `Iris Library Test/test script/statement/test_for_statement.ir:1` | `intentional-divergence` | [IRIS-V1-GRAMMAR-C018](02-lexical-grammar.md), [IRIS-V1-GRAMMAR-C040](02-lexical-grammar.md), [IRIS-V1-COLLECTIONS-C006](06-collections-text-regex.md), [IRIS-V1-COLLECTIONS-C038](06-collections-text-regex.md) | IRIS-V1-MIGRATION-EX011：遗留 `[1 => 10]` 或 PDF `1 -> 10` 变为 `1 ..= 10`；排除终点变为 `1 ..< 10`。 | D-463 分配专用 Range token，并把 `=>` 留给 match 分支和历史 Hash 语法迁移。 |
| IRIS-V1-MIG-012 | Integer 轮转运算符 `<<<`、`>>>`、`<<<=` 和 `>>>=` | `legacy/Document/IrisLangHighLight(for NP++).xml:16`; `Iris Library Test/test script/expression/test_binary_expression.ir:24`; `spec/drafts/iris-language-specification.md:70` | `removed` | [IRIS-V1-GRAMMAR-C016](02-lexical-grammar.md), [IRIS-V1-RUNTIME-C127](03-runtime-object-model.md), [IRIS-V1-RUNTIME-C128](03-runtime-object-model.md) | IRIS-V1-MIGRATION-EX012：把遗留 `x >>> n` 或 `x <<< n` 替换为未来 `BitVector` API 存在时的定宽库操作；任意精度 `Integer` 只保留 `<<` 和 `>>`。 | D-034 移除轮转，因为任意精度 Integer 没有内在轮转宽度。 |
| IRIS-V1-MIG-013 | 遗留 Array 读取副作用和无类型空 Array 默认值 | `legacy/Document/Iris Revival Design Review.md:348`; `Iris Library Test/test script/expression/test_array_expression.ir:1`; `Iris Library Test/test script/expression/test_index_exxpression.ir:1` | `intentional-divergence` | [IRIS-V1-COLLECTIONS-C023](06-collections-text-regex.md), [IRIS-V1-COLLECTIONS-C024](06-collections-text-regex.md), [IRIS-V1-TYPES-C059](05-types-contracts-generics.md) | IRIS-V1-MIGRATION-EX013：遗留 `value = array[100]` 不再增长 Array。使用 `array.append(value)` 或 `array.insert(index, value)` 增长，并为有类型空 Array 写 `let xs: Array<Integer> = []` 或 `Array<Integer>.new()`。 | D-459 防止读取分配或改变结构，并保持 Array 泛型不变性显式。 |
| IRIS-V1-MIG-014 | 遗留 `switch` 和 `when` | `legacy/Document/Iris Revival Design Review.md:260`; `legacy/Document/IrisLangHighLight(for NP++).xml:27`; `Iris Library Test/test script/statement/test_switch_statement.ir:1`; `spec/drafts/iris-language-specification.md:470` | `removed` | [IRIS-V1-GRAMMAR-C053B](02-lexical-grammar.md), [IRIS-V1-CONTROL-C050](04-bindings-callables-control-flow.md), [IRIS-V1-CONTROL-C054](04-bindings-callables-control-flow.md) | IRIS-V1-MIGRATION-EX014：遗留 `switch value { when 1 { "one" } else { "other" } }` 变为 `match value { 1 => "one", else => "other" }`。 | D-441 用穷尽、源顺序的 `match` 替换 switch，并且不保留 fallthrough。 |
| IRIS-V1-MIG-015 | 历史 `elseif` 拼写 | `legacy/Document/IrisLangHighLight(for NP++).xml:27`; `Iris Library Test/test script/statement/test_conditionif_statement.ir:15` | `removed` | [IRIS-V1-GRAMMAR-C049](02-lexical-grammar.md), [IRIS-V1-CONTROL-C041](04-bindings-callables-control-flow.md) | IRIS-V1-MIGRATION-EX015：遗留 `elseif condition { body }` 变为 `else if condition { body }`。 | V1 语法使用普通嵌套 `else if`，并且不保留 `elseif`。 |
| IRIS-V1-MIG-016 | 历史 `iterator` 关键字 | `legacy/Document/IrisLangHighLight(for NP++).xml:27`; `Iris Library Test/test script/statement/test_function_statement.ir:21`; `Iris Library Test/test script/statement/test_closure_block.ir:12` | `intentional-divergence` | [IRIS-V1-COLLECTIONS-C011](06-collections-text-regex.md), [IRIS-V1-COLLECTIONS-C012](06-collections-text-regex.md), [IRIS-V1-CONTROL-C044](04-bindings-callables-control-flow.md) | IRIS-V1-MIGRATION-EX016：遗留块头例如 `iterator => [e] : body` 变为类型化 Closure 参数，例如 `{ \|e: Object\| -> Nil; body }`，或 `for e in iterable { body }` 循环。 | D-466 把 `Iterator<T>` 和 `Iteration<T>` 作为 Contracts 和值，而不是块头中的源关键字。 |
| IRIS-V1-MIG-017 | 遗留访问器声明词 `get`、`set` 和 `gset` | `legacy/Document/IrisLangHighLight(for NP++).xml:27`; `Iris Library Test/test script/statement/test_getter_setter_gsetter_statement.ir:2`; `Iris Library Test/test script/expression/test_member_expression.ir:2` | `intentional-divergence` | [IRIS-V1-GRAMMAR-C049](02-lexical-grammar.md), [IRIS-V1-RUNTIME-C061](03-runtime-object-model.md), [IRIS-V1-RUNTIME-C065](03-runtime-object-model.md) | IRIS-V1-MIGRATION-EX017：遗留 `get [@foo]` 和 `set [@foo]` 变为 `property fun foo() -> T { ... }` 和 `property fun foo=(value: T) -> Nil { ... }`，或在适用处使用存储属性简写。 | V1 把属性建模为显式 Method 选择器，只有在声明处才有类型化存储。 |
| IRIS-V1-MIG-018 | Module 声明外的自由可执行语句 | `Iris Library Test/test script/main.ir`; `Iris Library Test/test script/expression/test_binary_expression.ir:1`; `spec/drafts/iris-language-specification.md:503` | `intentional-divergence` | [IRIS-V1-META-C011](08-modules-metaprogramming.md), [IRIS-V1-CONTROL-C012](04-bindings-callables-control-flow.md), [IRIS-V1-TYPES-C003](05-types-contracts-generics.md) | IRIS-V1-MIGRATION-EX018：把旧文件级代码包在显式 `module App { ... }` 中；当导入者需要时，用 `export fun` 导出公共入口 Methods。 | D-474 要求可执行内容由 Module 拥有，并且有一个默认私有的 `main` 接收者。 |
| IRIS-V1-MIG-019 | 遗留 Hash 字面量 `{ key => value }` 以及与 Range 箭头 `=>` 的冲突 | `legacy/Document/IrisLangHighLight(for NP++).xml:16`; `Iris Library Test/test script/expression/test_hash_expression.ir:4`; `Iris Library Test/test script/statement/test_for_statement.ir:12`; `spec/drafts/iris-language-specification.md:490` | `intentional-divergence` | [IRIS-V1-GRAMMAR-C021](02-lexical-grammar.md), [IRIS-V1-GRAMMAR-C040](02-lexical-grammar.md), [IRIS-V1-COLLECTIONS-C027](06-collections-text-regex.md) | IRIS-V1-MIGRATION-EX019：遗留 `{ 'a' => 1 }` 变为 `%{ 'a': 1 }`；当 `name` 是绑定表达式时，遗留 `{ name => value }` 变为 `%{ name: value }`。 | D-461 给 Hash 一个不同开头和表达式键，而 `=>` 保留给 match 分支。 |
| IRIS-V1-MIG-020 | 遗留或仅高亮器的 `alias` 关键字 | `legacy/Document/IrisLangHighLight(for NP++).xml:27`; `spec/drafts/iris-language-specification.md:538` | `removed` | [IRIS-V1-GRAMMAR-C014](02-lexical-grammar.md), [IRIS-V1-RUNTIME-C037](03-runtime-object-model.md), [IRIS-V1-META-C081](08-modules-metaprogramming.md) | IRIS-V1-MIGRATION-EX020：在已授权 open transaction 中使用元编程 API，例如 `alias_method(:new_name, :old_name)`，而不是源关键字 `alias`。 | D-509 把 alias 功能留给 Method API，并且不保留 `alias`。 |
| IRIS-V1-MIG-021 | 仅高亮器的 `retry` 和 `redo` | `legacy/Document/IrisLangHighLight(for NP++).xml:27`; `spec/drafts/iris-language-specification.md:538` | `removed` | [IRIS-V1-GRAMMAR-C014](02-lexical-grammar.md), [IRIS-V1-CONTROL-C043](04-bindings-callables-control-flow.md), [IRIS-V1-CONTROL-C055](04-bindings-callables-control-flow.md) | IRIS-V1-MIGRATION-EX021：写带状态的显式循环或显式 `try` 重试逻辑，例如 `while attempts < limit { try { break work() } catch e { attempts += 1 } }`。 | 没有冻结决策采纳 Ruby 风格隐式循环或异常重试控制转移。 |
| IRIS-V1-MIG-022 | 仅高亮器的 `goto` | `legacy/Document/IrisLangHighLight(for NP++).xml:27`; `spec/drafts/iris-language-specification.md:538` | `removed` | [IRIS-V1-GRAMMAR-C014](02-lexical-grammar.md), [IRIS-V1-CONTROL-C048](04-bindings-callables-control-flow.md), [IRIS-V1-CONTROL-C049](04-bindings-callables-control-flow.md) | IRIS-V1-MIGRATION-EX022：把计划的 `goto label` 替换为结构化 `while`、`for`、`break label: value`、`continue label`、`return` 或 `raise`。 | V1 只有结构化控制。标签指向循环，不能任意跳转。 |
| IRIS-V1-MIG-023 | 仅高亮器的 `static` | `legacy/Document/IrisLangHighLight(for NP++).xml:27`; `spec/drafts/iris-language-specification.md:538` | `removed` | [IRIS-V1-GRAMMAR-C014](02-lexical-grammar.md), [IRIS-V1-RUNTIME-C043](03-runtime-object-model.md), [IRIS-V1-TYPES-C064](05-types-contracts-generics.md) | IRIS-V1-MIGRATION-EX023：用 `class fun name(...)` 表示 Class 对象 Methods，用 `class let @@x` 或 `class mut @@x` 表示层级变量，并且只在泛型存储条款允许时使用 `shared class property`。 | V1 区分 Class 对象行为、层级存储和泛型共享存储，而不是使用宽泛的 `static` 关键字。 |
| IRIS-V1-MIG-024 | D-509 历史 `and`、`or` 和 `not` 词别名以及符号逻辑形式 | `spec/drafts/iris-language-specification.md:538`; `legacy/Document/IrisLangHighLight(for NP++).xml:16` | `removed` | [IRIS-V1-GRAMMAR-C014](02-lexical-grammar.md), [IRIS-V1-RUNTIME-C028](03-runtime-object-model.md), [IRIS-V1-CONTROL-C040](04-bindings-callables-control-flow.md) | IRIS-V1-MIGRATION-EX024：写 `!ready`、`a && b` 和 `a \|\| b`；不要写 `not ready`、`a and b` 或 `a or b`。 | D-351 冻结不可重载的符号逻辑控制形式，D-509 排除词别名。这些词是 D-509 历史，不是 Notepad++ `Keywords1` 条目。 |
| IRIS-V1-MIG-025 | D-509 `repeat` 和评审提出的 `repeat 10 as i` | `legacy/Document/Iris Revival Design Review.md:281`; `spec/drafts/iris-language-specification.md:538` | `removed` | [IRIS-V1-GRAMMAR-C014](02-lexical-grammar.md), [IRIS-V1-CONTROL-C044](04-bindings-callables-control-flow.md), [IRIS-V1-COLLECTIONS-C006](06-collections-text-regex.md) | IRIS-V1-MIGRATION-EX025：写 `for i in 1 ..= 10 { body }`，而不是 `repeat 10 as i { body }`。 | 评审提案未被采纳。D-438 把 v1 循环固定为 `while` 和 `for`。`repeat` 是 D-509 历史，不是 Notepad++ `Keywords1` 条目。 |
| IRIS-V1-MIG-026 | 评审提到的 `defer` 或 scope-guard 语句 | `legacy/Document/Iris Revival Design Review.md:415`; `spec/drafts/iris-language-specification.md:497`; `spec/drafts/iris-language-specification.md:538` | `removed` | [IRIS-V1-CONTROL-C055](04-bindings-callables-control-flow.md), [IRIS-V1-CONTROL-C063](04-bindings-callables-control-flow.md), [IRIS-V1-ASYNC-C030](07-async-resources-diagnostics.md), [IRIS-V1-ASYNC-C032](07-async-resources-diagnostics.md) | IRIS-V1-MIGRATION-EX026：使用 `try { body } finally { cleanup }`、迭代器自动关闭、显式 `close()`，或可用时的普通标准 `using(resource, &block)` helper。 | D-468 拒绝为 v1 添加新的 `defer` 语句。清理由现有控制和资源协议表达。 |
| IRIS-V1-MIG-027 | D-509 `undef` 和 Ruby 风格方法隐藏语法 | `spec/drafts/iris-language-specification.md:538`; `spec/drafts/iris-language-specification.md:477` | `removed` | [IRIS-V1-GRAMMAR-C014](02-lexical-grammar.md), [IRIS-V1-RUNTIME-C037](03-runtime-object-model.md), [IRIS-V1-META-C081](08-modules-metaprogramming.md) | IRIS-V1-MIGRATION-EX027：在已授权 open transaction 中使用 `undef_method(:selector)`，而不是源关键字 `undef`。 | V1 通过 Method API 保留 remove 和 undef 语义，而不是通过保留字。`undef` 是 D-509 历史，不是 Notepad++ `Keywords1` 条目。 |
| IRIS-V1-MIG-028 | `const` 和 `global` 作为遗留词，声明语义已改变 | `legacy/Document/IrisLangHighLight(for NP++).xml:27`; `Iris Library Test/test script/expression/test_identifier_expression.ir:1`; `spec/drafts/iris-language-specification.md:462`; `spec/drafts/iris-language-specification.md:463` | `intentional-divergence` | [IRIS-V1-GRAMMAR-C013](02-lexical-grammar.md), [IRIS-V1-CONTROL-C008](04-bindings-callables-control-flow.md), [IRIS-V1-META-C015](08-modules-metaprogramming.md) | IRIS-V1-MIGRATION-EX028：遗留裸 `CONSTANCE = 10` 变为 `const CONSTANCE: Integer = 10`；遗留 `$global_var = 5.0` 变为 `global mut $global_var: Float64 = 5.0`。 | V1 保留常量和全局变量，但要求显式声明、包限定和类型化 contract。 |
| IRIS-V1-MIG-029 | 修订后的带标签 break 语法和遗留无标签 break 来源 | `spec/drafts/iris-language-specification.md:469`; `Iris Library Test/test script/statement/test_break_statement.ir:7`; `Iris Library Test/test script/statement/test_break_statement.ir:11` | `intentional-divergence` | [IRIS-V1-GRAMMAR-C053A](02-lexical-grammar.md), [IRIS-V1-CONTROL-C048](04-bindings-callables-control-flow.md), [IRIS-V1-CONTROL-C049](04-bindings-callables-control-flow.md) | IRIS-V1-MIGRATION-EX029：遗留无标签嵌套 `break` 仍是 `break`；v1 带标签值 break 是 `outer: for item in items { break outer: result }`，带标签 continue 是 `continue outer`。 | 所引遗留脚本行只显示无标签嵌套 `break`，不显示带标签 break。D-440 是修订后带标签语法的真实来源；标签后的冒号避免解析器依赖符号表标签查找。 |
| IRIS-V1-MIG-030 | 仅 PDF 或仅评审的语法，未由冻结 v1 决策佐证 | `legacy/Document/Iris Programming Language Intro.pdf`; `legacy/Document/Iris Revival Design Review.md:386`; `legacy/Document/Iris Revival Design Review.md:397` | `removed` | [IRIS-V1-MIGRATION-C002](11-migration-divergence.md), [IRIS-V1-MIGRATION-C003](11-migration-divergence.md), [IRIS-V1-TRACE-C013](README.md) | IRIS-V1-MIGRATION-EX030：当旧 PDF 文本与脚本或冻结条款不一致时，迁移到冻结 v1 条款，例如 PDF Range `->` 变为 `..=` 或 `..<`，PDF 权限 `native` 只在旧意图匹配 protected 可见性时变为 `protected`。 | 评审明确把 PDF 视为早期设计制品。没有佐证的仅 PDF 语法不能被保留。 |
| IRIS-V1-MIG-031 | `super` 示例漂移，以及遗留 superclass 或 Module 查找规范不足 | `legacy/Document/Iris Revival Design Review.md:365`; `legacy/Document/Iris Revival Design Review.md:402`; `Iris Library Test/test script/statement/test_super_statement.ir:7`; `Iris Library Test/test script/statement/test_super_statement.ir:9` | `intentional-divergence` | [IRIS-V1-RUNTIME-C014](03-runtime-object-model.md), [IRIS-V1-RUNTIME-C046](03-runtime-object-model.md), [IRIS-V1-RUNTIME-C081](03-runtime-object-model.md) | IRIS-V1-MIGRATION-EX031：遗留 `class B extends A { fun foo() { super("msg") } }` 仍是显式 `super("msg")`，但 v1 通过当前 MRO 解析它，并以 `NoSuperMethodError` 拒绝裸 `super` 或缺失后继。 | 遗留脚本演示显式 `extends A` 加 `super`；评审指出一个 PDF 示例缺少 `extends A`，且 Module MRO 规范不足。V1 把 MRO 继续规则设为规范。 |
| IRIS-V1-MIG-032 | PDF 语句清单漂移：声称有 16 种语句却编号到 18 | `legacy/Document/Iris Revival Design Review.md:397`; `legacy/Document/Iris Revival Design Review.md:399` | `removed` | [IRIS-V1-GRAMMAR-C049](02-lexical-grammar.md), [IRIS-V1-MIGRATION-C002](11-migration-divergence.md), [IRIS-V1-MIGRATION-C003](11-migration-divergence.md) | IRIS-V1-MIGRATION-EX032：使用 v1 `statement` 产生式列表，而不是继承 PDF 语句数量。 | 这是文档漂移，不是要保留的行为。v1 语法拥有完整语句清单。 |
| IRIS-V1-MIG-033 | PDF Range 遍历说明说遍历尚未添加 | `legacy/Document/Iris Revival Design Review.md:397`; `legacy/Document/Iris Revival Design Review.md:400` | `intentional-divergence` | [IRIS-V1-COLLECTIONS-C038](06-collections-text-regex.md), [IRIS-V1-COLLECTIONS-C039](06-collections-text-regex.md), [IRIS-V1-CONTROL-C044](04-bindings-callables-control-flow.md) | IRIS-V1-MIGRATION-EX033：把任何 PDF 时代不可遍历 Range 假设替换为 `for i in 1 ..= 10 { ... }`，它使用规范 `Iterator<T>` 和 `Iteration<T>` 遍历规则。 | V1 保留 Range 作为值，并定义整数 Range 迭代；旧的“not added”说明只是考古材料。 |
| IRIS-V1-MIG-034 | PDF `display()` 与 `show()` 命名不一致 | `legacy/Document/Iris Revival Design Review.md:397`; `legacy/Document/Iris Revival Design Review.md:401` | `removed` | [IRIS-V1-COLLECTIONS-C049](06-collections-text-regex.md), [IRIS-V1-COLLECTIONS-C050](06-collections-text-regex.md), [IRIS-V1-MIGRATION-C002](11-migration-divergence.md) | IRIS-V1-MIGRATION-EX034：不要把任一名称迁移为核心显示原语；在标准 API 要求处，用 `to_string()` 做用户文本转换，用 `inspect()` 做调试风格表示。 | 不一致名称是 PDF 文档漂移。V1 通过 `to_string` 和 `inspect` 标准化对象文本表面，而不是引入仅 PDF 的显示动词。 |
| IRIS-V1-MIG-035 | PDF `i`/`I` 和 `integer`/`interger` 拼写漂移 | `legacy/Document/Iris Revival Design Review.md:397`; `legacy/Document/Iris Revival Design Review.md:403` | `removed` | [IRIS-V1-GRAMMAR-C009](02-lexical-grammar.md), [IRIS-V1-RUNTIME-C101](03-runtime-object-model.md), [IRIS-V1-MIGRATION-C002](11-migration-divergence.md) | IRIS-V1-MIGRATION-EX035：把拼错的类型文本例如 `interger` 迁移为规范 `Integer`；按 v1 大小写敏感标识符规则，把 `i` 和 `I` 等大小写变体视为不同标识符。 | 这些是拼写考古材料，不是别名。V1 保持规范 `Integer` 和大小写敏感标识符。 |
| IRIS-V1-MIG-036 | PDF 使用十六进制字面量，但未在字面量章节定义它们 | `legacy/Document/Iris Revival Design Review.md:397`; `legacy/Document/Iris Revival Design Review.md:404` | `intentional-divergence` | [IRIS-V1-GRAMMAR-C024](02-lexical-grammar.md), [IRIS-V1-GRAMMAR-C026](02-lexical-grammar.md), [IRIS-V1-GRAMMAR-C028](02-lexical-grammar.md) | IRIS-V1-MIGRATION-EX036：整数十六进制字面量使用 v1 定义的 `0xFF`，`0x1.fp3` 风格只在满足 v1 十六进制浮点字面量规则时使用。 | V1 保留十六进制源形式，但明确其词法定义和无效数字诊断。 |
| IRIS-V1-MIG-037 | PDF Block closure 和 metaprogramming 章节只有标题，没有正式语义 | `legacy/Document/Iris Revival Design Review.md:397`; `legacy/Document/Iris Revival Design Review.md:405` | `intentional-divergence` | [IRIS-V1-CONTROL-C016](04-bindings-callables-control-flow.md), [IRIS-V1-CONTROL-C030](04-bindings-callables-control-flow.md), [IRIS-V1-META-C022](08-modules-metaprogramming.md), [IRIS-V1-META-C030](08-modules-metaprogramming.md) | IRIS-V1-MIGRATION-EX037：把 PDF 时代只有标题的块概念迁移为显式 Closure 语法 `{ \|value: T\| -> R; body }`，并把元编程迁移到 `open class` 或 `open module` transactions。 | V1 通过冻结的 Closure 和元编程 transaction 语义填补这些领域，而不是保留无文档标题。 |

## 已移除词处置索引

IRIS-V1-MIGRATION-C009：下列索引对需要迁移处置，但不能仅因历史而作为 v1 保留关键字接受的词具有规范性。`Source class` 说明该词是否出现在精确的 Notepad++ `Keywords1` 清单中、只在 D-509/历史中，或两者都有。替代链接指向上面的台账行。

| 词 | 来源类别 | 处置 | 台账行 |
| --- | --- | --- | --- |
| `alias` | `Keywords1 and D-509` | `removed` | IRIS-V1-MIG-020 |
| `and` | `D-509 only` | `removed` | IRIS-V1-MIG-024 |
| `block` | `Keywords1` | `intentional-divergence` | IRIS-V1-MIG-003 |
| `cast` | `Keywords1` | `intentional-divergence` | IRIS-V1-MIG-004 |
| `const` | `Keywords1 current keyword` | `intentional-divergence` | IRIS-V1-MIG-028 |
| `defer` | `D-509 only` | `removed` | IRIS-V1-MIG-026 |
| `elseif` | `Keywords1` | `removed` | IRIS-V1-MIG-015 |
| `everyone` | `Keywords1` | `intentional-divergence` | IRIS-V1-MIG-005 |
| `get` | `Keywords1` | `intentional-divergence` | IRIS-V1-MIG-017 |
| `global` | `Keywords1 current keyword` | `intentional-divergence` | IRIS-V1-MIG-028 |
| `goto` | `Keywords1 and D-509` | `removed` | IRIS-V1-MIG-022 |
| `groan` | `Keywords1 and D-509` | `removed` | IRIS-V1-MIG-006 |
| `gset` | `Keywords1` | `intentional-divergence` | IRIS-V1-MIG-017 |
| `ignore` | `Keywords1 and D-509` | `removed` | IRIS-V1-MIG-007 |
| `implements` | `D-509 only` | `removed` | IRIS-V1-MIG-009 |
| `interface` | `Keywords1 and D-509` | `intentional-divergence` | IRIS-V1-MIG-008 |
| `involve` | `review drift only` | `intentional-divergence` | IRIS-V1-MIG-010 |
| `involves` | `Keywords1` | `intentional-divergence` | IRIS-V1-MIG-010 |
| `iterator` | `Keywords1` | `intentional-divergence` | IRIS-V1-MIG-016 |
| `joints` | `Keywords1` | `intentional-divergence` | IRIS-V1-MIG-010 |
| `native` | `D-509 only` | `removed` | IRIS-V1-MIG-005 |
| `not` | `D-509 only` | `removed` | IRIS-V1-MIG-024 |
| `or` | `D-509 only` | `removed` | IRIS-V1-MIG-024 |
| `order` | `Keywords1 and D-509` | `removed` | IRIS-V1-MIG-007 |
| `personal` | `Keywords1` | `intentional-divergence` | IRIS-V1-MIG-005 |
| `redo` | `Keywords1 and D-509` | `removed` | IRIS-V1-MIG-021 |
| `relative` | `Keywords1` | `intentional-divergence` | IRIS-V1-MIG-005 |
| `repeat` | `D-509 only` | `removed` | IRIS-V1-MIG-025 |
| `retry` | `Keywords1 and D-509` | `removed` | IRIS-V1-MIG-021 |
| `satisfies` | `D-509 only` | `removed` | IRIS-V1-MIG-009 |
| `serve` | `Keywords1 and D-509` | `removed` | IRIS-V1-MIG-007 |
| `set` | `Keywords1` | `intentional-divergence` | IRIS-V1-MIG-017 |
| `static` | `Keywords1 and D-509` | `removed` | IRIS-V1-MIG-023 |
| `switch` | `Keywords1 and D-509` | `removed` | IRIS-V1-MIG-014 |
| `undef` | `D-509 only` | `removed` | IRIS-V1-MIG-027 |
| `when` | `Keywords1 and D-509` | `removed` | IRIS-V1-MIG-014 |
| `with` | `Keywords1` | `intentional-divergence` | IRIS-V1-MIG-003 |
| `without` | `Keywords1` | `intentional-divergence` | IRIS-V1-MIG-003 |

## 完整高亮器清单分类

IRIS-V1-MIGRATION-C011：Notepad++ 高亮器在 `Keywords1` 中恰好包含本表列出的 44 个词，且没有其他词。这个精确清单是考古材料，不是 v1 关键字 contract。`and`、`defer`、`implements`、`involve`、`native`、`not`、`or`、`repeat`、`satisfies` 和 `undef` 等额外词不是 Notepad++ `Keywords1` 词，并且在 IRIS-V1-MIGRATION-C012 中单独分类。

| 类别 | 高亮器词 | 处置规则 |
| --- | --- | --- |
| `current-keyword-preserved` | `break`, `class`, `continue`, `else`, `extends`, `false`, `for`, `fun`, `if`, `in`, `module`, `nil`, `return`, `self`, `super`, `true` | 这些拼写在 [IRIS-V1-GRAMMAR-C013](02-lexical-grammar.md) 下仍是 v1 关键字，但其 v1 语义只由当前规范章节支配。 |
| `current-keyword-with-revised-semantics` | `const`, `global` | 这些拼写仍是 v1 关键字，但从遗留隐式用法或仅高亮器用法迁移时遵循 IRIS-V1-MIG-028。 |
| `intentional-divergence` | `block`, `cast`, `everyone`, `get`, `gset`, `interface`, `involves`, `iterator`, `joints`, `personal`, `relative`, `set`, `with`, `without` | 这些精确高亮器词不作为其历史角色的 v1 关键字接受，并由 IRIS-V1-MIGRATION-C009 映射到替代行。 |
| `removed` | `alias`, `elseif`, `goto`, `groan`, `ignore`, `order`, `redo`, `retry`, `serve`, `static`, `switch`, `when` | 这些精确高亮器词不能仅因历史而在 [IRIS-V1-GRAMMAR-C014](02-lexical-grammar.md) 下被保留，并且没有永久兼容别名。 |

## 历史和 D-509 非高亮器额外项

IRIS-V1-MIGRATION-C012：下列词是迁移相关的历史词或 D-509 词，但不存在于精确的 Notepad++ `Keywords1` 清单中。它们 MUST NOT 在 C011 验证中计为高亮器词。

| 词 | 来源 | 处置 | 台账行 |
| --- | --- | --- | --- |
| `and` | `D-509` | `removed` | IRIS-V1-MIG-024 |
| `defer` | `D-509` | `removed` | IRIS-V1-MIG-026 |
| `implements` | `D-509` | `removed` | IRIS-V1-MIG-009 |
| `involve` | `legacy/Document/Iris Revival Design Review.md:386` | `intentional-divergence` | IRIS-V1-MIG-010 |
| `native` | `legacy/Document/Iris Revival Design Review.md:386` and `D-509` | `removed` | IRIS-V1-MIG-005 |
| `not` | `D-509` | `removed` | IRIS-V1-MIG-024 |
| `or` | `D-509` | `removed` | IRIS-V1-MIG-024 |
| `repeat` | `D-509` | `removed` | IRIS-V1-MIG-025 |
| `satisfies` | `D-509` | `removed` | IRIS-V1-MIG-009 |
| `undef` | `D-509` | `removed` | IRIS-V1-MIG-027 |

## 证据覆盖说明

IRIS-V1-MIGRATION-N001：Historical note: 所需评审漂移列表由 IRIS-V1-MIG-001 到 IRIS-V1-MIG-014，以及 IRIS-V1-MIG-029 到 IRIS-V1-MIG-037 覆盖。精确的 Notepad++ `Keywords1` 清单由 IRIS-V1-MIGRATION-C011 分类，而 D-509 和仅评审的非高亮器额外项由 IRIS-V1-MIGRATION-C012 分类。

IRIS-V1-MIGRATION-N002：Historical note: 旧 PDF 无法在此环境中机械重新提取，因此台账使用设计评审中的 PDF 考古引用和漂移表作为可引用的 PDF 证据。IRIS-V1-MIGRATION-C003 中的保留规则防止未经佐证的仅 PDF 语法取得规范性权威。

IRIS-V1-MIGRATION-C010：本章合并 D-001、D-034、D-145、D-146、D-278、D-362、D-438、D-440、D-441、D-447、D-459、D-460、D-461、D-462、D-463、D-464、D-469、D-509 和 D-510 中要求或解释遗留分歧的内容。它还记录来自 `legacy/Document/Iris Revival Design Review.md`、`legacy/Document/IrisLangHighLight(for NP++).xml`、`legacy/Document/Iris Programming Language Intro.pdf` 和 `Iris Library Test/test script/` 的历史证据，但不赋予这些来源规范性权威。
