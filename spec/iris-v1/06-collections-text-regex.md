# Iris v1 Collections, Text, Binary, Regex, And Stable Hashing

Status: Iris v1 draft, frozen semantics.

IRIS-V1-COLLECTIONS-C001: This chapter defines Tuple, Array, Hash, Range, Iterable, Iterator, Iteration, String, MutableString, Symbol, Bytes, ByteArray, Regex, Match, mutation, fail-fast traversal, equality, hashability, and stable BLAKE3 hashing for Iris v1. It MUST be read after [README.md](README.md), [01-language-identity.md](01-language-identity.md), [02-lexical-grammar.md](02-lexical-grammar.md), [03-runtime-object-model.md](03-runtime-object-model.md), and [04-bindings-callables-control-flow.md](04-bindings-callables-control-flow.md).

IRIS-V1-COLLECTIONS-C002: This chapter MUST NOT define implementation storage, bucket layout, rope layout, regex engine internals, Unicode table generation, parser productions, type algebra, serialization byte schemas, or native ABI details. It refines the literal, indexing, iteration, equality, and public hash surfaces fixed by the earlier chapters.

## Core Classification

IRIS-V1-COLLECTIONS-C003: The following table is normative for core value and container classification. `Specification-stable` means the public `hash` value is fixed by this specification while the built-in hash Method remains selected. `Runtime-local` means stable only for the lifetime of one runtime object identity under [03-runtime-object-model.md](03-runtime-object-model.md). `Unhashable` means the built-in public `hash` raises `InvalidKeyError`.

| Category                                                                                        | Identity                    | Mutability                          | Built-in equality                                     | Built-in public hash                                                                                | Hash key eligibility                                             | Iterator element shape                                                     |
| ----------------------------------------------------------------------------------------------- | --------------------------- | ----------------------------------- | ----------------------------------------------------- | --------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------- | -------------------------------------------------------------------------- |
| `nil`                                                                                         | Identity-bearing singleton  | Fixed singleton                     | Singleton equality                                    | Specification-stable singleton hash from[03-runtime-object-model.md](03-runtime-object-model.md)     | Yes                                                              | Not iterable by default                                                    |
| `false`, `true`                                                                             | Identity-bearing singletons | Fixed singleton                     | Bool value equality                                   | Specification-stable singleton hash from[03-runtime-object-model.md](03-runtime-object-model.md)     | Yes                                                              | Not iterable by default                                                    |
| `Integer`                                                                                     | Identity-less               | Immutable                           | Exact numeric value equality                          | Specification-stable numeric hash from[03-runtime-object-model.md](03-runtime-object-model.md)       | Yes unless a future numeric value is invalid under runtime rules | Not iterable by default                                                    |
| `Float32`, `Float64` finite or infinity                                                     | Identity-less               | Immutable                           | Exact numeric value equality, with signed zeros equal | Specification-stable numeric hash from[03-runtime-object-model.md](03-runtime-object-model.md)       | Yes except NaN                                                   | Not iterable by default                                                    |
| `Float32`, `Float64` NaN                                                                    | Identity-less               | Immutable                           | Unequal to every value                                | `InvalidKeyError`                                                                                 | No                                                               | Not iterable by default                                                    |
| Class, Module, Contract, Method, BoundMethod, Closure, Type, revision, ordinary identity object | Identity-bearing            | Category-specific                   | Identity-first default unless replaced                | Runtime-local identity hash unless category defines a stable hash                                   | Yes when current`hash` succeeds                                | Category-specific                                                          |
| Contract view                                                                                   | Identity-less               | Immutable                           | Receiver relation plus Contract identity              | Specification-stable Contract-view hash from[03-runtime-object-model.md](03-runtime-object-model.md) | Yes when receiver hash succeeds                                  | Not iterable by default                                                    |
| `String`                                                                                      | Identity-less               | Immutable                           | Exact Unicode scalar sequence and case                | Specification-stable string hash                                                                    | Yes                                                              | One-scalar immutable`String`                                             |
| `MutableString`                                                                               | Identity-bearing            | Mutable text content                | Current scalar content, cross-type equal to`String` | Unhashable by default                                                                               | No by default                                                    | One-scalar immutable`String`, fail-fast on content mutation              |
| `Symbol`                                                                                      | Identity-less               | Immutable canonical content         | Exact canonical content                               | Specification-stable symbol hash                                                                    | Yes                                                              | Not iterable by default                                                    |
| `Bytes`                                                                                       | Identity-less               | Immutable byte sequence             | Exact byte sequence, cross-type equal to`ByteArray` | Specification-stable bytes hash                                                                     | Yes                                                              | `Integer` byte in `0..255`                                             |
| `ByteArray`                                                                                   | Identity-bearing            | Mutable byte sequence               | Current byte content, cross-type equal to`Bytes`    | Unhashable by default                                                                               | No by default                                                    | `Integer` byte in `0..255`, fail-fast on content mutation              |
| `Tuple<T...>`                                                                                 | Identity-less               | Immutable element sequence          | Element order and dynamic element equality            | Specification-stable tuple hash when every element hash succeeds                                    | Conditional on every element                                     | Elements in order                                                          |
| `Array<T>`                                                                                    | Identity-bearing            | Mutable element sequence            | Current element order and dynamic element equality    | Unhashable by default                                                                               | No by default                                                    | Elements in order, fail-fast on element replacement or structural mutation |
| `Hash<K,V>`                                                                                   | Identity-bearing            | Mutable entry set and values        | Identity-first default unless replaced                | Runtime-local identity hash unless replaced                                                         | Yes by identity default                                          | `Tuple<K,V>` entries in unspecified order, structural fail-fast          |
| `Range`                                                                                       | Identity-less               | Immutable endpoints, openness, step | Endpoint, openness, and step equality                 | Specification-stable range hash when component hashes succeed                                       | Conditional on components                                        | Integer sequence for integer Ranges                                        |
| `Iterator<T>`                                                                                 | Identity-bearing            | Mutable cursor state                | Identity-only default                                 | Runtime-local identity hash                                                                         | Yes by identity default                                          | Not itself traversed by default unless it also implements`Iterable`      |
| `Iteration<T>` yield                                                                          | Identity-less               | Immutable variant and payload       | Yield variant plus payload dynamic equality           | Specification-stable iteration hash when payload hash succeeds                                      | Conditional on payload                                           | Not iterable by default                                                    |
| `Iteration.done`                                                                              | Identity-bearing singleton  | Fixed singleton                     | Singleton equality                                    | Specification-stable iteration hash                                                                 | Yes                                                              | Not iterable by default                                                    |
| `Regex`                                                                                       | Identity-less               | Immutable pattern and flags         | Canonical pattern plus canonical flags                | Specification-stable regex hash                                                                     | Yes                                                              | Not iterable by default                                                    |
| `Match`                                                                                       | Identity-less               | Immutable match result              | Full match, range, and capture value equality         | Unhashable by default                                                                               | No by default                                                    | Captures only through explicit APIs                                        |

IRIS-V1-COLLECTIONS-C004: Equal-implies-same-hash applies only when both public `hash` calls succeed. A value MAY be equal to a hashable value and still be unhashable itself, as with `MutableString` compared to `String` and `ByteArray` compared to `Bytes`.

IRIS-V1-COLLECTIONS-C005: Hash table bucket selection MUST use runtime-secret internal mixing of public hashes. Public `hash` results MUST remain the values defined by this chapter or by [03-runtime-object-model.md](03-runtime-object-model.md). Internal mixing MUST NOT be exposed as public hash output, iteration order, or persisted format.

## Index Units And Range Tokens

IRIS-V1-COLLECTIONS-C006: The grammar tokens `..=` and `..<` are the only Range literal operators. `a ..= b` creates a left-closed inclusive-end integer Range. `a ..< b` creates a left-closed exclusive-end integer Range. The `..identifier` Contract-view token remains postfix Contract-view syntax and MUST NOT be parsed as a Range.

IRIS-V1-COLLECTIONS-C007: Range literal endpoints MUST be `Integer` values. General Range construction for non-literal or fully open intervals belongs to explicit APIs. V1 source has no fully open, left-open, stepped, or reverse Range literal token.

IRIS-V1-COLLECTIONS-C008: Indexing units are fixed by receiver category. `String` and `MutableString` count Unicode scalar values. `Bytes` and `ByteArray` count bytes. `Tuple` and `Array` count element positions. Integer Range iteration counts integer values. No receiver implicitly switches between scalar, byte, grapheme, and element units.

IRIS-V1-COLLECTIONS-C009: Integer indexes MAY be negative where this chapter allows indexed access. A negative index resolves as `length + index` in the receiver's indexing unit. Reads outside the valid resolved range return `nil` unless a more specific write rule raises. Out-of-range scalar writes to mutable indexed receivers raise `IndexError`.

IRIS-V1-COLLECTIONS-C010: Unit-forward Range slicing resolves negative endpoints in the receiver's unit, clamps effective endpoints to `[0, length]`, respects inclusive or exclusive end openness, and returns an empty same-family result when the effective start is after the effective end. V1 slice syntax has no step or reverse traversal semantics.

## Iterable, Iterator, And Iteration

IRIS-V1-COLLECTIONS-C011: The canonical traversal Contracts are exactly:

```iris
contract Iterable<T> {
  fun iterator() -> Iterator<T>
}

contract Iterator<T> {
  fun next() -> Iteration<T>
  fun close() -> Nil
}
```

IRIS-V1-COLLECTIONS-C012: `for pattern in iterable` MUST obtain an `Iterator<T>` through `Iterable<T>.iterator()`, repeatedly call `next() -> Iteration<T>`, bind each `Iteration.yield(value)` payload, and terminate normally on `Iteration.done`. It MUST NOT use exceptions to signal ordinary exhaustion.

IRIS-V1-COLLECTIONS-C013: `Iteration.yield(value)` is an immutable identity-less value object. It MAY carry any Iris value, including `nil`. `Iteration.done` is a unique identity-bearing singleton. Repeated `next()` after exhaustion MUST return that same `Iteration.done` singleton.

IRIS-V1-COLLECTIONS-C014: `Iteration` exposes get-only properties `yield?`, `done?`, and `value`. For `Iteration.yield(payload)`, `yield?` returns `true`, `done?` returns `false`, and `value` returns the payload. For `Iteration.done`, `yield?` returns `false`, `done?` returns `true`, and `value` raises `IteratorStateError`.

