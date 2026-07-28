# Iris v1 集合、文本、二进制、Regex 与稳定哈希

状态：Iris v1 草案，语义已冻结。

IRIS-V1-COLLECTIONS-C001: 本章定义 Iris v1 的 Tuple、Array、Hash、Range、Iterable、Iterator、Iteration、String、MutableString、Symbol、Bytes、ByteArray、Regex、Match、mutation、fail-fast traversal、equality、hashability，以及稳定 BLAKE3 hashing。必须在阅读 [README.md](README.md)、[01-language-identity.md](01-language-identity.md)、[02-lexical-grammar.md](02-lexical-grammar.md)、[03-runtime-object-model.md](03-runtime-object-model.md) 和 [04-bindings-callables-control-flow.md](04-bindings-callables-control-flow.md) 之后阅读本章。

IRIS-V1-COLLECTIONS-C002: 本章不得定义实现存储、bucket layout、rope layout、regex engine internals、Unicode table generation、parser productions、type algebra、serialization byte schemas，或 native ABI details。它细化前面章节已经固定的 literal、indexing、iteration、equality 和 public hash surface。

## 核心分类

IRIS-V1-COLLECTIONS-C003: 下表是 core value 和 container classification 的规范要求。`Specification-stable` 表示只要仍选择内建 hash Method，公共 `hash` 值就由本规范固定。`Runtime-local` 表示只在 [03-runtime-object-model.md](03-runtime-object-model.md) 下某个 runtime object identity 的生命周期内稳定。`Unhashable` 表示内建公共 `hash` 会引发 `InvalidKeyError`。

| 类别 | 身份 | 可变性 | 内建相等性 | 内建公共哈希 | 作为哈希键的资格 | Iterator 元素形状 |
| --- | --- | --- | --- | --- | --- | --- |
| `nil` | 带 identity 的 singleton | 固定 singleton | singleton 相等 | 来自 [03-runtime-object-model.md](03-runtime-object-model.md) 的 Specification-stable singleton hash | 是 | 默认不可迭代 |
| `false`, `true` | 带 identity 的 singleton | 固定 singleton | Bool 值相等 | 来自 [03-runtime-object-model.md](03-runtime-object-model.md) 的 Specification-stable singleton hash | 是 | 默认不可迭代 |
| `Integer` | 无 identity | 不可变 | 精确 numeric value equality | 来自 [03-runtime-object-model.md](03-runtime-object-model.md) 的 Specification-stable numeric hash | 是，除非未来某个 numeric value 在 runtime rules 下无效 | 默认不可迭代 |
| `Float32`, `Float64` finite or infinity | 无 identity | 不可变 | 精确 numeric value equality，signed zeros 相等 | 来自 [03-runtime-object-model.md](03-runtime-object-model.md) 的 Specification-stable numeric hash | 是，NaN 除外 | 默认不可迭代 |
| `Float32`, `Float64` NaN | 无 identity | 不可变 | 不等于任何值 | `InvalidKeyError` | 否 | 默认不可迭代 |
| Class, Module, Contract, Method, BoundMethod, Closure, Type, revision, ordinary identity object | 带 identity | 按类别而定 | 默认 identity-first，除非被替换 | Runtime-local identity hash，除非类别定义 stable hash | 当前 `hash` 成功时是 | 按类别而定 |
| Contract view | 无 identity | 不可变 | Receiver relation 加 Contract identity | 来自 [03-runtime-object-model.md](03-runtime-object-model.md) 的 Specification-stable Contract-view hash | receiver hash 成功时是 | 默认不可迭代 |
| `String` | 无 identity | 不可变 | 精确 Unicode scalar sequence 和大小写 | Specification-stable string hash | 是 | 单 scalar immutable `String` |
| `MutableString` | 带 identity | 可变 text content | 当前 scalar content，跨类型等于 `String` | 默认 Unhashable | 默认否 | 单 scalar immutable `String`，content mutation 时 fail-fast |
| `Symbol` | 无 identity | 不可变 canonical content | 精确 canonical content | Specification-stable symbol hash | 是 | 默认不可迭代 |
| `Bytes` | 无 identity | 不可变 byte sequence | 精确 byte sequence，跨类型等于 `ByteArray` | Specification-stable bytes hash | 是 | `0..255` 中的 `Integer` byte |
| `ByteArray` | 带 identity | 可变 byte sequence | 当前 byte content，跨类型等于 `Bytes` | 默认 Unhashable | 默认否 | `0..255` 中的 `Integer` byte，content mutation 时 fail-fast |
| `Tuple<T...>` | 无 identity | 不可变 element sequence | Element order 和 dynamic element equality | 每个 element hash 都成功时为 Specification-stable tuple hash | 取决于每个 element | 按顺序的 elements |
| `Array<T>` | 带 identity | 可变 element sequence | 当前 element order 和 dynamic element equality | 默认 Unhashable | 默认否 | 按顺序的 elements，element replacement 或 structural mutation 时 fail-fast |
| `Hash<K,V>` | 带 identity | 可变 entry set 和 values | 默认 identity-first，除非被替换 | Runtime-local identity hash，除非被替换 | 以 identity 默认规则为是 | 未指定顺序的 `Tuple<K,V>` entries，structural fail-fast |
| `Range` | 无 identity | 不可变 endpoints、openness、step | Endpoint、openness 和 step equality | components hash 成功时为 Specification-stable range hash | 取决于 components | integer Ranges 的 Integer sequence |
| `Iterator<T>` | 带 identity | 可变 cursor state | 默认 identity-only | Runtime-local identity hash | 以 identity 默认规则为是 | 默认不遍历自身，除非也实现 `Iterable` |
| `Iteration<T>` yield | 无 identity | 不可变 variant 和 payload | Yield variant 加 payload dynamic equality | payload hash 成功时为 Specification-stable iteration hash | 取决于 payload | 默认不可迭代 |
| `Iteration.done` | 带 identity 的 singleton | 固定 singleton | singleton 相等 | Specification-stable iteration hash | 是 | 默认不可迭代 |
| `Regex` | 无 identity | 不可变 pattern 和 flags | Canonical pattern 加 canonical flags | Specification-stable regex hash | 是 | 默认不可迭代 |
| `Match` | 无 identity | 不可变 match result | Full match、range 和 capture value equality | 默认 Unhashable | 默认否 | 只能通过显式 APIs 访问 captures |

IRIS-V1-COLLECTIONS-C004: Equal-implies-same-hash 只在两个公共 `hash` 调用都成功时适用。一个值可以等于可哈希值，同时自身仍Unhashable，例如 `MutableString` 与 `String` 比较，或 `ByteArray` 与 `Bytes` 比较。

IRIS-V1-COLLECTIONS-C005: Hash table 选择 bucket 时，必须用带运行时秘密的内部混合来处理公共哈希值。内部混合不得改变公共 `hash` 输出、相等性、迭代顺序或持久化格式；公共 `hash` 的结果必须仍然是本章或 [03-runtime-object-model.md](03-runtime-object-model.md) 定义的值。

## Index Units 与 Range Tokens

IRIS-V1-COLLECTIONS-C006: grammar tokens `..=` 和 `..<` 是仅有的 Range literal operators。`a ..= b` 创建 left-closed inclusive-end integer Range。`a ..< b` 创建 left-closed exclusive-end integer Range。`..identifier` Contract-view token 仍是 postfix Contract-view syntax，不得解析为 Range。

IRIS-V1-COLLECTIONS-C007: Range literal endpoints 必须是 `Integer` values。非 literal 或 fully open intervals 的通用 Range construction 属于显式 APIs。V1 source 没有 fully open、left-open、stepped 或 reverse Range literal token。

IRIS-V1-COLLECTIONS-C008: Indexing units 由 receiver category 固定。`String` 和 `MutableString` 计数 Unicode scalar values。`Bytes` 和 `ByteArray` 计数字节。`Tuple` 和 `Array` 计数 element positions。Integer Range iteration 计数 integer values。任何 receiver 都不会在 scalar、byte、grapheme 和 element units 之间隐式切换。

IRIS-V1-COLLECTIONS-C009: 本章允许 indexed access 的位置，Integer indexes 可以为负。负 index 按 receiver indexing unit 解析为 `length + index`。有效解析范围之外的 reads 返回 `nil`，除非更具体的 write rule 会引发。对 mutable indexed receivers 的 out-of-range scalar writes 引发 `IndexError`。

IRIS-V1-COLLECTIONS-C010: Unit-forward Range slicing 会按 receiver unit 解析负 endpoints，将有效 endpoints clamp 到 `[0, length]`，尊重 inclusive 或 exclusive end openness，并在有效 start 位于有效 end 之后时返回空的同族结果。V1 slice syntax 没有 step 或 reverse traversal semantics。

## Iterable、Iterator 与 Iteration

IRIS-V1-COLLECTIONS-C011: canonical traversal Contracts 精确为:

```iris
contract Iterable<T> {
  fun iterator() -> Iterator<T>
}

contract Iterator<T> {
  fun next() -> Iteration<T>
  fun close() -> Nil
}
```

IRIS-V1-COLLECTIONS-C012: `for pattern in iterable` 必须通过 `Iterable<T>.iterator()` 取得 `Iterator<T>`，重复调用 `next() -> Iteration<T>`，绑定每个 `Iteration.yield(value)` payload，并在 `Iteration.done` 上正常终止。它不得用 exceptions 表示普通 exhaustion。

IRIS-V1-COLLECTIONS-C013: `Iteration.yield(value)` 是 immutable identity-less value object。它可以携带任何 Iris value，包括 `nil`。`Iteration.done` 是唯一的带 identity singleton。Exhaustion 之后重复 `next()` 必须返回同一个 `Iteration.done` singleton。

IRIS-V1-COLLECTIONS-C014: `Iteration` 暴露 get-only properties `yield?`、`done?` 和 `value`。对 `Iteration.yield(payload)`，`yield?` 返回 `true`，`done?` 返回 `false`，`value` 返回 payload。对 `Iteration.done`，`yield?` 返回 `false`，`done?` 返回 `true`，`value` 引发 `IteratorStateError`。

