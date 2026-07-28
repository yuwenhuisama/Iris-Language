# Iris v1 Conformance Framework

Status: Iris v1 draft, frozen conformance contract.

IRIS-V1-CONFORMANCE-C001: This chapter defines the stable Iris v1 executable example and conformance vector schema. It specifies how normative examples, chapter vector tables, diagnostic cases, malformed inputs, differential backend checks, legacy migration tags, future corpus records, and freeze gates are represented. It MUST NOT implement a runner, add tests, define product source code, or change language semantics.

IRIS-V1-CONFORMANCE-C002: The conformance corpus is a specification artifact contract, not a shipped runner in this documentation wave. Future implementation work MAY create files at the locations named here, but this task only freezes the record shape, required categories, coverage rules, and validation gates.

## Conformance Terms

IRIS-V1-CONFORMANCE-C003: A conformance vector is one stable, machine-readable record that describes input, applicability, expected observations, required clause anchors, required decision anchors, and allowed implementation freedom for one Iris v1 behavior.

IRIS-V1-CONFORMANCE-C004: An executable example is source text in a normative chapter with a stable `IRIS-V1-<chapter>-EX<nnn>` ID. If an executable example is intended to affect conformance, it MUST be represented either by a vector with the same observable outcome or by a traceability row that explains why it is documentation-only.

IRIS-V1-CONFORMANCE-C005: A malformed input vector is a negative or diagnostic vector whose input is intentionally not a valid complete Iris program, package, metadata record, native artifact, serialized value, or FFI sidecar. It MUST still have stable expected diagnostics or rejection status.

IRIS-V1-CONFORMANCE-C006: A backend differential vector checks that the interpreter, JIT, native boundary, or Host integration path produces the same required Iris v1 observation where more than one backend applies. A differential vector MUST NOT allow semantically different results between those backends.

IRIS-V1-CONFORMANCE-C007: A legacy-tagged vector carries migration context from `11-migration-divergence.md`. The legacy tag records whether the case preserves, intentionally diverges from, removes, or defers Legacy Iris behavior, but the expected result remains governed only by the Iris v1 normative clause anchors.

## Stable IDs And Record Identity

IRIS-V1-CONFORMANCE-C008: Every vector ID MUST match `IRIS-V1-<chapter>-V<nnn>` using the fixed chapter codes in [README.md](README.md). Chapter-owned vector IDs MUST remain stable once published.

IRIS-V1-CONFORMANCE-C009: A corpus record MUST contain a human-readable `name`, but tools MUST NOT treat `name` as stable identity. Only `id` is the stable vector identity. `name` MAY change for clarity when `id`, anchors, input, and expected observations are unchanged.

IRIS-V1-CONFORMANCE-C010: A chapter vector table row using kind `Failure` is mapped to canonical category `negative` unless the expected result is only a diagnostic stream obligation, in which case it is mapped to `diagnostic`. The published vector ID and observable result MUST be preserved.

IRIS-V1-CONFORMANCE-C011: Stable record identity includes `id`, `schema_version`, `source.chapter`, `source.clauses`, `source.decisions`, `category`, `input.kind`, `applicability`, and `expect`. A change to any of those fields is a semantic corpus change and MUST be reviewed as a spec change.

IRIS-V1-CONFORMANCE-C012: The initial schema version is `iris-v1-vector-schema-1`. A future incompatible schema version MUST require either a future Iris language major version or a migration document that preserves all Iris v1 vector identities and observations.

## Canonical Categories

IRIS-V1-CONFORMANCE-C013: The `category` field MUST be one of `positive`, `negative`, `diagnostic`, or `differential`.

| Category | Required meaning | Required expected fields |
| --- | --- | --- |
| `positive` | Valid input completes and produces required observations | At least one of `stdout`, `value`, `type`, `diagnostics`, `status`, or `artifact` |
| `negative` | Input is rejected or execution raises a required ordinary Iris error | `error` or `status`, plus `diagnostics` when the owning clause names a diagnostic code |
| `diagnostic` | The primary obligation is diagnostic emission, warning policy, or structured diagnostic payload | `diagnostics` |
| `differential` | Multiple applicable execution paths must agree on the same semantic observation | `backends`, `equivalence`, and at least one ordinary expected observation |

IRIS-V1-CONFORMANCE-C014: `positive` vectors MUST NOT accept an error result unless the vector is explicitly testing an ordinary Iris value that represents an error object as a successful value.