IRIS-V1-COLLECTIONS-C015: Built-in `Iteration#<=>` returns `Integer(0)` for `done <=> done`, returns `nil` between `done` and any yield or between an `Iteration` and a non-Iteration, and forwards `yield(a) <=> yield(b)` to the payloads' current dynamic `<=>`. The `Iteration` Method MUST immediately validate that forwarded result as exact `Integer(-1)`, `Integer(0)`, `Integer(1)`, or `nil`, otherwise it raises `ComparisonContractError`.

IRIS-V1-COLLECTIONS-C016: Every explicit Iterator object is an identity-bearing mutable cursor. Each `iterator()` call MUST create a distinct Iterator identity unless identity observations remain exactly equivalent. Default Iterator equality is identity-only. Iterator public hash is runtime-local identity hash and MUST NOT depend on source position, source container content, expected structural version, or current entry.

IRIS-V1-COLLECTIONS-C017: An Iterator MUST strongly retain its source container until natural exhaustion or `close()`. On the first `next()` that returns `Iteration.done`, the Iterator enters permanent done state and releases its source, current-entry, and unneeded traversal references. A later `next()` returns `Iteration.done` without touching the source.

IRIS-V1-COLLECTIONS-C018: `Iterator#close() -> Nil` MUST be idempotent. Early close releases source, current-entry, and traversal references, enters permanent done state, and makes later `next()` return `Iteration.done`. Repeated `close()` calls succeed and return `nil`. Iterator identity and identity hash remain unchanged.

IRIS-V1-COLLECTIONS-C019: Language and standard traversal constructs MUST close active iterators on every exit path, including natural exhaustion, `break`, `continue` targeting an outer loop, `return`, body exception, Iterator exception, destructuring failure, and outer unwinding. Cleanup failure precedence follows [04-bindings-callables-control-flow.md](04-bindings-callables-control-flow.md).

IRIS-V1-COLLECTIONS-C020: Invalid Iterator-specific sequencing MUST raise `IteratorStateError`. This includes reading `Iteration.done.value`, invoking `HashIterator#remove_current` before a successful yield, invoking it twice for one yielded entry, invoking it after completion or close, and using concrete cursor operations after the cursor state no longer permits them.

## Container Operations And Fail-Fast Rules

IRIS-V1-COLLECTIONS-C021: Tuple literals are `()`, `(a,)`, and `(a, b, ...)`. Tuple values are immutable, identity-less heterogeneous product values with reified `Tuple<T...>` type. `Tuple#[]` accepts integer indexes with negative-index support and returns the element or `nil` out of range.

IRIS-V1-COLLECTIONS-C022: Tuple equality compares arity and elements in order through current dynamic element `==`. Tuple public hash succeeds only when every element public hash succeeds. A failed element hash propagates and prevents use as a Hash key.

IRIS-V1-COLLECTIONS-C023: Array literals `[a, b, ...]` create a fresh identity-bearing mutable `Array<T>`. `T` is inferred as the normalized union of element static types or constrained by an expected closed `Array<T>`. Empty `[]` requires expected closed `Array<T>` context. Programs that need an unconstrained empty Array MUST write an explicit construction such as `Array<T>.new()`.

IRIS-V1-COLLECTIONS-C024: Array integer reads use negative-index support and return `nil` out of range. Array scalar writes enforce the Array element Contract, replace an existing element, return `nil`, and raise `IndexError` out of range. Append and insert are the explicit growth operations.

IRIS-V1-COLLECTIONS-C025: Array Range slicing returns an independent mutable Array snapshot using unit-forward Range slicing. `array[range] = iterable` MUST atomically materialize and type-check replacement elements before changing the receiver. It MAY change length, returns `nil` on success, and leaves the Array unchanged on failure.

IRIS-V1-COLLECTIONS-C026: Array built-in equality compares current element sequence in order through dynamic equality. Array built-in `hash` raises `InvalidKeyError`. Any Array element replacement, append, insert, delete, clear, sort, range assignment, or length-changing operation increments the Array content version and causes active Array iterators to raise `ConcurrentModificationError` on their next advance.

IRIS-V1-COLLECTIONS-C027: Hash literals use `%{ key: value }`. Every key position is an ordinary expression. A bare identifier key reads that binding and MUST NOT become an implicit Symbol. Empty `%{}` requires expected `Hash<K,V>` context or explicit construction such as `Hash<K,V>.new()`.

IRIS-V1-COLLECTIONS-C028: Hash insertion, lookup, update, deletion, and rehash use each key's current dynamically dispatched `hash` and `==` Methods. They MUST NOT switch identity-bearing keys to `same?` unless the currently selected equality Method itself does so. A NaN key attempt raises `InvalidKeyError` under [03-runtime-object-model.md](03-runtime-object-model.md).

IRIS-V1-COLLECTIONS-C029: `hash[key] -> V?` returns `nil` when absent and MUST NOT insert. `hash[key] = value` enforces key and value Contracts, inserts or updates, and returns `nil`. `fetch(key)` raises `KeyError` when absent unless an explicit default or trailing block form handles absence. `delete(key) -> V?` returns the old value or `nil` if absent.

IRIS-V1-COLLECTIONS-C030: Existing Hash containers do not automatically track `==` or `hash` Method version changes. Users who mutate equality or hash behavior MUST call `rehash()` or recreate affected containers. Until then, lookup inconsistency is the user's responsibility.

IRIS-V1-COLLECTIONS-C031: `Hash#rehash()` MUST build and validate a temporary replacement from current keys, current public hashes, and current equality. If two previously distinct entries now collide into one equality class and no merge block is supplied, it raises `KeyConflictError` and publishes no partial result.

IRIS-V1-COLLECTIONS-C032: `Hash#rehash() { |kept_key, kept_value, incoming_key, incoming_value| ... }` MUST use the block result as a two-element `(key, value)` replacement for a collision class. The returned key MUST remain equal to the class under current equality and satisfy the current hash Contract. Block failure, shape failure, or new inconsistency aborts atomically and leaves the original Hash unchanged.

IRIS-V1-COLLECTIONS-C033: Hash iteration order is unspecified. It MUST NOT promise insertion order, stable-hash order, sorted order, cross-run order, or deterministic rehash merge order. Programs requiring deterministic ordering MUST extract entries and sort them through an explicit ordered structure.

IRIS-V1-COLLECTIONS-C034: Hash traversal is fail-fast for structural mutation. Adding a key, removing a key, clearing, or rehashing increments the structural version. Every active iterator whose expected structural version no longer matches MUST raise `ConcurrentModificationError` on next advance. Updating the value for an existing key is non-structural and allowed.

IRIS-V1-COLLECTIONS-C035: A Hash iterator yields each entry as a two-element `Tuple<K,V>`. The yielded key and value are ordinary bound arguments at yield time. Replacing a value after a tuple is yielded does not rewrite that tuple. An entry not yet yielded observes the latest value when yielded.

IRIS-V1-COLLECTIONS-C036: Hash concrete iterators MAY provide `remove_current() -> Nil`. It removes exactly the most recently yielded entry for that iterator, at most once per successful yield, updates that iterator's expected structural version, and does not make that same iterator fail-fast. Other active iterators observe the structural change and fail-fast on next advance.

IRIS-V1-COLLECTIONS-C037: `Hash#each` yields exactly `(key, value)` to its block. `Hash#each_with_iterator` yields exactly `(key, value, iterator)` and is the only standard block traversal surface that exposes `remove_current()`. No hidden current-iterator context exists. Nested traversals use distinct Iterator objects.

IRIS-V1-COLLECTIONS-C038: Range values are immutable identity-less interval values. Integer Range literals infer step `+1` when `end >= start` and `-1` when `end < start`. Equal endpoints yield one value for `..=` and empty for `..<`. `range.by(step: Integer)` returns a Range with nonzero step whose sign moves toward the end, otherwise it raises `RangeError`.

IRIS-V1-COLLECTIONS-C039: Integer Range iteration respects endpoint openness and step. A step that does not land exactly on the endpoint skips that endpoint. Range equality and public hash include start, end, endpoint openness, and step.

IRIS-V1-COLLECTIONS-C040: The following collection operation table is normative:

| Receiver          | Read                           | Write                                                                                   | Bounds or missing rule                                                                                     | Result rule                                                                                                                                 | Iterator rule                                                                                                 |
| ----------------- | ------------------------------ | --------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `Tuple`         | `tuple[i]`                   | No standard write                                                                       | Negative index allowed, out-of-range read returns`nil`                                                   | Element or`nil`                                                                                                                           | Stable element order                                                                                          |
| `Array`         | `array[i]`, `array[range]` | `array[i]=v`, `array[range]=iterable`, append, insert, delete, clear                | Negative index allowed, out-of-range read returns`nil`, out-of-range scalar write raises `IndexError`  | Scalar write returns`nil`, slice returns independent Array, range write returns `nil`                                                   | Any element replacement or structural mutation fail-fast                                                      |
| `Hash`          | `hash[key]`, `fetch(key)`  | `hash[key]=value`, `delete`, `clear`, `rehash`                                  | Missing`[]` returns `nil`, missing `fetch` raises unless handled, missing `delete` returns `nil` | Setter returns`nil`, delete returns old value or `nil`                                                                                  | Structural mutation fail-fast, existing-value update allowed,`remove_current` exception for owning iterator |
| `Range`         | Endpoint, step, openness APIs  | No standard write                                                                       | Wrong step direction or zero step raises`RangeError`                                                     | Iteration yields Integer values                                                                                                             | Stable immutable iteration                                                                                    |
| `String`        | `str[i]`, `str[range]`     | No standard write                                                                       | Scalar negative index allowed, out-of-range scalar read returns`nil`, slice clamps                       | Scalar read returns one-scalar`String?`, slice returns `String`                                                                         | Stable scalar iteration                                                                                       |
| `MutableString` | `text[i]`, `text[range]`   | `text[i]=one_scalar`, `text[range]=String`, append, clear, replace, bang transforms | Scalar out-of-range read returns`nil`, scalar write out of range raises `IndexError`                   | Writes return`nil` or receiver where Method says so, snapshots remain immutable                                                           | Any content mutation fail-fast                                                                                |
| `Bytes`         | `bytes[i]`, `bytes[range]` | No standard write                                                                       | Byte negative index allowed, out-of-range read returns`nil`, slice clamps                                | Scalar read returns`Integer?`, slice returns `Bytes`                                                                                    | Stable byte iteration                                                                                         |
| `ByteArray`     | `data[i]`, `data[range]`   | `data[i]=byte`, `data[range]=Bytes                                                    | ByteArray`, append, clear, replace                                                                         | Byte out-of-range read returns`nil`, scalar write out of range raises `IndexError`, byte value outside `0..255` raises `RangeError` | Writes return`nil` or receiver where Method says so, slice returns independent ByteArray                    |

