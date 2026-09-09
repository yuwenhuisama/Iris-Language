use super::*;
use BuiltinType::{Array, Bool, Bytes, Integer, MutableString, Regex, String, Symbol};

macro_rules! binary_rows {
    ($($family:ident),+) => { &[$(
        row!($family, Instance, "length", shapes![Both []], Known(Integer), "Byte count.", "crates/iris-eval/src/source_runtime.rs:11068; crates/iris-vm/src/machine/operations.rs:122"),
        row!($family, Instance, "empty?", shapes![Reference []], Known(Bool), "Whether byte sequence is empty; reference only.", "crates/iris-eval/src/source_runtime.rs:11078"),
        row!($family, Instance, "to_string", shapes![Both []], Known(String), "Strict UTF-8 decoding; invalid bytes raise EncodingError.", "crates/iris-eval/src/source_runtime.rs:4358; crates/iris-vm/src/machine/stdlib.rs:820"),
        row!($family, Instance, "to_bytes", shapes![Both []], Known(Bytes), "Immutable Bytes snapshot.", "crates/iris-eval/src/source_runtime.rs:4366; crates/iris-vm/src/machine/stdlib.rs:842"),
        row!($family, Instance, "to_array", shapes![Both []], Known(Array), "Array of Integer bytes.", "crates/iris-eval/src/source_runtime.rs:4461; crates/iris-vm/src/machine/stdlib.rs:684"),
        row!($family, Instance, "+", shapes![Reference [ARG1]], Known(BuiltinType::$family), "Fresh same-family concatenation with Bytes/ByteArray. VM supports operator syntax only.", "crates/iris-eval/src/source_runtime.rs:3967; crates/iris-vm/src/machine/stdlib.rs:192"),
    )+] };
}
const BINARY: &[BuiltinMember] = binary_rows![Bytes, ByteArray];
macro_rules! transform_rows {
    ($($family:ident),+) => { &[$(
        row!($family, Instance, "casefold", shapes![Both []], Known(String), "Unicode case folding returns immutable String for either receiver.", "crates/iris-eval/src/source_runtime.rs:4235; crates/iris-vm/src/machine/stdlib.rs:641"),
        row!($family, Instance, "nfc", shapes![Both []], Known(String), "NFC normalization returns immutable String.", "crates/iris-eval/src/source_runtime.rs:4249; crates/iris-vm/src/machine/stdlib.rs:641"),
        row!($family, Instance, "nfd", shapes![Both []], Known(String), "NFD normalization returns immutable String.", "crates/iris-eval/src/source_runtime.rs:4249; crates/iris-vm/src/machine/stdlib.rs:641"),
        row!($family, Instance, "graphemes", shapes![Both []], ArrayOf(String), "Eager Array of grapheme-cluster Strings, not a lazy view.", "crates/iris-eval/src/source_runtime.rs:4265; crates/iris-vm/src/machine/stdlib.rs:641"),
        row!($family, Instance, "=~", shapes![Both [positional("arg1", Some("Regex"))]], Unknown, "First Match snapshot or nil; Regex argument required.", "crates/iris-eval/src/source_runtime.rs:4000; crates/iris-vm/src/machine/stdlib.rs:424"),
        row!($family, Instance, "!~", shapes![Both [positional("arg1", Some("Regex"))]], Known(Bool), "True when no Regex match exists.", "crates/iris-eval/src/source_runtime.rs:4000; crates/iris-vm/src/machine/stdlib.rs:424"),
    )+] };
}
const TRANSFORMS: &[BuiltinMember] = transform_rows![String, MutableString];
const TEXT_ROWS: &[BuiltinMember] = &[
    row!(
        String,
        Instance,
        "length",
        shapes![Both []],
        Known(Integer),
        "Unicode scalar count, not UTF-8 byte count.",
        "crates/iris-eval/src/source_runtime.rs:11096; crates/iris-vm/src/machine/stdlib/hash_text.rs:131"
    ),
    row!(
        String,
        Instance,
        "byte_length",
        shapes![Both []],
        Known(Integer),
        "UTF-8 byte count.",
        "crates/iris-eval/src/source_runtime.rs:11103; crates/iris-vm/src/machine/stdlib/hash_text.rs:159"
    ),
    row!(
        String,
        Instance,
        "empty?",
        shapes![Reference []],
        Known(Bool),
        "Whether empty; reference only.",
        "crates/iris-eval/src/source_runtime.rs:11117"
    ),
    row!(
        String,
        Instance,
        "to_string",
        shapes![Both []],
        Known(String),
        "Returns receiver text.",
        "crates/iris-eval/src/source_runtime.rs:4294; crates/iris-vm/src/machine/stdlib/hash_text.rs:132"
    ),
    row!(
        String,
        Instance,
        "inspect",
        shapes![Both []],
        Known(String),
        "Escaped reparsable String literal.",
        "crates/iris-eval/src/source_runtime.rs:4332; crates/iris-vm/src/machine/stdlib/hash_text.rs:173"
    ),
    row!(
        String,
        Instance,
        "split",
        shapes![Both[TEXT]],
        ArrayOf(String),
        "Array of Strings split by required String separator.",
        "crates/iris-eval/src/source_runtime.rs:4295; crates/iris-vm/src/machine/stdlib/hash_text.rs:133"
    ),
    row!(
        String,
        Instance,
        "trim",
        shapes![Both []],
        Known(String),
        "Trims surrounding whitespace.",
        "crates/iris-eval/src/source_runtime.rs:4305; crates/iris-vm/src/machine/stdlib/hash_text.rs:138"
    ),
    row!(
        String,
        Instance,
        "downcase",
        shapes![Both []],
        Known(String),
        "Lowercase String.",
        "crates/iris-eval/src/source_runtime.rs:4160; crates/iris-vm/src/machine/stdlib/hash_text.rs:145"
    ),
    row!(
        String,
        Instance,
        "upcase",
        shapes![Reference []],
        Known(String),
        "Uppercase String; reference only.",
        "crates/iris-eval/src/source_runtime.rs:4160"
    ),
    row!(
        String,
        Instance,
        "replace",
        shapes![Both [TEXT, positional("arg2", Some("String"))]],
        Known(String),
        "Replaces occurrences of arg1 with arg2; both Strings.",
        "crates/iris-eval/src/source_runtime.rs:4306; crates/iris-vm/src/machine/stdlib/hash_text.rs:139"
    ),
    row!(
        String,
        Instance,
        "starts_with?",
        shapes![Both[TEXT]],
        Known(Bool),
        "Tests String prefix.",
        "crates/iris-eval/src/source_runtime.rs:4312; crates/iris-vm/src/machine/stdlib/hash_text.rs:142"
    ),
    row!(
        String,
        Instance,
        "ends_with?",
        shapes![Both[TEXT]],
        Known(Bool),
        "Tests String suffix.",
        "crates/iris-eval/src/source_runtime.rs:4315; crates/iris-vm/src/machine/stdlib/hash_text.rs:143"
    ),
    row!(
        String,
        Instance,
        "contains?",
        shapes![Both[TEXT]],
        Known(Bool),
        "Tests String substring.",
        "crates/iris-eval/src/source_runtime.rs:4318; crates/iris-vm/src/machine/stdlib/hash_text.rs:144"
    ),
    row!(
        String,
        Instance,
        "chars",
        shapes![Both []],
        ArrayOf(String),
        "Array of one-scalar Strings.",
        "crates/iris-eval/src/source_runtime.rs:4321; crates/iris-vm/src/machine/stdlib/hash_text.rs:146"
    ),
    row!(
        String,
        Instance,
        "to_array",
        shapes![Both []],
        ArrayOf(String),
        "Array of one-scalar Strings.",
        "crates/iris-eval/src/source_runtime.rs:4147; crates/iris-vm/src/machine/stdlib/hash_text.rs:163"
    ),
    row!(
        String,
        Instance,
        "to_symbol",
        shapes![Both []],
        Known(Symbol),
        "Symbol with this text.",
        "crates/iris-eval/src/source_runtime.rs:4326; crates/iris-vm/src/machine/stdlib/hash_text.rs:151"
    ),
    row!(
        String,
        Instance,
        "to_bytes",
        shapes![Both []],
        Known(Bytes),
        "UTF-8 snapshot.",
        "crates/iris-eval/src/source_runtime.rs:4368; crates/iris-vm/src/machine/stdlib/hash_text.rs:155"
    ),
    row!(
        String,
        Instance,
        "bytes",
        shapes![Reference []],
        Known(Bytes),
        "Eager UTF-8 Bytes snapshot; reference only.",
        "crates/iris-eval/src/source_runtime.rs:4232"
    ),
    row!(
        String,
        Property,
        "bytes",
        shapes![Reference []],
        Known(Bytes),
        "Eager UTF-8 Bytes snapshot property; reference only.",
        "crates/iris-eval/src/source_runtime.rs:4232; crates/iris-eval/src/source_runtime.rs:8734"
    ),
    row!(
        String,
        Instance,
        "+",
        shapes![Reference[ARG1]],
        Known(String),
        "Concatenates String or to_string-converted value. VM supports operator syntax only.",
        "crates/iris-eval/src/source_runtime.rs:3952; crates/iris-vm/src/machine/stdlib.rs:217"
    ),
    row!(
        MutableString,
        Instance,
        "length",
        shapes![Both []],
        Known(Integer),
        "Current Unicode scalar count.",
        "crates/iris-eval/src/source_runtime.rs:4196; crates/iris-vm/src/machine/operations.rs:128"
    ),
    row!(
        MutableString,
        Instance,
        "to_string",
        shapes![Both []],
        Known(String),
        "Immutable String snapshot.",
        "crates/iris-eval/src/source_runtime.rs:4192; crates/iris-vm/src/machine/stdlib.rs:836"
    ),
    row!(
        MutableString,
        Instance,
        "to_bytes",
        shapes![Reference []],
        Known(Bytes),
        "UTF-8 snapshot; reference only.",
        "crates/iris-eval/src/source_runtime.rs:4193"
    ),
    row!(
        MutableString,
        Instance,
        "to_array",
        shapes![Reference []],
        ArrayOf(String),
        "Array of scalar Strings; reference only.",
        "crates/iris-eval/src/source_runtime.rs:4147"
    ),
    row!(
        MutableString,
        Instance,
        "upcase",
        shapes![Both []],
        Known(MutableString),
        "Fresh uppercase MutableString.",
        "crates/iris-eval/src/source_runtime.rs:4167; crates/iris-vm/src/machine/stdlib.rs:598"
    ),
    row!(
        MutableString,
        Instance,
        "downcase",
        shapes![Both []],
        Known(MutableString),
        "Fresh lowercase MutableString.",
        "crates/iris-eval/src/source_runtime.rs:4167; crates/iris-vm/src/machine/stdlib.rs:598"
    ),
    row!(
        MutableString,
        Instance,
        "upcase!",
        shapes![Both []],
        Receiver,
        "Uppercases in place; returns receiver.",
        "crates/iris-eval/src/source_runtime.rs:4178; crates/iris-vm/src/machine/stdlib.rs:598"
    ),
    row!(
        MutableString,
        Instance,
        "downcase!",
        shapes![Both []],
        Receiver,
        "Lowercases in place; returns receiver.",
        "crates/iris-eval/src/source_runtime.rs:4178; crates/iris-vm/src/machine/stdlib.rs:598"
    ),
    row!(
        MutableString,
        Instance,
        "clear",
        shapes![Both []],
        Receiver,
        "Clears in place; returns receiver.",
        "crates/iris-eval/src/source_runtime.rs:4216; crates/iris-vm/src/machine/stdlib.rs:620"
    ),
    row!(
        MutableString,
        Instance,
        "append",
        shapes![Both[ARG1]],
        Receiver,
        "Converts through to_string and appends in place.",
        "crates/iris-eval/src/source_runtime.rs:4206; crates/iris-vm/src/machine/stdlib.rs:587"
    ),
    row!(
        MutableString,
        Instance,
        "replace",
        shapes![Both[ARG1]],
        Receiver,
        "Replaces whole content using text conversion; one argument unlike String.replace.",
        "crates/iris-eval/src/source_runtime.rs:4220; crates/iris-vm/src/machine/stdlib.rs:632"
    ),
    row!(
        MutableString,
        Instance,
        "<<",
        shapes![Reference[ARG1]],
        Receiver,
        "Appends converted text. VM operator-only route accepts String/MutableString.",
        "crates/iris-eval/src/source_runtime.rs:4206; crates/iris-vm/src/machine/stdlib.rs:144"
    ),
    row!(
        MutableString,
        Instance,
        "+",
        shapes![Reference[ARG1]],
        Known(MutableString),
        "Reference send returns fresh MutableString. VM operator-only route returns String and accepts text operands only.",
        "crates/iris-eval/src/source_runtime.rs:4206; crates/iris-vm/src/machine/stdlib.rs:144"
    ),
    row!(
        MutableString,
        Instance,
        "bytes",
        shapes![Both []],
        Receiver,
        "Current representation returns receiver; its iterator yields scalar Strings, not Integer bytes.",
        "crates/iris-eval/src/source_runtime.rs:4228; crates/iris-vm/src/machine/stdlib.rs:627"
    ),
    row!(
        MutableString,
        Property,
        "bytes",
        shapes![Reference []],
        Receiver,
        "Returns receiver, whose iterator yields scalar Strings. VM bare member route bypasses authored bytes handler; only its parenthesized call succeeds.",
        "crates/iris-eval/src/source_runtime.rs:4228; crates/iris-vm/src/machine/execute.rs:1392"
    ),
    row!(
        ByteArray,
        Instance,
        "append",
        shapes![Reference[ARG1]],
        Receiver,
        "Appends Bytes/ByteArray only, no text conversion; reference only.",
        "crates/iris-eval/src/source_runtime.rs:3967"
    ),
    row!(
        ByteArray,
        Instance,
        "<<",
        shapes![Reference[ARG1]],
        Receiver,
        "Appends Bytes/ByteArray only; reference only.",
        "crates/iris-eval/src/source_runtime.rs:3967"
    ),
    row!(
        Match,
        Instance,
        "capture",
        shapes![Both[ARG1]],
        Unknown,
        "String or nil by one-based Integer index or Symbol/String capture name.",
        "crates/iris-eval/src/source_runtime.rs:4068; crates/iris-vm/src/machine/stdlib.rs:565"
    ),
];
macro_rules! match_rows {
    ($($surface:ident),+) => { &[$(
        row!(Match, $surface, "text", shapes![Both []], Known(String), "Full match text snapshot.", "crates/iris-eval/src/source_runtime.rs:4048; crates/iris-vm/src/machine/stdlib.rs:548"),
        row!(Match, $surface, "to_string", shapes![Both []], Known(String), "Full match text snapshot.", "crates/iris-eval/src/source_runtime.rs:4048; crates/iris-vm/src/machine/stdlib.rs:548"),
        row!(Match, $surface, "regex", shapes![Both []], Known(Regex), "Regex used for the match.", "crates/iris-eval/src/source_runtime.rs:4051; crates/iris-vm/src/machine/stdlib.rs:550"),
        row!(Match, $surface, "start", shapes![Both []], Known(Integer), "Scalar start offset.", "crates/iris-eval/src/source_runtime.rs:4060; crates/iris-vm/src/machine/stdlib.rs:557"),
        row!(Match, $surface, "end", shapes![Both []], Known(Integer), "Scalar end offset.", "crates/iris-eval/src/source_runtime.rs:4065; crates/iris-vm/src/machine/stdlib.rs:562"),
        row!(Match, $surface, "byte_start", shapes![Both []], Known(Integer), "Byte start offset.", "crates/iris-eval/src/source_runtime.rs:4054; crates/iris-vm/src/machine/stdlib.rs:551"),
        row!(Match, $surface, "byte_end", shapes![Both []], Known(Integer), "Byte end offset.", "crates/iris-eval/src/source_runtime.rs:4057; crates/iris-vm/src/machine/stdlib.rs:554"),
    )+] };
}
const MATCH: &[BuiltinMember] = match_rows![Instance, Property];
const GROUPS: &[&[BuiltinMember]] = &[BINARY, TRANSFORMS, TEXT_ROWS, MATCH];
pub(super) const ROWS: &[BuiltinMember] = &flatten::<{ count(GROUPS) }>(GROUPS);
