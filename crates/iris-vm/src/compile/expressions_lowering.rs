//! Expression and closure lowering.

use iris_syntax::{BinaryOperator, Expression, Statement, UnaryOperator};

use super::calls::{binary_selector, compound_selector, construct_name};
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
            // Reading a deferred binding checks, at RUN time, whether an
            // assignment actually ran: the same code answers a value on a path
            // that assigned and fails on one that did not, so the check cannot
            // be settled here.
            Expression::Name(name) => {
                if let Some(binding) = self.lookup_binding(name).cloned() {
                    if let Some(assigned) = binding.assigned {
                        let destination = self.allocate()?;
                        self.instructions.push(Instruction::ReadDeferred {
                            destination,
                            value: binding.register,
                            assigned,
                        });
                        return Ok(destination);
                    }
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
                // A MODULE name is a value: `A.remove_module(Mo)` names the
                // module itself. A module COMPOSED into a class has its
                // methods lowered with a receiver, so requiring a
                // receiverless signature stopped recognising exactly the
                // modules a mixin names - which is every module such a call
                // is about.
                if (self
                    .signatures
                    .iter()
                    .any(|signature| signature.module == name)
                    || self.modules.iter().any(|module| module.name == *name))
                    && self.class_index(name).is_none()
                {
                    self.instructions.push(Instruction::LoadSymbol {
                        destination,
                        name: name.clone(),
                    });
                    return Ok(destination);
                }
                let destination = self.allocate()?;
                self.instructions
                    .push(Instruction::RaiseNameError { destination });
                Ok(destination)
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
            Expression::Await(operand) => {
                let task = self.expression(operand)?;
                let destination = self.allocate()?;
                self.instructions
                    .push(Instruction::Await { destination, task });
                Ok(destination)
            }
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
                    // The branch tests TRUTH but the result keeps the
                    // original value: `a || :rhs` answers `a` itself when `a`
                    // is truthy, not the Bool its `to_bool` produced.
                    let truth = self.truth_test(left)?;
                    let branch = self.instructions.len();
                    self.instructions.push(Instruction::JumpUnless {
                        condition: truth,
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
                // `as?` is a CHECKED cast: it answers the value when the test
                // holds and nil when it does not, rather than failing. It is
                // the type test with a selection on top.
                if *operator == BinaryOperator::AsOptional
                    && !matches!(right.as_ref(),
                        Expression::Name(name) if self.contract_index(name).is_some())
                {
                    let value = self.expression(left)?;
                    let target = self.expression(right)?;
                    let matches = self.allocate()?;
                    self.instructions.push(Instruction::TypeTest {
                        destination: matches,
                        value,
                        target,
                    });
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::Move {
                        destination,
                        source: value,
                    });
                    let branch = self.instructions.len();
                    self.instructions.push(Instruction::JumpUnless {
                        condition: matches,
                        target: 0,
                    });
                    let skip = self.instructions.len();
                    self.instructions.push(Instruction::Jump { target: 0 });
                    let otherwise = self.instructions.len();
                    self.instructions.push(Instruction::LoadNil { destination });
                    let after = self.instructions.len();
                    self.patch(branch, otherwise)?;
                    self.patch(skip, after)?;
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
                // C082 makes `=~`, `!~` and a NAMED infix ordinary sends on
                // the subject, so they dispatch like any other selector. They
                // cannot go through `Binary`, whose selector is a `'static`
                // str, because a named infix carries the name the source
                // wrote.
                if let Some(selector) = match operator {
                    iris_syntax::BinaryOperator::NamedInfix { selector } => Some(selector.clone()),
                    iris_syntax::BinaryOperator::Match => Some("=~".to_owned()),
                    iris_syntax::BinaryOperator::NotMatch
                    | iris_syntax::BinaryOperator::RegexDoesNotMatch => Some("!~".to_owned()),
                    _ => None,
                } {
                    let receiver = self.expression(left)?;
                    let (first, count) = self.argument_window(std::slice::from_ref(right))?;
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::Send {
                        destination,
                        receiver,
                        selector,
                        first,
                        count,
                        caller: self.current_method.as_ref().map(|(owner, _)| *owner),
                        caller_module: self.enclosing_module.clone(),
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
                // `!` is the TRUTH protocol negated rather than a send: it
                // answers a Bool for any operand, including one whose class
                // defines `to_bool`, so it goes through the same test `if`
                // does rather than looking for a `!` method.
                if *operator == UnaryOperator::Not {
                    let truth = self.truth_test(operand)?;
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::NegateTruth {
                        destination,
                        value: truth,
                    });
                    return Ok(destination);
                }
                let destination = self.allocate()?;
                self.instructions.push(Instruction::Unary {
                    destination,
                    selector: match operator {
                        UnaryOperator::Negate => "negate",
                        UnaryOperator::BitwiseNot => "~",
                        UnaryOperator::Not | UnaryOperator::Plus => return Ok(operand),
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
                // A NAME key is an ordinary expression, not a shorthand for a
                // symbol: `%{ first: 1 }` keys the hash by what `first` HOLDS,
                // which is what makes an exception context usable as a key.
                for (key, value) in entries {
                    lowered.push(self.expression(key)?);
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
                        | Expression::Literal(_)
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
                // A CLOSED generic's Type carries its ARGUMENTS: `Box<String>`
                // and `Box<Integer>` are two Types of one class, so reading
                // `type` off the bare class discarded exactly what tells them
                // apart and made them the same value.
                if selector == "type"
                    && let Expression::ClosedGeneric { name, arguments } = receiver.as_ref()
                    && self.class_index(name).is_some()
                {
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::BuildType {
                        destination,
                        expression: iris_syntax::TypeExpression::Generic {
                            name: name.clone(),
                            arguments: arguments.clone(),
                        },
                    });
                    return Ok(destination);
                }
                // `C064` puts a SHARED class property on the UNAPPLIED generic
                // definition, so a closed construction does not reach it:
                // `C.n` answers while `C<String>.n` is a MessageNotFound.
                // Falling through to ordinary dispatch found the definition's
                // own slot and answered a value the language has no slot for.
                if let Expression::ClosedGeneric { name, .. } = receiver.as_ref()
                    && let Some(class) = self.class_index(name)
                    && self.classes[class]
                        .shared_class_variables
                        .iter()
                        .any(|shared| shared == selector)
                {
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::RaiseMessageNotFound {
                        destination,
                        receiver_class: "Class".to_owned(),
                        selector: selector.clone(),
                    });
                    return Ok(destination);
                }
                // A CLOSED generic construction reads its own class-variable
                // slot, so `Cache<String>.value` and `Cache<Integer>.value` do
                // not share one.
                if let Some(selector) = self.construction_selector(receiver, selector) {
                    let receiver = self.expression(receiver)?;
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::BindMember {
                        destination,
                        receiver,
                        selector,
                    });
                    return Ok(destination);
                }
                // A module's `shared class property` is read as a bare MEMBER,
                // `M.first`, and was synthesized into a receiverless reader -
                // so it resolves by index here rather than dispatching, which
                // has no module receiver value to send to.
                if let Expression::Name(module) = receiver.as_ref()
                    && self.lookup(module).is_none()
                    && let Some(function) = self.resolve(module, selector)
                    && self.signatures[function].parameters.is_empty()
                    && !self.signatures[function].receiver
                {
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::Call {
                        destination,
                        function,
                        first: destination,
                        count: 0,
                    });
                    return Ok(destination);
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
                if matches!(name.as_str(), "Never" | "NonNil") {
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::BuildType {
                        destination,
                        expression: iris_syntax::TypeExpression::Name(name.clone()),
                    });
                    return Ok(destination);
                }
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
            Expression::ReifiedType(
                expression @ (iris_syntax::TypeExpression::Union(_)
                | iris_syntax::TypeExpression::Intersection(_)),
            ) => {
                self.validate_composed_type(expression)?;
                let destination = self.allocate()?;
                self.instructions.push(Instruction::BuildType {
                    destination,
                    expression: expression.clone(),
                });
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
                    let selector = self
                        .construction_selector(receiver, selector)
                        .unwrap_or_else(|| selector.clone());
                    let receiver = self.expression(receiver)?;
                    let (first, count) = self.argument_window(std::slice::from_ref(right))?;
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::Send {
                        destination,
                        receiver,
                        selector: format!("{selector}="),
                        first,
                        count,
                        caller: self.current_method.as_ref().map(|(owner, _)| *owner),
                        caller_module: self.enclosing_module.clone(),
                    });
                    return Ok(destination);
                }
                let Expression::Name(name) = left.as_ref() else {
                    // A left side that names no assignable place is a program
                    // error the reference raises when the assignment RUNS, so
                    // the backend answers a program that raises it. Declining
                    // made both backends refuse the same program while
                    // describing it differently, which holds the row.
                    let destination = self.allocate()?;
                    self.instructions
                        .push(Instruction::RaiseUnsupported { destination });
                    return Ok(destination);
                };
                let Some(binding) = self.lookup_binding(name).cloned() else {
                    let Some(binding) = self
                        .program_bindings
                        .iter()
                        .rev()
                        .find(|binding| binding.name == *name)
                    else {
                        // A CLASS or CONTRACT name is a binding the
                        // declaration made, so writing it is an IMMUTABLE
                        // write rather than a missing name. A MODULE name is
                        // not a binding at all, so it stays a NameError.
                        let destination = self.allocate()?;
                        if self.class_index(name).is_some() || self.contract_index(name).is_some() {
                            self.instructions
                                .push(Instruction::RaiseImmutableBinding { destination });
                            return Ok(destination);
                        }
                        // `C009` makes a bare `name = expr` never CREATE a
                        // binding, so an absent target is the reference's own
                        // NameError when the assignment runs rather than a
                        // construct the backend lacks.
                        self.instructions
                            .push(Instruction::RaiseNameError { destination });
                        return Ok(destination);
                    };
                    if !binding.shared {
                        // The reference RAISES when the write runs rather than
                        // refusing the program, so the backend answers a
                        // program that raises it.
                        let destination = self.allocate()?;
                        self.instructions
                            .push(Instruction::RaiseImmutableBinding { destination });
                        return Ok(destination);
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
                // `C009` makes `let`, a parameter and a loop variable
                // IMMUTABLE, so only a `mut` binding may be written. A
                // DEFERRED binding is the exception: it is declared without a
                // value and its first write is what supplies one.
                if !binding.shared && binding.assigned.is_none() {
                    let destination = self.allocate()?;
                    self.instructions
                        .push(Instruction::RaiseImmutableBinding { destination });
                    return Ok(destination);
                }
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
                if let Some(assigned) = binding.assigned {
                    self.instructions
                        .push(Instruction::MarkAssigned { assigned });
                }
                Ok(if binding.shared { source } else { destination })
            }
            // `IRIS-V1-CONTROL-C037`: a LOGICAL assignment reads the target
            // once, truth-tests it, and evaluates the right side only on the
            // writing path. Lowering it as `x = x || v` would evaluate the
            // right side unconditionally, so `x &&= log.append(:ran)` would
            // append even when it must not.
            Expression::Assignment {
                left,
                operator:
                    operator @ (iris_syntax::AssignmentOperator::LogicalAnd
                    | iris_syntax::AssignmentOperator::LogicalOr),
                right,
            } => {
                let Expression::Name(name) = left.as_ref() else {
                    return Err(CompileError::new("assignment target"));
                };
                let Some(binding) = self.lookup_binding(name).cloned() else {
                    return Err(CompileError::new("name assignment unbound"));
                };
                if binding.assigned.is_some() {
                    return Err(CompileError::new("assignment"));
                }
                // Every `mut` binding is a shared CELL, so the target is read
                // and written through the cell rather than as a register.
                let current = self.read_binding(&binding)?;
                let destination = self.allocate()?;
                self.instructions.push(Instruction::Move {
                    destination,
                    source: current,
                });
                let truth = self.truth_test(current)?;
                let branch = self.instructions.len();
                self.instructions.push(Instruction::JumpUnless {
                    condition: truth,
                    target: 0,
                });
                // `&&=` writes on the TRUTHY path and `||=` on the falsey one,
                // so they differ only in which edge the write sits on.
                let writes_when_truthy = *operator == iris_syntax::AssignmentOperator::LogicalAnd;
                let skip = self.instructions.len();
                self.instructions.push(Instruction::Jump { target: 0 });
                let otherwise = self.instructions.len();
                let value = self.expression(right)?;
                self.write_binding(&binding, value)?;
                self.instructions.push(Instruction::Move {
                    destination,
                    source: value,
                });
                let after = self.instructions.len();
                if writes_when_truthy {
                    self.patch(branch, after)?;
                    self.patch(skip, otherwise)?;
                } else {
                    self.patch(branch, otherwise)?;
                    self.patch(skip, after)?;
                }
                Ok(destination)
            }
            // `IRIS-V1-CONTROL-C036` reads the target ONCE and sends the
            // ordinary operator to the read value, which is exactly what
            // `x = x op v` does for a NAME target: the name is a register, so
            // reading it twice evaluates nothing twice.
            Expression::Assignment {
                left,
                operator,
                right,
            } if compound_selector(*operator).is_some() => {
                let Some(selector) = compound_selector(*operator) else {
                    return Err(CompileError::new("assignment"));
                };
                // `IRIS-V1-CONTROL-C036` reads the target ONCE, which for an
                // indexed target means the receiver and the index are each
                // evaluated once and reused for both the read and the write:
                // `p.factory()[p.idx()] += p.rhs()` calls each of them once,
                // in that order.
                if let Expression::Index { receiver, index } = left.as_ref() {
                    let receiver = self.expression(receiver)?;
                    let index = self.expression(index)?;
                    let right = self.expression(right)?;
                    let current = self.allocate()?;
                    self.instructions.push(Instruction::Index {
                        destination: current,
                        receiver,
                        index,
                    });
                    let combined = self.allocate()?;
                    self.instructions.push(Instruction::Binary {
                        destination: combined,
                        selector,
                        left: current,
                        right,
                    });
                    let destination = self.allocate()?;
                    self.instructions.push(Instruction::SetIndex {
                        destination,
                        receiver,
                        index,
                        value: combined,
                    });
                    return Ok(destination);
                }
                let Expression::Name(name) = left.as_ref() else {
                    return Err(CompileError::new("assignment target"));
                };
                let Some(binding) = self.lookup_binding(name).cloned() else {
                    return Err(CompileError::new("name assignment unbound"));
                };
                if binding.assigned.is_some() {
                    return Err(CompileError::new("assignment"));
                }
                let right = self.expression(right)?;
                let current = self.read_binding(&binding)?;
                let combined = self.allocate()?;
                self.instructions.push(Instruction::Binary {
                    destination: combined,
                    selector,
                    left: current,
                    right,
                });
                self.write_binding(&binding, combined)?;
                Ok(combined)
            }
            // An `if` is an EXPRESSION as well as a statement, so it lowers
            // identically in either position: `let a = if c { 1 }` needs the
            // same both-arms-write-one-destination shape the statement form
            // already had.
            Expression::If {
                condition,
                then_body,
                else_body,
            } => self.if_value(condition, then_body, else_body.as_deref()),
            // A `while` is an EXPRESSION as well as a statement, and answers
            // the operand a `break` carried: `let b = while true { break 7 }`.
            Expression::While {
                label,
                condition,
                body,
            } => self.while_value(label.as_deref(), condition, body),
            Expression::KeywordArgument { name, value } => {
                let value = self.expression(value)?;
                let destination = self.allocate()?;
                self.instructions.push(Instruction::MakeKeywordArgument {
                    destination,
                    name: name.clone(),
                    value,
                });
                Ok(destination)
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
        let declarations = self.declarations();
        let mut lowering = Lowering::new(
            declarations,
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
            fixed_arity: None,
            parameter_types: vec!["Dynamic<Object>".to_owned(); parameters.len()],
            return_type: "Dynamic<Object>".to_owned(),
            is_async: false,
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
        let declarations = self.declarations();
        let mut lowering = Lowering::new(
            declarations,
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
            fixed_arity: None,
            parameter_types: vec!["Dynamic<Object>".to_owned(); parameters.len()],
            return_type: "Dynamic<Object>".to_owned(),
            is_async: false,
            registers,
            instructions,
        });
        Ok(function)
    }
}
