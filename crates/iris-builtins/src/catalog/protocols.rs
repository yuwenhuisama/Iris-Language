use super::*;
use BuiltinType::{Bool, Integer, String};

macro_rules! ordered {
    ($($family:ident),+) => { &[$(
        row!($family, Instance, "==", shapes![Both [ARG1]], Known(Bool), "Equality from ordering; unordered comparisons are false.", "crates/iris-runtime/src/kernel.rs:619; crates/iris-eval/src/source_runtime.rs:11415"),
        row!($family, Instance, "!=", shapes![Both [ARG1]], Known(Bool), "Inequality; unordered operands are unequal.", "crates/iris-runtime/src/kernel.rs:625; crates/iris-eval/src/source_runtime.rs:11415"),
        row!($family, Instance, "<", shapes![Both [ARG1]], Known(Bool), "Less-than ordering test; unordered operands return false.", "crates/iris-runtime/src/kernel.rs:631"),
        row!($family, Instance, "<=", shapes![Both [ARG1]], Known(Bool), "Less-or-equal ordering test; unordered operands return false.", "crates/iris-runtime/src/kernel.rs:634"),
        row!($family, Instance, ">", shapes![Both [ARG1]], Known(Bool), "Greater-than ordering test; unordered operands return false.", "crates/iris-runtime/src/kernel.rs:638"),
        row!($family, Instance, ">=", shapes![Both [ARG1]], Known(Bool), "Greater-or-equal ordering test; unordered operands return false.", "crates/iris-runtime/src/kernel.rs:641"),
        row!($family, Instance, "<=>", shapes![Both [ARG1]], Unknown, "Integer -1/0/1 or nil when unordered; numeric/non-numeric comparison may return nil.", "crates/iris-runtime/src/kernel.rs:645; crates/iris-runtime/src/kernel.rs:689"),
    )+] };
}
const ORDERED: &[BuiltinMember] = ordered![Integer, Float32, Float64, Nil, Bool];
macro_rules! equality {
    ($availability:ident; $($family:ident),+) => { &[$(
        row!($family, Instance, "==", shapes![$availability [ARG1]], Known(Bool), "Implemented family equality; not inherited universally from Object. Text/binary permit cross mutable/immutable content equality; cursors and callable/context values compare identity.", "crates/iris-eval/src/source_runtime.rs:11240; crates/iris-eval/src/source_runtime.rs:11327; crates/iris-vm/src/machine/operations.rs:142; crates/iris-vm/src/machine/operations.rs:529"),
        row!($family, Instance, "!=", shapes![$availability [ARG1]], Known(Bool), "Negation of implemented family equality; backend availability is specific to this family.", "crates/iris-eval/src/source_runtime.rs:11240; crates/iris-eval/src/source_runtime.rs:11327; crates/iris-vm/src/machine/operations.rs:142; crates/iris-vm/src/machine/operations.rs:529"),
    )+] };
}
const EQUALITY: &[BuiltinMember] = equality![Both; Object, String, Symbol, Bytes, ByteArray, MutableString, Tuple, Regex, Iteration, ArrayIterator, HashIterator, ByteIterator, Closure, BoundMethod, ExceptionContext];
const VM_EQUALITY: &[BuiltinMember] =
    equality![Vm; Range, Task, Gate, Generator, Class, Method, Contract];
