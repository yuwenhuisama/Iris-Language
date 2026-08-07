use core::{cmp::Ordering, fmt};
use std::error::Error;

use crate::{
    BuiltinClass, Capability, ClassError, ClassId, ClassRegistry, DispatchError, DispatchOutcome,
    IntegerValue, MetaCapabilities, Method, MethodBody, Numeric, NumericError, NumericValue,
    Selector, StableHashError, StaticSpine, Value, Visibility,
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
    LessEqual,
    Greater,
    GreaterEqual,
    Compare,
    Negate,
    FromBits,
    Nan,
    Infinity,
    MulAdd,
    Hash,
    ToBool,
    IsNan,
    IsSignalingNan,
    IsInfinite,
    IsFinite,
    IsNormal,
    IsSubnormal,
    IsZero,
    SignBit,
    ToBits,
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
            "<=" => Some(Self::LessEqual),
            ">" => Some(Self::Greater),
            ">=" => Some(Self::GreaterEqual),
            "<=>" => Some(Self::Compare),
            "negate" => Some(Self::Negate),
            "from_bits" => Some(Self::FromBits),
            "nan" => Some(Self::Nan),
            "infinity" => Some(Self::Infinity),
            "mul_add" => Some(Self::MulAdd),
            "hash" => Some(Self::Hash),
            "to_bool" => Some(Self::ToBool),
            "is_nan" => Some(Self::IsNan),
            "is_signaling_nan" => Some(Self::IsSignalingNan),
            "is_infinite" => Some(Self::IsInfinite),
            "is_finite" => Some(Self::IsFinite),
            "is_normal" => Some(Self::IsNormal),
            "is_subnormal" => Some(Self::IsSubnormal),
            "is_zero" => Some(Self::IsZero),
            "sign_bit" => Some(Self::SignBit),
            "to_bits" => Some(Self::ToBits),
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
            Self::Hash => 20,
            Self::ToBool => 21,
            Self::LessEqual => 22,
            Self::Greater => 23,
            Self::GreaterEqual => 24,
            Self::IsNan => 25,
            Self::IsSignalingNan => 26,
            Self::IsInfinite => 27,
            Self::IsFinite => 28,
            Self::IsNormal => 29,
            Self::IsSubnormal => 30,
            Self::IsZero => 31,
            Self::SignBit => 32,
            Self::ToBits => 33,
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
            20 => Some(Self::Hash),
            21 => Some(Self::ToBool),
            22 => Some(Self::LessEqual),
            23 => Some(Self::Greater),
            24 => Some(Self::GreaterEqual),
            25 => Some(Self::IsNan),
            26 => Some(Self::IsSignalingNan),
            27 => Some(Self::IsInfinite),
            28 => Some(Self::IsFinite),
            29 => Some(Self::IsNormal),
            30 => Some(Self::IsSubnormal),
            31 => Some(Self::IsZero),
            32 => Some(Self::SignBit),
            33 => Some(Self::ToBits),
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
    StableHash(StableHashError),
    Arity,
    Type,
    Identity,
    MessageNotFound {
        receiver: ClassId,
        selector: Selector,
        arity: usize,
    },
}
impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Class(error) => error.fmt(f),
            Self::Dispatch(error) => write!(f, "dispatch error: {error:?}"),
            Self::Numeric(error) => error.fmt(f),
            Self::StableHash(error) => error.fmt(f),
            Self::Arity => f.write_str("wrong Iris method arity"),
            Self::Type => f.write_str("wrong Iris receiver or argument type"),
            Self::Identity => f.write_str("identity requires identity-bearing operands"),
            Self::MessageNotFound { .. } => f.write_str("Iris method is missing"),
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
impl From<StableHashError> for KernelError {
    fn from(error: StableHashError) -> Self {
        Self::StableHash(error)
    }
}

/// Built-in logical Classes with native method bodies selected by ordinary dispatch.
#[derive(Debug)]
pub struct Kernel {
    classes: [(BuiltinClass, ClassId); 7],
}

