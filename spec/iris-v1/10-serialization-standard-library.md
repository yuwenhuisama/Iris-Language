# Iris v1 Serialization And Standard Library Boundary

Status: Iris v1 draft, frozen semantics.

IRIS-V1-LIBRARY-C001: This chapter defines the Iris v1 serialization contract boundary, JSON responsibilities, separately versioned IrisValue responsibility, safe decoding limits, Encoding and Unicode standard surfaces, core package boundary, official standard-package categories, deferred library areas, and Regex package split. It MUST be read after [README.md](README.md), [01-language-identity.md](01-language-identity.md), [05-types-contracts-generics.md](05-types-contracts-generics.md), [06-collections-text-regex.md](06-collections-text-regex.md), [08-modules-metaprogramming.md](08-modules-metaprogramming.md), and [09-native-host-ffi.md](09-native-host-ffi.md).

IRIS-V1-LIBRARY-C002: This chapter MUST NOT define parser productions, collection storage layouts, stable hash canonical bytes beyond earlier chapters, exact IrisValue byte tags, JSON parser implementation strategy, Regex engine internals, cryptography APIs, HTTP APIs, database APIs, GUI APIs, databases, or Host ABI function names. It fixes ownership, eligibility, versioning, and safety rules only.

## Serialization Contract Scope

IRIS-V1-LIBRARY-C003: Iris v1 serialization is explicit and Contract-driven. The language core and standard library MUST NOT reflectively serialize arbitrary objects, receiver-name raw ivars, Class-object ivars, hierarchy Class variables, private fields, reflection metadata tables, Method bodies, Closure captures, Task state, FFI handles, native payloads, or external resources merely because such state exists.

IRIS-V1-LIBRARY-C004: The standard `Serializable` Contract is an opt-in promise that a Class supplies an explicit serialization representation. A Class participates only when its declared static spine lists `for Serializable` or a more specific standard serialization Contract that refines it. Duck typing, reflection visibility, `to_string`, `inspect`, raw ivar access, and public property presence MUST NOT imply serialization eligibility.

IRIS-V1-LIBRARY-C005: A `Serializable` representation is ordinary Iris data chosen by the Class implementation. It MUST be built from values that the selected target format accepts, plus explicit type identity and schema metadata when the value is meant to round-trip as a nominal Class instance. The representation MUST NOT contain hidden runtime pointers, object IDs, raw ivar names or values unless the Class deliberately publishes that exact data as ordinary representation content, native handles, open file descriptors, scheduler state, or ReflectionPolicy-bypassed data.

IRIS-V1-LIBRARY-C006: Deserialization into a nominal Class MUST validate the package identity, package API major, Iris language major, declared schema version, declared `Serializable` Contract conformance, generic constraints, constructor or factory Contract, and representation shape before publishing the result. Failure raises an ordinary Iris decoding or Type Contract error and MUST NOT publish a partially initialized object.

IRIS-V1-LIBRARY-C007: `Serializable` does not override visibility, MetaCapabilities, package permissions, or ReflectionPolicy. Private data may appear in a serialized representation only when the Class's own serialization implementation explicitly emits it under its ordinary authority. External callers cannot obtain private state by selecting JSON, IrisValue, reflection, or FFI APIs.

IRIS-V1-LIBRARY-C008: The following serialization eligibility matrix is normative:

| Value family                                                                                   | JSON default                                                             | IrisValue default                                                                  | Serializable opt-in                                                    | Required rejection or limit                                                                     |
| ---------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------ | ---------------------------------------------------------------------------------- | ---------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| `nil`, Bool, Integer, finite Float                                                           | Supported when JSON number limits are satisfied                          | Supported, including arbitrary Integer and declared Float widths or bits           | Not needed                                                             | JSON rejects non-finite Float unless an explicit package option maps it to data                 |
| String and Symbol                                                                              | String supported, Symbol only by explicit representation                 | Supported                                                                          | Symbol may be represented as content only where the format declares it | Invalid Unicode is impossible for String and Symbol                                             |
| Bytes                                                                                          | Not a JSON scalar, requires explicit representation such as encoded text | Supported                                                                          | Optional representation for JSON                                       | Decoder validates declared encoding and length before allocation                                |
| Tuple and Array                                                                                | Supported as arrays when every element is supported                      | Supported                                                                          | Not needed for container shape                                         | Element count and nesting depth are bounded                                                     |
| Hash                                                                                           | Supported only with String keys or explicit key representation           | Supported when each key and value is supported                                     | Optional representation for JSON                                       | Duplicate decoded keys and invalid key Contracts are errors                                     |
| Regex and Match                                                                                | Regex only through explicit representation, Match is not default JSON    | Regex supported only if IrisValue format version declares it, Match is not default | Optional representation may name Regex pattern and flags               | Core Regex semantics remain owned by[06-collections-text-regex.md](06-collections-text-regex.md) |
| Ordinary Class instance                                                                        | Not automatic                                                            | Not automatic                                                                      | Required for nominal round-trip                                        | No automatic raw ivar, property, or reflection-table dumping                                    |
| Class, Module, Contract, Type, Method, BoundMethod, Closure                                    | Not automatic                                                            | Not automatic except Type metadata where a format version explicitly declares it   | Only explicit metadata representations, not executable identity        | No code, revision, Closure environment, or Method body resurrection                             |
| Task, Awaitable, Iterator, ExceptionContext, diagnostic event                                  | Not automatic                                                            | Not automatic                                                                      | Diagnostic packages may define explicit records                        | Live scheduler, cursor, cleanup, and failure observation state are not serialized               |
| FFI Library, Host handle, native payload, File, socket, database connection, external resource | Not automatic                                                            | Not automatic                                                                      | Only explicit resource descriptors chosen by a package                 | Live handles and native pointers are rejected                                                   |

## JSON Responsibilities

IRIS-V1-LIBRARY-C009: `JSON` is a stable runtime package for parsing and emitting JSON-compatible data. It owns JSON text grammar conformance, JSON value mapping, canonical error reporting, and safe parser limits. It does not own arbitrary Iris object persistence, binary IrisValue compatibility, cryptography, HTTP transport, filesystem policy, or schema language design.

IRIS-V1-LIBRARY-C010: JSON decoding MUST produce only JSON-compatible Iris values unless the caller explicitly supplies a `Serializable` target, factory, or schema. The default JSON value set is `nil`, Bool, String, finite numeric values within the implementation's documented JSON numeric mode, Array, and Hash with String keys. JSON decoding MUST NOT instantiate arbitrary Classes from type names in input by default.

IRIS-V1-LIBRARY-C011: JSON encoding MUST accept only JSON-compatible values by default. A Class instance may be encoded only through its explicit `Serializable` representation or a caller-supplied encoder. The encoder MUST reject unsupported values rather than falling back to `to_string`, `inspect`, object identity, raw ivar scanning, Method enumeration, or ReflectionPolicy-bypassing metadata.

IRIS-V1-LIBRARY-C012: JSON parsing and generation use Unicode scalar Strings and UTF-8 Bytes at explicit text or binary boundaries. Invalid UTF-8 bytes raise `EncodingError` before JSON tokens are interpreted. JSON APIs MUST NOT select OS locale, process code page, terminal encoding, filesystem encoding, or Host default encoding implicitly.

IRIS-V1-LIBRARY-C013: JSON safe decoding limits MUST be explicit and configurable by ordinary options with safe defaults. At minimum, a conforming implementation MUST bound input byte length, decoded text length, nesting depth, object member count, array element count, string scalar length, numeric token length, and total output allocation. The decoder MUST validate sizes before allocating target buffers or containers.

IRIS-V1-LIBRARY-C014: JSON object duplicate-name behavior MUST be a documented option. The safe default MUST reject duplicate names when decoding to Hash or a typed representation unless a caller explicitly selects a deterministic last-wins, first-wins, or collect-all policy. The selected policy MUST be visible in diagnostics and MUST NOT depend on hash iteration order.

## IrisValue Format Responsibility

IRIS-V1-LIBRARY-C015: `IrisValue` is the official Iris binary value format family for supported primitives, immutable values, collections, and explicit `Serializable` representations. Iris v1 requires the existence of this separately specified versioned format, but this language specification does not define its byte tags, field ordering, compression, checksum, or complete byte schema.

IRIS-V1-LIBRARY-C016: Every IrisValue stream MUST carry magic, format version, compatibility metadata, and enough declared context to validate Iris language major, Unicode data version, public hash semantic version where relevant, schema or representation version, package identity and API major for nominal values, and feature requirements before decoding any payload that depends on those facts.