## Text Values And Unicode

IRIS-V1-COLLECTIONS-C041: Iris v1 String values contain only valid Unicode scalar values. They MUST NOT contain invalid UTF-8, surrogate code points, or raw bytes. `\xNN` in a String literal denotes scalar U+0000 through U+00FF, not a byte injection.

IRIS-V1-COLLECTIONS-C042: Iris language major version 1 fixes default Unicode data to Unicode 17.0.0 for identifier XID tables, grapheme segmentation, normalization, Unicode properties, case mapping, and casefold. This exact version is owner-confirmed by revised D-407. OS libraries, host configuration, process locale, and package selection MUST NOT change default language results. A different default Unicode data version requires a future Iris language major or an explicit versioned Unicode API.

IRIS-V1-COLLECTIONS-C043: String equality compares exact Unicode scalar sequence and case. It performs no implicit normalization, no case folding, no locale mapping, and no grapheme equivalence. `String#hash` uses the exact UTF-8 encoding of the scalar sequence after the String value has been formed.

IRIS-V1-COLLECTIONS-C044: `String#length`, integer `String#[]`, String Range slicing, and default String iteration use Unicode scalar units. `String#byte_length` and `String#bytes` expose UTF-8 bytes explicitly. `String#graphemes` exposes Unicode grapheme clusters explicitly using the fixed Unicode data version.

IRIS-V1-COLLECTIONS-C045: `str[i]` returns a one-scalar read-only String value for an in-range scalar index and `nil` out of range. V1 introduces no separate Character type for this operation.

IRIS-V1-COLLECTIONS-C046: `str[range]` returns a read-only String slice using scalar unit-forward Range slicing. Empty or start-after-end slices return the empty String. Unsupported stepped or reverse slicing is statically rejected when known and raises `ArgumentError` dynamically otherwise.

IRIS-V1-COLLECTIONS-C047: Standard String has no `[]=`. Functional text APIs return new String values. User-added String Methods MUST NOT mutate intrinsic String scalar content.

IRIS-V1-COLLECTIONS-C048: Interpolation evaluates each `${expr}` left-to-right. A non-String interpolation value is converted by dynamic `to_string() -> String`. Missing lookup may use `method_missing`. A non-String conversion result raises `TypeContractError`. On any failure, later segment expressions do not run and no partial String value is returned.

IRIS-V1-COLLECTIONS-C049: Root `Object#to_string() -> String` initially returns `<fully.qualified.ClassName>` using nominal Class identity and omitting memory address, identity hash, runtime ID, revision number, properties, and ivars. `Object#inspect() -> String` initially delegates to `to_string` and MUST NOT bypass ReflectionPolicy.

IRIS-V1-COLLECTIONS-C050: `String#to_string` returns the receiver. `String#inspect` returns a reparsable double-quoted escaped literal that recreates a scalar-equal String value and does not execute interpolation when parsed.

IRIS-V1-COLLECTIONS-C051: `String#+(other: Object) -> String` appends `other` directly when it is a String, otherwise invokes `other.to_string() -> String`. The Method is ordinary and dynamically replaceable. Adjacent String literal segments concatenate as one expression according to [02-lexical-grammar.md](02-lexical-grammar.md); Symbols, Bytes, ByteArray, and arbitrary expressions do not join implicitly.

IRIS-V1-COLLECTIONS-C052: MutableString is the standard mutable text type. Each `m` literal evaluation creates a fresh identity-bearing MutableString. Prefix order and raw fences are owned by [02-lexical-grammar.md](02-lexical-grammar.md). Content parsing, interpolation, and multiline indentation follow the corresponding String literal family before the MutableString value is created.

IRIS-V1-COLLECTIONS-C053: `MutableString#to_string() -> String` returns a String snapshot of current content. Later MutableString mutation MUST NOT affect prior snapshots. Copy-on-write, ropes, and shared read-only segments are allowed only when unobservable.

IRIS-V1-COLLECTIONS-C054: `MutableString#[]` reads use String scalar indexing. `MutableString#[i]=replacement` requires a one-scalar String, raises `IndexError` out of range, mutates in place, and returns `nil`. `MutableString#[range]=replacement` uses scalar unit-forward slicing, accepts any String, may change length, mutates in place, and returns `nil`.

IRIS-V1-COLLECTIONS-C055: MutableString built-in equality compares current exact scalar content and is cross-type equal to String with identical content. Built-in `hash` raises `InvalidKeyError`. A user-added MutableString hash MUST match String-compatible equality and carries the ordinary mutation and rehash responsibility.

IRIS-V1-COLLECTIONS-C056: `MutableString#+(other: Object) -> MutableString` converts the operand using String or MutableString snapshot semantics or `to_string`, then returns a fresh MutableString identity. `MutableString#<<(other: Object)` and `MutableString#append(other: Object)` use the same conversion, mutate the receiver in place after full conversion succeeds, and return the receiver.

IRIS-V1-COLLECTIONS-C057: `MutableString += other` uses ordinary compound assignment. It reads the current target, invokes `MutableString#+`, and writes the fresh MutableString result back. Existing aliases to the previous MutableString do not observe an append. In-place mutation requires `<<` or `append`.

IRIS-V1-COLLECTIONS-C058: MutableString `clear() -> MutableString`, `replace(other: Object) -> MutableString`, `<<`, `append`, scalar assignment, range assignment, and bang transforms MUST commit atomically. Conversion, allocation, type, or resource failure leaves previous content unchanged and propagates the error. Self or alias append snapshots source text before mutation.

IRIS-V1-COLLECTIONS-C059: Standard text transforms use non-bang names for allocating results and bang names for MutableString mutation. String transforms such as `upcase`, `downcase`, `normalize`, and `casefold` return new String values. MutableString non-bang counterparts return fresh MutableStrings, while `upcase!`, `downcase!`, `normalize!`, and `casefold!` mutate atomically and return the receiver.

IRIS-V1-COLLECTIONS-C060: MutableString is not logically thread-safe. Unsynchronized read/write or write/write concurrency may raise `ConcurrentMutationError`, but MUST remain memory-safe, MUST NOT expose invalid Unicode, and MUST publish either old complete content or new complete content to correctly synchronized observers.

IRIS-V1-COLLECTIONS-C061: MutableString scalar and grapheme iterators capture a content version. Append, clear, replace, scalar assignment, range assignment, bang transforms, or any content change increments it. The next advance on mismatch raises `ConcurrentModificationError`. Stable traversal uses an explicit `to_string` snapshot.

IRIS-V1-COLLECTIONS-C062: Default String and MutableString iteration yields one-scalar String values in scalar order. `bytes` and `graphemes` expose lazy Iterable views rather than eager Arrays. MutableString views capture content version and fail-fast on content mutation.

## Symbols

IRIS-V1-COLLECTIONS-C063: Symbol literals use the simple and quoted forms defined by [02-lexical-grammar.md](02-lexical-grammar.md). Simple Symbols from identifiers, selector identifiers, operators, and raw-ivar forms use their corresponding canonical source content. Quoted arbitrary-text Symbols use String escape rules, never interpolate, and preserve exact post-escape scalar sequence without identifier NFC normalization.

IRIS-V1-COLLECTIONS-C064: Symbols are immutable identity-less interned-name values. Symbol equality is exact canonical-content equality. `same?` on a Symbol raises `IdentityError`. Interning is an implementation sharing strategy and MUST NOT become observable identity.

IRIS-V1-COLLECTIONS-C065: Ordinary identifiers are case-sensitive and lexer-normalized to Unicode NFC before name, selector, and Type identity. Simple Symbols reuse that normalized content. Quoted arbitrary-text Symbols preserve case and canonical distinctions exactly. Stable Symbol hashing uses the resulting canonical Symbol content.

IRIS-V1-COLLECTIONS-C066: Raw ivar reflection APIs that accept names require a Symbol beginning with exactly one `@` followed by a valid ordinary identifier, with no selector suffix. Strings MUST NOT convert to raw-ivar Symbols implicitly.

## Binary Values And Encoding

IRIS-V1-COLLECTIONS-C067: `Bytes` is an immutable identity-less byte sequence. `ByteArray` is an identity-bearing mutable byte sequence. Both contain byte values in `0..255`. Non-ASCII source text in byte literals contributes its UTF-8 bytes, and escaped byte literal `\xNN` injects one raw byte.

IRIS-V1-COLLECTIONS-C068: Bytes and ByteArray built-in equality compares exact current byte sequence and permits cross-type equality. Bytes built-in public hash is stable. ByteArray built-in `hash` raises `InvalidKeyError`. A user-added ByteArray hash MUST match Bytes-compatible equality and carries mutation and rehash responsibility.

IRIS-V1-COLLECTIONS-C069: `Bytes#[]` and `ByteArray#[]` read byte units with negative-index support and return `Integer?` in `0..255`, or `nil` out of range. `ByteArray#[i]=value` requires an Integer byte in `0..255`, raises `RangeError` for an invalid byte value, raises `IndexError` out of range, mutates in place, and returns `nil`.

IRIS-V1-COLLECTIONS-C070: Bytes Range slicing returns Bytes. ByteArray Range slicing returns an independent ByteArray snapshot identity. Both use byte unit-forward Range slicing. `ByteArray#[range]=replacement` accepts Bytes or ByteArray, snapshots aliasing sources, may change length, atomically replaces the range, returns `nil`, and exposes no partial content.

IRIS-V1-COLLECTIONS-C071: `Bytes + (Bytes|ByteArray) -> Bytes` returns an immutable Bytes value. `ByteArray + (Bytes|ByteArray) -> ByteArray` returns a fresh ByteArray identity without changing the receiver. `ByteArray#<<` and `ByteArray#append` mutate in place after snapshotting aliases, return the receiver, and commit atomically. `ByteArray += other` uses `+` then rebinds.

