use super::*;
use BuiltinType::{Bool, Integer};

macro_rules! numeric_rows {
    ($($family:ident),+) => {
        &[$(
            row!($family, Instance, "+", shapes![Both [ARG1]], Unknown, "Adds numeric operands at their common width; two Integers produce Integer.", "crates/iris-runtime/src/kernel.rs:596; crates/iris-runtime/src/numeric.rs"),
            row!($family, Instance, "-", shapes![Both [ARG1]], Unknown, "Subtracts numeric operands at their common width; two Integers produce Integer.", "crates/iris-runtime/src/kernel.rs:597"),
            row!($family, Instance, "*", shapes![Both [ARG1]], Unknown, "Multiplies numeric operands at their common width; two Integers produce Integer.", "crates/iris-runtime/src/kernel.rs:598"),
            row!($family, Instance, "/", shapes![Both [ARG1]], Unknown, "Integer/Integer yields Float64; otherwise the common floating width.", "crates/iris-runtime/src/kernel.rs:599; crates/iris-runtime/src/numeric.rs:100"),
            row!($family, Instance, "**", shapes![Both [ARG1]], Unknown, "Integer with nonnegative Integer exponent yields Integer; negative exponent yields Float64. Float operands preserve/promote width.", "crates/iris-runtime/src/kernel.rs:600; crates/iris-runtime/src/numeric.rs:135"),
            row!($family, Instance, "negate", shapes![Both []], Known(BuiltinType::$family), "Negates in the same numeric family. Unary minus lowers here; runtime ignores extra arguments.", "crates/iris-runtime/src/kernel.rs:651"),
            row!($family, Instance, "mul_add", shapes![Both [ARG1, ARG2]], Unknown, "Fused multiply-add: Float64 if any operand is Float64, otherwise Float32, including all-Integer operands.", "crates/iris-runtime/src/kernel.rs:799; crates/iris-runtime/src/numeric.rs:119"),
        )+]
    };
}
const NUMERIC: &[BuiltinMember] = numeric_rows![Integer, Float32, Float64];

macro_rules! float_rows {
    ($($family:ident),+) => {
        &[$(
            row!($family, Instance, "is_nan", shapes![Both []], Known(Bool), "Tests NaN interchange bits without arithmetic.", "crates/iris-runtime/src/kernel.rs:844"),
            row!($family, Instance, "is_signaling_nan", shapes![Both []], Known(Bool), "Tests signaling NaN without quieting it.", "crates/iris-runtime/src/kernel.rs:844"),
            row!($family, Instance, "is_infinite", shapes![Both []], Known(Bool), "Tests infinity interchange bits.", "crates/iris-runtime/src/kernel.rs:844"),
            row!($family, Instance, "is_finite", shapes![Both []], Known(Bool), "Tests finite interchange bits.", "crates/iris-runtime/src/kernel.rs:844"),
            row!($family, Instance, "is_normal", shapes![Both []], Known(Bool), "Tests normal floating representation.", "crates/iris-runtime/src/kernel.rs:844"),
            row!($family, Instance, "is_subnormal", shapes![Both []], Known(Bool), "Tests subnormal floating representation.", "crates/iris-runtime/src/kernel.rs:844"),
            row!($family, Instance, "is_zero", shapes![Both []], Known(Bool), "Tests either signed zero.", "crates/iris-runtime/src/kernel.rs:844"),
            row!($family, Instance, "sign_bit", shapes![Both []], Known(Bool), "Reads the interchange sign bit.", "crates/iris-runtime/src/kernel.rs:844"),
            row!($family, Instance, "to_bits", shapes![Both []], Known(Integer), "Returns width-specific unsigned interchange bit pattern.", "crates/iris-runtime/src/kernel.rs:666"),
            row!($family, Class, "from_bits", shapes![Both [INTEGER]], Known(BuiltinType::$family), "Constructs matching float width from range-checked Integer bits.", "crates/iris-runtime/src/kernel.rs:753; crates/iris-vm/src/compile/calls.rs:794"),
            row!($family, Class, "nan", shapes![Both []], Known(BuiltinType::$family), "Returns NaN; requires the float Class, not an instance.", "crates/iris-runtime/src/kernel.rs:780"),
            row!($family, Class, "infinity", shapes![Both []], Known(BuiltinType::$family), "Returns positive infinity; requires the float Class.", "crates/iris-runtime/src/kernel.rs:780"),
            row!($family, Property, "nan", shapes![Both []], Known(BuiltinType::$family), "Class-side read of NaN, not an instance property.", "crates/iris-vm/src/machine/execute.rs:1360; crates/iris-eval/src/source_runtime.rs:8734"),
            row!($family, Property, "infinity", shapes![Both []], Known(BuiltinType::$family), "Class-side read of positive infinity, not an instance property.", "crates/iris-vm/src/machine/execute.rs:1360; crates/iris-eval/src/source_runtime.rs:8734"),
        )+]
    };
}
const FLOATS: &[BuiltinMember] = float_rows![Float32, Float64];
const INTEGERS: &[BuiltinMember] = &[
    row!(
        Integer,
        Instance,
        "div",
        shapes![Both[INTEGER]],
        Known(Integer),
        "Floor division of two Integers.",
        "crates/iris-runtime/src/kernel.rs:601"
    ),
    row!(
        Integer,
        Instance,
        "mod",
        shapes![Both[INTEGER]],
        Known(Integer),
        "Integer modulo under floor division.",
        "crates/iris-runtime/src/kernel.rs:602"
    ),
    row!(
        Integer,
        Instance,
        "<<",
        shapes![Both[INTEGER]],
        Known(Integer),
        "Integer left shift.",
        "crates/iris-runtime/src/kernel.rs:736"
    ),
    row!(
        Integer,
        Instance,
        ">>",
        shapes![Both[INTEGER]],
        Known(Integer),
        "Integer right shift.",
        "crates/iris-runtime/src/kernel.rs:736"
    ),
    row!(
        Integer,
        Instance,
        "&",
        shapes![Both[INTEGER]],
        Known(Integer),
        "Infinite two's-complement bitwise AND.",
        "crates/iris-runtime/src/kernel.rs:610"
    ),
    row!(
        Integer,
        Instance,
        "|",
        shapes![Both[INTEGER]],
        Known(Integer),
        "Infinite two's-complement bitwise OR.",
        "crates/iris-runtime/src/kernel.rs:613"
    ),
    row!(
        Integer,
        Instance,
        "^",
        shapes![Both[INTEGER]],
        Known(Integer),
        "Infinite two's-complement bitwise XOR.",
        "crates/iris-runtime/src/kernel.rs:616"
    ),
    row!(
        Integer,
        Instance,
        "~",
        shapes![Both []],
        Known(Integer),
        "Integer complement; intended unary form, runtime ignores extra arguments.",
        "crates/iris-runtime/src/kernel.rs:605"
    ),
];
const GROUPS: &[&[BuiltinMember]] = &[NUMERIC, FLOATS, INTEGERS];
pub(super) const ROWS: &[BuiltinMember] = &flatten::<{ count(GROUPS) }>(GROUPS);
