use core::{cmp::Ordering, fmt};
use std::error::Error;

use crate::{
    BuiltinClass, ClassError, ClassId, ClassRegistry, DispatchError, DispatchOutcome, IntegerValue,
    MethodBody, Numeric, NumericError, NumericValue, Selector, StaticSpine, Value, Visibility,
};

/// Built-in selector identities executed by the native runtime kernel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeSelector {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Div,
    Mod,
    ShiftLeft,
    ShiftRight,
    BitwiseNot,
    Equal,
    NotEqual,
    Less,
    Compare,
    Negate,
    FromBits,
    Nan,
    Infinity,
    MulAdd,
}

impl NativeSelector {
    pub const fn id(self) -> Selector {
        Selector::new(self.raw())
    }
    pub fn from_source(source: &str) -> Option<Self> {
        match source {
            "+" => Some(Self::Add),
            "-" => Some(Self::Subtract),
            "*" => Some(Self::Multiply),
            "/" => Some(Self::Divide),
            "**" => Some(Self::Power),
            "div" => Some(Self::Div),
            "mod" => Some(Self::Mod),
            "<<" => Some(Self::ShiftLeft),
            ">>" => Some(Self::ShiftRight),
            "~" => Some(Self::BitwiseNot),
            "==" => Some(Self::Equal),
            "!=" => Some(Self::NotEqual),
            "<" => Some(Self::Less),
            "<=>" => Some(Self::Compare),
            "negate" => Some(Self::Negate),
            "from_bits" => Some(Self::FromBits),
            "nan" => Some(Self::Nan),
            "infinity" => Some(Self::Infinity),
            "mul_add" => Some(Self::MulAdd),
            _ => None,
        }
    }
    const fn raw(self) -> u64 {
        match self {
            Self::Add => 1,
            Self::Subtract => 2,
            Self::Multiply => 3,
            Self::Divide => 4,
            Self::Power => 5,
            Self::Div => 6,
            Self::Mod => 7,
            Self::ShiftLeft => 8,
            Self::ShiftRight => 9,
            Self::BitwiseNot => 10,
            Self::Equal => 11,
            Self::Less => 12,
            Self::Compare => 13,
            Self::Negate => 14,
            Self::FromBits => 15,
            Self::Nan => 16,
            Self::Infinity => 17,
            Self::MulAdd => 18,
            Self::NotEqual => 19,
        }
    }
    const fn from_raw(raw: u64) -> Option<Self> {
        match raw {
            1 => Some(Self::Add),
            2 => Some(Self::Subtract),
            3 => Some(Self::Multiply),
            4 => Some(Self::Divide),
            5 => Some(Self::Power),
            6 => Some(Self::Div),
            7 => Some(Self::Mod),
            8 => Some(Self::ShiftLeft),
            9 => Some(Self::ShiftRight),
            10 => Some(Self::BitwiseNot),
            11 => Some(Self::Equal),
            12 => Some(Self::Less),
            13 => Some(Self::Compare),
            14 => Some(Self::Negate),
            15 => Some(Self::FromBits),
            16 => Some(Self::Nan),
            17 => Some(Self::Infinity),
            18 => Some(Self::MulAdd),
            19 => Some(Self::NotEqual),
            _ => None,
        }
    }
}

/// A recoverable native-kernel failure.
#[derive(Clone, Debug, PartialEq)]
pub enum KernelError {
    Class(ClassError),
    Dispatch(DispatchError),
    Numeric(NumericError),
    Arity,
    Type,
    Identity,
    MissingMethod,
}
impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Class(error) => error.fmt(f),
            Self::Dispatch(error) => write!(f, "dispatch error: {error:?}"),
            Self::Numeric(error) => error.fmt(f),
            Self::Arity => f.write_str("wrong Iris method arity"),
            Self::Type => f.write_str("wrong Iris receiver or argument type"),
            Self::Identity => f.write_str("identity requires identity-bearing operands"),
            Self::MissingMethod => f.write_str("Iris method is missing"),
        }
    }
}
impl Error for KernelError {}
impl From<ClassError> for KernelError {
    fn from(error: ClassError) -> Self {
        Self::Class(error)
    }
}
impl From<DispatchError> for KernelError {
    fn from(error: DispatchError) -> Self {
        Self::Dispatch(error)
    }
}
impl From<NumericError> for KernelError {
    fn from(error: NumericError) -> Self {
        Self::Numeric(error)
    }
}