IRIS-V1-COLLECTIONS-C015: 内建 `Iteration#<=>` 对 `done <=> done` 返回 `Integer(0)`，在 `done` 与任何 yield 之间或 `Iteration` 与非 Iteration 之间返回 `nil`，并将 `yield(a) <=> yield(b)` 转发给 payloads 当前 dynamic `<=>`。`Iteration` Method 必须立刻验证转发结果正好是 `Integer(-1)`、`Integer(0)`、`Integer(1)` 或 `nil`，否则引发 `ComparisonContractError`。

IRIS-V1-COLLECTIONS-C016: 每个显式 Iterator object 都是带 identity 的 mutable cursor。每次 `iterator()` 调用都必须创建不同的 Iterator identity，除非 identity observations 保持完全等价。默认 Iterator equality 仅按 identity。Iterator public hash 是 runtime-local identity hash，不得依赖 source position、source container content、expected structural version 或 current entry。

IRIS-V1-COLLECTIONS-C017: Iterator 必须强引用其 source container，直到自然 exhaustion 或 `close()`。第一次返回 `Iteration.done` 的 `next()` 使 Iterator 进入永久 done state，并释放 source、current-entry 和不需要的 traversal references。后续 `next()` 返回 `Iteration.done`，且不触及 source。

IRIS-V1-COLLECTIONS-C018: `Iterator#close() -> Nil` 必须 idempotent。Early close 释放 source、current-entry 和 traversal references，进入永久 done state，并使后续 `next()` 返回 `Iteration.done`。重复 `close()` calls 成功并返回 `nil`。Iterator identity 和 identity hash 保持不变。

IRIS-V1-COLLECTIONS-C019: Language 和 standard traversal constructs 必须在每条 exit path 上 close active iterators，包括 natural exhaustion、`break`、targeting outer loop 的 `continue`、`return`、body exception、Iterator exception、destructuring failure 和 outer unwinding。Cleanup failure precedence 遵循 [04-bindings-callables-control-flow.md](04-bindings-callables-control-flow.md)。

IRIS-V1-COLLECTIONS-C020: Invalid Iterator-specific sequencing 必须引发 `IteratorStateError`。这包括读取 `Iteration.done.value`、在成功 yield 前调用 `HashIterator#remove_current`、对一次 yielded entry 调用两次、在 completion 或 close 后调用，以及在 cursor state 不再允许时使用 concrete cursor operations。

## Container Operations 与 Fail-Fast Rules

IRIS-V1-COLLECTIONS-C021: Tuple literals 是 `()`、`(a,)` 和 `(a, b, ...)`。Tuple values 是 immutable、identity-less、heterogeneous product values，并具有 reified `Tuple<T...>` type。`Tuple#[]` 接受 integer indexes，支持 negative-index，并返回 element 或 out of range 时返回 `nil`。

IRIS-V1-COLLECTIONS-C022: Tuple equality 通过当前 dynamic element `==` 按 arity 和 elements in order 比较。只有每个 element public hash 都成功时，Tuple public hash 才成功。失败的 element hash 会传播，并阻止其用作 Hash key。

IRIS-V1-COLLECTIONS-C023: Array literals `[a, b, ...]` 创建 fresh identity-bearing mutable `Array<T>`。`T` 被推断为 element static types 的 normalized union，或由 expected closed `Array<T>` 约束。空 `[]` 需要 expected closed `Array<T>` context。需要 unconstrained empty Array 的 programs 必须写显式 construction，例如 `Array<T>.new()`。

IRIS-V1-COLLECTIONS-C024: Array integer reads 支持 negative-index，out of range 时返回 `nil`。Array scalar writes 强制 Array element Contract，替换 existing element，返回 `nil`，并在 out of range 时引发 `IndexError`。Append 和 insert 是显式 growth operations。

IRIS-V1-COLLECTIONS-C025: Array Range slicing 使用 unit-forward Range slicing 返回独立 mutable Array snapshot。`array[range] = iterable` 必须在改变 receiver 前 atomically materialize 并 type-check replacement elements。它可以改变长度，成功时返回 `nil`，失败时让 Array 保持不变。

IRIS-V1-COLLECTIONS-C026: Array built-in equality 通过 dynamic equality 按当前 element sequence in order 比较。Array built-in `hash` 引发 `InvalidKeyError`。任何 Array element replacement、append、insert、delete、clear、sort、range assignment 或 length-changing operation 都会递增 Array content version，并使 active Array iterators 在下一次 advance 时引发 `ConcurrentModificationError`。

IRIS-V1-COLLECTIONS-C027: Hash literals 使用 `%{ key: value }`。每个 key position 都是普通 expression。裸 identifier key 读取该 binding，不得变成隐式 Symbol。空 `%{}` 需要 expected `Hash<K,V>` context，或显式 construction 例如 `Hash<K,V>.new()`。

IRIS-V1-COLLECTIONS-C028: Hash insertion、lookup、update、deletion 和 rehash 使用每个 key 当前 dynamically dispatched `hash` 和 `==` Methods。它们不得将 identity-bearing keys 切换到 `same?`，除非当前选择的 equality Method 自身这样做。NaN key attempt 根据 [03-runtime-object-model.md](03-runtime-object-model.md) 引发 `InvalidKeyError`。

IRIS-V1-COLLECTIONS-C029: `hash[key] -> V?` 在 absent 时返回 `nil`，且不得 insert。`hash[key] = value` 强制 key 和 value Contracts，插入或更新，并返回 `nil`。`fetch(key)` 在 absent 时引发 `KeyError`，除非显式 default 或 trailing block form 处理 absence。`delete(key) -> V?` 返回 old value，absent 时返回 `nil`。

IRIS-V1-COLLECTIONS-C030: 现有 Hash containers 不会自动跟踪 `==` 或 `hash` Method version changes。改变 equality 或 hash behavior 的 users 必须调用 `rehash()` 或重建受影响 containers。在此之前，lookup inconsistency 由 user 负责。

IRIS-V1-COLLECTIONS-C031: `Hash#rehash()` 必须基于 current keys、current public hashes 和 current equality 构建并验证 temporary replacement。如果两个先前不同的 entries 现在碰撞到同一 equality class，且没有提供 merge block，它会引发 `KeyConflictError` 并不发布任何 partial result。

IRIS-V1-COLLECTIONS-C032: `Hash#rehash() { |kept_key, kept_value, incoming_key, incoming_value| ... }` 必须使用 block result 作为 collision class 的 two-element `(key, value)` replacement。返回的 key 必须在 current equality 下仍等于该 class，并满足 current hash Contract。Block failure、shape failure 或 new inconsistency 会 atomically abort，并让原 Hash 保持不变。

IRIS-V1-COLLECTIONS-C033: Hash iteration order 未指定。它不得承诺 insertion order、stable-hash order、sorted order、cross-run order 或 deterministic rehash merge order。需要 deterministic ordering 的 programs 必须提取 entries 并通过显式 ordered structure 排序。

IRIS-V1-COLLECTIONS-C034: Hash traversal 对 structural mutation 是 fail-fast。Adding a key、removing a key、clearing 或 rehashing 会递增 structural version。每个 active iterator 的 expected structural version 不再匹配时，必须在 next advance 引发 `ConcurrentModificationError`。更新 existing key 的 value 是 non-structural，允许发生。

IRIS-V1-COLLECTIONS-C035: Hash iterator 将每个 entry 作为 two-element `Tuple<K,V>` yield。Yielded key 和 value 是 yield 时的普通 bound arguments。Tuple yield 后替换 value 不会改写该 tuple。尚未 yielded 的 entry 在 yield 时观察最新 value。

IRIS-V1-COLLECTIONS-C036: Hash concrete iterators 可以提供 `remove_current() -> Nil`。它精确移除该 iterator 最近一次 yielded entry，每次 successful yield 最多一次，更新该 iterator 的 expected structural version，且不让同一个 iterator fail-fast。其他 active iterators 会观察 structural change，并在 next advance fail-fast。

IRIS-V1-COLLECTIONS-C037: `Hash#each` 向 block 精确 yield `(key, value)`。`Hash#each_with_iterator` 精确 yield `(key, value, iterator)`，且是唯一暴露 `remove_current()` 的 standard block traversal surface。不存在 hidden current-iterator context。Nested traversals 使用不同的 Iterator objects。

IRIS-V1-COLLECTIONS-C038: Range values 是 immutable identity-less interval values。Integer Range literals 在 `end >= start` 时推断 step `+1`，在 `end < start` 时推断 `-1`。Equal endpoints 对 `..=` yield 一个值，对 `..<` yield 空。`range.by(step: Integer)` 返回非零 step 且其符号朝向 end 的 Range，否则引发 `RangeError`。

IRIS-V1-COLLECTIONS-C039: Integer Range iteration 尊重 endpoint openness 和 step。未精确落在 endpoint 上的 step 会跳过该 endpoint。Range equality 和 public hash 包含 start、end、endpoint openness 和 step。

IRIS-V1-COLLECTIONS-C040: 下列 collection operation table 是规范要求:

| 接收者 | 读取 | 写入 | 越界或缺失规则 | 结果规则 | Iterator 规则 |
| --- | --- | --- | --- | --- | --- |
| `Tuple` | `tuple[i]` | 无 standard write | 允许 negative index，out-of-range read 返回 `nil` | Element 或 `nil` | Stable element order |
| `Array` | `array[i]`, `array[range]` | `array[i]=v`, `array[range]=iterable`, append, insert, delete, clear | 允许 negative index，out-of-range read 返回 `nil`，out-of-range scalar write 引发 `IndexError` | Scalar write 返回 `nil`，slice 返回 independent Array，range write 返回 `nil` | 任何 element replacement 或 structural mutation fail-fast |
| `Hash` | `hash[key]`, `fetch(key)` | `hash[key]=value`, `delete`, `clear`, `rehash` | Missing `[]` 返回 `nil`，missing `fetch` 引发除非被处理，missing `delete` 返回 `nil` | Setter 返回 `nil`，delete 返回 old value 或 `nil` | Structural mutation fail-fast，existing-value update 允许，owning iterator 有 `remove_current` exception |
| `Range` | Endpoint, step, openness APIs | 无 standard write | 错误 step direction 或 zero step 引发 `RangeError` | Iteration yields Integer values | Stable immutable iteration |
| `String` | `str[i]`, `str[range]` | 无 standard write | 允许 scalar negative index，out-of-range scalar read 返回 `nil`，slice clamps | Scalar read 返回 one-scalar `String?`，slice 返回 `String` | Stable scalar iteration |
| `MutableString` | `text[i]`, `text[range]` | `text[i]=one_scalar`, `text[range]=String`, append, clear, replace, bang transforms | Scalar out-of-range read 返回 `nil`，scalar write out of range 引发 `IndexError` | Writes 返回 `nil` 或 Method 指定的 receiver，snapshots 保持 immutable | 任何 content mutation fail-fast |
| `Bytes` | `bytes[i]`, `bytes[range]` | 无 standard write | 允许 byte negative index，out-of-range read 返回 `nil`，slice clamps | Scalar read 返回 `Integer?`，slice 返回 `Bytes` | Stable byte iteration |
| `ByteArray` | `data[i]`, `data[range]` | `data[i]=byte`, `data[range]=Bytes` 或 `ByteArray`, append, clear, replace | Byte out-of-range read 返回 `nil`，scalar write out of range 引发 `IndexError`，byte value outside `0..255` 引发 `RangeError` | Writes 返回 `nil` 或 Method 指定的 receiver，slice 返回 independent ByteArray | 任何 content mutation fail-fast |

