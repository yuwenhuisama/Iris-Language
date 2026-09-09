# iris-builtins

Dependency-free, `no_std`, allocation-free implementation-evidence catalog for
Iris tooling. There is no build script, filesystem discovery, runtime linkage,
runtime initialization, process launch, or evaluator call in this library.
`members()` returns one immutable compile-time-flattened static slice.

The frozen public API is `BuiltinType`, `Surface`, `ParameterKind`, `Parameter`,
`Availability`, `ReturnFact`, `CallShape`, `BuiltinMember`, `members()`,
`class_names()`, and `service_names()`. No additional fields were required.
`BuiltinType::name()` and `from_name()` map exact family names; Rust's `Text`
payload is Iris `String`, not a second family. `FfiLibrary` names `FFI::Library`.
`Type` and `ComposedType` distinguish nominal and composed metadata behavior;
`ExternalResource` is a payload category whose actual nominal name is supplied
by an extension. These family labels do not establish source Class bindings.

## Consumer Contract

- Key by owner, receiver family, surface, and selector. Never search unrelated
  owners to resolve an unknown receiver or infer universal Object inheritance.
- Prefer authored declarations. Catalog return facts are observed results, not
  declared annotations: omitted annotations remain `Dynamic<Object>`.
- `Known` means the fixed family on success, not guaranteed success. `Receiver`
  means preserved receiver identity/family; FFI bind retains identity while
  returning updated binding metadata. `Unknown` also covers unions, optional
  results, element/callback results, and backend-divergent results. Unknown
  return labels are absent; descriptive facts remain in documentation.
- Each shape is an independently supported useful argument layout. Optional
  keywords have separate absent/present shapes; their present slot is required
  within that shape. Ignored arbitrary tails are not advertised as rest slots.
  Only `print` has an evidenced zero-or-more rest surface. `using` has explicit
  Closure-argument and trailing-block alternatives.
- `arg1`, `arg2`, `arg3`, and `callback` are display-only positional placeholders,
  never inferred keyword names. Parameter type labels occur only where checked
  or otherwise directly evidenced. No defaults are invented.
- Empty `shapes` on Closure/BoundMethod `call` or generic Class `new` means the
  signature belongs to the particular callable/initializer and is unknown,
  **not** that the call accepts zero arguments. Their documentation records
  both-backend availability. A property has a zero-parameter shape carrying
  read availability, not permission to add parentheses.
- `Surface::Class` with receiver `Class` is generic Class metadata. Float
  class-side members instead carry the float family. Float `nan`/`infinity`
  properties are Class-side only, despite that family tag. Module Class rows
  describe declared module source names, not general Symbol instances.
- `Surface::Service` uses `receiver: None` and a qualified source-route owner.
  Service properties such as `Iteration.done` also have no receiver. Globals
  use empty owner and no receiver. Services are not first-class Class objects.
- `class_names()` lists only Object, Nil, Bool, Integer, Float32, Float64,
  String, as verified in `iris-eval/src/source_method.rs:208` and
  `iris-runtime/src/kernel.rs:245`. It implies neither `.new` conversion nor
  Array/Hash/Bytes/ByteArray/MutableString/Tuple/Regex/Task constructors.
  `service_names()` lists successful service routes, not refusal-only roots.
  The special `Transformation` name denotes an empty value, not a Class or
  service; its value members are cataloged under that family.

## Evidence And Backend Differences

The complete input inventory was read from the sibling tools checkout's
`analysis/BUILTIN-INVENTORY.md` (2026-09-08). Rows were checked against the
cited evaluator dispatch, kernel installation/invocation, VM compiler call
lowering, authored sends, member reads, and low-level operations. Every
descriptor records repository-relative runtime evidence; line anchors are
audit-time locations, not durable semantic IDs. The catalog does not read
the sibling document or source files at build/runtime.

The tables retain reference-only Array Range slicing, String upcase, ByteArray
append and explicit iterator, generator sends, reflective package adapters,
and class metadata calls whose VM implementation only supports property reads.
Hash each returns nil in reference but receiver Hash in VM. Integer mul_add
returns Float32 with all Integer inputs. MutableString normalization returns
String. Bytes/MutableString cursors differ in concrete kind across backends.
SourceLocation.path is Symbol in reference and String in VM property reads.

