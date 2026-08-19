# Authored Convenience Standard Library Surface

The methods listed here are authored convenience surface for the current Iris
interpreter. They are not mandated by a frozen specification clause. Chapter 6
defines the Iterable, Iterator, and Iteration traversal protocol, not a
collection or String convenience-method catalog.

## Array

`map`, `each`, `select`, `reject`, `reduce`, `find`, `count`, `sum`, `min`,
`max`, `sort`, `reverse`, `first`, `last`, `push`, `pop`, `join`, `include?`,
`index_of`, `concat`, `slice`, `take`, `drop`, `uniq`, `flatten`, `all?`,
`any?`, `each_with_index`, `at`, and `to_string`.

`size` is deliberately ABSENT: an existing test pins it as `MessageNotFound`
alongside `+`, and `length` is the spelling that exists.

## Hash

`map`, `select`, `keys`, `values`, `merge`, `include?`, `has_key?`, `delete`,
and `to_array`.

## String

`split`, `trim`, `replace`, `starts_with?`, `ends_with?`, `contains?`,
`downcase`, `chars`, and `to_symbol`. `upcase` already existed.

## Note on literal receivers

These methods live in the source runtime, which is the only evaluator carrying
the full built-in selector surface. A send on a LITERAL receiver used to stay in
the literal evaluator, whose selector table is tiny, so `"a".upcase()` answered
`MessageNotFoundError` while `let s = "a"; s.upcase()` succeeded. The router in
`crates/iris-eval/src/lib.rs` now sends literal receivers to the source runtime.
A NAME receiver still recurses, so an undeclared `A.new_current()` keeps
reporting `MessageNotFoundError` rather than a `NameError`.