IRIS-V1-CONFORMANCE-C015: `negative` vectors MUST distinguish parser, static validation, runtime exception, ABI status, package load, metadata validation, serialized input rejection, and Host boundary rejection when the owning clause makes that distinction.

IRIS-V1-CONFORMANCE-C016: `diagnostic` vectors MUST record the stable primary diagnostic code, severity, phase, and anchor clause. They MAY record secondary notes, spans, warning-policy behavior, and structured payload keys when those details are normative.

IRIS-V1-CONFORMANCE-C017: `differential` vectors MUST name every backend or path compared. If a backend is inapplicable, the record MUST mark it as `not_applicable` with a clause-backed reason rather than omitting it silently.

## Record Schema

IRIS-V1-CONFORMANCE-C018: Future machine-readable vector records MUST be UTF-8 JSON objects using only stable field names from this chapter. JSON object member order is not semantic.

IRIS-V1-CONFORMANCE-C019: A vector record MUST contain these top-level fields: `schema_version`, `id`, `name`, `category`, `source`, `input`, `applicability`, `expect`, and `tags`. The `name` field MUST be a non-empty human-readable label and MUST NOT be used for identity, lookup stability, deduplication, replacement matching, or traceability joins.

IRIS-V1-CONFORMANCE-C020: The `source` object MUST contain `chapter`, `artifact`, `clauses`, and `decisions`. `clauses` MUST contain at least one `IRIS-V1-<chapter>-C<nnn>` anchor. `decisions` MUST contain every known frozen `D-<nnn>` decision needed to explain the expected result, or an empty array only when the vector is purely editorial and no decision anchor exists.

IRIS-V1-CONFORMANCE-C021: The `input` object MUST contain `kind` and one of `source_text`, `files`, `package`, `metadata`, `native_artifact`, `serialized_value`, `host_calls`, or `fixture_ref`. `kind` MUST be one of `source`, `package`, `metadata`, `native`, `ffi`, `serialized`, `host`, `legacy`, or `fixture`.

IRIS-V1-CONFORMANCE-C022: The `input` object MAY contain `malformed: true` only for negative or diagnostic vectors. When `malformed` is true, the record MUST still identify the malformed surface in `kind` and MUST name the rejection phase in `expect.diagnostics` or `expect.error.phase`.

IRIS-V1-CONFORMANCE-C023: The `applicability` object MUST contain `interpreter`, `jit`, and `native`. Each value MUST be one of `required`, `optional`, `not_applicable`, or `prohibited`.

IRIS-V1-CONFORMANCE-C024: `applicability.reason` MUST exist when any applicability value is `optional`, `not_applicable`, or `prohibited`. The reason MUST cite at least one source clause or explain that the backend does not exist for the input kind.

IRIS-V1-CONFORMANCE-C025: The `expect` object MUST use only these stable observation fields: `stdout`, `stderr`, `value`, `type`, `error`, `diagnostics`, `status`, `artifact`, `backends`, `equivalence`, and `side_effects`.

IRIS-V1-CONFORMANCE-C026: `expect.stdout` and `expect.stderr` MUST be arrays of exact UTF-8 lines without line terminators. Byte-stream observations MUST use `expect.artifact` with an explicit media type and byte encoding.

IRIS-V1-CONFORMANCE-C027: `expect.value` MUST describe the observable Iris value using one of `nil`, `bool`, `integer`, `float32_bits`, `float64_bits`, `string`, `symbol`, `bytes_hex`, `array`, `tuple`, `hash_entries`, `range`, `regex`, `object_identity`, `class_name`, `module_name`, `contract_name`, `task`, `exception_context`, or `opaque_handle_status`.

IRIS-V1-CONFORMANCE-C028: `expect.type` MUST use canonical Iris v1 Type spelling after normalization. Type algebra vectors from [05-types-contracts-generics.md](05-types-contracts-generics.md) MUST compare normalized Type identity, not display spelling alone.

IRIS-V1-CONFORMANCE-C029: `expect.error` MUST contain `kind`, `phase`, and `message_policy`. `kind` MUST be the ordinary Iris error class, ABI status name, package rejection kind, metadata rejection kind, serialized input rejection kind, or parser/static diagnostic category required by the owning clause.

