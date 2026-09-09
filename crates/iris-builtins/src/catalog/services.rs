use super::*;
use BuiltinType::{
    Array, FfiLibrary, Float64, Gate, Hash, Integer, Iteration, Nil, ReadonlyArray, String, Symbol,
    Tuple,
};

const REST: Parameter = Parameter {
    label: "arg1",
    kind: ParameterKind::Rest,
    type_label: None,
    optional: true,
};
const BLOCK: Parameter = Parameter {
    label: "callback",
    kind: ParameterKind::Block,
    type_label: Some("Closure"),
    optional: false,
};
const CLASS: Parameter = positional("arg1", Some("Class"));
const SELECTOR: Parameter = positional("arg2", Some("Symbol"));
const METHOD: Parameter = positional("arg1", Some("Method"));
const ARGUMENTS: Parameter = positional("arg3", Some("Array"));

macro_rules! encodings {
    ($($name:literal),+) => { &[$(
        service!($name, Service, "decode", shapes![Both [ARG1], Both [ARG1, keyword("errors", "Symbol")]], Known(String), "Decodes Bytes/ByteArray. Only errors: :replace selects lossy handling; other modes stay strict, no ignore implementation. Both backends currently use UTF-8 lossy fallback even for malformed UTF-16. Ignored tail is not a declared rest parameter.", "crates/iris-eval/src/source_runtime.rs:8979; crates/iris-vm/src/compile/calls.rs:389; crates/iris-vm/src/machine/stdlib.rs:1981"),
    )+] };
}
const ENCODINGS: &[BuiltinMember] = encodings![
    "Encoding::UTF_8",
    "Encoding::UTF_16LE",
    "Encoding::UTF_16BE",
    "Encoding::Latin_1"
];
const SERVICES: &[BuiltinMember] = &[
    service!(
        "",
        Global,
        "print",
        shapes![Both[REST]],
        Known(Nil),
        "Fallback bare helper: zero or more values rendered via text conversion. Authored bindings win where routed; reference checks main methods, VM special-cases after binding lookup.",
        "crates/iris-eval/src/source_runtime.rs:6506; crates/iris-vm/src/compile/calls.rs:247"
    ),
    service!(
        "",
        Global,
        "using",
        shapes![Both [ARG1, CALLBACK], Both [ARG1, BLOCK]],
        Unknown,
        "Resource scope; callback receives zero arguments. Returns body result or raises cleanup/body failure. Reference honors shadowing; VM special-cases before binding lookup.",
        "crates/iris-eval/src/source_runtime.rs:8809; crates/iris-vm/src/compile/calls.rs:206"
    ),
    service!(
        "",
        Global,
        "Integer",
        shapes![Both[INTEGER]],
        Known(Integer),
        "Integer-only conversion, not a String parser.",
        "crates/iris-runtime/src/kernel.rs:497; crates/iris-vm/src/machine/execute.rs:2086"
    ),
    service!(
        "",
        Global,
        "Float64",
        shapes![Reference[ARG1], Vm[INTEGER]],
        Known(Float64),
        "Reference accepts Integer/Float32/Float64; ordinary VM form accepts Integer only. Float64(-Infinity) is a separate special spelling, not evidence for a bare Infinity name.",
        "crates/iris-runtime/src/kernel.rs:772; crates/iris-vm/src/machine/execute.rs:2089; crates/iris-vm/src/compile/calls.rs:277"
    ),
    service!(
        "Iteration",
        Service,
        "yield",
        shapes![Both[ARG1]],
        Known(Iteration),
        "Creates yield signal carrying arg1.",
        "crates/iris-eval/src/source_runtime.rs:6404; crates/iris-vm/src/compile/calls.rs:339"
    ),
    service!(
        "Iteration",
        Property,
        "done",
        shapes![Both []],
        Known(Iteration),
        "Unique done signal; property only, not done().",
        "crates/iris-eval/src/source_runtime.rs:6968; crates/iris-vm/src/compile/expressions_lowering.rs:564"
    ),
    service!(
        "Unicode",
        Service,
        "version",
        shapes![Both []],
        Known(String),
        "Unicode table version. Reference ignores extra arguments; VM requires zero.",
        "crates/iris-eval/src/source_runtime.rs:9159; crates/iris-vm/src/compile/calls.rs:535"
    ),
    service!(
        "JSON",
        Service,
        "encode",
        shapes![Both [ARG1], Both [ARG1, keyword("canonical", "Bool")]],
        Known(String),
        "Encodes eligible serializable representation. Bool true requests canonical ordering. Extra tail entries may be ignored, not declared rest parameters.",
        "crates/iris-eval/src/source_runtime.rs:9163; crates/iris-vm/src/machine/stdlib/json.rs:65"
    ),
    service!(
        "JSON",
        Service,
        "decode",
        shapes![Both [ARG1], Both [ARG1, keyword("depth", "Integer")]],
        Unknown,
        "Decodes String/Bytes/ByteArray to nil/Bool/Integer/String/Array/Hash. Direct depth: option, not limits:. No nominal result inference.",
        "crates/iris-eval/src/source_runtime.rs:9178; crates/iris-vm/src/machine/stdlib/json.rs:27"
    ),
    service!(
        "IrisValue",
        Service,
        "encode",
        shapes![Both[ARG1]],
        Unknown,
        "Eligibility/representation adapter: returns validated serializable representation, NOT Bytes or a wire encoding. Additional ignored tail is not a declared rest parameter.",
        "crates/iris-eval/src/source_runtime.rs:9086; crates/iris-vm/src/machine/stdlib.rs:1460"
    ),
    service!(
        "IrisValue",
        Service,
        "decode",
        shapes![Both [positional("arg1", Some("Hash"))], Both [positional("arg1", Some("Hash")), keyword("element_limit", "Integer")]],
        Unknown,
        "Validates Hash stream magic/format_version, then returns payload or nominal factory result. No canonical default signature inferred from implementation fallback limit.",
        "crates/iris-eval/src/source_runtime.rs:9097; crates/iris-vm/src/machine/stdlib.rs:1487"
    ),
    service!(
        "FFI",
        Service,
        "open",
        shapes![Both [ARG1], Both [ARG1, keyword("declarations", "Hash")], Reference [ARG1, positional("arg2", Some("Hash"))]],
        Known(FfiLibrary),
        "Metadata adapter, no binary load. Path uses text conversion. Reference also accepts second positional Hash; VM reads declarations keyword only. Other tail entries are not declared rest parameters.",
        "crates/iris-eval/src/source_runtime.rs:9228; crates/iris-vm/src/machine/stdlib.rs:1794"
    ),
    service!(
        "Host",
        Service,
        "run",
        shapes![Both[positional("arg1", Some("Task"))]],
        Unknown,
        "Drives Task to result or re-raised failure. Unavailable inside async/Closure/open contexts; not a Task wait method.",
        "crates/iris-eval/src/source_runtime.rs:9387; crates/iris-vm/src/compile/calls.rs:577"
    ),
    service!(
        "Gate",
        Service,
        "new",
        shapes![Both []],
        Known(Gate),
        "Creates Gate. Reference ignores extra arguments; VM requires zero. A source service, not a Class constructor.",
        "crates/iris-eval/src/source_runtime.rs:8846; crates/iris-vm/src/compile/calls.rs:545"
    ),
    service!(
        "Gate",
        Service,
        "complete",
        shapes![Both [positional("arg1", Some("Gate"))], Both [positional("arg1", Some("Gate")), ARG2]],
        Known(Nil),
        "Completes Gate with arg2 when supplied; omitted value is nil. No keyword contract.",
        "crates/iris-eval/src/source_runtime.rs:8853; crates/iris-vm/src/compile/calls.rs:554"
    ),
    service!(
        "Diagnostics",
        Service,
        "discarded_contexts",
        shapes![Both []],
        Known(Array),
        "Array of discarded payloads. Reference ignores extra arguments; VM requires zero.",
        "crates/iris-eval/src/source_runtime.rs:8830; crates/iris-vm/src/compile/calls.rs:458"
    ),
    service!(
        "Diagnostics",
        Service,
        "unobserved_failures",
        shapes![Both []],
        Known(Array),
        "Array of diagnostic Tuples: marker, Task, captured payload/context and state. Reference ignores extra arguments.",
        "crates/iris-eval/src/source_runtime.rs:8833; crates/iris-vm/src/compile/calls.rs:589"
    ),
    service!(
        "Revision",
        Service,
        "subscribe",
        shapes![Both [CALLBACK], Both [CALLBACK, positional("arg2", Some("Integer"))]],
        Known(Nil),
        "Subscribes callback(event). Optional positional capacity is Integer in reference; VM treats other values as unbounded.",
        "crates/iris-eval/src/source_runtime.rs:8872; crates/iris-vm/src/machine/execute.rs:3364"
    ),
    service!(
        "Revision",
        Service,
        "flush",
        shapes![Both []],
        Known(Tuple),
        "Tuple(status Symbol, delivered Array, undelivered Integer, errors Array). Status delivered/incomplete; reference ignores extra arguments.",
        "crates/iris-eval/src/source_runtime.rs:5119; crates/iris-vm/src/machine/execute.rs:3386"
    ),
    service!(
        "Revision",
        Service,
        "shutdown",
        shapes![Both []],
        Known(Nil),
        "Shuts down delivery; reference ignores extra arguments.",
        "crates/iris-eval/src/source_runtime.rs:8895; crates/iris-vm/src/machine/execute.rs:3386"
    ),
    service!(
        "Revision",
        Service,
        "event_errors",
        shapes![Both []],
        Known(Array),
        "Recorded delivery errors; reference ignores extra arguments.",
        "crates/iris-eval/src/source_runtime.rs:8898; crates/iris-vm/src/machine/execute.rs:3484"
    ),
    service!(
        "RevisionHistory",
        Service,
        "events",
        shapes![Both [INTEGER, positional("arg2", Some("Integer"))]],
        Known(Array),
        "Integer commit IDs between inclusive bounds, not Revision objects.",
        "crates/iris-eval/src/source_runtime.rs:8904; crates/iris-vm/src/machine/execute.rs:3488"
    ),
    service!(
        "RevisionHistory",
        Service,
        "recover",
        shapes![Both [INTEGER, positional("arg2", Some("Integer"))]],
        Known(Array),
        "Integer commit IDs between inclusive bounds; requires configured sink.",
        "crates/iris-eval/src/source_runtime.rs:8939; crates/iris-vm/src/machine/execute.rs:3516"
    ),
    service!(
        "RevisionHistory",
        Service,
        "configure_sink",
        shapes![Both [], Both [ARG1]],
        Known(Nil),
        "Partial host adapter: argument ignored, configures in-memory history copy only, not persistent storage.",
        "crates/iris-eval/src/source_runtime.rs:8927; crates/iris-vm/src/machine/execute.rs:3507"
    ),
    service!(
        "Reflection::Object",
        Service,
        "list_ivars",
        shapes![Reference[ARG1]],
        Known(Array),
        "Array of ivar Symbols on Object/Class; permission checked; reference only.",
        "crates/iris-eval/src/source_runtime.rs:9574"
    ),
    service!(
        "Reflection::Object",
        Service,
        "get_ivar",
        shapes![Both [ARG1, SELECTOR]],
        Unknown,
        "Stored value. Reference accepts Object/Class; VM primarily Object; permission checked.",
        "crates/iris-eval/src/source_runtime.rs:9581; crates/iris-vm/src/machine/execute.rs:3281"
    ),
    service!(
        "Reflection::Object",
        Service,
        "set_ivar",
        shapes![Both [ARG1, SELECTOR, ARG3]],
        Unknown,
        "Stores and returns arg3. Reference accepts Object/Class; VM primarily Object; permission checked.",
        "crates/iris-eval/src/source_runtime.rs:9589; crates/iris-vm/src/machine/execute.rs:3281"
    ),
    service!(
        "Reflection::Object",
        Service,
        "remove_ivar",
        shapes![Reference [ARG1, SELECTOR]],
        Unknown,
        "Removed value; missing ivar raises. Object/Class target, permission checked; reference only.",
        "crates/iris-eval/src/source_runtime.rs:9597"
    ),
    service!(
        "Reflection::Class",
        Service,
        "method",
        shapes![Both [CLASS, SELECTOR]],
        Unknown,
        "Method or nil for target Class and selector Symbol.",
        "crates/iris-eval/src/source_runtime.rs:9605; crates/iris-vm/src/machine/execute.rs:2918"
    ),
    service!(
        "Reflection::Module",
        Service,
        "method",
        shapes![Both [SYMBOL, SELECTOR]],
        Unknown,
        "Method for module-name Symbol and selector. Missing method raises UnsupportedConstruct in reference.",
        "crates/iris-eval/src/source_runtime.rs:9605; crates/iris-vm/src/machine/execute.rs:3183"
    ),
    service!(
        "Reflection::Class",
        Service,
        "invoke",
        shapes![Both [METHOD, ARG2, ARGUMENTS]],
        Unknown,
        "Invokes retained Method with receiver and argument Array.",
        "crates/iris-eval/src/source_runtime.rs:9611; crates/iris-vm/src/machine/execute.rs:2938"
    ),
    service!(
        "Reflection::Module",
        Service,
        "invoke",
        shapes![Both [METHOD, ARG2, ARGUMENTS]],
        Unknown,
        "Invokes retained module-owned Method with receiver and argument Array.",
        "crates/iris-eval/src/source_runtime.rs:9617; crates/iris-vm/src/machine/execute.rs:2938"
    ),
    service!(
        "Reflection::Class",
        Service,
        "remove_module",
        shapes![Both [CLASS, SELECTOR]],
        Known(Nil),
        "Removes module Symbol from Class when permitted.",
        "crates/iris-eval/src/source_runtime.rs:9631; crates/iris-vm/src/machine/execute.rs:2895"
    ),
    service!(
        "Reflection::Class",
        Service,
        "remove_contract",
        shapes![Both [CLASS, positional("arg2", Some("Contract"))]],
        Known(Nil),
        "Only undeclared conformance can be removed (no-op); declared Contract refuses.",
        "crates/iris-eval/src/source_runtime.rs:9641; crates/iris-vm/src/machine/execute.rs:2895"
    ),
    service!(
        "Reflection::Class",
        Service,
        "set_superclass",
        shapes![Both [CLASS, positional("arg2", Some("Class"))]],
        Known(Nil),
        "Changes target superclass when permitted.",
        "crates/iris-eval/src/source_runtime.rs:9652; crates/iris-vm/src/machine/execute.rs:2895"
    ),
    service!(
        "Reflection::Class",
        Service,
        "properties",
        shapes![Both[CLASS]],
        Known(ReadonlyArray),
        "Readonly property Symbols.",
        "crates/iris-eval/src/source_runtime.rs:9728; crates/iris-vm/src/machine/execute.rs:3150"
    ),
    service!(
        "Reflection::Class",
        Service,
        "revision",
        shapes![Both[CLASS]],
        Known(Hash),
        "Hash with Symbol keys number and commit_id, not Revision object.",
        "crates/iris-eval/src/source_runtime.rs:9696; crates/iris-vm/src/machine/execute.rs:3150"
    ),
    service!(
        "Reflection::Class",
        Service,
        "ancestors",
        shapes![Reference[CLASS]],
        Known(Array),
        "Array of Classes, Modules filtered out; reference only.",
        "crates/iris-eval/src/source_runtime.rs:9719; crates/iris-eval/src/source_runtime.rs:10062"
    ),
    service!(
        "Reflection::Class",
        Service,
        "define_method",
        shapes![Both [CLASS, SELECTOR, CALLBACK]],
        Known(Nil),
        "Defines Symbol selector with Closure body; VM requires literal Closure AST.",
        "crates/iris-eval/src/source_runtime.rs:9740; crates/iris-vm/src/compile/calls.rs:351"
    ),
    service!(
        "Reflection::Class",
        Service,
        "define_property",
        shapes![Reference [CLASS, SELECTOR, CALLBACK]],
        Known(Nil),
        "Defines property with Closure initializer; service form reference-only.",
        "crates/iris-eval/src/source_runtime.rs:9734"
    ),
    service!(
        "Reflection::Contract",
        Service,
        "requirement",
        shapes![Both [positional("arg1", Some("Contract")), SELECTOR]],
        Unknown,
        "Requirement Hash with reified :return_type, or nil; accepts generic Contract arguments. Not direct Contract.requirement.",
        "crates/iris-eval/src/source_runtime.rs:9684; crates/iris-vm/src/machine/execute.rs:3199"
    ),
    service!(
        "Reflection::Package",
        Service,
        "identity",
        shapes![Reference []],
        Known(Array),
        "Array [package Symbol, major Integer]; reference ignores extra arguments.",
        "crates/iris-eval/src/source_runtime.rs:9553"
    ),
    service!(
        "Reflection::Package",
        Service,
        "version",
        shapes![Reference []],
        Unknown,
        "Version Symbol or nil; reference ignores extra arguments.",
        "crates/iris-eval/src/source_runtime.rs:9557"
    ),
    service!(
        "Reflection::Package",
        Service,
        "dependencies",
        shapes![Reference []],
        Known(ReadonlyArray),
        "Readonly dependency Arrays; reference ignores extra arguments.",
        "crates/iris-eval/src/source_runtime.rs:9561"
    ),
    service!(
        "Reflection::Package",
        Service,
        "module_status",
        shapes![Reference[SYMBOL]],
        Known(Symbol),
        "Symbol failed, initialized or absent for module Symbol; reference only.",
        "crates/iris-eval/src/source_runtime.rs:9415"
    ),
    service!(
        "Reflection::Package",
        Service,
        "reload",
        shapes![Reference [], Reference [ARG1]],
        Known(Integer),
        "Partial adapter: count of cleared failed-module marks; argument ignored, no reload IO.",
        "crates/iris-eval/src/source_runtime.rs:9434"
    ),
    service!(
        "Reflection::Package",
        Service,
        "load",
        shapes![Reference [SYMBOL], Reference [SYMBOL, ARG2]],
        Known(Symbol),
        "Partial adapter: already-prelinked module Symbol handle; second argument ignored, no package fetch.",
        "crates/iris-eval/src/source_runtime.rs:9442"
    ),
    service!(
        "Reflection::Package",
        Service,
        "upgrade",
        shapes![Reference[SYMBOL]],
        Unknown,
        "Partial version/hook transaction for target version Symbol; hook result or nil, not complete package manager.",
        "crates/iris-eval/src/source_runtime.rs:9488"
    ),
    service!(
        "Package",
        Service,
        "validate",
        shapes![Both [ARG1], Both [ARG1, keyword("core_abi", "Bool")], Both [ARG1, keyword("replaces_core_regex_literals", "Bool")], Both [ARG1, keyword("core_abi", "Bool"), keyword("replaces_core_regex_literals", "Bool")]],
        Known(Symbol),
        "Validation adapter: ignores arg1, refuses core claims, otherwise Symbol validated. VM detects literal true flags at compile time; reference checks evaluated Bool. Not full manifest validation; ignored tail is not a declared rest parameter.",
        "crates/iris-eval/src/source_runtime.rs:9067; crates/iris-vm/src/compile/calls.rs:468"
    ),
];
const GROUPS: &[&[BuiltinMember]] = &[ENCODINGS, SERVICES];
pub(super) const ROWS: &[BuiltinMember] = &flatten::<{ count(GROUPS) }>(GROUPS);