/// Built-in logical Classes with native method bodies selected by ordinary dispatch.
#[derive(Debug)]
pub struct Kernel {
    registry: ClassRegistry,
    classes: [(BuiltinClass, ClassId); 5],
}

impl Kernel {
    pub fn new() -> Result<Self, KernelError> {
        let mut registry = ClassRegistry::new();
        let mut classes = [(BuiltinClass::Nil, ClassId::new(0)); 5];
        for (index, kind) in [
            BuiltinClass::Nil,
            BuiltinClass::Bool,
            BuiltinClass::Integer,
            BuiltinClass::Float32,
            BuiltinClass::Float64,
        ]
        .into_iter()
        .enumerate()
        {
            classes[index] = (
                kind,
                registry.define_builtin_class(kind, StaticSpine::new(1), None)?,
            );
        }
        let mut kernel = Self { registry, classes };
        kernel.install(
            BuiltinClass::Integer,
            &[
                NativeSelector::Add,
                NativeSelector::Subtract,
                NativeSelector::Multiply,
                NativeSelector::Divide,
                NativeSelector::Power,
                NativeSelector::Div,
                NativeSelector::Mod,
                NativeSelector::ShiftLeft,
                NativeSelector::ShiftRight,
                NativeSelector::BitwiseNot,
                NativeSelector::Equal,
                NativeSelector::NotEqual,
                NativeSelector::Less,
                NativeSelector::Compare,
                NativeSelector::Negate,
                NativeSelector::MulAdd,
            ],
        )?;
        kernel.install(
            BuiltinClass::Float32,
            &[
                NativeSelector::Add,
                NativeSelector::Subtract,
                NativeSelector::Multiply,
                NativeSelector::Divide,
                NativeSelector::Power,
                NativeSelector::Equal,
                NativeSelector::NotEqual,
                NativeSelector::Less,
                NativeSelector::Compare,
                NativeSelector::Negate,
                NativeSelector::FromBits,
                NativeSelector::Nan,
                NativeSelector::Infinity,
                NativeSelector::MulAdd,
            ],
        )?;
        kernel.install(
            BuiltinClass::Float64,
            &[
                NativeSelector::Add,
                NativeSelector::Subtract,
                NativeSelector::Multiply,
                NativeSelector::Divide,
                NativeSelector::Power,
                NativeSelector::Equal,
                NativeSelector::NotEqual,
                NativeSelector::Less,
                NativeSelector::Compare,
                NativeSelector::Negate,
                NativeSelector::FromBits,
                NativeSelector::Nan,
                NativeSelector::Infinity,
                NativeSelector::MulAdd,
            ],
        )?;
        Ok(kernel)
    }
    pub fn class(&self, kind: BuiltinClass) -> Result<ClassId, KernelError> {
        self.classes
            .iter()
            .find_map(|(candidate, class)| (*candidate == kind).then_some(*class))
            .ok_or(KernelError::Type)
    }