macro_rules! hash_rows {
    ($availability:ident; $($family:ident),+) => { &[$(
        row!($family, Instance, "hash", shapes![$availability []], Known(Integer), "Public Integer hash on success. NaN refuses; Tuple and Iteration yield require hashable contents. Identity-bearing values retain stable identity hashes; no blanket payload inheritance.", "crates/iris-runtime/src/kernel.rs:809; crates/iris-eval/src/source_runtime.rs:11155; crates/iris-eval/src/source_runtime.rs:11223; crates/iris-vm/src/machine/operations.rs:178"),
    )+] };
}
const HASHES: &[BuiltinMember] = hash_rows![Both; Object, Integer, Float32, Float64, Nil, Bool, String, Symbol, Bytes, Tuple, Range, Regex, Iteration, ArrayIterator, HashIterator, ByteIterator, ExceptionContext, Contract, ContractView];
const VM_HASHES: &[BuiltinMember] = hash_rows![Vm; Closure, Task, Gate, Generator];
macro_rules! identity_rows {
    ($availability:ident; $($family:ident),+) => { &[$(
        row!($family, Instance, "same?", shapes![$availability [ARG1]], Known(Bool), "Primitive identity question, not an overridable Method. Unsupported or identity-less operands may raise IdentityError; Iteration only succeeds for done singleton, not yield.", "crates/iris-eval/src/source_runtime.rs:7832; crates/iris-vm/src/compile/calls.rs:869; crates/iris-vm/src/machine/operations.rs:212"),
    )+] };
}
const IDENTITIES: &[BuiltinMember] = identity_rows![Both; Object, Nil, Bool, Array, Hash, MutableString, ArrayIterator, HashIterator, ByteIterator, Generator, Iteration, Closure, BoundMethod, Method, Class, Type, ComposedType, Contract, Task, Gate, ExceptionContext];
const REFERENCE_IDENTITIES: &[BuiltinMember] = identity_rows![Reference; ByteArray, FfiLibrary];
macro_rules! root_rows {
    ($($family:ident),+) => { &[$(
        row!($family, Instance, "to_bool", shapes![Both []], Known(Bool), "Default truth conversion: nil false, Bool itself, others true. Authored overrides win; this is not a universal inherited payload list.", "crates/iris-runtime/src/kernel.rs:245; crates/iris-runtime/src/kernel.rs:815; crates/iris-eval/src/source_runtime.rs:8624"),
    )+] };
}
const ROOT: &[BuiltinMember] = root_rows![
    Object,
    Nil,
    Bool,
    Integer,
    Float32,
    Float64,
    String,
    Symbol,
    Array,
    Hash,
    Tuple,
    Range,
    ReadonlyArray,
    MutableString,
    Bytes,
    ByteArray,
    Regex,
    Match,
    ArrayIterator,
    HashIterator,
    ByteIterator,
    Generator,
    Iteration,
    Closure,
    BoundMethod,
    Method,
    Task,
    Gate,
    ExceptionContext,
    SourceLocation,
    StackFrame,
    RaiseSite,
    Type,
    ComposedType,
    Contract,
    FfiLibrary
];
macro_rules! rendering {
    ($($family:ident),+) => { &[$(
        row!($family, Instance, "to_string", shapes![Both []], Known(String), "String rendering of value; ordinary Object fallback renders nominal name directly.", "crates/iris-eval/src/source_runtime.rs:4280; crates/iris-eval/src/source_runtime.rs:12277; crates/iris-vm/src/machine/stdlib.rs:736; crates/iris-vm/src/machine/stdlib.rs:2450"),
    )+] };
}
const RENDERING: &[BuiltinMember] =
    rendering![Object, Integer, Float32, Float64, Nil, Bool, Symbol];