IRIS-V1-CONFORMANCE-C030: `expect.diagnostics` MUST be an array. Each diagnostic entry MUST contain `code`, `severity`, `phase`, and `clause`. It MAY contain `span`, `notes`, `payload`, and `policy` only when those fields are normative.

IRIS-V1-CONFORMANCE-C031: `expect.status` MUST be used for Host ABI, native extension, package load, metadata validation, FFI binding, and serialized value decoding outcomes that are not ordinary Iris values. It MUST contain `kind` and `success`.

IRIS-V1-CONFORMANCE-C032: `expect.artifact` MUST describe generated or validated artifacts only at the semantic level frozen by the owning chapter. It MUST NOT require byte-for-byte implementation output unless the owning chapter makes that byte sequence normative.

IRIS-V1-CONFORMANCE-C033: `expect.side_effects` MUST be present when a vector depends on no partial mutation, exactly-once cleanup, no native call, no package publication, no heap mutation, no candidate commit, no Hash mutation, or a specific diagnostic stream emission.

IRIS-V1-CONFORMANCE-C034: `tags` MUST be an array of lowercase strings. The reserved tag prefixes are `legacy:`, `phase:`, `surface:`, `backend:`, `hash:`, `unicode:`, `ffi:`, `package:`, `diagnostic:`, and `differential:`.

IRIS-V1-CONFORMANCE-C035: Legacy tags MUST use one of `legacy:preserve`, `legacy:intentional-divergence`, `legacy:removed`, `legacy:deferred`, or `legacy:not-applicable`. A legacy-tagged vector SHOULD also cite a migration row once `11-migration-divergence.md` is complete.

## Applicability Rules

IRIS-V1-CONFORMANCE-C036: Interpreter applicability is `required` for every source, package, runtime, type, collection, async, metadata, and diagnostic vector unless the vector only exercises Host ABI or native artifact loading without Iris source execution.

IRIS-V1-CONFORMANCE-C037: JIT applicability is `required` when the vector covers observable language semantics that optimized code may execute, including numeric rounding, stable hash results, dispatch, generic guards, exception selection, iteration, async resumption, open-transaction invalidation, and backend differential behavior.

IRIS-V1-CONFORMANCE-C038: JIT applicability is `not_applicable` for pure lexical rejection, parse rejection, static metadata validation before executable code, and Host ABI table negotiation unless a chapter explicitly requires JIT participation.

IRIS-V1-CONFORMANCE-C039: Native applicability is `required` for Host ABI, native extension, FFI, native metadata, native payload, native async bridge, and native boundary Contract vectors. It is `not_applicable` for pure source grammar vectors that do not cross a native or Host boundary.

IRIS-V1-CONFORMANCE-C040: No vector MAY permit interpreter and JIT to produce different Iris values, Types, errors, diagnostics, side effects, or stable hash results for the same applicable semantic case.

IRIS-V1-CONFORMANCE-C041: D-049 and D-050 require source float literal vectors to compare exact emitted bits and diagnostics across compiler hosts, interpreter, JIT, and target platforms. Host locale, host rounding mode, and unsuitable host float parsers MUST NOT affect expected results.

IRIS-V1-CONFORMANCE-C042: D-077 through D-086 and D-113 require hash vectors to compare exact public 64-bit Integer results, exact derive-key context strings, and exact digest extraction rules where those details are in the owning chapter.

IRIS-V1-CONFORMANCE-C043: D-272 requires revision artifact integrity vectors to compare full BLAKE3-256 manifest digests where artifact recovery behavior is tested. Corpus records MUST NOT shorten that digest for conformance convenience.

IRIS-V1-CONFORMANCE-C044: D-503 requires serialized `IrisValue` vectors to identify the separately specified format version and strict decode limit obligations. This chapter MUST NOT define that byte format, but freeze gates MUST require vectors once the format artifact exists.

IRIS-V1-CONFORMANCE-C045: D-507 through D-510 require grammar vectors to cover complete precedence, associativity, non-chainable operators, closed reserved keywords, and contextual longest-match conflicts with positive and malformed records.

## Required Vector Classes By Chapter

IRIS-V1-CONFORMANCE-C046: The corpus MUST preserve all normative vector IDs published by any Iris v1 chapter, including `LIBRARY` vectors and the stable hash, algebra, and sample-shape tables that precede ordinary vector tables. Chapters that publish no vector IDs still need either mapped vector obligations in another chapter or an explicit metadata-only exception.

