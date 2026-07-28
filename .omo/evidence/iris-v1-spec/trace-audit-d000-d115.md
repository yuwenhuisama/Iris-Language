# Trace Audit D-000 Through D-115

Scope: independent audit of D-000 through D-115 against `.omo/drafts/iris-language-specification.md`, current `spec/iris-v1` chapters, `traceability-matrix.md`, and actual vector bodies.

Audit rules:

- Reject broad cluster vectors as exact coverage when they only cite a range or suite. Rejected for exact per-row coverage: `IRIS-V1-RUNTIME-V049`, `IRIS-V1-GRAMMAR-V008`, `IRIS-V1-GRAMMAR-V009`, `IRIS-V1-COLLECTIONS-V043`.
- Static syntax is a valid positive or diagnostic observable.
- Use `metadata-only` only for directory or consolidation facts with no direct parser, runtime, diagnostic, Host, or corpus observable.
- Known exact fixes: D-020 is covered by `IRIS-V1-RUNTIME-V046`; D-060 is covered by `IRIS-V1-RUNTIME-V047`; `IRIS-V1-RUNTIME-V048` belongs to D-061.
- Numeric and singleton public hash ownership is runtime-owned by `IRIS-V1-RUNTIME-C136` through `IRIS-V1-RUNTIME-C146` and `IRIS-V1-RUNTIME-V001` through `IRIS-V1-RUNTIME-V010` or `IRIS-V1-RUNTIME-V045`, not by collection-family suite coverage.

Counts:

| Status | Count |
| --- | ---: |
| existing-covered | 40 |
| needs-vector | 73 |
| metadata-only | 3 |
| total | 116 |

## Manifest

