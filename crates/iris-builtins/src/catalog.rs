use crate::{
    Availability, BuiltinMember, BuiltinType, CallShape, Parameter, ParameterKind, ReturnFact,
    Surface,
};
use Availability::{Both, Reference, Vm};
use ReturnFact::{ArrayOf, Known, Receiver, Unknown};

const fn positional(label: &'static str, type_label: Option<&'static str>) -> Parameter {
    Parameter {
        label,
        kind: ParameterKind::Positional,
        type_label,
        optional: false,
    }
}
const fn keyword(label: &'static str, type_label: &'static str) -> Parameter {
    Parameter {
        label,
        kind: ParameterKind::Keyword,
        type_label: Some(type_label),
        optional: false,
    }
}
const ARG1: Parameter = positional("arg1", None);
const ARG2: Parameter = positional("arg2", None);
const ARG3: Parameter = positional("arg3", None);
const CALLBACK: Parameter = positional("callback", Some("Closure"));
const INTEGER: Parameter = positional("arg1", Some("Integer"));
const SYMBOL: Parameter = positional("arg1", Some("Symbol"));
const TEXT: Parameter = positional("arg1", Some("String"));

macro_rules! shapes {
    ($($availability:ident [$($parameter:expr),* $(,)?]),+ $(,)?) => {
        &[$(CallShape { parameters: &[$($parameter),*], availability: $availability }),+]
    };
}
macro_rules! row {
    ($family:ident, $surface:ident, $selector:literal, $shapes:expr, $result:expr, $doc:literal, $evidence:literal) => {
        BuiltinMember {
            owner: BuiltinType::$family.name(),
            receiver: Some(BuiltinType::$family),
            surface: Surface::$surface,
            selector: $selector,
            shapes: $shapes,
            return_label: match $result {
                Known(kind) => Some(kind.name()),
                ArrayOf(kind) => Some(kind.array_name()),
                Receiver => Some(BuiltinType::$family.name()),
                Unknown => None,
            },
            result: $result,
            documentation: $doc,
            evidence: $evidence,
        }
    };
}
macro_rules! service {
    ($owner:literal, $surface:ident, $selector:literal, $shapes:expr, $result:expr, $doc:literal, $evidence:literal) => {
        BuiltinMember {
            owner: $owner,
            receiver: None,
            surface: Surface::$surface,
            selector: $selector,
            shapes: $shapes,
            return_label: match $result {
                Known(kind) => Some(kind.name()),
                ArrayOf(kind) => Some(kind.array_name()),
                Receiver | Unknown => None,
            },
            result: $result,
            documentation: $doc,
            evidence: $evidence,
        }
    };
}

mod collections;
mod metadata;
mod numeric;
mod protocols;
mod services;
mod text;
mod values;

const GROUPS: &[&[BuiltinMember]] = &[
    numeric::ROWS,
    collections::ROWS,
    text::ROWS,
    values::ROWS,
    metadata::ROWS,
    services::ROWS,
    protocols::ROWS,
];
const fn count(groups: &[&[BuiltinMember]]) -> usize {
    let mut count = 0;
    let mut group = 0;
    while group < groups.len() {
        count += groups[group].len();
        group += 1;
    }
    count
}
const fn flatten<const SIZE: usize>(groups: &[&[BuiltinMember]]) -> [BuiltinMember; SIZE] {
    let mut result = [groups[0][0]; SIZE];
    let mut group = 0;
    let mut target = 0;
    while group < groups.len() {
        let mut source = 0;
        while source < groups[group].len() {
            result[target] = groups[group][source];
            target += 1;
            source += 1;
        }
        group += 1;
    }
    result
}
pub(super) static MEMBERS: [BuiltinMember; count(GROUPS)] = flatten(GROUPS);
