//! Register-frame instruction execution.

use iris_runtime::{
    BoundReceiver, ClassError, ClassId, ConstructionError, ContractId, KernelError, NumericError,
    Selector, Value,
};

use crate::compile::{FloatWidth, Instruction, Program, Register};

use super::{ClosureRecord, Machine, MachineError, VerifyError, selector_id, truthy};

impl Machine {
    /// Runs ONE frame to completion, answering its register file.
    ///
    /// Each call gets a fresh file, so nothing is shared with the caller: a
    /// callee cannot read a caller's registers, and recursion needs no
    /// save/restore of individual registers. The frame is also the unit a
    /// future collector would walk, since the live frames ARE the root set.
    pub(super) fn run_body(
        &mut self,
        instructions: &[Instruction],
        size: usize,
        arguments: Vec<Value>,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Vec<Value>, MachineError> {
        // Verification proved every read is in range and written, so indexing
        // below cannot be out of bounds and no operand check is repeated.
        let mut registers = vec![Value::Nil; size];
        // Parameters arrive pre-bound in the leading registers.
        for (slot, argument) in arguments.into_iter().enumerate() {
            registers[slot] = argument;
        }

        let mut counter = 0;
        let mut handlers: Vec<(usize, Register)> = Vec::new();
        macro_rules! run_frame {
            ($label:lifetime, $call:expr) => {
                match $call {
                    Ok(values) => values,
                    Err(MachineError::Raised(value)) => {
                        let Some((handler, exception)) = handlers.pop() else {
                            return Err(MachineError::Raised(value));
                        };
                        registers[exception as usize] = value;
                        counter = handler;
                        continue $label;
                    }
                    Err(error) => return Err(error),
                }
            };
        }
        'frame: while let Some(instruction) = instructions.get(counter) {
            counter += 1;
            let produced = match instruction {
                Instruction::LoadInteger { digits, .. } => {
                    let Ok(number) = digits.parse() else {
                        // The lexer produced this text, so a rejection would
                        // mean the two disagree about integer syntax.
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    Value::Integer(number)
                }
                Instruction::LoadFloat64 { bits, .. } => Value::Float64(f64::from_bits(*bits)),
                Instruction::LoadFloat32 { bits, .. } => Value::Float32(f32::from_bits(*bits)),
                Instruction::LoadText { text, .. } => Value::Text(text.clone()),
                Instruction::LoadSymbol { name, .. } => Value::Symbol(name.clone()),
                Instruction::LoadBool { value, .. } => Value::Bool(*value),
                Instruction::LoadNil { .. } => Value::Nil,
                Instruction::LoadClass { class, .. } => {
                    let Some(class) = classes.get(*class).copied() else {
                        return Err(MachineError::Class(ClassError::ClassIdentityExhausted));
                    };
                    Value::Class(class)
                }
                Instruction::LoadContract { contract, .. } => {
                    Value::Contract(ContractId::new(*contract as u64 + 1))
                }
                Instruction::LoadGlobal { name, .. } => self
                    .globals
                    .get(name)
                    .cloned()
                    .ok_or(MachineError::NameError)?,
                Instruction::StoreGlobal { name, value, .. } => {
                    let value = registers[*value as usize].clone();
                    self.globals.insert(name.clone(), value.clone());
                    value
                }
                Instruction::Move { source, .. } => registers[*source as usize].clone(),
                Instruction::DeclareDeferred { .. } => continue,
                Instruction::RaiseDefiniteAssignment { .. } => {
                    return Err(MachineError::DefiniteAssignment);
                }
                Instruction::Binary {
                    selector,
                    left,
                    right,
                    ..
                } => {
                    let left = registers[*left as usize].clone();
                    let right = registers[*right as usize].clone();
                    run_frame!(
                        'frame,
                        self.binary_send(selector, left, right, program, classes)
                    )
                }
                Instruction::Unary {
                    selector, operand, ..
                } => {
                    let operand = registers[*operand as usize].clone();
                    self.send(selector, operand, &[])?
                }
                Instruction::BuildArray { first, count, .. } => {
                    let start = *first as usize;
                    let elements = registers[start..start + *count as usize].to_vec();
                    Value::Array(iris_runtime::ArrayRef::new(elements))
                }
                Instruction::LoadBuiltinType { name, .. } => {
                    Value::Type(self.builtin_class(name)?, Vec::new())
                }
                Instruction::LoadBuiltinClass { name, .. } => {
                    Value::Class(self.builtin_class(name)?)
                }
                Instruction::BuildTuple { first, count, .. } => {
                    let start = *first as usize;
                    Value::Tuple(registers[start..start + *count as usize].to_vec())
                }
                Instruction::BuildRange {
                    start,
                    end,
                    inclusive_end,
                    ..
                } => {
                    let Value::Integer(start) = &registers[*start as usize] else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let Value::Integer(end) = &registers[*end as usize] else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let Value::Bool(descending) = self.send(
                        "<",
                        Value::Integer(end.clone()),
                        &[Value::Integer(start.clone())],
                    )?
                    else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    Value::Range(Box::new(iris_runtime::RangeValue {
                        start: start.clone(),
                        end: end.clone(),
                        inclusive_end: *inclusive_end,
                        step: if descending { "-1" } else { "1" }
                            .parse()
                            .map_err(|_| MachineError::Kernel(KernelError::Type))?,
                    }))
                }
                Instruction::BuildHash { first, count, .. } => {
                    let start = *first as usize;
                    let mut entries = Vec::with_capacity(*count as usize);
                    for pair in registers[start..start + *count as usize * 2].chunks_exact(2) {
                        iris_runtime::public_hash(&pair[0])
                            .map_err(KernelError::StableHash)
                            .map_err(MachineError::Kernel)?;
                        if let Some((_, value)) =
                            entries.iter_mut().find(|(key, _)| *key == pair[0])
                        {
                            *value = pair[1].clone();
                        } else {
                            entries.push((pair[0].clone(), pair[1].clone()));
                        }
                    }
                    Value::Hash(iris_runtime::HashRef::new(entries))
                }
                Instruction::MakeClosure {
                    function,
                    first,
                    count,
                    ..
                } => {
                    let start = *first as usize;
                    let identity = iris_runtime::ObjectId::new(self.next_closure);
                    self.next_closure = self.next_closure.saturating_add(1);
                    self.closures.insert(
                        identity,
                        ClosureRecord {
                            function: *function,
                            captures: registers[start..start + *count as usize].to_vec(),
                        },
                    );
                    Value::Closure(identity)
                }
                Instruction::Index {
                    receiver, index, ..
                } => self.index(
                    registers[*receiver as usize].clone(),
                    registers[*index as usize].clone(),
                )?,
                Instruction::SetIndex {
                    receiver,
                    index,
                    value,
                    ..
                } => self.set_index(
                    registers[*receiver as usize].clone(),
                    registers[*index as usize].clone(),
                    registers[*value as usize].clone(),
                )?,
                Instruction::BindMember {
                    receiver, selector, ..
                } => {
                    if let Value::Class(class) = registers[*receiver as usize] {
                        match selector.as_str() {
                            "type" => Value::Type(class, Vec::new()),
                            "name" => classes
                                .iter()
                                .position(|known| *known == class)
                                .and_then(|index| program.classes.get(index))
                                .map(|declaration| Value::Symbol(declaration.name.clone()))
                                .unwrap_or(Value::Nil),
                            _ => return Err(MachineError::UnknownSelector(selector.clone())),
                        }
                    } else {
                        let Value::Object(object) = registers[*receiver as usize] else {
                            return Err(MachineError::UnknownSelector(selector.clone()));
                        };
                        let bound_selector = selector_id(program, selector)
                            .ok_or_else(|| MachineError::UnknownSelector(selector.clone()))?;
                        let class = self
                            .runtime
                            .class_of(object)
                            .map_err(MachineError::Construction)?;
                        let class_index = classes
                            .iter()
                            .position(|known| *known == class)
                            .ok_or(MachineError::Class(ClassError::ClassIdentityExhausted))?;
                        if program.classes[class_index]
                            .stored_properties
                            .iter()
                            .any(|property| property.name == *selector)
                        {
                            let selector = selector_id(program, selector)
                                .ok_or_else(|| MachineError::UnknownSelector(selector.clone()))?;
                            self.runtime
                                .raw_ivar(object, selector)
                                .map_err(MachineError::Construction)?
                        } else if program.classes[class_index]
                            .property_methods
                            .iter()
                            .any(|property| property == selector)
                        {
                            let selector_id = selector_id(program, selector)
                                .ok_or_else(|| MachineError::UnknownSelector(selector.clone()))?;
                            let method = self
                                .runtime
                                .dispatch_instance(object, selector_id)
                                .map_err(MachineError::Construction)?;
                            let function = usize::try_from(method.body().raw()).map_err(|_| {
                                MachineError::Invalid(VerifyError::UnknownFunction {
                                    function: usize::MAX,
                                })
                            })?;
                            let callee = program.functions.get(function).cloned().ok_or(
                                MachineError::Invalid(VerifyError::UnknownFunction { function }),
                            )?;
                            let returned = run_frame!('frame, self.run_body(
                                &callee.instructions,
                                callee.registers,
                                vec![Value::Object(object)],
                                program,
                                classes,
                            ));
                            returned.into_iter().next().unwrap_or(Value::Nil)
                        } else {
                            self.runtime
                                .registry_mut()
                                .bind_instance(object, class, bound_selector)
                                .map(Value::BoundMethod)
                                .map_err(iris_runtime::ConstructionError::from)
                                .map_err(MachineError::Construction)?
                        }
                    }
                }
                Instruction::Identity { left, right, .. } => {
                    self.identity(&registers[*left as usize], &registers[*right as usize])?
                }
                Instruction::TypeTest { value, target, .. } => {
                    self.type_test(&registers[*value as usize], &registers[*target as usize])?
                }
                // C113 fixes the accepted range per WIDTH and requires
                // RangeError outside it; C114 requires the round trip to hold
                // for every pattern including signaling NaN, so the bits are
                // reinterpreted rather than converted numerically.
                Instruction::FromBits { width, bits, .. } => {
                    let Value::Integer(bits) = &registers[*bits as usize] else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    // The runtime spells an out-of-range width `Numeric(Range)`
                    // and the reference answers exactly that, so inventing a
                    // separate error would make the backends disagree for a
                    // reason that is not semantic.
                    let Some(bits) = bits.to_u64() else {
                        return Err(MachineError::Kernel(KernelError::Numeric(
                            NumericError::Range,
                        )));
                    };
                    match width {
                        FloatWidth::Bits32 => {
                            let Ok(bits) = u32::try_from(bits) else {
                                return Err(MachineError::Kernel(KernelError::Numeric(
                                    NumericError::Range,
                                )));
                            };
                            Value::Float32(f32::from_bits(bits))
                        }
                        FloatWidth::Bits64 => Value::Float64(f64::from_bits(bits)),
                    }
                }
                // Truth is decided by the RUNTIME rather than re-derived here.
                // `IRIS-V1-CONTROL-C022` makes only `false` and `nil` falsey,
                // and a second copy of that rule would be one more place for
                // the backends to diverge.
                Instruction::JumpUnless { condition, target } => {
                    if !truthy(&registers[*condition as usize]) {
                        counter = *target;
                    }
                    continue;
                }
                Instruction::Jump { target } => {
                    counter = *target;
                    continue;
                }
                Instruction::ArrayVersion { array, .. } => {
                    let Value::Array(array) = &registers[*array as usize] else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    Value::Integer(iris_runtime::IntegerValue::from(array.version()))
                }
                Instruction::ArrayNext {
                    array,
                    index,
                    version,
                    exhausted,
                    ..
                } => {
                    let Value::Integer(expected) = &registers[*version as usize] else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let Value::Array(array) = &registers[*array as usize] else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    // C026 raises on the iterator's NEXT advance once the
                    // Array changed, so this is checked before the element is
                    // read rather than after the loop finishes.
                    if expected.to_u64() != Some(array.version()) {
                        return Err(MachineError::ConcurrentModification);
                    }
                    let Value::Integer(index) = &registers[*index as usize] else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let Some(index) = index.to_usize() else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let elements = array.elements();
                    let Some(value) = elements.get(index).cloned() else {
                        counter = *exhausted;
                        continue;
                    };
                    value
                }
                Instruction::RangeNext {
                    range,
                    index,
                    exhausted,
                    ..
                } => {
                    let Value::Range(range) = &registers[*range as usize] else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let Value::Integer(index) = &registers[*index as usize] else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let offset = self.send(
                        "*",
                        Value::Integer(index.clone()),
                        &[Value::Integer(range.step.clone())],
                    )?;
                    let Value::Integer(value) =
                        self.send("+", Value::Integer(range.start.clone()), &[offset])?
                    else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let descending = range.step.decimal_text().starts_with('-');
                    let selector = match (descending, range.inclusive_end) {
                        (false, false) => "<",
                        (false, true) => "<=",
                        (true, false) => ">",
                        (true, true) => ">=",
                    };
                    let Value::Bool(within) = self.send(
                        selector,
                        Value::Integer(value.clone()),
                        &[Value::Integer(range.end.clone())],
                    )?
                    else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    if !within {
                        counter = *exhausted;
                        continue;
                    }
                    Value::Integer(value)
                }
                Instruction::EnterTry {
                    handler, exception, ..
                } => {
                    handlers.push((*handler, *exception));
                    continue;
                }
                Instruction::CatchMatch {
                    exception, class, ..
                } => Value::Bool(self.catch_matches(
                    &registers[*exception as usize],
                    class,
                    program,
                    classes,
                )?),
                Instruction::LeaveTry => {
                    handlers.pop();
                    continue;
                }
                Instruction::Raise { value } => {
                    let value = registers[*value as usize].clone();
                    let Some((handler, exception)) = handlers.pop() else {
                        return Err(MachineError::Raised(value));
                    };
                    registers[exception as usize] = value;
                    counter = handler;
                    continue;
                }
                Instruction::Return { value } => {
                    let value = registers[*value as usize].clone();
                    // The answer is handed back in the frame's own result
                    // slot, so a caller reads it without knowing the callee's
                    // register layout.
                    return Ok(vec![value]);
                }
                Instruction::Call {
                    function,
                    first,
                    count,
                    ..
                } => {
                    let Some(callee) = program.functions.get(*function).cloned() else {
                        return Err(MachineError::Invalid(VerifyError::UnknownFunction {
                            function: *function,
                        }));
                    };
                    let start = *first as usize;
                    let arguments = registers[start..start + *count as usize].to_vec();
                    let returned = run_frame!('frame, self.run_body(
                        &callee.instructions,
                        callee.registers,
                        arguments,
                        program,
                        classes,
                    ));
                    returned.into_iter().next().unwrap_or(Value::Nil)
                }
                Instruction::New {
                    class,
                    first,
                    count,
                    ..
                } => {
                    let Some(class) = classes.get(*class).copied() else {
                        return Err(MachineError::Class(ClassError::ClassIdentityExhausted));
                    };
                    let start = *first as usize;
                    let arguments = registers[start..start + *count as usize].to_vec();
                    let object = self
                        .runtime
                        .allocate(class)
                        .map_err(MachineError::Construction)?;
                    self.initialize_properties(program, classes, class, object)?;
                    if let Ok(method) = self.runtime.dispatch_instance(object, Selector::INITIALIZE)
                    {
                        let function = usize::try_from(method.body().raw()).map_err(|_| {
                            MachineError::Invalid(VerifyError::UnknownFunction {
                                function: usize::MAX,
                            })
                        })?;
                        let mut passed = Vec::with_capacity(arguments.len() + 1);
                        passed.push(Value::Object(object));
                        passed.extend(arguments);
                        let callee = program.functions.get(function).cloned().ok_or(
                            MachineError::Invalid(VerifyError::UnknownFunction { function }),
                        )?;
                        let _ = run_frame!('frame, self.run_body(
                            &callee.instructions,
                            callee.registers,
                            passed,
                            program,
                            classes,
                        ));
                    }
                    Value::Object(object)
                }
                Instruction::Send {
                    receiver,
                    selector,
                    first,
                    count,
                    ..
                } => {
                    let receiver = registers[*receiver as usize].clone();
                    let start = *first as usize;
                    let arguments = registers[start..start + *count as usize].to_vec();
                    if let Some(value) = run_frame!(
                        'frame,
                        self.authored_send(
                            &receiver,
                            selector,
                            &arguments,
                            program,
                            classes,
                        )
                    ) {
                        if let Some(destination) = instruction.destination() {
                            registers[destination as usize] = value;
                        }
                        continue;
                    }
                    if selector == "call" {
                        let (callee, passed) = match receiver {
                            Value::Closure(identity) => {
                                let Some(closure) = self.closures.get(&identity).cloned() else {
                                    return Err(MachineError::Kernel(KernelError::Type));
                                };
                                let Some(callee) = program.functions.get(closure.function).cloned()
                                else {
                                    return Err(MachineError::Invalid(
                                        VerifyError::UnknownFunction {
                                            function: closure.function,
                                        },
                                    ));
                                };
                                let mut passed = closure.captures;
                                passed.extend(arguments);
                                (callee, passed)
                            }
                            Value::BoundMethod(bound) => {
                                let (class, receiver) = match bound.receiver() {
                                    BoundReceiver::Object(object) => (
                                        self.runtime
                                            .class_of(object)
                                            .map_err(MachineError::Construction)?,
                                        Value::Object(object),
                                    ),
                                    BoundReceiver::Class(class) => (class, Value::Class(class)),
                                };
                                self.runtime
                                    .registry()
                                    .validate_method_binding(class, bound.method())
                                    .map_err(ConstructionError::from)
                                    .map_err(MachineError::Construction)?;
                                let function = usize::try_from(bound.method().body().raw())
                                    .map_err(|_| {
                                        MachineError::Invalid(VerifyError::UnknownFunction {
                                            function: usize::MAX,
                                        })
                                    })?;
                                let Some(callee) = program.functions.get(function).cloned() else {
                                    return Err(MachineError::Invalid(
                                        VerifyError::UnknownFunction { function },
                                    ));
                                };
                                let mut passed = Vec::with_capacity(arguments.len() + 1);
                                passed.push(receiver);
                                passed.extend(arguments);
                                (callee, passed)
                            }
                            receiver => {
                                let value = self.send(selector, receiver, &arguments)?;
                                if let Some(destination) = instruction.destination() {
                                    registers[destination as usize] = value;
                                }
                                continue;
                            }
                        };
                        if passed.len() != callee.parameters {
                            return Err(MachineError::Kernel(KernelError::Arity));
                        }
                        let returned = run_frame!('frame, self.run_body(
                            &callee.instructions,
                            callee.registers,
                            passed,
                            program,
                            classes,
                        ));
                        returned.into_iter().next().unwrap_or(Value::Nil)
                    } else {
                        let object = match receiver {
                            Value::Class(class) if selector == "new" => {
                                let object = self
                                    .runtime
                                    .allocate(class)
                                    .map_err(MachineError::Construction)?;
                                self.initialize_properties(program, classes, class, object)?;
                                if let Ok(method) =
                                    self.runtime.dispatch_instance(object, Selector::INITIALIZE)
                                {
                                    let function =
                                        usize::try_from(method.body().raw()).map_err(|_| {
                                            MachineError::Invalid(VerifyError::UnknownFunction {
                                                function: usize::MAX,
                                            })
                                        })?;
                                    let mut passed = Vec::with_capacity(arguments.len() + 1);
                                    passed.push(Value::Object(object));
                                    passed.extend(arguments);
                                    let callee = program.functions.get(function).cloned().ok_or(
                                        MachineError::Invalid(VerifyError::UnknownFunction {
                                            function,
                                        }),
                                    )?;
                                    let _ = run_frame!('frame, self.run_body(
                                        &callee.instructions,
                                        callee.registers,
                                        passed,
                                        program,
                                        classes,
                                    ));
                                }
                                if let Some(destination) = instruction.destination() {
                                    registers[destination as usize] = Value::Object(object);
                                }
                                continue;
                            }
                            Value::Class(class) => {
                                let selector_id =
                                    selector_id(program, selector).ok_or_else(|| {
                                        MachineError::UnknownSelector(selector.clone())
                                    })?;
                                let method = match self
                                    .runtime
                                    .registry()
                                    .dispatch_class_object(class, selector_id)
                                    .map_err(iris_runtime::ConstructionError::from)
                                    .map_err(MachineError::Construction)?
                                {
                                    iris_runtime::DispatchOutcome::Invoke(method) => method,
                                    iris_runtime::DispatchOutcome::WouldInvokeMethodMissing {
                                        selector,
                                    } => {
                                        return Err(MachineError::Construction(
                                            iris_runtime::DispatchError::MissingMethod { selector }
                                                .into(),
                                        ));
                                    }
                                };
                                let function =
                                    usize::try_from(method.body().raw()).map_err(|_| {
                                        MachineError::Invalid(VerifyError::UnknownFunction {
                                            function: usize::MAX,
                                        })
                                    })?;
                                let mut passed = Vec::with_capacity(arguments.len() + 1);
                                passed.push(Value::Class(class));
                                passed.extend(arguments);
                                let callee = program.functions.get(function).cloned().ok_or(
                                    MachineError::Invalid(VerifyError::UnknownFunction {
                                        function,
                                    }),
                                )?;
                                let returned = run_frame!('frame, self.run_body(
                                    &callee.instructions,
                                    callee.registers,
                                    passed,
                                    program,
                                    classes,
                                ));
                                let value = returned.into_iter().next().unwrap_or(Value::Nil);
                                if let Some(destination) = instruction.destination() {
                                    registers[destination as usize] = value;
                                }
                                continue;
                            }
                            Value::Object(object) => object,
                            receiver => {
                                let value = self.send(selector, receiver, &arguments)?;
                                if let Some(destination) = instruction.destination() {
                                    registers[destination as usize] = value;
                                }
                                continue;
                            }
                        };
                        let selector = selector_id(program, selector)
                            .ok_or_else(|| MachineError::UnknownSelector(selector.clone()))?;
                        let method = self
                            .runtime
                            .dispatch_instance(object, selector)
                            .map_err(MachineError::Construction)?;
                        let function = usize::try_from(method.body().raw()).map_err(|_| {
                            MachineError::Invalid(VerifyError::UnknownFunction {
                                function: usize::MAX,
                            })
                        })?;
                        let mut arguments = Vec::with_capacity(*count as usize + 1);
                        arguments.push(Value::Object(object));
                        arguments.extend_from_slice(&registers[start..start + *count as usize]);
                        let callee = program.functions.get(function).cloned().ok_or(
                            MachineError::Invalid(VerifyError::UnknownFunction { function }),
                        )?;
                        let returned = run_frame!('frame, self.run_body(
                            &callee.instructions,
                            callee.registers,
                            arguments,
                            program,
                            classes,
                        ));
                        returned.into_iter().next().unwrap_or(Value::Nil)
                    }
                }
                Instruction::SendClass {
                    class,
                    selector,
                    first,
                    count,
                    ..
                } => {
                    let Some(class) = classes.get(*class).copied() else {
                        return Err(MachineError::Class(ClassError::ClassIdentityExhausted));
                    };
                    let selector = selector_id(program, selector)
                        .ok_or_else(|| MachineError::UnknownSelector(selector.clone()))?;
                    let method = match self
                        .runtime
                        .registry()
                        .dispatch_class_object(class, selector)
                        .map_err(iris_runtime::ConstructionError::from)
                        .map_err(MachineError::Construction)?
                    {
                        iris_runtime::DispatchOutcome::Invoke(method) => method,
                        iris_runtime::DispatchOutcome::WouldInvokeMethodMissing { selector } => {
                            return Err(MachineError::Construction(
                                iris_runtime::DispatchError::MissingMethod { selector }.into(),
                            ));
                        }
                    };
                    let function = usize::try_from(method.body().raw()).map_err(|_| {
                        MachineError::Invalid(VerifyError::UnknownFunction {
                            function: usize::MAX,
                        })
                    })?;
                    let start = *first as usize;
                    let mut arguments = Vec::with_capacity(*count as usize + 1);
                    arguments.push(Value::Class(class));
                    arguments.extend_from_slice(&registers[start..start + *count as usize]);
                    let callee =
                        program
                            .functions
                            .get(function)
                            .cloned()
                            .ok_or(MachineError::Invalid(VerifyError::UnknownFunction {
                                function,
                            }))?;
                    let returned = run_frame!('frame, self.run_body(
                        &callee.instructions,
                        callee.registers,
                        arguments,
                        program,
                        classes,
                    ));
                    returned.into_iter().next().unwrap_or(Value::Nil)
                }
                Instruction::ContractCast {
                    receiver, contract, ..
                } => {
                    let receiver = registers[*receiver as usize].clone();
                    let Value::Object(object) = receiver else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let class = self
                        .runtime
                        .class_of(object)
                        .map_err(MachineError::Construction)?;
                    let conforms = classes
                        .iter()
                        .position(|known| *known == class)
                        .is_some_and(|index| program.classes[index].contracts.contains(contract));
                    if !conforms {
                        return Err(MachineError::Kernel(KernelError::Type));
                    }
                    Value::ContractView(
                        Box::new(Value::Object(object)),
                        ContractId::new(*contract as u64 + 1),
                    )
                }
                Instruction::SendContract {
                    receiver,
                    selector,
                    first,
                    count,
                    ..
                } => {
                    let Value::ContractView(receiver, contract) =
                        registers[*receiver as usize].clone()
                    else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let Value::Object(object) = *receiver else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let contract_index = usize::try_from(contract.raw().saturating_sub(1))
                        .map_err(|_| MachineError::Kernel(KernelError::Type))?;
                    let required = program
                        .contracts
                        .get(contract_index)
                        .is_some_and(|contract| {
                            contract
                                .requirements
                                .iter()
                                .any(|(name, arity)| name == selector && *arity == *count as usize)
                        });
                    if !required {
                        return Err(MachineError::MessageNotFound {
                            receiver_class: "ContractView".to_owned(),
                            selector: selector.clone(),
                        });
                    }
                    let selector_id = selector_id(program, selector)
                        .ok_or_else(|| MachineError::UnknownSelector(selector.clone()))?;
                    let method = self
                        .runtime
                        .dispatch_instance(object, selector_id)
                        .map_err(MachineError::Construction)?;
                    let function = usize::try_from(method.body().raw()).map_err(|_| {
                        MachineError::Invalid(VerifyError::UnknownFunction {
                            function: usize::MAX,
                        })
                    })?;
                    let callee =
                        program
                            .functions
                            .get(function)
                            .cloned()
                            .ok_or(MachineError::Invalid(VerifyError::UnknownFunction {
                                function,
                            }))?;
                    let start = *first as usize;
                    let mut arguments = Vec::with_capacity(*count as usize + 1);
                    arguments.push(Value::Object(object));
                    arguments.extend_from_slice(&registers[start..start + *count as usize]);
                    let returned = run_frame!('frame, self.run_body(
                        &callee.instructions,
                        callee.registers,
                        arguments,
                        program,
                        classes,
                    ));
                    returned.into_iter().next().unwrap_or(Value::Nil)
                }
                Instruction::GetIvar { receiver, name, .. } => {
                    let Value::Object(object) = registers[*receiver as usize] else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let selector = selector_id(program, name)
                        .ok_or_else(|| MachineError::UnknownSelector(name.clone()))?;
                    self.runtime
                        .raw_ivar(object, selector)
                        .map_err(MachineError::Construction)?
                }
                Instruction::SetIvar {
                    receiver,
                    name,
                    value,
                    ..
                } => {
                    let Value::Object(object) = registers[*receiver as usize] else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let selector = selector_id(program, name)
                        .ok_or_else(|| MachineError::UnknownSelector(name.clone()))?;
                    self.runtime
                        .assign_raw_ivar(object, selector, registers[*value as usize].clone())
                        .map_err(MachineError::Construction)?
                }
                Instruction::GetClassVar { receiver, name, .. } => {
                    let class = self.receiver_class(&registers[*receiver as usize])?;
                    let selector = selector_id(program, name)
                        .ok_or_else(|| MachineError::UnknownSelector(name.clone()))?;
                    self.runtime
                        .class_var(class, selector)
                        .map_err(MachineError::Construction)?
                        .ok_or(MachineError::NameError)?
                }
                Instruction::SetClassVar {
                    receiver,
                    name,
                    value,
                    ..
                } => {
                    let class = self.receiver_class(&registers[*receiver as usize])?;
                    let selector = selector_id(program, name)
                        .ok_or_else(|| MachineError::UnknownSelector(name.clone()))?;
                    self.runtime
                        .assign_class_var(class, selector, registers[*value as usize].clone())
                        .map_err(MachineError::Construction)?
                }
            };
            if let Some(destination) = instruction.destination() {
                registers[destination as usize] = produced;
            }
        }
        Ok(registers)
    }
}
