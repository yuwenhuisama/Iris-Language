use super::*;
use BuiltinType::{Array, Bool, Hash, Integer, Nil, Range, String};

pub(super) const ROWS: &[BuiltinMember] = &[
    row!(
        Array,
        Instance,
        "length",
        shapes![Both []],
        Known(Integer),
        "Number of elements.",
        "crates/iris-eval/src/source_runtime.rs:11084; crates/iris-vm/src/machine/stdlib/array.rs:17"
    ),
    row!(
        Array,
        Instance,
        "count",
        shapes![Both [], Both [CALLBACK]],
        Known(Integer),
        "Counts all elements or truthy callback(element) results.",
        "crates/iris-eval/src/source_runtime.rs:4624; crates/iris-vm/src/machine/stdlib/array.rs:76"
    ),
    row!(
        Array,
        Instance,
        "empty?",
        shapes![Reference []],
        Known(Bool),
        "Whether empty; reference only.",
        "crates/iris-eval/src/source_runtime.rs:11111"
    ),
    row!(
        Array,
        Instance,
        "to_array",
        shapes![Both []],
        Known(Array),
        "Fresh independent Array copy.",
        "crates/iris-eval/src/source_runtime.rs:4493; crates/iris-vm/src/machine/stdlib/array.rs:20"
    ),
    row!(
        Array,
        Instance,
        "reverse",
        shapes![Both []],
        Known(Array),
        "Fresh reversed Array.",
        "crates/iris-eval/src/source_runtime.rs:4646; crates/iris-vm/src/machine/stdlib/array.rs:159"
    ),
    row!(
        Array,
        Instance,
        "sort",
        shapes![Both []],
        Known(Array),
        "Fresh Array sorted by element comparison.",
        "crates/iris-eval/src/source_runtime.rs:4645; crates/iris-vm/src/machine/stdlib/array.rs:109"
    ),
    row!(
        Array,
        Instance,
        "uniq",
        shapes![Both []],
        Known(Array),
        "Fresh Array with equal duplicates removed.",
        "crates/iris-eval/src/source_runtime.rs:4728; crates/iris-vm/src/machine/stdlib/array.rs:197"
    ),
    row!(
        Array,
        Instance,
        "flatten",
        shapes![Both []],
        Known(Array),
        "Fresh flattened Array.",
        "crates/iris-eval/src/source_runtime.rs:4746; crates/iris-vm/src/machine/stdlib/array.rs:198"
    ),
    row!(
        Array,
        Instance,
        "map",
        shapes![Both[CALLBACK]],
        Known(Array),
        "Array of callback(element) results; trailing block lowers to the Closure argument.",
        "crates/iris-eval/src/source_runtime.rs:4562; crates/iris-vm/src/machine/stdlib/array.rs:32"
    ),
    row!(
        Array,
        Instance,
        "select",
        shapes![Both[CALLBACK]],
        Known(Array),
        "Keeps elements with truthy callback(element).",
        "crates/iris-eval/src/source_runtime.rs:4587; crates/iris-vm/src/machine/stdlib/array.rs:50"
    ),
    row!(
        Array,
        Instance,
        "reject",
        shapes![Both[CALLBACK]],
        Known(Array),
        "Keeps elements with falsey callback(element).",
        "crates/iris-eval/src/source_runtime.rs:4587; crates/iris-vm/src/machine/stdlib/array.rs:50"
    ),
    row!(
        Array,
        Instance,
        "each",
        shapes![Both[CALLBACK]],
        Receiver,
        "Calls callback(element); returns receiver.",
        "crates/iris-eval/src/source_runtime.rs:4569; crates/iris-vm/src/machine/stdlib/array.rs:39"
    ),
    row!(
        Array,
        Instance,
        "each_with_index",
        shapes![Both[CALLBACK]],
        Receiver,
        "Calls callback(element, index); returns receiver.",
        "crates/iris-eval/src/source_runtime.rs:4575; crates/iris-vm/src/machine/stdlib/array.rs:39"
    ),
    row!(
        Array,
        Instance,
        "find",
        shapes![Both[CALLBACK]],
        Unknown,
        "First element with truthy callback(element), or nil.",
        "crates/iris-eval/src/source_runtime.rs:4615; crates/iris-vm/src/machine/stdlib/array.rs:61"
    ),
    row!(
        Array,
        Instance,
        "reduce",
        shapes![Both [CALLBACK], Both [ARG1, CALLBACK]],
        Unknown,
        "Calls callback(accumulator, element). Empty unseeded reduction is nil; seeded reduction starts with arg1.",
        "crates/iris-eval/src/source_runtime.rs:4598; crates/iris-vm/src/machine/stdlib/array.rs:85"
    ),
    row!(
        Array,
        Instance,
        "sum",
        shapes![Both []],
        Unknown,
        "Accumulates + from Integer zero; arbitrary elements need not produce Integer.",
        "crates/iris-eval/src/source_runtime.rs:4637; crates/iris-vm/src/machine/stdlib/array.rs:101"
    ),
    row!(
        Array,
        Instance,
        "min",
        shapes![Both []],
        Unknown,
        "Minimum element or nil.",
        "crates/iris-eval/src/source_runtime.rs:4644; crates/iris-vm/src/machine/stdlib/array.rs:108"
    ),
    row!(
        Array,
        Instance,
        "max",
        shapes![Both []],
        Unknown,
        "Maximum element or nil.",
        "crates/iris-eval/src/source_runtime.rs:4644; crates/iris-vm/src/machine/stdlib/array.rs:108"
    ),
    row!(
        Array,
        Instance,
        "first",
        shapes![Both []],
        Unknown,
        "First element or nil.",
        "crates/iris-eval/src/source_runtime.rs:4651; crates/iris-vm/src/machine/stdlib/array.rs:157"
    ),
    row!(
        Array,
        Instance,
        "last",
        shapes![Both []],
        Unknown,
        "Last element or nil.",
        "crates/iris-eval/src/source_runtime.rs:4652; crates/iris-vm/src/machine/stdlib/array.rs:158"
    ),
    row!(
        Array,
        Instance,
        "pop",
        shapes![Both []],
        Unknown,
        "Removes and returns last element, or nil.",
        "crates/iris-eval/src/source_runtime.rs:4659; crates/iris-vm/src/machine/stdlib/array.rs:153"
    ),
    row!(
        Array,
        Instance,
        "push",
        shapes![Both[ARG1]],
        Receiver,
        "Appends element and returns receiver, unlike append.",
        "crates/iris-eval/src/source_runtime.rs:4655; crates/iris-vm/src/machine/stdlib/array.rs:110"
    ),
    row!(
        Array,
        Instance,
        "append",
        shapes![Both[ARG1]],
        Known(Nil),
        "Appends one element; returns nil, not receiver.",
        "crates/iris-eval/src/source_runtime.rs:198; crates/iris-vm/src/machine/stdlib/array.rs:118"
    ),
    row!(
        Array,
        Instance,
        "delete",
        shapes![Both[ARG1]],
        Known(Nil),
        "Removes first equal element; returns nil.",
        "crates/iris-eval/src/source_runtime.rs:198; crates/iris-vm/src/machine/stdlib/array.rs:128"
    ),
    row!(
        Array,
        Instance,
        "insert",
        shapes![Both [INTEGER, ARG2]],
        Known(Nil),
        "Inserts arg2 at Integer position; returns nil.",
        "crates/iris-eval/src/source_runtime.rs:198; crates/iris-vm/src/machine/stdlib/array.rs:136"
    ),
    row!(
        Array,
        Instance,
        "clear",
        shapes![Both []],
        Known(Nil),
        "Removes all elements; returns nil.",
        "crates/iris-eval/src/source_runtime.rs:198; crates/iris-vm/src/machine/stdlib/array.rs:122"
    ),
    row!(
        Array,
        Instance,
        "join",
        shapes![Reference[ARG1], Vm[TEXT]],
        Known(String),
        "Joins rendered elements. Reference converts separator with to_string; VM requires String.",
        "crates/iris-eval/src/source_runtime.rs:4660; crates/iris-vm/src/machine/stdlib/array.rs:154"
    ),
    row!(
        Array,
        Instance,
        "include?",
        shapes![Both[ARG1]],
        Known(Bool),
        "Tests element equality.",
        "crates/iris-eval/src/source_runtime.rs:4668; crates/iris-vm/src/machine/stdlib/array.rs:164"
    ),
    row!(
        Array,
        Instance,
        "index_of",
        shapes![Both[ARG1]],
        Unknown,
        "First equal element's Integer index or nil.",
        "crates/iris-eval/src/source_runtime.rs:4677; crates/iris-vm/src/machine/stdlib/array.rs:165"
    ),
    row!(
        Array,
        Instance,
        "concat",
        shapes![Both[positional("arg1", Some("Array"))]],
        Known(Array),
        "Fresh concatenation of two Arrays.",
        "crates/iris-eval/src/source_runtime.rs:4688; crates/iris-vm/src/machine/stdlib/array.rs:169"
    ),
    row!(
        Array,
        Instance,
        "slice",
        shapes![Both [INTEGER, positional("arg2", Some("Integer"))], Reference [positional("arg1", Some("Range"))]],
        Known(Array),
        "Fresh slice. Start/count form exists in both; Range form is reference-only.",
        "crates/iris-eval/src/source_runtime.rs:4693; crates/iris-vm/src/machine/stdlib/array.rs:174"
    ),
    row!(
        Array,
        Instance,
        "take",
        shapes![Both[INTEGER]],
        Known(Array),
        "Fresh prefix; nonnegative count required.",
        "crates/iris-eval/src/source_runtime.rs:4717; crates/iris-vm/src/machine/stdlib/array.rs:185"
    ),
    row!(
        Array,
        Instance,
        "drop",
        shapes![Both[INTEGER]],
        Known(Array),
        "Fresh suffix; nonnegative count required.",
        "crates/iris-eval/src/source_runtime.rs:4717; crates/iris-vm/src/machine/stdlib/array.rs:185"
    ),
    row!(
        Array,
        Instance,
        "all?",
        shapes![Both[CALLBACK]],
        Known(Bool),
        "All callback(element) results truthy; callback required.",
        "crates/iris-eval/src/source_runtime.rs:4749; crates/iris-vm/src/machine/stdlib/array.rs:201"
    ),
    row!(
        Array,
        Instance,
        "any?",
        shapes![Both[CALLBACK]],
        Known(Bool),
        "Any callback(element) result truthy; callback required.",
        "crates/iris-eval/src/source_runtime.rs:4749; crates/iris-vm/src/machine/stdlib/array.rs:201"
    ),
    row!(
        Array,
        Instance,
        "at",
        shapes![Both[INTEGER]],
        Unknown,
        "Element at Integer index or nil.",
        "crates/iris-eval/src/source_runtime.rs:4760; crates/iris-vm/src/machine/stdlib/array.rs:214"
    ),
    row!(
        Array,
        Instance,
        "to_string",
        shapes![Both []],
        Known(String),
        "Renders array elements.",
        "crates/iris-eval/src/source_runtime.rs:4765; crates/iris-vm/src/machine/stdlib/array.rs:217"
    ),
    row!(
        Hash,
        Instance,
        "length",
        shapes![Both []],
        Known(Integer),
        "Number of entries.",
        "crates/iris-eval/src/source_runtime.rs:11089; crates/iris-vm/src/machine/stdlib/hash_text.rs:16"
    ),
    row!(
        Hash,
        Instance,
        "empty?",
        shapes![Reference []],
        Known(Bool),
        "Whether no entries exist; reference only.",
        "crates/iris-eval/src/source_runtime.rs:11114"
    ),
    row!(
        Hash,
        Instance,
        "keys",
        shapes![Both []],
        Known(Array),
        "Array of keys; no ordering promise.",
        "crates/iris-eval/src/source_runtime.rs:4506; crates/iris-vm/src/machine/stdlib/hash_text.rs:28"
    ),
    row!(
        Hash,
        Instance,
        "values",
        shapes![Both []],
        Known(Array),
        "Array of values; no ordering promise.",
        "crates/iris-eval/src/source_runtime.rs:4509; crates/iris-vm/src/machine/stdlib/hash_text.rs:31"
    ),
    row!(
        Hash,
        Instance,
        "to_array",
        shapes![Both []],
        Known(Array),
        "Array of key/value Tuples; no ordering promise.",
        "crates/iris-eval/src/source_runtime.rs:4499; crates/iris-vm/src/machine/stdlib/hash_text.rs:116"
    ),
    row!(
        Hash,
        Instance,
        "include?",
        shapes![Both[ARG1]],
        Known(Bool),
        "Tests key presence.",
        "crates/iris-eval/src/source_runtime.rs:4519; crates/iris-vm/src/machine/stdlib/hash_text.rs:42"
    ),
    row!(
        Hash,
        Instance,
        "has_key?",
        shapes![Both[ARG1]],
        Known(Bool),
        "Tests key presence.",
        "crates/iris-eval/src/source_runtime.rs:4516; crates/iris-vm/src/machine/stdlib/hash_text.rs:42"
    ),
    row!(
        Hash,
        Instance,
        "fetch",
        shapes![Both[ARG1]],
        Unknown,
        "Stored value; absent raises KeyError. No default or block alternative.",
        "crates/iris-eval/src/source_runtime.rs:4413; crates/iris-vm/src/machine/stdlib/hash_text.rs:48"
    ),
    row!(
        Hash,
        Instance,
        "delete",
        shapes![Both[ARG1]],
        Unknown,
        "Removed value or nil.",
        "crates/iris-eval/src/source_runtime.rs:4421; crates/iris-vm/src/machine/stdlib/hash_text.rs:52"
    ),
    row!(
        Hash,
        Instance,
        "rehash",
        shapes![Both [], Both [CALLBACK]],
        Known(Nil),
        "Rebuilds hashes. Merge callback(kept_key, kept_value, incoming_key, incoming_value) must return key/value Tuple.",
        "crates/iris-eval/src/source_runtime.rs:4406; crates/iris-vm/src/machine/stdlib/hash_text.rs:202"
    ),
    row!(
        Hash,
        Instance,
        "each",
        shapes![Both[CALLBACK]],
        Unknown,
        "Calls callback(key, value). Reference returns nil; VM returns receiver Hash.",
        "crates/iris-eval/src/source_runtime.rs:4434; crates/iris-vm/src/machine/stdlib/hash_text.rs:60"
    ),
    row!(
        Hash,
        Instance,
        "each_with_iterator",
        shapes![Both[CALLBACK]],
        Unknown,
        "Calls callback(key, value, iterator). Reference returns nil; VM returns receiver Hash.",
        "crates/iris-eval/src/source_runtime.rs:4434; crates/iris-vm/src/machine/stdlib/hash_text.rs:60"
    ),
    row!(
        Hash,
        Instance,
        "map",
        shapes![Both[CALLBACK]],
        Known(Array),
        "Array of callback(key, value) results.",
        "crates/iris-eval/src/source_runtime.rs:4522; crates/iris-vm/src/machine/stdlib/hash_text.rs:83"
    ),
    row!(
        Hash,
        Instance,
        "select",
        shapes![Both[CALLBACK]],
        Known(Hash),
        "Hash of entries with truthy callback(key, value).",
        "crates/iris-eval/src/source_runtime.rs:4529; crates/iris-vm/src/machine/stdlib/hash_text.rs:90"
    ),
    row!(
        Hash,
        Instance,
        "merge",
        shapes![Both[positional("arg1", Some("Hash"))]],
        Known(Hash),
        "Fresh merged Hash.",
        "crates/iris-eval/src/source_runtime.rs:4539; crates/iris-vm/src/machine/stdlib/hash_text.rs:104"
    ),
    row!(
        Tuple,
        Instance,
        "to_array",
        shapes![Both []],
        Known(Array),
        "Array of tuple elements; no blanket Array methods.",
        "crates/iris-eval/src/source_runtime.rs:4461; crates/iris-vm/src/machine/stdlib.rs:681"
    ),
    row!(
        Range,
        Instance,
        "by",
        shapes![Both[INTEGER], Both[keyword("step", "Integer")]],
        Known(Range),
        "New Range with nonzero correctly directed stride; exactly one positional or step keyword argument.",
        "crates/iris-eval/src/source_runtime.rs:4375; crates/iris-vm/src/machine/stdlib.rs:704"
    ),
    row!(
        Range,
        Instance,
        "to_array",
        shapes![Both []],
        Known(Array),
        "Materializes Integer elements.",
        "crates/iris-eval/src/source_runtime.rs:4461; crates/iris-vm/src/machine/stdlib.rs:674"
    ),
    row!(
        Range,
        Instance,
        "start",
        shapes![Reference []],
        Known(Integer),
        "Start endpoint; reference only.",
        "crates/iris-eval/src/source_runtime.rs:11121"
    ),
    row!(
        Range,
        Instance,
        "end",
        shapes![Reference []],
        Known(Integer),
        "End endpoint; reference only.",
        "crates/iris-eval/src/source_runtime.rs:11124"
    ),
    row!(
        Range,
        Instance,
        "inclusive_end?",
        shapes![Reference []],
        Known(Bool),
        "End inclusivity; reference only.",
        "crates/iris-eval/src/source_runtime.rs:11127"
    ),
    row!(
        Range,
        Property,
        "start",
        shapes![Reference []],
        Known(Integer),
        "Start endpoint read; reference only.",
        "crates/iris-eval/src/source_runtime.rs:11121; crates/iris-eval/src/source_runtime.rs:8734"
    ),
    row!(
        Range,
        Property,
        "end",
        shapes![Reference []],
        Known(Integer),
        "End endpoint read; reference only.",
        "crates/iris-eval/src/source_runtime.rs:11124; crates/iris-eval/src/source_runtime.rs:8734"
    ),
    row!(
        Range,
        Property,
        "inclusive_end?",
        shapes![Reference []],
        Known(Bool),
        "End inclusivity read; reference only.",
        "crates/iris-eval/src/source_runtime.rs:11127; crates/iris-eval/src/source_runtime.rs:8734"
    ),
    row!(
        ReadonlyArray,
        Instance,
        "length",
        shapes![Both []],
        Known(Integer),
        "Number of readonly elements.",
        "crates/iris-eval/src/source_runtime.rs:11049; crates/iris-vm/src/machine/stdlib.rs:858"
    ),
    row!(
        ReadonlyArray,
        Instance,
        "empty?",
        shapes![Reference []],
        Known(Bool),
        "Whether readonly view is empty; reference only.",
        "crates/iris-eval/src/source_runtime.rs:11108"
    ),
];