IRIS-V1-COLLECTIONS-C072: Binary operations MUST NOT encode String, call `to_string`, or accept Object through text conversion. Text and binary conversion happens only through explicit APIs.

IRIS-V1-COLLECTIONS-C073: `String#to_bytes` and `MutableString#to_bytes` return immutable UTF-8 Bytes snapshots. `Bytes#to_string` strictly decodes UTF-8 and raises `EncodingError` on invalid sequences. ByteArray snapshots then decodes equivalently. Lossy replacement or ignore behavior requires explicit decoding options and is never the default.

IRIS-V1-COLLECTIONS-C074: Language core guarantees UTF-8 source text and UTF-8 text-to-binary convenience conversion only. Standard library Encoding objects define additional encodings such as UTF-16LE, UTF-16BE, and Latin-1 with strict default errors and explicit replace or ignore options. OS locale and code page MUST NOT be selected implicitly.

IRIS-V1-COLLECTIONS-C075: Default Bytes and ByteArray iteration yields Integer byte values in source order. ByteArray iterators capture content version, and any ByteArray content mutation causes `ConcurrentModificationError` on next advance.

## Regex And Match

IRIS-V1-COLLECTIONS-C076: Regex literals use `/pattern/flags` or raw `r/pattern/flags` with raw fences as defined by [02-lexical-grammar.md](02-lexical-grammar.md). Non-raw Regex literals allow `${expr}` interpolation. Each interpolated value is evaluated once, converted through dynamic `to_string() -> String`, then escaped with default Regex escaping before insertion into the pattern.

IRIS-V1-COLLECTIONS-C077: Regex is an immutable identity-less core value. Regex equality and public hash use canonical pattern text plus canonical flags. `same?` on Regex raises `IdentityError`.

IRIS-V1-COLLECTIONS-C078: Core Regex supports a fixed Unicode-aware safe subset with deterministic linear-time-oriented matching. It includes literal text, concatenation, alternation, grouping, character classes, Unicode properties tied to Unicode 17.0.0, anchors, word boundaries, greedy and lazy bounded or unbounded repetition, named captures, numbered captures, and non-capturing groups.

IRIS-V1-COLLECTIONS-C079: Core Regex MUST NOT support backreferences, lookbehind, subroutine calls, conditionals, atomic groups, possessive quantifiers, recursion, embedded code, engine callbacks, global match variables, or constructs that require unbounded catastrophic backtracking. Advanced PCRE-style engines belong in separately versioned standard packages and MUST use distinct types.

IRIS-V1-COLLECTIONS-C080: Regex flags are exactly `i`, `m`, `s`, and `x`. `i` enables Unicode case-insensitive matching, `m` makes line anchors operate per line, `s` lets dot match newline, and `x` ignores pattern whitespace and comments according to the core Regex lexical rules. Unicode mode is always on and fixed by the Iris language major. Duplicate or unsupported flags MUST be diagnosed as `LEX_BAD_REGEX_FLAGS` or a stricter Regex diagnostic.

IRIS-V1-COLLECTIONS-C081: Canonical Regex flags MUST be stored in fixed order `imsx`, with absent flags omitted. Equality and hashing compare canonical flags, so `/a/im` and `/a/mi` are equal if both source forms are accepted. Duplicate flags are invalid before canonicalization.

IRIS-V1-COLLECTIONS-C082: `text =~ regex -> Match?` evaluates the text and Regex once, requires text to be String or MutableString snapshot content, and returns an immutable Match for the first match or `nil`. `text !~ regex -> Bool` returns `true` exactly when `text =~ regex` would return `nil`, except that evaluation or Regex errors propagate.

IRIS-V1-COLLECTIONS-C083: Match values are immutable. A Match exposes the full match, scalar ranges, byte ranges in UTF-8, numbered captures, named captures, and the Regex used. Capture absence is distinct from an empty capture. Match APIs MUST NOT expose global variables or mutable engine state.

IRIS-V1-COLLECTIONS-C084: Regex matching over MutableString MUST operate on a snapshot of the content captured before matching starts. Later MutableString mutation MUST NOT rewrite an existing Match.

## Stable Public Hash Domains And Encodings

IRIS-V1-COLLECTIONS-C085: Every stable public hash in Iris v1 MUST return a nonnegative Integer in `0..2^64-1`. Unless this chapter says another failure applies, stable hashes use BLAKE3 derive-key mode with the exact ASCII context in the table, the canonical input bytes in the table, the standard 32-byte digest, and digest bytes `0..8` interpreted as unsigned little-endian. Context bytes are not manually concatenated to input.

IRIS-V1-COLLECTIONS-C086: Canonical `ULEB128(n)` means the shortest unsigned LEB128 encoding of nonnegative Integer `n`. Encodings with redundant continuation groups are noncanonical. `u64_le(hash)` means exactly eight bytes of a public hash value in unsigned little-endian order.

IRIS-V1-COLLECTIONS-C087: Stable hash context strings are globally domain-separated. A conforming implementation MUST use these exact contexts and MUST NOT reuse one context for another value family.

| Family        | Exact ASCII BLAKE3 derive-key context      | Canonical input bytes                                                                                             | Hash failure rule             |
| ------------- | ------------------------------------------ | ----------------------------------------------------------------------------------------------------------------- | ----------------------------- |
| Numeric       | `Iris Language v1 stable numeric hash`   | Numeric canonical bytes from [03-runtime-object-model.md](03-runtime-object-model.md) | NaN raises `InvalidKeyError` |
| Singleton     | `Iris Language v1 stable singleton hash` | One-byte inputs for `nil`, `false`, and `true` from [03-runtime-object-model.md](03-runtime-object-model.md) | None for the three singletons |
| Contract view | `Iris Language v1 contract view hash`    | `receiver_public_hash_u64_le || contract_type_hash_u64_le` | Receiver or Contract Type hash failure propagates |
| Symbol        | `Iris Language v1 stable symbol hash`    | `ULEB128(utf8_byte_length) || canonical_symbol_utf8` | None |
| String        | `Iris Language v1 stable string hash`    | `ULEB128(utf8_byte_length) || exact_scalar_sequence_utf8` | None |
| Bytes         | `Iris Language v1 stable bytes hash`     | `ULEB128(byte_length) || bytes` | None |
| Tuple         | `Iris Language v1 stable tuple hash`     | `ULEB128(element_count) || element_0_public_hash_u64_le || ...` | Element hash failure propagates |
| Range         | `Iris Language v1 stable range hash`     | `openness_tag || start_public_hash_u64_le || end_public_hash_u64_le || step_public_hash_u64_le` | Component hash failure propagates |
| Iteration     | `Iris Language v1 stable iteration hash` | `Iteration.done`: `[0x00]`; `Iteration.yield(payload)`: `[0x01] || payload_public_hash_u64_le` | Payload hash failure propagates |
| Regex         | `Iris Language v1 stable regex hash`     | `ULEB128(pattern_utf8_length) || pattern_utf8 || ULEB128(canonical_flags_ascii_length) || canonical_flags_ascii` | None |

IRIS-V1-COLLECTIONS-C088: Range hash `openness_tag` is `[0x00]` for inclusive-end Ranges and `[0x01]` for exclusive-end Ranges. Start, end, and step use the current public hashes of their values. Integer Range components use numeric public hashes from [03-runtime-object-model.md](03-runtime-object-model.md).

IRIS-V1-COLLECTIONS-C089: Tuple, Range, Iteration yield, and Contract-view stable hashes are compositional over public hash values. They MUST propagate `InvalidKeyError` or another hash failure from any component. They MUST NOT fall back to object identity for identity-less wrappers.

IRIS-V1-COLLECTIONS-C090: Stable public hash canonical bytes are specification-internal unless this specification separately exposes a value format. User programs observe only the public `hash` result. The future serialization and standard-library chapter owns persistent IrisValue format responsibilities.

IRIS-V1-COLLECTIONS-C091: The following stable hash vector table is normative. Digest prefixes and public hash values were computed with BLAKE3 derive-key mode using the contexts named in IRIS-V1-COLLECTIONS-C087.

| Vector ID                    | Value                                | Canonical input hex                                    | Context                                    | Digest bytes`0..8` hex | Public hash Integer      |
| ---------------------------- | ------------------------------------ | ------------------------------------------------------ | ------------------------------------------ | ------------------------ | ------------------------ |
| `IRIS-V1-COLLECTIONS-V001` | `:name`                            | `046e616d65`                                         | `Iris Language v1 stable symbol hash`    | `e190f8954b4fa498`     | `10999003376002830561` |
| `IRIS-V1-COLLECTIONS-V002` | `:@x`                              | `024078`                                             | `Iris Language v1 stable symbol hash`    | `f498d4107fb3e9de`     | `16062566904318171380` |
| `IRIS-V1-COLLECTIONS-V003` | `""`                               | `00`                                                 | `Iris Language v1 stable string hash`    | `23886f048e56bf77`     | `8628710579024922659`  |
| `IRIS-V1-COLLECTIONS-V004` | `"Iris"`                           | `0449726973`                                         | `Iris Language v1 stable string hash`    | `2df84e8634b29e29`     | `2999030340536694829`  |
| `IRIS-V1-COLLECTIONS-V005` | `b""`                              | `00`                                                 | `Iris Language v1 stable bytes hash`     | `475a703c0515b46d`     | `7904966358175078983`  |
| `IRIS-V1-COLLECTIONS-V006` | Bytes`[0xff, 0x00, 0x01, 0x02]`    | `04ff000102`                                         | `Iris Language v1 stable bytes hash`     | `9f87b39f88b29df1`     | `17410268034348844959` |
| `IRIS-V1-COLLECTIONS-V007` | `()`                               | `00`                                                 | `Iris Language v1 stable tuple hash`     | `0cc6fb61f1157f8c`     | `10123834613827356172` |
| `IRIS-V1-COLLECTIONS-V008` | `(nil, true)` using runtime hashes | `02d37c681abf4074a440168bee0b35dfc4`                 | `Iris Language v1 stable tuple hash`     | `1697cf0d42d50e9c`     | `11245159799266973462` |
| `IRIS-V1-COLLECTIONS-V009` | `1 ..= 3` with step `1`          | `00384e0f3cb1fc5bf76bc4f1137cdbfcc8384e0f3cb1fc5bf7` | `Iris Language v1 stable range hash`     | `894974389baabd35`     | `3872438838252292489`  |
| `IRIS-V1-COLLECTIONS-V010` | `/a+/im` canonical flags `im`    | `02612b02696d`                                       | `Iris Language v1 stable regex hash`     | `4446d3f2271a0b18`     | `1732507240534066756`  |
| `IRIS-V1-COLLECTIONS-V011` | `Iteration.done`                   | `00`                                                 | `Iris Language v1 stable iteration hash` | `bc123979d794e248`     | `5251923768640082620`  |
| `IRIS-V1-COLLECTIONS-V012` | `Iteration.yield(nil)`             | `01d37c681abf4074a4`                                 | `Iris Language v1 stable iteration hash` | `44848ba05f9798bf`     | `13805951094675440708` |

