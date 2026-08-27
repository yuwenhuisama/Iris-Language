//! Function, statement, expression, call, and closure lowering.

use super::declarations::Signature;
use super::{Class, CompileError, Contract, Function, Instruction, ParameterKind, Register};

/// Lowers one function into its own frame.
pub(super) fn lower_function(
    signature: &Signature<'_>,
    signatures: &[Signature<'_>],
    classes: &[Class],
    contracts: &[Contract],
    declared_functions: usize,
    closures: &mut Vec<Function>,
    program_bindings: &[ProgramBinding],
) -> Result<Function, CompileError> {
    let mut lowering = Lowering::new(
        signatures,
        classes,
        contracts,
        declared_functions,
        closures,
        program_bindings,
        false,
    );
    lowering.current_method = classes
        .iter()
        .position(|class| class.name == signature.module)
        .map(|owner| (owner, signature.selector.to_owned()));
    lowering.async_body = signature.is_async;
    if signature.receiver {
        let receiver = lowering.allocate()?;
        lowering
            .names
            .push(Binding::value("self".to_owned(), receiver));
    }
    // Parameters occupy the LEADING registers, so a call can copy arguments
    // into a fresh frame without the callee knowing where they came from.
    // Nothing may be allocated before them: binding the owner's constants
    // first displaced every parameter, and the verifier then proved the
    // argument registers unwritten - a compiler defect that reached any
    // module function taking both a constant and a parameter.
    for parameter in &signature.parameters {
        let register = lowering.allocate()?;
        lowering
            .names
            .push(Binding::value(parameter.name.clone(), register));
    }
    // A parameter's DEFAULT is filled in the callee, after the registers are
    // reserved, because a dynamic send cannot substitute it at the call site:
    // it does not know the signature until dispatch. Filling it here made
    // `f(a, b: Integer = 2)` answer `2` for a method send as well as for a
    // resolved call, where it had silently answered nil.
    let defaults: Vec<(usize, &iris_syntax::Expression)> = signature
        .parameters
        .iter()
        .enumerate()
        .filter_map(|(index, parameter)| {
            parameter
                .default
                .as_ref()
                .map(|default| (index + usize::from(signature.receiver), default))
        })
        .collect();
    for (index, default) in defaults {
        let source = lowering.expression(default)?;
        let destination = lowering.allocate()?;
        lowering.instructions.push(Instruction::DefaultParameter {
            destination,
            source,
            index,
        });
    }
    // A signature with a NON-positional category binds its own parameters, so
    // the frame reads the argument window itself. A purely positional one
    // needs no instruction at all: the caller already wrote the values into
    // the leading registers.
    if signature
        .parameters
        .iter()
        .any(|parameter| parameter.category != iris_syntax::ParameterCategory::Positional)
    {
        let kinds = signature
            .parameters
            .iter()
            .map(|parameter| {
                (
                    match parameter.category {
                        iris_syntax::ParameterCategory::Rest => ParameterKind::Rest,
                        iris_syntax::ParameterCategory::Keyword => ParameterKind::Keyword,
                        iris_syntax::ParameterCategory::KeywordRest => ParameterKind::KeywordRest,
                        iris_syntax::ParameterCategory::Block => ParameterKind::Block,
                        iris_syntax::ParameterCategory::Positional => ParameterKind::Positional,
                    },
                    parameter.name.clone(),
                )
            })
            .collect();
        let first = u16::try_from(usize::from(signature.receiver))
            .map_err(|_| CompileError::new("call too wide"))?;
        let count = u16::try_from(signature.parameters.len())
            .map_err(|_| CompileError::new("call too wide"))?;
        let destination = lowering.allocate()?;
        lowering.instructions.push(Instruction::BindParameters {
            destination,
            kinds,
            receiver: signature.receiver,
            first,
            count,
        });
    }
    // The owner's constants are bound after them, and `lookup` searches in
    // REVERSE, so a parameter of the same name would lose to the constant.
    // A constant whose name a parameter already claims is therefore skipped,
    // which keeps `fun f(K) { K }` reading its parameter.
    let constants = signature.constants.clone();
    for (name, value) in &constants {
        if signature
            .parameters
            .iter()
            .any(|parameter| parameter.name == *name)
        {
            continue;
        }
        let register = lowering.expression(value)?;
        lowering
            .names
            .push(Binding::value((*name).to_owned(), register));
    }
    // A synthesized stored-property initializer carries an EXPRESSION rather
    // than a block, and answers it directly.
    if let Some(expression) = signature.expression_body {
        let value = lowering.expression(expression)?;
        lowering.instructions.push(Instruction::Return { value });
        return Ok(Function {
            name: format!("{}.{}", signature.module, signature.selector),
            parameters: usize::from(signature.receiver),
            captures: 0,
            parameter_types: Vec::new(),
            return_type: String::new(),
            is_async: false,
            registers: lowering.next_register as usize,
            instructions: lowering.instructions,
        });
    }
    // An EMPTY body answers nil rather than being refused: `fun f() { }` is a
    // method that returns nil, not a method the backend cannot express.
    let value = match signature.body.split_last() {
        Some((last, leading)) => {
            for statement in leading {
                lowering.statement(statement)?;
            }
            // A body's LAST expression is its value, which an explicit
            // `Return` makes uniform: every path out of a frame goes through
            // one instruction.
            lowering.statement(last)?
        }
        None => {
            let destination = lowering.allocate()?;
            lowering
                .instructions
                .push(Instruction::LoadNil { destination });
            destination
        }
    };
    lowering.instructions.push(Instruction::Return { value });
    Ok(Function {
        name: format!("{}.{}", signature.module, signature.selector),
        parameters: signature.parameters.len() + usize::from(signature.receiver),
        captures: 0,
        parameter_types: signature
            .parameters
            .iter()
            .map(|parameter| reflected_type(parameter.annotation.as_ref()))
            .collect(),
        return_type: reflected_type(signature.return_type),
        is_async: signature.is_async,
        registers: lowering.next_register as usize,
        instructions: lowering.instructions,
    })
}

fn reflected_type(annotation: Option<&iris_syntax::TypeExpression>) -> String {
    match annotation {
        Some(iris_syntax::TypeExpression::Name(name)) => name.clone(),
        _ => "Dynamic<Object>".to_owned(),
    }
}

pub(super) struct Lowering<'a, 'b> {
    pub(super) instructions: Vec<Instruction>,
    pub(super) next_register: Register,
    /// Names bound so far, each pinned to the register holding its value.
    pub(super) names: Vec<Binding>,
    /// Functions callable from this frame, resolved before lowering.
    pub(super) signatures: &'a [Signature<'b>],
    pub(super) classes: &'a [Class],
    pub(super) contracts: &'a [Contract],
    pub(super) declared_functions: usize,
    pub(super) closures: &'a mut Vec<Function>,
    pub(super) loops: Vec<LoopContext>,
    pub(super) exception_contexts: Vec<(Register, Register)>,
    pub(super) method_values: Vec<Register>,
    pub(super) program_bindings: &'a [ProgramBinding],
    pub(super) top_level: bool,
    pub(super) current_method: Option<(usize, String)>,
    pub(super) async_body: bool,
    /// The MODULE whose body is being lowered, for a bare call inside it.
    ///
    /// `module M { fun helper() { .. } helper() }` calls `M.helper`, but the
    /// statement is lowered into the top-level frame where no receiver is in
    /// scope, so the owner has to be carried explicitly.
    pub(super) enclosing_module: Option<String>,
}

