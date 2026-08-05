# Errata draft: IRIS-V1-GRAMMAR-C071 — the `await` production

Status: awaiting owner approval. Nothing published.

## The gap

`await` is a reserved keyword. `IRIS-V1-GRAMMAR-C013` fixes it in the v1 reserved
inventory, and the async chapter uses it throughout:

- `IRIS-V1-ASYNC-C008`: "The awaited type of `await expr` is the `T` from
  `Awaitable<T>`."
- `IRIS-V1-ASYNC-C013`: awaiting an incomplete Awaitable suspends the Task.
- `IRIS-V1-META-C037`: a transaction body "MUST NOT `await`", with static
  violations being compile errors and dynamic ones raising
  `MetaTransactionError`.

But chapter 02 contains no production for it. `unary_expr`, `postfix_expr` and
`primary_expr` admit no `await`, so no conforming implementation can parse the
spelling three clauses require, and `IRIS-V1-META-V355` and `V431` cannot be
authored at all.

This is the same class of gap as `IRIS-V1-TYPES-C099` and
`IRIS-V1-CONTROL-C080`: a clause forbids or types an operation the grammar
never gave a spelling.

## Proposed clause

`IRIS-V1-GRAMMAR-C071`: The v1.32 errata supplies the production for the `await`
operator that `IRIS-V1-ASYNC-C008` and `IRIS-V1-META-C037` already presuppose.
`unary_expr ::= "await" unary_expr | ("+" | "-" | "~" | "!") unary_expr |
exponent_expr`. The operand is a `unary_expr`, so `await` binds tighter than
every binary operator and looser than a postfix call, making `await f()` an
await of the call's result rather than a call on an awaited callee. This clause
supplies SYNTAX only: the awaited type is owned by `IRIS-V1-ASYNC-C008`,
suspension by `C013`, and the transaction prohibition by `IRIS-V1-META-C037`,
none of which this clause changes.

## Scope: what this milestone implements

The clause supplies the production. This milestone implements only what
`IRIS-V1-META-V355` and `V431` observe, which is the `IRIS-V1-META-C037`
prohibition:

- a transaction body containing `await` is a STATIC violation and a compile
  error, which is what V355's "await emits a static transaction-suspension
  diagnostic and starts no candidate" states, and
- a dynamic violation raises `MetaTransactionError`, which is V431's
  "attempting `await` in the transform raises `MetaTransactionError` and
  publishes nothing".

The scheduler, `Task<T>`, `Awaitable<T>` and suspension are NOT implemented.
They are chapter 07's own 40 rows, outside this milestone, and neither META row
observes them. An `await` outside a transaction therefore stays unimplemented
rather than being given a placeholder meaning.

## Risk to probe before publication

Whether adding the production changes any currently passing row.

PROBED, and the risk is nil. `await` is already reserved, so `let await = 1`
is rejected today with `PARSE_RESERVED_KEYWORD_BINDING`; no transcribed vector
contains the token at all; and the spelling currently fails with
`PARSE_UNEXPECTED_TOKEN`, so nothing can depend on its present behaviour.
Adding the production can only make previously unparseable source parse.

## Asks

1. Approve publishing `IRIS-V1-GRAMMAR-C071` (EN + zh-cn, v1.31 → v1.32).
2. Confirm the binding is `unary_expr`, or name a different precedence.