## Examples And Conformance Vectors

IRIS-V1-COLLECTIONS-EX001: Informative example, Range tokens and indexing units:

```iris
let closed = 1 ..= 3      // yields 1, 2, 3
let half_open = 1 ..< 3   // yields 1, 2
let text = "a\u{1f600}b"
text.length               // 3 scalar values
text[1]                   // one-scalar String containing U+1F600
text.to_bytes().length    // UTF-8 byte count, not scalar count
```

IRIS-V1-COLLECTIONS-EX002: Informative example, mutable aliases and snapshots:

```iris
let builder = m"ab"
let snapshot = builder.to_string()
builder << "c"
snapshot                  // "ab"
builder.to_string()       // "abc"
```

IRIS-V1-COLLECTIONS-EX003: Informative example, Hash traversal mutation:

```iris
let table = %{ :a: 1, :b: 2 }
table.each_with_iterator() { |key: Symbol, value: Integer, iterator: Iterator<Tuple<Symbol,Integer>>| -> Nil
  if key == :a {
    iterator.remove_current()
  }
}
```

IRIS-V1-COLLECTIONS-EX004: Informative example, Regex snapshot and safe subset:

```iris
let name = "item"
let regex = /^${name}[0-9]+$/i
let result = "Item42" =~ regex
```

IRIS-V1-COLLECTIONS-C092: The following vector table is normative. The conformance chapter MUST preserve these vector IDs or map them to machine-readable records with the same observable outcomes:

| Vector ID                    | Kind     | Scenario                                                 | Expected result                                        |
| ---------------------------- | -------- | -------------------------------------------------------- | ------------------------------------------------------ |
| `IRIS-V1-COLLECTIONS-V013` | Positive | `String#[]` over `"a\u{1f600}b"` at index `1`      | One-scalar String U+1F600                              |
| `IRIS-V1-COLLECTIONS-V014` | Failure  | String index assignment                                  | Missing standard`[]=` or mutation rejection          |
| `IRIS-V1-COLLECTIONS-V015` | Positive | MutableString`to_string` snapshot before append        | Snapshot unchanged after later mutation                |
| `IRIS-V1-COLLECTIONS-V016` | Failure  | MutableString as Hash key with built-in hash             | `InvalidKeyError`                                    |
| `IRIS-V1-COLLECTIONS-V017` | Positive | `Bytes#to_string` on valid UTF-8                       | Exact String result                                    |
| `IRIS-V1-COLLECTIONS-V018` | Failure  | `Bytes#to_string` on invalid UTF-8                     | `EncodingError`                                      |
| `IRIS-V1-COLLECTIONS-V019` | Failure  | `ByteArray#[i]=256`                                    | `RangeError`                                         |
| `IRIS-V1-COLLECTIONS-V020` | Positive | Tuple containing only hashable elements used as Hash key | Tuple hash succeeds                                    |
| `IRIS-V1-COLLECTIONS-V021` | Failure  | Tuple containing MutableString used as Hash key          | `InvalidKeyError`                                    |
| `IRIS-V1-COLLECTIONS-V022` | Positive | Array slice then mutate original Array                   | Slice remains independent snapshot                     |
| `IRIS-V1-COLLECTIONS-V023` | Failure  | Active Array iterator after element replacement          | `ConcurrentModificationError` on next advance        |
| `IRIS-V1-COLLECTIONS-V024` | Positive | Active Hash iterator after existing value update         | Not-yet-yielded entry observes latest value            |
| `IRIS-V1-COLLECTIONS-V025` | Failure  | Active Hash iterator after key insertion                 | `ConcurrentModificationError` on next advance        |
| `IRIS-V1-COLLECTIONS-V026` | Failure  | Hash`rehash()` where two keys now compare equal        | `KeyConflictError`, original Hash unchanged          |
| `IRIS-V1-COLLECTIONS-V027` | Positive | `Hash#each_with_iterator` valid `remove_current`     | Current entry removed, owning iterator continues       |
| `IRIS-V1-COLLECTIONS-V028` | Failure  | `remove_current` twice for one yielded entry           | `IteratorStateError`                                 |
| `IRIS-V1-COLLECTIONS-V029` | Positive | `Iteration.yield(nil)` in a `for` traversal          | Body receives legitimate`nil`, done terminates later |
| `IRIS-V1-COLLECTIONS-V030` | Failure  | Reading`Iteration.done.value`                          | `IteratorStateError`                                 |
| `IRIS-V1-COLLECTIONS-V031` | Positive | `1 ..= 3` and `1 ..< 3`                              | Closed yields three values, half-open yields two       |
| `IRIS-V1-COLLECTIONS-V032` | Failure  | `range.by(step: 0)`                                    | `RangeError`                                         |
| `IRIS-V1-COLLECTIONS-V033` | Positive | Regex interpolation of`"a+b"`                          | Inserted as escaped literal text                       |
| `IRIS-V1-COLLECTIONS-V034` | Failure  | Regex backreference in core Regex                        | Regex diagnostic for unsupported construct             |
| `IRIS-V1-COLLECTIONS-V035` | Failure  | Duplicate Regex flag`/a/ii`                            | `LEX_BAD_REGEX_FLAGS` or stricter Regex diagnostic   |
| `IRIS-V1-COLLECTIONS-V036` | Positive | Stable hash vectors V001 through V012                    | Exact Integer outputs in IRIS-V1-COLLECTIONS-C091      |
| `IRIS-V1-COLLECTIONS-V037` | Positive | Two Hash values with the same entries are iterated after different insertion histories | Each yields every entry exactly once; no insertion, stable-hash, sorted, or cross-run order is asserted |
| `IRIS-V1-COLLECTIONS-V038` | Positive | Default Unicode data version used by identifiers, graphemes, normalization, casefold, and Regex properties | All observations use Unicode 17.0.0 tables, independent of OS locale or host libraries |
| `IRIS-V1-COLLECTIONS-V039` | Failure  | Core Regex uses a Unicode property result from the host Unicode version instead of Unicode 17.0.0 | Conformance rejects host-version behavior unless it matches Unicode 17.0.0 |


## Direct Cross-Decision Regex Vector

IRIS-V1-COLLECTIONS-C097: The following vector is normative because its concrete source directly observes the Unicode-version, core Regex surface, and safe-subset/package-split decisions together.

