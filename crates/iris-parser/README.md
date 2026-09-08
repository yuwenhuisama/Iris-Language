# Parser Layout And Limits

`parse(&str) -> ParseResult` remains the public parsing entry point. Token text
comes from the lexer's exclusive byte spans. Newlines remain statement boundaries
unless the current grammar production requires continuation, or an expression is
inside a delimited list. Statement bodies suspend the surrounding list context.

`else`, `catch`, `finally`, and an explicit raise's `from` can attach across
newlines, but not across semicolons. Closure headers retain the shipped parser's
acceptance of an omitted terminator as compatibility input; canonical formatter
output uses an explicit semicolon. Property accessor members still require
explicit semicolons. Bodyless contract
method requirements remain bodyless across a following method's newline.

Parsing untrusted input has implementation resource limits. At most 32 guarded
recursive production frames and 256 expression parsing units are permitted per
outer expression. Token advancement, including speculative parsing, has a budget
of 64 times the token count plus 1024. Exceeding a limit rejects the program with
`PARSE_RESOURCE_LIMIT`; a partial AST must not be executed. These are resource
bounds, not new language grammar restrictions. Ordinary long sequences of
independent statements do not share the expression-unit bound.

The existing syntax tree does not model stored-property accessor metadata or
method/type-alias `where` clauses. This layout repair does not add those semantic
representations; property accessor validation preserves the existing AST shape.

Run parser-only verification with `cargo test -p iris-parser`. The integration
tests exercise the public API against the real lexer, compare compact and laid
out ASTs, and isolate hostile inputs in bounded child processes.

## Source Metadata

`parse_with_source(&str) -> source::SourceParse` uses the same grammar as `parse`.
Its `parse` field is the ordinary `ParseResult`; `source` is a `SourceDocument`.
`parse_editor` returns the same shape, additionally retaining EOF-terminated
bodies and incomplete `receiver.` expressions. Diagnostics remain rejecting:
never execute an editor recovery AST.

`SourceDocument::node(SyntaxId)` and `scope(ScopeId)` address document-local
arenas. Each node has a half-open UTF-8 byte `span`, lexical `scope`, explicit
`children`, and a typed `SourceKind`. IDs are not stable across reparses.
`roots` records source productions once, not the AST's duplicated Program
collections. `tokens` preserves the original lexer kinds and ends, including
contextually split `>>` and `||`. `protected` conservatively includes literal
tokens and non-whitespace trivia gaps; it is not a comment classification API.

Declaration facts carry exact name/path sites, activation offset, modifiers,
visibility/surface, written annotation, initializer, parameters/defaults, and
written return type. An absent return annotation is absent metadata, not an
inferred signature (TYPES-C003). Deferred-binding and closure-parameter types
survive here even where the existing runtime AST discards them. Type nodes
retain the normalized AST type plus source-only child name/type sites; optional
`T?` does not invent a source occurrence of `Nil`.

Expression facts link names, literals, member receivers, calls and their ordered
arguments, assignments, grouping, generic construction, reified types, closures,
and keyword arguments. Other expression forms explicitly report `Unsupported`
with production children, rather than claiming an inferred type. Imports retain
per-segment targets, per-spec names, aliases, and replacement authorization.

`SourceDeclaration.return_hint_offset` is the byte immediately after a Method's
parameter-list closing `)`, before trivia or a written return annotation. It is
present for complete Method signatures (including bodyless requirements and
annotated methods), absent for other declarations. Consumers should display an
omitted-return hint only when `return_type` is also absent.

Nominal declarations now carry `header: Option<DeclarationHeader>`. Its `span`
ends at the opening body brace, excluding that brace. `complete` means the
header reached this brace without parse damage, independent of body recovery.
Presence flags are `has_extends`, `has_implements` (the `for` clause, including
the parser-accepted but statically invalid Module form), `has_mixins`,
`has_type_parameters`, `has_constraints`, `has_meta_policy`, and `has_decorators`.
These are conservative suppression facts, not inheritance resolution. Analysis
that only handles simple declarations must reject unsupported flagged shapes,
reopen modifiers, incomplete headers, and damaged scopes before inferring
construction or member results. An all-false header is not a semantic validity
proof. Individual type child edges still lack composition-role labels.

`ImportFact.separators` retains grammar-consumed `Dot` / `Qualified` separators
and their exact half-open byte spans, in source order. For accepted imports it
has `target.len() - 1` entries: separator `i` precedes target segment `i + 1`.
Thus `org.dep::Core` differs from `org::dep::Core`. A dotted prefix becomes a
package qualifier only if it reaches a qualified separator; a dotted-only path
must not be guessed into a package. Malformed wildcard imports can retain a
terminal separator without a target segment and remain rejecting input.

