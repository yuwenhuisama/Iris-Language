use super::*;
use BuiltinType::{
    Array, ArrayIterator, Bool, BoundMethod, ByteIterator, HashIterator, Integer, Iteration, Nil,
    ReadonlyArray, String, Symbol,
};

const ITERATORS: &[BuiltinMember] = &[
    row!(
        Array,
        Instance,
        "iterator",
        shapes![Both []],
        Known(ArrayIterator),
        "Fresh Array cursor yielding elements.",
        "crates/iris-eval/src/source_runtime.rs:10635; crates/iris-vm/src/machine/stdlib/iteration.rs:36"
    ),
    row!(
        ReadonlyArray,
        Instance,
        "iterator",
        shapes![Reference []],
        Known(ArrayIterator),
        "Detached readonly cursor; explicit send reference-only.",
        "crates/iris-eval/src/source_runtime.rs:10635"
    ),
    row!(
        Range,
        Instance,
        "iterator",
        shapes![Both []],
        Known(ArrayIterator),
        "Cursor over Integer range elements.",
        "crates/iris-eval/src/source_runtime.rs:10664; crates/iris-vm/src/machine/stdlib/iteration.rs:50"
    ),
    row!(
        Hash,
        Instance,
        "iterator",
        shapes![Both []],
        Known(HashIterator),
        "Cursor yielding key/value Tuples.",
        "crates/iris-eval/src/source_runtime.rs:10806; crates/iris-vm/src/machine/stdlib/iteration.rs:45"
    ),
    row!(
        Bytes,
        Instance,
        "iterator",
        shapes![Both []],
        Unknown,
        "Cursor yielding Integer bytes. Reference produces ByteIterator; VM produces ArrayIterator.",
        "crates/iris-eval/src/source_runtime.rs:10703; crates/iris-vm/src/machine/stdlib/iteration.rs:66"
    ),
    row!(
        ByteArray,
        Instance,
        "iterator",
        shapes![Reference []],
        Known(ByteIterator),
        "Live fail-fast byte cursor; explicit send reference-only.",
        "crates/iris-eval/src/source_runtime.rs:10703"
    ),
    row!(
        MutableString,
        Instance,
        "iterator",
        shapes![Both []],
        Unknown,
        "Live scalar String cursor. Reference produces ByteIterator; VM produces ArrayIterator.",
        "crates/iris-eval/src/source_runtime.rs:10703; crates/iris-vm/src/machine/stdlib/iteration.rs:55"
    ),
    row!(
        ArrayIterator,
        Instance,
        "next",
        shapes![Both []],
        Known(Iteration),
        "Next yield or done signal; exhaustion releases source.",
        "crates/iris-eval/src/source_runtime.rs:10957; crates/iris-vm/src/machine/stdlib/iteration.rs:115"
    ),
    row!(
        ArrayIterator,
        Instance,
        "close",
        shapes![Both []],
        Known(Nil),
        "Releases cursor source; idempotent.",
        "crates/iris-eval/src/source_runtime.rs:10992; crates/iris-vm/src/machine/stdlib/iteration.rs:109"
    ),
    row!(
        HashIterator,
        Instance,
        "next",
        shapes![Both []],
        Known(Iteration),
        "Next key/value Tuple yield or done signal.",
        "crates/iris-eval/src/source_runtime.rs:10824; crates/iris-vm/src/machine/stdlib/iteration.rs:115"
    ),
    row!(
        HashIterator,
        Instance,
        "close",
        shapes![Both []],
        Known(Nil),
        "Returns nil. VM releases source; reference currently returns nil without releasing traversal state.",
        "crates/iris-eval/src/source_runtime.rs:10878; crates/iris-vm/src/machine/stdlib/iteration.rs:109"
    ),
    row!(
        HashIterator,
        Instance,
        "remove_current",
        shapes![Both []],
        Known(Nil),
        "Removes most recently yielded entry once; checks state and concurrent modification.",
        "crates/iris-eval/src/source_runtime.rs:10860; crates/iris-vm/src/machine/stdlib/iteration.rs:124"
    ),
    row!(
        ByteIterator,
        Instance,
        "next",
        shapes![Reference []],
        Known(Iteration),
        "Next byte/scalar yield or done; reference-only cursor.",
        "crates/iris-eval/src/source_runtime.rs:10757"
    ),
    row!(
        ByteIterator,
        Instance,
        "close",
        shapes![Reference []],
        Known(Nil),
        "Releases cursor and enters done state; reference only.",
        "crates/iris-eval/src/source_runtime.rs:10794"
    ),
    row!(
        Generator,
        Instance,
        "iterator",
        shapes![Reference []],
        Receiver,
        "Generator is its own cursor; explicit reference route.",
        "crates/iris-eval/src/source_runtime.rs:10889"
    ),
    row!(
        Generator,
        Instance,
        "next",
        shapes![Reference []],
        Known(Iteration),
        "Resumes generator to yield or done; explicit reference route.",
        "crates/iris-eval/src/source_runtime.rs:10892"
    ),
    row!(
        Generator,
        Instance,
        "close",
        shapes![Reference []],
        Known(Nil),
        "Marks generator finished; explicit reference route.",
        "crates/iris-eval/src/source_runtime.rs:10896"
    ),
    row!(
        Closure,
        Instance,
        "call",
        &[],
        Unknown,
        "Invokes this Closure. Callable-specific signature and result are unknown to the catalog; empty shapes do not mean zero arguments. Bare Closure application is refused. Both backends support .call.",
        "crates/iris-eval/src/source_runtime.rs:11001; crates/iris-vm/src/machine/execute.rs:2329"
    ),
    row!(
        BoundMethod,
        Instance,
        "call",
        &[],
        Unknown,
        "Invokes retained Method with its bound receiver. Callable-specific signature and result are unknown; empty shapes do not mean zero arguments. Both backends support .call.",
        "crates/iris-eval/src/source_runtime.rs:11001; crates/iris-vm/src/machine/execute.rs:2329"
    ),
    row!(
        Method,
        Instance,
        "bind",
        shapes![Both[ARG1]],
        Known(BoundMethod),
        "Binds retained Method to heap Object or Class after binding checks.",
        "crates/iris-eval/src/source_runtime.rs:10909; crates/iris-vm/src/machine/stdlib.rs:1118"
    ),
    row!(
        ExternalResource,
        Instance,
        "close",
        shapes![Both []],
        Known(Nil),
        "Closes extension-declared native resource through host registry; not a globally named Class.",
        "crates/iris-eval/src/source_runtime/native.rs:44; crates/iris-vm/src/machine/operations.rs:106"
    ),
    row!(
        ExternalResource,
        Instance,
        "closed?",
        shapes![Both []],
        Known(Bool),
        "Whether extension resource is closed.",
        "crates/iris-eval/src/source_runtime/native.rs:57; crates/iris-vm/src/machine/operations.rs:114"
    ),
    row!(
        ExternalResource,
        Instance,
        "==",
        shapes![Both[ARG1]],
        Known(Bool),
        "Resource identity equality; argument must be ExternalResource.",
        "crates/iris-eval/src/source_runtime/native.rs:58; crates/iris-vm/src/machine/operations.rs:115"
    ),
    row!(
        ExternalResource,
        Instance,
        "same?",
        shapes![Both[ARG1]],
        Known(Bool),
        "Primitive resource identity question; argument must be ExternalResource for the native send route.",
        "crates/iris-eval/src/source_runtime/native.rs:58; crates/iris-vm/src/machine/operations.rs:115"
    ),
    row!(
        FfiLibrary,
        Instance,
        "bind",
        shapes![Both [ARG1, positional("arg2", Some("Hash"))]],
        Receiver,
        "Returns Library retaining identity with new binding metadata. arg1 Symbol/String; signature Hash validated. Ignored tail is not a declared rest parameter; no native invocation.",
        "crates/iris-eval/src/source_runtime.rs:4086; crates/iris-vm/src/machine/stdlib.rs:1911"
    ),
    row!(
        FfiLibrary,
        Instance,
        "signature",
        shapes![Both[ARG1]],
        Unknown,
        "Stored signature Hash or nil by Symbol/String name.",
        "crates/iris-eval/src/source_runtime.rs:4107; crates/iris-vm/src/machine/stdlib.rs:1929"
    ),
    row!(
        FfiLibrary,
        Instance,
        "bound?",
        shapes![Both[ARG1]],
        Known(Bool),
        "Whether Symbol/String name has binding metadata.",
        "crates/iris-eval/src/source_runtime.rs:4119; crates/iris-vm/src/machine/stdlib.rs:1940"
    ),
    row!(
        FfiLibrary,
        Instance,
        "class_name",
        shapes![Both []],
        Known(String),
        "String FFI::Library, unlike callable class_name Symbols.",
        "crates/iris-eval/src/source_runtime.rs:10589; crates/iris-vm/src/machine/stdlib.rs:1911"
    ),
];
macro_rules! getters {
    ($($surface:ident),+) => { &[$(
        row!(Iteration, $surface, "yield?", shapes![Both []], Known(Bool), "Whether signal carries a yielded value.", "crates/iris-eval/src/source_runtime.rs:11137; crates/iris-vm/src/machine/stdlib/iteration.rs:102"),
        row!(Iteration, $surface, "done?", shapes![Both []], Known(Bool), "Whether signal denotes exhaustion.", "crates/iris-eval/src/source_runtime.rs:11137; crates/iris-vm/src/machine/stdlib/iteration.rs:102"),
        row!(Iteration, $surface, "value", shapes![Both []], Unknown, "Yielded payload; done.value raises IteratorStateError.", "crates/iris-eval/src/source_runtime.rs:11137; crates/iris-vm/src/machine/stdlib/iteration.rs:102"),
        row!(Method, $surface, "selector", shapes![Both []], Unknown, "Selector Symbol or nil. Reference ignores extra arguments; VM requires zero.", "crates/iris-eval/src/source_runtime.rs:8349; crates/iris-vm/src/machine/stdlib.rs:1042"),
        row!(Method, $surface, "owner", shapes![Both []], Unknown, "Class, module Symbol or nil; VM reports nil for Module owner. Reference ignores extra arguments.", "crates/iris-eval/src/source_runtime.rs:8356; crates/iris-vm/src/machine/stdlib.rs:1048"),
        row!(Method, $surface, "visibility", shapes![Both []], Known(Symbol), "Visibility Symbol; reference ignores extra arguments, VM requires zero.", "crates/iris-eval/src/source_runtime.rs:8366; crates/iris-vm/src/machine/stdlib.rs:1052"),
        row!(Method, $surface, "parameters", shapes![Both []], Known(Array), "Array of authored type-name Symbols; not kernel signature discovery. Reference ignores extra arguments.", "crates/iris-eval/src/source_runtime.rs:8390; crates/iris-vm/src/machine/stdlib.rs:1060"),
        row!(Method, $surface, "return_type", shapes![Both []], Known(Symbol), "Authored return-type Symbol; omitted annotation remains Dynamic<Object>. Reference ignores extra arguments.", "crates/iris-eval/src/source_runtime.rs:8390; crates/iris-vm/src/machine/stdlib.rs:1081"),
        row!(Method, $surface, "source", shapes![Both []], Unknown, "Class-owned: Array [package Symbol, revision Integer, commit Integer, status Symbol]; module-owned: Symbol. Not SourceLocation.", "crates/iris-eval/src/source_runtime.rs:8373; crates/iris-vm/src/machine/stdlib.rs:1095"),
        row!(Method, $surface, "class_name", shapes![Reference []], Known(Symbol), "Symbol Method; VM has no Method class_name arm.", "crates/iris-eval/src/source_runtime.rs:10601"),
        row!(Closure, $surface, "class_name", shapes![Both []], Known(Symbol), "Symbol Closure.", "crates/iris-eval/src/source_runtime.rs:10601; crates/iris-vm/src/machine/stdlib.rs:402"),
        row!(BoundMethod, $surface, "class_name", shapes![Both []], Known(Symbol), "Symbol BoundMethod.", "crates/iris-eval/src/source_runtime.rs:10601; crates/iris-vm/src/machine/stdlib.rs:399"),
        row!(Task, $surface, "class_name", shapes![Both []], Known(Symbol), "Symbol Task; wait/join/result/cancel are not installed methods.", "crates/iris-eval/src/source_runtime.rs:10886; crates/iris-vm/src/machine/stdlib.rs:396"),
        row!(ExceptionContext, $surface, "value", shapes![Both []], Unknown, "Raised payload; readonly. Reference getter ignores extra arguments.", "crates/iris-eval/src/source_runtime.rs:10522; crates/iris-vm/src/machine/stdlib.rs:783"),
        row!(ExceptionContext, $surface, "cause", shapes![Both []], Unknown, "Cause ExceptionContext or nil; readonly.", "crates/iris-eval/src/source_runtime.rs:10523; crates/iris-vm/src/machine/stdlib.rs:783"),
        row!(ExceptionContext, $surface, "suppressed", shapes![Both []], Known(ReadonlyArray), "Readonly suppressed contexts.", "crates/iris-eval/src/source_runtime.rs:10524; crates/iris-vm/src/machine/stdlib.rs:783"),
        row!(ExceptionContext, $surface, "re_raise_sites", shapes![Both []], Known(ReadonlyArray), "Readonly re-raise sites in occurrence order.", "crates/iris-eval/src/source_runtime.rs:10527; crates/iris-vm/src/machine/stdlib.rs:783"),
        row!(ExceptionContext, $surface, "original_stack", shapes![Both []], Known(ReadonlyArray), "Readonly original stack records.", "crates/iris-eval/src/source_runtime.rs:10528; crates/iris-vm/src/machine/stdlib.rs:783"),
        row!(ExceptionContext, $surface, "raise_location", shapes![Both []], Unknown, "SourceLocation or nil; no fabricated location.", "crates/iris-eval/src/source_runtime.rs:10531; crates/iris-vm/src/machine/stdlib.rs:783"),
        row!(ExceptionContext, $surface, "class_name", shapes![Both []], Known(Symbol), "Symbol ExceptionContext, not raised payload class.", "crates/iris-eval/src/source_runtime.rs:10535; crates/iris-vm/src/machine/stdlib.rs:783"),
        row!(ExceptionContext, $surface, "operation", shapes![Reference []], Unknown, "Reflection diagnostic Symbol or nil; reference only.", "crates/iris-eval/src/source_runtime.rs:10544"),
        row!(ExceptionContext, $surface, "caller_package", shapes![Reference []], Unknown, "Reflection diagnostic Symbol or nil; reference only.", "crates/iris-eval/src/source_runtime.rs:10544"),
        row!(ExceptionContext, $surface, "target_scope", shapes![Reference []], Unknown, "Reflection diagnostic Symbol or nil; reference only.", "crates/iris-eval/src/source_runtime.rs:10544"),
        row!(ExceptionContext, $surface, "denial_origin", shapes![Reference []], Unknown, "Reflection diagnostic Symbol or nil; reference only.", "crates/iris-eval/src/source_runtime.rs:10544"),
        row!(StackFrame, $surface, "callable_name", shapes![Reference []], Known(Symbol), "Callable-name Symbol; reference getter ignores arity.", "crates/iris-eval/src/source_runtime.rs:10624"),
        row!(StackFrame, $surface, "location", shapes![Reference []], Unknown, "Stored SourceLocation-like value; reference only.", "crates/iris-eval/src/source_runtime.rs:10627"),
    )+] };
}
const GETTERS: &[BuiltinMember] = getters![Instance, Property];
macro_rules! diagnostic_getters {
    ($surface:ident, $availability:ident) => { &[
        row!(ExceptionContext, $surface, "decoder", shapes![$availability []], Unknown, "Decoder Symbol or nil. Reference call/read; VM property only.", "crates/iris-eval/src/source_runtime.rs:10557; crates/iris-vm/src/machine/execute.rs:1086"),
        row!(ExceptionContext, $surface, "offset", shapes![$availability []], Unknown, "Diagnostic Integer offset or nil. Reference call/read; VM property only.", "crates/iris-eval/src/source_runtime.rs:10557; crates/iris-vm/src/machine/execute.rs:1086"),
        row!(ExceptionContext, $surface, "expected", shapes![$availability []], Unknown, "Expected Symbol or nil. Reference call/read; VM property only.", "crates/iris-eval/src/source_runtime.rs:10557; crates/iris-vm/src/machine/execute.rs:1086"),
        row!(SourceLocation, $surface, "line", shapes![$availability []], Known(Integer), "Line number. Reference call/read; VM property only.", "crates/iris-eval/src/source_runtime.rs:10618; crates/iris-vm/src/machine/execute.rs:1105"),
        row!(SourceLocation, $surface, "column", shapes![$availability []], Known(Integer), "Column number. Reference call/read; VM property only.", "crates/iris-eval/src/source_runtime.rs:10621; crates/iris-vm/src/machine/execute.rs:1105"),
        row!(RaiseSite, $surface, "location", shapes![$availability []], Unknown, "Stored SourceLocation-like value. Reference call/read; VM property only.", "crates/iris-eval/src/source_runtime.rs:10627; crates/iris-vm/src/machine/execute.rs:1100"),
    ] };
}
const DIAGNOSTIC_CALLS: &[BuiltinMember] = diagnostic_getters![Instance, Reference];
const DIAGNOSTIC_READS: &[BuiltinMember] = diagnostic_getters![Property, Both];
const PATH: &[BuiltinMember] = &[
    row!(
        SourceLocation,
        Instance,
        "path",
        shapes![Reference []],
        Known(Symbol),
        "Path Symbol; reference call only. VM property returns String.",
        "crates/iris-eval/src/source_runtime.rs:10614; crates/iris-vm/src/machine/execute.rs:1105"
    ),
    row!(
        SourceLocation,
        Property,
        "path",
        shapes![Both []],
        Unknown,
        "Reference path Symbol; VM path String.",
        "crates/iris-eval/src/source_runtime.rs:10614; crates/iris-vm/src/machine/execute.rs:1105"
    ),
];
const GROUPS: &[&[BuiltinMember]] = &[ITERATORS, GETTERS, DIAGNOSTIC_CALLS, DIAGNOSTIC_READS, PATH];
pub(super) const ROWS: &[BuiltinMember] = &flatten::<{ count(GROUPS) }>(GROUPS);