## Text Values 与 Unicode

IRIS-V1-COLLECTIONS-C041: Iris v1 String values 只包含 valid Unicode scalar values。它们不得包含 invalid UTF-8、surrogate code points 或 raw bytes。String literal 中的 `\xNN` 表示 scalar U+0000 到 U+00FF，不是 byte injection。

IRIS-V1-COLLECTIONS-C042: Iris language major version 1 将 identifier XID tables、grapheme segmentation、normalization、Unicode properties、case mapping 和 casefold 所用的默认 Unicode data 固定为 Unicode 17.0.0。修订后的 D-407 已确认这一精确版本。OS libraries、host configuration、process locale 和 package selection 不得改变默认语言结果。若要使用不同的默认 Unicode data version，必须通过未来的 Iris language major 或显式 versioned Unicode API 引入。

IRIS-V1-COLLECTIONS-C043: String equality 比较精确 Unicode scalar sequence 和大小写。它不执行 implicit normalization、case folding、locale mapping 或 grapheme equivalence。`String#hash` 使用 String value 形成后 scalar sequence 的精确 UTF-8 encoding。

IRIS-V1-COLLECTIONS-C044: `String#length`、integer `String#[]`、String Range slicing 和 default String iteration 使用 Unicode scalar units。`String#byte_length` 和 `String#bytes` 显式暴露 UTF-8 bytes。`String#graphemes` 使用固定 Unicode data version 显式暴露 Unicode grapheme clusters。

IRIS-V1-COLLECTIONS-C045: `str[i]` 对 in-range scalar index 返回 one-scalar read-only String value，out of range 时返回 `nil`。V1 没有为此操作引入独立 Character type。

IRIS-V1-COLLECTIONS-C046: `str[range]` 使用 scalar unit-forward Range slicing 返回 read-only String slice。Empty 或 start-after-end slices 返回 empty String。Unsupported stepped 或 reverse slicing 在已知时被 statically rejected，否则在动态时引发 `ArgumentError`。

IRIS-V1-COLLECTIONS-C047: Standard String 没有 `[]=`。Functional text APIs 返回新 String values。User-added String Methods 不得改变 intrinsic String scalar content。

IRIS-V1-COLLECTIONS-C048: Interpolation 从左到右求值每个 `${expr}`。非 String interpolation value 通过 dynamic `to_string() -> String` 转换。Missing lookup 可以使用 `method_missing`。非 String conversion result 引发 `TypeContractError`。任何 failure 都会阻止后续 segment expressions 运行，且不返回 partial String value。

IRIS-V1-COLLECTIONS-C049: Root `Object#to_string() -> String` 初始返回 `<fully.qualified.ClassName>`，使用 nominal Class identity，并省略 memory address、identity hash、runtime ID、revision number、properties 和 ivars。`Object#inspect() -> String` 初始委托给 `to_string`，且不得绕过 ReflectionPolicy。

IRIS-V1-COLLECTIONS-C050: `String#to_string` 返回 receiver。`String#inspect` 返回可重新解析的 double-quoted escaped literal，它重建 scalar-equal String value，且解析时不执行 interpolation。

IRIS-V1-COLLECTIONS-C051: `String#+(other: Object) -> String` 在 `other` 是 String 时直接追加，否则调用 `other.to_string() -> String`。该 Method 是普通且 dynamically replaceable 的。Adjacent String literal segments 按 [02-lexical-grammar.md](02-lexical-grammar.md) 作为一个 expression concatenate。Symbols、Bytes、ByteArray 和任意 expressions 不会隐式 join。

IRIS-V1-COLLECTIONS-C052: MutableString 是 standard mutable text type。每次 `m` literal evaluation 创建 fresh identity-bearing MutableString。Prefix order 和 raw fences 由 [02-lexical-grammar.md](02-lexical-grammar.md) 拥有。Content parsing、interpolation 和 multiline indentation 遵循对应 String literal family，然后才创建 MutableString value。

IRIS-V1-COLLECTIONS-C053: `MutableString#to_string() -> String` 返回当前 content 的 String snapshot。后续 MutableString mutation 不得影响先前 snapshots。Copy-on-write、ropes 和 shared read-only segments 只有在不可观察时才允许。

IRIS-V1-COLLECTIONS-C054: `MutableString#[]` reads 使用 String scalar indexing。`MutableString#[i]=replacement` 要求 one-scalar String，out of range 时引发 `IndexError`，in place mutation，并返回 `nil`。`MutableString#[range]=replacement` 使用 scalar unit-forward slicing，接受任何 String，可以改变长度，in place mutation，并返回 `nil`。

IRIS-V1-COLLECTIONS-C055: MutableString built-in equality 比较当前精确 scalar content，并跨类型等于具有相同 content 的 String。内建 `hash` 引发 `InvalidKeyError`。User-added MutableString hash 必须匹配 String-compatible equality，并承担普通 mutation 和 rehash responsibility。

IRIS-V1-COLLECTIONS-C056: `MutableString#+(other: Object) -> MutableString` 使用 String 或 MutableString snapshot semantics 或 `to_string` 转换 operand，然后返回 fresh MutableString identity。`MutableString#<<(other: Object)` 和 `MutableString#append(other: Object)` 使用相同转换，在完整转换成功后 in place mutate receiver，并返回 receiver。

IRIS-V1-COLLECTIONS-C057: `MutableString += other` 使用 ordinary compound assignment。它读取 current target，调用 `MutableString#+`，并将 fresh MutableString result 写回。指向旧 MutableString 的 existing aliases 不会观察到 append。In-place mutation 需要 `<<` 或 `append`。

IRIS-V1-COLLECTIONS-C058: MutableString `clear() -> MutableString`、`replace(other: Object) -> MutableString`、`<<`、`append`、scalar assignment、range assignment 和 bang transforms 必须 atomically commit。Conversion、allocation、type 或 resource failure 会让 previous content 保持不变并传播 error。Self 或 alias append 会在 mutation 前 snapshot source text。

IRIS-V1-COLLECTIONS-C059: Standard text transforms 对 allocating results 使用 non-bang names，对 MutableString mutation 使用 bang names。String transforms 如 `upcase`、`downcase`、`normalize` 和 `casefold` 返回新 String values。MutableString non-bang counterparts 返回 fresh MutableStrings，而 `upcase!`、`downcase!`、`normalize!` 和 `casefold!` atomically mutate 并返回 receiver。

IRIS-V1-COLLECTIONS-C060: MutableString 在逻辑上不是 thread-safe。未同步的 read/write 或 write/write concurrency 可以引发 `ConcurrentMutationError`，但必须保持 memory-safe，不得暴露 invalid Unicode。正确同步的观察者只能看到变更前的完整内容或变更后的完整内容，不得看到撕裂或部分更新状态。

IRIS-V1-COLLECTIONS-C061: MutableString scalar 和 grapheme iterators 捕获 content version。Append、clear、replace、scalar assignment、range assignment、bang transforms 或任何 content change 都会递增它。Mismatch 时下一次 advance 引发 `ConcurrentModificationError`。Stable traversal 使用显式 `to_string` snapshot。

IRIS-V1-COLLECTIONS-C062: Default String 和 MutableString iteration 按 scalar order yield one-scalar String values。`bytes` 和 `graphemes` 暴露 lazy Iterable views，而不是 eager Arrays。MutableString views 捕获 content version，并在 content mutation 时 fail-fast。

## Symbols

IRIS-V1-COLLECTIONS-C063: Symbol literals 使用 [02-lexical-grammar.md](02-lexical-grammar.md) 定义的 simple 和 quoted forms。来自 identifiers、selector identifiers、operators 和 raw-ivar forms 的 Simple Symbols 使用其对应 canonical source content。Quoted arbitrary-text Symbols 使用 String escape rules，从不 interpolate，并保留精确 post-escape scalar sequence，不做 identifier NFC normalization。

IRIS-V1-COLLECTIONS-C064: Symbols 是 immutable identity-less interned-name values。Symbol equality 是精确 canonical-content equality。对 Symbol 使用 `same?` 引发 `IdentityError`。Interning 是 implementation sharing strategy，不得变成可观察 identity。

IRIS-V1-COLLECTIONS-C065: Ordinary identifiers 是 case-sensitive，并在 lexer 中规范化为 Unicode NFC，然后用于 name、selector 和 Type identity。Simple Symbols 复用该 normalized content。Quoted arbitrary-text Symbols 精确保留 case 和 canonical distinctions。Stable Symbol hashing 使用所得 canonical Symbol content。

IRIS-V1-COLLECTIONS-C066: 接受 names 的 Raw ivar reflection APIs 要求 Symbol 以恰好一个 `@` 开头，后接 valid ordinary identifier，且没有 selector suffix。Strings 不得隐式转换为 raw-ivar Symbols。

## Binary Values 与 Encoding

IRIS-V1-COLLECTIONS-C067: `Bytes` 是 immutable identity-less byte sequence。`ByteArray` 是 identity-bearing mutable byte sequence。两者都包含 `0..255` 中的 byte values。Byte literals 中的 non-ASCII source text 贡献其 UTF-8 bytes，escaped byte literal `\xNN` 注入一个 raw byte。