| Chapter | Required vector classes before freeze |
| --- | --- |
| `IDENTITY` | No chapter-owned vector IDs are required because the chapter defines identity, versioning, deferral, and non-goal metadata. Freeze still requires every executable identity obligation to be mapped to dependent chapter vectors or a traceability-matrix metadata-only exception. |
| `GRAMMAR` | Keyword inventory, operator inventory, precedence, associativity, contextual longest match, malformed lexical input, malformed parse input, and legacy non-keyword rejection |
| `RUNTIME` | Object identity, dynamic dispatch, Method and BoundMethod identity, MRO, construction, raw ivars, truth, numeric equality and arithmetic, stable numeric and singleton hashes, mutation invalidation, and runtime failures |
| `CONTROL` | Bindings, Closure capture, callable parameters, assignment evaluation order, conditionals, loops, traversal cleanup, labels, match, raise/catch/finally, ExceptionContext, and invalid control transfers |
| `TYPES` | Every Type constructor, algebra normalization, Contract qualified dispatch, Contract view equality and hash, generic invariance and materialization, Dynamic boundaries, type aliases, `Never`, reflection, and failure diagnostics |
| `COLLECTIONS` | String, MutableString, Bytes, ByteArray, Tuple, Array, Hash, Range, Iteration, Regex, Unicode 17.0.0 behavior, stable collection hashes, malformed Regex and text cases, and concurrent modification failures |
| `ASYNC` | Task, Awaitable, await placement, FIFO scheduler behavior, failed Task observation, unobserved Task diagnostics, `using`, Closeable cleanup, async cleanup across suspension, and diagnostic stream payloads |
| `META` | Packages, imports, exports, initialization DAGs, declaration bodies, open transactions, rollback, revision events, MetaCapabilities, decorators, ReflectionPolicy, raw ivars, static extension boundaries, hot upgrade, and failure rollback |
| `FFI` | Host ABI negotiation, opaque handles, GC roots, thread affinity, post queue, native errors, metadata binding, native payloads, Task completion tokens, FFI.open, Library binding, and unsafe-call rejection |
| `LIBRARY` | Published vectors `IRIS-V1-LIBRARY-V001` through `IRIS-V1-LIBRARY-V014`, covering JSON scope, `IrisValue` format boundaries, Encoding API, core package membership, standard package categories, deferred package areas, safe decoding limits, and non-core library rejection |
| `MIGRATION` | Migration ledger rows are `IRIS-V1-MIG-<nnn>` rows, not vector IDs. Freeze requires migration-related conformance vectors, or explicit mapped vector obligations in the grammar, runtime, control, type, collection, meta, library, or conformance corpus, for every ledger tag, changed legacy syntax, removed legacy keyword, replacement example, intentional divergence, preservation case, and deferred legacy feature. |

IRIS-V1-CONFORMANCE-C047: A chapter may satisfy a required vector class through one vector that covers several clauses only when each covered clause is listed in `source.clauses` and the expected observation would fail if any covered clause were wrong.

IRIS-V1-CONFORMANCE-C048: The freeze gate MUST reject a chapter that has normative clauses but no vector class coverage, except for traceability-only clauses that the traceability matrix marks as documentation structure rather than executable behavior.

IRIS-V1-CONFORMANCE-C049: A vector class marked by a chapter as required MUST have at least one positive vector and one negative or diagnostic vector unless the owning clause defines only prohibited behavior. Prohibited-only areas MUST have at least one negative or diagnostic vector.

## Future Corpus Location And Format

IRIS-V1-CONFORMANCE-C050: Future machine-readable corpus files SHOULD live under `conformance/iris-v1/` outside the 14-file spec inventory. The preferred layout is `conformance/iris-v1/vectors/<chapter>/<id>.json`, `conformance/iris-v1/fixtures/`, and `conformance/iris-v1/schema/iris-v1-vector-schema-1.json`.

IRIS-V1-CONFORMANCE-C051: Corpus files MUST NOT be added under `spec/iris-v1/` unless the product artifact inventory in [README.md](README.md) is changed by an approved plan.

IRIS-V1-CONFORMANCE-C052: A corpus bundle MUST include a manifest named `conformance/iris-v1/manifest.json` that records corpus schema version, Iris language major, Unicode version, hash schema version, source spec commit or release identifier, vector count, fixture digest list, and any separately versioned `IrisValue` format reference.