IRIS-V1-LIBRARY-C017: IrisValue decoding MUST impose strict bounded lengths and counts. The format specification MUST define maximum or caller-configurable limits for stream length, nesting depth, element count, byte length, scalar length, symbol length, package-name length, type-argument count, schema metadata length, and total allocation. A decoder MUST NOT trust unvalidated input sizes, multiply sizes without overflow checks, allocate from declared lengths before limit validation, or keep reading after a failed structural check.

IRIS-V1-LIBRARY-C018: IrisValue supports arbitrary Integer, declared Float widths and bits, `nil`, Bool, String, Symbol, Bytes, Tuple, Array, Hash, and explicit `Serializable` representations. It MAY support additional stable identity-less value families only when the format version declares them and the owning semantic chapter defines their value equality and version constraints. It MUST NOT silently encode live Class, Module, Method, Closure, Task, FFI, native resource, iterator cursor, or runtime handle identity.

IRIS-V1-LIBRARY-C019: IrisValue Hash decoding MUST validate every decoded key under current Type and hash Contracts before insertion. Duplicate logical keys after decoding MUST raise a deterministic decoding error unless a format version explicitly declares a deterministic merge rule. The merge rule MUST NOT depend on runtime hash bucket order.

IRIS-V1-LIBRARY-C020: IrisValue nominal deserialization MUST call only declared standard deserialization factories or constructors for the relevant `Serializable` representation. It MUST validate generic constraints as required by [05-types-contracts-generics.md](05-types-contracts-generics.md), package identity as required by [08-modules-metaprogramming.md](08-modules-metaprogramming.md), and native payload restrictions as required by [09-native-host-ffi.md](09-native-host-ffi.md).

## Encoding And Unicode Division

IRIS-V1-LIBRARY-C021: Iris language core guarantees UTF-8 source text, Unicode scalar String content, and strict UTF-8 text-to-binary convenience conversion as defined by [06-collections-text-regex.md](06-collections-text-regex.md). `String#to_bytes`, `MutableString#to_bytes`, `Bytes#to_string`, and ByteArray snapshot decoding use strict UTF-8 by default.

IRIS-V1-LIBRARY-C022: The standard `Encoding` package owns explicit Encoding objects. It MUST provide at least UTF-8, UTF-16LE, UTF-16BE, and Latin-1. Strict error handling is the default for every Encoding object. Replacement, ignore, or other lossy behavior requires explicit options at the call site and MUST NOT be selected by default.

IRIS-V1-LIBRARY-C023: String remains Unicode scalar content and never retains source encoding, decoded file encoding, terminal encoding, or package encoding as hidden state. Converting String to Bytes always requires an explicit target Encoding except for the core UTF-8 convenience Method. Converting Bytes or ByteArray to String always requires strict validation unless the caller explicitly selects a non-strict error option.

IRIS-V1-LIBRARY-C024: Default Unicode data for Iris language major version 1 is Unicode 17.0.0 as fixed by [06-collections-text-regex.md](06-collections-text-regex.md). Encoding, Unicode property, normalization, case mapping, casefold, JSON, IrisValue, and Regex APIs that use default Unicode semantics MUST use that version unless they are explicitly versioned APIs outside the Iris v1 language default.

IRIS-V1-LIBRARY-C025: Encoding APIs MUST NOT select an OS locale, current process code page, filesystem code page, console code page, environment variable, or Host default implicitly. Host or standard packages may expose those settings as explicit values, but selecting them for decoding or encoding requires caller choice.

IRIS-V1-LIBRARY-C026: The following Encoding division table is normative:

| Surface                                                      | Core or standard               | Required behavior                           | Explicitly not included          |
| ------------------------------------------------------------ | ------------------------------ | ------------------------------------------- | -------------------------------- |
| Source file decoding                                         | Core language                  | UTF-8 source text only                      | Locale-selected source decoding  |
| `String#to_bytes` and `MutableString#to_bytes`           | Core text convenience          | Immutable UTF-8 Bytes snapshot              | Hidden source encoding retention |
| `Bytes#to_string` and ByteArray snapshot decode            | Core text convenience          | Strict UTF-8 or`EncodingError`            | Lossy default replacement        |
| `Encoding::UTF_8`, `UTF_16LE`, `UTF_16BE`, `LATIN_1` | Stable standard package        | Explicit objects with strict default errors | Locale-selected fallback         |
| Additional encodings                                         | Separately versioned packages  | Explicit package identity and version       | Language-core ABI promise        |
| Unicode properties and normalization                         | Core plus standard Unicode API | Default Unicode 17.0.0                      | Host Unicode table drift         |

