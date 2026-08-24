//! Literal and call lowering.

use iris_runtime::NativeSelector;
use iris_syntax::{BinaryOperator, Expression};

use super::expressions;
use super::lowering::Lowering;
use super::{CompileError, FloatWidth, Instruction, Register};

impl<'a, 'b> Lowering<'a, 'b> {
    /// Lowers a literal by RE-LEXING its text.
    ///
    /// The parser keeps a literal as source text, and the lexer is what turns
    /// it into a value under the chapter 02 rules - suffixes, radix prefixes,
    /// separators and precision warnings. Parsing the text here would be a
    /// second literal implementation, and a differential row would then compare
    /// this backend's literal rules against the lexer's.
    pub(super) fn literal(&mut self, text: &str) -> Result<Register, CompileError> {
        let destination = self.allocate()?;
        // The parser keeps these keyword values as literal TEXT and the lexer
        // does not convert them, so they are recognised here.
        match text {
            "nil" => {
                self.instructions.push(Instruction::LoadNil { destination });
                return Ok(destination);
            }
            "true" | "false" => {
                self.instructions.push(Instruction::LoadBool {
                    destination,
                    value: text == "true",
                });
                return Ok(destination);
            }
            _ => {}
        }
        let conversion = iris_lexer::convert_literals(text);
        if !conversion.diagnostics().is_empty() {
            return Err(CompileError::new("rejected literal"));
        }
        let [literal] = conversion.values() else {
            return Err(CompileError::new("literal"));
        };
        self.instructions.push(match literal {
            iris_lexer::Literal::Integer(digits) => Instruction::LoadInteger {
                destination,
                digits: digits.clone(),
            },
            iris_lexer::Literal::Float64(number) => Instruction::LoadFloat64 {
                destination,
                bits: number.to_bits(),
            },
            iris_lexer::Literal::Float32(number) => Instruction::LoadFloat32 {
                destination,
                bits: number.to_bits(),
            },
            iris_lexer::Literal::String(text) => {
                if text.contains("${") {
                    return self.interpolated_text(text);
                }
                Instruction::LoadText {
                    destination,
                    text: text.clone(),
                }
            }
        });
        Ok(destination)
    }