IRIS-V1-CONFORMANCE-C053: Fixture files MUST be content-addressed by BLAKE3-256 and records MUST cite fixture digest, logical path, media type, and role. Fixture paths and mirrors MUST NOT affect expected semantic results.

IRIS-V1-CONFORMANCE-C054: Corpus validators MUST reject duplicate vector IDs, malformed IDs, unknown chapter codes, unresolved source clauses, unresolved D-IDs, unsupported category values, missing expected observations, malformed applicability values, and legacy tags outside the reserved vocabulary.

## Valid And Invalid Sample Records

IRIS-V1-CONFORMANCE-C055: The following valid sample record is normative for schema shape. It is not a new language test beyond the cited grammar clauses and decisions.

```json iris-v1-vector-valid
{
  "schema_version": "iris-v1-vector-schema-1",
  "id": "IRIS-V1-GRAMMAR-V003",
  "name": "Exponent precedence parses right-associative exponent before unary negation",
  "category": "positive",
  "source": {
    "chapter": "GRAMMAR",
    "artifact": "spec/iris-v1/02-lexical-grammar.md",
    "clauses": ["IRIS-V1-GRAMMAR-C041", "IRIS-V1-GRAMMAR-C042"],
    "decisions": ["D-033", "D-507", "D-508"]
  },
  "input": {
    "kind": "source",
    "source_text": "2 ** 3 ** 2\n-2 ** 2\n2 ** -3"
  },
  "applicability": {
    "interpreter": "required",
    "jit": "required",
    "native": "not_applicable",
    "reason": "Pure source expression parsing has no native boundary."
  },
  "expect": {
    "artifact": {
      "parse_shapes": ["2 ** (3 ** 2)", "-(2 ** 2)", "2 ** (-3)"]
    }
  },
  "tags": ["phase:parse", "surface:operator", "differential:interpreter-jit"]
}
```

IRIS-V1-CONFORMANCE-C056: The following invalid sample record is normative for checker failure shape. A schema checker MUST reject it because a negative vector lacks both `expect.error` and `source.decisions`.

```json iris-v1-vector-invalid
{
  "schema_version": "iris-v1-vector-schema-1",
  "id": "IRIS-V1-FFI-V018",
  "name": "Signature-less FFI native call is rejected",
  "category": "negative",
  "source": {
    "chapter": "FFI",
    "artifact": "spec/iris-v1/09-native-host-ffi.md",
    "clauses": ["IRIS-V1-FFI-C046"],
    "decisions": []
  },
  "input": {
    "kind": "ffi",
    "source_text": "FFI.open(path).call(:danger)"
  },
  "applicability": {
    "interpreter": "required",
    "jit": "not_applicable",
    "native": "required",
    "reason": "FFI binding crosses the native boundary."
  },
  "expect": {},
  "tags": ["ffi:binding", "phase:runtime"]
}
```

## Freeze Gates

IRIS-V1-CONFORMANCE-C057: Iris v1 specification freeze MUST reject any corpus or traceability set with an uncovered normative clause, unless that clause is explicitly traceability-only, informative, or marked `DEFERRED V1` with no executable obligation.

IRIS-V1-CONFORMANCE-C058: Freeze MUST reject unresolved links, unresolved Markdown anchors, unresolved D-IDs, unresolved vector IDs, unresolved example IDs, duplicate IDs, malformed ID prefixes, and references to chapters that are not in the fixed inventory.

IRIS-V1-CONFORMANCE-C059: Freeze MUST reject any deferred item that is tested or described as normative behavior. A deferred item MAY have a diagnostic or migration vector only when the expected result is rejection, no feature, or explicit deferred-status reporting.

IRIS-V1-CONFORMANCE-C060: Freeze MUST reject backend-dependent behavior. Any interpreter, JIT, native, Host, or platform difference that affects Iris value, Type, error, diagnostic, stable hash, artifact integrity, side effect, or ordering semantics is non-conforming unless a source clause explicitly marks the backend as not applicable.

IRIS-V1-CONFORMANCE-C061: Freeze MUST reject vector records that omit required expected observations for their category, omit a required error or diagnostic for malformed input, omit applicability, use a legacy tag outside IRIS-V1-CONFORMANCE-C035, or cite a clause without preserving that clause's observable result.

IRIS-V1-CONFORMANCE-C062: Freeze MUST reject chapter tables whose vector IDs are not represented in the future corpus manifest, unless the manifest contains an explicit replacement record with the same vector ID and same expected observation.

