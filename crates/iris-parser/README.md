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