| D-ID | clauses | existing coverage | missing vector specification | rationale |
| --- | --- | --- | --- | --- |
| D-000 | TRACE-C013, TRACE-C014, TRACE-C002 | metadata-only | none | Directory and inventory metadata only. |
| D-001 | IDENTITY-C003, MIGRATION-C008 | IDENTITY-V001, MIGRATION-EX030 | none | Existing diagnostic checks approved source, fixed artifacts, and archaeology boundary. |
| D-002 | IDENTITY-C004, RUNTIME-C002, CONTROL-C016 | partial broad: IDENTITY-V002 | positive source using object values, custom operator dispatch, Method mutation, trailing Closure, `raise 1`, and FFI declaration; interpreter/JIT required, native optional; expect all identity surfaces accepted and no legacy restoration claim | Current vector says corpus but lacks concrete input and observable. |
| D-003 | RUNTIME-C101, RUNTIME-C135 | rejected broad: RUNTIME-V049 | positive source `((2 ** 200) + 1) - (2 ** 200)` plus Type checks; expect `Integer(1)`, visible Type `Integer`, no wrap, no SmallInt/BigInt type split | Arbitrary precision and hidden representation need exact value and Type observation. |
| D-004 | RUNTIME-C102 | metadata-only | none | Superseded by exact Float32 and Float64 decisions. |
| D-005 | RUNTIME-C102, RUNTIME-C006 | rejected broad: RUNTIME-V049 | positive source creating `1.0f32`, `1.0f64`, `1.0`; reflect Types and width-specific dispatch; expect distinct `Float32` and `Float64`, unsuffixed `Float64`, no built-in `Float16` or `Float128` | Distinct width model has no exact vector. |
| D-006 | GRAMMAR-C029, RUNTIME-C102, RUNTIME-C135 | rejected broad: GRAMMAR-V008, RUNTIME-V049 | positive parser/runtime input `1.0`, `.5`, `1.`, `1e3`, and annotated `Float32` context; expect all unsuffixed literals are `Float64` and no implicit narrowing | Existing input does not assert literal Types. |
| D-007 | GRAMMAR-C029, RUNTIME-C102, RUNTIME-C156 | rejected broad: GRAMMAR-V008, RUNTIME-V049 | positive and diagnostic input `1.0f32`, `1e10f64`, `1.0F32`; expect lower-case suffix widths, uppercase `LEX_BAD_FLOAT_SUFFIX`, no contextual conversion | Existing vector lacks f64 exponent and suffix error ownership. |
| D-008 | RUNTIME-C022, RUNTIME-C027, RUNTIME-C156 | rejected broad: RUNTIME-V049 | positive source with `left() + right()` logging and later replacing `+`; expect left once, right once, then receiver Method selected at send time, later replacement affects later send | Operator dispatch order and guard invalidation are not exact in broad vector. |
| D-009 | RUNTIME-C103, RUNTIME-C104, RUNTIME-C105, RUNTIME-C106 | metadata-only | none | Consolidated by directional arithmetic D-010 through D-012. |
| D-010 | RUNTIME-C104, RUNTIME-C135 | rejected broad: RUNTIME-V049 | positive `Integer(16777217) + Float32(0.0)` and `Integer(9007199254740993) + Float64(0.0)`; expect receiver converts to argument width, result Type is that width, precision loss observable | Integer receiver to FloatN argument has no exact vector. |
| D-011 | RUNTIME-C105, RUNTIME-C135 | rejected broad: RUNTIME-V049 | positive `Float32(0.0) + Integer(16777217)` and `Float64(0.0) + Integer(9007199254740993)`; expect argument converts to receiver width, result Type receiver width, precision loss observable | FloatN receiver to Integer argument has no exact vector. |
| D-012 | RUNTIME-C106, RUNTIME-C135 | rejected broad: RUNTIME-V049 | positive `Float32(1.5)+Float64(2.25)` and reverse plus same-width float add; expect mixed returns `Float64` both orders, same width returns same width | Mixed-width arithmetic needs concrete direction and Type observations. |
| D-013 | RUNTIME-C110, RUNTIME-C135 | rejected broad: RUNTIME-V049 | positive `1.0f64/0.0f64`, `1.0f64/-0.0f64`, `0.0f64/0.0f64`; expect signed infinities, NaN, no `DivisionByZeroError` | Float division by zero has no exact signed-zero vector. |
| D-014 | RUNTIME-C111, RUNTIME-C135 | rejected broad: RUNTIME-V049 | positive finite Float32 and Float64 overflow and oversized Integer to Float64 conversion; expect signed infinity, no overflow or conversion exception, no saturation | Overflow behavior lacks exact inputs. |
| D-015 | RUNTIME-C006, RUNTIME-C130, RUNTIME-C156 | RUNTIME-V035 | none | Existing vector directly checks NaN equality and inequality. |
| D-016 | RUNTIME-C130, RUNTIME-C135, RUNTIME-C156 | RUNTIME-V036 | none | Existing vector checks signed-zero equality and same hash. |
| D-017 | RUNTIME-C130, RUNTIME-C135, RUNTIME-C156 | partial hash only: RUNTIME-V001 through V005; rejected broad: RUNTIME-V049 | positive compare `Integer(9007199254740993)` to rounded `Float64`, exact cross-type `1`, and mixed float widths; expect exact mathematical equality, symmetry, NaN still unequal | Hash vectors do not prove equality edge behavior. |
| D-018 | RUNTIME-C131, RUNTIME-C156 | rejected broad: RUNTIME-V049 | positive all ordered comparisons with NaN on either side and `<=>`; expect ordered operators false, `<=> nil`, no exception | NaN ordering has no exact vector. |
| D-019 | RUNTIME-C121, RUNTIME-C135, RUNTIME-C156 | rejected broad: RUNTIME-V049 | negative `1 / 0`, `1 div 0`, `1 mod 0`; expect catchable `DivisionByZeroError`, no nil, wrapper, infinity, or NaN | Integer zero divisor behavior lacks concrete vector. |
| D-020 | RUNTIME-C120, RUNTIME-C135 | RUNTIME-V046 | none | Existing vector exactly checks `5 / 2` and `4 / 2` return `Float64` values. |
| D-021 | GRAMMAR-C044, RUNTIME-C026, RUNTIME-C156 | rejected broad: RUNTIME-V049 | positive class with one-argument `scale`; call `a.scale(b)` and `a scale b`; include invalid two-argument named infix attempt; expect same send and normal arity validation | Named infix equivalence has no exact vector. |
| D-022 | GRAMMAR-C044, RUNTIME-C135, RUNTIME-C156 | partial broad: GRAMMAR-V009; rejected broad: RUNTIME-V049 | positive parse/runtime `a div b + c`, `a div b div c`, `a.div(b)`; expect `a div (b + c)`, `(a div b) div c`, and same quotient Method | Current vector has one parse phrase, not full surface and associativity. |
| D-023 | RUNTIME-C121, RUNTIME-C135, RUNTIME-C156 | RUNTIME-V039 | none | Existing vector exactly checks floor division with negative operands. |
| D-024 | RUNTIME-C135, RUNTIME-C156 | rejected broad: RUNTIME-V049 | positive and diagnostic `5.mod(2)`, `5 mod 2`, `5 % 2`, `5 mod 0`; expect method and infix match, `%` rejected, zero divisor `DivisionByZeroError` | Existing modulo vector covers semantics but not surface. |
| D-025 | RUNTIME-C122, RUNTIME-C156 | RUNTIME-V040 | none | Existing vector exactly checks modulo signs. |
| D-026 | RUNTIME-C123, RUNTIME-C135, RUNTIME-C156 | rejected broad: RUNTIME-V049 | positive `2 ** 10` and `2 ** -3`; expect `Integer(1024)` then `Float64(0.125)`, no `Rational` | Exponent sign result Type lacks exact vector. |
| D-027 | RUNTIME-C124, RUNTIME-C135, RUNTIME-C156 | RUNTIME-V041 | none | Existing vector covers integer `0 ** 0` `DomainError`. |
| D-028 | RUNTIME-C124, RUNTIME-C135, RUNTIME-C156 | RUNTIME-EX005 | none | Existing example directly states `0 ** -1` yields `Float64.infinity`. |
| D-029 | RUNTIME-C125, RUNTIME-C135 | RUNTIME-V041 | none | Existing vector covers zero float exponent cases as `DomainError`. |
| D-030 | RUNTIME-C125, RUNTIME-C135 | rejected broad: RUNTIME-V049 | positive `Float32(+0.0) ** -3`, `Float64(-0.0) ** -3`, `Float64(-0.0) ** -2`; expect +inf, -inf, +inf at base width, no exception | Signed-zero parity lacks exact vector. |
| D-031 | RUNTIME-C126, RUNTIME-C135, RUNTIME-C156 | RUNTIME-V042 | none | Existing vector checks negative finite float non-integer exponent returns NaN without `DomainError`. |
| D-032 | RUNTIME-C126, RUNTIME-C135 | rejected broad: RUNTIME-V049 | positive Float32 and Float64 exponentiation with Integer, same-width float, and mixed-width float operands; expect result widths `Float32`, `Float64`, and mixed `Float64` | Width policy needs exact Type observations. |
| D-033 | GRAMMAR-C041, GRAMMAR-C042 | GRAMMAR-V003 | none | Existing vector exactly checks exponent associativity and unary binding. |
| D-034 | GRAMMAR-C016, RUNTIME-C127, RUNTIME-C135, RUNTIME-C155, MIGRATION-C008 | GRAMMAR-V002, RUNTIME-V043, MIG-012 | none | Operator inventory, bitwise runtime examples, and rotate migration row cover the decision. |
| D-035 | RUNTIME-C127, RUNTIME-C135 | RUNTIME-V043 | none | Existing vector checks `~0` and arithmetic right shift. |
| D-036 | RUNTIME-C128, RUNTIME-C135 | partial: RUNTIME-V043 covers `x << -2`; rejected broad: RUNTIME-V049 | positive `x << -2`, `x >> -2`, `x << 0`, `x >> 0`; expect reversed directions, zero identity, no `RangeError` | Existing vector covers one negative direction only. |
| D-037 | RUNTIME-C129, RUNTIME-C135 | rejected broad: RUNTIME-V049 | positive `1 >> 1000000`, `-1 >> 1000000`, `-3 >> 1000000`; expect `0`, `-1`, `-1`, no range error | Large right-shift stabilization lacks exact vector. |
| D-038 | RUNTIME-C129, RUNTIME-C135 | rejected broad: RUNTIME-V049 | negative under configured memory quota: huge left shift and huge power; expect `ResourceError` or `MemoryLimitError`, no `RangeError`, no nil, no host panic | Resource failure needs quota fixture. |
| D-039 | GRAMMAR-C024, GRAMMAR-C054 | GRAMMAR-V008 | none | Existing vector input includes binary, octal, decimal leading zero, and hexadecimal. |
| D-040 | GRAMMAR-C025, GRAMMAR-C054 | partial: GRAMMAR-V005; rejected broad: GRAMMAR-V008 | diagnostic and positive valid `1_000`, `0xFF_FF`, `1.234_567`, `1e1_000`, `1.0e+1_024`; invalid `_1`, `1_`, `1__0`, `0x_FF`, `1_.0`, `1._0`, `1e_3`, `1e+_3`, `1_f32`; expect values or `LEX_BAD_NUMERIC_SEPARATOR` | Current vectors cover only some malformed separators. |
| D-041 | GRAMMAR-C024, GRAMMAR-C054 | GRAMMAR-V008 | none | Existing vector includes `00755` under decimal interpretation. |
| D-042 | GRAMMAR-C048A, GRAMMAR-C054 | partial: GRAMMAR-V005 uppercase suffix; rejected broad: GRAMMAR-V008 | positive and diagnostic `0B1010`, `0O755`, `0Xff`, `1E3`, `1.0F32`, `1.0F64`; expect uppercase prefixes/digits/exponent accepted, uppercase suffix `LEX_BAD_FLOAT_SUFFIX` | Prefix and exponent case positives are missing. |
| D-043 | GRAMMAR-C026, GRAMMAR-C054 | GRAMMAR-V005 | none | Existing malformed vector maps `0b102` to `LEX_INVALID_RADIX_DIGIT`. |
| D-044 | GRAMMAR-C048A, GRAMMAR-C054 | partial: GRAMMAR-V005 covers `0x` | diagnostic `0x`, `0X`, `0b`, `0B`, `0o`, `0O`, `0x_FF`, `0b_1010`, `0o_755`; expect one invalid numeric token and `LEX_EMPTY_RADIX_PREFIX`, never split as zero plus identifier | Current coverage is too narrow. |
| D-045 | GRAMMAR-C027, GRAMMAR-C054 | none | positive and diagnostic `.5`, `1.`, `.`, `._5`, `1._0`, `1_.`; expect first two `Float64` values, lone dot punctuation, bad forms diagnosed | Decimal-point omission has no vector. |
| D-046 | GRAMMAR-C027, GRAMMAR-C054 | partial: GRAMMAR-V005, GRAMMAR-V008 | positive and diagnostic `1e3`, `2E-4`, `1e+3f32`, `1e`, `1e+`, `1e_3`; expect valid float widths, malformed exponent diagnostics | Missing exponent diagnostics and f32 exponent suffix not covered. |
| D-047 | GRAMMAR-C028, GRAMMAR-C054 | GRAMMAR-V008 | none | Existing vector includes `0x1.fp3`. |
| D-048 | GRAMMAR-C028, GRAMMAR-C054 | partial: GRAMMAR-V008 | positive and diagnostic `0x1p0`, `0x1.p0`, `0x.8p0`, `0x1.`, `0x1e3`; expect first three hex floats, `0x1.` invalid, `0x1e3` hex Integer | Edge significands not covered. |
| D-049 | GRAMMAR-C030, GRAMMAR-C057 | rejected broad: GRAMMAR-V008 | differential midpoint decimal and hex literals for both widths with host locale and rounding mode varied; expect exact roundTiesToEven bits equal across compiler host, interpreter, JIT, target | Token acceptance does not prove correct rounding. |
| D-050 | GRAMMAR-C031, GRAMMAR-C054 | rejected broad: GRAMMAR-V008 | diagnostic finite nonzero literals rounding to infinity, subnormal, and signed zero for both widths; expect normative bits, warning for infinity and zero only | Warning and underflow policy lacks vector. |
| D-051 | RUNTIME-C116, RUNTIME-C156 | rejected broad: RUNTIME-V049 | positive and diagnostic `Float32.nan`, `Float64.infinity`, `-Float64.infinity`, bare `nan`, bare `inf`; expect class properties and no global literal tokens | Source access not exactly covered. |
| D-052 | RUNTIME-C116, RUNTIME-C156 | rejected broad: RUNTIME-V049 | positive and diagnostic read `Float64.infinity`, assign `Float64.infinity = value` before setter; expect getter message returns canonical value and assignment invalid | Get-only property surface lacks direct vector. |
| D-053 | RUNTIME-C117, RUNTIME-C156 | rejected broad: RUNTIME-V049 | positive authorized open replaces `Float64.infinity` getter; expect later read returns replacement and optimized assumptions invalidated | Dynamic special getter mutation lacks vector. |
| D-054 | RUNTIME-C117, RUNTIME-C155 | rejected broad: RUNTIME-V049 | positive authorized open adds setter to `Float64.infinity`; assignment records side effect, getter unchanged; expect no implicit backing storage | Dynamic setter addition lacks vector. |
| D-055 | RUNTIME-C107, RUNTIME-C135 | rejected broad: RUNTIME-V049 | differential operation sensitive to directed rounding with host rounding mode changed; expect Iris bits use roundTiesToEven and host mode ignored | Rounding-mode independence needs bit vector. |
| D-056 | RUNTIME-C108, RUNTIME-C135 | rejected broad: RUNTIME-V049 | differential `a * b + c` where fused differs from separate plus `mul_add`; expect expression rounds separately and `mul_add` may differ | FMA contraction prohibition lacks vector. |
| D-057 | RUNTIME-C109, RUNTIME-C135 | rejected broad: RUNTIME-V049 | differential chained Float32 calculation where excess precision changes result; expect intermediate rounded to Float32 across interpreter and JIT | Excess precision rule lacks vector. |
| D-058 | RUNTIME-C135, RUNTIME-C156 | rejected broad: RUNTIME-V049 | positive `a.mul_add(b,c)`, `a*b+c`, attempted named infix `a mul_add b`; expect one fused rounding, possible difference, no named-infix eligibility | General fused API surface lacks vector. |
| D-059 | RUNTIME-C118, RUNTIME-C135 | rejected broad: RUNTIME-V049 | positive `Float32(1).mul_add(Integer(2), Float32(3))`, mixed Float64 case, and Integer addend; expect accepted numeric set, common width, exact integer in fused expression | Operand set and width rule lack vector. |
| D-060 | RUNTIME-C119 | RUNTIME-V047 | none | Existing vector exactly checks infinity times numeric zero returns NaN at common width with no exception. |
| D-061 | RUNTIME-C119, RUNTIME-C135, RUNTIME-C156 | RUNTIME-V048 | none | Existing vector exactly checks opposite-signed infinities in `mul_add`. |
| D-062 | RUNTIME-C112, RUNTIME-C156 | rejected broad: RUNTIME-V049 | positive ordinary arithmetic with NaN payloads and invalid operation; expect quiet NaN at width, payload/sign not asserted, exact-payload ordinary arithmetic tests rejected | NaN result bit rule lacks vector. |
| D-063 | RUNTIME-C113, RUNTIME-C135, RUNTIME-C156 | partial: RUNTIME-V038 | positive from_bits/to_bits for signed zero, infinity, subnormal, quiet NaN, signaling NaN patterns in both widths; expect exact bit round trips | Existing vector only covers signaling NaN. |
| D-064 | RUNTIME-C113, RUNTIME-C135 | none | positive and negative to_bits range checks plus `from_bits(-1)`, `from_bits(2**32)`, `from_bits(2**64)`; expect nonnegative Integer ranges and `RangeError`, no truncation or modulo | Bit-container range behavior lacks vector. |
| D-065 | RUNTIME-C114, RUNTIME-C156 | RUNTIME-V038 | none | Existing vector exactly checks signaling-NaN bits round-trip. |
| D-066 | RUNTIME-C115, RUNTIME-C135 | rejected broad: RUNTIME-V049 | positive classify signaling NaN, quiet NaN, infinity, subnormal, zero, negative zero then `to_bits`; expect correct booleans, no quieting, bits unchanged | Classification purity lacks vector. |
| D-067 | RUNTIME-C135, RUNTIME-C156 | rejected broad: RUNTIME-V049 | positive compare signaling NaN with all comparison operators then `to_bits`; expect NaN comparison results, no exception, original bits unchanged | Signaling-NaN comparisons lack vector. |
| D-068 | RUNTIME-C006, RUNTIME-C156 | rejected broad: RUNTIME-V049 | positive and negative float alias, arithmetic/from_bits, and receiver-state attempt; expect original bits unchanged, new values produced, `InstanceStateError` for state | Float immutability lacks direct vector. |
| D-069 | RUNTIME-C006, RUNTIME-C160 | rejected broad: RUNTIME-V049 | positive and negative Integer alias plus arithmetic/exponent/bitwise/shift and state attempt; expect original value unchanged, new values, `InstanceStateError` | Integer immutability lacks direct vector. |
| D-070 | RUNTIME-C156, RUNTIME-C160 | partial: RUNTIME-V018 covers Integer only | negative `same?` on `Integer`, `Float32`, `Float64`; expect `IdentityError` for all identity-less numerics | Numeric no-identity coverage is incomplete. |
| D-071 | RUNTIME-C136, RUNTIME-C156 | RUNTIME-V001 through V005, RUNTIME-V036, RUNTIME-V045 | none | Runtime hash vectors show equal numeric values share hash, including signed zero. |
| D-072 | RUNTIME-C134, RUNTIME-C156 | RUNTIME-V037 | none | Existing vector checks NaN Hash key insertion raises `InvalidKeyError`. |
| D-073 | RUNTIME-C137, RUNTIME-C156 | RUNTIME-V001 through V010, RUNTIME-V045 | none | Exact public hash outputs cover stability; secret table mixing is not directly observable through `hash`. |
| D-074 | RUNTIME-C136, RUNTIME-C155, RUNTIME-C156 | RUNTIME-V001 through V010, RUNTIME-V045 | none | Existing hashes are nonnegative 64-bit Integer outputs. |
| D-075 | RUNTIME-C006, RUNTIME-C156 | RUNTIME-V001 through V010, RUNTIME-V045 | none | Exact public outputs establish same-major compatibility for numeric and singleton hashes. |
| D-076 | RUNTIME-C006, RUNTIME-C136, RUNTIME-C140, RUNTIME-C156 | RUNTIME-V001 through V007, RUNTIME-V045 | none | Canonical input hex rows cover equal numeric canonicalization, finite values, zeros, infinities. |
| D-077 | RUNTIME-C138, RUNTIME-C146 | RUNTIME-V001 through V010, RUNTIME-V045 | none | Existing rows name digest bytes `0..8` and public Integer results. |
| D-078 | RUNTIME-C138, RUNTIME-C146 | RUNTIME-V001 through V010, RUNTIME-V045 | none | Existing rows use exact numeric and singleton derive-key context strings. |
| D-079 | RUNTIME-C139, RUNTIME-C156 | RUNTIME-V001, RUNTIME-V006, RUNTIME-V007, RUNTIME-V045 | none | Canonical inputs show finite tag, positive infinity tag, and negative infinity tag. |
| D-080 | RUNTIME-C006, RUNTIME-C134, RUNTIME-C156 | partial: RUNTIME-V037 covers Hash insertion | negative direct built-in `hash` on quiet and signaling NaN of both widths; expect `InvalidKeyError`, no hash value | Direct public NaN hash is not covered by insertion only. |
| D-081 | RUNTIME-C140, RUNTIME-C156 | RUNTIME-V002 through V005, RUNTIME-V045 | none | Existing canonical inputs cover normalized finite forms. |
| D-082 | RUNTIME-C140, RUNTIME-C156 | RUNTIME-V001, RUNTIME-V045 | none | Zero vector encodes integer and signed float zeros as `0000`. |
| D-083 | RUNTIME-C141, RUNTIME-C125 | RUNTIME-V002 through V005, RUNTIME-V045 | none | Finite hash rows include length and big-endian odd magnitude bytes. |
| D-084 | RUNTIME-C142, RUNTIME-C135 | RUNTIME-V004, RUNTIME-V005, RUNTIME-V045 | none | Values `2` and `3/2` cover positive and negative exponent mappings. |
| D-085 | RUNTIME-C142, RUNTIME-C156 | RUNTIME-V004, RUNTIME-V005, RUNTIME-V045 | none | Canonical inputs cover ZigZag cases `1 -> 2` and `-1 -> 1`. |
| D-086 | RUNTIME-C139, RUNTIME-C143 | RUNTIME-V006, RUNTIME-V007, RUNTIME-V045 | none | Infinity encodings are exactly one byte. |
| D-087 | RUNTIME-C144, RUNTIME-C156 | none | negative attempt to call standard `canonical_numeric_bytes` or equivalent on numeric values; expect absent Method or reflection absence while conformance harness may inspect bytes | Non-public bytes have observable absence from standard API. |
| D-088 | RUNTIME-C131, RUNTIME-C156 | none | positive cross-type numeric ordering edge cases with large Integer, rounded Float64, Float32/Float64 exact values, infinities, reverse order; expect exact mathematical ordering and NaN unordered | Cross-type ordering lacks vector. |
| D-089 | RUNTIME-C133, RUNTIME-C155 | none | positive authorized replacement of numeric `<=>` or `==`; expect replacement affects ordinary Method, built-in rule applies only before replacement, user owns inconsistency | Dynamic numeric comparison replacement lacks vector. |
| D-090 | RUNTIME-C084, RUNTIME-C156 | none | positive object with `<=>` returning `-1`, `0`, `1`, `nil`; expect default comparisons map exactly | Default comparison derivation lacks vector. |
| D-091 | RUNTIME-C087, RUNTIME-C156 | none | positive replace `<=>`, then separately replace `==`; expect default slots follow `<=>`, replaced `==` remains independent | Independent override behavior lacks vector. |
| D-092 | RUNTIME-C084, RUNTIME-C156 | none | positive call `<=>` on numeric and ordinary objects; expect one-argument Method slot returns exact `-1`, `0`, `1`, or `nil` | Three-way comparison surface lacks vector. |
| D-093 | RUNTIME-C084, RUNTIME-C156 | none | negative object whose `<=>` returns `2`, `"bad"`, and `true`; expect `ComparisonContractError` or `TypeError` before default comparison result | Default comparison contract validation lacks vector. |
| D-094 | RUNTIME-C085, RUNTIME-C156 | none | diagnostic and runtime boundary Method `<=>` declared or returning outside `Integer?`; expect static rejection when provable or runtime `TypeError` at boundary | Static return contract of `<=>` lacks vector. |
| D-095 | RUNTIME-C006, RUNTIME-C084, RUNTIME-C132 | none | positive built-in numeric `<=>` with `Integer`, `Float32`, `Float64`, NaN, and nonnumeric object; expect `Integer?` results and `nil` for NaN/nonnumeric | Built-in heterogeneous `<=>` lacks vector. |
| D-096 | RUNTIME-C083, RUNTIME-C156 | none | positive root `Object#<=>` on two ordinary distinct objects; expect `nil`, no address order | Root comparison surface lacks vector. |
| D-097 | RUNTIME-C086, RUNTIME-C156 | none | positive ordinary identity-bearing object default equality with same reference, distinct references, and custom `<=>`; expect identity-first `true` for same object, then `<=>` only for distinct | Identity-first equality lacks vector. |
| D-098 | RUNTIME-C029, RUNTIME-C155 | partial: RUNTIME-V018, RUNTIME-V019 | positive and negative `same?` evaluates operands once, bypasses user `==` and `<=>`, accepts identity-bearing singleton/meta objects, rejects identity-less numbers; also selector `ready?` Method parses and dispatches | Existing vectors cover only a subset of primitive identity and `?` selector behavior. |
| D-099 | RUNTIME-C006, RUNTIME-C155 | RUNTIME-V019 | none | Existing vector directly checks singleton identity for `nil`, `true`, `false`; meta identity is covered by same identity-bearing table but could be extended. |
| D-100 | RUNTIME-C010, RUNTIME-C156 | RUNTIME-V011, RUNTIME-EX001 | none | Existing vector and example check reopen preserves Class identity. |
| D-101 | RUNTIME-C013, RUNTIME-C022, RUNTIME-C156 | RUNTIME-V012, RUNTIME-V013, RUNTIME-EX002 | none | Existing vectors check Method replacement and entered old frames. |
| D-102 | RUNTIME-C014, RUNTIME-C022, RUNTIME-C156 | RUNTIME-V015 | none | Existing vector checks retained Method `super` after owner removal raises `InvalidSuperError`. |
| D-103 | RUNTIME-C014, RUNTIME-C156 | RUNTIME-V015 | none | Same existing vector covers missing lexical owner during retained-Method `super`. |
| D-104 | RUNTIME-C015, RUNTIME-C156 | RUNTIME-V014 | none | Existing vector checks reflective retained Method binding validation with `MethodBindingError`. |
| D-105 | RUNTIME-C006, RUNTIME-C038, RUNTIME-C039, RUNTIME-C156 | RUNTIME-V017, RUNTIME-EX002 | none | Saved BoundMethod identity and invocation after replacement are covered by vector and example. |
| D-106 | RUNTIME-C040, RUNTIME-C156 | RUNTIME-V016, RUNTIME-V017, RUNTIME-EX002 | none | Existing vectors check repeated reads distinct and saved reference same. |
| D-107 | RUNTIME-C006, RUNTIME-C041, RUNTIME-C155 | none | positive compare two BoundMethods with same receiver and Method using default `==` plus `same?`; expect equality identity-only, not structural | BoundMethod default equality lacks exact vector beyond `same?`. |
| D-108 | RUNTIME-C006, RUNTIME-C042 | none | positive evaluate same Closure expression twice and compare plus saved Closure to itself; expect distinct identities and identity-only equality | Closure identity/equality lacks vector. |
| D-109 | RUNTIME-C006, RUNTIME-C036, RUNTIME-C156 | none | positive compare aliased Method object to itself and two distinct Method objects with same body; expect default equality identity-only | Method default equality lacks vector. |
| D-110 | RUNTIME-C006, RUNTIME-C156 | none | positive compare Class, Module, Contract, and Type definition objects by same reference and distinct definitions; expect default equality identity-only unless category overrides | Definition-object equality lacks vector. |
| D-111 | RUNTIME-C006, RUNTIME-C088 | none | positive identity-bearing object hash before and after GC/movement fixture; expect same runtime-local `0..2^64-1` hash, no raw address, not cross-process stable | Identity-object hashing lacks vector. |
| D-112 | RUNTIME-C006, RUNTIME-C145, RUNTIME-C146 | RUNTIME-V008, RUNTIME-V009, RUNTIME-V010, RUNTIME-V045 | none | Existing singleton hash rows cover stable hashes for `nil`, `false`, `true`. |
| D-113 | RUNTIME-C145, RUNTIME-C146 | RUNTIME-V008, RUNTIME-V009, RUNTIME-V010, RUNTIME-V045 | none | Existing rows give exact singleton input bytes, context, digest bytes, and public hash. |
| D-114 | RUNTIME-C091, RUNTIME-C156 | none | positive `false <=> true`, `true <=> false`, equality of same Bool, Bool with `Integer(0)`/`Integer(1)`; expect total Bool order and no numeric equality | Bool ordering lacks vector. |
| D-115 | RUNTIME-C092, RUNTIME-C156 | none | positive `nil <=> nil`, `nil <=> false`, `nil <=> Object.new`, ordered comparisons involving nil; expect only same nil returns `0`, non-nil unordered, no global min/max | Nil comparison lacks vector. |
