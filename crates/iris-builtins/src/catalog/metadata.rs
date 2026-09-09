use super::*;
use BuiltinType::{Array, Bool, Integer, Nil, ReadonlyArray, Symbol, Transformation, Type};

const CLASS_CALLS: &[BuiltinMember] = &[
    row!(
        Object,
        Class,
        "new",
        shapes![Both []],
        Known(BuiltinType::Object),
        "Allocates ordinary Object. Declared Class constructor signatures come from initialize; absent initialize ignores extra arguments. Does not imply intrinsic collection/scalar constructors.",
        "crates/iris-eval/src/source_runtime.rs:7991; crates/iris-runtime/src/runtime.rs:199; crates/iris-vm/src/machine/execute.rs:2418"
    ),
    row!(
        Class,
        Class,
        "new",
        &[],
        Unknown,
        "Allocates declared/closed Class instance. Constructor-specific shape comes from initialize, not a fixed builtin signature; absent initialize ignores arguments. Both backends support construction.",
        "crates/iris-eval/src/source_runtime.rs:7922; crates/iris-vm/src/machine/execute.rs:2418"
    ),
    row!(
        Class,
        Class,
        "open",
        shapes![Both[CALLBACK]],
        Unknown,
        "Runs callback(Class) in a transaction; result is callback result. VM requires source-resolved declared Class; closed generic opening refuses.",
        "crates/iris-eval/src/source_runtime.rs:2461; crates/iris-vm/src/compile/calls.rs:635"
    ),
    row!(
        Class,
        Class,
        "define_method",
        shapes![Both [SYMBOL, CALLBACK]],
        Known(Nil),
        "Stages Symbol selector with Closure body. VM requires literal Closure AST, not arbitrary stored Closure.",
        "crates/iris-eval/src/source_runtime.rs:2559; crates/iris-vm/src/compile/calls.rs:351"
    ),
    row!(
        Class,
        Class,
        "define_property",
        shapes![Both [SYMBOL, CALLBACK]],
        Known(Nil),
        "Stages Symbol property with Closure initializer.",
        "crates/iris-eval/src/source_runtime.rs:2632; crates/iris-vm/src/machine/stdlib.rs:899"
    ),
    row!(
        Class,
        Class,
        "alias_method",
        shapes![Both [SYMBOL, positional("arg2", Some("Symbol"))]],
        Known(Nil),
        "Aliases two Symbol selectors.",
        "crates/iris-eval/src/source_runtime/boundaries.rs:178; crates/iris-vm/src/machine/stdlib.rs:976"
    ),
    row!(
        Class,
        Class,
        "remove_method",
        shapes![Both[SYMBOL]],
        Known(Nil),
        "Removes own Symbol selector subject to static promises.",
        "crates/iris-eval/src/source_runtime.rs:8000; crates/iris-vm/src/machine/stdlib.rs:991"
    ),
    row!(
        Class,
        Class,
        "undef_method",
        shapes![Both[SYMBOL]],
        Known(Nil),
        "Blocks Symbol selector subject to static promises.",
        "crates/iris-eval/src/source_runtime.rs:8013; crates/iris-vm/src/machine/stdlib.rs:991"
    ),
    row!(
        Class,
        Class,
        "add_module",
        shapes![Both[SYMBOL]],
        Known(Nil),
        "Adds module-name Symbol to composition.",
        "crates/iris-eval/src/source_runtime.rs:8095; crates/iris-vm/src/machine/stdlib.rs:930"
    ),
    row!(
        Class,
        Class,
        "remove_module",
        shapes![Both[SYMBOL]],
        Known(Nil),
        "Removes module-name Symbol from composition.",
        "crates/iris-eval/src/source_runtime.rs:8095; crates/iris-vm/src/machine/stdlib.rs:930"
    ),
    row!(
        Class,
        Class,
        "remove_contract",
        shapes![Both[positional("arg1", Some("Contract"))]],
        Known(Nil),
        "No-op only when Contract was not declared; declared conformance cannot be removed.",
        "crates/iris-eval/src/source_runtime.rs:8080; crates/iris-vm/src/machine/stdlib.rs:952"
    ),
    row!(
        Class,
        Class,
        "set_superclass",
        shapes![Reference[positional("arg1", Some("Class"))]],
        Known(Nil),
        "Changes superclass when permitted; direct reference route. VM uses Reflection::Class service.",
        "crates/iris-eval/src/source_runtime.rs:8106"
    ),
    row!(
        Class,
        Class,
        "method",
        shapes![Both[SYMBOL]],
        Unknown,
        "Retained Method or nil by selector Symbol.",
        "crates/iris-eval/src/source_runtime.rs:8026; crates/iris-vm/src/machine/stdlib.rs:1007"
    ),
    row!(
        Class,
        Class,
        "invoke",
        shapes![Reference [positional("arg1", Some("Method")), ARG2, positional("arg3", Some("Array"))]],
        Unknown,
        "Invokes retained Method on arg2 using argument Array; direct reference route, VM uses service form.",
        "crates/iris-eval/src/source_runtime.rs:8039"
    ),
    row!(
        Module,
        Class,
        "invoke",
        shapes![Reference [positional("arg1", Some("Method")), ARG2, positional("arg3", Some("Array"))]],
        Unknown,
        "Module source-name route invokes retained Method; direct reference only.",
        "crates/iris-eval/src/source_runtime.rs:6553"
    ),
    row!(
        Module,
        Class,
        "method",
        shapes![Reference[SYMBOL]],
        Unknown,
        "Module source-name lookup of retained Method or nil; reference only.",
        "crates/iris-eval/src/source_runtime.rs:6541"
    ),
    row!(
        Module,
        Property,
        "modules",
        shapes![Reference []],
        Known(Array),
        "Array of module-name Symbols from source-name read; not modules().",
        "crates/iris-eval/src/source_runtime.rs:6997"
    ),
    row!(
        Contract,
        Instance,
        "view",
        shapes![Reference[positional("arg1", Some("Class"))]],
        Known(BuiltinType::ContractView),
        "ContractView around Class; reference only.",
        "crates/iris-eval/src/source_runtime.rs:8272"
    ),
    row!(
        ContractView,
        Instance,
        "respond_to_contract?",
        shapes![Reference[SYMBOL]],
        Known(Bool),
        "Probes qualified contract slot on view around Class; reference only.",
        "crates/iris-eval/src/source_runtime.rs:8281"
    ),
    row!(
        Type,
        Instance,
        "subtype?",
        shapes![Both[positional("arg1", Some("Type"))]],
        Known(Bool),
        "Nominal Type subtype relation; not a composed Type method.",
        "crates/iris-eval/src/source_runtime.rs:8485; crates/iris-vm/src/machine/stdlib.rs:1236"
    ),
    row!(
        Type,
        Instance,
        "assignable?",
        shapes![Both[positional("arg1", Some("Type"))]],
        Known(Bool),
        "Nominal Type assignability relation.",
        "crates/iris-eval/src/source_runtime.rs:8498; crates/iris-vm/src/machine/stdlib.rs:1236"
    ),
    row!(
        Transformation,
        Instance,
        "add_method",
        shapes![Reference [SYMBOL, CALLBACK]],
        Known(Transformation),
        "Fresh Transformation with staged selector/Closure entry; reference only.",
        "crates/iris-eval/src/source_runtime.rs:7960"
    ),
];
macro_rules! class_getters {
    ($surface:ident, $availability:ident) => { &[
        row!(Class, $surface, "name", shapes![$availability []], Unknown, "Class name Symbol or nil. Reference call/read; VM property only; reference ignores extra arguments.", "crates/iris-eval/src/source_runtime.rs:8171; crates/iris-vm/src/machine/execute.rs:1114"),
        row!(Class, $surface, "methods", shapes![$availability []], Known(ReadonlyArray), "Readonly selector Symbols. Reference call/read; VM property only.", "crates/iris-eval/src/source_runtime.rs:8179; crates/iris-vm/src/machine/execute.rs:1114"),
        row!(Class, $surface, "modules", shapes![$availability []], Known(Array), "Array of module Symbols. Reference call/read; VM property only.", "crates/iris-eval/src/source_runtime.rs:8203; crates/iris-vm/src/machine/execute.rs:1310"),
        row!(Class, $surface, "static_spine", shapes![$availability []], Known(Integer), "Static spine identity Integer. Reference call/read; VM property only.", "crates/iris-eval/src/source_runtime.rs:8136; crates/iris-vm/src/machine/execute.rs:1114"),
        row!(Class, $surface, "denied_capabilities", shapes![$availability []], Known(Array), "Array of denied-capability Symbols. Reference call/read; VM property only.", "crates/iris-eval/src/source_runtime.rs:8315; crates/iris-vm/src/machine/execute.rs:1114"),
        row!(Class, $surface, "decorators", shapes![$availability []], Known(ReadonlyArray), "Readonly ordered decorator Symbols. Reference call/read; VM property only.", "crates/iris-eval/src/source_runtime.rs:8222; crates/iris-vm/src/machine/execute.rs:1114"),
        row!(Class, $surface, "decorator_arguments", shapes![$availability []], Known(ReadonlyArray), "Nested readonly argument arrays. Reference call/read; VM property only.", "crates/iris-eval/src/source_runtime.rs:8242; crates/iris-vm/src/machine/execute.rs:1114"),
        row!(Class, $surface, "decorator_phases", shapes![$availability []], Known(ReadonlyArray), "Readonly phase Symbols. Reference call/read; VM property only.", "crates/iris-eval/src/source_runtime.rs:8245; crates/iris-vm/src/machine/execute.rs:1114"),
        row!(Class, $surface, "type", shapes![$availability []], Known(Type), "Reified nominal Type. Closed generic arguments require source .type handling; do not infer closed .type() preservation. VM property only.", "crates/iris-eval/src/source_runtime.rs:8416; crates/iris-vm/src/machine/execute.rs:1114"),
    ] };
}
const CLASS_READS: &[BuiltinMember] = class_getters![Property, Both];
const CLASS_GETTERS: &[BuiltinMember] = class_getters![Class, Reference];
macro_rules! other_class_getters {
    ($($surface:ident),+) => { &[$(
        row!(Class, $surface, "properties", shapes![Both []], Known(ReadonlyArray), "Readonly property Symbols; reference ignores extra arguments, VM requires zero.", "crates/iris-eval/src/source_runtime.rs:8126; crates/iris-vm/src/machine/stdlib.rs:884"),
        row!(Class, $surface, "active_revision", shapes![Both []], Known(Integer), "Active revision number, not Revision object; reference ignores extra arguments.", "crates/iris-eval/src/source_runtime.rs:8256; crates/iris-vm/src/machine/stdlib.rs:874"),
        row!(Class, $surface, "contracts", shapes![Both []], Known(Array), "Array of declared Contracts; reference ignores extra arguments.", "crates/iris-eval/src/source_runtime.rs:8246; crates/iris-vm/src/machine/stdlib.rs:1023"),
        row!(Class, $surface, "package", shapes![Reference []], Known(Symbol), "Package Symbol; reference only, ignores extra arguments.", "crates/iris-eval/src/source_runtime.rs:8133"),
        row!(Class, $surface, "runtime_superclass", shapes![Reference []], Unknown, "Superclass Class or nil; reference only.", "crates/iris-eval/src/source_runtime.rs:8145"),
        row!(Class, $surface, "mro", shapes![Reference []], Known(Array), "Array of Classes, Modules filtered out; reference only.", "crates/iris-eval/src/source_runtime.rs:8152; crates/iris-eval/src/source_runtime.rs:10062"),
        row!(Class, $surface, "ancestors", shapes![Reference []], Known(Array), "Array of Classes, Modules filtered out; reference only.", "crates/iris-eval/src/source_runtime.rs:8335; crates/iris-eval/src/source_runtime.rs:10062"),
        row!(Class, $surface, "meta_capabilities", shapes![Reference []], Known(Array), "Array of denied-capability Symbols; reference only.", "crates/iris-eval/src/source_runtime.rs:8153"),
    )+] };
}
const OTHER_CLASS: &[BuiltinMember] = other_class_getters![Class, Property];
macro_rules! value_getters {
    ($($surface:ident),+) => { &[$(
        row!(Type, $surface, "type", shapes![Both []], Receiver, "Nominal Type returns itself; reference ignores extra arguments.", "crates/iris-eval/src/source_runtime.rs:8450; crates/iris-vm/src/machine/stdlib.rs:1184"),
        row!(Type, $surface, "kind", shapes![Both []], Known(Symbol), "Symbol nominal; reference ignores extra arguments.", "crates/iris-eval/src/source_runtime.rs:8462; crates/iris-vm/src/machine/stdlib.rs:1181"),
        row!(Type, $surface, "package", shapes![Both []], Known(Symbol), "Nominal package Symbol.", "crates/iris-eval/src/source_runtime.rs:8482; crates/iris-vm/src/machine/stdlib.rs:1141"),
        row!(Type, $surface, "hash", shapes![Both []], Known(Integer), "Nominal publishable Type hash; not Class hash.", "crates/iris-eval/src/source_runtime.rs:8481; crates/iris-vm/src/machine/stdlib.rs:1152"),
        row!(Type, $surface, "arguments", shapes![Both []], Known(Array), "Array of closed Type arguments.", "crates/iris-eval/src/source_runtime.rs:8510; crates/iris-vm/src/machine/stdlib.rs:1171"),
        row!(Type, $surface, "members", shapes![Both []], Known(Array), "Array of selector Symbols; unlike composed Type members.", "crates/iris-eval/src/source_runtime.rs:8522; crates/iris-vm/src/machine/stdlib.rs:1196"),
        row!(ComposedType, $surface, "type", shapes![Both []], Receiver, "Composed Type returns itself.", "crates/iris-eval/src/source_runtime.rs:8450; crates/iris-vm/src/machine/stdlib.rs:1184"),
        row!(ComposedType, $surface, "kind", shapes![Both []], Known(Symbol), "Symbol never, union or intersection.", "crates/iris-eval/src/source_runtime.rs:8454; crates/iris-vm/src/machine/stdlib.rs:1226"),
        row!(ComposedType, $surface, "members", shapes![Both []], Known(Array), "Array of Type atoms, not selector Symbols.", "crates/iris-eval/src/source_runtime.rs:8463; crates/iris-vm/src/machine/stdlib.rs:1216"),
        row!(Contract, $surface, "parents", shapes![Reference []], Known(Array), "Array of parent Contracts; reference only.", "crates/iris-eval/src/source_runtime.rs:8304"),
        row!(Transformation, $surface, "empty", shapes![Reference []], Known(Transformation), "Empty Transformation; special Transformation source name denotes a value, not a Class. Reference only, ignores arity.", "crates/iris-eval/src/source_runtime.rs:7950; crates/iris-eval/src/source_method.rs:241"),
        row!(Transformation, $surface, "kind", shapes![Reference []], Known(Symbol), "Symbol class; reference only, ignores arity.", "crates/iris-eval/src/source_runtime.rs:7957"),
    )+] };
}
const VALUE_GETTERS: &[BuiltinMember] = value_getters![Instance, Property];
const GROUPS: &[&[BuiltinMember]] = &[
    CLASS_CALLS,
    CLASS_READS,
    CLASS_GETTERS,
    OTHER_CLASS,
    VALUE_GETTERS,
];
pub(super) const ROWS: &[BuiltinMember] = &flatten::<{ count(GROUPS) }>(GROUPS);