IRIS-V1-COLLECTIONS-C068: Bytes 和 ByteArray built-in equality 比较精确 current byte sequence，并允许 cross-type equality。Bytes built-in public hash 是 stable。ByteArray built-in `hash` 引发 `InvalidKeyError`。User-added ByteArray hash 必须匹配 Bytes-compatible equality，并承担 mutation 和 rehash responsibility。

IRIS-V1-COLLECTIONS-C069: `Bytes#[]` 和 `ByteArray#[]` reads 使用 byte units，支持 negative-index，返回 `0..255` 中的 `Integer?`，或 out of range 时返回 `nil`。`ByteArray#[i]=value` 要求 `0..255` 中的 Integer byte，对 invalid byte value 引发 `RangeError`，out of range 时引发 `IndexError`，in place mutate，并返回 `nil`。

IRIS-V1-COLLECTIONS-C070: Bytes Range slicing 返回 Bytes。ByteArray Range slicing 返回 independent ByteArray snapshot identity。两者都使用 byte unit-forward Range slicing。`ByteArray#[range]=replacement` 接受 Bytes 或 ByteArray，snapshots aliasing sources，可以改变长度，atomically replaces range，返回 `nil`，且不暴露 partial content。

IRIS-V1-COLLECTIONS-C071: `Bytes + (Bytes|ByteArray) -> Bytes` 返回 immutable Bytes value。`ByteArray + (Bytes|ByteArray) -> ByteArray` 返回 fresh ByteArray identity，不改变 receiver。`ByteArray#<<` 和 `ByteArray#append` 在 snapshotting aliases 后 in place mutate，返回 receiver，并 atomically commit。`ByteArray += other` 使用 `+` 然后 rebinds。

IRIS-V1-COLLECTIONS-C072: Binary operations 不得 encode String、调用 `to_string`，或通过 text conversion 接受 Object。Text 和 binary conversion 只通过显式 APIs 发生。

IRIS-V1-COLLECTIONS-C073: `String#to_bytes` 和 `MutableString#to_bytes` 返回 immutable UTF-8 Bytes snapshots。`Bytes#to_string` 严格解码 UTF-8，并在 invalid sequences 上引发 `EncodingError`。ByteArray snapshots 后按相同方式解码。Lossy replacement 或 ignore behavior 需要显式 decoding options，且绝不是默认行为。

IRIS-V1-COLLECTIONS-C074: Language core 只保证 UTF-8 source text 和 UTF-8 text-to-binary convenience conversion。Standard library Encoding objects 定义额外 encodings，例如 UTF-16LE、UTF-16BE 和 Latin-1，默认 strict errors，并提供显式 replace 或 ignore options。OS locale 和 code page 不得被隐式选择。

IRIS-V1-COLLECTIONS-C075: Default Bytes 和 ByteArray iteration 按 source order yield Integer byte values。ByteArray iterators 捕获 content version，任何 ByteArray content mutation 都会使 next advance 引发 `ConcurrentModificationError`。

## Regex 与 Match

IRIS-V1-COLLECTIONS-C076: Regex literals 使用 `/pattern/flags` 或 raw `r/pattern/flags`，raw fences 按 [02-lexical-grammar.md](02-lexical-grammar.md) 定义。Non-raw Regex literals 允许 `${expr}` interpolation。每个 interpolated value 求值一次，通过 dynamic `to_string() -> String` 转换，然后用 default Regex escaping 转义后插入 pattern。

IRIS-V1-COLLECTIONS-C077: Regex 是 immutable identity-less core value。Regex equality 和 public hash 使用 canonical pattern text 加 canonical flags。对 Regex 使用 `same?` 引发 `IdentityError`。

IRIS-V1-COLLECTIONS-C078: Core Regex 支持固定 Unicode-aware safe subset，匹配取向为 deterministic linear-time。它包括 literal text、concatenation、alternation、grouping、character classes、绑定 Unicode 17.0.0 的 Unicode properties、anchors、word boundaries、greedy 和 lazy bounded or unbounded repetition、named captures、numbered captures，以及 non-capturing groups。

IRIS-V1-COLLECTIONS-C079: Core Regex 不得支持 backreferences、lookbehind、subroutine calls、conditionals、atomic groups、possessive quantifiers、recursion、embedded code、engine callbacks、global match variables，或需要 unbounded catastrophic backtracking 的 constructs。Advanced PCRE-style engines 属于单独 versioned standard packages，并且必须使用 distinct types。

IRIS-V1-COLLECTIONS-C080: Regex flags 精确为 `i`、`m`、`s` 和 `x`。`i` 启用 Unicode case-insensitive matching，`m` 让 line anchors 按行操作，`s` 让 dot 匹配 newline，`x` 按 core Regex lexical rules 忽略 pattern whitespace 和 comments。Unicode mode 始终开启，并由 Iris language major 固定。Duplicate 或 unsupported flags 必须被诊断为 `LEX_BAD_REGEX_FLAGS` 或更严格 Regex diagnostic。

IRIS-V1-COLLECTIONS-C081: Canonical Regex flags 必须以固定顺序 `imsx` 存储，省略 absent flags。Equality 和 hashing 比较 canonical flags，因此如果 `/a/im` 和 `/a/mi` 两种 source forms 都被接受，它们相等。Duplicate flags 在 canonicalization 前无效。

IRIS-V1-COLLECTIONS-C082: `text =~ regex -> Match?` 对 text 和 Regex 各求值一次，要求 text 是 String 或 MutableString snapshot content，并返回 first match 的 immutable Match 或 `nil`。`text !~ regex -> Bool` 精确在 `text =~ regex` 会返回 `nil` 时返回 `true`，但 evaluation 或 Regex errors 会传播。

IRIS-V1-COLLECTIONS-C083: Match values 是 immutable。Match 暴露 full match、scalar ranges、UTF-8 中的 byte ranges、numbered captures、named captures 和使用的 Regex。Capture absence 与 empty capture 不同。Match APIs 不得暴露 global variables 或 mutable engine state。

IRIS-V1-COLLECTIONS-C084: 对 MutableString 的 Regex matching 必须在 matching 开始前捕获 content snapshot 并在其上操作。后续 MutableString mutation 不得改写 existing Match。

## Stable Public Hash Domains 与 Encodings

IRIS-V1-COLLECTIONS-C085: Iris v1 中每个 stable public hash 都必须返回 `0..2^64-1` 中的 nonnegative Integer。除非本章规定其他 failure，stable hashes 使用 BLAKE3 derive-key mode，使用表中的 exact ASCII context、表中的 canonical input bytes、standard 32-byte digest，并将 digest bytes `0..8` 按 unsigned little-endian 解释。Context bytes 不会手动拼接到 input。

IRIS-V1-COLLECTIONS-C086: Canonical `ULEB128(n)` 表示 nonnegative Integer `n` 的最短 unsigned LEB128 encoding。带冗余 continuation groups 的 encodings 是 noncanonical。`u64_le(hash)` 表示 public hash value 的恰好八个 unsigned little-endian bytes。

IRIS-V1-COLLECTIONS-C087: Stable hash context strings 是全局 domain-separated。符合规范的实现 必须使用这些 exact contexts，且不得为另一个 value family 复用某个 context。

| 家族 | 精确 ASCII BLAKE3 derive-key context | 规范输入字节 | 哈希失败规则 |
| --- | --- | --- | --- |
| Numeric | `Iris Language v1 stable numeric hash` | 来自 [03-runtime-object-model.md](03-runtime-object-model.md) 的 Numeric 规范字节 | NaN 引发 `InvalidKeyError` |
| Singleton | `Iris Language v1 stable singleton hash` | 来自 [03-runtime-object-model.md](03-runtime-object-model.md) 的 `nil`、`false` 和 `true` 单字节输入 | 对三个 singleton 无 failure |
| Contract view | `Iris Language v1 contract view hash` | `receiver_public_hash_u64_le || contract_type_hash_u64_le` | Receiver 或 Contract Type 的哈希失败会传播 |
| Symbol | `Iris Language v1 stable symbol hash` | `ULEB128(utf8_byte_length) || canonical_symbol_utf8` | 无 |
| String | `Iris Language v1 stable string hash` | `ULEB128(utf8_byte_length) || exact_scalar_sequence_utf8` | 无 |
| Bytes | `Iris Language v1 stable bytes hash` | `ULEB128(byte_length) || bytes` | 无 |
| Tuple | `Iris Language v1 stable tuple hash` | `ULEB128(element_count) || element_0_public_hash_u64_le || ...` | Element 哈希失败会传播 |
| Range | `Iris Language v1 stable range hash` | `openness_tag || start_public_hash_u64_le || end_public_hash_u64_le || step_public_hash_u64_le` | Component 哈希失败会传播 |
| Iteration | `Iris Language v1 stable iteration hash` | `Iteration.done`: `[0x00]`; `Iteration.yield(payload)`: `[0x01] || payload_public_hash_u64_le` | Payload 哈希失败会传播 |
| Regex | `Iris Language v1 stable regex hash` | `ULEB128(pattern_utf8_length) || pattern_utf8 || ULEB128(canonical_flags_ascii_length) || canonical_flags_ascii` | 无 |

IRIS-V1-COLLECTIONS-C088: Range hash `openness_tag` 对 inclusive-end Ranges 是 `[0x00]`，对 exclusive-end Ranges 是 `[0x01]`。Start、end 和 step 使用其 values 的 current public hashes。Integer Range components 使用 [03-runtime-object-model.md](03-runtime-object-model.md) 中的 numeric public hashes。

IRIS-V1-COLLECTIONS-C089: Tuple、Range、Iteration yield 和 Contract-view stable hashes 是基于 public hash values 的 compositional hashes。它们必须传播来自任何 component 的 `InvalidKeyError` 或其他 hash failure。它们不得对 identity-less wrappers fallback 到 object identity。

IRIS-V1-COLLECTIONS-C090: Stable public hash canonical bytes 是 specification-internal，除非本规范单独暴露 value format。User programs 只观察 public `hash` result。未来 serialization 和 standard-library chapter 拥有 persistent IrisValue format responsibilities。

IRIS-V1-COLLECTIONS-C091: 下列 stable hash vector table 是规范要求。Digest prefixes 和 public hash values 使用 IRIS-V1-COLLECTIONS-C087 命名的 contexts，以 BLAKE3 derive-key mode 计算。