## Core Runtime Packages

IRIS-V1-LIBRARY-C027: Iris v1 core and stable runtime packages are part of the language-major compatibility surface but are not all parser or VM primitives. Their public Contracts, value mappings, diagnostics, and semantic versioning MUST preserve the Iris v1 compatibility rules in [01-language-identity.md](01-language-identity.md).

IRIS-V1-LIBRARY-C028: The following core versus standard table is normative:

| Area                                                      | Core or stable runtime package                            | Separately versioned official standard package             | V1 boundary rule                                                         |
| --------------------------------------------------------- | --------------------------------------------------------- | ---------------------------------------------------------- | ------------------------------------------------------------------------ |
| Object, Class, Module, Contract, Type, Method, Closure    | Core                                                      | Reflection convenience packages may provide queries        | Core owns identity and dispatch, convenience APIs cannot alter semantics |
| ReflectionPolicy, MetaCapabilities, package metadata      | Core                                                      | Tooling packages may present views                         | Mutation still uses core transactions and permissions                    |
| Collections, text, bytes, Symbol, Range, iteration        | Core                                                      | Algorithms and adapters may be standard packages           | Core owns value semantics and stable hashes                              |
| Regex and Match                                           | Core safe Regex subset                                    | Advanced PCRE-style Regex engines                          | Standard engines use distinct types and do not replace core Regex        |
| Async Task, Awaitable, event loop, Closeable, diagnostics | Core                                                      | Extra schedulers or observability sinks                    | Standard packages cannot introduce cancellation semantics to core v1     |
| IO, File, Path                                            | Stable runtime package                                    | Filesystem watchers, archives, globbing, shell integration | Host permissions still govern effects                                    |
| Encoding and Unicode                                      | Stable runtime package                                    | Extra encodings and locale adapters                        | Core defaults stay UTF-8 and Unicode 17.0.0                              |
| JSON                                                      | Stable runtime package                                    | Schema, JSON Lines, canonicalization profiles              | JSON does not imply arbitrary object dumping                             |
| IrisValue                                                 | Stable runtime package plus separate format specification | Format tooling and migration helpers                       | Language spec requires versioning and limits, not byte tags              |
| FFI                                                       | Stable runtime package                                    | Binding generators and platform libraries                  | Script binary calls still use the sole FFI path                          |
| Package, manifest, lock, permissions                      | Core                                                      | Registries, publishing tools, update advisors              | Runtime never resolves latest implicitly                                 |
| Testing and conformance helpers                           | Stable runtime package                                    | Test frameworks and reporters                              | Conformance vectors remain specification-owned                           |
| Cryptography                                              | Not core ABI                                              | Official crypto packages                                   | Internal BLAKE3 use is not a public crypto suite                         |
| HTTP and general networking protocols                     | Not core ABI                                              | Official network and protocol packages                     | Permission names may exist, protocol APIs are separate                   |
| Databases                                                 | Not core ABI                                              | Official database packages                                 | Connections are external resources, not serializable live state          |
| GUI and platform UI                                       | Not core ABI                                              | Official GUI packages                                      | Platform event loops cannot change Iris task semantics                   |

IRIS-V1-LIBRARY-C029: Required core Modules or namespaces include `Object`, `Type`, `Contract`, `Reflection`, `Collections`, `Text`, `Bytes`, `Regex`, `Async`, `IO`, `File`, `Path`, `Encoding`, `Unicode`, `JSON`, `IrisValue`, `FFI`, `Package`, `Diagnostics`, and `Testing`. A conforming implementation MAY organize files or internal modules differently, but these public package surfaces MUST be present or aliased by the standard package map.

IRIS-V1-LIBRARY-C030: Internal BLAKE3 use for stable hashes, revision artifact integrity, package artifact digests, or IrisValue integrity metadata MUST NOT imply a public cryptography API in the core runtime. Public cryptography belongs only to separately versioned official standard packages.

## Official Standard Packages And Deferred Areas