Additional dispatcher verification found that VM `MutableString.bytes()`
succeeds but `.bytes` bypasses authored dispatch (`execute.rs:1392`) and fails;
the property descriptor therefore says Reference. VM primitive identity also
lacks same-family ByteArray and Library cases (`operations.rs:212`), so those
same? shapes are Reference. `Package.validate` is an adapter: VM inspects
literal true flags at compile time while reference tests evaluated Bools.
Reference HashIterator.close currently returns nil without releasing its
cursor, unlike VM. These are documented differences, not runtime fixes.
Module source-name calls also intercept ordinary lookup in reference:
`M.respond_to?` is not offered and direct `M.same?(M)` is VM-only (the
reference primitive still supports module values). Verified `to_bool` fallback
is expanded only across payload routes that reach the installed root slot;
Class, Module, ContractView and external-resource interception is not guessed.

## Non-Callable Syntax Inventory

These routes are intentionally absent from `members()`; do not use their
existence to offer dot-call completion or signature help:

| Syntax | Families | Implemented result / qualification |
| --- | --- | --- |
| `[index]` | Array, Hash, Tuple, ReadonlyArray | Element/value or nil; Integer sequence index, arbitrary Hash key |
| `[range]` | Array | Fresh Array slice in both backends |
| `[range]` | ReadonlyArray | Mutable Array copy, reference only |
| `[index]`, `[range]` | String | Scalar String or nil / String slice |
| `[index]`, `[range]` | MutableString | VM scalar String or nil / String slice; reference enters byte path and refuses |
| `[index]`, `[range]` | Bytes, ByteArray | Integer byte or nil / same-family slice |
| `[index] = value` | Array, Hash | Source expression returns RHS; no Array Range write |
| `[index/range] = value` | MutableString | Text replacement; scalar replacement exactly one scalar; helper nil, reference source expression RHS |
| `[index/range] = value` | ByteArray | Byte 0..255 or binary range replacement; helper nil, reference source expression RHS |
| `+` | String, MutableString, Bytes, ByteArray | VM operator route only; reference selector sends are separately listed |
| `<<` | MutableString | VM operator-only append; reference selector send separately listed |
| `<=>`, `<`, `<=`, `>`, `>=` | Ordinary Object | VM default comparison operator routes; reference selector sends listed |
| `<=>` | Iteration | VM operator route; reference selector send listed |
| unary `-` | Numeric | Lowers to cataloged negate, not a zero-argument binary minus method |
| `Float64(-Infinity)` | Special source spelling | Negative infinity; no bare Infinity binding |

Index evidence: `iris-eval/src/source_runtime.rs:7383,7441,7565` and
`iris-vm/src/machine/operations.rs:295,403`; operator evidence:
`iris-vm/src/machine/stdlib.rs:13,144,192,217,2287`. Assignment expression facts
are not formal setter return annotations.

## Exclusions

- Internal probes/fixtures: Array.share_count, NativeFixture and its
  NativeResource/release counters, RevisionHistory.prune, HostABI/C ABI exports,
  and extension-defined functions without their own metadata.
- Refusal-only: File.read_text, Encoding.default, FFI::Library.call,
  Reflection::Class.reactivate, readonly mutations and context setters.
- Class.rollback is only a partial artifact validator, not working rollback
  publication, and has no honest fixed callable contract.
- No successful hash catalog for Array, Hash, MutableString, ByteArray, Match,
  ReadonlyArray; no Array/Hash/Match public equality inferred from Rust values.
- Method.call and bare Closure application are refused. Task wait/join/result/
  then/cancel/await methods are absent; await is syntax. Closeable.close is a
  contract obligation, not an installed method on every receiver.
- No spec-only Array size/+/sort!/reverse!, zero-argument join/all?/any?, Hash
  clear or default/block fetch, ByteArray clear/replace, String iterator,
  normalize/normalize!/casefold!, blanket readonly/tuple Array conveniences,
  Range step/length/construction, direct Contract.requirement, unlisted
  reflection signatures, Package.load/reload/upgrade, Plan protocols, or bare
  type_of/type_and_value/same?/puts/len/Float32 helpers.
- serialize/deserialize are declared Serializable user hooks, not installed
  builtins; diagnostic exception names do not imply constructors or methods.

## Verification

```sh
cargo test -p iris-builtins
cargo clippy -p iris-builtins --all-targets -- -D warnings
cargo fmt -p iris-builtins -- --check
cargo run -p iris-builtins --example catalog
IRIS_BUILTINS_CLI="$PWD/target/debug/iris" cargo test -p iris-builtins --test runtime_parity -- --ignored
```

Invariant tests were first run against empty descriptor/name slices and failed
for missing catalog data. Independent explicit selector tables cover every
receiver family and service, with property/class checks, refusal exclusions,
keyword alternatives and return/availability regressions. Opt-in CLI tests
use an existing executable, never add runtime dependencies to this crate and
never build/start the runtime for catalog discovery. Large catalog files are
pure static data tables; executable catalog assembly remains a small const
flattening routine.