IRIS-V1-CONFORMANCE-C063: Freeze MUST treat `LIBRARY` as complete for chapter-owned vector publication only when `IRIS-V1-LIBRARY-V001` through `IRIS-V1-LIBRARY-V014` are preserved or mapped with the same expected observations. Freeze MUST continue to treat `MIGRATION` as incomplete until every migration ledger row has a migration-related conformance vector or an explicit mapped vector obligation. Freeze MUST treat `IDENTITY` as complete without chapter-owned vector IDs only when all executable identity obligations are covered by dependent chapter vectors and all remaining identity clauses are marked as metadata-only in traceability.

IRIS-V1-CONFORMANCE-C064: Freeze MUST reject use of Legacy Iris scripts, old generated parser files, old PDF text, current implementation quirks, or native extension examples as normative authority unless a vector cites a frozen Iris v1 clause that explicitly adopts the behavior.

IRIS-V1-CONFORMANCE-C065: Freeze MUST reject sample schema validation if the valid sample in IRIS-V1-CONFORMANCE-C055 fails or the invalid sample in IRIS-V1-CONFORMANCE-C056 passes.

## Validation Checklist

IRIS-V1-CONFORMANCE-C066: A documentation validator for this chapter MUST check local clause sequence, duplicate IDs, Markdown table shape, local links, sample record validation, required category vocabulary, required expected fields, reserved legacy tags, cited D-ID format, and absence of runner or product implementation files in this task.

IRIS-V1-CONFORMANCE-C067: A corpus coverage checker MUST extract normative chapter clauses, examples, vector tables, and traceability rows, then report coverage by chapter, category, vector class, backend applicability, malformed input coverage, legacy tag coverage, and D-ID coverage.

IRIS-V1-CONFORMANCE-C068: A differential checker MUST compare only semantic observations defined by `expect`. It MUST NOT compare implementation logs, heap addresses, object pointer values, timing, private bytecode layout, or non-normative debug strings.

IRIS-V1-CONFORMANCE-C069: A diagnostic checker MUST compare stable diagnostic code, severity, phase, clause anchor, and required payload keys. It MUST NOT require exact English diagnostic wording unless an owning clause explicitly freezes that wording.

IRIS-V1-CONFORMANCE-C070: A malformed-input checker MUST prove that malformed lexical, parse, metadata, FFI, Host, native artifact, package, and serialized-value inputs fail at the specified phase without partial semantic effects when side effects are part of the expected observation.

IRIS-V1-CONFORMANCE-C071: A legacy coverage checker MUST compare the migration ledger tags with vector tags and reject any removed or intentionally divergent Legacy Iris behavior that lacks a v1 replacement, rejection, or deferred-status vector.

## Traceability Notes

IRIS-V1-CONFORMANCE-C072: This chapter anchors the approved verification strategy that examples compile into a future machine-readable conformance corpus. It does not own new language semantics.

IRIS-V1-CONFORMANCE-C073: This chapter cites decision IDs `D-049`, `D-050`, `D-077`, `D-078`, `D-079`, `D-080`, `D-081`, `D-082`, `D-083`, `D-084`, `D-085`, `D-086`, `D-113`, `D-272`, `D-503`, `D-507`, `D-508`, `D-509`, and `D-510` for conformance-sensitive literal, hash, artifact, serialization, precedence, associativity, keyword, and contextual-token obligations.

IRIS-V1-CONFORMANCE-C074: This chapter also depends on [README.md](README.md) clauses IRIS-V1-TRACE-C001 through IRIS-V1-TRACE-C018 for inventory, ID, editorial, non-goal, frozen-semantics, and backend-independence rules.

## Audit-Exact Conformance Vectors

IRIS-V1-CONFORMANCE-C075: The following conformance vectors are normative meta-level vector definitions. Each vector uses a concrete JSON record fixture as input and defines exact validator acceptance, rejection, status, diagnostics, or artifact observations.