| 向量 ID | 值 | 规范输入十六进制 | Context | Digest bytes`0..8` 十六进制 | 公共哈希 Integer |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-COLLECTIONS-V001` | `:name` | `046e616d65` | `Iris Language v1 stable symbol hash` | `e190f8954b4fa498` | `10999003376002830561` |
| `IRIS-V1-COLLECTIONS-V002` | `:@x` | `024078` | `Iris Language v1 stable symbol hash` | `f498d4107fb3e9de` | `16062566904318171380` |
| `IRIS-V1-COLLECTIONS-V003` | `""` | `00` | `Iris Language v1 stable string hash` | `23886f048e56bf77` | `8628710579024922659` |
| `IRIS-V1-COLLECTIONS-V004` | `"Iris"` | `0449726973` | `Iris Language v1 stable string hash` | `2df84e8634b29e29` | `2999030340536694829` |
| `IRIS-V1-COLLECTIONS-V005` | `b""` | `00` | `Iris Language v1 stable bytes hash` | `475a703c0515b46d` | `7904966358175078983` |
| `IRIS-V1-COLLECTIONS-V006` | Bytes`[0xff, 0x00, 0x01, 0x02]` | `04ff000102` | `Iris Language v1 stable bytes hash` | `9f87b39f88b29df1` | `17410268034348844959` |
| `IRIS-V1-COLLECTIONS-V007` | `()` | `00` | `Iris Language v1 stable tuple hash` | `0cc6fb61f1157f8c` | `10123834613827356172` |
| `IRIS-V1-COLLECTIONS-V008` | `(nil, true)` using runtime hashes | `02d37c681abf4074a440168bee0b35dfc4` | `Iris Language v1 stable tuple hash` | `1697cf0d42d50e9c` | `11245159799266973462` |
| `IRIS-V1-COLLECTIONS-V009` | `1 ..= 3` with step `1` | `00384e0f3cb1fc5bf76bc4f1137cdbfcc8384e0f3cb1fc5bf7` | `Iris Language v1 stable range hash` | `894974389baabd35` | `3872438838252292489` |
| `IRIS-V1-COLLECTIONS-V010` | `/a+/im` canonical flags `im` | `02612b02696d` | `Iris Language v1 stable regex hash` | `4446d3f2271a0b18` | `1732507240534066756` |
| `IRIS-V1-COLLECTIONS-V011` | `Iteration.done` | `00` | `Iris Language v1 stable iteration hash` | `bc123979d794e248` | `5251923768640082620` |
| `IRIS-V1-COLLECTIONS-V012` | `Iteration.yield(nil)` | `01d37c681abf4074a4` | `Iris Language v1 stable iteration hash` | `44848ba05f9798bf` | `13805951094675440708` |

## Examples 与 Conformance Vectors

IRIS-V1-COLLECTIONS-EX001: 信息性示例，Range 词元和索引单位:

```iris
let closed = 1 ..= 3      // yields 1, 2, 3
let half_open = 1 ..< 3   // yields 1, 2
let text = "a\u{1f600}b"
text.length               // 3 scalar values
text[1]                   // one-scalar String containing U+1F600
text.to_bytes().length    // UTF-8 byte count, not scalar count
```

IRIS-V1-COLLECTIONS-EX002: 信息性示例，可变别名和快照:

```iris
let builder = m"ab"
let snapshot = builder.to_string()
builder << "c"
snapshot                  // "ab"
builder.to_string()       // "abc"
```

IRIS-V1-COLLECTIONS-EX003: 信息性示例，Hash 遍历期间的变更:

```iris
let table = %{ :a: 1, :b: 2 }
table.each_with_iterator() { |key: Symbol, value: Integer, iterator: Iterator<Tuple<Symbol,Integer>>| -> Nil
  if key == :a {
    iterator.remove_current()
  }
}
```

IRIS-V1-COLLECTIONS-EX004: 信息性示例，Regex 快照和安全子集:

```iris
let name = "item"
let regex = /^${name}[0-9]+$/i
let result = "Item42" =~ regex
```

IRIS-V1-COLLECTIONS-C092: 下列 vector table 是规范要求。Conformance chapter 必须保留这些 vector IDs，或将其映射到具有相同 observable outcomes 的 machine-readable records:

| 向量 ID | 种类 | 场景 | 期望结果 |
| --- | --- | --- | --- |
| `IRIS-V1-COLLECTIONS-V013` | 正向 | `String#[]` over `"a\u{1f600}b"` at index `1` | One-scalar String U+1F600 |
| `IRIS-V1-COLLECTIONS-V014` | 失败 | String 索引赋值 | 缺少标准 `[]=` 或拒绝变更 |
| `IRIS-V1-COLLECTIONS-V015` | 正向 | MutableString `to_string` snapshot before append | 后续变更后快照不变 |
| `IRIS-V1-COLLECTIONS-V016` | 失败 | MutableString as Hash key with built-in hash | `InvalidKeyError` |
| `IRIS-V1-COLLECTIONS-V017` | 正向 | 对有效 UTF-8 调用 `Bytes#to_string` | 精确 String 结果 |
| `IRIS-V1-COLLECTIONS-V018` | 失败 | 对无效 UTF-8 调用 `Bytes#to_string` | `EncodingError` |
| `IRIS-V1-COLLECTIONS-V019` | 失败 | `ByteArray#[i]=256` | `RangeError` |
| `IRIS-V1-COLLECTIONS-V020` | 正向 | 只含可哈希元素的 Tuple 用作 Hash key | Tuple hash 成功 |
| `IRIS-V1-COLLECTIONS-V021` | 失败 | Tuple containing MutableString used as Hash key | `InvalidKeyError` |
| `IRIS-V1-COLLECTIONS-V022` | 正向 | Array slice 后改变原 Array | slice 保持为独立快照 |
| `IRIS-V1-COLLECTIONS-V023` | 失败 | element replacement 后的 active Array iterator | `ConcurrentModificationError` on next advance |
| `IRIS-V1-COLLECTIONS-V024` | 正向 | existing value update 后的 active Hash iterator | 尚未 yield 的 entry 观察到最新 value |
| `IRIS-V1-COLLECTIONS-V025` | 失败 | key insertion 后的 active Hash iterator | `ConcurrentModificationError` on next advance |
| `IRIS-V1-COLLECTIONS-V026` | 失败 | 两个 key 现在比较相等时调用 Hash `rehash()` | `KeyConflictError`，原 Hash 不变 |
| `IRIS-V1-COLLECTIONS-V027` | 正向 | `Hash#each_with_iterator` 中有效的 `remove_current` | current entry 被移除，owning iterator 继续 |
| `IRIS-V1-COLLECTIONS-V028` | 失败 | 对同一个 yielded entry 调用两次 `remove_current` | `IteratorStateError` |
| `IRIS-V1-COLLECTIONS-V029` | 正向 | `for` traversal 中的 `Iteration.yield(nil)` | body 接收合法 `nil`，done 之后终止 |
| `IRIS-V1-COLLECTIONS-V030` | 失败 | 读取 `Iteration.done.value` | `IteratorStateError` |
| `IRIS-V1-COLLECTIONS-V031` | 正向 | `1 ..= 3` and `1 ..< 3` | closed yield 三个值，half-open yield 两个值 |
| `IRIS-V1-COLLECTIONS-V032` | 失败 | `range.by(step: 0)` | `RangeError` |
| `IRIS-V1-COLLECTIONS-V033` | 正向 | 对 `"a+b"` 进行 Regex interpolation | 作为 escaped literal text 插入 |
| `IRIS-V1-COLLECTIONS-V034` | 失败 | core Regex 中的 Regex backreference | 对 unsupported construct 给出 Regex diagnostic |
| `IRIS-V1-COLLECTIONS-V035` | 失败 | 重复 Regex flag `/a/ii` | `LEX_BAD_REGEX_FLAGS` or stricter Regex diagnostic |
| `IRIS-V1-COLLECTIONS-V036` | 正向 | stable hash vectors V001 到 V012 | IRIS-V1-COLLECTIONS-C091 中的精确 Integer 输出 |
| `IRIS-V1-COLLECTIONS-V037` | 正向 | 以不同 insertion histories 创建的两个含相同 entries 的 Hash 被迭代 | 每个 entry 都恰好 yield 一次；不主张 insertion、stable-hash、sorted 或 cross-run order |
| `IRIS-V1-COLLECTIONS-V038` | 正向 | identifiers、graphemes、normalization、casefold 和 Regex properties 使用默认 Unicode data version | 所有观察都使用 Unicode 17.0.0 tables，独立于 OS locale 或 host libraries |
| `IRIS-V1-COLLECTIONS-V039` | 失败 | Core Regex 使用 host Unicode version 的 Unicode property 结果，而不是 Unicode 17.0.0 | 除非匹配 Unicode 17.0.0，否则 conformance 拒绝 host-version behavior |

## 直接跨决策 Regex 向量

IRIS-V1-COLLECTIONS-C097: 下列向量是规范要求，因为其具体源代码直接同时观察 Unicode 版本、core Regex 表面以及安全子集/包拆分决策。