impl Kernel {
    /// Defines the built-in Classes inside the supplied registry.
    ///
    /// The registry is borrowed rather than owned so that built-in and declared
    /// Classes share ONE `ClassId` space. Two registries would each allocate ids
    /// from zero, making the same id denote different Classes and leaving
    /// `IRIS-V1-RUNTIME-C005` unsatisfiable, since `Object` could never appear in
    /// a declared Class's MRO.
    pub fn new(registry: &mut ClassRegistry) -> Result<Self, KernelError> {
        let mut classes = [(BuiltinClass::Object, ClassId::new(0)); 7];
        for (index, kind) in [
            BuiltinClass::Object,
            BuiltinClass::Nil,
            BuiltinClass::Bool,
            BuiltinClass::Integer,
            BuiltinClass::Float32,
            BuiltinClass::Float64,
            BuiltinClass::String,
        ]
        .into_iter()
        .enumerate()
        {
            let spine = if kind == BuiltinClass::Object {
                StaticSpine::new(1)
            } else {
                StaticSpine::new(1)
                    .with_meta_capabilities(MetaCapabilities::denying(&[Capability::InstanceState]))
            };
            classes[index] = (kind, registry.define_builtin_class(kind, spine, None)?);
        }
        let kernel = Self { classes };
        // Object gets no native comparison selectors. IRIS-V1-RUNTIME-C083 makes
        // its `<=>` answer nil for every operand, and C084/C086 derive the six
        // comparison Methods from that response, so the native numeric bodies
        // installed on the value Classes would be wrong here.
        // Hash is NOT installed here either. IRIS-V1-RUNTIME-C088 gives an
        // ordinary object a runtime-stable IDENTITY hash, which the evaluator
        // supplies from the heap; the native numeric body would be wrong.
        kernel.install(registry, BuiltinClass::Object, &[NativeSelector::ToBool])?;
        // IRIS-V1-COLLECTIONS-C043 compares the exact Unicode scalar sequence
        // and case with no normalization, case folding, or locale mapping.
        // Ordering selectors are NOT installed: the chapter defines String
        // equality and hashing here but leaves `<=>` to the collation rules,
        // so installing a native ordering body would be inventing one.
        kernel.install(
            registry,
            BuiltinClass::String,
            &[
                NativeSelector::Equal,
                NativeSelector::NotEqual,
                // C087 fixes a specification-stable String hash, so the
                // selector is installed even though ordering stays out.
                NativeSelector::Hash,
                NativeSelector::ToBool,
            ],
        )?;
        kernel.install(
            registry,
            BuiltinClass::Nil,
            &[
                NativeSelector::Equal,
                NativeSelector::NotEqual,
                NativeSelector::Less,
                NativeSelector::LessEqual,
                NativeSelector::Greater,
                NativeSelector::GreaterEqual,
                NativeSelector::Compare,
                NativeSelector::Hash,
                NativeSelector::ToBool,
            ],
        )?;
        kernel.install(
            registry,
            BuiltinClass::Bool,
            &[
                NativeSelector::Equal,
                NativeSelector::NotEqual,
                NativeSelector::Less,
                NativeSelector::LessEqual,
                NativeSelector::Greater,
                NativeSelector::GreaterEqual,
                NativeSelector::Compare,
                NativeSelector::Hash,
                NativeSelector::ToBool,
            ],
        )?;
        kernel.install(
            registry,
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
                NativeSelector::LessEqual,
                NativeSelector::Greater,
                NativeSelector::GreaterEqual,
                NativeSelector::Compare,
                NativeSelector::Negate,
                NativeSelector::MulAdd,
                NativeSelector::Hash,
                NativeSelector::ToBool,
            ],
        )?;
        kernel.install(
            registry,
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
                NativeSelector::LessEqual,
                NativeSelector::Greater,
                NativeSelector::GreaterEqual,
                NativeSelector::Compare,
                NativeSelector::Negate,
                NativeSelector::FromBits,
                NativeSelector::Nan,
                NativeSelector::Infinity,
                NativeSelector::MulAdd,
                NativeSelector::Hash,
                NativeSelector::ToBool,
                NativeSelector::IsNan,
                NativeSelector::IsSignalingNan,
                NativeSelector::IsInfinite,
                NativeSelector::IsFinite,
                NativeSelector::IsNormal,
                NativeSelector::IsSubnormal,
                NativeSelector::IsZero,
                NativeSelector::SignBit,
                NativeSelector::ToBits,
            ],
        )?;
        kernel.install(
            registry,
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
                NativeSelector::LessEqual,
                NativeSelector::Greater,
                NativeSelector::GreaterEqual,
                NativeSelector::Compare,
                NativeSelector::Negate,
                NativeSelector::FromBits,
                NativeSelector::Nan,
                NativeSelector::Infinity,
                NativeSelector::MulAdd,
                NativeSelector::Hash,
                NativeSelector::ToBool,
                NativeSelector::IsNan,
                NativeSelector::IsSignalingNan,
                NativeSelector::IsInfinite,
                NativeSelector::IsFinite,
                NativeSelector::IsNormal,
                NativeSelector::IsSubnormal,
                NativeSelector::IsZero,
                NativeSelector::SignBit,
                NativeSelector::ToBits,
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

    /// Enforces an operation against a built-in Class's active effective policy.
    pub fn require_meta_capability(
        &self,
        registry: &ClassRegistry,
        class: ClassId,
        operation: Capability,
    ) -> Result<(), ClassError> {
        registry.require_meta_capability(class, operation)
    }
    /// Resolves an ordinary selector for a built-in value through its active Class revision.
    pub fn dispatch_value(
        &self,
        registry: &ClassRegistry,
        receiver: &Value,
        selector: Selector,
    ) -> Result<DispatchOutcome, KernelError> {
        registry
            .dispatch(self.class_of(receiver)?, selector)
            .map_err(KernelError::from)
    }

    /// Resolves a selector sent to a built-in Class object through its active revision.
    pub fn dispatch_class_object(
        &self,
        registry: &ClassRegistry,
        class: ClassId,
        selector: Selector,
    ) -> Result<DispatchOutcome, KernelError> {
        registry
            .dispatch_class_object(class, selector)
            .map_err(KernelError::from)
    }

    /// Invokes a native Method selected by ordinary runtime dispatch.
    pub fn invoke_selected(
        &self,
        method: Method,
        receiver: Value,
        arguments: &[Value],
    ) -> Result<Value, KernelError> {
        NativeSelector::from_raw(method.body().raw())
            .ok_or(KernelError::MessageNotFound {
                receiver: self.class_of(&receiver)?,
                selector: method.selector(),
                arity: arguments.len(),
            })
            .and_then(|selector| self.invoke(selector, receiver, arguments))
    }
    pub fn send(
        &self,
        registry: &ClassRegistry,
        receiver: Value,
        selector: NativeSelector,
        arguments: &[Value],
    ) -> Result<Value, KernelError> {
        let class = self.class_of(&receiver)?;
        match registry.dispatch(class, selector.id())? {
            DispatchOutcome::Invoke(method) => self.invoke_selected(method, receiver, arguments),
            DispatchOutcome::WouldInvokeMethodMissing { selector } => {
                Err(KernelError::MessageNotFound {
                    receiver: class,
                    selector,
                    arity: arguments.len(),
                })
            }
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
        &self,
        registry: &mut ClassRegistry,
        kind: BuiltinClass,
        selectors: &[NativeSelector],
    ) -> Result<(), KernelError> {
        let class = self.class(kind)?;
        for selector in selectors {
            registry.publish_method(
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
            Value::Text(_) => self.class(BuiltinClass::String),
            Value::Class(class) => Ok(*class),
            // IRIS-V1-RUNTIME-C005 makes `Object` the single root, and C094
            // gives it a `to_bool` returning `true`. A value with no dedicated
            // builtin Class is therefore an ordinary `Object` rather than an
            // error: rejecting these made `if :sym` and `if [1]` fail outright.
            Value::Array(_)
            | Value::Bytes(_)
            | Value::ByteArray(_)
            | Value::Tuple(_)
            | Value::Hash(_)
            | Value::Symbol(_)
            | Value::Type(..)
            | Value::ComposedType(_)
            | Value::Contract(_)
            | Value::Closure(_)
            | Value::KeywordArgument(_, _)
            | Value::IterationYield(_)
            | Value::ReadonlyArray(_)
            | Value::SourceLocation(..)
            | Value::StackFrame(..)
            | Value::RaiseSite(_)
            | Value::ArrayIterator(_)
            | Value::HashIterator(_)
            | Value::ByteIterator(_)
            | Value::Generator(_)
            | Value::Task(_)
            | Value::Range(..)
            | Value::IterationDone
            | Value::Transformation { .. }
            | Value::ExceptionContext(..)
            | Value::ContractView(_, _)
            | Value::Object(_)
            | Value::BoundMethod(_)
            | Value::Method(_) => self.class(BuiltinClass::Object),
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
            NativeSelector::Equal => {
                Ok(Value::Bool(match singleton_equal(&receiver, arguments)? {
                    Some(equal) => equal,
                    None => Numeric::equal(&numeric(&receiver)?, &numeric_arg(arguments)?),
                }))
            }
            NativeSelector::NotEqual => {
                Ok(Value::Bool(match singleton_equal(&receiver, arguments)? {
                    Some(equal) => !equal,
                    None => Numeric::not_equal(&numeric(&receiver)?, &numeric_arg(arguments)?),
                }))
            }
            NativeSelector::Less => Ok(Value::Bool(
                self.ordering(&receiver, arguments)? == Some(Ordering::Less),
            )),
            NativeSelector::LessEqual => Ok(Value::Bool(matches!(
                self.ordering(&receiver, arguments)?,
                Some(Ordering::Less | Ordering::Equal)
            ))),
            NativeSelector::Greater => Ok(Value::Bool(
                self.ordering(&receiver, arguments)? == Some(Ordering::Greater),
            )),
            NativeSelector::GreaterEqual => Ok(Value::Bool(matches!(
                self.ordering(&receiver, arguments)?,
                Some(Ordering::Equal | Ordering::Greater)
            ))),
            NativeSelector::Compare => Ok(match self.ordering(&receiver, arguments)? {
                Some(Ordering::Less) => Value::Integer((-1_i8).into()),
                Some(Ordering::Equal) => Value::Integer(0_u8.into()),
                Some(Ordering::Greater) => Value::Integer(1_u8.into()),
                None => Value::Nil,
            }),
            NativeSelector::Negate => Ok(value(Numeric::negate(&numeric(&receiver)?)?)),
            NativeSelector::FromBits => self.bits_from_integer(receiver, arguments),
            NativeSelector::Nan => self.special(receiver, arguments, true),
            NativeSelector::Infinity => self.special(receiver, arguments, false),
            NativeSelector::MulAdd => self.mul_add(receiver, arguments),
            NativeSelector::Hash => self.hash(receiver, arguments),
            NativeSelector::ToBool => self.to_bool(receiver, arguments),
            NativeSelector::IsNan
            | NativeSelector::IsSignalingNan
            | NativeSelector::IsInfinite
            | NativeSelector::IsFinite
            | NativeSelector::IsNormal
            | NativeSelector::IsSubnormal
            | NativeSelector::IsZero
            | NativeSelector::SignBit => classify(selector, &receiver, arguments),
            NativeSelector::ToBits => {
                if !arguments.is_empty() {
                    return Err(KernelError::Arity);
                }
                match &receiver {
                    Value::Float32(_) => Ok(Value::Integer(Numeric::float32_to_bits(&numeric(
                        &receiver,
                    )?)?)),
                    Value::Float64(_) => Ok(Value::Integer(Numeric::float64_to_bits(&numeric(
                        &receiver,
                    )?)?)),
                    _ => Err(KernelError::Type),
                }
            }
        }
    }
    /// Orders a receiver against its single argument.
    ///
    /// `IRIS-V1-RUNTIME-C091` gives Bool a total order that never ranks against a
    /// numeric, and `IRIS-V1-RUNTIME-C092` makes `nil` order-equivalent only to
    /// itself, so neither becomes a global minimum or maximum. Both answer `None`
    /// for an operand outside their own category, which the ordered comparisons
    /// render as `false` and `<=>` renders as `nil`.
    fn ordering(
        &self,
        receiver: &Value,
        arguments: &[Value],
    ) -> Result<Option<Ordering>, KernelError> {
        let [argument] = arguments else {
            return Err(KernelError::Arity);
        };
        match (receiver, argument) {
            (Value::Bool(left), Value::Bool(right)) => Ok(Some(left.cmp(right))),
            (Value::Nil, Value::Nil) => Ok(Some(Ordering::Equal)),
            (Value::Bool(_) | Value::Nil, _) | (_, Value::Bool(_) | Value::Nil) => Ok(None),
            _ => Ok(Numeric::compare(&numeric(receiver)?, &numeric(argument)?)),
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
    fn hash(&self, receiver: Value, arguments: &[Value]) -> Result<Value, KernelError> {
        if !arguments.is_empty() {
            return Err(KernelError::Arity);
        }
        Ok(Value::Integer(crate::public_hash(&receiver)?))
    }
    fn to_bool(&self, receiver: Value, arguments: &[Value]) -> Result<Value, KernelError> {
        if !arguments.is_empty() {
            return Err(KernelError::Arity);
        }
        // IRIS-V1-RUNTIME-C094: root `Object` provides `to_bool` returning
        // `true`, so EVERY value answers it. Only `Nil` is false and `Bool`
        // returns itself; rejecting the rest made `if :sym`, `if [1]` and
        // `x ||= v` on a truthy target fail with a type error.
        Ok(match receiver {
            Value::Nil => Value::Bool(false),
            Value::Bool(value) => Value::Bool(value),
            _ => Value::Bool(true),
        })
    }
}
/// Decides equality when either operand is a Bool or `nil`.
///
/// Returns `None` when both operands are numeric, leaving them to the exact
/// numeric path. `IRIS-V1-RUNTIME-C091` keeps Bool distinct from `Integer(0)`
/// and `Integer(1)`, so a Bool compared with a numeric is unequal rather than a
/// type error.
/// Classifies a float by its interchange bits alone.
///
/// `IRIS-V1-RUNTIME-C115` requires these Methods to be pure bit classification:
/// they must not perform floating arithmetic, quiet a signaling NaN, alter
/// payload or sign, or change `to_bits()`. Both widths are therefore decomposed
/// into sign, biased exponent and trailing significand rather than routed
/// through any arithmetic path, and a signaling NaN is recognised by IEEE-754 as
/// a NaN whose leading significand bit is clear.
fn classify(
    selector: NativeSelector,
    receiver: &Value,
    arguments: &[Value],
) -> Result<Value, KernelError> {
    if !arguments.is_empty() {
        return Err(KernelError::Arity);
    }
    let (sign, exponent, significand, exponent_max, quiet_bit) = match receiver {
        Value::Float32(value) => {
            let bits = value.to_bits();
            (
                bits >> 31 == 1,
                u64::from((bits >> 23) & 0xff),
                u64::from(bits & 0x007f_ffff),
                0xff_u64,
                1_u64 << 22,
            )
        }
        Value::Float64(value) => {
            let bits = value.to_bits();
            (
                bits >> 63 == 1,
                (bits >> 52) & 0x7ff,
                bits & 0x000f_ffff_ffff_ffff,
                0x7ff_u64,
                1_u64 << 51,
            )
        }
        _ => return Err(KernelError::Type),
    };
    let is_nan = exponent == exponent_max && significand != 0;
    Ok(Value::Bool(match selector {
        NativeSelector::IsNan => is_nan,
        NativeSelector::IsSignalingNan => is_nan && significand & quiet_bit == 0,
        NativeSelector::IsInfinite => exponent == exponent_max && significand == 0,
        NativeSelector::IsFinite => exponent != exponent_max,
        NativeSelector::IsNormal => exponent != 0 && exponent != exponent_max,
        NativeSelector::IsSubnormal => exponent == 0 && significand != 0,
        NativeSelector::IsZero => exponent == 0 && significand == 0,
        NativeSelector::SignBit => sign,
        _ => return Err(KernelError::Type),
    }))
}

fn singleton_equal(receiver: &Value, arguments: &[Value]) -> Result<Option<bool>, KernelError> {
    let [argument] = arguments else {
        return Err(KernelError::Arity);
    };
    Ok(match (receiver, argument) {
        (Value::Bool(left), Value::Bool(right)) => Some(left == right),
        (Value::Nil, Value::Nil) => Some(true),
        (Value::Bool(_) | Value::Nil, _) | (_, Value::Bool(_) | Value::Nil) => Some(false),
        // C043: exact scalar sequence and case, which is what a host `String`
        // comparison already performs. A String is never equal to a non-String,
        // so it never falls through to the numeric path.
        (Value::Text(left), Value::Text(right)) => Some(left == right),
        (Value::Text(_), _) | (_, Value::Text(_)) => Some(false),
        _ => None,
    })
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
