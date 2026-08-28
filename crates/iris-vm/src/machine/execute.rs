//! Register-frame instruction execution.

use iris_runtime::{
    BoundReceiver, ClassError, ClassId, ConstructionError, ContractId, KernelError, NumericError,
    Selector, Value,
};

use crate::compile::{FloatWidth, Instruction, Program, Register};

use super::{ClosureRecord, Machine, MachineError, PendingFrame, VerifyError, selector_id, truthy};

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
        self.run_frame_from(instructions, size, arguments, program, classes, None)
    }

    /// Runs a frame, optionally RESUMING one that suspended at an `await`.
    ///
    /// A resumed frame keeps the register file and handler stack it paused
    /// with, and continues at the instruction after the `await`, with the
    /// Gate's posted value written into that await's destination.
    pub(super) fn run_frame_from(
        &mut self,
        instructions: &[Instruction],
        size: usize,
        arguments: Vec<Value>,
        program: &Program,
        classes: &[ClassId],
        resume: Option<(PendingFrame, Value)>,
    ) -> Result<Vec<Value>, MachineError> {
        // Verification proved every read is in range and written, so indexing
        // below cannot be out of bounds and no operand check is repeated.
        // A caller may pass MORE arguments than the signature declares - two
        // keywords for one `key` parameter, or extra positionals for a
        // `*rest` - and the frame is entered with all of them, so the file has
        // to hold what arrived rather than only what was verified.
        let arity = arguments.len();
        let mut registers = vec![Value::Nil; size.max(arity)];
        // Parameters arrive pre-bound in the leading registers.
        for (slot, argument) in arguments.into_iter().enumerate() {
            registers[slot] = argument;
        }

        let mut counter = 0;
        let mut handlers: Vec<(usize, Register, Register)> = Vec::new();
        if let Some((frame, posted)) = resume {
            registers = frame.registers;
            counter = frame.counter;
            handlers = frame.handlers;
            if let Some(destination) = frame.destination {
                registers[destination as usize] = posted;
            }
        }
        // A NAMED runtime error is catchable, so it is handed to a handler as
        // a Symbol. If no handler answers it, the program must still fail with
        // the ORIGINAL error rather than with a raised Symbol: an uncaught
        // ConcurrentModification is that error, not `Raised(:...)`.
        let mut converted: Option<MachineError> = None;
        macro_rules! run_frame {
            ($label:lifetime, $call:expr) => {
                match $call {
                    Ok(values) => values,
                    Err(MachineError::Raised(propagation)) => {
                        let (value, context) = *propagation;
                        let Some((handler, exception, context_register)) = handlers.pop() else {
                            return Err(MachineError::Raised(Box::new((value, context))));
                        };
                        registers[exception as usize] = value;
                        registers[context_register as usize] = context;
                        counter = handler;
                        continue $label;
                    }
                    // A failure the specification NAMES is an ordinary
                    // catchable Iris error, so it belongs to the innermost
                    // handler rather than to the frame boundary. Returning it
                    // here let `try { [].iterator().next().value } catch ...`
                    // escape the catch entirely - the handler was reachable
                    // and simply never consulted.
                    Err(error) => match (super::catchable_name(&error), handlers.pop()) {
                        (Some(name), Some((handler, exception, context_register))) => {
                            registers[exception as usize] = Value::Symbol(name.to_owned());
                            registers[context_register as usize] = Value::Nil;
                            converted = Some(error);
                            counter = handler;
                            continue $label;
                        }
                        _ => return Err(error),
                    },
                }
            };
        }
        'frame: while let Some(instruction) = instructions.get(counter) {
            counter += 1;
            // Every instruction's own failure goes through `dispatch` so a
            // NAMED runtime error reaches the innermost handler. Letting the
            // `?` operators inside leave the frame directly meant a `try`
            // around `a[0] = 1` never saw the IndexError: the handler was on
            // the stack and simply never consulted.
            macro_rules! dispatch {
                ($body:expr) => {
                    match (|| -> Result<Value, MachineError> { Ok($body) })() {
                        Ok(value) => value,
                        Err(MachineError::Raised(propagation)) => {
                            let (value, context) = *propagation;
                            let Some((handler, exception, context_register)) = handlers.pop()
                            else {
                                return Err(MachineError::Raised(Box::new((value, context))));
                            };
                            registers[exception as usize] = value;
                            registers[context_register as usize] = context;
                            counter = handler;
                            continue 'frame;
                        }
                        Err(error) => match (super::catchable_name(&error), handlers.pop()) {
                            (Some(name), Some((handler, exception, context_register))) => {
                                registers[exception as usize] = Value::Symbol(name.to_owned());
                                registers[context_register as usize] = Value::Nil;
                                converted = Some(error);
                                counter = handler;
                                continue 'frame;
                            }
                            _ => return Err(error),
                        },
                    }
                };
            }
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
                Instruction::LoadBytes { bytes, .. } => Value::Bytes(bytes.clone()),
                Instruction::LoadByteArray { bytes, .. } => {
                    Value::ByteArray(iris_runtime::ByteArrayRef::new(bytes.clone()))
                }
                Instruction::MakeMutableString { source, .. } => {
                    let Value::Text(text) = &registers[*source as usize] else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    Value::MutableString(iris_runtime::MutableStringRef::new(text.clone()))
                }
                Instruction::LoadSymbol { name, .. } => Value::Symbol(name.clone()),
                Instruction::LoadBool { value, .. } => Value::Bool(*value),
                Instruction::LoadNil { .. } => Value::Nil,
                Instruction::LoadIterationDone { .. } => Value::IterationDone,
                Instruction::BuildIterationYield { value, .. } => {
                    Value::IterationYield(Box::new(registers[*value as usize].clone()))
                }
                Instruction::LoadClass { class, .. } => {
                    let Some(class) = classes.get(*class).copied() else {
                        return Err(MachineError::Class(ClassError::ClassIdentityExhausted));
                    };
                    Value::Class(class)
                }
                Instruction::LoadType { class, .. } => {
                    let Some(class) = classes.get(*class).copied() else {
                        return Err(MachineError::Class(ClassError::ClassIdentityExhausted));
                    };
                    Value::Type(class, Vec::new())
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
                Instruction::PublishBinding { name, source, .. } => {
                    let value = registers[*source as usize].clone();
                    self.bindings.insert(name.clone(), value.clone());
                    value
                }
                Instruction::LoadBinding { name, shared, .. } => {
                    let value = self
                        .bindings
                        .get(name)
                        .cloned()
                        .ok_or(MachineError::NameError)?;
                    if *shared {
                        let Value::Array(cell) = value else {
                            return Err(MachineError::Kernel(KernelError::Type));
                        };
                        cell.get(0).ok_or(MachineError::Kernel(KernelError::Type))?
                    } else {
                        value
                    }
                }
                Instruction::StoreBinding { name, source, .. } => {
                    let Some(Value::Array(cell)) = self.bindings.get(name) else {
                        return Err(MachineError::NameError);
                    };
                    let value = registers[*source as usize].clone();
                    cell.mutate(|elements| elements[0] = value.clone());
                    value
                }
                Instruction::Move { source, .. } => registers[*source as usize].clone(),
                Instruction::MakeCell { source, .. } => {
                    Value::Array(iris_runtime::ArrayRef::new(vec![
                        registers[*source as usize].clone(),
                    ]))
                }
                Instruction::LoadCell { cell, .. } => {
                    let Value::Array(cell) = &registers[*cell as usize] else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    cell.get(0).ok_or(MachineError::Kernel(KernelError::Type))?
                }
                Instruction::StoreCell { cell, source, .. } => {
                    let Value::Array(cell) = &registers[*cell as usize] else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let value = registers[*source as usize].clone();
                    cell.mutate(|elements| elements[0] = value.clone());
                    value
                }
                Instruction::DeclareDeferred { assigned, .. } => {
                    registers[*assigned as usize] = Value::Bool(false);
                    Value::Nil
                }
                Instruction::MarkAssigned { .. } => Value::Bool(true),
                Instruction::ReadDeferred {
                    value, assigned, ..
                } => {
                    if !truthy(&registers[*assigned as usize]) {
                        return Err(MachineError::DefiniteAssignment);
                    }
                    registers[*value as usize].clone()
                }
                Instruction::TestTruth { value, .. } => {
                    let value = registers[*value as usize].clone();
                    Value::Bool(run_frame!(
                        'frame,
                        self.test_truth(&value, program, classes)
                    ))
                }
                Instruction::MakeKeywordArgument { name, value, .. } => Value::KeywordArgument(
                    name.clone(),
                    Box::new(registers[*value as usize].clone()),
                ),
                Instruction::RaiseEncodingSelection { code, .. } => {
                    return Err(MachineError::LexicalDiagnostic(code));
                }
                Instruction::EncodingDecode {
                    encoding,
                    value,
                    first,
                    count,
                    ..
                } => {
                    let value = registers[*value as usize].clone();
                    let start = *first as usize;
                    let options = registers[start..start + *count as usize].to_vec();
                    dispatch!(Self::encoding_decode(encoding, &value, &options)?)
                }
                Instruction::FfiOpen {
                    path, first, count, ..
                } => {
                    let path = registers[*path as usize].clone();
                    let start = *first as usize;
                    let options = registers[start..start + *count as usize].to_vec();
                    run_frame!('frame, self.ffi_open(&path, &options, program, classes))
                }
                Instruction::IrisValueEncode { value, .. } => {
                    let value = registers[*value as usize].clone();
                    run_frame!('frame, self.irisvalue_encode(value, program, classes))
                }
                Instruction::IrisValueDecode {
                    stream,
                    first,
                    count,
                    ..
                } => {
                    let stream = registers[*stream as usize].clone();
                    let start = *first as usize;
                    let options = registers[start..start + *count as usize].to_vec();
                    run_frame!(
                        'frame,
                        self.irisvalue_decode(stream, &options, program, classes)
                    )
                }
                Instruction::EscapeRegex { value, .. } => {
                    let value = registers[*value as usize].clone();
                    let text = dispatch!(Value::Text(self.text_operand(value, program, classes)?));
                    let Value::Text(text) = text else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    Value::Text(regex::escape(&text))
                }
                Instruction::MakeRegex { pattern, flags, .. } => {
                    let Value::Text(pattern) = &registers[*pattern as usize] else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let pattern = pattern.clone();
                    dispatch!(Self::make_regex(&pattern, flags)?)
                }
                Instruction::CheckReturn { value, annotation } => {
                    let value = registers[*value as usize].clone();
                    if !self.annotation_admits(&value, annotation, program, classes)? {
                        return Err(MachineError::TypeContractError);
                    }
                    continue;
                }
                Instruction::ApplyReopen { class, reopen } => {
                    self.apply_reopen(program, classes, *class, *reopen)?;
                    continue;
                }
                Instruction::NativeFixture {
                    selector,
                    first,
                    count,
                    ..
                } => {
                    let start = *first as usize;
                    let arguments = registers[start..start + *count as usize].to_vec();
                    dispatch!(self.native_fixture(selector, &arguments)?)
                }
                Instruction::DiscardedContexts { .. } => {
                    Value::Array(iris_runtime::ArrayRef::new(self.discarded_contexts.clone()))
                }
                Instruction::PackageValidate { claims_core, .. } => {
                    if *claims_core {
                        return Err(MachineError::LexicalDiagnostic("PACKAGE_CORE_ABI_CLAIM"));
                    }
                    Value::Symbol("validated".to_owned())
                }
                Instruction::UnicodeVersion { .. } => {
                    let (major, minor, patch) = unicode_normalization::UNICODE_VERSION;
                    Value::Text(format!("{major}.{minor}.{patch}"))
                }
                Instruction::GateNew { .. } => {
                    let identity = iris_runtime::ObjectId::new(self.next_context);
                    self.next_context = self.next_context.saturating_add(1);
                    self.gates.insert(identity, None);
                    Value::Gate(identity)
                }
                Instruction::GateComplete { gate, value, .. } => {
                    let Value::Gate(identity) = registers[*gate as usize] else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let posted = registers[*value as usize].clone();
                    // Completing a Gate makes its parked frames READY, it does
                    // not run them: the reference resumes when a Task is
                    // OBSERVED, so `Gate.complete(g, 1); log` still shows only
                    // the prefix. Resuming here ran the continuation early and
                    // made the effect visible before anything observed it.
                    self.gates.insert(identity, Some(posted));
                    Value::Nil
                }
                Instruction::LoadRegex { pattern, flags, .. } => {
                    Value::Regex(Box::new(iris_runtime::RegexValue {
                        pattern: pattern.clone(),
                        flags: flags.clone(),
                    }))
                }
                Instruction::NegateTruth { value, .. } => {
                    Value::Bool(!truthy(&registers[*value as usize]))
                }
                Instruction::RaiseUnsupported { .. } => {
                    return Err(MachineError::UnsupportedConstruct);
                }
                // `IRIS-V1-CONTROL-C023` binds the categories: positionals in
                // order, `*rest` taking the remainder as a fresh Array, a
                // `key` parameter by NAME, and `**kwargs` collecting the
                // keywords nothing else matched. It happens HERE because a
                // dynamic send does not know the signature until dispatch.
                Instruction::BindParameters {
                    kinds,
                    receiver,
                    first,
                    count,
                    ..
                } => {
                    // The window spans what the caller ACTUALLY passed, not the
                    // signature's width. Sizing it by the signature padded the
                    // arguments with the unset registers a wider frame carries,
                    // so `*rest` collected `[nil, nil]` where it should have
                    // collected nothing - a wrong answer rather than a gap.
                    // A caller may also pass MORE than the signature declares:
                    // two keywords for one `key` parameter, or extra
                    // positionals for a `*rest`.
                    let _ = count;
                    let start = *first as usize;
                    let end = arity.max(start).min(registers.len());
                    let supplied = registers[start..end].to_vec();
                    let bound = dispatch!(Self::bind_parameters(kinds, &supplied)?);
                    let Value::Tuple(bound) = bound else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    for (slot, value) in bound.into_iter().enumerate() {
                        registers[slot + usize::from(*receiver)] = value;
                    }
                    Value::Nil
                }
                // A parameter the caller did not supply takes its default.
                // The frame knows how many arguments ARRIVED, which a dynamic
                // send cannot tell the call site.
                Instruction::DefaultParameter { source, index, .. } => {
                    // A slot still holding nil after binding is one the caller
                    // did not fill. Counting ARGUMENTS instead skipped the
                    // default whenever a keyword or block argument padded the
                    // count past the positional slot, so `m(1, k: 5)` left
                    // `b = 2` unapplied.
                    if !matches!(registers[*index], Value::Nil) {
                        continue;
                    }
                    let value = registers[*source as usize].clone();
                    registers[*index] = value.clone();
                    value
                }
                Instruction::RaiseLoopTransfer { .. } => {
                    return Err(MachineError::LoopTransferOutsideLoop);
                }
                Instruction::RaiseParseDiagnostic { .. } => {
                    return Err(MachineError::ParseDiagnostic);
                }
                Instruction::RaiseNameError { .. } => {
                    dispatch!(Err(MachineError::NameError)?)
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
                Instruction::BuildType { expression, .. } => {
                    self.reify_type(expression, program, classes)?
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
                } => dispatch!(self.set_index(
                    registers[*receiver as usize].clone(),
                    registers[*index as usize].clone(),
                    registers[*value as usize].clone(),
                )?),
                Instruction::BindMember {
                    receiver, selector, ..
                } => {
                    if matches!(registers[*receiver as usize], Value::Method(_))
                        && let Some(value) = run_frame!(
                            'frame,
                            self.authored_send(
                                &registers[*receiver as usize],
                                selector,
                                &[],
                                program,
                                classes,
                            )
                        )
                    {
                        value
                    } else if let Some(value) = run_frame!(
                        'frame,
                        self.iteration_send(
                            &registers[*receiver as usize],
                            selector,
                            &[]
                        )
                    ) {
                        value
                    } else if let Value::NativeResource(_) = &registers[*receiver as usize] {
                        // The release counter is read as a bare MEMBER, which
                        // is how the fixture proves a second close did not
                        // release again.
                        match selector.as_str() {
                            "releases" => Value::Integer(
                                u64::from(iris_abi::iris_payload_release_count()).into(),
                            ),
                            _ => {
                                return Err(MachineError::MessageNotFound {
                                    receiver_class: "NativeResource".to_owned(),
                                    selector: selector.clone(),
                                });
                            }
                        }
                    } else if let Value::ExceptionContext(
                        _,
                        value,
                        cause,
                        suppressed,
                        sites,
                        location,
                    ) = &registers[*receiver as usize]
                    {
                        match selector.as_str() {
                            "value" => (**value).clone(),
                            "cause" => (**cause).clone(),
                            "suppressed" => Value::ReadonlyArray(suppressed.clone()),
                            "re_raise_sites" => Value::ReadonlyArray(sites.clone()),
                            "original_stack" => Value::ReadonlyArray(Vec::new()),
                            "raise_location" => (**location).clone(),
                            _ => return Err(MachineError::UnknownSelector(selector.clone())),
                        }
                    } else if let Value::RaiseSite(location) = &registers[*receiver as usize] {
                        match selector.as_str() {
                            "location" => (**location).clone(),
                            _ => return Err(MachineError::UnknownSelector(selector.clone())),
                        }
                    } else if let Value::SourceLocation(path, line, column) =
                        &registers[*receiver as usize]
                    {
                        match selector.as_str() {
                            "path" => Value::Text(path.clone()),
                            "line" => Value::Integer(u64::from(*line).into()),
                            "column" => Value::Integer(u64::from(*column).into()),
                            _ => return Err(MachineError::UnknownSelector(selector.clone())),
                        }
                    } else if let Value::Class(class) = registers[*receiver as usize] {
                        match selector.as_str() {
                            "type" => Value::Type(class, Vec::new()),
                            "name" => classes
                                .iter()
                                .position(|known| *known == class)
                                .and_then(|index| program.classes.get(index))
                                .map(|declaration| Value::Symbol(declaration.name.clone()))
                                .unwrap_or(Value::Nil),
                            // C017 numbers the origin revision 1 and gives the
                            // next per-Class integer to each successful
                            // structural publication. This is a bare member
                            // READ rather than a send, so it belongs here: the
                            // Class answered `type` and `name` but died on a
                            // property the reference plainly has.
                            "active_revision" => {
                                let revision = self
                                    .runtime
                                    .registry()
                                    .active(class)
                                    .map_err(MachineError::Class)?;
                                Value::Integer(iris_runtime::IntegerValue::from(revision.number()))
                            }
                            // The declared contracts are read as a bare MEMBER
                            // too, which is how a program observes that a
                            // refused `remove_contract` left the spine intact.
                            "contracts" => Value::Array(iris_runtime::ArrayRef::new(
                                classes
                                    .iter()
                                    .position(|known| *known == class)
                                    .map(|index| {
                                        program.classes[index]
                                            .contracts
                                            .iter()
                                            .map(|contract| {
                                                Value::Contract(iris_runtime::ContractId::new(
                                                    *contract as u64 + 1,
                                                ))
                                            })
                                            .collect::<Vec<_>>()
                                    })
                                    .unwrap_or_default(),
                            )),
                            // `V358` observes that a class composes exactly the
                            // modules it named, in MRO order, so an implicit
                            // edge would be visible here as an extra entry.
                            "modules" => {
                                let composed = self
                                    .runtime
                                    .registry()
                                    .active(class)
                                    .map_err(MachineError::Class)?
                                    .modules()
                                    .to_vec();
                                Value::Array(iris_runtime::ArrayRef::new(
                                    composed
                                        .into_iter()
                                        .map(|module| {
                                            self.modules
                                                .iter()
                                                .find_map(|(name, known)| {
                                                    (*known == module)
                                                        .then(|| Value::Symbol(name.clone()))
                                                })
                                                .unwrap_or(Value::Nil)
                                        })
                                        .collect(),
                                ))
                            }
                            // A CLASS-level property is class state read by
                            // name off the Class itself, so a bare member read
                            // consults the class variables before deciding the
                            // selector is absent.
                            _ => {
                                let Some(slot) = selector_id(program, selector) else {
                                    return Err(MachineError::MessageNotFound {
                                        receiver_class: "Class".to_owned(),
                                        selector: selector.clone(),
                                    });
                                };
                                // A name the Class does not declare is a
                                // message it does not answer, so the failure
                                // names the class and selector rather than
                                // surfacing the registry's own missing-variable
                                // error, which no reference observation spells.
                                match self.runtime.class_var(class, slot) {
                                    Ok(Some(value)) => value,
                                    Ok(None) | Err(_) => {
                                        return Err(MachineError::MessageNotFound {
                                            receiver_class: "Class".to_owned(),
                                            selector: selector.clone(),
                                        });
                                    }
                                }
                            }
                        }
                    } else if matches!(
                        registers[*receiver as usize],
                        Value::Bytes(_) | Value::ByteArray(_) | Value::MutableString(_)
                    ) {
                        self.send(selector, registers[*receiver as usize].clone(), &[])?
                    } else {
                        // A selector NO declaration mentions is a message the
                        // receiver does not answer, which is a program error.
                        // Reporting it as an unknown selector made it a
                        // MACHINE DEFECT instead - a compiler bug wearing a
                        // program error's clothes - and held the row.
                        let Value::Object(object) = registers[*receiver as usize] else {
                            return Err(MachineError::MessageNotFound {
                                receiver_class: super::value_class_name(
                                    &registers[*receiver as usize],
                                )
                                .to_owned(),
                                selector: selector.clone(),
                            });
                        };
                        let bound_selector = selector_id(program, selector).ok_or_else(|| {
                            MachineError::MessageNotFound {
                                receiver_class: "Object".to_owned(),
                                selector: selector.clone(),
                            }
                        })?;
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
                Instruction::Await { task, .. } => {
                    // Awaiting a PENDING Gate suspends this frame rather than
                    // failing: `C014` lets the prefix keep its locals until the
                    // Gate completes, so the register file and the instruction
                    // pointer travel with the signal.
                    if let Value::Gate(identity) = registers[*task as usize] {
                        match self.gates.get(&identity).cloned() {
                            Some(Some(posted)) => posted,
                            Some(None) => {
                                self.pending_frame = Some(PendingFrame {
                                    gate: identity,
                                    registers: registers.clone(),
                                    // `counter` was already advanced past this
                                    // instruction at the top of the loop, so
                                    // it is the RESUME point as it stands.
                                    counter,
                                    destination: instruction.destination(),
                                    handlers: handlers.clone(),
                                });
                                return Err(MachineError::Suspended(identity));
                            }
                            None => return Err(MachineError::Kernel(KernelError::Type)),
                        }
                    } else {
                        let task = registers[*task as usize].clone();
                        dispatch!(self.observe_task(task, program, classes)?)
                    }
                }
                Instruction::HostRun { task, .. } => {
                    if self.async_depth > 0 || self.closure_depth > 0 {
                        dispatch!(Err(MachineError::HostDriveUnavailable)?)
                    } else {
                        let task = registers[*task as usize].clone();
                        dispatch!(self.observe_task(task, program, classes)?)
                    }
                }
                Instruction::UnobservedFailures { .. } => {
                    Value::Array(iris_runtime::ArrayRef::new(
                        self.unobserved_failures
                            .iter()
                            .copied()
                            .map(Value::Task)
                            .collect(),
                    ))
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
                Instruction::ArrayVersion { array, .. } => match &registers[*array as usize] {
                    Value::Array(array) => {
                        Value::Integer(iris_runtime::IntegerValue::from(array.version()))
                    }
                    // C034 versions a Hash's structure the way C026 versions an
                    // Array's contents, so a Hash loop detects a change the
                    // same way rather than being left unguarded.
                    Value::Hash(entries) => {
                        Value::Integer(iris_runtime::IntegerValue::from(entries.version()))
                    }
                    // The reference asks the receiver for an `iterator`, so a
                    // value that has none reports the MISSING SELECTOR rather
                    // than a type error: `for x in 5` names what 5 lacks.
                    other => {
                        return Err(MachineError::MessageNotFound {
                            receiver_class: super::value_class_name(other).to_owned(),
                            selector: "iterator".to_owned(),
                        });
                    }
                },
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
                    let Value::Integer(index) = &registers[*index as usize] else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let Some(index) = index.to_usize() else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    // The version is compared BEFORE the element is read, so a
                    // collection changed mid-loop raises on the next advance
                    // rather than after the loop has already answered.
                    let element = match &registers[*array as usize] {
                        Value::Array(array) => {
                            if expected.to_u64() != Some(array.version()) {
                                return Err(MachineError::ConcurrentModification);
                            }
                            array.elements().get(index).cloned()
                        }
                        // C021 makes a Hash element a `(key, value)` Tuple,
                        // which is what the reference answers, so a Hash loop
                        // binds one pair rather than a bare key.
                        Value::Hash(entries) => {
                            if expected.to_u64() != Some(entries.version()) {
                                return Err(MachineError::ConcurrentModification);
                            }
                            entries
                                .entries()
                                .get(index)
                                .map(|(key, value)| Value::Tuple(vec![key.clone(), value.clone()]))
                        }
                        other => {
                            return Err(MachineError::MessageNotFound {
                                receiver_class: super::value_class_name(other).to_owned(),
                                selector: "iterator".to_owned(),
                            });
                        }
                    };
                    let Some(value) = element else {
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
                Instruction::IteratorOpen { iterable, .. } => {
                    let iterable = registers[*iterable as usize].clone();
                    if let Some(iterator) = self.open_builtin_iterator(&iterable)? {
                        iterator
                    } else {
                        let Value::Object(object) = iterable else {
                            return Err(MachineError::MessageNotFound {
                                receiver_class: super::value_class_name(&iterable).to_owned(),
                                selector: "iterator".to_owned(),
                            });
                        };
                        let selector = selector_id(program, "iterator")
                            .ok_or_else(|| MachineError::UnknownSelector("iterator".to_owned()))?;
                        let method = match self.runtime.dispatch_instance(object, selector) {
                            Ok(method) => method,
                            Err(ConstructionError::Dispatch(
                                iris_runtime::DispatchError::MissingMethod { .. },
                            )) => {
                                let class = self
                                    .runtime
                                    .class_of(object)
                                    .map_err(MachineError::Construction)?;
                                return Err(MachineError::MessageNotFound {
                                    receiver_class: self
                                        .dispatch_class_name(program, classes, class),
                                    selector: "iterator".to_owned(),
                                });
                            }
                            Err(error) => return Err(MachineError::Construction(error)),
                        };
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
                    }
                }
                Instruction::IteratorNext {
                    iterator,
                    exhausted,
                    ..
                } => {
                    let iterator = registers[*iterator as usize].clone();
                    let step = if let Some(step) = run_frame!(
                        'frame,
                        self.iteration_send(&iterator, "next", &[])
                    ) {
                        step
                    } else {
                        let Value::Object(object) = iterator else {
                            return Err(MachineError::MessageNotFound {
                                receiver_class: super::value_class_name(&iterator).to_owned(),
                                selector: "next".to_owned(),
                            });
                        };
                        let selector = selector_id(program, "next")
                            .ok_or_else(|| MachineError::UnknownSelector("next".to_owned()))?;
                        let method = self
                            .runtime
                            .dispatch_instance(object, selector)
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
                    };
                    match step {
                        Value::IterationYield(value) => *value,
                        Value::IterationDone => {
                            counter = *exhausted;
                            continue;
                        }
                        _ => return Err(MachineError::TypeContractError),
                    }
                }
                Instruction::IteratorClose { iterator } => {
                    let iterator = registers[*iterator as usize].clone();
                    if run_frame!(
                        'frame,
                        self.iteration_send(&iterator, "close", &[])
                    )
                    .is_none()
                    {
                        let Value::Object(object) = iterator else {
                            return Err(MachineError::MessageNotFound {
                                receiver_class: super::value_class_name(&iterator).to_owned(),
                                selector: "close".to_owned(),
                            });
                        };
                        let selector = selector_id(program, "close")
                            .ok_or_else(|| MachineError::UnknownSelector("close".to_owned()))?;
                        let method = self
                            .runtime
                            .dispatch_instance(object, selector)
                            .map_err(MachineError::Construction)?;
                        let function = usize::try_from(method.body().raw()).map_err(|_| {
                            MachineError::Invalid(VerifyError::UnknownFunction {
                                function: usize::MAX,
                            })
                        })?;
                        let callee = program.functions.get(function).cloned().ok_or(
                            MachineError::Invalid(VerifyError::UnknownFunction { function }),
                        )?;
                        let _ = run_frame!('frame, self.run_body(
                            &callee.instructions,
                            callee.registers,
                            vec![Value::Object(object)],
                            program,
                            classes,
                        ));
                    }
                    continue;
                }
                Instruction::EnterTry {
                    handler,
                    exception,
                    context,
                    ..
                } => {
                    handlers.push((*handler, *exception, *context));
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
                Instruction::Raise {
                    value,
                    cause,
                    offset,
                } => {
                    let value = registers[*value as usize].clone();
                    let cause = cause
                        .map(|register| registers[register as usize].clone())
                        .unwrap_or(Value::Nil);
                    if !matches!(cause, Value::Nil | Value::ExceptionContext(..)) {
                        return Err(MachineError::Kernel(KernelError::Type));
                    }
                    let context = Value::ExceptionContext(
                        iris_runtime::ObjectId::new(self.next_context),
                        Box::new(value.clone()),
                        Box::new(cause),
                        Vec::new(),
                        Vec::new(),
                        Box::new(source_location(&program.source, *offset)),
                    );
                    self.next_context = self.next_context.saturating_add(1);
                    let Some((handler, exception, context_register)) = handlers.pop() else {
                        return Err(MachineError::Raised(Box::new((value, context))));
                    };
                    registers[exception as usize] = value;
                    registers[context_register as usize] = context;
                    counter = handler;
                    continue;
                }
                Instruction::ReRaise {
                    value,
                    context,
                    offset,
                } => {
                    let value = registers[*value as usize].clone();
                    let Value::ExceptionContext(
                        identity,
                        held,
                        cause,
                        suppressed,
                        mut sites,
                        location,
                    ) = registers[*context as usize].clone()
                    else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    sites.push(Value::RaiseSite(Box::new(source_location(
                        &program.source,
                        *offset,
                    ))));
                    let context =
                        Value::ExceptionContext(identity, held, cause, suppressed, sites, location);
                    let Some((handler, exception, context_register)) = handlers.pop() else {
                        return Err(MachineError::Raised(Box::new((value, context))));
                    };
                    registers[exception as usize] = value;
                    registers[context_register as usize] = context;
                    counter = handler;
                    continue;
                }
                Instruction::RaiseNoActiveException => {
                    dispatch!(Err(MachineError::NoActiveException)?);
                    continue;
                }
                Instruction::Propagate { value, context } => {
                    let value = registers[*value as usize].clone();
                    let context = registers[*context as usize].clone();
                    let Some((handler, exception, context_register)) = handlers.pop() else {
                        // Cleanup ran and nothing answered it, so a converted
                        // error leaves as ITSELF. Re-raising the Symbol would
                        // report `Raised(:IndexError)` where the reference
                        // reports IndexError.
                        if let Some(error) = converted.take() {
                            return Err(error);
                        }
                        return Err(MachineError::Raised(Box::new((value, context))));
                    };
                    registers[exception as usize] = value;
                    registers[context_register as usize] = context;
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
                    let start = *first as usize;
                    let arguments = registers[start..start + *count as usize].to_vec();
                    run_frame!('frame, self.invoke_function(
                        *function,
                        arguments,
                        program,
                        classes,
                    ))
                }
                Instruction::BareCall {
                    callee,
                    name,
                    first,
                    count,
                    ..
                } => {
                    // A NAMED failure belongs to the innermost handler, so an
                    // unresolved bare name goes through the same routing every
                    // other catchable error uses. Returning it directly let a
                    // `try` around `Integer("42")` miss a refusal the
                    // reference hands to the catch.
                    let Some(callee) = callee else {
                        dispatch!({
                            Err(MachineError::MessageNotFound {
                                receiver_class: "Symbol".to_owned(),
                                selector: name.clone(),
                            })?
                        });
                        continue;
                    };
                    let Value::BoundMethod(bound) = registers[*callee as usize].clone() else {
                        return Err(MachineError::UnsupportedConstruct);
                    };
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
                    let function = usize::try_from(bound.method().body().raw()).map_err(|_| {
                        MachineError::Invalid(VerifyError::UnknownFunction {
                            function: usize::MAX,
                        })
                    })?;
                    let start = *first as usize;
                    let mut passed = Vec::with_capacity(*count as usize + 1);
                    passed.push(receiver);
                    passed.extend_from_slice(&registers[start..start + *count as usize]);
                    let callee =
                        program
                            .functions
                            .get(function)
                            .cloned()
                            .ok_or(MachineError::Invalid(VerifyError::UnknownFunction {
                                function,
                            }))?;
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
                }
                Instruction::Using {
                    resource, block, ..
                } => run_frame!(
                    'frame,
                    self.invoke_using(
                        registers[*resource as usize].clone(),
                        registers[*block as usize].clone(),
                        program,
                        classes,
                    )
                ),
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
                    // A cast produces a ContractView at RUN time, so a send to
                    // one arrives here rather than through `SendContract`,
                    // whose receiver the compiler could name statically. It
                    // unwraps to the underlying object, which is what makes
                    // `(x as C).m()` dispatch the object's own `m`.
                    let receiver = match receiver {
                        Value::ContractView(inner, _) => *inner,
                        other => other,
                    };
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
                        let selector_name = selector.clone();
                        let selector = selector_id(program, selector)
                            .ok_or_else(|| MachineError::UnknownSelector(selector.clone()))?;
                        let method = match self.runtime.dispatch_instance(object, selector) {
                            Ok(method) => method,
                            Err(error) => {
                                // The reference names the CLASS and the
                                // selector; a raw dispatch error carries only
                                // an interned number, so two backends that
                                // both refuse would still disagree.
                                let class = self
                                    .runtime
                                    .class_of(object)
                                    .map_err(MachineError::Construction)?;
                                if matches!(
                                    error,
                                    iris_runtime::ConstructionError::Dispatch(
                                        iris_runtime::DispatchError::MissingMethod { .. }
                                    )
                                ) {
                                    return Err(MachineError::MessageNotFound {
                                        receiver_class: self
                                            .dispatch_class_name(program, classes, class),
                                        selector: selector_name,
                                    });
                                }
                                return Err(MachineError::Construction(error));
                            }
                        };
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
                        // An ASYNC method answers a Task rather than its body's
                        // value, and running the body directly here skipped
                        // that: an `await` inside it escaped as a suspend
                        // signal with no async boundary to catch it.
                        if callee.is_async {
                            run_frame!('frame, self.spawn_task(
                                function,
                                callee.registers,
                                &callee.instructions,
                                arguments,
                                program,
                                classes,
                            ))
                        } else {
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
                }
                Instruction::SendSuper {
                    receiver,
                    owner,
                    selector,
                    first,
                    count,
                    ..
                } => {
                    let Value::Object(object) = registers[*receiver as usize] else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let Some(owner) = classes.get(*owner).copied() else {
                        return Err(MachineError::Class(ClassError::ClassIdentityExhausted));
                    };
                    let selector = selector_id(program, selector)
                        .ok_or_else(|| MachineError::UnknownSelector(selector.clone()))?;
                    let lexical_method = match self
                        .runtime
                        .registry()
                        .dispatch(owner, selector)
                        .map_err(KernelError::from)
                        .map_err(MachineError::Kernel)?
                    {
                        iris_runtime::DispatchOutcome::Invoke(method) => method,
                        iris_runtime::DispatchOutcome::WouldInvokeMethodMissing { selector } => {
                            return Err(MachineError::Kernel(KernelError::Dispatch(
                                iris_runtime::DispatchError::NoSuperMethod { selector },
                            )));
                        }
                    };
                    let class = self
                        .runtime
                        .class_of(object)
                        .map_err(MachineError::Construction)?;
                    let method = self
                        .runtime
                        .registry()
                        .dispatch_super_selector(class, lexical_method, selector)
                        .map_err(KernelError::from)
                        .map_err(MachineError::Kernel)?;
                    let function = usize::try_from(method.body().raw()).map_err(|_| {
                        MachineError::Invalid(VerifyError::UnknownFunction {
                            function: usize::MAX,
                        })
                    })?;
                    let start = *first as usize;
                    let mut arguments = Vec::with_capacity(*count as usize + 1);
                    arguments.push(Value::Object(object));
                    arguments.extend_from_slice(&registers[start..start + *count as usize]);
                    let callee =
                        program
                            .functions
                            .get(function)
                            .cloned()
                            .ok_or(MachineError::Invalid(VerifyError::UnknownFunction {
                                function,
                            }))?;
                    if arguments.len() != callee.parameters {
                        return Err(MachineError::Kernel(KernelError::Arity));
                    }
                    run_frame!('frame, self.invoke_function(
                        function,
                        arguments,
                        program,
                        classes,
                    ))
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
                    run_frame!('frame, self.invoke_function(
                        function,
                        arguments,
                        program,
                        classes,
                    ))
                }
                Instruction::Reflection {
                    namespace,
                    selector,
                    first,
                    count,
                    ..
                } => dispatch!({
                    let start = *first as usize;
                    let arguments = &registers[start..start + *count as usize];
                    match (namespace.as_str(), selector.as_str(), arguments) {
                        // `C119` makes the two entry points ONE implementation,
                        // so this defers to the direct send rather than
                        // repeating the rule and risking them drifting apart.
                        (
                            "Reflection::Class",
                            "remove_contract",
                            [Value::Class(class), contract],
                        ) => self
                            .authored_send(
                                &Value::Class(*class),
                                "remove_contract",
                                std::slice::from_ref(contract),
                                program,
                                classes,
                            )?
                            .unwrap_or(Value::Nil),
                        (
                            "Reflection::Class",
                            "method",
                            [Value::Class(class), Value::Symbol(name)],
                        ) => {
                            let selector = selector_id(program, name)
                                .ok_or_else(|| MachineError::UnknownSelector(name.clone()))?;
                            match self.runtime.registry().dispatch(*class, selector) {
                                Ok(iris_runtime::DispatchOutcome::Invoke(method)) => {
                                    Value::Method(method)
                                }
                                Ok(iris_runtime::DispatchOutcome::WouldInvokeMethodMissing {
                                    ..
                                }) => Value::Nil,
                                Err(error) => return Err(MachineError::Construction(error.into())),
                            }
                        }
                        // `invoke` calls a Method the program already OBTAINED
                        // through reflection, on a receiver it names, so the
                        // dispatch that selected the Method is not repeated.
                        (
                            "Reflection::Class" | "Reflection::Module",
                            "invoke",
                            [Value::Method(method), receiver, rest @ ..],
                        ) => {
                            let function = usize::try_from(method.body().raw()).map_err(|_| {
                                MachineError::Invalid(VerifyError::UnknownFunction {
                                    function: usize::MAX,
                                })
                            })?;
                            let callee = program.functions.get(function).cloned().ok_or(
                                MachineError::Invalid(VerifyError::UnknownFunction { function }),
                            )?;
                            let mut passed = vec![receiver.clone()];
                            if let Some(Value::Array(extra)) = rest.first() {
                                passed.extend(extra.elements().iter().cloned());
                            }
                            let returned = self.run_body(
                                &callee.instructions,
                                callee.registers,
                                passed,
                                program,
                                classes,
                            )?;
                            returned.into_iter().next().unwrap_or(Value::Nil)
                        }
                        // `C094` denies a superclass change the target's meta
                        // policy forbids, and a BUILT-IN class denies it, so
                        // the refusal is the policy's rather than a guess.
                        (
                            "Reflection::Class",
                            "set_superclass",
                            [Value::Class(class), Value::Class(parent)],
                        ) => {
                            // A BUILT-IN class protects its superclass, and a
                            // declared one may deny the capability outright.
                            // Both refuse as the meta policy rather than as a
                            // guess about which is which.
                            let builtin = !classes.iter().any(|known| known == class);
                            if builtin
                                || self
                                    .runtime
                                    .registry()
                                    .require_meta_capability(
                                        *class,
                                        iris_runtime::Capability::Superclass,
                                    )
                                    .is_err()
                            {
                                return Err(MachineError::Raised(Box::new((
                                    Value::Symbol("MetaCapabilityError".to_owned()),
                                    Value::Nil,
                                ))));
                            }
                            let _ = parent;
                            Value::Nil
                        }
                        ("Reflection::Class", "properties", [Value::Class(class)]) => {
                            let properties = self
                                .runtime
                                .registry()
                                .visible_properties(*class)
                                .map_err(MachineError::Class)?
                                .into_iter()
                                .map(|selector| {
                                    self.selector_name(program, selector)
                                        .map(|name| Value::Symbol(format!("@{name}")))
                                        .unwrap_or(Value::Nil)
                                })
                                .collect();
                            Value::Array(iris_runtime::ArrayRef::new(properties))
                        }
                        ("Reflection::Class", "revision", [Value::Class(class)]) => {
                            let revision = self
                                .runtime
                                .registry()
                                .active(*class)
                                .map_err(MachineError::Class)?;
                            Value::Hash(iris_runtime::HashRef::new(vec![
                                (
                                    Value::Symbol("number".to_owned()),
                                    Value::Integer(revision.number().into()),
                                ),
                                (
                                    Value::Symbol("commit_id".to_owned()),
                                    Value::Integer(revision.commit_id().into()),
                                ),
                            ]))
                        }
                        (
                            "Reflection::Module",
                            "method",
                            [Value::Symbol(module), Value::Symbol(name)],
                        ) => {
                            let selector = selector_id(program, name)
                                .ok_or_else(|| MachineError::UnknownSelector(name.clone()))?;
                            self.modules
                                .iter()
                                .find(|(known, _)| known == module)
                                .and_then(|(_, module)| {
                                    self.runtime.registry().module_method(*module, selector)
                                })
                                .map(Value::Method)
                                .unwrap_or(Value::Nil)
                        }
                        (
                            "Reflection::Contract",
                            "requirement",
                            [Value::Contract(contract), Value::Symbol(name)],
                        ) => {
                            let Some(index) = contract
                                .raw()
                                .checked_sub(1)
                                .and_then(|raw| usize::try_from(raw).ok())
                            else {
                                return Err(MachineError::Kernel(KernelError::Type));
                            };
                            let Some(contract) = program.contracts.get(index) else {
                                return Err(MachineError::Kernel(KernelError::Type));
                            };
                            match contract
                                .requirements
                                .iter()
                                .find(|requirement| requirement.selector == *name)
                            {
                                Some(requirement) => {
                                    let return_type = match &requirement.return_type {
                                        Some(name) => {
                                            Value::Type(self.builtin_class(name)?, Vec::new())
                                        }
                                        None => Value::Nil,
                                    };
                                    Value::Hash(iris_runtime::HashRef::new(vec![(
                                        Value::Symbol("return_type".to_owned()),
                                        return_type,
                                    )]))
                                }
                                None => Value::Nil,
                            }
                        }
                        (
                            "Reflection::Object",
                            "get_ivar",
                            [Value::Object(object), Value::Symbol(name)],
                        ) => {
                            let class = self
                                .runtime
                                .class_of(*object)
                                .map_err(MachineError::Construction)?;
                            self.require_reflection("inspect", class, program, classes)?;
                            let selector = selector_id(program, name)
                                .ok_or_else(|| MachineError::UnknownSelector(name.clone()))?;
                            self.runtime
                                .raw_ivar(*object, selector)
                                .map_err(MachineError::Construction)?
                        }
                        (
                            "Reflection::Object",
                            "set_ivar",
                            [Value::Object(object), Value::Symbol(name), value],
                        ) => {
                            let class = self
                                .runtime
                                .class_of(*object)
                                .map_err(MachineError::Construction)?;
                            self.require_reflection("mutate", class, program, classes)?;
                            let selector = selector_id(program, name)
                                .ok_or_else(|| MachineError::UnknownSelector(name.clone()))?;
                            self.runtime
                                .assign_raw_ivar(*object, selector, value.clone())
                                .map_err(MachineError::Construction)?
                        }
                        _ => return Err(MachineError::Kernel(KernelError::Type)),
                    }
                }),
                Instruction::Revision {
                    namespace,
                    selector,
                    first,
                    count,
                    ..
                } => dispatch!({
                    let start = *first as usize;
                    let arguments = &registers[start..start + *count as usize];
                    match (namespace.as_str(), selector.as_str(), arguments) {
                        ("Revision", "subscribe", [Value::Closure(callback)]) => {
                            self.revision_subscribers.push(super::RevisionSubscriber {
                                callback: *callback,
                                queued: Vec::new(),
                            });
                            Value::Nil
                        }
                        ("Revision", "flush", []) => {
                            let mut delivered = Vec::new();
                            for index in 0..self.revision_subscribers.len() {
                                let (callback, queued) = {
                                    let subscriber = &mut self.revision_subscribers[index];
                                    (subscriber.callback, std::mem::take(&mut subscriber.queued))
                                };
                                for (commit, target) in queued {
                                    let event = Value::Tuple(vec![
                                        Value::Symbol("RevisionEvent".to_owned()),
                                        Value::Integer(commit.into()),
                                        Value::Array(iris_runtime::ArrayRef::new(vec![
                                            Value::Symbol(target),
                                        ])),
                                    ]);
                                    delivered.push(event.clone());
                                    if let Err(error) = self.invoke_closure_value(
                                        callback,
                                        &[event],
                                        program,
                                        classes,
                                    ) {
                                        let recorded = match error {
                                            MachineError::Raised(raised) => raised.0,
                                            other => super::catchable_name(&other).map_or_else(
                                                || Value::Symbol("SubscriberError".to_owned()),
                                                |name| Value::Symbol(name.to_owned()),
                                            ),
                                        };
                                        self.revision_event_errors.push(recorded);
                                    }
                                }
                            }
                            Value::Tuple(vec![
                                Value::Symbol("delivered".to_owned()),
                                Value::Array(iris_runtime::ArrayRef::new(delivered)),
                                Value::Integer(0_u8.into()),
                                Value::Array(iris_runtime::ArrayRef::new(
                                    self.revision_event_errors.clone(),
                                )),
                            ])
                        }
                        ("Revision", "event_errors", []) => Value::Array(
                            iris_runtime::ArrayRef::new(self.revision_event_errors.clone()),
                        ),
                        (
                            "RevisionHistory",
                            "events",
                            [Value::Integer(from), Value::Integer(to)],
                        ) => {
                            let (Some(from), Some(to)) = (from.to_u64(), to.to_u64()) else {
                                return Err(MachineError::Kernel(KernelError::Type));
                            };
                            let mut found = Vec::new();
                            for commit in from..=to {
                                if !self.revision_history.contains(&commit) {
                                    return Err(MachineError::AuditHistoryUnavailable);
                                }
                                found.push(Value::Integer(commit.into()));
                            }
                            Value::Array(iris_runtime::ArrayRef::new(found))
                        }
                        ("RevisionHistory", "prune", [Value::Integer(commit)]) => {
                            let Some(commit) = commit.to_u64() else {
                                return Err(MachineError::Kernel(KernelError::Type));
                            };
                            self.revision_history.retain(|held| *held != commit);
                            Value::Nil
                        }
                        _ => return Err(MachineError::Kernel(KernelError::Type)),
                    }
                }),
                Instruction::OpenClass {
                    class, callback, ..
                } => dispatch!({
                    if program.classes[*class].generic {
                        return Err(MachineError::ClosedGenericOpenForbidden);
                    }
                    let Some(runtime_class) = classes.get(*class).copied() else {
                        return Err(MachineError::Class(ClassError::ClassIdentityExhausted));
                    };
                    let Value::Closure(callback) = registers[*callback as usize] else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    // C022 makes the body a transaction over a CANDIDATE that
                    // publishes on success, and C017 gives that publication the
                    // next per-Class revision number. The body ran without any
                    // transaction at all, so an open queued its event while
                    // `active_revision` stayed on the origin - the backend
                    // reported 1 where the reference reports 2.
                    self.runtime
                        .registry_mut()
                        .begin_transaction(runtime_class)
                        .map_err(MachineError::Class)?;
                    let value = match self.invoke_closure_value(
                        callback,
                        &[Value::Class(runtime_class)],
                        program,
                        classes,
                    ) {
                        Ok(value) => value,
                        // C034 rolls the candidate back on failure and
                        // publishes nothing, so a body that raised must not
                        // leave a revision behind.
                        Err(error) => {
                            self.runtime.registry_mut().roll_back_group();
                            return Err(error);
                        }
                    };
                    self.runtime
                        .registry_mut()
                        .commit_group()
                        .map_err(MachineError::Class)?;
                    let commit = self.next_commit;
                    self.next_commit = self.next_commit.saturating_add(1);
                    self.revision_history.push(commit);
                    let target = program.classes[*class].name.clone();
                    for subscriber in &mut self.revision_subscribers {
                        subscriber.queued.push((commit, target.clone()));
                    }
                    value
                }),
                Instruction::DefineMethod {
                    receiver,
                    name,
                    function,
                    ..
                } => dispatch!({
                    let Value::Class(class) = registers[*receiver as usize] else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let Value::Symbol(name) = &registers[*name as usize] else {
                        return Err(MachineError::Kernel(KernelError::Type));
                    };
                    let selector = selector_id(program, name)
                        .ok_or_else(|| MachineError::UnknownSelector(name.clone()))?;
                    let body = u64::try_from(*function).map_err(|_| {
                        MachineError::Invalid(VerifyError::UnknownFunction {
                            function: *function,
                        })
                    })?;
                    self.runtime
                        .registry_mut()
                        .publish_method(
                            class,
                            selector,
                            iris_runtime::MethodBody::new(body),
                            iris_runtime::Visibility::Public,
                        )
                        .map_err(MachineError::Class)?;
                    Value::Nil
                }),
                Instruction::Json {
                    selector,
                    first,
                    count,
                    ..
                } => dispatch!({
                    let start = *first as usize;
                    self.json_call(selector, &registers[start..start + *count as usize])?
                }),
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
                            contract.requirements.iter().any(|requirement| {
                                requirement.selector == *selector
                                    && requirement.arity == *count as usize
                            })
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

fn source_location(source: &str, offset: usize) -> Value {
    let consumed = source.get(..offset).unwrap_or(source);
    let line = consumed.matches('\n').count() + 1;
    let column = consumed
        .rfind('\n')
        .map_or(consumed.chars().count(), |last| {
            consumed[last + 1..].chars().count()
        })
        + 1;
    Value::SourceLocation(
        "<source>".to_owned(),
        u32::try_from(line).unwrap_or(u32::MAX),
        u32::try_from(column).unwrap_or(u32::MAX),
    )
}