| Vector ID | Category | Applicability | Source/Input | Expected observable | Decisions |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-CONFORMANCE-V010` | diagnostic | interpreter required; JIT not applicable; native not applicable; corpus validator required | JSON record fixture `{"schema_version":"iris-v1-vector-schema-1","id":"IRIS-V1-LIBRARY-V005","name":"Reject unsupported IrisValue format version","category":"negative","source":{"chapter":"LIBRARY","artifact":"spec/iris-v1/10-serialization-standard-library.md","clauses":["IRIS-V1-LIBRARY-C007","IRIS-V1-LIBRARY-C008"],"decisions":["D-503"]},"input":{"kind":"serialized","serialized_value":{"format":"IrisValue","bytes_hex":"4952563102","declared_version":2,"limit_profile":"v1-default"},"malformed":true},"applicability":{"interpreter":"required","jit":"not_applicable","native":"not_applicable","reason":"Serialized IrisValue validation is a decode-surface check before executable code."},"expect":{"status":{"kind":"IrisValueUnsupportedVersion","success":false},"diagnostics":[{"code":"IRISVALUE_UNSUPPORTED_VERSION","severity":"error","phase":"decode","clause":"IRIS-V1-LIBRARY-C008"}],"side_effects":{"published_value":false,"allocation_from_unvalidated_size":false}},"tags":["phase:decode","surface:serialized"]}` | Validator accepts the record; `expect.status.success` is `false`; diagnostic code is exactly `IRISVALUE_UNSUPPORTED_VERSION`; `phase` is exactly `decode`; `side_effects.published_value` is `false`; `side_effects.allocation_from_unvalidated_size` is `false`. | `D-503` |
| `IRIS-V1-CONFORMANCE-V011` | positive | interpreter required; JIT required; native not applicable; corpus validator required | JSON record fixture `{"schema_version":"iris-v1-vector-schema-1","id":"IRIS-V1-GRAMMAR-V008","name":"Full precedence source forms parse to canonical shapes","category":"positive","source":{"chapter":"GRAMMAR","artifact":"spec/iris-v1/02-lexical-grammar.md","clauses":["IRIS-V1-GRAMMAR-C041","IRIS-V1-CONFORMANCE-C045"],"decisions":["D-507"]},"input":{"kind":"source","source_text":"a.b(c)[d] ** -e * f + g << h & i ^ j \u007c k ..< l < m == n named o && p \u007c\u007c q = r"},"applicability":{"interpreter":"required","jit":"required","native":"not_applicable","reason":"Pure source parsing has no native boundary."},"expect":{"artifact":{"parse_shape":"assign(q_or_chain(logical_or(logical_and(named_infix(equality(relational(range(bit_or(bit_xor(bit_and(shift(add(mul(pow(primary_chain(a.b(c)[d]), unary(-e)), f), g), h), i), j), k), l), m), n), named, o), p)), r)","precedence_rows_covered":17},"diagnostics":[]},"tags":["phase:parse","surface:operator","differential:interpreter-jit"]}` | Validator accepts the record; `expect.artifact.precedence_rows_covered` is exactly `17`; `expect.diagnostics` is empty; interpreter and JIT applicability are both `required`; native applicability is `not_applicable`. | `D-507` |
| `IRIS-V1-CONFORMANCE-V012` | diagnostic | interpreter required; JIT required; native not applicable; corpus validator required | JSON record fixture `{"schema_version":"iris-v1-vector-schema-1","id":"IRIS-V1-GRAMMAR-V007","name":"Reject non-associative operator chains","category":"diagnostic","source":{"chapter":"GRAMMAR","artifact":"spec/iris-v1/02-lexical-grammar.md","clauses":["IRIS-V1-GRAMMAR-C043","IRIS-V1-CONFORMANCE-C045"],"decisions":["D-508"]},"input":{"kind":"source","source_text":"a < b < c\nx as T as U","malformed":true},"applicability":{"interpreter":"required","jit":"required","native":"not_applicable","reason":"Pure parse diagnostics have no native boundary."},"expect":{"diagnostics":[{"code":"PARSE_NONASSOCIATIVE_CHAIN","severity":"error","phase":"parse","clause":"IRIS-V1-GRAMMAR-C043","payload":{"forms":["a < b < c","x as T as U"]}}],"side_effects":{"program_accepted":false,"bytecode_emitted":false}},"tags":["phase:parse","surface:operator","diagnostic:nonassociative-chain"]}` | Validator accepts the record; diagnostic array length is `1`; diagnostic code is exactly `PARSE_NONASSOCIATIVE_CHAIN`; `phase` is exactly `parse`; `side_effects.program_accepted` is `false`; `side_effects.bytecode_emitted` is `false`. | `D-508` |