IRIS-V1-LIBRARY-C031: Official standard packages outside the core ABI are separately versioned packages with package identity, API major, manifests, permissions, and compatibility rules from [08-modules-metaprogramming.md](08-modules-metaprogramming.md). They may be distributed with an implementation, but their APIs are not Iris v1 language-core ABI unless this chapter lists the area as core or stable runtime.

IRIS-V1-LIBRARY-C032: The following official standard-package categories are named for v1 boundary purposes:

| Category                                                     | Status                               | Boundary rule                                                                     |
| ------------------------------------------------------------ | ------------------------------------ | --------------------------------------------------------------------------------- |
| Advanced Regex engines                                       | `DEFERRED V1 to standard packages` | Must use distinct types from core Regex and cannot change`/.../flags` semantics |
| Cryptography                                                 | `DEFERRED V1 to standard packages` | No public core crypto suite follows from BLAKE3 internals                         |
| HTTP clients and servers                                     | `DEFERRED V1 to standard packages` | Protocol APIs are not language-core ABI                                           |
| General networking protocols                                 | `DEFERRED V1 to standard packages` | Permissions may gate access, but APIs are separate packages                       |
| Databases and query clients                                  | `DEFERRED V1 to standard packages` | Connections are Closeable external resources, not serializable live values        |
| GUI, graphics, and platform UI                               | `DEFERRED V1 to standard packages` | Platform loops must preserve Iris scheduler semantics                             |
| Compression, archives, and checksums                         | `DEFERRED V1 to standard packages` | Not part of IrisValue byte schema in this language spec                           |
| Schema languages and validation profiles                     | `DEFERRED V1 to standard packages` | JSON and IrisValue define base mapping only                                       |
| Locale, collation, and culture adapters                      | `DEFERRED V1 to standard packages` | Defaults cannot override Unicode 17.0.0 or OS-free Encoding rules                 |
| Filesystem watchers and shell/process integration            | `DEFERRED V1 to standard packages` | IO/File/Path core remains the stable base surface                                 |
| Time zones, calendars, and clocks beyond minimal diagnostics | `DEFERRED V1 to standard packages` | No hidden dependency for serialization timestamps                                 |
| Image, audio, video, and binary media codecs                 | `DEFERRED V1 to standard packages` | Bytes stays untyped byte content by core semantics                                |

IRIS-V1-LIBRARY-C033: A package in a deferred category MAY define its own serializable records, permission names, resource records, and Encoding adapters. It MUST NOT claim language-core ABI status, bypass Host permission grants, mutate core value semantics, grant implicit JSON or IrisValue eligibility for unrelated Classes, or change the safe defaults defined in this chapter.

IRIS-V1-LIBRARY-C034: Advanced Regex packages MUST cross-link to the core Regex rules instead of replacing them. They may provide PCRE-style features, backtracking controls, or host engine bindings only through distinct package Types and explicit construction APIs. Core Regex literals, `=~`, `!~`, flags, Match immutability, and safe subset limits remain owned by [06-collections-text-regex.md](06-collections-text-regex.md).

## Safe Decoding And Diagnostics

IRIS-V1-LIBRARY-C035: Every standard decoder for JSON, IrisValue, Encoding, package metadata, or standard package records MUST validate complete structure before trusting sizes, counts, type identities, schema versions, or allocation requests. A malformed input failure MUST be an ordinary Iris decoding exception or structured diagnostic, not an assertion failure, host panic, process abort, unchecked allocation, or undefined state.

IRIS-V1-LIBRARY-C036: Safe decoding diagnostics MUST identify the decoder, format or package name, format version when known, byte or scalar offset when available, structural path when available, violated limit or expected Contract, and whether any partial result was discarded. Diagnostics MUST NOT include secret Host paths, raw native pointers, object addresses, or private data beyond the input fragment needed to report the failure under policy.

IRIS-V1-LIBRARY-C037: The historical `.irc` hazard where binary input was trusted without magic, version, size, read, or allocation checks is negative evidence only. Iris v1 serialization and package formats MUST reject that pattern by requiring version metadata, bounded sizes, successful reads, and validation before allocation or publication.

## Examples And Conformance Vectors

IRIS-V1-LIBRARY-EX001: Informative example, explicit Serializable representation:

```iris
class User for Serializable {
  property name: String { get; }
  property age: Integer { get; }

  impl fun to_serializable() -> Hash<String,Object> {
    %{ "schema": 1, "name": name, "age": age }
  }
}
```