| 向量 ID | 类别 | 适用性 | Source/Input | 期望可观察结果 | 决策 |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-COLLECTIONS-V042` | diagnostic | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | 在早于 Unicode 17.0.0 的 host Unicode implementation 下运行四个隔离 source: `(1)` `[Unicode.version(), "A" =~ /\p{Lu}/, "a\nB" =~ /^B/m]`; `(2)` `AdvancedRegex.compile("(?<=a)b").match("ab")`; `(3)` `/(?<=a)b/`; `(4)` `/a/ii`. | Source 1 返回 `["17.0.0", Match, Match]`; source 2 通过 distinct `AdvancedRegex` Type 返回其 package-defined Match；source 3 报告 `REGEX_UNSUPPORTED_LOOKBEHIND`; source 4 报告 `LEX_BAD_REGEX_FLAGS`. Core 结果保持 Unicode 17.0.0，advanced package 既不替换 core Regex，也不改变 literal semantics。 | `D-407`, `D-505`, `D-506` |

## 可追溯性说明

IRIS-V1-COLLECTIONS-C093: 本章拥有或细化 D-116 到 D-139、D-337 到 D-341、D-368 到 D-414、D-459 到 D-467、D-505 和 D-506 的 集合、文本、二进制、Regex、迭代和稳定哈希部分。D-336 是 跨章节 raw-ivar Symbol 验证依赖，在 IRIS-V1-COLLECTIONS-C066 中有本地 collections 锚点。本章不拥有 raw-ivar reflection 操作。它复用 D-071 through D-089 及 [03-runtime-object-model.md](03-runtime-object-model.md) 中的 numeric、singleton 和 Contract-view hash 锚点，而不重新定义它们。

IRIS-V1-COLLECTIONS-C094: D-319 到 D-336 被读作 runtime、Module、ReflectionPolicy 和 raw-ivar Symbol-name 依赖。它们的 规范性归属 仍在 [03-runtime-object-model.md](03-runtime-object-model.md)、[02-lexical-grammar.md](02-lexical-grammar.md) 和后续 metaprogramming 章节。本章只使用其 Symbol、raw-ivar name 和 receiver-state 后果，当 collection 与 text APIs 引用它们时。

IRIS-V1-COLLECTIONS-C095: D-365 到 D-367 是 source-text 和 comment 依赖，由 [02-lexical-grammar.md](02-lexical-grammar.md) 拥有。本章拥有 runtime String、MutableString、Bytes、ByteArray、Unicode、Encoding 和 Regex 值行为，从 grammar chapter 定义的 literal 结果 开始。修订后的 D-407 已确认 Unicode 17.0.0 是 IRIS-V1-COLLECTIONS-C042 和 Regex Unicode properties 使用的精确 Iris v1 Unicode data version。

IRIS-V1-COLLECTIONS-C096: 覆盖和依赖的决策 ID 为 `D-071`, `D-072`, `D-073`, `D-074`, `D-075`, `D-076`, `D-077`, `D-078`, `D-079`, `D-080`, `D-081`, `D-082`, `D-083`, `D-084`, `D-085`, `D-086`, `D-087`, `D-088`, `D-089`, `D-116`, `D-117`, `D-118`, `D-119`, `D-120`, `D-121`, `D-122`, `D-123`, `D-124`, `D-125`, `D-126`, `D-127`, `D-128`, `D-129`, `D-130`, `D-131`, `D-132`, `D-133`, `D-134`, `D-135`, `D-136`, `D-137`, `D-138`, `D-139`, `D-319`, `D-320`, `D-321`, `D-322`, `D-323`, `D-324`, `D-325`, `D-326`, `D-327`, `D-328`, `D-329`, `D-330`, `D-331`, `D-332`, `D-333`, `D-334`, `D-335`, `D-336`, `D-337`, `D-338`, `D-339`, `D-340`, `D-341`, `D-365`, `D-366`, `D-367`, `D-368`, `D-369`, `D-370`, `D-371`, `D-372`, `D-373`, `D-374`, `D-375`, `D-376`, `D-377`, `D-378`, `D-379`, `D-380`, `D-381`, `D-382`, `D-383`, `D-384`, `D-385`, `D-386`, `D-387`, `D-388`, `D-389`, `D-390`, `D-391`, `D-392`, `D-393`, `D-394`, `D-395`, `D-396`, `D-397`, `D-398`, `D-399`, `D-400`, `D-401`, `D-402`, `D-403`, `D-404`, `D-405`, `D-406`, `D-407`, `D-408`, `D-409`, `D-410`, `D-411`, `D-412`, `D-413`, `D-414`, `D-459`, `D-460`, `D-461`, `D-462`, `D-463`, `D-464`, `D-465`, `D-466`, `D-467`, `D-505`, and `D-506`.

## 审计精确一致性向量

这些行是规范性的审计精确向量。除非 `Source/Input` 明确命名原始字节或 Host fixture，否则它就是可执行 Iris。

| 向量 ID | 类别 | 适用性 | Source/Input | 期望可观察结果 | 决策 |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-COLLECTIONS-V275` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `a = Key.new(7); b = Key.new(7); h = %{ a: 1 }; h[b] = 2; [h.fetch(a), h.delete(b)]`, 其中 `Key#==` 和 `Key#hash` 使用 `id`. | 结果 `[2, 2]`; Hash 使用 dynamic `==`/`hash`，而不是 identity. | `D-116` |
| `IRIS-V1-COLLECTIONS-V276` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `k = Key.new(1); h = %{ k: :ok }; k.hash_code = 2; before = h[k]; h.rehash(); [before, h.fetch(k)]`. | 结果 `[nil, :ok]`; 只有 `rehash()` 会使新的 hash protocol 生效. | `D-117` |
| `IRIS-V1-COLLECTIONS-V277` | negative | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `a = Key.new(1, 11); b = Key.new(2, 22); h = %{ a: :a, b: :b }; a.equal_id = 0; b.equal_id = 0; h.rehash()`, 其中 `Key#==` 比较 `equal_id`，`Key#hash` 对其进行哈希. | 引发 `KeyConflictError`; 没有副作用: `h.size == 2`, `h.fetch(a) == :a`, and `h.fetch(b) == :b`. | `D-118` |
| `IRIS-V1-COLLECTIONS-V278` | negative | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `a = Key.new(0); b = Key.new(1); h = %{ a: 2, b: 3 }; b.equal_id = 0; h.rehash() { |kept, left, incoming, right| (kept, left + right) }; before = h.to_array(); h.rehash() { |kept, left, incoming, right| :bad }`. | 第一次调用返回 `nil` 并留下一个值为 Integer 的 entry `5`; 第二次调用引发 `TypeContractError` 并使 `h.to_array()` 作为 unordered entry set 等于 `before`. | `D-119` |
| `IRIS-V1-COLLECTIONS-V279` | positive | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `a = %{ :a: 1, :b: 2 }; b: Hash<Symbol,Integer> = %{}; b[:b] = 2; b[:a] = 1; [a.to_array(), b.to_array()]`. | 两个 Array 只解释为 unordered entry multisets，等于 `{(:a, 1), (:b, 2)}` 且每个 entry 恰好一次. 不期望 relative、insertion、sorted、stable-hash、rehash 或 cross-run order. | `D-120` |
| `IRIS-V1-COLLECTIONS-V280` | positive | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `keys = [Key.new(1, 10), Key.new(2, 10), Key.new(3, 20), Key.new(4, 20)]; h = %{ keys[0]: 1, keys[1]: 2, keys[2]: 4, keys[3]: 8 }; pairs = []; h.rehash() { |kept, left, incoming, right| pairs << Set.of(kept.id, incoming.id); (kept, left + right) }`. | `pairs` 是 unordered set `{Set.of(1,2), Set.of(3,4)}` 且最终 values 是 unordered set `{3,12}`; 不期望 callback order，也不期望保留哪个 equal key. | `D-121` |
| `IRIS-V1-COLLECTIONS-V281` | diagnostic | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `h = %{ :a: 1 }; it = h.iterator(); h[:b] = 2; it.next()`. | 引发 `ConcurrentModificationError`. | `D-122` |
| `IRIS-V1-COLLECTIONS-V282` | positive | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `h = %{ :a: 1, :b: 2 }; it = h.iterator(); first = it.next().value; h[first[0]] = 9; remaining_key = first[0] == :a ? :b : :a; h[remaining_key] = 7; second = it.next().value; [first, second]`. | `first` 保持为替换前捕获的 Tuple `(first[0], 1 or 2)`；`second == (remaining_key, 7)`。两个值的 Type 都是 `Tuple<Symbol,Integer>`。 | `D-123` |
| `IRIS-V1-COLLECTIONS-V283` | positive | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `h = %{ :a: 1, :b: 2 }; other = h.iterator(); removed = nil; h.each_with_iterator() { |key, value, it| removed = (key, value, it.remove_current()); break }; owner_next = h.iterator().next(); other.next()`. | `removed[2] == nil`，`h.size == 1`，且 owning traversal 仍有效；最后的 `other.next()` 引发 `ConcurrentModificationError`。 | `D-124` |
| `IRIS-V1-COLLECTIONS-V284` | positive | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `h = %{ :a: 1 }; each_args = []; exposed = []; nested = []; h.each() { |key, value| each_args << (key, value) }; h.each_with_iterator() { |key, value, it| exposed << it; h.each_with_iterator() { |k2, v2, it2| nested << it2 } }`. | `each_args == [(:a,1)]`；各 block 分别精确收到两个和三个参数；`exposed[0].same?(nested[0]) == false`；two-argument `each` block 中没有可用 iterator。 | `D-125` |
| `IRIS-V1-COLLECTIONS-V285` | negative | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | 三个隔离程序：`(1)` `it = %{ :a: 1 }.iterator(); it.remove_current()`；`(2)` 将 one-entry iterator advance 到 `Iteration.done` 后执行 `it.remove_current()`；`(3)` `it = %{ :a: 1 }.iterator(); it.close(); it.remove_current()`。 | 每个程序都在 `remove_current()` 处引发 `IteratorStateError`，且没有 Hash mutation。 | `D-126` |
| `IRIS-V1-COLLECTIONS-V286` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `for value in YieldNil.new() { seen << value }`，其中 `next()` yield `Iteration.yield(nil)` 后 done。 | `seen == [nil]`；completion 不是 exception。 | `D-127` |
| `IRIS-V1-COLLECTIONS-V287` | negative | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `a = Iteration.yield("x"); b = Iteration.yield("x"); [a == b, a.same?(b)]`. | Equality 为 `true`；`same?` 引发 `IdentityError`；`Iteration.done.same?(Iteration.done)` 为 `true`。 | `D-128` |
| `IRIS-V1-COLLECTIONS-V288` | negative | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `y = Iteration.yield(4); d = Iteration.done; [y.yield?, y.done?, y.value, d.yield?, d.done?]; d.value`. | 第一个结果为 `[true, false, 4, false, true]`; 最后访问引发 `IteratorStateError`. | `D-129` |
| `IRIS-V1-COLLECTIONS-V289` | negative | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `%{ Iteration.done: :d, Iteration.yield(1): :y }; %{ Iteration.yield(m"x"): :bad }`. | 第一个 Hash 成功；第二次 insertion 引发 `InvalidKeyError`. | `D-130` |
| `IRIS-V1-COLLECTIONS-V290` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `Iteration.done.hash; Iteration.yield(nil).hash`. | 精确 Integer `5251923768640082620`, `13805951094675440708`; canonical inputs `00`, `01d37c681abf4074a4`. | `D-131` |
| `IRIS-V1-COLLECTIONS-V291` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `[Iteration.done <=> Iteration.done, Iteration.done <=> Iteration.yield(1), Iteration.yield(1) <=> Iteration.yield(2)]`. | 结果 `[0, nil, -1]`. | `D-132` |
| `IRIS-V1-COLLECTIONS-V292` | negative | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `Iteration.yield(BadCompare.new()) <=> Iteration.yield(BadCompare.new())`, where `<=>` returns `2`. | 引发 `ComparisonContractError`. | `D-133` |
| `IRIS-V1-COLLECTIONS-V293` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `a = [1].iterator(); b = [1].iterator(); a.next(); [a == b, a.same?(b)]`. | 结果 `[false, false]`. | `D-134` |
| `IRIS-V1-COLLECTIONS-V294` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `it = [1].iterator(); h = %{ it: :cursor }; before = it.hash; it.next(); [h.fetch(it), before == it.hash]`. | 结果 `[:cursor, true]`. | `D-135` |
| `IRIS-V1-COLLECTIONS-V295` | positive | 需要 interpreter; JIT 可选; 需要 native | Host fixture `WeakProbe.array([1])`；创建 iterator，丢弃 source，强制 collection，并检查 `probe.finalized?`。 | `false` before exhaustion or `close()`. | `D-136` |
| `IRIS-V1-COLLECTIONS-V296` | positive | 需要 interpreter; JIT 可选; 需要 native | 耗尽 V295 iterator，强制 collection，然后调用 `next()` 两次。 | Probe 可被收集；两个结果都是同一个 `Iteration.done` singleton。 | `D-137` |
| `IRIS-V1-COLLECTIONS-V297` | positive | 需要 interpreter; JIT 可选; 需要 native | 保存 `it.hash`；调用 `it.close()` 两次；强制 collection；调用 `it.next()`。 | 两次 close 都返回 `nil`；probe 可被收集；next 为 done；hash 不变。 | `D-138` |
| `IRIS-V1-COLLECTIONS-V298` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `[:name, :"a b", :"\\n"]`. | 内容为 `name`、`a b` 和一个 newline scalar。 | `D-337` |
| `IRIS-V1-COLLECTIONS-V299` | negative | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `a = :name; b = :name; [a == b, a.same?(b)]`. | Equality 为 `true`; `same?` 引发 `IdentityError`. | `D-338` |
| `IRIS-V1-COLLECTIONS-V300` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `:name.hash`. | `10999003376002830561`, using context `Iris Language v1 stable symbol hash` and bytes `046e616d65`. | `D-339` |
| `IRIS-V1-COLLECTIONS-V301` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | Identifier `e\u{301}`、simple Symbol `:é` 和 quoted Symbol `:"e\u{301}"`。 | 前两者为 NFC `é`；quoted form 保持为 `e` 加 U+0301，且不相等。 | `D-340` |
| `IRIS-V1-COLLECTIONS-V302` | diagnostic | 需要 parser; interpreter 不适用; JIT 不适用; native 不适用 | `let \u{03b1} = 1`; `let \u{200b}x = 1`. | Alpha 按 Unicode 17.0.0 XID 被接受；U+200B 报告 `LEX_INVALID_IDENTIFIER`。 | `D-341` |
| `IRIS-V1-COLLECTIONS-V303` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `"\xFF".to_bytes()`. | 十六进制 bytes 为 `c3bf`, 证明是 U+00FF 而不是 byte `ff`. | `D-370` |
| `IRIS-V1-COLLECTIONS-V304` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `s = "a\u{1f600}b"; [s.length, s[1], s.byte_length]`. | `[3, "😀", 6]`; index Type 为 `String`. | `D-371` |
| `IRIS-V1-COLLECTIONS-V305` | negative | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | 两个隔离 source: `a = "x"; b = "x"; a.same?(b)` and `a = "x"; a[0] = "y"`. | 第一个引发 `IdentityError`；第二个在 static validation 期间报告缺少 standard selector `[]=`；二者都不改变 String。 | `D-372` |
| `IRIS-V1-COLLECTIONS-V306` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `["é" == "e\u{301}", "A" == "a"]`. | `[false, false]`. | `D-373` |
| `IRIS-V1-COLLECTIONS-V307` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `"Iris".hash`. | `2999030340536694829`, canonical bytes `0449726973`, context `Iris Language v1 stable string hash`. | `D-374` |
| `IRIS-V1-COLLECTIONS-V308` | negative | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `"${BadText.new()}"`, 其中 `to_string()` 返回 `1`. | 一次 `to_string()` 调用后引发 `TypeContractError`。 | `D-375` |
| `IRIS-V1-COLLECTIONS-V309` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `Widget.new().to_string()`. | 精确结果 `"<app::Widget>"`; 没有 address、revision、ivar 或 property text. | `D-376` |
| `IRIS-V1-COLLECTIONS-V310` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `"${1}".inspect()`. | 可重新解析 literal `"\\\"1\\\""`; 解析它得到 scalar-equal `"1"` 且不发生 interpolation. | `D-377` |
| `IRIS-V1-COLLECTIONS-V311` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `["x".to_string(), "x".inspect()]`. | `[`"x"`, "\"x\""]`; display 与 inspect 不同. | `D-378` |
| `IRIS-V1-COLLECTIONS-V312` | diagnostic | 需要 parser; interpreter 不适用; JIT 不适用; native 不适用 | `"${1 + 2}"; "${1:hex}"`. | 第一个 String 是 `"3"`; 第二个 source 报告 `PARSE_BAD_INTERPOLATION`. | `D-383` |
| `IRIS-V1-COLLECTIONS-V313` | negative | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `"${log << 1}${Raise.new()}${log << 3}"`. | `log == [1]`; 引发 sentinel exception；不发布 String result. | `D-384` |
| `IRIS-V1-COLLECTIONS-V314` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `"a" + Textable.new(); "a" "b"`. | 结果为 `"ax"` 和 `"ab"`; `Textable#to_string()` 运行一次. | `D-385` |
| `IRIS-V1-COLLECTIONS-V315` | positive | 需要 parser; interpreter 不适用; JIT 不适用; native 不适用 | Source `"a"\n"b"` 和 `("a"\n"b")`。 | 第一个是两个 statements；grouped form 是一个 String `"ab"`. | `D-387` |
| `IRIS-V1-COLLECTIONS-V316` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `s = "abc"; [s[-1], s[3]]`. | `[`"c"`, `nil`]`. | `D-389` |
| `IRIS-V1-COLLECTIONS-V317` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `s = "abc"; [s[-9 ..< 2], s[2 ..< 1]]`. | `[`"ab"`, ""]`; 两个值的 Type 都是 `String`. | `D-390` |
| `IRIS-V1-COLLECTIONS-V318` | negative | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `"abc"[(1 ..= 3).by(step: 2)]`. | 引发 `ArgumentError`. | `D-391` |
| `IRIS-V1-COLLECTIONS-V319` | diagnostic | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `"abc"[0] = "x"`. | 缺少标准 `[]=` 诊断. | `D-392` |
| `IRIS-V1-COLLECTIONS-V320` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `m = m"ab"; snap = m.to_string(); m << "c"; [snap, m.to_string()]`. | `[`"ab"`, "abc"]`. | `D-393` |
| `IRIS-V1-COLLECTIONS-V321` | negative | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `m = m"ab"; m[1] = "x"; m[9] = "z"`. | 第一次变更得到 `"ax"`; 第二次引发 `IndexError`. | `D-394` |
| `IRIS-V1-COLLECTIONS-V322` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `a = m"x"; b = m"x"; a.same?(b)`. | `false`; 二者 initial content 相等. | `D-395` |
| `IRIS-V1-COLLECTIONS-V323` | negative | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `m = m"x"; [m == "x", m.same?(m), m.hash]`. | 前两个值为 `true`; hash 引发 `InvalidKeyError`. | `D-396` |
| `IRIS-V1-COLLECTIONS-V324` | negative | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `m = m"x"; [m == "x", m.hash]`. | Equality 为 `true`; hash 引发 `InvalidKeyError`. | `D-397` |
| `IRIS-V1-COLLECTIONS-V325` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `a = m"a"; b = a + "b"; a << "c"; [a.to_string(), b.to_string()]`. | `[`"ac"`, "ab"]`. | `D-398` |
| `IRIS-V1-COLLECTIONS-V326` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `a = m"a"; alias = a; a += "b"; [alias.to_string(), a.to_string()]`. | `[`"a"`, "ab"]`. | `D-399` |
| `IRIS-V1-COLLECTIONS-V327` | negative | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `m = m"a"; m.append(BadText.new())`, 其中 conversion 引发异常. | 传播 error，且 `m.to_string() == "a"`. | `D-400` |
| `IRIS-V1-COLLECTIONS-V328` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `m = m"ab"; [m.clear().same?(m), m.replace("x").to_string()]`. | `[true, "x"]`. | `D-401` |
| `IRIS-V1-COLLECTIONS-V329` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `m = m"a"; copy = m.upcase(); bang = m.upcase!(); [copy, bang.same?(m)]`. | `copy` 是 fresh `m"A"`; 第二个结果为 `true`. | `D-402` |
| `IRIS-V1-COLLECTIONS-V330` | differential | 需要 interpreter; 需要 JIT; native 不适用 | `m = m"x"; gate = Barrier.new(3); results = Concurrent.collect([ { gate.wait(); m.append("é") }, { gate.wait(); m.append("😀") } ]); gate.wait(); Concurrent.join_all(); snapshot = m.to_string(); [results, snapshot, snapshot.to_bytes()]`. | 每个后端观察到一个 `ConcurrentMutationError` 且 snapshot 精确为 `"xé"` or `"x😀"`, or two successes 且 snapshot 精确为 `"xé😀"` or `"x😀é"`; Bytes 分别为 `78c3a9`, `78f09f9880`, `78c3a9f09f9880`, or `78f09f9880c3a9`, 绝不是 invalid 或 partial UTF-8. | `D-403` |
| `IRIS-V1-COLLECTIONS-V331` | negative | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `m = m"ab"; it = m.iterator(); m.append("c"); it.next()`. | 引发 `ConcurrentModificationError`. | `D-404` |
| `IRIS-V1-COLLECTIONS-V332` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `"a\u{1f600}".to_array()`. | `["a", "😀"]`, 带有 immutable one-scalar Strings. | `D-405` |
| `IRIS-V1-COLLECTIONS-V333` | negative | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `m = m"é"; view = m.bytes(); m.append("x"); view.iterator().next()`. | `view` is Iterable, not Array; advance 引发 `ConcurrentModificationError`. | `D-406` |
| `IRIS-V1-COLLECTIONS-V334` | differential | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `let xid_17 = \u{1C89}; [Unicode.version(), "\u{00DF}".casefold(), "A" =~ /\p{Lu}/, "a\u{0308}".graphemes().to_array(), xid_17]`, 在 host locales 下运行 `C`, `tr-TR`, 以及一个早于 Unicode 17.0.0 的 host Unicode implementation 下运行. | 每次运行都返回 String `"17.0.0"`, String `"ss"`, 一个 non-`nil` Match, Array `['a\u{0308}']`, 并在 Unicode 17.0.0 XID tables 下接受 U+1C89 作为 identifier; locale 和 host Unicode data 不改变任何结果. | `D-407` |
| `IRIS-V1-COLLECTIONS-V335` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `a = b"x"; b = mb"x"; [a.same?(a), b.same?(b)]`. | Bytes identity access 引发 `IdentityError`; ByteArray identity access 为 `true`. | `D-408` |
| `IRIS-V1-COLLECTIONS-V336` | negative | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `a = b"\x00"; b = mb"\x00"; [a == b, b.hash]`. | Equality 为 `true`; ByteArray hash 引发 `InvalidKeyError`. | `D-409` |
| `IRIS-V1-COLLECTIONS-V337` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `Bytes[0xff, 0x00, 0x01, 0x02].hash`. | `17410268034348844959`, canonical bytes `04ff000102`, context `Iris Language v1 stable bytes hash`. | `D-410` |
| `IRIS-V1-COLLECTIONS-V338` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `a = mb"abc"; b = a[0 ..< 2]; a[0 ..< 2] = b"ZZ"; [b.to_bytes(), a.to_bytes()]`. | 结果为 `b"ab"` 和 `b"ZZc"`；`b` 是独立 ByteArray snapshot。 | `D-411` |
| `IRIS-V1-COLLECTIONS-V339` | negative | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `a = b"a"; b = mb"b"; [a + b, b + a]; a + "x"`. | 结果的 Type 为 Bytes 和 ByteArray；最后的 operation 引发 `TypeContractError`。 | `D-412` |
| `IRIS-V1-COLLECTIONS-V340` | negative | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `["é".to_bytes(), Bytes[0xc3, 0x28].to_string()]`. | 第一个值是 hex `c3a9`；第二个 operation 引发 `EncodingError`。 | `D-413` |
| `IRIS-V1-COLLECTIONS-V341` | negative | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `Encoding::UTF_16LE.decode(Bytes[0x41, 0x00]); Encoding.default()`. | Decode 为 `"A"`; implicit default selection 引发 `EncodingSelectionError`. | `D-414` |
| `IRIS-V1-COLLECTIONS-V342` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `for value in ProbeIterable.new() { seen << value }`. | Trace 为 `[:iterator, :next, :next_done]`; `seen` 包含每个 yielded value. | `D-439` |
| `IRIS-V1-COLLECTIONS-V343` | negative | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | 对以下内容的三个隔离观察 `let a: Array<Integer> = [1]`: `a[1]`, `a.hash`, and `a[1] = 2`. | Read 为 `nil`; hash 引发 `InvalidKeyError`; write 引发 `IndexError`; Array 保持为 `[1]`. | `D-459` |
| `IRIS-V1-COLLECTIONS-V344` | negative | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `a = [1, 2]; b = a[0 ..< 1]; b[0] = 9; [a, b]; a.hash`. | 值为 `[[1,2],[9]]`；最后的 hash 引发 `InvalidKeyError`。 | `D-460` |
| `IRIS-V1-COLLECTIONS-V345` | diagnostic | 需要 parser; interpreter 不适用; JIT 不适用; native 不适用 | `%{ 1 + 2: :v }; { :k => :v }; %{} `. | 第一个 key 是 Integer `3`; legacy form 得到 migration diagnostic；untyped empty Hash 被拒绝. | `D-461` |
| `IRIS-V1-COLLECTIONS-V346` | negative | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `h = %{ :a: 1 }; [h[:x], h.delete(:x)]; h.fetch(:x)`. | 第一对 `[nil, nil]`; fetch 引发 `KeyError`; Hash 保持为 `%{:a:1}`. | `D-462` |
| `IRIS-V1-COLLECTIONS-V347` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `[(1 ..= 3).to_array(), (1 ..< 3).to_array()]`. | `[[1, 2, 3], [1, 2]]`. | `D-463` |
| `IRIS-V1-COLLECTIONS-V348` | negative | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `[(3 ..= 1).to_array(), (1 ..= 5).by(step: 2).to_array(), (1 ..= 3).by(step: 0)]`. | 前两个值为 `[[3, 2, 1], [1, 3, 5]]`; zero step 引发 `RangeError`. | `D-464` |
| `IRIS-V1-COLLECTIONS-V349` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `t = (1, "x"); [t[0], t[1], t.same?(t)]`. | `[1, "x"]`; `same?` 引发 `IdentityError`. | `D-465` |
| `IRIS-V1-COLLECTIONS-V350` | positive | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | Source 声明 `contract Numbers for Iterable<Integer> { fun iterator() -> Iterator<Integer> }`; reflection 求值 `[Numbers.requirement(:iterator).return_type, Iterator<Integer>.requirement(:next).return_type, Iterator<Integer>.requirement(:close).return_type]`. | Compilation 成功，精确 normalized Types 为 `[Iterator<Integer>, Iteration<Integer>, Nil]`. | `D-466` |
| `IRIS-V1-COLLECTIONS-V351` | positive | 需要 compiler; 需要 interpreter; JIT 可选; native 不适用 | `[ (1, 2).to_array(), [1, 2].to_array(), b"\x01\x02".to_array() ]`. | Element shapes 为 `[1, 2]`, `[1, 2]`, 和 `[1, 2]` 其中 byte values 的 Type 为 `Integer`. | `D-467` |

