#![no_std]
#![forbid(unsafe_code)]

mod catalog;
mod types;
pub use types::BuiltinType;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Surface {
    Instance,
    Class,
    Service,
    Global,
    Property,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ParameterKind {
    Positional,
    Keyword,
    Rest,
    Block,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Parameter {
    pub label: &'static str,
    pub kind: ParameterKind,
    pub type_label: Option<&'static str>,
    pub optional: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Availability {
    Both,
    Reference,
    Vm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReturnFact {
    Unknown,
    Known(BuiltinType),
    ArrayOf(BuiltinType),
    Receiver,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CallShape {
    pub parameters: &'static [Parameter],
    pub availability: Availability,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuiltinMember {
    pub owner: &'static str,
    pub receiver: Option<BuiltinType>,
    pub surface: Surface,
    pub selector: &'static str,
    pub shapes: &'static [CallShape],
    pub return_label: Option<&'static str>,
    pub result: ReturnFact,
    pub documentation: &'static str,
    pub evidence: &'static str,
}

pub const fn members() -> &'static [BuiltinMember] {
    &catalog::MEMBERS
}
pub const fn class_names() -> &'static [&'static str] {
    &[
        "Object", "Nil", "Bool", "Integer", "Float32", "Float64", "String",
    ]
}
pub const fn service_names() -> &'static [&'static str] {
    &[
        "Iteration",
        "Unicode",
        "Encoding::UTF_8",
        "Encoding::UTF_16LE",
        "Encoding::UTF_16BE",
        "Encoding::Latin_1",
        "JSON",
        "IrisValue",
        "FFI",
        "Host",
        "Gate",
        "Diagnostics",
        "Revision",
        "RevisionHistory",
        "Reflection::Object",
        "Reflection::Class",
        "Reflection::Module",
        "Reflection::Contract",
        "Reflection::Package",
        "Package",
    ]
}