#[derive(Clone)]
pub(super) struct ProgramBinding {
    pub(super) name: String,
    pub(super) shared: bool,
}

#[derive(Clone)]
pub(super) struct Binding {
    pub(super) name: String,
    pub(super) register: Register,
    pub(super) shared: bool,
    /// Register holding whether a DEFERRED binding has been assigned yet.
    pub(super) assigned: Option<Register>,
}

impl Binding {
    pub(super) const fn value(name: String, register: Register) -> Self {
        Self {
            name,
            register,
            shared: false,
            assigned: None,
        }
    }

    pub(super) const fn deferred(name: String, register: Register, assigned: Register) -> Self {
        Self {
            name,
            register,
            shared: false,
            assigned: Some(assigned),
        }
    }

    pub(super) const fn shared(name: String, register: Register) -> Self {
        Self {
            name,
            register,
            shared: true,
            assigned: None,
        }
    }
}

pub(super) struct LoopContext {
    pub(super) continue_target: usize,
    pub(super) breaks: Vec<usize>,
    pub(super) iterator: Option<Register>,
}

impl<'a, 'b> Lowering<'a, 'b> {
    pub(super) fn validate_composed_type(
        &self,
        expression: &iris_syntax::TypeExpression,
    ) -> Result<(), CompileError> {
        match expression {
            iris_syntax::TypeExpression::Name(name)
                if matches!(name.as_str(), "Never" | "NonNil")
                    || self.class_index(name).is_some()
                    || self.contract_index(name).is_some()
                    || matches!(
                        name.as_str(),
                        "Object" | "Nil" | "Bool" | "Integer" | "Float32" | "Float64" | "String"
                    ) =>
            {
                Ok(())
            }
            iris_syntax::TypeExpression::Union(members)
            | iris_syntax::TypeExpression::Intersection(members) => members
                .iter()
                .try_for_each(|member| self.validate_composed_type(member)),
            _ => Err(CompileError::new("expression reified type")),
        }
    }
    pub(super) fn new(
        signatures: &'a [Signature<'b>],
        classes: &'a [Class],
        contracts: &'a [Contract],
        declared_functions: usize,
        closures: &'a mut Vec<Function>,
        program_bindings: &'a [ProgramBinding],
        top_level: bool,
    ) -> Self {
        Self {
            instructions: Vec::new(),
            next_register: 0,
            names: Vec::new(),
            signatures,
            classes,
            contracts,
            declared_functions,
            closures,
            loops: Vec::new(),
            exception_contexts: Vec::new(),
            method_values: Vec::new(),
            program_bindings,
            top_level,
            current_method: None,
            async_body: false,
            enclosing_module: None,
        }
    }

    /// Resolves `Module.selector` to a function index.
    pub(super) fn resolve(&self, module: &str, selector: &str) -> Option<usize> {
        self.signatures
            .iter()
            .position(|signature| signature.module == module && signature.selector == selector)
    }
    /// Reserves a fresh register.
    pub(super) fn allocate(&mut self) -> Result<Register, CompileError> {
        let register = self.next_register;
        self.next_register = self
            .next_register
            .checked_add(1)
            .ok_or_else(|| CompileError::new("register exhaustion"))?;
        Ok(register)
    }

    pub(super) fn lookup(&self, name: &str) -> Option<Register> {
        self.lookup_binding(name).map(|binding| binding.register)
    }

    pub(super) fn lookup_binding(&self, name: &str) -> Option<&Binding> {
        self.names.iter().rev().find(|binding| binding.name == name)
    }

    /// Answers a register holding the TRUTH of `value`, per `C022`.
    ///
    /// Branching on the value directly decides truth structurally, which is
    /// wrong for a class that defines `to_bool`: `if p` would take the then
    /// branch for a `p` whose `to_bool` answers false. Only a condition the
    /// PROGRAM wrote goes through here - an internally produced Bool, like a
    /// catch-class match, is already a Bool and re-testing it would send
    /// `to_bool` the program never asked for.
    pub(super) fn truth_test(&mut self, value: Register) -> Result<Register, CompileError> {
        let destination = self.allocate()?;
        self.instructions
            .push(Instruction::TestTruth { destination, value });
        Ok(destination)
    }

    /// Answers a register holding the binding's CURRENT value.
    ///
    /// A `mut` binding is a shared cell rather than a plain register, so a
    /// read has to go through the cell or it would see the value the cell
    /// held when it was created.
    pub(super) fn read_binding(&mut self, binding: &Binding) -> Result<Register, CompileError> {
        if !binding.shared {
            return Ok(binding.register);
        }
        let destination = self.allocate()?;
        self.instructions.push(Instruction::LoadCell {
            destination,
            cell: binding.register,
        });
        Ok(destination)
    }

    /// Writes `source` to the binding, through its cell when it has one.
    pub(super) fn write_binding(
        &mut self,
        binding: &Binding,
        source: Register,
    ) -> Result<(), CompileError> {
        if binding.shared {
            self.instructions.push(Instruction::StoreCell {
                destination: source,
                cell: binding.register,
                source,
            });
        } else {
            self.instructions.push(Instruction::Move {
                destination: binding.register,
                source,
            });
        }
        Ok(())
    }
}