Plain `parse` leaves all source-document arenas unallocated. Token edit journals
remain active in both modes because restoring contextual generic/closure tokens
is required for grammar correctness, not just metadata.

Resolution consumers must honor `visible_from` and scope ancestry, and stop or
mark uncertainty at `Scope::damaged` / `recovery` barriers. Method scopes identify
the callable boundary; consumers must apply method non-capture rules themselves.
The graph records syntax, not resolved symbols or inferred types.

Current conservative limits: malformed headers and lexer failures do not retain
partial declarations; recovery is not arbitrary token insertion. Property
accessor metadata is still not modeled. Class-header type children preserve
source facts but are not labeled by extends/implements/mixin role. Destructuring
patterns do not model extraction types or alternative-binding unification.
Control-flow expression result inference is unsupported. Recovery preserves
successful preceding statements only inside retained bodies or at document
level, and removes failed productions rather than emitting phantom symbols.
The existing large `lib.rs` and `expression.rs` production modules remain large;
this stage isolates the recorder, graph contract, and fact builders but does not
complete a production-module decomposition. The Class declaration production
is extracted to `class_declaration.rs`. Top-level decorator applications
are separate roots preceding their declaration; consumers should not assume
the declaration node's span includes those decorators.

Run `cargo run -p iris-parser --example source_graph` for a parser-only driver.

### Documentation And Signature Sites

`iris_lexer::lex_with_comments` opts into scanner-classified outer comments;
`lex` and `lex_type` do not allocate comment storage. `Comment` carries
`kind: CommentKind`, `offset: ByteOffset`, and exclusive `end: ByteOffset`.
Kinds are `Line`, `Block`, `DocumentationLine`, and `DocumentationBlock`.
Nested block contents and markers inside literals are never separate comments.

The following additive `SourceDocument` vectors are public through `source`:

- `comments: Vec<iris_lexer::Comment>` preserves classified source spans.
- `documentation: Vec<Documentation>` stores `declaration: SyntaxId`, `span`,
  UTF-8 `text` bounded to 2048 bytes, and `truncated`. Attachments use production
  ownership, including decorators and wrapped exported declarations, not names.
  Only standalone immediately preceding doc lines or one block attach. Blank
  lines, ordinary comments, intervening statements, trailing comments, and parse
  damage break attachment. Decorator gaps follow the same rule. Line markers
  remove one optional space; block cleanup preserves paragraphs and removes
  conventional leading stars only. No Markdown interpretation is performed.
- `parameter_slots: Vec<ParameterSlot>` is ordered by production encounter.
  Filter by `owner: SyntaxId` for a callable's written parameter order. Slots
  have `span`, `name: NameSite` (including `_`), `category: ParameterCategory`,
  optional named `declaration`, optional `annotation`, and optional `default`
  expression IDs. `_` creates no symbol. Existing named `parameters` edges
  remain unchanged. Closure bodies have a `Body` child, isolating header facts.
- `signatures: Vec<SignatureSite>` carries callable `owner`, written header
  `span`, and optional `return_type`, captured before parsing the body.
- `calls: Vec<CallSite>` carries `call: SyntaxId`, `open: Span`, optional `close`,
  this call's `commas`, ordered `arguments: Vec<ArgumentSlot>`, bounded `end`,
  and `incomplete`. Each argument has `span`, `kind: ArgumentKind`, optional
  `expression`, and `incomplete`. Kinds are `Positional`, `Keyword(NameSite)`,
  and `TrailingBlock`. Trailing commas before an actual `)` add no empty slot;
  missing arguments at an editor boundary do. Nested separators belong only to
  the production that consumed them. The call vector is completion order, not
  source order; use the node graph for ancestry.

`parse_editor` additionally retains calls ending at EOF or an existing closer,
including unfinished expressions and nested calls. Retained partial arguments
have no complete expression ID. Recovery never consumes an enclosing `}` or
`]`; diagnostics remain rejecting. Reserved `key:` can be retained as a damaged
editor keyword slot but is not accepted grammar. Arbitrary malformed headers,
lexical errors, and arbitrary token insertion remain unsupported. Failed parent
productions can remove their complete subtree. Checkpoints and arena truncation
roll back all source sidecars. Plain `parse` leaves all new vectors unallocated.

Run `cargo run -p iris-parser --example source_metadata` for a documentation,
parameter-slot, and incomplete-call driver.