    /// `Float32.from_bits(bits)`, `value.to_bits()` and `value.hash()` are the
    /// call shapes this subset covers. A general call needs frames and user
    /// Methods it does not have.
    pub(super) fn call(
        &mut self,
        callee: &Expression,
        arguments: &[Expression],
    ) -> Result<Register, CompileError> {
        if let Expression::ContractView { receiver, selector } = callee {
            let receiver = self.expression(receiver)?;
            let (first, count) = self.argument_window(arguments)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::SendContract {
                destination,
                receiver,
                selector: selector.clone(),
                first,
                count,
            });
            return Ok(destination);
        }
        let Expression::Member { receiver, selector } = callee else {
            return Err(CompileError::new(match callee {
                Expression::Name(_) => "call bare name",
                Expression::Closure { .. } => "call closure",
                _ => "call callee",
            }));
        };
        if let Expression::Name(name) = receiver.as_ref()
            && self
                .lookup(name)
                .is_some_and(|register| self.method_values.contains(&register))
            && selector == "call"
        {
            return Err(CompileError::new("Method.call"));
        }
        if matches!(receiver.as_ref(), Expression::Name(name) if name == "Iteration")
            && selector == "yield"
        {
            let [value] = arguments else {
                return Err(CompileError::new("yield arity"));
            };
            let value = self.expression(value)?;
            let destination = self.allocate()?;
            self.instructions
                .push(Instruction::BuildIterationYield { destination, value });
            return Ok(destination);
        }
        if matches!(receiver.as_ref(), Expression::Name(name) if name == "JSON")
            && matches!(selector.as_str(), "decode" | "encode")
        {
            let (first, count) = self.argument_window(arguments)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::Json {
                destination,
                selector: selector.clone(),
                first,
                count,
            });
            return Ok(destination);
        }
        if let Expression::Name(namespace) = receiver.as_ref()
            && matches!(namespace.as_str(), "Revision" | "RevisionHistory")
            && matches!(
                (namespace.as_str(), selector.as_str()),
                ("Revision", "subscribe" | "flush" | "event_errors")
                    | ("RevisionHistory", "events" | "prune")
            )
        {
            let (first, count) = self.argument_window(arguments)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::Revision {
                destination,
                namespace: namespace.clone(),
                selector: selector.clone(),
                first,
                count,
            });
            return Ok(destination);
        }
        if let Expression::Name(class) = receiver.as_ref()
            && selector == "open"
            && let Some(class) = self.class_index(class)
        {
            let [callback] = arguments else {
                return Err(CompileError::new("Class.open arity"));
            };
            let callback = self.expression(callback)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::OpenClass {
                destination,
                class,
                callback,
            });
            return Ok(destination);
        }
        if let Expression::Name(namespace) = receiver.as_ref()
            && matches!(
                namespace.as_str(),
                "Reflection::Class" | "Reflection::Object"
            )
            && matches!(
                (namespace.as_str(), selector.as_str()),
                ("Reflection::Class", "method") | ("Reflection::Object", "get_ivar" | "set_ivar")
            )
        {
            let (first, count) = self.argument_window(arguments)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::Reflection {
                destination,
                namespace: namespace.clone(),
                selector: selector.clone(),
                first,
                count,
            });
            if namespace == "Reflection::Class" && selector == "method" {
                self.method_values.push(destination);
            }
            return Ok(destination);
        }
        if matches!(receiver.as_ref(), Expression::Member { receiver, selector }
            if selector == "type" && matches!(receiver.as_ref(), Expression::ReifiedType(_)))
            && selector == "kind"
            && arguments.is_empty()
        {
            let destination = self.allocate()?;
            self.instructions.push(Instruction::LoadSymbol {
                destination,
                name: "nominal".to_owned(),
            });
            return Ok(destination);
        }
        if let Expression::Name(name) = receiver.as_ref()
            && selector == "from_bits"
        {
            let width = match name.as_str() {
                "Float32" => FloatWidth::Bits32,
                "Float64" => FloatWidth::Bits64,
                _ => return Err(CompileError::new("call")),
            };
            let [bits] = arguments else {
                return Err(CompileError::new("from_bits arity"));
            };
            let bits = self.expression(bits)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::FromBits {
                destination,
                width,
                bits,
            });
            return Ok(destination);
        }
        if let Expression::Name(class) = receiver.as_ref()
            && selector == "new"
            && let Some(class) = self.class_index(class)
        {
            let (first, count) = self.argument_window(arguments)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::New {
                destination,
                class,
                first,
                count,
            });
            return Ok(destination);
        }
        if selector == "same?" {
            let [right] = arguments else {
                return Err(CompileError::new("same? arity"));
            };
            let left = self.expression(receiver)?;
            let right = self.expression(right)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::Identity {
                destination,
                left,
                right,
            });
            return Ok(destination);
        }
        if selector == "call" {
            let receiver = self.expression(receiver)?;
            let (first, count) = self.argument_window(arguments)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::Send {
                destination,
                receiver,
                selector: selector.clone(),
                first,
                count,
            });
            return Ok(destination);
        }
        if let Expression::Name(class) = receiver.as_ref()
            && let Some(class) = self.class_index(class)
            && self.signatures.iter().any(|signature| {
                signature.module == self.classes[class].name
                    && signature.selector == selector
                    && signature.class_method
            })
        {
            let (first, count) = self.argument_window(arguments)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::SendClass {
                destination,
                class,
                selector: selector.clone(),
                first,
                count,
            });
            return Ok(destination);
        }
        // A resolved module function is called by INDEX. Resolution happened
        // before lowering, so no name is looked up at run time.
        if let Expression::Name(module) = receiver.as_ref()
            && let Some(function) = self.resolve(module, selector)
        {
            let parameters = &self.signatures[function].parameters;
            let required = parameters
                .iter()
                .take_while(|parameter| parameter.default.is_none())
                .count();
            if arguments.len() < required || arguments.len() > parameters.len() {
                return Err(CompileError::new("call arity"));
            }
            let count =
                u16::try_from(parameters.len()).map_err(|_| CompileError::new("call too wide"))?;
            let mut lowered = Vec::with_capacity(parameters.len());
            for argument in arguments {
                lowered.push(self.expression(argument)?);
            }
            for parameter in &parameters[arguments.len()..] {
                let default = parameter
                    .default
                    .as_ref()
                    .ok_or_else(|| CompileError::new("call arity"))?;
                lowered.push(self.expression(default)?);
            }
            // Arguments are copied into a CONTIGUOUS window, so the call names
            // a range and the callee sees them as its leading registers.
            let first = self.next_register;
            for source in lowered {
                let destination = self.allocate()?;
                self.instructions.push(Instruction::Move {
                    destination,
                    source,
                });
            }
            // A zero-argument call still needs a window start inside the file.
            if count == 0 {
                let destination = self.allocate()?;
                self.instructions.push(Instruction::LoadNil { destination });
            }
            let destination = self.allocate()?;
            self.instructions.push(Instruction::Call {
                destination,
                function,
                first,
                count,
            });
            return Ok(destination);
        }
        if arguments.is_empty()
            && let Some(native) = match selector.as_str() {
                "to_bits" => Some("to_bits"),
                // C146 fixes the public hash of each numeric and singleton
                // value, which V073 compares across backends.
                "hash" => Some("hash"),
                _ => None,
            }
        {
            let operand = self.expression(receiver)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::Unary {
                destination,
                selector: native,
                operand,
            });
            return Ok(destination);
        }
        if NativeSelector::from_source(selector).is_some() {
            let receiver = self.expression(receiver)?;
            let (first, count) = self.argument_window(arguments)?;
            let destination = self.allocate()?;
            self.instructions.push(Instruction::Send {
                destination,
                receiver,
                selector: selector.clone(),
                first,
                count,
            });
            return Ok(destination);
        }
        if let Some(construct) = expressions::ordinary_receiver_decline(receiver, |name| {
            self.lookup(name).is_some()
                || self.class_index(name).is_some()
                || matches!(name, "nil" | "true" | "false")
        }) {
            return Err(CompileError::new(construct));
        }
        let receiver = self.expression(receiver)?;
        let (first, count) = self.argument_window(arguments)?;
        let destination = self.allocate()?;
        self.instructions.push(Instruction::Send {
            destination,
            receiver,
            selector: selector.clone(),
            first,
            count,
        });
        Ok(destination)
    }

    pub(super) fn class_index(&self, name: &str) -> Option<usize> {
        self.classes.iter().position(|class| class.name == name)
    }

    pub(super) fn contract_index(&self, name: &str) -> Option<usize> {
        self.contracts
            .iter()
            .position(|contract| contract.name == name)
    }

    pub(super) fn argument_window(
        &mut self,
        arguments: &[Expression],
    ) -> Result<(Register, u16), CompileError> {
        let count =
            u16::try_from(arguments.len()).map_err(|_| CompileError::new("call too wide"))?;
        let mut lowered = Vec::with_capacity(arguments.len());
        for argument in arguments {
            lowered.push(self.expression(argument)?);
        }
        let first = self.next_register;
        for source in lowered {
            let destination = self.allocate()?;
            self.instructions.push(Instruction::Move {
                destination,
                source,
            });
        }
        if count == 0 {
            let destination = self.allocate()?;
            self.instructions.push(Instruction::LoadNil { destination });
        }
        Ok((first, count))
    }
}

