//! Function, statement, expression, call, and closure lowering.

use super::declarations::Signature;
use super::{Class, CompileError, Contract, Function, Instruction, Register};

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
    // Parameters occupy the leading registers, so a call can copy arguments
    // into a fresh frame without the callee knowing where they came from.
    for parameter in &signature.parameters {
        let register = lowering.allocate()?;
        lowering
            .names
            .push(Binding::value(parameter.name.clone(), register));
    }
    let Some((last, leading)) = signature.body.split_last() else {
        return Err(CompileError::new("empty body"));
    };
    for statement in leading {
        lowering.statement(statement)?;
    }
    // A body's LAST expression is its value, which an explicit `Return`
    // makes uniform: every path out of a frame goes through one instruction.
    let value = lowering.statement(last)?;
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
    pub(super) deferred: Vec<String>,
    /// Functions callable from this frame, resolved before lowering.
    pub(super) signatures: &'a [Signature<'b>],
    pub(super) classes: &'a [Class],
    pub(super) contracts: &'a [Contract],
    pub(super) declared_functions: usize,
    pub(super) closures: &'a mut Vec<Function>,
    pub(super) loops: Vec<LoopContext>,
    pub(super) exception_contexts: Vec<Register>,
    pub(super) method_values: Vec<Register>,
    pub(super) program_bindings: &'a [ProgramBinding],
    pub(super) top_level: bool,
    pub(super) current_method: Option<(usize, String)>,
    pub(super) async_body: bool,
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
}

impl Binding {
    pub(super) const fn value(name: String, register: Register) -> Self {
        Self {
            name,
            register,
            shared: false,
        }
    }

    pub(super) const fn shared(name: String, register: Register) -> Self {
        Self {
            name,
            register,
            shared: true,
        }
    }
}

pub(super) struct LoopContext {
    pub(super) continue_target: usize,
    pub(super) breaks: Vec<usize>,
    pub(super) iterator: Option<Register>,
}

impl<'a, 'b> Lowering<'a, 'b> {
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
            deferred: Vec::new(),
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
}