## 直接 Regex 一致性向量

IRIS-V1-COLLECTIONS-C098: 下列向量是对 core Regex 构造、匹配、Match 观察、诊断、快照和稳定哈希的规范性直接覆盖。

IRIS-V1-COLLECTIONS-C099: 直接 Regex 表中的多决策映射只在同一具体场景包含每个具名观察时才允许。V042 观察 Unicode 17.0.0 匹配、core Regex 字面量和匹配运算符行为，以及它与 advanced Regex package 的分离。V352 观察 core 字面量/运算符表面和安全子集支持的构造。V353 观察 core 对构造和安全子集排除 flag 的诊断。V354 观察 Unicode 17.0.0 属性、core 匹配和受支持安全子集的 Match 快照。V355 观察 core Regex 规范相等性，以及安全子集的规范 flag 和哈希规则。

| 向量 ID | 类别 | 适用性 | Source/Input | 期望可观察结果 | 决策 |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-COLLECTIONS-V352` | positive | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `term = "a+b"; r = /^${term}(?<digits>[0-9]+)$/mi; m = "A+B42" =~ r; [r, m.full, m.scalar_range, m.byte_range, m[1], m[:digits], "none" !~ r]`. | `r` Type 为 `Regex`, canonical pattern `^a\+b(?<digits>[0-9]+)$`, flags 为 `im`; `m` 是 immutable `Match`; 结果为 full String `"A+B42"`, scalar Range `0 ..< 5`, byte Range `0 ..< 5`, capture String `"42"`, named capture String `"42"`, 和 Bool `true`. | `D-505`, `D-506` |
| `IRIS-V1-COLLECTIONS-V353` | negative | 需要 compiler; interpreter 不适用; JIT 不适用; native 不适用 | 三个隔离 Regex source: `/(a)\1/`, `/(?<=a)b/`, 和 `/a/ii`. | 第一个报告 `REGEX_UNSUPPORTED_BACKREFERENCE`, 第二个报告 `REGEX_UNSUPPORTED_LOOKBEHIND`, 第三个报告 `LEX_BAD_REGEX_FLAGS`; 每个都有 severity `error`, phase `regex-compile` 或 duplicate flags 的更早 lexical phase, 且不发布 Regex value. | `D-505`, `D-506` |
| `IRIS-V1-COLLECTIONS-V354` | positive | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `text = m"ab12"; match = text =~ /(?<letters>\p{L}+)(?<digits>\p{Nd}+)/; text.replace("zz"); [match.full, match[:letters], match[:digits], match.scalar_range, match.byte_range]`. | 精确结果 `["ab12", "ab", "12", 0 ..< 4, 0 ..< 4]`; 后续 MutableString mutation 不会改变 Match snapshot。 | `D-407`, `D-505`, `D-506` |
| `IRIS-V1-COLLECTIONS-V355` | positive | 需要 compiler; 需要 interpreter; 需要 JIT; native 不适用 | `[/a+/im == /a+/mi, /a+/im.hash, /a+/mi.hash]`. | 精确结果 `[true, 1732507240534066756, 1732507240534066756]`; 每个 hash 使用 canonical input hex `02612b02696d`, context `Iris Language v1 stable regex hash`, 以及 digest bytes `4446d3f2271a0b18`. | `D-505`, `D-506` |

## 直接文本续行向量

IRIS-V1-COLLECTIONS-C100: 下列向量是 trace audit 分配给本章的文本续行决策的规范性直接覆盖。

| 向量 ID | 类别 | 适用性 | Source/Input | 期望可观察结果 | 决策 |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-COLLECTIONS-V356` | diagnostic | 需要 compiler; 需要 interpreter; JIT 不适用; native 不适用 | 三个隔离 UTF-8 source fixtures: hex `2261225c0a226222` (`"a"` 后面紧接 backslash、LF，然后是 `"b"`); hex `2261225c200a226222` (backslash 与 LF 之间有 space); 以及 source `"a" \ // comment` 后接 LF 和 `"b"`. | 第一个 fixture 求值得到 String `"ab"`. 第二个和第三个报告 `LEX_BAD_CONTINUATION`, severity `error`, phase `lex`, 且不发布 String value. | `D-388` |