IRIS-V1-LIBRARY-EX002: Informative example, JSON encoding rejects implicit object dumping:

```iris
let task = read_async()
JSON.encode(task)        // error, Task is not a JSON value
JSON.encode(task.to_string())
```

IRIS-V1-LIBRARY-C038: The following vector table is normative. The conformance chapter MUST preserve these vector IDs or map them to machine-readable records with the same observable outcomes:

| Vector ID | Category | Applicability | Source/Input | Expected observable | Decisions |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-LIBRARY-V001` | negative | interpreter required; JIT required; native not applicable; JSON encoder surface | `JSON.encode(Plain.new())`, where `Plain` does not declare `for Serializable` and has raw ivar `@secret` plus public property `name`. | Raises `SerializationError`; emitted bytes are absent; no raw ivar, property value, object ID, Method, or reflection metadata is observed; clauses `IRIS-V1-LIBRARY-C003`, `IRIS-V1-LIBRARY-C004`, and `IRIS-V1-LIBRARY-C011` apply. | `D-502` |
| `IRIS-V1-LIBRARY-V002` | positive | interpreter required; JIT required; native not applicable; JSON encoder surface | `JSON.encode(User.new("Ada", 37), canonical: true)`, where `User for Serializable` returns `%{ "schema": 1, "name": "Ada", "age": 37 }`. | Returns exact UTF-8 String `{"age":37,"name":"Ada","schema":1}` under canonical ordering; no non-representation state is emitted; clauses `IRIS-V1-LIBRARY-C004`, `IRIS-V1-LIBRARY-C005`, and `IRIS-V1-LIBRARY-C011` apply. | `D-502` |
| `IRIS-V1-LIBRARY-V003` | negative | interpreter required; JIT required; native not applicable; JSON decoder surface | `JSON.decode(nested_array_depth_4, limits: { depth: 3 })`, where the input is valid UTF-8 JSON and no caller schema is supplied. | Raises `JSONLimitError` before allocating the fourth nested Array; no partial value is published; clauses `IRIS-V1-LIBRARY-C013` and `IRIS-V1-LIBRARY-C035` apply. | `D-502` |
| `IRIS-V1-LIBRARY-V004` | negative | interpreter required; JIT required; native not applicable; JSON Bytes input surface | `JSON.decode(Bytes[0xc3,0x28])`. | Raises `EncodingError` before JSON token interpretation and publishes no JSON value; clauses `IRIS-V1-LIBRARY-C012` and `IRIS-V1-LIBRARY-C021` apply. | `D-413`, `D-414` |
| `IRIS-V1-LIBRARY-V005` | negative | interpreter required; JIT required; native not applicable; IrisValue header surface | Decode IrisValue fixture `bad-header` with invalid magic `49525630` and fixture `unsupported-version` with format version `2`. | Each fixture fails before payload allocation with status `IRISVALUE_INCOMPATIBLE_HEADER`; clauses `IRIS-V1-LIBRARY-C015`, `IRIS-V1-LIBRARY-C016`, and `IRIS-V1-LIBRARY-C017` apply. | `D-503` |
| `IRIS-V1-LIBRARY-V006` | negative | interpreter required; JIT required; native not applicable; IrisValue length surface | Decode IrisValue fixture `array-count-over-limit`, where declared element count is one above configured limit `1024`. | Fails with status `IRISVALUE_LIMIT_OR_STRUCTURE` before container allocation; no partial Array is published; clauses `IRIS-V1-LIBRARY-C017` and `IRIS-V1-LIBRARY-C035` apply. | `D-503` |
| `IRIS-V1-LIBRARY-V007` | positive | interpreter required; JIT required; native not applicable; IrisValue supported-values surface | Decode then encode IrisValue fixture `roundtrip-core-values` containing arbitrary Integer `18446744073709551617`, String `"é"`, Bytes hex `00ff`, Tuple `(1,"x")`, Array `[1,2]`, and Hash entries `{(:a,1),(:b,2)}`. | Values round-trip with equal Iris value semantics; Hash comparison is by unordered entries; the language chapter asserts no byte tags or field order; clauses `IRIS-V1-LIBRARY-C015` and `IRIS-V1-LIBRARY-C018` apply. | `D-503` |
| `IRIS-V1-LIBRARY-V008` | negative | interpreter required; JIT required; native required; IrisValue encoder surface | Attempt default IrisValue encoding of an `FFI::Library` returned by `FFI.open`, an open `File`, and a native payload object. | Every call raises `SerializationError`; no pointer, runtime identity, file descriptor, native payload bytes, or live resource state is emitted; clauses `IRIS-V1-LIBRARY-C003` and `IRIS-V1-LIBRARY-C018` apply. | `D-502`, `D-503` |
| `IRIS-V1-LIBRARY-V009` | negative | interpreter required; JIT required; native not applicable; Encoding strict UTF-8 surface | `Encoding::UTF_8.decode(Bytes[0xc3,0x28])`. | Raises `EncodingError` under strict default behavior; clauses `IRIS-V1-LIBRARY-C021` and `IRIS-V1-LIBRARY-C022` apply. | `D-413`, `D-414` |
| `IRIS-V1-LIBRARY-V010` | positive | interpreter required; JIT required; native not applicable; Encoding explicit error option surface | `Encoding::UTF_8.decode(Bytes[0xc3,0x28], errors: :replace)`. | Returns String `"�("`; replacement behavior occurs only because the call selected `errors: :replace`; clauses `IRIS-V1-LIBRARY-C022` and `IRIS-V1-LIBRARY-C023` apply. | `D-414` |
| `IRIS-V1-LIBRARY-V011` | negative | interpreter required; JIT required; native not applicable; Encoding selection surface | Call `File.read_text("input.txt", encoding: :host_default)` or equivalent implicit host-default Encoding selection in conforming mode. | Call is rejected with diagnostic `ENCODING_EXPLICIT_REQUIRED`, or the API requires an explicit Encoding object before decoding; clauses `IRIS-V1-LIBRARY-C025` and `IRIS-V1-LIBRARY-C026` apply. | `D-414` |
| `IRIS-V1-LIBRARY-V012` | positive | interpreter required; JIT required; native not applicable; package metadata and Regex dispatch surface | Package map contains core `Regex` and separately versioned `std/regex-pcre@2` exposing Type `PcreRegex`; source evaluates `"abc" =~ /a+/` and separately constructs `PcreRegex.new("(?<=a)b")`. | Core `=~` uses core Regex and returns a core Match or `nil`; `PcreRegex` is distinct and only usable through explicit package API; clauses `IRIS-V1-LIBRARY-C028` and `IRIS-V1-LIBRARY-C034` apply. | `D-504`, `D-505`, `D-506` |
| `IRIS-V1-LIBRARY-V013` | negative | interpreter required; JIT not applicable; native not applicable; package metadata and Regex literal boundary | Package fixture `std/regex-pcre@2` declares `replaces_core_regex_literals: true` and attempts to bind `/.../flags` to `PcreRegex`. | Package validation rejects the replacement claim with status `PACKAGE_CORE_ABI_CLAIM`; core literal semantics remain unchanged; clauses `IRIS-V1-LIBRARY-C028` and `IRIS-V1-LIBRARY-C034` apply. | `D-504`, `D-505`, `D-506` |
| `IRIS-V1-LIBRARY-V014` | negative | interpreter required; JIT not applicable; native not applicable; package metadata boundary | Package fixtures `std/http@1 { core_abi: true }` and `std/crypto@1 { core_abi: true }` are validated against the v1 package map. | Each fixture is rejected with status `PACKAGE_CORE_ABI_CLAIM`; no package is published as language-core ABI; clauses `IRIS-V1-LIBRARY-C028`, `IRIS-V1-LIBRARY-C030`, and `IRIS-V1-LIBRARY-C032` apply. | `D-504` |

## Traceability Notes

IRIS-V1-LIBRARY-C039: This chapter owns the serialization and library-boundary decisions D-502 through D-504. It refines D-413 and D-414 for Encoding and strict UTF-8 conversion, uses revised D-407 through [06-collections-text-regex.md](06-collections-text-regex.md) for Unicode 17.0.0, and cross-links D-505 and D-506 rather than redefining Regex semantics.

IRIS-V1-LIBRARY-C040: Chapter-owned decision IDs are `D-502`, `D-503`, and `D-504`.

IRIS-V1-LIBRARY-C041: Referenced non-owned decision IDs include `D-407`, `D-413`, `D-414`, `D-505`, and `D-506`.