| Vector ID | Category | Applicability | Source/Input | Expected observable | Decisions |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-COLLECTIONS-V042` | diagnostic | compiler required; interpreter required; JIT required; native not applicable | Four isolated sources under a host Unicode implementation older than 17.0.0: `(1)` `[Unicode.version(), "A" =~ /\p{Lu}/, "a\nB" =~ /^B/m]`; `(2)` `AdvancedRegex.compile("(?<=a)b").match("ab")`; `(3)` `/(?<=a)b/`; `(4)` `/a/ii`. | Source 1 returns `["17.0.0", Match, Match]`; source 2 returns its package-defined Match through the distinct `AdvancedRegex` Type; source 3 reports `REGEX_UNSUPPORTED_LOOKBEHIND`; source 4 reports `LEX_BAD_REGEX_FLAGS`. Core results remain Unicode 17.0.0, and the advanced package neither replaces core Regex nor changes literal semantics. | `D-407`, `D-505`, `D-506` |

## Traceability Notes

IRIS-V1-COLLECTIONS-C093: This chapter owns or refines the collections, text, binary, Regex, iteration, and stable-hash portions of D-116 through D-139, D-337 through D-341, D-368 through D-414, D-459 through D-467, D-505, and D-506. D-336 is a cross-chapter raw-ivar Symbol-validation dependency with a local collections anchor in IRIS-V1-COLLECTIONS-C066; this chapter does not own raw-ivar reflection operations. It reuses the numeric, singleton, and Contract-view hash anchors from D-071 through D-089 and [03-runtime-object-model.md](03-runtime-object-model.md) instead of redefining them.

IRIS-V1-COLLECTIONS-C094: D-319 through D-336 were read as runtime, Module, ReflectionPolicy, and raw-ivar Symbol-name dependencies. Their normative ownership remains in [03-runtime-object-model.md](03-runtime-object-model.md), [02-lexical-grammar.md](02-lexical-grammar.md), and later metaprogramming chapters. This chapter uses only their Symbol, raw-ivar name, and receiver-state consequences where collection and text APIs refer to them.

IRIS-V1-COLLECTIONS-C095: D-365 through D-367 are source-text and comment dependencies owned by [02-lexical-grammar.md](02-lexical-grammar.md). This chapter owns runtime String, MutableString, Bytes, ByteArray, Unicode, Encoding, and Regex value behavior beginning with the literal results defined by that grammar chapter. Revised D-407 owner-confirmed Unicode 17.0.0 as the exact Iris v1 Unicode data version used by IRIS-V1-COLLECTIONS-C042 and Regex Unicode properties.

IRIS-V1-COLLECTIONS-C096: Covered and dependency decision IDs are `D-071`, `D-072`, `D-073`, `D-074`, `D-075`, `D-076`, `D-077`, `D-078`, `D-079`, `D-080`, `D-081`, `D-082`, `D-083`, `D-084`, `D-085`, `D-086`, `D-087`, `D-088`, `D-089`, `D-116`, `D-117`, `D-118`, `D-119`, `D-120`, `D-121`, `D-122`, `D-123`, `D-124`, `D-125`, `D-126`, `D-127`, `D-128`, `D-129`, `D-130`, `D-131`, `D-132`, `D-133`, `D-134`, `D-135`, `D-136`, `D-137`, `D-138`, `D-139`, `D-319`, `D-320`, `D-321`, `D-322`, `D-323`, `D-324`, `D-325`, `D-326`, `D-327`, `D-328`, `D-329`, `D-330`, `D-331`, `D-332`, `D-333`, `D-334`, `D-335`, `D-336`, `D-337`, `D-338`, `D-339`, `D-340`, `D-341`, `D-365`, `D-366`, `D-367`, `D-368`, `D-369`, `D-370`, `D-371`, `D-372`, `D-373`, `D-374`, `D-375`, `D-376`, `D-377`, `D-378`, `D-379`, `D-380`, `D-381`, `D-382`, `D-383`, `D-384`, `D-385`, `D-386`, `D-387`, `D-388`, `D-389`, `D-390`, `D-391`, `D-392`, `D-393`, `D-394`, `D-395`, `D-396`, `D-397`, `D-398`, `D-399`, `D-400`, `D-401`, `D-402`, `D-403`, `D-404`, `D-405`, `D-406`, `D-407`, `D-408`, `D-409`, `D-410`, `D-411`, `D-412`, `D-413`, `D-414`, `D-459`, `D-460`, `D-461`, `D-462`, `D-463`, `D-464`, `D-465`, `D-466`, `D-467`, `D-505`, and `D-506`.

## Audit-Exact Conformance Vectors

These rows are normative audit-exact vectors. `Source/Input` is executable Iris unless it explicitly names a raw-byte or Host fixture.

| Vector ID | Category | Applicability | Source/Input | Expected observable | Decisions |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-COLLECTIONS-V275` | positive | compiler required; interpreter required; JIT optional; native not applicable | `a = Key.new(7); b = Key.new(7); h = %{ a: 1 }; h[b] = 2; [h.fetch(a), h.delete(b)]`, where `Key#==` and `Key#hash` use `id`. | Result `[2, 2]`; Hash uses dynamic `==`/`hash`, not identity. | `D-116` |
| `IRIS-V1-COLLECTIONS-V276` | positive | compiler required; interpreter required; JIT optional; native not applicable | `k = Key.new(1); h = %{ k: :ok }; k.hash_code = 2; before = h[k]; h.rehash(); [before, h.fetch(k)]`. | Result `[nil, :ok]`; only `rehash()` makes the new hash protocol effective. | `D-117` |
| `IRIS-V1-COLLECTIONS-V277` | negative | compiler required; interpreter required; JIT required; native not applicable | `a = Key.new(1, 11); b = Key.new(2, 22); h = %{ a: :a, b: :b }; a.equal_id = 0; b.equal_id = 0; h.rehash()`, where `Key#==` compares `equal_id` and `Key#hash` hashes it. | Raises `KeyConflictError`; side effect is none: `h.size == 2`, `h.fetch(a) == :a`, and `h.fetch(b) == :b`. | `D-118` |
| `IRIS-V1-COLLECTIONS-V278` | negative | compiler required; interpreter required; JIT required; native not applicable | `a = Key.new(0); b = Key.new(1); h = %{ a: 2, b: 3 }; b.equal_id = 0; h.rehash() { |kept, left, incoming, right| (kept, left + right) }; before = h.to_array(); h.rehash() { |kept, left, incoming, right| :bad }`. | First call returns `nil` and leaves one entry whose value is Integer `5`; second call raises `TypeContractError` and leaves `h.to_array()` equal as an unordered entry set to `before`. | `D-119` |
| `IRIS-V1-COLLECTIONS-V279` | positive | compiler required; interpreter required; JIT required; native not applicable | `a = %{ :a: 1, :b: 2 }; b: Hash<Symbol,Integer> = %{}; b[:b] = 2; b[:a] = 1; [a.to_array(), b.to_array()]`. | Both Arrays, interpreted only as unordered entry multisets, equal `{(:a, 1), (:b, 2)}` with each entry exactly once. No relative, insertion, sorted, stable-hash, rehash, or cross-run order is expected. | `D-120` |
| `IRIS-V1-COLLECTIONS-V280` | positive | compiler required; interpreter required; JIT required; native not applicable | `keys = [Key.new(1, 10), Key.new(2, 10), Key.new(3, 20), Key.new(4, 20)]; h = %{ keys[0]: 1, keys[1]: 2, keys[2]: 4, keys[3]: 8 }; pairs = []; h.rehash() { |kept, left, incoming, right| pairs << Set.of(kept.id, incoming.id); (kept, left + right) }`. | `pairs` is the unordered set `{Set.of(1,2), Set.of(3,4)}` and final values are the unordered set `{3,12}`; callback order and which equal key is kept are not expected. | `D-121` |
| `IRIS-V1-COLLECTIONS-V281` | diagnostic | compiler required; interpreter required; JIT optional; native not applicable | `h = %{ :a: 1 }; it = h.iterator(); h[:b] = 2; it.next()`. | Raises `ConcurrentModificationError`. | `D-122` |
| `IRIS-V1-COLLECTIONS-V282` | positive | compiler required; interpreter required; JIT required; native not applicable | `h = %{ :a: 1, :b: 2 }; it = h.iterator(); first = it.next().value; h[first[0]] = 9; remaining_key = first[0] == :a ? :b : :a; h[remaining_key] = 7; second = it.next().value; [first, second]`. | `first` remains the Tuple `(first[0], 1 or 2)` captured before its replacement; `second == (remaining_key, 7)`. Both values have Type `Tuple<Symbol,Integer>`. | `D-123` |
| `IRIS-V1-COLLECTIONS-V283` | positive | compiler required; interpreter required; JIT required; native not applicable | `h = %{ :a: 1, :b: 2 }; other = h.iterator(); removed = nil; h.each_with_iterator() { |key, value, it| removed = (key, value, it.remove_current()); break }; owner_next = h.iterator().next(); other.next()`. | `removed[2] == nil`, `h.size == 1`, and the owning traversal remains valid; the final `other.next()` raises `ConcurrentModificationError`. | `D-124` |
| `IRIS-V1-COLLECTIONS-V284` | positive | compiler required; interpreter required; JIT required; native not applicable | `h = %{ :a: 1 }; each_args = []; exposed = []; nested = []; h.each() { |key, value| each_args << (key, value) }; h.each_with_iterator() { |key, value, it| exposed << it; h.each_with_iterator() { |k2, v2, it2| nested << it2 } }`. | `each_args == [(:a,1)]`; each block receives exactly two then three arguments; `exposed[0].same?(nested[0]) == false`; no iterator is available in the two-argument `each` block. | `D-125` |
| `IRIS-V1-COLLECTIONS-V285` | negative | compiler required; interpreter required; JIT required; native not applicable | Three isolated programs: `(1)` `it = %{ :a: 1 }.iterator(); it.remove_current()`; `(2)` advance a one-entry iterator through `Iteration.done`, then `it.remove_current()`; `(3)` `it = %{ :a: 1 }.iterator(); it.close(); it.remove_current()`. | Each program raises `IteratorStateError` at `remove_current()` with no Hash mutation. | `D-126` |
| `IRIS-V1-COLLECTIONS-V286` | positive | compiler required; interpreter required; JIT optional; native not applicable | `for value in YieldNil.new() { seen << value }`, where `next()` yields `Iteration.yield(nil)` then done. | `seen == [nil]`; completion is not an exception. | `D-127` |
| `IRIS-V1-COLLECTIONS-V287` | negative | compiler required; interpreter required; JIT required; native not applicable | `a = Iteration.yield("x"); b = Iteration.yield("x"); [a == b, a.same?(b)]`. | Equality is `true`; `same?` raises `IdentityError`; `Iteration.done.same?(Iteration.done)` is `true`. | `D-128` |
| `IRIS-V1-COLLECTIONS-V288` | negative | compiler required; interpreter required; JIT required; native not applicable | `y = Iteration.yield(4); d = Iteration.done; [y.yield?, y.done?, y.value, d.yield?, d.done?]; d.value`. | First result is `[true, false, 4, false, true]`; final access raises `IteratorStateError`. | `D-129` |
| `IRIS-V1-COLLECTIONS-V289` | negative | compiler required; interpreter required; JIT required; native not applicable | `%{ Iteration.done: :d, Iteration.yield(1): :y }; %{ Iteration.yield(m"x"): :bad }`. | First Hash succeeds; second insertion raises `InvalidKeyError`. | `D-130` |
| `IRIS-V1-COLLECTIONS-V290` | positive | compiler required; interpreter required; JIT optional; native not applicable | `Iteration.done.hash; Iteration.yield(nil).hash`. | Exact Integers `5251923768640082620`, `13805951094675440708`; canonical inputs `00`, `01d37c681abf4074a4`. | `D-131` |
| `IRIS-V1-COLLECTIONS-V291` | positive | compiler required; interpreter required; JIT optional; native not applicable | `[Iteration.done <=> Iteration.done, Iteration.done <=> Iteration.yield(1), Iteration.yield(1) <=> Iteration.yield(2)]`. | Result `[0, nil, -1]`. | `D-132` |
| `IRIS-V1-COLLECTIONS-V292` | negative | compiler required; interpreter required; JIT required; native not applicable | `Iteration.yield(BadCompare.new()) <=> Iteration.yield(BadCompare.new())`, where `<=>` returns `2`. | Raises `ComparisonContractError`. | `D-133` |
| `IRIS-V1-COLLECTIONS-V293` | positive | compiler required; interpreter required; JIT optional; native not applicable | `a = [1].iterator(); b = [1].iterator(); a.next(); [a == b, a.same?(b)]`. | Result `[false, false]`. | `D-134` |
| `IRIS-V1-COLLECTIONS-V294` | positive | compiler required; interpreter required; JIT optional; native not applicable | `it = [1].iterator(); h = %{ it: :cursor }; before = it.hash; it.next(); [h.fetch(it), before == it.hash]`. | Result `[:cursor, true]`. | `D-135` |
| `IRIS-V1-COLLECTIONS-V295` | positive | interpreter required; JIT optional; native required | Host fixture `WeakProbe.array([1])`; create iterator, drop source, force collection, inspect `probe.finalized?`. | `false` before exhaustion or `close()`. | `D-136` |
| `IRIS-V1-COLLECTIONS-V296` | positive | interpreter required; JIT optional; native required | Exhaust the V295 iterator, force collection, then call `next()` twice. | Probe is collectible; both results are the same `Iteration.done` singleton. | `D-137` |
| `IRIS-V1-COLLECTIONS-V297` | positive | interpreter required; JIT optional; native required | Save `it.hash`; call `it.close()` twice; force collection; call `it.next()`. | Both closes return `nil`; probe is collectible; next is done; hash is unchanged. | `D-138` |
| `IRIS-V1-COLLECTIONS-V298` | positive | compiler required; interpreter required; JIT optional; native not applicable | `[:name, :"a b", :"\\n"]`. | Contents are `name`, `a b`, and one newline scalar. | `D-337` |
| `IRIS-V1-COLLECTIONS-V299` | negative | compiler required; interpreter required; JIT required; native not applicable | `a = :name; b = :name; [a == b, a.same?(b)]`. | Equality is `true`; `same?` raises `IdentityError`. | `D-338` |
| `IRIS-V1-COLLECTIONS-V300` | positive | compiler required; interpreter required; JIT optional; native not applicable | `:name.hash`. | `10999003376002830561`, using context `Iris Language v1 stable symbol hash` and bytes `046e616d65`. | `D-339` |
| `IRIS-V1-COLLECTIONS-V301` | positive | compiler required; interpreter required; JIT optional; native not applicable | Identifier `e\u{301}`, simple Symbol `:é`, and quoted Symbol `:"e\u{301}"`. | First two are NFC `é`; quoted form remains `e` plus U+0301 and is unequal. | `D-340` |
| `IRIS-V1-COLLECTIONS-V302` | diagnostic | parser required; interpreter not applicable; JIT not applicable; native not applicable | `let \u{03b1} = 1`; `let \u{200b}x = 1`. | Alpha is accepted with Unicode 17.0.0 XID; U+200B reports `LEX_INVALID_IDENTIFIER`. | `D-341` |
| `IRIS-V1-COLLECTIONS-V303` | positive | compiler required; interpreter required; JIT optional; native not applicable | `"\xFF".to_bytes()`. | Hex bytes are `c3bf`, proving U+00FF rather than byte `ff`. | `D-370` |
| `IRIS-V1-COLLECTIONS-V304` | positive | compiler required; interpreter required; JIT optional; native not applicable | `s = "a\u{1f600}b"; [s.length, s[1], s.byte_length]`. | `[3, "😀", 6]`; index Type is `String`. | `D-371` |
| `IRIS-V1-COLLECTIONS-V305` | negative | compiler required; interpreter required; JIT required; native not applicable | Two isolated sources: `a = "x"; b = "x"; a.same?(b)` and `a = "x"; a[0] = "y"`. | First raises `IdentityError`; second reports missing standard selector `[]=` during static validation; neither mutates a String. | `D-372` |
| `IRIS-V1-COLLECTIONS-V306` | positive | compiler required; interpreter required; JIT optional; native not applicable | `["é" == "e\u{301}", "A" == "a"]`. | `[false, false]`. | `D-373` |
| `IRIS-V1-COLLECTIONS-V307` | positive | compiler required; interpreter required; JIT optional; native not applicable | `"Iris".hash`. | `2999030340536694829`, canonical bytes `0449726973`, context `Iris Language v1 stable string hash`. | `D-374` |
| `IRIS-V1-COLLECTIONS-V308` | negative | compiler required; interpreter required; JIT required; native not applicable | `"${BadText.new()}"`, where `to_string()` returns `1`. | Raises `TypeContractError` after one `to_string()` call. | `D-375` |
| `IRIS-V1-COLLECTIONS-V309` | positive | compiler required; interpreter required; JIT optional; native not applicable | `Widget.new().to_string()`. | Exact result `"<app::Widget>"`; no address, revision, ivar, or property text. | `D-376` |
| `IRIS-V1-COLLECTIONS-V310` | positive | compiler required; interpreter required; JIT optional; native not applicable | `"${1}".inspect()`. | Reparsable literal `"\\\"1\\\""`; parsing it yields scalar-equal `"1"` without interpolation. | `D-377` |
| `IRIS-V1-COLLECTIONS-V311` | positive | compiler required; interpreter required; JIT optional; native not applicable | `["x".to_string(), "x".inspect()]`. | `[`"x"`, `"\"x\""`]`; display and inspect differ. | `D-378` |
| `IRIS-V1-COLLECTIONS-V312` | diagnostic | parser required; interpreter not applicable; JIT not applicable; native not applicable | `"${1 + 2}"; "${1:hex}"`. | First String is `"3"`; second source reports `PARSE_BAD_INTERPOLATION`. | `D-383` |
| `IRIS-V1-COLLECTIONS-V313` | negative | compiler required; interpreter required; JIT required; native not applicable | `"${log << 1}${Raise.new()}${log << 3}"`. | `log == [1]`; raises the sentinel exception; no String result is published. | `D-384` |
| `IRIS-V1-COLLECTIONS-V314` | positive | compiler required; interpreter required; JIT optional; native not applicable | `"a" + Textable.new(); "a" "b"`. | Results are `"ax"` and `"ab"`; `Textable#to_string()` runs once. | `D-385` |
| `IRIS-V1-COLLECTIONS-V315` | positive | parser required; interpreter not applicable; JIT not applicable; native not applicable | Source `"a"\n"b"` and `("a"\n"b")`. | First is two statements; grouped form is one String `"ab"`. | `D-387` |
| `IRIS-V1-COLLECTIONS-V316` | positive | compiler required; interpreter required; JIT optional; native not applicable | `s = "abc"; [s[-1], s[3]]`. | `[`"c"`, `nil`]`. | `D-389` |
| `IRIS-V1-COLLECTIONS-V317` | positive | compiler required; interpreter required; JIT optional; native not applicable | `s = "abc"; [s[-9 ..< 2], s[2 ..< 1]]`. | `[`"ab"`, `""]`; both values have Type `String`. | `D-390` |
| `IRIS-V1-COLLECTIONS-V318` | negative | compiler required; interpreter required; JIT required; native not applicable | `"abc"[(1 ..= 3).by(step: 2)]`. | Raises `ArgumentError`. | `D-391` |
| `IRIS-V1-COLLECTIONS-V319` | diagnostic | compiler required; interpreter required; JIT optional; native not applicable | `"abc"[0] = "x"`. | Missing standard `[]=` diagnostic. | `D-392` |
| `IRIS-V1-COLLECTIONS-V320` | positive | compiler required; interpreter required; JIT optional; native not applicable | `m = m"ab"; snap = m.to_string(); m << "c"; [snap, m.to_string()]`. | `[`"ab"`, `"abc"`]`. | `D-393` |
| `IRIS-V1-COLLECTIONS-V321` | negative | compiler required; interpreter required; JIT required; native not applicable | `m = m"ab"; m[1] = "x"; m[9] = "z"`. | First mutation makes `"ax"`; second raises `IndexError`. | `D-394` |
| `IRIS-V1-COLLECTIONS-V322` | positive | compiler required; interpreter required; JIT optional; native not applicable | `a = m"x"; b = m"x"; a.same?(b)`. | `false`; both have equal initial content. | `D-395` |
| `IRIS-V1-COLLECTIONS-V323` | negative | compiler required; interpreter required; JIT required; native not applicable | `m = m"x"; [m == "x", m.same?(m), m.hash]`. | First two values are `true`; hash raises `InvalidKeyError`. | `D-396` |
| `IRIS-V1-COLLECTIONS-V324` | negative | compiler required; interpreter required; JIT required; native not applicable | `m = m"x"; [m == "x", m.hash]`. | Equality is `true`; hash raises `InvalidKeyError`. | `D-397` |
| `IRIS-V1-COLLECTIONS-V325` | positive | compiler required; interpreter required; JIT optional; native not applicable | `a = m"a"; b = a + "b"; a << "c"; [a.to_string(), b.to_string()]`. | `[`"ac"`, `"ab"`]`. | `D-398` |
| `IRIS-V1-COLLECTIONS-V326` | positive | compiler required; interpreter required; JIT optional; native not applicable | `a = m"a"; alias = a; a += "b"; [alias.to_string(), a.to_string()]`. | `[`"a"`, `"ab"`]`. | `D-399` |
| `IRIS-V1-COLLECTIONS-V327` | negative | compiler required; interpreter required; JIT required; native not applicable | `m = m"a"; m.append(BadText.new())`, where conversion raises. | Propagates error and `m.to_string() == "a"`. | `D-400` |
| `IRIS-V1-COLLECTIONS-V328` | positive | compiler required; interpreter required; JIT optional; native not applicable | `m = m"ab"; [m.clear().same?(m), m.replace("x").to_string()]`. | `[true, "x"]`. | `D-401` |
| `IRIS-V1-COLLECTIONS-V329` | positive | compiler required; interpreter required; JIT optional; native not applicable | `m = m"a"; copy = m.upcase(); bang = m.upcase!(); [copy, bang.same?(m)]`. | `copy` is fresh `m"A"`; second result is `true`. | `D-402` |
| `IRIS-V1-COLLECTIONS-V330` | differential | interpreter required; JIT required; native not applicable | `m = m"x"; gate = Barrier.new(3); results = Concurrent.collect([ { gate.wait(); m.append("é") }, { gate.wait(); m.append("😀") } ]); gate.wait(); Concurrent.join_all(); snapshot = m.to_string(); [results, snapshot, snapshot.to_bytes()]`. | Each backend observes either one `ConcurrentMutationError` with snapshot exactly `"xé"` or `"x😀"`, or two successes with snapshot exactly `"xé😀"` or `"x😀é"`; Bytes are respectively `78c3a9`, `78f09f9880`, `78c3a9f09f9880`, or `78f09f9880c3a9`, never invalid or partial UTF-8. | `D-403` |
| `IRIS-V1-COLLECTIONS-V331` | negative | compiler required; interpreter required; JIT required; native not applicable | `m = m"ab"; it = m.iterator(); m.append("c"); it.next()`. | Raises `ConcurrentModificationError`. | `D-404` |
| `IRIS-V1-COLLECTIONS-V332` | positive | compiler required; interpreter required; JIT optional; native not applicable | `"a\u{1f600}".to_array()`. | `["a", "😀"]`, with immutable one-scalar Strings. | `D-405` |
| `IRIS-V1-COLLECTIONS-V333` | negative | compiler required; interpreter required; JIT required; native not applicable | `m = m"é"; view = m.bytes(); m.append("x"); view.iterator().next()`. | `view` is Iterable, not Array; advance raises `ConcurrentModificationError`. | `D-406` |
| `IRIS-V1-COLLECTIONS-V334` | differential | compiler required; interpreter required; JIT required; native not applicable | `let xid_17 = \u{1C89}; [Unicode.version(), "\u{00DF}".casefold(), "A" =~ /\p{Lu}/, "a\u{0308}".graphemes().to_array(), xid_17]`, run under host locales `C`, `tr-TR`, and one host Unicode implementation older than 17.0.0. | Every run returns String `"17.0.0"`, String `"ss"`, a non-`nil` Match, Array `['a\u{0308}']`, and accepts U+1C89 as an identifier under Unicode 17.0.0 XID tables; locale and host Unicode data do not alter any result. | `D-407` |
| `IRIS-V1-COLLECTIONS-V335` | positive | compiler required; interpreter required; JIT optional; native not applicable | `a = b"x"; b = mb"x"; [a.same?(a), b.same?(b)]`. | Bytes identity access raises `IdentityError`; ByteArray identity access is `true`. | `D-408` |
| `IRIS-V1-COLLECTIONS-V336` | negative | compiler required; interpreter required; JIT required; native not applicable | `a = b"\x00"; b = mb"\x00"; [a == b, b.hash]`. | Equality is `true`; ByteArray hash raises `InvalidKeyError`. | `D-409` |
| `IRIS-V1-COLLECTIONS-V337` | positive | compiler required; interpreter required; JIT optional; native not applicable | `Bytes[0xff, 0x00, 0x01, 0x02].hash`. | `17410268034348844959`, canonical bytes `04ff000102`, context `Iris Language v1 stable bytes hash`. | `D-410` |
| `IRIS-V1-COLLECTIONS-V338` | positive | compiler required; interpreter required; JIT optional; native not applicable | `a = mb"abc"; b = a[0 ..< 2]; a[0 ..< 2] = b"ZZ"; [b.to_bytes(), a.to_bytes()]`. | Results are `b"ab"` and `b"ZZc"`; `b` is an independent ByteArray snapshot. | `D-411` |
| `IRIS-V1-COLLECTIONS-V339` | negative | compiler required; interpreter required; JIT required; native not applicable | `a = b"a"; b = mb"b"; [a + b, b + a]; a + "x"`. | Results have Types Bytes and ByteArray; final operation raises `TypeContractError`. | `D-412` |
| `IRIS-V1-COLLECTIONS-V340` | negative | compiler required; interpreter required; JIT required; native not applicable | `["é".to_bytes(), Bytes[0xc3, 0x28].to_string()]`. | First value is hex `c3a9`; second operation raises `EncodingError`. | `D-413` |
| `IRIS-V1-COLLECTIONS-V341` | negative | compiler required; interpreter required; JIT required; native not applicable | `Encoding::UTF_16LE.decode(Bytes[0x41, 0x00]); Encoding.default()`. | Decode is `"A"`; implicit default selection raises `EncodingSelectionError`. | `D-414` |
| `IRIS-V1-COLLECTIONS-V342` | positive | compiler required; interpreter required; JIT optional; native not applicable | `for value in ProbeIterable.new() { seen << value }`. | Trace is `[:iterator, :next, :next_done]`; `seen` has every yielded value. | `D-439` |
| `IRIS-V1-COLLECTIONS-V343` | negative | compiler required; interpreter required; JIT required; native not applicable | Three isolated observations over `let a: Array<Integer> = [1]`: `a[1]`, `a.hash`, and `a[1] = 2`. | Read is `nil`; hash raises `InvalidKeyError`; write raises `IndexError`; Array remains `[1]`. | `D-459` |
| `IRIS-V1-COLLECTIONS-V344` | negative | compiler required; interpreter required; JIT required; native not applicable | `a = [1, 2]; b = a[0 ..< 1]; b[0] = 9; [a, b]; a.hash`. | Values are `[[1,2],[9]]`; final hash raises `InvalidKeyError`. | `D-460` |
| `IRIS-V1-COLLECTIONS-V345` | diagnostic | parser required; interpreter not applicable; JIT not applicable; native not applicable | `%{ 1 + 2: :v }; { :k => :v }; %{} `. | First key is Integer `3`; legacy form gets migration diagnostic; untyped empty Hash is rejected. | `D-461` |
| `IRIS-V1-COLLECTIONS-V346` | negative | compiler required; interpreter required; JIT required; native not applicable | `h = %{ :a: 1 }; [h[:x], h.delete(:x)]; h.fetch(:x)`. | First pair `[nil, nil]`; fetch raises `KeyError`; Hash remains `%{:a:1}`. | `D-462` |
| `IRIS-V1-COLLECTIONS-V347` | positive | compiler required; interpreter required; JIT optional; native not applicable | `[(1 ..= 3).to_array(), (1 ..< 3).to_array()]`. | `[[1, 2, 3], [1, 2]]`. | `D-463` |
| `IRIS-V1-COLLECTIONS-V348` | negative | compiler required; interpreter required; JIT required; native not applicable | `[(3 ..= 1).to_array(), (1 ..= 5).by(step: 2).to_array(), (1 ..= 3).by(step: 0)]`. | First two values are `[[3, 2, 1], [1, 3, 5]]`; zero step raises `RangeError`. | `D-464` |
| `IRIS-V1-COLLECTIONS-V349` | positive | compiler required; interpreter required; JIT optional; native not applicable | `t = (1, "x"); [t[0], t[1], t.same?(t)]`. | `[1, "x"]`; `same?` raises `IdentityError`. | `D-465` |
| `IRIS-V1-COLLECTIONS-V350` | positive | compiler required; interpreter required; JIT required; native not applicable | Source declares `contract Numbers for Iterable<Integer> { fun iterator() -> Iterator<Integer> }`; reflection evaluates `[Numbers.requirement(:iterator).return_type, Iterator<Integer>.requirement(:next).return_type, Iterator<Integer>.requirement(:close).return_type]`. | Compilation succeeds and the exact normalized Types are `[Iterator<Integer>, Iteration<Integer>, Nil]`. | `D-466` |
| `IRIS-V1-COLLECTIONS-V351` | positive | compiler required; interpreter required; JIT optional; native not applicable | `[ (1, 2).to_array(), [1, 2].to_array(), b"\x01\x02".to_array() ]`. | Element shapes are `[1, 2]`, `[1, 2]`, and `[1, 2]` where byte values have Type `Integer`. | `D-467` |

## Direct Regex Conformance Vectors

IRIS-V1-COLLECTIONS-C098: The following vectors are normative direct coverage for core Regex construction, matching, Match observations, diagnostics, snapshots, and stable hashing.

IRIS-V1-COLLECTIONS-C099: Multi-decision mappings in the direct Regex tables are permitted only because each named observation is present in the same concrete scenario. V042 observes Unicode 17.0.0 matching, core Regex literal and match-operator behavior, and separation from the advanced Regex package. V352 observes the core literal/operator surface and safe supported constructs. V353 observes core diagnostics for constructs and flags excluded by the safe subset. V354 observes Unicode 17.0.0 properties, core matching, and a supported safe-subset Match snapshot. V355 observes core Regex canonical equality together with the safe-subset canonical flag and hash rules.

| Vector ID | Category | Applicability | Source/Input | Expected observable | Decisions |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-COLLECTIONS-V352` | positive | compiler required; interpreter required; JIT required; native not applicable | `term = "a+b"; r = /^${term}(?<digits>[0-9]+)$/mi; m = "A+B42" =~ r; [r, m.full, m.scalar_range, m.byte_range, m[1], m[:digits], "none" !~ r]`. | `r` has Type `Regex`, canonical pattern `^a\+b(?<digits>[0-9]+)$`, and flags `im`; `m` is an immutable `Match`; results are full String `"A+B42"`, scalar Range `0 ..< 5`, byte Range `0 ..< 5`, capture String `"42"`, named capture String `"42"`, and Bool `true`. | `D-505`, `D-506` |
| `IRIS-V1-COLLECTIONS-V353` | negative | compiler required; interpreter not applicable; JIT not applicable; native not applicable | Three isolated Regex sources: `/(a)\1/`, `/(?<=a)b/`, and `/a/ii`. | First reports `REGEX_UNSUPPORTED_BACKREFERENCE`, second reports `REGEX_UNSUPPORTED_LOOKBEHIND`, and third reports `LEX_BAD_REGEX_FLAGS`; each has severity `error`, phase `regex-compile` or earlier lexical phase for duplicate flags, and publishes no Regex value. | `D-505`, `D-506` |
| `IRIS-V1-COLLECTIONS-V354` | positive | compiler required; interpreter required; JIT required; native not applicable | `text = m"ab12"; match = text =~ /(?<letters>\p{L}+)(?<digits>\p{Nd}+)/; text.replace("zz"); [match.full, match[:letters], match[:digits], match.scalar_range, match.byte_range]`. | Exact result `["ab12", "ab", "12", 0 ..< 4, 0 ..< 4]`; later MutableString mutation does not change the Match snapshot. | `D-407`, `D-505`, `D-506` |
| `IRIS-V1-COLLECTIONS-V355` | positive | compiler required; interpreter required; JIT required; native not applicable | `[/a+/im == /a+/mi, /a+/im.hash, /a+/mi.hash]`. | Exact result `[true, 1732507240534066756, 1732507240534066756]`; each hash uses canonical input hex `02612b02696d`, context `Iris Language v1 stable regex hash`, and digest bytes `4446d3f2271a0b18`. | `D-505`, `D-506` |

## Direct Text Continuation Vector

IRIS-V1-COLLECTIONS-C100: The following vector is normative direct coverage for the text-continuation decision assigned to this chapter by the trace audit.

| Vector ID | Category | Applicability | Source/Input | Expected observable | Decisions |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-COLLECTIONS-V356` | diagnostic | compiler required; interpreter required; JIT not applicable; native not applicable | Three isolated UTF-8 source fixtures: hex `2261225c0a226222` (`"a"` followed immediately by backslash, LF, then `"b"`); hex `2261225c200a226222` (space between backslash and LF); and source `"a" \ // comment` followed by LF and `"b"`. | The first fixture evaluates to String `"ab"`. The second and third report `LEX_BAD_CONTINUATION`, severity `error`, phase `lex`, and publish no String value. | `D-388` |
