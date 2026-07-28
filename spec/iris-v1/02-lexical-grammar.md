# Iris v1 Lexical Grammar

Status: Iris v1 draft, frozen semantics.

IRIS-V1-GRAMMAR-C001: This chapter defines the normative source encoding, lexical token set, literals, reserved keywords, contextual token rules, precedence, associativity, declaration headers, calls, blocks, and EBNF grammar for Iris v1. Later semantic chapters MUST use the syntax anchors and token names in this chapter.

IRIS-V1-GRAMMAR-C002: Historical files, including `Document/IrisLangHighLight(for NP++).xml` and legacy `.ir` scripts, are evidence only. They MUST NOT add reserved words, operators, literal forms, or parse rules beyond the frozen decisions in the approved draft.

## Source Text

IRIS-V1-GRAMMAR-C003: Iris source text MUST be strict UTF-8. A source file MAY begin with one UTF-8 BOM. Other BOMs, UTF-16, local encodings, and malformed UTF-8 MUST be diagnosed as `LEX_INVALID_UTF8` with byte offset and source position. A lexer MUST NOT replace malformed input with U+FFFD.

IRIS-V1-GRAMMAR-C004: The physical newline sequences LF, CRLF, and isolated CR are recognized as newlines. Formatters SHOULD emit LF. A newline participates in statement termination unless it is removed by explicit continuation or the parser is inside a grammar context that still expects more tokens.

IRIS-V1-GRAMMAR-C005: A physical first line beginning with `#!`, optionally after the UTF-8 BOM, is a shebang comment. `#!` at any other source position MUST be diagnosed as `LEX_SHEBANG_NOT_FIRST`.

IRIS-V1-GRAMMAR-C006: `//` starts an ordinary line comment that ends before the physical newline. `///` starts a documentation line comment. `/*` and `/**` start ordinary and documentation block comments respectively. Block comments MUST nest and MUST end with a matching `*/`. An unterminated block comment MUST be diagnosed as `LEX_UNTERMINATED_COMMENT`.

IRIS-V1-GRAMMAR-C007: A backslash outside a literal is explicit line continuation only when immediately followed by LF, CRLF, or CR. The lexer removes both the backslash and the newline. A backslash followed by spaces, comments, or any other token outside a literal MUST be diagnosed as `LEX_BAD_CONTINUATION`.

IRIS-V1-GRAMMAR-C008: A complete statement MAY end with one trailing semicolon, and a semicolon MAY separate two same-line statements. A standalone semicolon, repeated semicolon, or legacy leading semicolon before a statement MUST be diagnosed as `PARSE_EMPTY_STATEMENT` or `PARSE_LEGACY_LEADING_SEMICOLON` as applicable.

## Identifiers And Names

IRIS-V1-GRAMMAR-C009: Ordinary identifiers are case-sensitive and MUST be normalized to Unicode NFC before name, selector, and Type identity are assigned. Identifier start is Unicode XID_Start or `_`. Identifier continuation is Unicode XID_Continue or `_`. Pattern_Syntax, Pattern_White_Space, controls, and default-ignorable code points are forbidden in identifiers even when otherwise classified by Unicode.

IRIS-V1-GRAMMAR-C010: Callable and property selector identifiers MAY end in exactly one `?` or `!`. That suffix is part of selector identity. Ordinary local names, parameter names, constants, Class names, Module names, Contract names, type-parameter names, and raw ivar names MUST NOT use selector suffix punctuation.

IRIS-V1-GRAMMAR-C011: Raw ivar source syntax is `@` followed by an ordinary identifier with no selector suffix. `@@` followed by an ordinary identifier denotes a shared or class-level storage name in grammar positions that allow it. `$` followed by an ordinary identifier denotes a global name in grammar positions that allow it.

IRIS-V1-GRAMMAR-C012: A simple Symbol literal beginning with `:` accepts an ordinary identifier, a selector identifier, an operator symbol listed in IRIS-V1-GRAMMAR-C028, or an ivar name of the form `@` plus ordinary identifier. A quoted Symbol literal `:"..."` uses String escape rules, performs no interpolation, and preserves the post-escape scalar sequence exactly without identifier NFC normalization.

## Reserved Keywords

IRIS-V1-GRAMMAR-C013: The v1 reserved keyword set contains exactly 48 lowercase words. A conforming lexer MUST emit a keyword token for these spellings only when the complete normalized identifier text equals the keyword.

| Keyword     | Keyword      | Keyword      | Keyword       |
| ----------- | ------------ | ------------ | ------------- |
| `class`   | `module`   | `contract` | `open`      |
| `extends` | `for`      | `mixin`    | `where`     |
| `meta`    | `deny`     | `public`   | `protected` |
| `private` | `override` | `impl`     | `property`  |
| `shared`  | `key`      | `async`    | `await`     |
| `fun`     | `let`      | `mut`      | `const`     |
| `global`  | `import`   | `from`     | `as`        |
| `export`  | `type`     | `if`       | `else`      |
| `while`   | `in`       | `break`    | `continue`  |
| `match`   | `try`      | `catch`    | `finally`   |
| `raise`   | `return`   | `is`       | `nil`       |
| `true`    | `false`    | `self`     | `super`     |

IRIS-V1-GRAMMAR-C014: The words `and`, `or`, `not`, `repeat`, `switch`, `when`, `groan`, `order`, `serve`, `ignore`, `defer`, `implements`, `satisfies`, `interface`, `goto`, `retry`, `redo`, `static`, `alias`, and `undef` are not reserved by history alone. `new`, `using`, `close`, and `method_missing` are ordinary Method names. A lexer MUST classify these words as identifiers unless another rule in this specification later gives them a contextual role.

## Fixed Tokens

IRIS-V1-GRAMMAR-C015: The lexer MUST recognize the following 69 fixed token names. Some names share a spelling in different grammar contexts, such as division slash and Regex opener slash. For multi-character fixed tokens, longest match applies before shorter tokens are emitted.

