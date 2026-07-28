# GRAMMAR Vector Classification

This document classifies the 41 committed `IRIS-V1-GRAMMAR` vectors that appear as rows in the chapter 02 vector tables at `spec/iris-v1/02-lexical-grammar.md:549-554` and `spec/iris-v1/02-lexical-grammar.md:568-606`.

| Vector ID | Chapter 02 line | Category | Applicability | Classification | Concrete front-end artifact |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-GRAMMAR-V003` | `spec/iris-v1/02-lexical-grammar.md:551` | positive | interpreter required; JIT required; native not applicable | executable | Assert parse-shape output for `2 ** 3 ** 2`, `-2 ** 2`, and `2 ** -3` matches the three required associativity strings. |
| `IRIS-V1-GRAMMAR-V010` | `spec/iris-v1/02-lexical-grammar.md:552` | diagnostic | parser required; interpreter not applicable; JIT not applicable; native not applicable | authored-expect | Author the keyword inventory expectation for D-509, including reserved keywords and historical non-keyword identifier handling. |
| `IRIS-V1-GRAMMAR-V011` | `spec/iris-v1/02-lexical-grammar.md:553` | positive | parser required; interpreter optional; JIT optional; native not applicable | prose-fixture | Assert contextual tokenization for ranges, member access, hash literal opening, interpolation, generic closers, Regex opening, and literal prefixes. |
| `IRIS-V1-GRAMMAR-V013` | `spec/iris-v1/02-lexical-grammar.md:554` | diagnostic | parser required; interpreter not applicable; JIT not applicable; native not applicable | authored-expect | Author the labeled-loop expectation, including accepted `break outer: expr` and rejected non-loop or missing-colon forms. |
| `IRIS-V1-GRAMMAR-V144` | `spec/iris-v1/02-lexical-grammar.md:570` | positive | interpreter required; JIT required; native not applicable | executable | Assert integer literal values `[10, 493, 755, 255]` and `Integer` type observations. |
| `IRIS-V1-GRAMMAR-V145` | `spec/iris-v1/02-lexical-grammar.md:571` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | executable | Assert valid numeric separator tokenization and `LEX_BAD_NUMERIC_SEPARATOR` for each malformed separator fixture. |
| `IRIS-V1-GRAMMAR-V146` | `spec/iris-v1/02-lexical-grammar.md:572` | positive | interpreter required; JIT required; native not applicable | executable | Assert `00755` evaluates to `Integer(755)` rather than octal `493`. |
| `IRIS-V1-GRAMMAR-V147` | `spec/iris-v1/02-lexical-grammar.md:573` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | executable | Assert uppercase radix and exponent spellings parse, while uppercase float suffixes emit `LEX_BAD_FLOAT_SUFFIX`. |
| `IRIS-V1-GRAMMAR-V148` | `spec/iris-v1/02-lexical-grammar.md:574` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | executable | Assert each invalid radix fixture emits `LEX_INVALID_RADIX_DIGIT` as one invalid numeric token. |
| `IRIS-V1-GRAMMAR-V149` | `spec/iris-v1/02-lexical-grammar.md:575` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | executable | Assert empty radix prefixes and separator-after-prefix forms emit `LEX_EMPTY_RADIX_PREFIX` without token splitting. |
| `IRIS-V1-GRAMMAR-V150` | `spec/iris-v1/02-lexical-grammar.md:576` | diagnostic | interpreter required; JIT required; native not applicable | executable | Assert `.5` and `1.` float values, `.` as `DOT`, and separator diagnostics for malformed dot forms. |
| `IRIS-V1-GRAMMAR-V151` | `spec/iris-v1/02-lexical-grammar.md:577` | diagnostic | interpreter required; JIT required; native not applicable | executable | Assert exponent literal values and lexical diagnostics for malformed exponent fixtures. |
| `IRIS-V1-GRAMMAR-V152` | `spec/iris-v1/02-lexical-grammar.md:578` | positive | interpreter required; JIT required; native not applicable | executable | Assert hexadecimal float and integer values `[15.5, 1.0, 1.0, 0.5, 483]` with expected numeric types. |
| `IRIS-V1-GRAMMAR-V153` | `spec/iris-v1/02-lexical-grammar.md:579` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | authored-expect | Author the malformed hex-float expectation for `0x1.` without inventing a diagnostic code beyond the spec text. |
| `IRIS-V1-GRAMMAR-V154` | `spec/iris-v1/02-lexical-grammar.md:580` | differential | interpreter required; JIT required; native not applicable | deferred | Defer differential interpreter and JIT `float32_bits` and `float64_bits` comparison under varied host locales and rounding modes. |
| `IRIS-V1-GRAMMAR-V155` | `spec/iris-v1/02-lexical-grammar.md:581` | diagnostic | interpreter required; JIT required; native not applicable | prose-fixture | Assert signed overflow, zero-rounding, subnormal bit results, and precision-warning presence or absence. |
| `IRIS-V1-GRAMMAR-V161` | `spec/iris-v1/02-lexical-grammar.md:582` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | prose-fixture | Assert canonical headers parse and reordered or repeated headers emit `PARSE_BAD_HEADER_ORDER`. |
| `IRIS-V1-GRAMMAR-V163` | `spec/iris-v1/02-lexical-grammar.md:583` | positive | interpreter not applicable; JIT not applicable; native not applicable | executable | Assert parser output contains constraints `T: A & B` and `U: C`, with comma as assignment separator. |
| `IRIS-V1-GRAMMAR-V165` | `spec/iris-v1/02-lexical-grammar.md:584` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | executable | Assert header `meta deny shape` parses and body placement emits `PARSE_BAD_HEADER_ORDER` with no declaration candidate. |
| `IRIS-V1-GRAMMAR-V167` | `spec/iris-v1/02-lexical-grammar.md:585` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | executable | Assert `PARSE_LEGACY_LEADING_SEMICOLON` and `PARSE_EMPTY_STATEMENT` for the listed semicolon fixtures. |
| `IRIS-V1-GRAMMAR-V168` | `spec/iris-v1/02-lexical-grammar.md:586` | positive | interpreter required; JIT required; native not applicable | prose-fixture | Assert statement boundaries, parenthesized newline continuation, escaped newline continuation, and trailing semicolon acceptance. |
| `IRIS-V1-GRAMMAR-V170` | `spec/iris-v1/02-lexical-grammar.md:587` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | executable | Assert comment removal for line, doc, and nested block comments, plus `LEX_UNTERMINATED_COMMENT` for the malformed fixture. |
| `IRIS-V1-GRAMMAR-V171` | `spec/iris-v1/02-lexical-grammar.md:588` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | executable | Assert first-line shebang acceptance and `LEX_SHEBANG_NOT_FIRST` for a later shebang. |
| `IRIS-V1-GRAMMAR-V172` | `spec/iris-v1/02-lexical-grammar.md:589` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | prose-fixture | Assert valid UTF-8 BOM parsing and `LEX_INVALID_UTF8` for malformed UTF-8 and UTF-16LE BOM bytes. |
| `IRIS-V1-GRAMMAR-V173` | `spec/iris-v1/02-lexical-grammar.md:590` | positive | interpreter required; JIT required; native not applicable | executable | Assert string-family values `"a\n2"`, `"a\n${x}"`, `"a\\n${x}"`, and `"a"`, all as `String`. |
| `IRIS-V1-GRAMMAR-V174` | `spec/iris-v1/02-lexical-grammar.md:591` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | executable | Assert the valid escape scalar sequence and `LEX_BAD_ESCAPE` for bad escape and invalid Unicode scalar fixtures. |
| `IRIS-V1-GRAMMAR-V175` | `spec/iris-v1/02-lexical-grammar.md:592` | diagnostic | interpreter required; JIT required; native not applicable | prose-fixture | Assert triple literal value `"alpha\nbeta"` and `LEX_BAD_MULTILINE_INDENT` for bad indentation. |
| `IRIS-V1-GRAMMAR-V176` | `spec/iris-v1/02-lexical-grammar.md:593` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | prose-fixture | Assert tabs are not normalized to spaces and the mixed-indent fixture emits `LEX_BAD_MULTILINE_INDENT`. |
| `IRIS-V1-GRAMMAR-V177` | `spec/iris-v1/02-lexical-grammar.md:594` | diagnostic | interpreter required; JIT required; native not applicable | prose-fixture | Assert raw literal preservation for zero and nonzero fences, plus `LEX_BAD_RAW_FENCE` for a 256-fence fixture. |
| `IRIS-V1-GRAMMAR-V178` | `spec/iris-v1/02-lexical-grammar.md:595` | positive | interpreter required; JIT required; native not applicable | executable | Assert raw triple value `"${x}\\n"` after strict indent stripping with no escape or interpolation evaluation. |
| `IRIS-V1-GRAMMAR-V179` | `spec/iris-v1/02-lexical-grammar.md:596` | positive | interpreter required; JIT required; native not applicable | executable | Assert adjacent literal concatenation yields `String("abcd")` without a statement terminator between segments. |
| `IRIS-V1-GRAMMAR-V189` | `spec/iris-v1/02-lexical-grammar.md:597` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | prose-fixture | Assert `"\xFF"` yields text `String("U+00FF")` and surrogate escape emits `LEX_BAD_ESCAPE`. |
| `IRIS-V1-GRAMMAR-V190` | `spec/iris-v1/02-lexical-grammar.md:598` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | authored-expect | Author the interpolation expectation, including `String("3")` for the expression form and parse rejection for the format mini-language. |
| `IRIS-V1-GRAMMAR-V181` | `spec/iris-v1/02-lexical-grammar.md:599` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | authored-expect | Author the Regex literal and diagnostic expectations for supported flags, duplicate flags, and unsupported backreference syntax. |
| `IRIS-V1-GRAMMAR-V183` | `spec/iris-v1/02-lexical-grammar.md:600` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | executable | Assert historical words parse as identifiers while `let class = 1` is rejected as a reserved-keyword binding. |
| `IRIS-V1-GRAMMAR-V184` | `spec/iris-v1/02-lexical-grammar.md:601` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | executable | Assert conflict tokenization for each listed source, including `LEX_INTERPOLATION_OUTSIDE_LITERAL` and division for `a / b`. |
| `IRIS-V1-GRAMMAR-V187` | `spec/iris-v1/02-lexical-grammar.md:602` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | executable | Assert `ready?` and `save!` selectors parse, while `ready?!` and `done?` local binding are rejected. |
| `IRIS-V1-GRAMMAR-V188` | `spec/iris-v1/02-lexical-grammar.md:603` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | executable | Assert listed compound assignments parse, `%=` is rejected, and logical assignment remains reserved core syntax. |
| `IRIS-V1-GRAMMAR-V191` | `spec/iris-v1/02-lexical-grammar.md:604` | diagnostic | interpreter required; JIT required; native not applicable | executable | Assert `<BS><LF>` continuation evaluates `sum` to `Integer(3)` and bad continuations emit `LEX_BAD_CONTINUATION`. |
| `IRIS-V1-GRAMMAR-V192` | `spec/iris-v1/02-lexical-grammar.md:605` | positive | interpreter not applicable; JIT not applicable; native not applicable | executable | Assert Contract declaration parsing with ordered parent Type list `ParentA`, `ParentB`. |
| `IRIS-V1-GRAMMAR-V193` | `spec/iris-v1/02-lexical-grammar.md:606` | positive | interpreter not applicable; JIT not applicable; native not applicable | executable | Assert implicit-root and explicit-`Object` Class headers parse as distinct source forms. |

## Classification Totals

| Classification | Count |
| --- | ---: |
| executable | 26 |
| prose-fixture | 9 |
| authored-expect | 5 |
| deferred | 1 |

## Prose Fixture Reclassification

Nine rows were originally classified `executable` but their frozen chapter 02 rows supply prose instead of an executable fixture or an executable expectation. They are reclassified `prose-fixture`, carry the record tag `status:prose-fixture`, and are reported by the runner in its own `unrunnable_source` bucket. They are never counted as passing.

| Vector ID | Why it is not executable |
| --- | --- |
| `IRIS-V1-GRAMMAR-V011` | `expect.artifact.parse_shapes` is an English sentence describing conformant tokenization, not a renderable shape. |
| `IRIS-V1-GRAMMAR-V155` | `input.source_text` is entirely prose describing rounding fixtures. |
| `IRIS-V1-GRAMMAR-V161` | `input.source_text` appends a prose description of the reordered-header fixture to the code prefix. |
| `IRIS-V1-GRAMMAR-V168` | `expect.artifact.parse_shapes` lists behavior labels such as `newline-after-1-terminates`, not shapes. |
| `IRIS-V1-GRAMMAR-V172` | `input.source_text` describes byte sequences in prose rather than supplying them. |
| `IRIS-V1-GRAMMAR-V175` | `input.source_text` describes the invalid closing-indent fixture in prose. |
| `IRIS-V1-GRAMMAR-V176` | `input.source_text` is entirely prose describing indentation. |
| `IRIS-V1-GRAMMAR-V177` | `input.source_text` describes the 256-fence invalid fixture in prose. |
| `IRIS-V1-GRAMMAR-V189` | `expect.value.string` is the notation `U+00FF`; the implementation correctly evaluates `"\xFF"` to `ÿ`. |

The underlying frozen-spec defect is recorded in [`docs/spec-defects-v1.md`](../../docs/spec-defects-v1.md). Authoring concrete fixtures for these rows is future corpus work and is out of milestone 1 scope.

## Out Of Milestone Scope

The six prose-only vectors at `spec/iris-v1/02-lexical-grammar.md:532-542` are out of milestone scope: `IRIS-V1-GRAMMAR-V001`, `IRIS-V1-GRAMMAR-V002`, `IRIS-V1-GRAMMAR-V004`, `IRIS-V1-GRAMMAR-V005`, `IRIS-V1-GRAMMAR-V006`, and `IRIS-V1-GRAMMAR-V007`.

All ID gaps are also out of milestone scope: `V004`-`V009`, `V012`, `V014`-`V143`, `V156`-`V160`, `V162`, `V164`, `V166`, `V180`, `V182`, and `V185`-`V186`.