macro_rules! probes {
    ($($family:ident),+) => { &[$(
        row!($family, Instance, "respond_to?", shapes![Reference [SYMBOL]], Known(Bool), "Reference-only Symbol selector probe. Only ordinary heap Object tests visible slots; other receiver families return false. Does not invoke method_missing or execute a candidate method.", "crates/iris-eval/src/source_runtime.rs:7973"),
    )+] };
}
const PROBES: &[BuiltinMember] = probes![
    Object,
    Nil,
    Bool,
    Integer,
    Float32,
    Float64,
    String,
    Symbol,
    Array,
    Hash,
    Tuple,
    Range,
    ReadonlyArray,
    MutableString,
    Bytes,
    ByteArray,
    Regex,
    Match,
    ArrayIterator,
    HashIterator,
    ByteIterator,
    Generator,
    Iteration,
    Closure,
    BoundMethod,
    Method,
    Task,
    Gate,
    ExceptionContext,
    SourceLocation,
    StackFrame,
    RaiseSite,
    ExternalResource,
    Class,
    Type,
    ComposedType,
    Contract,
    ContractView,
    Transformation,
    FfiLibrary
];
const SPECIAL: &[BuiltinMember] = &[
    row!(
        Transformation,
        Instance,
        "to_bool",
        shapes![Reference []],
        Known(Bool),
        "Default true for Transformation value; reference-only value route.",
        "crates/iris-eval/src/source_runtime.rs:7950; crates/iris-runtime/src/kernel.rs:538"
    ),
    row!(
        Module,
        Instance,
        "same?",
        shapes![Vm[ARG1]],
        Known(Bool),
        "VM primitive identity on declared Module names. Reference direct Module source-name calls intercept ordinary method lookup; value-level module Symbols can still reach reference identity primitive.",
        "crates/iris-eval/src/source_runtime.rs:6529; crates/iris-vm/src/machine/operations.rs:244"
    ),
    row!(
        Object,
        Instance,
        "inspect",
        shapes![Both []],
        Known(String),
        "Direct nominal-name fallback; does not call an override of to_string.",
        "crates/iris-eval/src/source_runtime.rs:12277; crates/iris-vm/src/machine/stdlib.rs:2450"
    ),
    row!(
        Object,
        Instance,
        "<=>",
        shapes![Reference[ARG1]],
        Unknown,
        "Default ordinary Object ordering is nil. Authored ordering may replace it. VM default ordering is operator-only.",
        "crates/iris-eval/src/source_runtime.rs:8659; crates/iris-vm/src/machine/stdlib.rs:13"
    ),
    row!(
        Object,
        Instance,
        "<",
        shapes![Reference[ARG1]],
        Known(Bool),
        "Derived Object ordering; default unordered. VM route is operator-only.",
        "crates/iris-eval/src/source_runtime.rs:8649; crates/iris-vm/src/machine/stdlib.rs:13"
    ),
    row!(
        Object,
        Instance,
        "<=",
        shapes![Reference[ARG1]],
        Known(Bool),
        "Derived Object ordering; default unordered. VM route is operator-only.",
        "crates/iris-eval/src/source_runtime.rs:8649; crates/iris-vm/src/machine/stdlib.rs:13"
    ),
    row!(
        Object,
        Instance,
        ">",
        shapes![Reference[ARG1]],
        Known(Bool),
        "Derived Object ordering; default unordered. VM route is operator-only.",
        "crates/iris-eval/src/source_runtime.rs:8649; crates/iris-vm/src/machine/stdlib.rs:13"
    ),
    row!(
        Object,
        Instance,
        ">=",
        shapes![Reference[ARG1]],
        Known(Bool),
        "Derived Object ordering; default unordered. VM route is operator-only.",
        "crates/iris-eval/src/source_runtime.rs:8649; crates/iris-vm/src/machine/stdlib.rs:13"
    ),
    row!(
        Iteration,
        Instance,
        "<=>",
        shapes![Reference[ARG1]],
        Unknown,
        "Integer ordering or nil. Both backends reject unordered nil payload comparison; VM ordering is operator-only.",
        "crates/iris-eval/src/source_runtime.rs:11282; crates/iris-vm/src/machine/stdlib.rs:2287"
    ),
    row!(
        FfiLibrary,
        Instance,
        "==",
        shapes![Both[ARG1]],
        Known(Bool),
        "Compares Library handle identity, not paths; requires Library argument.",
        "crates/iris-eval/src/source_runtime.rs:11355; crates/iris-vm/src/machine/stdlib.rs:1964"
    ),
    row!(
        FfiLibrary,
        Instance,
        "!=",
        shapes![Reference[ARG1]],
        Known(Bool),
        "Library handle inequality; reference only.",
        "crates/iris-eval/src/source_runtime.rs:11355"
    ),
];
const GROUPS: &[&[BuiltinMember]] = &[
    ORDERED,
    EQUALITY,
    VM_EQUALITY,
    HASHES,
    VM_HASHES,
    IDENTITIES,
    REFERENCE_IDENTITIES,
    ROOT,
    RENDERING,
    PROBES,
    SPECIAL,
];
pub(super) const ROWS: &[BuiltinMember] = &flatten::<{ count(GROUPS) }>(GROUPS);