    pub fn registry_mut(&mut self) -> &mut ClassRegistry {
        &mut self.registry
    }
    pub fn send(
        &self,
        receiver: Value,
        selector: NativeSelector,
        arguments: &[Value],
    ) -> Result<Value, KernelError> {
        let class = self.class_of(&receiver)?;
        match self.registry.dispatch(class, selector.id())? {
            DispatchOutcome::Invoke(method) => NativeSelector::from_raw(method.body().raw())
                .ok_or(KernelError::MissingMethod)
                .and_then(|selected| self.invoke(selected, receiver, arguments)),
            DispatchOutcome::WouldInvokeMethodMissing { .. } => Err(KernelError::MissingMethod),
        }
    }
    pub fn construct(&self, class: ClassId, arguments: &[Value]) -> Result<Value, KernelError> {
        if class == self.class(BuiltinClass::Integer)? {
            return match arguments {
                [Value::Integer(value)] => Ok(Value::Integer(value.clone())),
                _ => Err(KernelError::Type),
            };
        }
        if class == self.class(BuiltinClass::Float64)? {
            return self.construct_float64(arguments);
        }
        Err(KernelError::Type)
    }
    pub fn same_identity(left: &Value, right: &Value) -> Result<bool, KernelError> {
        match (left, right) {
            (Value::Nil, Value::Nil)
            | (Value::Bool(false), Value::Bool(false))
            | (Value::Bool(true), Value::Bool(true)) => Ok(true),
            (Value::Nil, _) | (Value::Bool(_), _) | (_, Value::Nil) | (_, Value::Bool(_)) => {
                Ok(false)
            }
            (Value::Object(left), Value::Object(right)) => Ok(left == right),
            _ => Err(KernelError::Identity),
        }
    }
    fn install(
        &mut self,
        kind: BuiltinClass,
        selectors: &[NativeSelector],
    ) -> Result<(), KernelError> {
        let class = self.class(kind)?;
        for selector in selectors {
            self.registry.publish_method(
                class,
                selector.id(),
                MethodBody::new(selector.raw()),
                Visibility::Public,
            )?;
        }
        Ok(())
    }
    fn class_of(&self, value: &Value) -> Result<ClassId, KernelError> {
        match value {
            Value::Nil => self.class(BuiltinClass::Nil),
            Value::Bool(_) => self.class(BuiltinClass::Bool),
            Value::Integer(_) => self.class(BuiltinClass::Integer),
            Value::Float32(_) => self.class(BuiltinClass::Float32),
            Value::Float64(_) => self.class(BuiltinClass::Float64),
            Value::Class(class) => Ok(*class),
            Value::Array(_) | Value::Object(_) => Err(KernelError::Type),
        }
    }
    fn invoke(
        &self,
        selector: NativeSelector,
        receiver: Value,
        arguments: &[Value],
    ) -> Result<Value, KernelError> {
        match selector {
            NativeSelector::Add => self.binary(receiver, arguments, Numeric::add),
            NativeSelector::Subtract => self.binary(receiver, arguments, Numeric::sub),
            NativeSelector::Multiply => self.binary(receiver, arguments, Numeric::mul),
            NativeSelector::Divide => self.binary(receiver, arguments, Numeric::div),
            NativeSelector::Power => self.binary(receiver, arguments, Numeric::pow),
            NativeSelector::Div => self.integer_binary(receiver, arguments, Numeric::integer_div),
            NativeSelector::Mod => self.integer_binary(receiver, arguments, Numeric::integer_mod),
            NativeSelector::ShiftLeft => self.shift(receiver, arguments, true),
            NativeSelector::ShiftRight => self.shift(receiver, arguments, false),
            NativeSelector::BitwiseNot => {
                Ok(Value::Integer(Numeric::integer_not(&numeric(&receiver)?)?))
            }
            NativeSelector::Equal => Ok(Value::Bool(Numeric::equal(
                &numeric(&receiver)?,
                &numeric_arg(arguments)?,
            ))),
            NativeSelector::NotEqual => Ok(Value::Bool(Numeric::not_equal(
                &numeric(&receiver)?,
                &numeric_arg(arguments)?,
            ))),
            NativeSelector::Less => Ok(Value::Bool(
                Numeric::compare(&numeric(&receiver)?, &numeric_arg(arguments)?)
                    == Some(Ordering::Less),
            )),
            NativeSelector::Compare => Ok(
                match Numeric::compare(&numeric(&receiver)?, &numeric_arg(arguments)?) {
                    Some(Ordering::Less) => Value::Integer((-1_i8).into()),
                    Some(Ordering::Equal) => Value::Integer(0_u8.into()),
                    Some(Ordering::Greater) => Value::Integer(1_u8.into()),
                    None => Value::Nil,
                },
            ),
            NativeSelector::Negate => Ok(value(Numeric::negate(&numeric(&receiver)?)?)),
            NativeSelector::FromBits => self.bits_from_integer(receiver, arguments),
            NativeSelector::Nan => self.special(receiver, arguments, true),
            NativeSelector::Infinity => self.special(receiver, arguments, false),
            NativeSelector::MulAdd => self.mul_add(receiver, arguments),
        }
    }
    fn binary(
        &self,
        receiver: Value,
        arguments: &[Value],
        operation: fn(&NumericValue, &NumericValue) -> Result<NumericValue, NumericError>,
    ) -> Result<Value, KernelError> {
        Ok(value(operation(
            &numeric(&receiver)?,
            &numeric_arg(arguments)?,
        )?))
    }
    fn integer_binary(
        &self,
        receiver: Value,
        arguments: &[Value],
        operation: fn(&NumericValue, &NumericValue) -> Result<IntegerValue, NumericError>,
    ) -> Result<Value, KernelError> {
        Ok(Value::Integer(operation(
            &numeric(&receiver)?,
            &numeric_arg(arguments)?,
        )?))
    }
    fn shift(
        &self,
        receiver: Value,
        arguments: &[Value],
        left: bool,
    ) -> Result<Value, KernelError> {
        let count = match numeric_arg(arguments)? {
            NumericValue::Integer(value) => value,
            _ => return Err(KernelError::Type),
        };
        let receiver = numeric(&receiver)?;
        Ok(Value::Integer(if left {
            Numeric::integer_shift_left(&receiver, &count)?
        } else {
            Numeric::integer_shift_right(&receiver, &count)?
        }))
    }
    fn bits_from_integer(
        &self,
        receiver: Value,
        arguments: &[Value],
    ) -> Result<Value, KernelError> {
        let bits = match numeric_arg(arguments)? {
            NumericValue::Integer(value) => value,
            _ => return Err(KernelError::Type),
        };
        match receiver {
            Value::Class(class) if class == self.class(BuiltinClass::Float32)? => {
                Ok(value(Numeric::float32_from_bits(&bits)?))
            }
            Value::Class(class) if class == self.class(BuiltinClass::Float64)? => {
                Ok(value(Numeric::float64_from_bits(&bits)?))
            }
            _ => Err(KernelError::Type),
        }
    }
    fn construct_float64(&self, arguments: &[Value]) -> Result<Value, KernelError> {
        let numeric = numeric_arg(arguments)?;
        Ok(match numeric {
            NumericValue::Integer(value) => Value::Float64(value.to_f64()),
            NumericValue::Float32(value) => Value::Float64(f64::from(value)),
            NumericValue::Float64(value) => Value::Float64(value),
        })
    }
    fn special(
        &self,
        receiver: Value,
        arguments: &[Value],
        nan: bool,
    ) -> Result<Value, KernelError> {
        if !arguments.is_empty() {
            return Err(KernelError::Arity);
        }
        match receiver {
            Value::Class(class) if class == self.class(BuiltinClass::Float32)? => {
                Ok(Value::Float32(if nan { f32::NAN } else { f32::INFINITY }))
            }
            Value::Class(class) if class == self.class(BuiltinClass::Float64)? => {
                Ok(Value::Float64(if nan { f64::NAN } else { f64::INFINITY }))
            }
            _ => Err(KernelError::Type),
        }
    }
    fn mul_add(&self, receiver: Value, arguments: &[Value]) -> Result<Value, KernelError> {
        if arguments.len() != 2 {
            return Err(KernelError::Arity);
        }
        Ok(value(Numeric::mul_add(
            &numeric(&receiver)?,
            &numeric(&arguments[0])?,
            &numeric(&arguments[1])?,
        )?))
    }
}
fn numeric(value: &Value) -> Result<NumericValue, KernelError> {
    match value {
        Value::Integer(value) => Ok(NumericValue::Integer(value.clone())),
        Value::Float32(value) => Ok(NumericValue::Float32(*value)),
        Value::Float64(value) => Ok(NumericValue::Float64(*value)),
        _ => Err(KernelError::Type),
    }
}
fn numeric_arg(arguments: &[Value]) -> Result<NumericValue, KernelError> {
    match arguments {
        [value] => numeric(value),
        _ => Err(KernelError::Arity),
    }
}
fn value(value: NumericValue) -> Value {
    match value {
        NumericValue::Integer(value) => Value::Integer(value),
        NumericValue::Float32(value) => Value::Float32(value),
        NumericValue::Float64(value) => Value::Float64(value),
    }
}