| Token name              | Spelling | Rule                                                                                               |
| ----------------------- | -------- | -------------------------------------------------------------------------------------------------- |
| `LPAREN`              | `(`    | Opens parameter, grouping, call, and tuple contexts.                                               |
| `RPAREN`              | `)`    | Closes`LPAREN`.                                                                                  |
| `LBRACKET`            | `[`    | Opens index, slice, and Array literal contexts.                                                    |
| `RBRACKET`            | `]`    | Closes`LBRACKET`.                                                                                |
| `LBRACE`              | `{`    | Opens blocks, declaration bodies, Closure literals, and Hash-like bodies after contextual openers. |
| `RBRACE`              | `}`    | Closes`LBRACE`.                                                                                  |
| `COMMA`               | `,`    | Separates list elements.                                                                           |
| `SEMICOLON`           | `;`    | Separates or trails statements under IRIS-V1-GRAMMAR-C008.                                         |
| `COLON`               | `:`    | Starts Symbol literals or separates labels, types, keys, and constraints by grammar context.       |
| `DOUBLE_COLON`        | `::`   | Forms qualified names and declaration slots.                                                       |
| `DOT`                 | `.`    | Starts property/member postfix syntax or participates in numeric literal recognition.              |
| `CONTRACT_VIEW`       | `..`   | Starts Contract-view postfix syntax in primary context.                                            |
| `HASH_OPEN`           | `%{`   | Opens a Hash literal.                                                                              |
| `INTERPOLATION_OPEN`  | `${`   | Opens interpolation only inside interpolated String or Regex text.                                 |
| `ARROW`               | `->`   | Separates callable header from return type.                                                        |
| `MATCH_ARROW`         | `=>`   | Separates match arm pattern from body.                                                             |
| `AT`                  | `@`    | Starts a raw ivar token when followed by an ordinary identifier.                                   |
| `DOUBLE_AT`           | `@@`   | Starts a shared storage token when followed by an ordinary identifier.                             |
| `DOLLAR`              | `$`    | Starts a global storage token when followed by an ordinary identifier.                             |
| `QUESTION`            | `?`    | Appears only as selector suffix or in contextual`as?`.                                           |
| `BANG`                | `!`    | Prefix logical negation or selector suffix when not part of a longer token.                        |
| `PLUS`                | `+`    | Prefix or binary operator.                                                                         |
| `MINUS`               | `-`    | Prefix or binary operator.                                                                         |
| `STAR`                | `*`    | Binary operator or positional rest marker by grammar context.                                      |
| `STAR_STAR`           | `**`   | Exponent operator.                                                                                 |
| `SLASH`               | `/`    | Division operator or Regex delimiter by grammar context.                                           |
| `AMP`                 | `&`    | Bitwise operator, block-channel marker, or type intersection by grammar context.                   |
| `PIPE`                | `        | `                                                                                                  |
| `CARET`               | `^`    | Bitwise xor operator.                                                                              |
| `TILDE`               | `~`    | Prefix bitwise-not operator.                                                                       |
| `LT_LT`               | `<<`   | Left-shift operator.                                                                               |
| `GT_GT`               | `>>`   | Right-shift operator or two generic closers in type grammar.                                       |
| `LT`                  | `<`    | Relational operator or generic opener in type grammar.                                             |
| `LT_EQ`               | `<=`   | Relational operator.                                                                               |
| `GT`                  | `>`    | Relational operator or generic closer in type grammar.                                             |
| `GT_EQ`               | `>=`   | Relational operator.                                                                               |
| `SPACESHIP`           | `<=>`  | Three-way comparison operator.                                                                     |
| `REGEX_MATCH`         | `=~`   | Regex match operator.                                                                              |
| `REGEX_NOT_MATCH`     | `!~`   | Regex non-match operator.                                                                          |
| `EQ_EQ`               | `==`   | Equality operator.                                                                                 |
| `BANG_EQ`             | `!=`   | Inequality operator.                                                                               |
| `AND_AND`             | `&&`   | Short-circuit logical operator.                                                                    |
| `PIPE_PIPE`           | `        |                                                                                                    |
| `RANGE_INCLUSIVE`     | `..=`  | Inclusive Range operator.                                                                          |
| `RANGE_EXCLUSIVE`     | `..<`  | Exclusive Range operator.                                                                          |
| `ASSIGN`              | `=`    | Assignment operator or setter suffix by declaration context.                                       |
| `PLUS_EQ`             | `+=`   | Compound assignment.                                                                               |
| `MINUS_EQ`            | `-=`   | Compound assignment.                                                                               |
| `STAR_EQ`             | `*=`   | Compound assignment.                                                                               |
| `SLASH_EQ`            | `/=`   | Compound assignment.                                                                               |
| `STAR_STAR_EQ`        | `**=`  | Compound assignment.                                                                               |
| `AMP_EQ`              | `&=`   | Compound assignment.                                                                               |
| `PIPE_EQ`             | `        | =`                                                                                                 |
| `CARET_EQ`            | `^=`   | Compound assignment.                                                                               |
| `LT_LT_EQ`            | `<<=`  | Compound assignment.                                                                               |
| `GT_GT_EQ`            | `>>=`  | Compound assignment.                                                                               |
| `AND_AND_EQ`          | `&&=`  | Logical assignment.                                                                                |
| `PIPE_PIPE_EQ`        | `        |                                                                                                    |
| `AS_QUERY`            | `as?`  | Cast operator recognized as contextual longest match.                                              |
| `RAW_PREFIX`          | `r`    | Literal prefix only when immediately followed by a valid quote or raw fence.                       |
| `MUTABLE_PREFIX`      | `m`    | Mutable literal prefix only in allowed literal prefix sequences.                                   |
| `BYTES_PREFIX`        | `b`    | Bytes literal prefix only in allowed literal prefix sequences.                                     |
| `REGEX_RAW_PREFIX`    | `r/`   | Raw Regex opener in Regex-literal context.                                                         |
| `REGEX_OPEN`          | `/`    | Regex opener in Regex-literal context.                                                             |
| `QUOTE_DOUBLE`        | `"`    | String delimiter in literal context.                                                               |
| `QUOTE_SINGLE`        | `'`    | String delimiter in literal context.                                                               |
| `QUOTE_TRIPLE_DOUBLE` | `"""`  | Multiline String delimiter in literal context.                                                     |
| `QUOTE_TRIPLE_SINGLE` | `'''`  | Multiline String delimiter in literal context.                                                     |
| `BACKSLASH`           | `\`    | Escape introducer inside escaped literals or explicit continuation outside literals.               |

IRIS-V1-GRAMMAR-C016: The fixed expression operator set contains 45 fixed spellings and 47 expression operator forms because unary and binary `+` are distinct forms, and unary and binary `-` are distinct forms. The spellings are: `.`, `..`, `(`, `[`, `**`, `+`, `-`, `~`, `!`, `*`, `/`, `<<`, `>>`, `&`, `^`, `|`, `..=`, `..<`, `<`, `<=`, `>`, `>=`, `<=>`, `=~`, `!~`, `is`, `as`, `as?`, `==`, `!=`, `&&`, `||`, `=`, `+=`, `-=`, `*=`, `/=`, `**=`, `&=`, `|=`, `^=`, `<<=`, `>>=`, `&&=`, and `||=`. Named infix Method syntax is one contextual operator class carried by a selector identifier, not a fixed spelling.

## Longest-Match And Named Conflicts

IRIS-V1-GRAMMAR-C017: The lexer MUST prefer `..=` and `..<` over `..`; `!=` over `!` plus `=`; `!~` over `!` plus `~`; `**=` over `**` plus `=`; `<<=` over `<<` plus `=`; `>>=` over `>>` plus `=`; `&&=` over `&&` plus `=`; `||=` over `||` plus `=`; `%{` over `%` plus `{`; `${` over `$` plus `{` inside interpolated literal text; and all other longest fixed tokens over shorter prefixes.

IRIS-V1-GRAMMAR-C018: `..identifier` is Contract-view postfix syntax only after a primary expression. `..=` and `..<` are Range operators only between expressions. A token sequence that cannot satisfy either context MUST be diagnosed as `PARSE_BAD_DOT_DOT_CONTEXT` rather than reparsed as two dots.

IRIS-V1-GRAMMAR-C019: In expression grammar, `a != b` is inequality. In property declaration grammar, a setter selector MAY include a selector suffix before `=`, including `ready?=` and `value!=`. The parser MUST use declaration or property-assignment context to recognize complete setter selectors and MUST NOT change the expression meaning of `!=`.

IRIS-V1-GRAMMAR-C020: Generic angle brackets are recognized only in declaration and type grammar contexts. Expression `<`, `>`, `<=`, `>=`, `<<`, and `>>` retain operator tokenization. Type grammar MUST allow `>>` to close two nested generic argument lists without changing expression right-shift tokenization.

IRIS-V1-GRAMMAR-C021: `%{` is the only Hash literal opener. `%` alone is not a v1 operator. `${` is meaningful only inside interpolated String or Regex text. Outside such text, `${` MUST be diagnosed as `LEX_INTERPOLATION_OUTSIDE_LITERAL`.

IRIS-V1-GRAMMAR-C022: Literal prefixes are contextual. `m`, `b`, and `r` remain ordinary identifiers unless immediately followed by a legal literal prefix sequence and delimiter. Mutable String raw prefixes use `mr`. Bytes prefixes use `b` or `br`. ByteArray prefixes use `mb` or `mbr`. Regex raw prefix uses `r/`. The sequences `rm`, `bm`, `rb`, `rbm`, and `brm` are invalid literal prefixes when followed by a literal delimiter.

IRIS-V1-GRAMMAR-C023: Regex literal recognition is contextual. A slash starts a Regex literal only in expression-start positions where a primary expression is expected. In expression-continuation positions, slash is division. Regex literal text MUST end at an unescaped slash with valid trailing flags.

## Numeric Literals

IRIS-V1-GRAMMAR-C024: Integer literals support binary `0b` or `0B`, octal `0o` or `0O`, decimal without prefix, and hexadecimal `0x` or `0X`. Leading zeros in unprefixed decimal integers are decimal and MUST NOT select octal.

IRIS-V1-GRAMMAR-C025: Numeric digit separators use a single underscore only between two digits valid within the same lexical digit segment. Separators at a boundary, adjacent to a radix prefix, decimal point, exponent marker, exponent sign, or float suffix, and consecutive separators MUST be diagnosed as `LEX_BAD_NUMERIC_SEPARATOR`.

IRIS-V1-GRAMMAR-C026: After a radix prefix begins a candidate numeric literal, the lexer MUST consume the contiguous candidate segment for validation. `0b102`, `0o89`, and `0xFG` each form one invalid numeric literal token and MUST be diagnosed with a radix-specific `LEX_INVALID_RADIX_DIGIT`.

IRIS-V1-GRAMMAR-C027: Decimal floating literals MAY omit digits on either side of the decimal point if at least one side has digits. `.5`, `1.`, `1e3`, and `2E-4` are floating literals. A lone `.` is punctuation. A decimal exponent marker `e` or `E` MUST be followed by an optional single sign and at least one decimal digit.

IRIS-V1-GRAMMAR-C028: Hexadecimal floating literals use a `0x` or `0X` hexadecimal significand and a mandatory `p` or `P` binary exponent with decimal exponent digits and optional sign. The significand may be `0x1p0`, `0x1.p0`, or `0x.8p0`, but it MUST contain at least one hexadecimal digit. `0x1.` is invalid, and `0x1e3` is a hexadecimal Integer.

IRIS-V1-GRAMMAR-C029: Unsuffixed floating literals have type `Float64`. Lowercase suffixes `f32` and `f64` select `Float32` and `Float64`. Suffixes `F32` and `F64` MUST be diagnosed as `LEX_BAD_FLOAT_SUFFIX`.

IRIS-V1-GRAMMAR-C030: Decimal and hexadecimal source floating literals MUST be converted to their target float width with correctly rounded IEEE-754 `roundTiesToEven` semantics. Literal conversion MUST NOT depend on host locale, host floating environment, current rounding mode, interpreter versus JIT mode, target platform, or an inaccurate host parser.

IRIS-V1-GRAMMAR-C031: Floating literal overflow produces signed infinity and sufficiently small nonzero literals may round through subnormal values to signed zero. These are valid literals. The compiler MUST emit a default-enabled precision warning when a finite nonzero source literal rounds to infinity or to zero, and MUST NOT warn only because a finite subnormal is produced.

## Text, Binary, Symbol, Regex, And Collection Literals

IRIS-V1-GRAMMAR-C032: Escaped double-quoted String literals process escapes and `${expr}` interpolation. Escaped single-quoted String literals process escapes and never interpolate. Triple double and triple single quotes are multiline forms with the same interpolation distinction. Adjacent String literal segments concatenate as one expression when statement termination does not intervene.

IRIS-V1-GRAMMAR-C033: Escaped String literals support exactly these escapes: `\\`, `\"`, `\'`, `\n`, `\r`, `\t`, `\0`, `\b`, `\f`, `\v`, `\xNN`, and `\u{H...}` with 1 to 6 hex digits. Unicode escapes MUST denote valid scalar values, excluding surrogates and values above U+10FFFF. Unknown or incomplete escapes MUST be diagnosed as `LEX_BAD_ESCAPE`.

IRIS-V1-GRAMMAR-C034: String values contain only valid Unicode scalars. `\xNN` in a String denotes scalar U+0000 through U+00FF, not a raw byte. Arbitrary binary data belongs in `Bytes` or `ByteArray` literals and APIs.

IRIS-V1-GRAMMAR-C035: Raw String literals use prefix `r` with zero through 255 `#` fence characters before the quote delimiter and the same fence count after the closing delimiter. Raw content performs no escapes and no interpolation. Fence mismatch or more than 255 fence characters MUST be diagnosed as `LEX_BAD_RAW_FENCE`.

IRIS-V1-GRAMMAR-C036: Triple-quoted String literals apply strict closing-indent stripping when the opening triple quote is immediately followed by a newline and the closing delimiter is alone on its line. The first newline is omitted, the immediately preceding final newline is omitted, and the exact whitespace prefix before the closing delimiter is removed from every non-empty content line. A non-empty content line lacking that exact prefix MUST be diagnosed as `LEX_BAD_MULTILINE_INDENT`.

IRIS-V1-GRAMMAR-C037: MutableString literals use prefix `m` before any String literal family. Raw MutableString prefix order is `mr`. `rm` is invalid when followed by a String delimiter. Each evaluation of an `m` literal produces a fresh MutableString identity after applying the underlying String literal rules.

IRIS-V1-GRAMMAR-C038: Bytes literals use `b` plus a String literal family, or `br` plus a raw String literal family. ByteArray literals use `mb` or `mbr` with the same delimiter families. Non-ASCII source text contributes its UTF-8 bytes. In byte literal escape mode, `\xNN` injects one raw byte. Prefix order MUST be mutable `m`, bytes `b`, optional raw `r`; alternatives are invalid.

IRIS-V1-GRAMMAR-C039: Regex literals use `/pattern/flags` or raw `r/pattern/flags`. Interpolated Regex literals accept `${expr}` and convert interpolation values through `to_string` followed by default Regex escaping. Raw Regex literals perform no interpolation. Flags are lexical trailing identifiers composed of ASCII letters and MUST be validated by the Regex chapter.

IRIS-V1-GRAMMAR-C040: Array literals use `[elements]`, Tuple literals use parenthesized comma forms, Hash literals use `%{ entries }`, and Range literals use `expr ..= expr` or `expr ..< expr`. Hash literal keys are ordinary expressions before `:`; a bare identifier key means a binding reference, never an implicit Symbol.

## Operator Precedence And Associativity

IRIS-V1-GRAMMAR-C041: The precedence table is normative and ordered from highest to lowest. Each row is introduced by this clause and MUST be used by parsers to build an unambiguous expression tree.

| Rank | Operators or forms                                                                                        | Associativity                              | Chaining                              |
| ---- | --------------------------------------------------------------------------------------------------------- | ------------------------------------------ | ------------------------------------- |
| 1    | primary, call`()`, index `[]`, property `.`, Contract-view `..`                                   | left postfix                               | chains allowed                        |
| 2    | exponent`**`                                                                                            | right                                      | chains allowed by right associativity |
| 3    | prefix unary`+`, `-`, `~`, `!`                                                                    | prefix                                     | repeats allowed                       |
| 4    | multiplicative`*`, `/`                                                                                | left                                       | chains allowed                        |
| 5    | additive`+`, `-`                                                                                      | left                                       | chains allowed                        |
| 6    | shifts`<<`, `>>`                                                                                      | left                                       | chains allowed                        |
| 7    | bitwise`&`                                                                                              | left                                       | chains allowed                        |
| 8    | bitwise`^`                                                                                              | left                                       | chains allowed                        |
| 9    | bitwise `                                                                                                 | `                                          | left                                  |
| 10   | Range`..=`, `..<`                                                                                     | non-associative                            | no chains                             |
| 11   | relational, type, and Regex`<`, `<=`, `>`, `>=`, `<=>`, `=~`, `!~`, `is`, `as`, `as?` | non-associative                            | no chains                             |
| 12   | equality`==`, `!=`                                                                                    | non-associative                            | no chains                             |
| 13   | named infix Method                                                                                        | left                                       | chains allowed                        |
| 14   | logical`&&`                                                                                             | left                                       | chains allowed                        |
| 15   | logical `                                                                                                 |                                            | `                                     |
| 16   | assignment`=`, `+=`, `-=`, `*=`, `/=`, `**=`, `&=`, `                                       | =`, `^=`, `<<=`, `>>=`, `&&=`, ` |                                       |

IRIS-V1-GRAMMAR-C042: Exponentiation binds more tightly than prefix unary negation. `2 ** 3 ** 2` parses as `2 ** (3 ** 2)`. `-2 ** 2` parses as `-(2 ** 2)`. `2 ** -3` parses as `2 ** (-3)`. A negative base requires parentheses.

IRIS-V1-GRAMMAR-C043: Range, relational, equality, `<=>`, Regex match, `is`, `as`, and `as?` are non-associative. Source such as `a < b < c`, `a == b == c`, `a ..= b ..= c`, or `x as T as U` MUST be diagnosed as `PARSE_NONASSOCIATIVE_CHAIN` unless parentheses or a logical operator make the grouping explicit.

IRIS-V1-GRAMMAR-C044: Named infix Method syntax parses `expression selector expression` at rank 13 only when `selector` is a selector identifier and the surrounding grammar can supply both operands. Method declarations cannot change named infix precedence or associativity.

IRIS-V1-GRAMMAR-C045: `!`, `&&`, `||`, `&&=`, and `||=` are core control-flow syntax and MUST NOT be parsed as overloadable Method selectors. `&`, `|`, `^`, and `~` remain overloadable operator messages.

## EBNF Conventions

IRIS-V1-GRAMMAR-C046: The EBNF in this chapter uses quoted text for fixed tokens, lowercase names for lexical classes, uppercase words for token categories, `?` for optional, `*` for repetition, and `+` for one or more. Each production is complete for parser shape, while runtime meaning belongs to later chapters.

IRIS-V1-GRAMMAR-C047: A `terminator` is a syntax-aware newline, a semicolon allowed by IRIS-V1-GRAMMAR-C008, or end of file. Newlines inside unmatched delimiters or after incomplete grammar productions are not `terminator` tokens.

## Lexical EBNF

IRIS-V1-GRAMMAR-C048: The following lexical productions define identifier, number, and literal token shapes.

IRIS-V1-GRAMMAR-C048A: The lexical body names in this table are primitive scanner classes. They are intentionally not decomposed into parser-visible productions, but each name is defined for grammar-reference accounting and conformance diagnostics.

| Primitive                                    | Definition                                                                                                                              |
| -------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| `lexical_primitive`                        | A scanner-recognized character class or balanced literal body whose detailed validation is defined by the clauses in this chapter.      |
| `xid_start_or_underscore`                  | One Unicode XID_Start scalar or`_`, after applying the exclusions in IRIS-V1-GRAMMAR-C009.                                            |
| `xid_continue_or_underscore`               | One Unicode XID_Continue scalar or`_`, after applying the exclusions in IRIS-V1-GRAMMAR-C009.                                         |
| `binary_digit`                             | `0` or `1`.                                                                                                                         |
| `octal_digit`                              | `0` through `7`.                                                                                                                    |
| `decimal_digit`                            | `0` through `9`.                                                                                                                    |
| `hex_digit`                                | `0` through `9`, `a` through `f`, or `A` through `F`.                                                                       |
| `ascii_letter`                             | `A` through `Z` or `a` through `z`.                                                                                             |
| `not_newline`                              | Any source scalar that is not LF, CR, or the start of CRLF.                                                                             |
| `newline`                                  | LF, CRLF, or isolated CR.                                                                                                               |
| `eof`                                      | End of source file.                                                                                                                     |
| `whitespace`                               | Non-newline token-separating whitespace.                                                                                                |
| `double_string`                            | A complete escaped double-quoted String body, including delimiter validation, escapes, interpolation, and newline restrictions.         |
| `single_string`                            | A complete escaped single-quoted String body, including delimiter validation, escapes, and newline restrictions, with no interpolation. |
| `triple_double_string`                     | A complete triple-double-quoted multiline String body, including interpolation and strict indentation handling.                         |
| `triple_single_string`                     | A complete triple-single-quoted multiline String body, including strict indentation handling and no interpolation.                      |
| `string_double_body_without_interpolation` | The quoted Symbol body accepted by`:"..."`, using String escapes but rejecting interpolation openers.                                 |
| `regex_text`                               | A complete non-raw Regex body with escapes, balanced interpolation, and terminating slash recognition.                                  |
| `raw_regex_text`                           | A complete raw Regex body with terminating slash recognition and no interpolation.                                                      |

```ebnf
source_file        ::= bom? shebang? token_stream eof
bom                ::= "U+FEFF"
shebang            ::= "#!" not_newline* newline
token_stream       ::= (lexical_token | whitespace | newline)*
lexical_token      ::= identifier | ivar_name | shared_name | global_name | symbol_literal | integer_literal | float_literal | string_literal | bytes_literal | regex_literal | array_literal | hash_literal | fixed_token
fixed_token        ::= "fixed token from IRIS-V1-GRAMMAR-C015"
xid_start_or_underscore ::= lexical_primitive
xid_continue_or_underscore ::= lexical_primitive
binary_digit       ::= lexical_primitive
octal_digit        ::= lexical_primitive
decimal_digit      ::= lexical_primitive
hex_digit          ::= lexical_primitive
ascii_letter       ::= lexical_primitive
not_newline        ::= lexical_primitive
newline            ::= lexical_primitive
eof                ::= lexical_primitive
whitespace         ::= lexical_primitive
lexical_primitive  ::= "scanner primitive"
identifier         ::= xid_start_or_underscore xid_continue_or_underscore* selector_suffix?
ordinary_name      ::= xid_start_or_underscore xid_continue_or_underscore*
selector_suffix    ::= "?" | "!"
ivar_name          ::= "@" ordinary_name
shared_name        ::= "@@" ordinary_name
global_name        ::= "$" ordinary_name
symbol_literal     ::= ":" (ordinary_name selector_suffix? | ivar_name | operator_symbol | quoted_symbol)
quoted_symbol      ::= string_double_body_without_interpolation
operator_symbol    ::= "+" | "-" | "*" | "**" | "/" | "&" | "|" | "^" | "~" | "<<" | ">>" | "<" | "<=" | ">" | ">=" | "<=>" | "=~" | "!~" | "==" | "!=" | "same?"

integer_literal    ::= binary_integer | octal_integer | decimal_integer | hexadecimal_integer
binary_integer     ::= ("0b" | "0B") binary_digit digit_sep_binary*
octal_integer      ::= ("0o" | "0O") octal_digit digit_sep_octal*
decimal_integer    ::= decimal_digit digit_sep_decimal*
hexadecimal_integer ::= ("0x" | "0X") hex_digit digit_sep_hex*
float_literal      ::= decimal_float float_suffix? | hex_float float_suffix?
float_suffix       ::= "f32" | "f64"
decimal_float      ::= decimal_point_float decimal_exponent? | decimal_digit digit_sep_decimal* decimal_exponent
decimal_point_float ::= decimal_digit digit_sep_decimal* "." decimal_fraction_digits? | decimal_digit digit_sep_decimal* "." | "." decimal_fraction_digits
decimal_fraction_digits ::= decimal_digit digit_sep_decimal*
decimal_exponent   ::= ("e" | "E") ("+" | "-")? decimal_digit digit_sep_decimal*
hex_float          ::= ("0x" | "0X") hex_significand ("p" | "P") ("+" | "-")? decimal_digit digit_sep_decimal*
hex_significand    ::= hex_digit digit_sep_hex* | hex_digit digit_sep_hex* "." hex_fraction_digits? | hex_digit digit_sep_hex* "." | "." hex_fraction_digits
hex_fraction_digits ::= hex_digit digit_sep_hex*
digit_sep_binary   ::= binary_digit | "_" binary_digit
digit_sep_octal    ::= octal_digit | "_" octal_digit
digit_sep_decimal  ::= decimal_digit | "_" decimal_digit
digit_sep_hex      ::= hex_digit | "_" hex_digit

string_literal     ::= string_prefix? string_body
string_prefix      ::= "r" raw_fence? | "m" | "mr" raw_fence?
bytes_literal      ::= bytes_prefix string_body
bytes_prefix       ::= "b" | "br" raw_fence? | "mb" | "mbr" raw_fence?
raw_fence          ::= "#"{0,255}
string_body        ::= double_string | single_string | triple_double_string | triple_single_string
double_string      ::= lexical_primitive
single_string      ::= lexical_primitive
triple_double_string ::= lexical_primitive
triple_single_string ::= lexical_primitive
string_double_body_without_interpolation ::= lexical_primitive
regex_literal      ::= "/" regex_text "/" regex_flags? | "r" raw_fence? "/" raw_regex_text "/" raw_fence? regex_flags?
regex_text         ::= lexical_primitive
raw_regex_text     ::= lexical_primitive
regex_flags        ::= ascii_letter*
array_literal      ::= "[" argument_list? "]"
argument_list      ::= expression ("," expression)* ","?
hash_literal       ::= "%{" hash_entry_list? "}"
hash_entry_list    ::= hash_entry ("," hash_entry)* ","?
hash_entry         ::= expression ":" expression
```

## Parser EBNF

IRIS-V1-GRAMMAR-C049: The following parser productions define declarations, calls, blocks, statements, and expressions sufficiently for an unambiguous Iris v1 parser.

```ebnf
program            ::= terminator* declaration_or_statement (terminator+ declaration_or_statement)* terminator*
terminator         ::= newline | ";" | eof
declaration_or_statement ::= declaration | statement

declaration        ::= class_decl | module_decl | contract_decl | method_decl | property_decl | import_decl | export_decl | type_alias_decl | global_decl | let_decl
import_decl        ::= "import" qualified_type_name import_alias? | "from" qualified_type_name "import" import_spec_list
import_alias       ::= "as" ordinary_name
import_spec_list   ::= import_spec ("," import_spec)* ","?
import_spec        ::= ordinary_name import_alias?
export_decl        ::= "export" (declaration | ordinary_name ("," ordinary_name)* ","?)
type_alias_decl    ::= "type" type_name generic_params? "=" type_expr where_clause?
global_decl        ::= "global" ("let" | "mut") global_name type_annotation? "=" expression
class_decl         ::= "open"? "class" type_name generic_params? class_extends? class_for? class_mixin? where_clause? meta_clause? declaration_body
class_extends      ::= "extends" type_expr
class_for          ::= "for" type_expr_list
class_mixin        ::= "mixin" type_expr_list
module_decl        ::= "open"? "module" type_name generic_params? module_mixin? where_clause? meta_clause? declaration_body
module_mixin       ::= "mixin" type_expr_list
contract_decl      ::= "contract" type_name generic_params? contract_extends? where_clause? meta_clause? declaration_body
contract_extends   ::= "extends" type_expr_list
meta_clause        ::= "meta" "deny" meta_capability_list
meta_capability_list ::= meta_capability ("," meta_capability)* ","?
meta_capability    ::= ordinary_name
declaration_body   ::= "{" terminator* declaration_or_statement* "}"

method_decl        ::= visibility? "override"? "impl"? "async"? "class"? "fun" selector generic_params? parameter_list return_type? where_clause? block_body
property_decl      ::= visibility? "override"? "impl"? "property" "fun" property_selector parameter_list return_type? where_clause? block_body
property_selector  ::= selector "="?
visibility         ::= "public" | "protected" | "private"
parameter_list     ::= "(" parameter_sequence? ")"
parameter_sequence ::= required_positional* optional_positional* rest_positional? required_keyword* optional_keyword* rest_keyword? block_parameter?
required_positional ::= ordinary_name ":" type_expr ","?
optional_positional ::= ordinary_name ":" type_expr "=" expression ","?
rest_positional    ::= "*" ordinary_name ":" type_expr ","?
required_keyword   ::= "key" ordinary_name ":" type_expr ","?
optional_keyword   ::= "key" ordinary_name ":" type_expr "=" expression ","?
rest_keyword       ::= "**" ordinary_name ":" type_expr ","?
block_parameter    ::= "&" ordinary_name ":" function_type ("=" "nil")? ","?
return_type        ::= "->" type_expr
block_body         ::= "{" terminator* statement_list? "}"

statement_list     ::= statement (terminator+ statement)* terminator*
statement          ::= let_decl | expression_statement | return_statement | break_statement | continue_statement | raise_statement | if_statement | while_statement | for_statement | match_statement | try_statement
let_decl           ::= ("let" | "mut" | "const") binding_pattern type_annotation? ("=" expression)?
type_annotation    ::= ":" type_expr
expression_statement ::= expression
return_statement   ::= "return" expression?
break_statement    ::= "break" break_payload?
break_payload      ::= labeled_break_payload | expression
labeled_break_payload ::= ordinary_name ":" expression
continue_statement ::= "continue" ordinary_name?
raise_statement    ::= "raise" | "raise" expression raise_cause?
raise_cause        ::= "from" expression
if_statement       ::= "if" expression block_body ("else" (if_statement | block_body))?
while_statement    ::= loop_label? "while" expression block_body
for_statement      ::= loop_label? "for" binding_pattern "in" expression block_body
loop_label         ::= ordinary_name ":"
match_statement    ::= "match" expression "{" terminator* match_arm+ match_fallback? "}"
match_arm          ::= match_pattern match_guard? "=>" match_body match_separator
match_fallback     ::= "else" "=>" match_body match_separator?
match_guard        ::= "if" expression
match_body         ::= expression | block_body
match_separator    ::= "," terminator* | terminator+
try_statement      ::= "try" block_body catch_clause* finally_clause?
catch_clause       ::= "catch" (catch_binding (":" type_expr)? ("," ordinary_name)?)? block_body
finally_clause     ::= "finally" block_body

expression         ::= assignment_expr
assignment_expr    ::= assignment_target assignment_operator assignment_expr | logical_or_expr
assignment_operator ::= "=" | "+=" | "-=" | "*=" | "/=" | "**=" | "&=" | "|=" | "^=" | "<<=" | ">>=" | "&&=" | "||="
assignment_target  ::= ordinary_name | ivar_name | shared_name | global_name | member_assignment_target | index_assignment_target
member_assignment_target ::= postfix_expr property_suffix
index_assignment_target ::= postfix_expr index_suffix
logical_or_expr    ::= logical_and_expr ("||" logical_and_expr)*
logical_and_expr   ::= named_infix_expr ("&&" named_infix_expr)*
named_infix_expr   ::= equality_expr (selector equality_expr)*
equality_expr      ::= relational_expr (("==" | "!=") relational_expr)?
relational_expr    ::= range_expr (("<" | "<=" | ">" | ">=" | "<=>" | "=~" | "!~" | "is" | "as" | "as?") range_expr)?
range_expr         ::= bitwise_or_expr (("..=" | "..<") bitwise_or_expr)?
bitwise_or_expr    ::= bitwise_xor_expr ("|" bitwise_xor_expr)*
bitwise_xor_expr   ::= bitwise_and_expr ("^" bitwise_and_expr)*
bitwise_and_expr   ::= shift_expr ("&" shift_expr)*
shift_expr         ::= additive_expr (("<<" | ">>") additive_expr)*
additive_expr      ::= multiplicative_expr (("+" | "-") multiplicative_expr)*
multiplicative_expr ::= unary_expr (("*" | "/") unary_expr)*
unary_expr         ::= ("+" | "-" | "~" | "!") unary_expr | exponent_expr
exponent_expr      ::= postfix_expr ("**" unary_expr)?
postfix_expr       ::= primary_expr postfix_part*
postfix_part       ::= call_suffix | index_suffix | property_suffix | contract_view_suffix | trailing_block
call_suffix        ::= "(" call_argument_list? ")"
call_argument_list ::= call_argument ("," call_argument)* ","?
call_argument      ::= expression | ordinary_name ":" expression | "*" expression | "**" expression | "&" expression
index_suffix       ::= "[" call_argument_list? "]"
property_suffix    ::= "." selector
contract_view_suffix ::= ".." selector call_suffix?
trailing_block     ::= closure_literal
primary_expr       ::= literal | ordinary_name | ivar_name | shared_name | global_name | "self" | "super" | "nil" | "true" | "false" | grouped_or_tuple | array_literal | hash_literal | closure_literal
grouped_or_tuple   ::= "(" expression ("," expression)* ","? ")"
closure_literal    ::= "{" closure_header? closure_body "}"
closure_header     ::= "|" closure_parameters? "|" return_type? (terminator | ";")
closure_parameters ::= parameter_sequence
closure_body       ::= statement_list?

generic_params     ::= "<" ordinary_name ("," ordinary_name)* ">"
where_clause       ::= "where" constraint_assignment ("," constraint_assignment)*
constraint_assignment ::= (ordinary_name | "Self") ":" type_expr
type_expr_list     ::= type_expr ("," type_expr)*
type_expr          ::= type_union
type_union         ::= type_intersection ("|" type_intersection)*
type_intersection  ::= type_postfix ("&" type_postfix)*
type_postfix       ::= type_primary "?"?
type_primary       ::= qualified_type_name generic_args? | function_type | "(" type_expr ")"
qualified_type_name ::= type_name ("::" type_name)*
generic_args       ::= "<" type_expr ("," type_expr)* ">"
function_type      ::= "(" type_expr_list? ")" "->" type_expr
binding_pattern    ::= ordinary_name | "_" | "(" binding_pattern ("," binding_pattern)* ","? ")" | "[" binding_pattern ("," binding_pattern)* rest_binding_pattern? ","? "]"
rest_binding_pattern ::= "," "*" binding_pattern
catch_binding      ::= ordinary_name | "_"
pattern            ::= binding_pattern | literal | type_pattern
match_pattern      ::= match_pattern_alternative ("|" match_pattern_alternative)*
match_pattern_alternative ::= literal | "nil" | "true" | "false" | "is" match_type_expr binding_pattern? | binding_pattern | match_tuple_pattern | match_array_pattern
match_type_expr    ::= type_intersection | type_primary
match_tuple_pattern ::= "(" match_pattern ("," match_pattern)+ ","? ")"
match_array_pattern ::= "[" match_pattern ("," match_pattern)* match_rest_pattern? ","? "]"
match_rest_pattern ::= "," "*" binding_pattern
type_pattern       ::= type_expr
selector           ::= ordinary_name selector_suffix?
type_name          ::= ordinary_name
literal            ::= integer_literal | float_literal | string_literal | bytes_literal | symbol_literal | regex_literal
```

IRIS-V1-GRAMMAR-C050: Ordinary calls require parentheses. A property getter omits parentheses only in property/member syntax. A trailing Closure follows a completed call or postfix expression and binds through the dedicated `&block` channel. A call MUST contain at most one block, either `&expr` in the argument list or one trailing Closure.

IRIS-V1-GRAMMAR-C051: Closure literals use `{ |parameters| -> ReturnType body }`. Empty parameters use `{ || -> R body }`. A single-line Closure body begins after `;`; a multiline body begins after the header terminator. The return annotation MAY be omitted only when an expected callable context uniquely supplies a closed type.

IRIS-V1-GRAMMAR-C052: Class header clause order is fixed as generic parameters, `extends`, `for`, `mixin`, `where`, `meta`, then body. Module header clause order is generic parameters, `mixin`, `where`, `meta`, then body. Contract header clause order is generic parameters, `extends`, `where`, `meta`, then body. Each allowed clause kind MAY appear at most once. The `meta deny` clause is static header syntax only and MUST NOT appear in executable or conditional body code. The `meta` clause's capability names are parsed as ordinary names whose vocabulary and deny semantics are specified by the metaprogramming chapter.

IRIS-V1-GRAMMAR-C053: Parameter declaration order is required positional, optional positional, at most one positional rest, required keyword-only, optional keyword-only, at most one keyword rest, then trailing-block binding. Calls spell keyword arguments as `name: value` and MUST use parentheses for ordinary call syntax.

IRIS-V1-GRAMMAR-C053A: Loop labels are source-expressible only as `ordinary_name :` immediately before `while` or `for`. A label prefix MUST NOT apply to non-loop statements. `break` alone is bare break. Unlabeled break uses ordinary `break expression`. Labeled break uses `break ordinary_name : expression`, as in `break outer: value`; the colon after the label makes it syntactically distinct from unrestricted `break expr`, including named-infix break values. Parsing MUST NOT consult declared label names or any symbol table. `continue name` is the labeled continue form, and bare `continue` targets the nearest loop. Label target validity, duplicate-label errors, Closure-boundary restrictions, and result typing are control-flow semantics.

IRIS-V1-GRAMMAR-C053B: Match arms use `pattern [if guard] => expression-or-block` with source-order arms and an optional final `else => expression-or-block` fallback. Top-level `|` in `match_pattern` separates pattern alternatives. An `is` pattern uses `match_type_expr`, which excludes top-level union type syntax; a union type in an `is` pattern MUST be parenthesized as `is (A | B)` when a union type is intended. A fallback arm MUST be last because `else` is parsed only by `match_fallback`, not by `match_pattern`. Arm separators are comma and/or statement terminators as defined by `match_separator`. Exhaustiveness, guard truth testing, binding compatibility, and destructuring failure behavior are control-flow semantics.

## Diagnostics For Malformed Tokens And Syntax

IRIS-V1-GRAMMAR-C054: A conforming diagnostic system MUST classify malformed lexical and grammar cases using at least these stable categories. Tools MAY add more detail, but the primary category MUST remain stable for conformance vectors.

| Category                              | Required trigger                                                                                                          |
| ------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `LEX_INVALID_UTF8`                  | Malformed UTF-8, wrong BOM, or non-UTF-8 source encoding.                                                                 |
| `LEX_SHEBANG_NOT_FIRST`             | `#!` outside the first physical line.                                                                                   |
| `LEX_UNTERMINATED_COMMENT`          | Nested block comment reaches EOF without a matching close.                                                                |
| `LEX_BAD_CONTINUATION`              | Backslash outside a literal is not immediately followed by a newline sequence.                                            |
| `LEX_INVALID_RADIX_DIGIT`           | Candidate radix literal contains a digit or identifier character invalid for that radix.                                  |
| `LEX_EMPTY_RADIX_PREFIX`            | Radix prefix has no immediately valid digit.                                                                              |
| `LEX_BAD_NUMERIC_SEPARATOR`         | Numeric separator placement violates IRIS-V1-GRAMMAR-C025.                                                                |
| `LEX_BAD_FLOAT_SUFFIX`              | Float suffix is not lowercase`f32` or `f64`.                                                                          |
| `LEX_BAD_ESCAPE`                    | Escaped literal contains an unknown, incomplete, or invalid scalar escape.                                                |
| `LEX_UNTERMINATED_LITERAL`          | String, Bytes, ByteArray, Symbol, or Regex literal reaches EOF or invalid newline before its close.                       |
| `LEX_BAD_RAW_FENCE`                 | Raw fence count mismatches or exceeds 255.                                                                                |
| `LEX_BAD_MULTILINE_INDENT`          | Triple literal strict closing-indent stripping cannot be applied.                                                         |
| `LEX_INTERPOLATION_OUTSIDE_LITERAL` | `${` appears outside interpolated literal text.                                                                         |
| `LEX_BAD_LITERAL_PREFIX`            | Literal prefix order or family is invalid, such as`rm`, `bm`, `rb`, or `brm`.                                     |
| `LEX_BAD_REGEX_FLAGS`               | Regex flags contain invalid characters or duplicate unsupported flags.                                                    |
| `PARSE_EMPTY_STATEMENT`             | Standalone or repeated semicolon creates an empty statement.                                                              |
| `PARSE_LEGACY_LEADING_SEMICOLON`    | Statement begins with legacy leading semicolon.                                                                           |
| `PARSE_BAD_DOT_DOT_CONTEXT`         | `..` cannot serve as Contract view and cannot be a Range token.                                                         |
| `PARSE_NONASSOCIATIVE_CHAIN`        | Non-associative operators are chained without parentheses or logical combination.                                         |
| `PARSE_BAD_GENERIC_CONTEXT`         | Generic angle brackets appear where expression operators are required, or`>>` cannot close the expected type arguments. |
| `PARSE_BAD_CALL_BLOCK`              | A call supplies more than one block channel.                                                                              |
| `PARSE_BAD_HEADER_ORDER`            | Class, Module, or Contract header clauses are out of order or repeated.                                                   |
| `PARSE_BAD_PARAMETER_ORDER`         | Parameter categories are out of order or repeated beyond allowed rest positions.                                          |

## Positive Examples And Malformed Vectors

IRIS-V1-GRAMMAR-EX001: Informative example, numeric precedence:

```iris
value = -2 ** 2 + 3 << 1
same = 2 ** -3
```

IRIS-V1-GRAMMAR-EX002: Informative example, declarations and calls:

```iris
class Box<T> extends Object for Printable mixin Trace where T: Object {
  property fun ready?() -> Bool { true }
  property fun value!=(next: T) -> Nil { @value = next }
  fun map<U>(value: T, key label: Symbol, &block: (T) -> U) -> U where U: Object {
    block(value)
  }
}
```

IRIS-V1-GRAMMAR-EX003: Informative example, literals and contextual conflicts:

```iris
name = :@slot
text = m"hello ${user}"
bytes = mbr#"\xff"#
range = 0 ..< 10
match = /item-${name}/i
table = %{ :name: name, 1 + 2: "sum" }
view = parser..parse(input)
```

IRIS-V1-GRAMMAR-V001: Positive vector `grammar.keyword.inventory` expects exactly 48 reserved keywords from IRIS-V1-GRAMMAR-C013 and rejects the historical non-keywords in IRIS-V1-GRAMMAR-C014.

IRIS-V1-GRAMMAR-V002: Positive vector `grammar.operator.inventory` expects 69 fixed token names from IRIS-V1-GRAMMAR-C015, 45 fixed expression operator spellings and 47 expression operator forms from IRIS-V1-GRAMMAR-C016, and one contextual named-infix operator class.

IRIS-V1-GRAMMAR-V004: Positive vector `grammar.conflict.longest-match` classifies `a ..= b`, `a ..< b`, `view..member`, `a != b`, `property fun value!=(v: T)`, `Box<Array<String>>`, `%{}`, `${x}` inside an interpolated literal, `m"x"`, `br"x"`, `mbr"x"`, and `/x/` in expression-start context according to IRIS-V1-GRAMMAR-C017 through IRIS-V1-GRAMMAR-C023.

IRIS-V1-GRAMMAR-V005: Malformed vector `grammar.bad-numeric` maps `0b102` to `LEX_INVALID_RADIX_DIGIT`, `0x` to `LEX_EMPTY_RADIX_PREFIX`, `1__0` to `LEX_BAD_NUMERIC_SEPARATOR`, `1e+_3` to `LEX_BAD_NUMERIC_SEPARATOR`, and `1.0F32` to `LEX_BAD_FLOAT_SUFFIX`.

IRIS-V1-GRAMMAR-V006: Malformed vector `grammar.bad-literals` maps `"\q"` to `LEX_BAD_ESCAPE`, `r###"x"##` to `LEX_BAD_RAW_FENCE`, `rm"x"` to `LEX_BAD_LITERAL_PREFIX`, an unterminated `/abc` Regex literal to `LEX_UNTERMINATED_LITERAL`, and `${x}` outside literal text to `LEX_INTERPOLATION_OUTSIDE_LITERAL`.

IRIS-V1-GRAMMAR-V007: Malformed vector `grammar.bad-parse` maps `;return nil` to `PARSE_LEGACY_LEADING_SEMICOLON`, `a < b < c` to `PARSE_NONASSOCIATIVE_CHAIN`, `x as T as U` to `PARSE_NONASSOCIATIVE_CHAIN`, `class A for C extends B {}` to `PARSE_BAD_HEADER_ORDER`, and `fun f(key x: T, y: T) -> Nil {}` to `PARSE_BAD_PARAMETER_ORDER`.


## Grammar Coverage Vectors

IRIS-V1-GRAMMAR-C057: The following vectors are normative traceability vectors with concrete source input and expected parse or diagnostic observations.

| Vector ID | Category | Applicability | Source/Input | Expected observable | Decisions |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-GRAMMAR-V003` | positive | interpreter required; JIT required; native not applicable | Iris source: `2 ** 3 ** 2; -2 ** 2; 2 ** -3`. | Parse shapes are `2 ** (3 ** 2)`, `-(2 ** 2)`, and `2 ** (-3)`; exponentiation is right-associative and binds more tightly than unary negation. | `D-033` |
| `IRIS-V1-GRAMMAR-V010` | diagnostic | parser required; interpreter not applicable; JIT not applicable; native not applicable | Reserved keyword inventory plus historical words `alias`, `switch`, `when`, `and`, `or`, `not`, `undef`. | Exactly D-509 keywords are reserved; historical non-keywords are ordinary identifiers or rejected only by context-specific grammar. | `D-509` |
| `IRIS-V1-GRAMMAR-V011` | positive | parser required; interpreter optional; JIT optional; native not applicable | `a ..= b`, `a ..< b`, `view..member`, `%{k: v}`, `${x}` inside interpolation, `Box<Array<String>>`, `/x/`, `m"x"`, `br"x"`, `mbr"x"`. | Contextual longest-match tokenization matches the clause-specific interpretation for every listed conflict. | `D-510` |
| `IRIS-V1-GRAMMAR-V013` | diagnostic | parser required; interpreter not applicable; JIT not applicable; native not applicable | `outer: while ready { break outer: 1 }`, `break outer 1`, `label: return 1`, `continue outer`. | Labeled loop forms and `break outer: expr` parse exactly; non-loop labels and missing colon forms are rejected. | `D-440` |

## Traceability Notes

IRIS-V1-GRAMMAR-C055: This chapter consolidates D-033 through D-050, D-183, D-277 through D-283, D-294 through D-300 where they define source header syntax, D-336 through D-386, D-395, D-408, D-417 through D-424, D-461, D-463, and D-505 through D-510. Later revised wording in those decisions wins over older historical syntax evidence.

IRIS-V1-GRAMMAR-C056: Covered decision IDs are `D-033`, `D-034`, `D-035`, `D-036`, `D-037`, `D-038`, `D-039`, `D-040`, `D-041`, `D-042`, `D-043`, `D-044`, `D-045`, `D-046`, `D-047`, `D-048`, `D-049`, `D-050`, `D-183`, `D-277`, `D-278`, `D-279`, `D-280`, `D-281`, `D-282`, `D-283`, `D-294`, `D-295`, `D-296`, `D-297`, `D-298`, `D-299`, `D-300`, `D-336`, `D-337`, `D-338`, `D-339`, `D-340`, `D-341`, `D-342`, `D-343`, `D-344`, `D-345`, `D-346`, `D-347`, `D-348`, `D-349`, `D-350`, `D-351`, `D-352`, `D-354`, `D-355`, `D-356`, `D-357`, `D-358`, `D-359`, `D-360`, `D-361`, `D-362`, `D-363`, `D-364`, `D-365`, `D-366`, `D-367`, `D-368`, `D-369`, `D-370`, `D-371`, `D-372`, `D-373`, `D-374`, `D-375`, `D-376`, `D-377`, `D-378`, `D-379`, `D-380`, `D-381`, `D-382`, `D-383`, `D-384`, `D-385`, `D-386`, `D-388`, `D-395`, `D-408`, `D-417`, `D-418`, `D-419`, `D-420`, `D-421`, `D-422`, `D-423`, `D-424`, `D-461`, `D-463`, `D-505`, `D-506`, `D-507`, `D-508`, `D-509`, and `D-510`.

IRIS-V1-GRAMMAR-N001: Historical note: the Notepad++ highlighter lists legacy words and rotate operators such as `interface`, `groan`, `order`, `serve`, `ignore`, `alias`, `retry`, `redo`, `goto`, `static`, `<<<`, and `>>>`. This chapter records them only as migration evidence and does not reserve them for Iris v1.

## Concrete Grammar Coverage Vectors

Each row names concrete source text or bytes and the exact parser, lexer, or runtime observation required by its cited decision.

| Vector ID | Category | Applicability | Source/Input | Expected observable | Decisions |
| --- | --- | --- | --- | --- | --- |
| `IRIS-V1-GRAMMAR-V144` | positive | interpreter required; JIT required; native not applicable | Iris source: `[0b1010, 0o755, 00755, 0xFF]`. | Values are `[10, 493, 755, 255]`, each has type `Integer`; unprefixed `00755` is decimal. | `D-039`, `D-041` |
| `IRIS-V1-GRAMMAR-V145` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | Source cases: `1_000`, `0xFF_FF`, `1.234_567`, `1e1_000`, `1.0e+1_024`; malformed `_1`, `1_`, `1__0`, `0x_FF`, `1_.0`, `1._0`, `1e_3`, `1e+_3`, `1_f32`. | The five valid literals tokenize as numbers; every malformed case emits `LEX_BAD_NUMERIC_SEPARATOR` in lexical phase. | `D-040` |
| `IRIS-V1-GRAMMAR-V146` | positive | interpreter required; JIT required; native not applicable | Iris source: `00755`. | The literal evaluates to `Integer(755)`, never octal `493`. | `D-041` |
| `IRIS-V1-GRAMMAR-V147` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | Source cases: `0B1010`, `0O755`, `0Xff`, `1E3`, `1.0F32`, `1.0F64`. | The first four parse as `Integer(10)`, `Integer(493)`, `Integer(255)`, and `Float64(1000.0)`; each uppercase suffix emits `LEX_BAD_FLOAT_SUFFIX`. | `D-042` |
| `IRIS-V1-GRAMMAR-V148` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | Source cases: `0b102`, `0o89`, `0xFG`. | Each contiguous candidate is one invalid numeric token and emits `LEX_INVALID_RADIX_DIGIT`; it is not split into a number plus identifier. | `D-043` |
| `IRIS-V1-GRAMMAR-V149` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | Source cases: `0x`, `0X`, `0b`, `0B`, `0o`, `0O`, `0x_FF`, `0b_1010`, `0o_755`. | Every case emits `LEX_EMPTY_RADIX_PREFIX` as one invalid numeric token, without a zero-plus-identifier recovery. | `D-044` |
| `IRIS-V1-GRAMMAR-V150` | diagnostic | interpreter required; JIT required; native not applicable | Iris source values `.5` and `1.`; source cases `.`, `._5`, `1._0`, `1_.`. | `.5` is `Float64(0.5)` and `1.` is `Float64(1.0)`; `.` is `DOT`; the remaining three cases emit `LEX_BAD_NUMERIC_SEPARATOR`. | `D-045` |
| `IRIS-V1-GRAMMAR-V151` | diagnostic | interpreter required; JIT required; native not applicable | Iris source values `1e3`, `2E-4`, `1e+3f32`; source cases `1e`, `1e+`, `1e_3`. | Values are `Float64(1000.0)`, `Float64(0.0002)`, and `Float32(1000.0)`; malformed exponents emit `LEX_BAD_NUMERIC_SEPARATOR` or lexical exponent diagnostic before execution. | `D-046` |
| `IRIS-V1-GRAMMAR-V152` | positive | interpreter required; JIT required; native not applicable | Iris source: `[0x1.fp3, 0x1p0, 0x1.p0, 0x.8p0, 0x1e3]`. | Values are `[15.5, 1.0, 1.0, 0.5, 483]`; the first four floating values have type `Float64`, and `0x1e3` has type `Integer`. | `D-047`, `D-048` |
| `IRIS-V1-GRAMMAR-V153` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | Source case: `0x1.`. | The candidate emits a lexical malformed-hex-float diagnostic and does not tokenize as `Integer(1)` followed by `DOT`. | `D-048` |
| `IRIS-V1-GRAMMAR-V154` | differential | interpreter required; JIT required; native not applicable | Source literals are the decimal and hexadecimal midpoint fixtures for `Float32` and `Float64`, run with varied host locales and host rounding modes. | Interpreter and JIT report identical `float32_bits` and `float64_bits` for every fixture, each equal to the required round-ties-to-even result. | `D-049` |
| `IRIS-V1-GRAMMAR-V155` | diagnostic | interpreter required; JIT required; native not applicable | Finite nonzero decimal and hexadecimal fixtures that round to `+infinity`, `-infinity`, a subnormal, `+0.0`, and `-0.0` at both float widths. | Overflow and zero-rounding fixtures produce their signed IEEE values and one default-enabled precision warning; subnormal fixtures produce their exact nonzero bits with no warning. | `D-050` |
| `IRIS-V1-GRAMMAR-V161` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | Source pairs: `class A extends Object for C mixin M where T: Object meta deny shape {}` and reordered Class, Module, and Contract headers. | Each canonical header parses; every reordered or repeated header emits `PARSE_BAD_HEADER_ORDER`. | `D-280`, `D-281` |
| `IRIS-V1-GRAMMAR-V163` | positive | interpreter not applicable; JIT not applicable; native not applicable | Source: `class Pair<T, U> where T: A & B, U: C {}`. | The parser produces two constraint assignments, `T: A & B` and `U: C`; the comma is not an intersection operator. | `D-282` |
| `IRIS-V1-GRAMMAR-V165` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | Valid source: `class A meta deny shape {}`; invalid source: `class A { if true { meta deny shape } }`. | The header form parses; the body form emits `PARSE_BAD_HEADER_ORDER` and creates no declaration candidate. | `D-290`, `D-298` |
| `IRIS-V1-GRAMMAR-V167` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | Source cases: `;return nil`, `;`, and `x;;y`. | `;return nil` emits `PARSE_LEGACY_LEADING_SEMICOLON`; standalone and repeated semicolons emit `PARSE_EMPTY_STATEMENT`. | `D-362`, `D-364` |
| `IRIS-V1-GRAMMAR-V168` | positive | interpreter required; JIT required; native not applicable | Iris source uses `let a = 1\nlet b = (1\n+ 2)\nlet c = 1 \\\n+ 2\nlet d = 3;`. | The newline after `1` terminates its statement, the newline inside parentheses and escaped newline do not, and one trailing semicolon is accepted. | `D-363` |
| `IRIS-V1-GRAMMAR-V170` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | Source includes `// note`, `/// docs`, and nested `/* outer /* inner */ outer */`; a second fixture is `/* outer /* inner */`. | The first fixture tokenizes without comments; the second emits `LEX_UNTERMINATED_COMMENT`. | `D-365` |
| `IRIS-V1-GRAMMAR-V171` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | One UTF-8 source fixture begins `#! /usr/bin/env iris\nlet x = 1`; another contains `let x = 1\n#! late`. | The first fixture accepts the first-line shebang as a comment; the second emits `LEX_SHEBANG_NOT_FIRST`. | `D-366` |
| `IRIS-V1-GRAMMAR-V172` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | Byte fixtures: UTF-8 BOM plus `let cafe = "ok"`; malformed UTF-8 `0xC3 0x28`; and UTF-16LE BOM `0xFF 0xFE`. | The valid fixture parses; malformed UTF-8 and UTF-16 each emit `LEX_INVALID_UTF8` with no replacement-character token. | `D-367` |
| `IRIS-V1-GRAMMAR-V173` | positive | interpreter required; JIT required; native not applicable | Iris source: `["a\\n${1 + 1}", 'a\\n${x}', r#"a\\n${x}"#, """\n  a\n  """]`. | Values are `["a\n2", "a\n${x}", "a\\n${x}", "a"]`, all type `String`; escaped, single, raw, and multiline families are distinct as specified. | `D-368` |
| `IRIS-V1-GRAMMAR-V174` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | Source uses `"\\\\\\\"\\'\\n\\r\\t\\0\\b\\f\\v\\xFF\\u{1F600}"`; malformed cases use `"\\q"`, `"\\u{D800}"`, and `"\\u{110000}"`. | The valid literal has the stated scalar sequence; each malformed case emits `LEX_BAD_ESCAPE`. | `D-369` |
| `IRIS-V1-GRAMMAR-V175` | diagnostic | interpreter required; JIT required; native not applicable | Valid triple source is `"""\n  alpha\n  beta\n  """`; malformed fixture has a non-empty line without the closing indent. | The valid value is `"alpha\nbeta"`; the malformed fixture emits `LEX_BAD_MULTILINE_INDENT`. | `D-379` |
| `IRIS-V1-GRAMMAR-V176` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | Triple literal has a closing indent of two spaces and a content line prefixed by one tab plus two spaces. | The line does not match the exact space prefix and emits `LEX_BAD_MULTILINE_INDENT`; tabs are not normalized to spaces. | `D-380` |
| `IRIS-V1-GRAMMAR-V177` | diagnostic | interpreter required; JIT required; native not applicable | Source literals: `r"a\\n${x}"`, `r##"a\\n${x}"##`, and a fixture with 256 fence characters. | Matching zero and nonzero fences preserve literal backslashes and `${x}`; the 256-fence fixture emits `LEX_BAD_RAW_FENCE`. | `D-381` |
| `IRIS-V1-GRAMMAR-V178` | positive | interpreter required; JIT required; native not applicable | Raw triple source: `r"""\n  ${x}\\n\n  """`. | The result is `"${x}\\n"`; strict indent stripping occurs while interpolation and escapes remain raw text. | `D-382` |
| `IRIS-V1-GRAMMAR-V179` | positive | interpreter required; JIT required; native not applicable | Iris source: `"a" 'b' r"c" """d"""`. | The expression yields `String("abcd")`; no statement terminator occurs between the literal segments. | `D-386` |
| `IRIS-V1-GRAMMAR-V189` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | Source values: `"\xFF"`; source case: `"\u{D800}"`. | The first value is `String("U+00FF")`, not `Bytes`; the surrogate escape emits `LEX_BAD_ESCAPE`. | `D-370` |
| `IRIS-V1-GRAMMAR-V190` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | Source values: `"${1 + 2}"`; source case: `"${value:04d}"`. | The first interpolation accepts the full expression and yields `String("3")`; the format mini-language form is rejected in parse phase. | `D-383` |
| `IRIS-V1-GRAMMAR-V181` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | Source cases: `/a+/i`, `r/a\\/b/u`, `/a/ii`, and `/(a)\\1/`. | The first two tokenize as Regex literals with their listed flags; duplicate flags and unsupported backreference syntax emit the Regex lexical or validation diagnostic required by the Regex clauses. | `D-505`, `D-506` |
| `IRIS-V1-GRAMMAR-V183` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | Source declares identifiers `alias`, `switch`, `when`, `and`, `or`, `not`, and attempts `let class = 1`. | Historical words are ordinary identifiers; `class` is a reserved-keyword token and the binding is rejected in parse phase. | `D-509` |
| `IRIS-V1-GRAMMAR-V184` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | Source cases: `a ..= b`, `a ..< b`, `view..member`, `a != b`, `property fun ready?=(v: Bool) {}`, `Box<Array<String>>`, `%{k: v}`, `${x}`, `m"x"`, `br"x"`, `mbr"x"`, and `a / b`. | Each case receives the context-specific tokenization prescribed by C017-C023; `${x}` outside a literal emits `LEX_INTERPOLATION_OUTSIDE_LITERAL` and `a / b` uses division rather than Regex opening. | `D-510` |
| `IRIS-V1-GRAMMAR-V187` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | Source declares `fun ready?() { true }`, `fun save!() { nil }`, then attempts `fun ready?!() { nil }` and `let done? = true`. | The first two selectors parse with distinct identities from `ready` and `save`; the malformed selector and suffixed local name are rejected in parse phase. | `D-342` |
| `IRIS-V1-GRAMMAR-V188` | diagnostic | interpreter not applicable; JIT not applicable; native not applicable | Source assignments use `+=`, `-=`, `*=`, `/=`, `**=`, `&=`, `&#124;=`, `^=`, `<<=`, `>>=`, and invalid `%=`, with a separate `&&=` and `&#124;&#124;=` control fixture. | Every listed compound operator parses; `%= ` is rejected because `%` is not an Iris v1 operator, while logical assignment remains reserved core syntax. | `D-348` |
| `IRIS-V1-GRAMMAR-V191` | diagnostic | interpreter required; JIT required; native not applicable | Source-text fixtures use `<BS><LF>` for a backslash byte followed immediately by LF: `let sum = 1 + <BS><LF>2`, `let bad = 1 + <BS><SP><LF>2`, and `let also_bad = 1 + <BS><SP>// note<LF>2`. | The first source evaluates `sum` as `Integer(3)` after the lexer removes `<BS><LF>`. The latter two each emit `LEX_BAD_CONTINUATION` before parsing or execution. | `D-388` |
| `IRIS-V1-GRAMMAR-V192` | positive | interpreter not applicable; JIT not applicable; native not applicable | Source: `contract Child extends ParentA, ParentB {}`. | The parser accepts a Contract declaration with an `extends` clause containing the ordered parent Type list `ParentA`, `ParentB`; it does not parse this form as Module composition. | `D-277` |
| `IRIS-V1-GRAMMAR-V193` | positive | interpreter not applicable; JIT not applicable; native not applicable | Source pair: `class ImplicitRoot {}` and `class ExplicitRoot extends Object {}`. | Both declarations parse as Class headers; the first has an absent `extends` clause and the second has explicit `Object`, preserving the two distinct source forms required for the type layer's identical-root rule. | `D-283` |