/// The native selector for a binary operator, when this backend covers it.
///
/// Only operators whose whole meaning is a native send appear here. A range,
/// regex or type operator carries semantics beyond the kernel send, so it is
/// declined rather than approximated.
pub(super) fn binary_selector(operator: &BinaryOperator) -> Result<&'static str, CompileError> {
    Ok(match operator {
        BinaryOperator::Add => "+",
        BinaryOperator::Subtract => "-",
        BinaryOperator::Multiply => "*",
        BinaryOperator::Divide => "/",
        BinaryOperator::Power => "**",
        BinaryOperator::ShiftLeft => "<<",
        BinaryOperator::ShiftRight => ">>",
        BinaryOperator::BitwiseAnd => "&",
        BinaryOperator::BitwiseXor => "^",
        BinaryOperator::BitwiseOr => "|",
        BinaryOperator::Equal => "==",
        BinaryOperator::NotEqual => "!=",
        BinaryOperator::Less => "<",
        BinaryOperator::LessEqual => "<=",
        BinaryOperator::Greater => ">",
        BinaryOperator::GreaterEqual => ">=",
        BinaryOperator::Compare => "<=>",
        other => return Err(CompileError::new(format!("operator {other:?}"))),
    })
}

pub(super) fn construct_name(expression: &Expression) -> String {
    let name = match expression {
        Expression::ReifiedType(_) => "expression reified type",
        Expression::ClosedGeneric { .. } => "expression closed generic",
        Expression::KeywordArgument { .. } => "expression keyword argument",
        Expression::ContractView { .. } => "expression contract view",
        Expression::ClassVar(_) => "expression class variable",
        Expression::Symbol(_) => "symbol",
        Expression::Hash(_) => "hash",
        Expression::Tuple(_) => "tuple",
        Expression::Member { .. } => "member",
        Expression::Index { .. } => "index",
        Expression::Closure { .. } => "closure",
        Expression::If { .. } => "if",
        Expression::While { .. } => "while",
        Expression::Try { .. } => "try",
        Expression::Await(_) => "await",
        Expression::Yield(_) => "yield",
        Expression::Assignment { .. } => "assignment",
        Expression::Name(_)
        | Expression::Literal(_)
        | Expression::Array(_)
        | Expression::Call { .. }
        | Expression::Unary { .. }
        | Expression::Binary { .. }
        | Expression::Grouped(_)
        | Expression::RawIvar(_)
        | Expression::GlobalVar(_) => "expression covered",
    };
    name.to_owned()
}
