//! Expression and closure lowering.

use iris_syntax::{BinaryOperator, Expression, Statement, UnaryOperator};

use super::calls::{binary_selector, construct_name};
use super::lowering::Lowering;
use super::{CompileError, Function, Instruction, Register};

impl<'a, 'b> Lowering<'a, 'b> {
    pub(super) fn expression(&mut self, expression: &Expression) -> Result<Register, CompileError> {
        match expression {
            Expression::Literal(text) => self.literal(text),
            Expression::Symbol(name) => {
                let destination = self.allocate()?;
                self.instructions.push(Instruction::LoadSymbol {
                    destination,
                    name: name.clone(),
                });
                Ok(destination)
            }
            // As a RECEIVER these keyword values arrive as a Name rather than
            // a Literal, so `nil.hash()` would otherwise be an unbound name.
            Expression::Name(name) if matches!(name.as_str(), "nil" | "true" | "false") => {
                self.literal(name)
            }
            Expression::Name(name)
                if matches!(
                    name.as_str(),
                    "Object" | "Nil" | "Bool" | "Integer" | "Float32" | "Float64" | "String"
                ) =>
            {
                let destination = self.allocate()?;
                self.instructions.push(Instruction::LoadBuiltinClass {
                    destination,
                    name: name.clone(),
                });
                Ok(destination)
            }
            Expression::Name(name) if self.deferred.iter().any(|held| held == name) => {
                Err(CompileError::new("deferred read before assignment"))
            }
            Expression::Name(name) => {
                if let Some(binding) = self.lookup_binding(name).cloned() {
                    if !binding.shared {
                        return Ok(binding.register);
                    }
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::LoadCell {
                        destination,
                        cell: binding.register,
                    });
                    return Ok(destination);
                }
                if let Some(binding) = self
                    .program_bindings
                    .iter()
                    .rev()
                    .find(|binding| binding.name == *name)
                {
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::LoadBinding {
                        destination,
                        name: name.clone(),
                        shared: binding.shared,
                    });
                    return Ok(destination);
                }
                let destination = self.allocate()?;
                if let Some(class) = self.class_index(name) {
                    self.instructions
                        .push(Instruction::LoadClass { destination, class });
                    return Ok(destination);
                }
                if let Some(contract) = self.contract_index(name) {
                    self.instructions.push(Instruction::LoadContract {
                        destination,
                        contract,
                    });
                    return Ok(destination);
                }
                if self
                    .signatures
                    .iter()
                    .any(|signature| signature.module == name && !signature.receiver)
                {
                    self.instructions.push(Instruction::LoadSymbol {
                        destination,
                        name: name.clone(),
                    });
                    return Ok(destination);
                }
                Err(CompileError::new("name unbound"))
            }
            Expression::GlobalVar(name) => {
                let destination = self.allocate()?;
                self.instructions.push(Instruction::LoadGlobal {
                    destination,
                    name: name.clone(),
                });
                Ok(destination)
            }
            Expression::RawIvar(name) => {
                let receiver = self
                    .lookup("self")
                    .ok_or_else(|| CompileError::new("ivar"))?;
                let destination = self.allocate()?;
                self.instructions.push(Instruction::GetIvar {
                    destination,
                    receiver,
                    name: name.clone(),
                });
                Ok(destination)
            }
            Expression::ClassVar(name) => {
                let receiver = self
                    .lookup("self")
                    .ok_or_else(|| CompileError::new("expression class variable"))?;
                let destination = self.allocate()?;
                self.instructions.push(Instruction::GetClassVar {
                    destination,
                    receiver,
                    name: name.clone(),
                });
                Ok(destination)
            }
            Expression::Grouped(inner) => self.expression(inner),
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                if matches!(
                    operator,
                    BinaryOperator::RangeInclusive | BinaryOperator::RangeExclusive
                ) {
                    let start = self.expression(left)?;
                    let end = self.expression(right)?;
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::BuildRange {
                        destination,
                        start,
                        end,
                        inclusive_end: *operator == BinaryOperator::RangeInclusive,
                    });
                    return Ok(destination);
                }
                if matches!(
                    operator,
                    BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr
                ) {
                    let destination = self.allocate()?;
                    let left = self.expression(left)?;
                    self.instructions.push(Instruction::Move {
                        destination,
                        source: left,
                    });
                    let branch = self.instructions.len();
                    self.instructions.push(Instruction::JumpUnless {
                        condition: left,
                        target: 0,
                    });
                    if *operator == BinaryOperator::LogicalOr {
                        let skip = self.instructions.len();
                        self.instructions.push(Instruction::Jump { target: 0 });
                        let otherwise = self.instructions.len();
                        let right = self.expression(right)?;
                        self.instructions.push(Instruction::Move {
                            destination,
                            source: right,
                        });
                        let after = self.instructions.len();
                        self.patch(branch, otherwise)?;
                        self.patch(skip, after)?;
                    } else {
                        let right = self.expression(right)?;
                        self.instructions.push(Instruction::Move {
                            destination,
                            source: right,
                        });
                        let after = self.instructions.len();
                        self.patch(branch, after)?;
                    }
                    return Ok(destination);
                }
                if *operator == BinaryOperator::As
                    && let Expression::Name(name) = right.as_ref()
                    && let Some(contract) = self.contract_index(name)
                {
                    let receiver = self.expression(left)?;
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::ContractCast {
                        destination,
                        receiver,
                        contract,
                    });
                    return Ok(destination);
                }
                if *operator == BinaryOperator::Identity {
                    let left = self.expression(left)?;
                    let right = self.expression(right)?;
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::Identity {
                        destination,
                        left,
                        right,
                    });
                    return Ok(destination);
                }
                if *operator == BinaryOperator::Is {
                    if matches!(right.as_ref(), Expression::Name(name) if self.contract_index(name).is_some())
                    {
                        return Err(CompileError::new("operator Is contract"));
                    }
                    let value = self.expression(left)?;
                    let target = self.expression(right)?;
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::TypeTest {
                        destination,
                        value,
                        target,
                    });
                    return Ok(destination);
                }
                let selector = binary_selector(operator)?;
                let left = self.expression(left)?;
                let right = self.expression(right)?;
                let destination = self.allocate()?;
                self.instructions.push(Instruction::Binary {
                    destination,
                    selector,
                    left,
                    right,
                });
                Ok(destination)
            }
            Expression::Unary { operator, operand } => {
                let operand = self.expression(operand)?;
                if *operator == UnaryOperator::Plus {
                    return Ok(operand);
                }
                let destination = self.allocate()?;
                self.instructions.push(Instruction::Unary {
                    destination,
                    selector: match operator {
                        UnaryOperator::Negate => "negate",
                        UnaryOperator::BitwiseNot => "~",
                        UnaryOperator::Not => {
                            return Err(CompileError::new("unary Not"));
                        }
                        UnaryOperator::Plus => return Ok(operand),
                    },
                    operand,
                });
                Ok(destination)
            }
            // An Array literal is a pure aggregate, so it needs no frames. The
            // elements are lowered into a CONTIGUOUS run of registers, which
            // lets the instruction name the range instead of carrying a list.
            Expression::Array(elements) => {
                let count = u16::try_from(elements.len())
                    .map_err(|_| CompileError::new("array too long"))?;
                let mut lowered = Vec::with_capacity(elements.len());
                for element in elements {
                    lowered.push(self.expression(element)?);
                }
                let first = self.next_register;
                for source in lowered {
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::Move {
                        destination,
                        source,
                    });
                }
                let destination = self.allocate()?;
                self.instructions.push(Instruction::BuildArray {
                    destination,
                    first,
                    count,
                });
                Ok(destination)
            }
            Expression::Tuple(elements) => {
                let count = u16::try_from(elements.len())
                    .map_err(|_| CompileError::new("tuple too long"))?;
                let mut lowered = Vec::with_capacity(elements.len());
                for element in elements {
                    lowered.push(self.expression(element)?);
                }
                let first = self.next_register;
                for source in lowered {
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::Move {
                        destination,
                        source,
                    });
                }
                let destination = self.allocate()?;
                self.instructions.push(Instruction::BuildTuple {
                    destination,
                    first,
                    count,
                });
                Ok(destination)
            }
            Expression::Hash(entries) => {
                let count =
                    u16::try_from(entries.len()).map_err(|_| CompileError::new("hash too long"))?;
                let mut lowered = Vec::with_capacity(entries.len() * 2);
                for (key, value) in entries {
                    match key {
                        Expression::Name(_) => return Err(CompileError::new("hash key name")),
                        key => lowered.push(self.expression(key)?),
                    }
                    lowered.push(self.expression(value)?);
                }
                let first = self.next_register;
                for source in lowered {
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::Move {
                        destination,
                        source,
                    });
                }
                let destination = self.allocate()?;
                self.instructions.push(Instruction::BuildHash {
                    destination,
                    first,
                    count,
                });
                Ok(destination)
            }
            Expression::Index { receiver, index } => {
                if !matches!(
                    receiver.as_ref(),
                    Expression::Array(_)
                        | Expression::Hash(_)
                        | Expression::Tuple(_)
                        | Expression::Call { .. }
                        | Expression::Member { .. }
                        | Expression::Name(_)
                        | Expression::GlobalVar(_)
                ) {
                    return Err(CompileError::new("index receiver"));
                }
                let receiver = self.expression(receiver)?;
                let index = self.expression(index)?;
                let destination = self.allocate()?;
                self.instructions.push(Instruction::Index {
                    destination,
                    receiver,
                    index,
                });
                Ok(destination)
            }
            Expression::Member { receiver, selector } => {
                if matches!(receiver.as_ref(), Expression::Name(name) if name == "Iteration")
                    && selector == "done"
                {
                    let destination = self.allocate()?;
                    self.instructions
                        .push(Instruction::LoadIterationDone { destination });
                    return Ok(destination);
                }
                if selector == "type" && matches!(receiver.as_ref(), Expression::ReifiedType(_)) {
                    return self.expression(receiver);
                }
                let receiver = self.expression(receiver)?;
                if self.method_values.contains(&receiver)
                    && matches!(selector.as_str(), "signature" | "package" | "call")
                {
                    return Err(CompileError::new(format!("Method.{selector}")));
                }
                let destination = self.allocate()?;
                self.instructions.push(Instruction::BindMember {
                    destination,
                    receiver,
                    selector: selector.clone(),
                });
                Ok(destination)
            }
            Expression::ContractView { .. } => {
                Err(CompileError::new("expression contract view outside call"))
            }
            Expression::ReifiedType(iris_syntax::TypeExpression::Name(name)) => {
                if let Some(class) = self.class_index(name) {
                    let destination = self.allocate()?;
                    self.instructions
                        .push(Instruction::LoadType { destination, class });
                    return Ok(destination);
                }
                if !matches!(
                    name.as_str(),
                    "Object" | "Nil" | "Bool" | "Integer" | "Float32" | "Float64" | "String"
                ) {
                    return Err(CompileError::new("expression reified type"));
                }
                let destination = self.allocate()?;
                self.instructions.push(Instruction::LoadBuiltinType {
                    destination,
                    name: name.clone(),
                });
                Ok(destination)
            }
            Expression::ReifiedType(iris_syntax::TypeExpression::Generic { name, .. }) => {
                let class = self
                    .class_index(name)
                    .ok_or_else(|| CompileError::new("expression reified type"))?;
                let destination = self.allocate()?;
                self.instructions
                    .push(Instruction::LoadType { destination, class });
                Ok(destination)
            }
            Expression::ClosedGeneric { name, .. } => {
                let destination = self.allocate()?;
                if let Some(class) = self.class_index(name) {
                    self.instructions
                        .push(Instruction::LoadClass { destination, class });
                } else if let Some(contract) = self.contract_index(name) {
                    self.instructions.push(Instruction::LoadContract {
                        destination,
                        contract,
                    });
                } else {
                    self.instructions.push(Instruction::LoadGlobal {
                        destination,
                        name: name.clone(),
                    });
                }
                Ok(destination)
            }
            // Assignment writes the name's EXISTING register, which is what
            // carries a value across a loop's back edge.
            Expression::Assignment {
                left,
                operator: iris_syntax::AssignmentOperator::Assign,
                right,
            } => {
                if let Expression::RawIvar(name) = left.as_ref() {
                    let receiver = self
                        .lookup("self")
                        .ok_or_else(|| CompileError::new("ivar"))?;
                    let value = self.expression(right)?;
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::SetIvar {
                        destination,
                        receiver,
                        name: name.clone(),
                        value,
                    });
                    return Ok(destination);
                }
                if let Expression::ClassVar(name) = left.as_ref() {
                    let receiver = self
                        .lookup("self")
                        .ok_or_else(|| CompileError::new("expression class variable"))?;
                    let value = self.expression(right)?;
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::SetClassVar {
                        destination,
                        receiver,
                        name: name.clone(),
                        value,
                    });
                    return Ok(destination);
                }
                if let Expression::Index { receiver, index } = left.as_ref() {
                    let receiver = self.expression(receiver)?;
                    let index = self.expression(index)?;
                    let value = self.expression(right)?;
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::SetIndex {
                        destination,
                        receiver,
                        index,
                        value,
                    });
                    return Ok(destination);
                }
                if let Expression::GlobalVar(name) = left.as_ref() {
                    let value = self.expression(right)?;
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::StoreGlobal {
                        destination,
                        name: name.clone(),
                        value,
                    });
                    return Ok(destination);
                }
                // The reference lowers `o.p = v` to a `p=` SEND rather than to
                // a stored slot, so a class without that setter answers
                // MessageNotFound for `p=`. Writing a field directly would
                // succeed where the reference refuses, and would bypass a
                // property setter's body where one exists.
                if let Expression::Member { receiver, selector } = left.as_ref() {
                    let receiver = self.expression(receiver)?;
                    let (first, count) = self.argument_window(std::slice::from_ref(right))?;
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::Send {
                        destination,
                        receiver,
                        selector: format!("{selector}="),
                        first,
                        count,
                    });
                    return Ok(destination);
                }
                let Expression::Name(name) = left.as_ref() else {
                    return Err(CompileError::new("assignment target"));
                };
                let Some(binding) = self.lookup_binding(name).cloned() else {
                    let Some(binding) = self
                        .program_bindings
                        .iter()
                        .rev()
                        .find(|binding| binding.name == *name)
                    else {
                        return Err(CompileError::new("name assignment unbound"));
                    };
                    if !binding.shared {
                        return Err(CompileError::new("name assignment immutable"));
                    }
                    let source = self.expression(right)?;
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::StoreBinding {
                        destination,
                        name: name.clone(),
                        source,
                    });
                    return Ok(destination);
                };
                let source = self.expression(right)?;
                let destination = binding.register;
                if binding.shared {
                    self.instructions.push(Instruction::StoreCell {
                        destination: source,
                        cell: destination,
                        source,
                    });
                } else {
                    self.instructions.push(Instruction::Move {
                        destination,
                        source,
                    });
                }
                self.deferred.retain(|held| held != name);
                Ok(if binding.shared { source } else { destination })
            }
            Expression::Call {
                callee, arguments, ..
            } => self.call(callee, arguments),
            Expression::Try {
                body,
                catches,
                finally,
            } => self.try_body(body, catches, finally),
            Expression::Closure {
                parameters, body, ..
            } => self.closure(parameters, body),
            other => Err(CompileError::new(construct_name(other))),
        }
    }

    pub(super) fn closure(
        &mut self,
        parameters: &[String],
        body: &[Statement],
    ) -> Result<Register, CompileError> {
        let captures = self.names.clone();
        let mut closure_functions = Vec::new();
        let mut lowering = Lowering::new(
            self.signatures,
            self.classes,
            self.contracts,
            self.declared_functions + self.closures.len(),
            &mut closure_functions,
            self.program_bindings,
            false,
        );
        for capture in &captures {
            let register = lowering.allocate()?;
            lowering.names.push(if capture.shared {
                super::lowering::Binding::shared(capture.name.clone(), register)
            } else {
                super::lowering::Binding::value(capture.name.clone(), register)
            });
        }
        for parameter in parameters {
            let register = lowering.allocate()?;
            lowering
                .names
                .push(super::lowering::Binding::value(parameter.clone(), register));
        }
        let value = lowering.body(body)?;
        lowering.instructions.push(Instruction::Return { value });
        let registers = lowering.next_register as usize;
        let instructions = std::mem::take(&mut lowering.instructions);
        drop(lowering);
        let function = self.declared_functions + self.closures.len() + closure_functions.len();
        self.closures.extend(closure_functions);
        self.closures.push(Function {
            name: "<closure>".to_owned(),
            parameters: captures.len() + parameters.len(),
            captures: captures.len(),
            parameter_types: vec!["Dynamic<Object>".to_owned(); parameters.len()],
            return_type: "Dynamic<Object>".to_owned(),
            registers,
            instructions,
        });
        let count = u16::try_from(captures.len())
            .map_err(|_| CompileError::new("closure capture too wide"))?;
        let first = self.next_register;
        for capture in captures {
            let destination = self.allocate()?;
            self.instructions.push(Instruction::Move {
                destination,
                source: capture.register,
            });
        }
        if count == 0 {
            let destination = self.allocate()?;
            self.instructions.push(Instruction::LoadNil { destination });
        }
        let destination = self.allocate()?;
        self.instructions.push(Instruction::MakeClosure {
            destination,
            function,
            first,
            count,
        });
        Ok(destination)
    }

    pub(super) fn dynamic_method(
        &mut self,
        parameters: &[String],
        body: &[Statement],
    ) -> Result<usize, CompileError> {
        let mut nested = Vec::new();
        let mut lowering = Lowering::new(
            self.signatures,
            self.classes,
            self.contracts,
            self.declared_functions + self.closures.len(),
            &mut nested,
            self.program_bindings,
            false,
        );
        let receiver = lowering.allocate()?;
        lowering
            .names
            .push(super::lowering::Binding::value("self".to_owned(), receiver));
        for parameter in parameters {
            let register = lowering.allocate()?;
            lowering
                .names
                .push(super::lowering::Binding::value(parameter.clone(), register));
        }
        let value = lowering.body(body)?;
        lowering.instructions.push(Instruction::Return { value });
        let registers = lowering.next_register as usize;
        let instructions = std::mem::take(&mut lowering.instructions);
        drop(lowering);
        let function = self.declared_functions + self.closures.len() + nested.len();
        self.closures.extend(nested);
        self.closures.push(Function {
            name: "<dynamic-method>".to_owned(),
            parameters: parameters.len() + 1,
            captures: 0,
            parameter_types: vec!["Dynamic<Object>".to_owned(); parameters.len()],
            return_type: "Dynamic<Object>".to_owned(),
            registers,
            instructions,
        });
        Ok(function)
    }
}
