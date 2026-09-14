# AST Async Continuation Prerequisite

Async methods and async closures allocate their Task before executing the eager
prefix. The evaluator retains an AST work stack, local scopes, shared mutable
cells, expression operands, receiver, lexical dispatch context, source, exception
context, loop cursors, and cleanup extents. Gate and pending Task completions
resume that state; completed statements are not replayed. No VM compilation or
fallback is used.

Supported suspension positions include nested ordinary expression operands,
assignments, bindings, returns, raises, conditionals, while/for loops, match
guards, catch/finally bodies, and the standard `using` block. Suspension does not
close resources or run finally. Task failures retain their exception contexts.
Synchronous decorator Next scopes use the evaluator's current Task identity.

This is not the complete async decorator adapter implementation. Async wrapper
chains and an async closure passed as the standard `using` block are not yet
supported. Suspension escaping an ordinary synchronous callback invoked by a
native helper, or an interpolation evaluated by the legacy synchronous string
walker, is explicitly refused rather than replayed. Generators retain their
existing, separate implementation. Return annotations and full async-closure
argument binding retain the existing evaluator behavior.

The Host driver reports `UnsupportedConstruct` when its requested Task remains
pending and no completion is queued. Hosts can post another Gate completion and
drive again; the continuation remains registered.
