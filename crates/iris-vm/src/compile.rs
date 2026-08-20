//! Lowers a parsed program to register instructions.
//!
//! The execution IR is REGISTER-based rather than stack-based, per the design
//! review's section 5.7: it lowers more directly to Cranelift IR, has no stack
//! effect to track, gives a simpler verifier, disassembles readably, cannot
//! underflow an operand stack, and suits later SSA lowering.

use iris_runtime::NativeSelector;
use iris_syntax::{BinaryOperator, Expression, Statement, TypeExpression, UnaryOperator};

/// A virtual register index.
///
/// Registers are virtual and unbounded at this stage. Allocation to a fixed
/// bank belongs to a later pass; assigning them here would bake a machine
/// constraint into the IR before any backend needs it.
pub type Register = u16;

/// One three-address instruction.
///
/// Every instruction names its operands and its destination explicitly, so an
/// instruction's meaning does not depend on execution history. That is what
/// makes the verifier a single linear pass.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Instruction {
    /// Loads an arbitrary-precision Integer, held as canonical decimal text.
    LoadInteger {
        destination: Register,
        digits: String,
    },
    /// Loads an IEEE-754 binary64 value, held as BITS so a literal cannot
    /// drift through a decimal round trip.
    LoadFloat64 {
        destination: Register,
        bits: u64,
    },
    /// Loads an IEEE-754 binary32 value, held as bits for the same reason.
    LoadFloat32 {
        destination: Register,
        bits: u32,
    },
    /// Loads a String.
    LoadText {
        destination: Register,
        text: String,
    },
    /// Loads an interned Symbol spelling.
    LoadSymbol {
        destination: Register,
        name: String,
    },
    /// Loads a Bool.
    LoadBool {
        destination: Register,
        value: bool,
    },
    /// Loads nil.
    LoadNil {
        destination: Register,
    },
    LoadGlobal {
        destination: Register,
        name: String,
    },
    StoreGlobal {
        destination: Register,
        name: String,
        value: Register,
    },
    /// Copies one register to another.
    Move {
        destination: Register,
        source: Register,
    },
    /// Sends a native binary selector: `destination = left <selector> right`.
    Binary {
        destination: Register,
        selector: &'static str,
        left: Register,
        right: Register,
    },
    /// Sends a native selector with no arguments to one register.
    Unary {
        destination: Register,
        selector: &'static str,
        operand: Register,
    },
    /// Builds an Array from a contiguous register range, in source order.
    BuildArray {
        destination: Register,
        first: Register,
        count: u16,
    },
    BuildHash {
        destination: Register,
        first: Register,
        count: u16,
    },
    Index {
        destination: Register,
        receiver: Register,
        index: Register,
    },
    SetIndex {
        destination: Register,
        receiver: Register,
        index: Register,
        value: Register,
    },
    BindMember {
        destination: Register,
        receiver: Register,
        selector: String,
    },
    Identity {
        destination: Register,
        left: Register,
        right: Register,
    },
    /// Reinterprets an Integer register's bits as a float of the given width.
    FromBits {
        destination: Register,
        width: FloatWidth,
        bits: Register,
    },
    /// Jumps to `target` when `condition` holds FALSE.
    ///
    /// Only the false branch is conditional. One conditional form plus an
    /// unconditional `Jump` expresses every shape this subset needs, and each
    /// extra branch opcode is another case the verifier must reason about.
    JumpUnless {
        condition: Register,
        target: usize,
    },
    /// Jumps to `target` unconditionally.
    Jump {
        target: usize,
    },
    /// Installs an exception handler for the following protected region.
    EnterTry {
        handler: usize,
        cleanup: usize,
        exception: Register,
    },
    CatchMatch {
        destination: Register,
        exception: Register,
        class: String,
    },
    /// Removes the innermost handler after normal completion.
    LeaveTry,
    /// Raises the value in the current frame.
    Raise {
        value: Register,
    },
    /// Allocates a Closure with a snapshot of a contiguous capture window.
    MakeClosure {
        destination: Register,
        function: usize,
        first: Register,
        count: u16,
    },
    /// Calls function `function` with a contiguous argument window.
    ///
    /// The arguments occupy `first .. first+count`, mirroring `BuildArray`, so
    /// a call names a REGISTER WINDOW rather than carrying an operand list.
    /// That is what keeps the callee's parameters addressable as ordinary
    /// registers once the frame is pushed.
    Call {
        destination: Register,
        function: usize,
        first: Register,
        count: u16,
    },
    New {
        destination: Register,
        class: usize,
        first: Register,
        count: u16,
    },
    Send {
        destination: Register,
        receiver: Register,
        selector: String,
        first: Register,
        count: u16,
    },
    SendClass {
        destination: Register,
        class: usize,
        selector: String,
        first: Register,
        count: u16,
    },
    GetIvar {
        destination: Register,
        receiver: Register,
        name: String,
    },
    SetIvar {
        destination: Register,
        receiver: Register,
        name: String,
        value: Register,
    },
    /// Returns `value` from the current frame.
    Return {
        value: Register,
    },
}

impl Instruction {
    /// The register this instruction writes, when it writes one.
    pub(crate) const fn destination(&self) -> Option<Register> {
        match self {
            Self::LoadInteger { destination, .. }
            | Self::LoadFloat64 { destination, .. }
            | Self::LoadFloat32 { destination, .. }
            | Self::LoadText { destination, .. }
            | Self::LoadSymbol { destination, .. }
            | Self::LoadBool { destination, .. }
            | Self::LoadNil { destination }
            | Self::LoadGlobal { destination, .. }
            | Self::StoreGlobal { destination, .. }
            | Self::Move { destination, .. }
            | Self::Binary { destination, .. }
            | Self::Unary { destination, .. }
            | Self::BuildArray { destination, .. }
            | Self::BuildHash { destination, .. }
            | Self::Index { destination, .. }
            | Self::SetIndex { destination, .. }
            | Self::BindMember { destination, .. }
            | Self::Identity { destination, .. }
            | Self::Call { destination, .. }
            | Self::New { destination, .. }
            | Self::Send { destination, .. }
            | Self::SendClass { destination, .. }
            | Self::GetIvar { destination, .. }
            | Self::SetIvar { destination, .. }
            | Self::MakeClosure { destination, .. }
            | Self::FromBits { destination, .. } => Some(*destination),
            Self::CatchMatch { destination, .. } => Some(*destination),
            // A branch or a return produces no value.
            Self::JumpUnless { .. }
            | Self::Jump { .. }
            | Self::EnterTry { .. }
            | Self::LeaveTry
            | Self::Raise { .. }
            | Self::Return { .. } => None,
        }
    }
}

/// Which IEEE interchange width a `from_bits` names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FloatWidth {
    /// IEEE-754 binary32.
    Bits32,
    /// IEEE-754 binary64.
    Bits64,
}

/// One callable body with its own register file.
///
/// A frame is a REGISTER WINDOW: each call gets a fresh file of `registers`
/// slots, and parameters arrive pre-bound in registers `0 .. parameters`.
/// Nothing is shared with the caller, so a callee cannot read a caller's
/// registers and recursion needs no save/restore of individual registers.
///
/// This is also what makes a GC root set enumerable: the live frames ARE the
/// roots. The tree-walking evaluator threads locals through a `&HashMap`
/// parameter, so its caller frames sit on the Rust stack and cannot be walked -
/// which is why a collection there refuses inside a method body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Function {
    /// The name this function was declared under, for diagnostics.
    pub(crate) name: String,
    /// How many leading registers hold parameters.
    pub(crate) parameters: usize,
    pub(crate) captures: usize,
    /// The size of this frame's register file.
    pub(crate) registers: usize,
    pub(crate) instructions: Vec<Instruction>,
}

/// A compiled program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program {
    pub(crate) instructions: Vec<Instruction>,
    /// How many registers the top-level frame uses.
    pub(crate) registers: usize,
    /// The register holding the program's answer.
    pub(crate) result: Register,
    /// Callable bodies, addressed by index from `Call`.
    pub(crate) functions: Vec<Function>,
    pub(crate) classes: Vec<Class>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Class {
    pub(crate) name: String,
    pub(crate) superclass: Option<usize>,
    pub(crate) methods: Vec<(String, usize)>,
    pub(crate) class_methods: Vec<(String, usize)>,
    pub(crate) reopens: Vec<ClassReopen>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClassReopen {
    pub(crate) methods: Vec<(String, usize)>,
}

/// Why a program could not be compiled.
///
/// This is NOT a program error. It means this backend does not yet cover the
/// construct, so the caller must decline rather than produce a result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompileError {
    /// The construct that is not covered.
    pub construct: String,
}

impl CompileError {
    fn new(construct: impl Into<String>) -> Self {
        Self {
            construct: construct.into(),
        }
    }
}

/// Compiles `source`, or reports the first construct this backend lacks.
///
/// # Errors
/// Returns the uncovered construct, or a parse rejection.
pub fn compile(source: &str) -> Result<Program, CompileError> {
    let parsed = iris_parser::parse(source);
    if !parsed.program_accepted {
        return Err(CompileError::new("rejected source"));
    }

    // Name resolution happens HERE, before any instruction is emitted: a call
    // is lowered to a function INDEX, never to a name looked up at run time.
    // The design review places this responsibility in a HIR layer between AST
    // and execution IR; this is that resolution, done in one pass while the
    // covered surface is small enough not to need a separate representation.
    let (signatures, classes) = collect_signatures(&parsed.program.declarations)?;

    let mut functions = Vec::with_capacity(signatures.len());
    let mut closures = Vec::new();
    for signature in &signatures {
        functions.push(lower_function(
            signature,
            &signatures,
            &classes,
            signatures.len(),
            &mut closures,
        )?);
    }

    if parsed.program.statements.is_empty() {
        return Err(CompileError::new("empty program"));
    }
    let mut lowering = Lowering::new(&signatures, &classes, signatures.len(), &mut closures);
    // A top-level program answers the values of its non-BINDING statements:
    // one value directly, several as an Array. That convention belongs to the
    // reference evaluator, and a backend that answered only the last statement
    // would disagree with it for a reason that is not semantic - which is
    // exactly what the differential harness exists to catch.
    let mut produced = Vec::new();
    for statement in &parsed.program.statements {
        let value = lowering.statement(statement)?;
        if !matches!(statement, Statement::Binding { .. } | Statement::Method(_)) {
            produced.push(value);
        }
    }
    let result = match produced.as_slice() {
        [] => return Err(CompileError::new("no program value")),
        [single] => *single,
        _ => {
            let first = lowering.next_register;
            let count =
                u16::try_from(produced.len()).map_err(|_| CompileError::new("program too wide"))?;
            for source in produced {
                let destination = lowering.allocate()?;
                lowering.instructions.push(Instruction::Move {
                    destination,
                    source,
                });
            }
            let destination = lowering.allocate()?;
            lowering.instructions.push(Instruction::BuildArray {
                destination,
                first,
                count,
            });
            destination
        }
    };
    let registers = lowering.next_register as usize;
    let instructions = std::mem::take(&mut lowering.instructions);
    drop(lowering);
    functions.extend(closures);
    Ok(Program {
        instructions,
        registers,
        result,
        functions,
        classes,
    })
}

/// A module function this backend can call, resolved before lowering.
struct Signature<'a> {
    module: &'a str,
    selector: &'a str,
    parameters: Vec<&'a str>,
    body: &'a [Statement],
    receiver: bool,
    class_method: bool,
}

type MethodTable = Vec<(String, usize)>;

/// Collects the module functions this subset covers.
///
/// Only a plain `module` holding plain functions is covered. A Class, a
/// Contract, an import, a mixin, a decorator, a generic or an async function
/// carries semantics - MRO, revisions, capability checks, suspension - that
/// belong to the runtime rather than to this backend, so they are DECLINED
/// rather than approximated.
fn collect_signatures(
    declarations: &[iris_syntax::Declaration],
) -> Result<(Vec<Signature<'_>>, Vec<Class>), CompileError> {
    let mut signatures = Vec::new();
    let mut classes = Vec::new();
    for declaration in declarations {
        let iris_syntax::Declaration::Module(module) = declaration else {
            let iris_syntax::Declaration::Class(class) = declaration else {
                return Err(CompileError::new(match declaration {
                    iris_syntax::Declaration::Contract(_) => "declaration contract",
                    iris_syntax::Declaration::Import(_) => "declaration import",
                    iris_syntax::Declaration::Export(_) => "declaration export",
                    iris_syntax::Declaration::TypeAlias(_) => "declaration type alias",
                    iris_syntax::Declaration::Class(_) | iris_syntax::Declaration::Module(_) => {
                        "declaration covered"
                    }
                }));
            };
            if !class.decorators.is_empty() {
                return Err(CompileError::new("class decorator"));
            }
            if class.reopen {
                if class.extends.is_some()
                    || !class.implements.is_empty()
                    || !class.mixins.is_empty()
                    || !class.constraints.is_empty()
                    || !class.parameters.is_empty()
                    || !class.meta_deny.is_empty()
                {
                    return Err(CompileError::new("class reopen header"));
                }
                let Some(target) = classes
                    .iter()
                    .position(|known: &Class| known.name == class.name)
                else {
                    return Err(CompileError::new("class reopen target"));
                };
                let first_function = signatures.len();
                collect_methods(&class.name, &class.body, true, &mut signatures)?;
                let (methods, class_methods) = collected_method_tables(&signatures, first_function);
                if !class_methods.is_empty() {
                    return Err(CompileError::new("class reopen class method"));
                }
                classes[target].reopens.push(ClassReopen { methods });
                continue;
            }
            if !class.implements.is_empty() {
                return Err(CompileError::new("class implements"));
            }
            if !class.mixins.is_empty() {
                return Err(CompileError::new("class mixin"));
            }
            if !class.constraints.is_empty() {
                return Err(CompileError::new("class constraints"));
            }
            if !class.parameters.is_empty() {
                return Err(CompileError::new("class generics"));
            }
            if !class.meta_deny.is_empty() {
                return Err(CompileError::new("class meta deny"));
            }
            let superclass_name = match &class.extends {
                Some(TypeExpression::Name(name)) => Some(name.as_str()),
                Some(_) => return Err(CompileError::new("class superclass")),
                None => None,
            };
            let class_index = classes.len();
            let first_function = signatures.len();
            collect_methods(&class.name, &class.body, true, &mut signatures)?;
            let superclass = match superclass_name {
                Some("Object") | None => None,
                Some(name) => declarations
                    .iter()
                    .filter_map(|declaration| match declaration {
                        iris_syntax::Declaration::Class(candidate) => Some(&candidate.name),
                        _ => None,
                    })
                    .position(|candidate| candidate == name)
                    .ok_or_else(|| CompileError::new("class superclass"))?
                    .into(),
            };
            let (methods, class_methods) = collected_method_tables(&signatures, first_function);
            classes.push(Class {
                name: class.name.clone(),
                superclass,
                methods,
                class_methods,
                reopens: Vec::new(),
            });
            let _ = class_index;
            continue;
        };
        if module.reopen
            || !module.mixins.is_empty()
            || !module.parameters.is_empty()
            || !module.decorators.is_empty()
        {
            return Err(CompileError::new("module"));
        }
        collect_methods(&module.name, &module.body, false, &mut signatures)?;
    }
    Ok((signatures, classes))
}

fn collected_method_tables(
    signatures: &[Signature<'_>],
    first_function: usize,
) -> (MethodTable, MethodTable) {
    let mut methods = Vec::new();
    let mut class_methods = Vec::new();
    for (offset, signature) in signatures[first_function..].iter().enumerate() {
        let entry = (signature.selector.to_owned(), first_function + offset);
        if signature.class_method {
            class_methods.push(entry);
        } else {
            methods.push(entry);
        }
    }
    (methods, class_methods)
}

fn collect_methods<'a>(
    owner: &'a str,
    body: &'a [Statement],
    receiver: bool,
    signatures: &mut Vec<Signature<'a>>,
) -> Result<(), CompileError> {
    for statement in body {
        let Statement::Method(method) = statement else {
            return Err(CompileError::new(if receiver {
                "class body"
            } else {
                "module body"
            }));
        };
        if method.is_async
            || method.impl_contract.is_some()
            || !method.decorators.is_empty()
            || !method.type_parameters.is_empty()
            || !matches!(
                method.kind,
                iris_syntax::MethodKind::Instance | iris_syntax::MethodKind::Class
            )
            || (!receiver && method.kind != iris_syntax::MethodKind::Instance)
        {
            return Err(CompileError::new(if method.is_async {
                "method async"
            } else if method.impl_contract.is_some() {
                "method contract implementation"
            } else if !method.decorators.is_empty() {
                "method decorator"
            } else if !method.type_parameters.is_empty() {
                "method generics"
            } else {
                match method.kind {
                    iris_syntax::MethodKind::Module => "method module",
                    iris_syntax::MethodKind::Property => "method property",
                    iris_syntax::MethodKind::Instance | iris_syntax::MethodKind::Class => {
                        "method kind"
                    }
                }
            }));
        }
        let Some(body) = method.body.as_deref() else {
            return Err(CompileError::new("abstract method"));
        };
        let mut parameters = Vec::with_capacity(method.parameters.len());
        for parameter in &method.parameters {
            // Only positional parameters. A rest, keyword or block
            // parameter needs argument shapes this subset does not build.
            if parameter.category != iris_syntax::ParameterCategory::Positional {
                return Err(CompileError::new("parameter"));
            }
            parameters.push(parameter.name.as_str());
        }
        signatures.push(Signature {
            module: owner,
            selector: &method.selector,
            parameters,
            body,
            receiver,
            class_method: method.kind == iris_syntax::MethodKind::Class,
        });
    }
    Ok(())
}

/// Lowers one function into its own frame.
fn lower_function(
    signature: &Signature<'_>,
    signatures: &[Signature<'_>],
    classes: &[Class],
    declared_functions: usize,
    closures: &mut Vec<Function>,
) -> Result<Function, CompileError> {
    let mut lowering = Lowering::new(signatures, classes, declared_functions, closures);
    if signature.receiver {
        let receiver = lowering.allocate()?;
        lowering.names.push(("self".to_owned(), receiver));
    }
    // Parameters occupy the leading registers, so a call can copy arguments
    // into a fresh frame without the callee knowing where they came from.
    for parameter in &signature.parameters {
        let register = lowering.allocate()?;
        lowering.names.push(((*parameter).to_owned(), register));
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
        registers: lowering.next_register as usize,
        instructions: lowering.instructions,
    })
}

struct Lowering<'a, 'b> {
    instructions: Vec<Instruction>,
    next_register: Register,
    /// Names bound so far, each pinned to the register holding its value.
    names: Vec<(String, Register)>,
    /// Functions callable from this frame, resolved before lowering.
    signatures: &'a [Signature<'b>],
    classes: &'a [Class],
    declared_functions: usize,
    closures: &'a mut Vec<Function>,
}

impl<'a, 'b> Lowering<'a, 'b> {
    fn new(
        signatures: &'a [Signature<'b>],
        classes: &'a [Class],
        declared_functions: usize,
        closures: &'a mut Vec<Function>,
    ) -> Self {
        Self {
            instructions: Vec::new(),
            next_register: 0,
            names: Vec::new(),
            signatures,
            classes,
            declared_functions,
            closures,
        }
    }

    /// Resolves `Module.selector` to a function index.
    fn resolve(&self, module: &str, selector: &str) -> Option<usize> {
        self.signatures
            .iter()
            .position(|signature| signature.module == module && signature.selector == selector)
    }
    /// Reserves a fresh register.
    fn allocate(&mut self) -> Result<Register, CompileError> {
        let register = self.next_register;
        self.next_register = self
            .next_register
            .checked_add(1)
            .ok_or_else(|| CompileError::new("register exhaustion"))?;
        Ok(register)
    }

    fn lookup(&self, name: &str) -> Option<Register> {
        self.names
            .iter()
            .rev()
            .find_map(|(held, register)| (held == name).then_some(*register))
    }

    fn statement(&mut self, statement: &Statement) -> Result<Register, CompileError> {
        match statement {
            Statement::Expression(expression) => self.expression(expression),
            // A plain immutable binding is covered. `mut`, `const`, globals and
            // deferred bindings carry rules - reassignment, definite
            // assignment, package-qualified identity - this subset lacks.
            Statement::Binding {
                mutable,
                name,
                annotation: None,
                value,
                ..
            } => {
                let _ = mutable;
                let value = self.expression(value)?;
                // A rebinding SHADOWS rather than overwrites: the earlier
                // register may still be read by a closure or an earlier
                // instruction, so reusing it would corrupt that read.
                //
                // A `mut` binding still gets ONE register, which assignment
                // then updates in place. That is what lets a loop carry a
                // value across iterations: a fresh register per assignment
                // would leave the loop reading its pre-loop value forever.
                let destination = self.allocate()?;
                self.instructions.push(Instruction::Move {
                    destination,
                    source: value,
                });
                self.names.push((name.clone(), destination));
                Ok(destination)
            }
            Statement::GlobalBinding {
                name,
                annotation: None,
                value,
                ..
            } => {
                let value = self.expression(value)?;
                let destination = self.allocate()?;
                self.instructions.push(Instruction::StoreGlobal {
                    destination,
                    name: name.clone(),
                    value,
                });
                Ok(destination)
            }
            // A loop is a BACKWARD jump, which is why the verifier had to
            // become a dataflow fixpoint: a body is entered before its own
            // writes have happened, so a linear scan cannot decide definite
            // assignment across the back edge.
            Statement::While {
                label: None,
                condition,
                body,
            } => {
                // The loop answers nil: `IRIS-V1-CONTROL-C023` gives a normal
                // loop completion no value of its own, and only a `break` with
                // an operand carries one - which this subset declines.
                let destination = self.allocate()?;
                self.instructions.push(Instruction::LoadNil { destination });
                let top = self.instructions.len();
                let condition = self.expression(condition)?;
                let exit = self.instructions.len();
                self.instructions.push(Instruction::JumpUnless {
                    condition,
                    target: 0,
                });
                self.body(body)?;
                self.instructions.push(Instruction::Jump { target: top });
                let after = self.instructions.len();
                self.patch(exit, after)?;
                Ok(destination)
            }
            // An `if` yields a value, so both arms write the SAME destination
            // register. That is what lets the value be read afterwards without
            // knowing which arm ran.
            Statement::If {
                condition,
                then_body,
                else_body,
            } => {
                let destination = self.allocate()?;
                let condition = self.expression(condition)?;
                let branch = self.instructions.len();
                self.instructions.push(Instruction::JumpUnless {
                    condition,
                    target: 0,
                });
                let taken = self.body(then_body)?;
                self.instructions.push(Instruction::Move {
                    destination,
                    source: taken,
                });
                let skip = self.instructions.len();
                self.instructions.push(Instruction::Jump { target: 0 });

                let otherwise = self.instructions.len();
                match else_body {
                    Some(body) => {
                        let value = self.body(body)?;
                        self.instructions.push(Instruction::Move {
                            destination,
                            source: value,
                        });
                    }
                    // A missing else answers nil, so the destination is
                    // written on EVERY path and the verifier's
                    // written-before-read rule holds however the branch goes.
                    None => self.instructions.push(Instruction::LoadNil { destination }),
                }
                let after = self.instructions.len();
                self.patch(branch, otherwise)?;
                self.patch(skip, after)?;
                Ok(destination)
            }
            Statement::Return(value) => {
                let value = match value {
                    Some(value) => self.expression(value)?,
                    None => {
                        let destination = self.allocate()?;
                        self.instructions.push(Instruction::LoadNil { destination });
                        destination
                    }
                };
                self.instructions.push(Instruction::Return { value });
                Ok(value)
            }
            Statement::Raise(Some(raise)) if raise.cause.is_none() => {
                let value = self.expression(&raise.value)?;
                self.instructions.push(Instruction::Raise { value });
                Ok(value)
            }
            Statement::Try {
                body,
                catches,
                finally,
            } => self.try_body(body, catches, finally),
            other => Err(CompileError::new(format!(
                "statement {}",
                match other {
                    Statement::Binding { .. } => "binding",
                    Statement::Method(_) => "method",
                    Statement::GlobalBinding { .. } => "global",
                    Statement::SharedBinding { .. } => "shared",
                    Statement::DeferredBinding { .. } => "deferred",
                    Statement::StoredProperty { .. } => "stored property",
                    Statement::Match { .. } => "match",
                    Statement::For { .. } => "for",
                    Statement::Try { .. } => "try",
                    Statement::Raise(_) => "raise",
                    Statement::Break { .. } => "break",
                    Statement::Continue(_) => "continue",
                    _ => "other",
                }
            ))),
        }
    }

    /// Lowers a block, answering the register holding its last value.
    fn body(&mut self, statements: &[Statement]) -> Result<Register, CompileError> {
        let Some((last, leading)) = statements.split_last() else {
            let destination = self.allocate()?;
            self.instructions.push(Instruction::LoadNil { destination });
            return Ok(destination);
        };
        // A block scopes its bindings: a name bound inside must not leak out.
        let outer = self.names.len();
        for statement in leading {
            self.statement(statement)?;
        }
        let value = self.statement(last)?;
        self.names.truncate(outer);
        Ok(value)
    }

    fn try_body(
        &mut self,
        body: &[Statement],
        catches: &[iris_syntax::CatchClause],
        finally: &Option<Vec<Statement>>,
    ) -> Result<Register, CompileError> {
        if catches.iter().any(|catch| catch.context.is_some()) {
            return Err(CompileError::new("try exception context"));
        }
        if catches.iter().any(|catch| {
            catch
                .filter
                .as_ref()
                .is_some_and(|filter| !matches!(filter, TypeExpression::Name(_)))
        }) {
            return Err(CompileError::new("try catch filter"));
        }
        let destination = self.allocate()?;
        let exception = self.allocate()?;
        let enter = self.instructions.len();
        self.instructions.push(Instruction::EnterTry {
            handler: 0,
            cleanup: 0,
            exception,
        });
        let value = self.body(body)?;
        self.instructions.push(Instruction::LeaveTry);
        self.instructions.push(Instruction::Move {
            destination,
            source: value,
        });
        let normal_skip = self.instructions.len();
        self.instructions.push(Instruction::Jump { target: 0 });

        let handler = self.instructions.len();
        self.patch(enter, handler)?;
        let mut caught_skips = Vec::with_capacity(catches.len());
        for catch in catches {
            let mismatch = if let Some(TypeExpression::Name(class)) = &catch.filter {
                let matches = self.allocate()?;
                self.instructions.push(Instruction::CatchMatch {
                    destination: matches,
                    exception,
                    class: class.clone(),
                });
                let mismatch = self.instructions.len();
                self.instructions.push(Instruction::JumpUnless {
                    condition: matches,
                    target: 0,
                });
                Some(mismatch)
            } else {
                None
            };
            let catch_enter = self.instructions.len();
            self.instructions.push(Instruction::EnterTry {
                handler: 0,
                cleanup: 0,
                exception,
            });
            let outer = self.names.len();
            if let Some(iris_syntax::CatchBinding::Name(name)) = &catch.binding {
                self.names.push((name.clone(), exception));
            }
            let caught = self.body(&catch.body)?;
            self.names.truncate(outer);
            self.instructions.push(Instruction::LeaveTry);
            self.instructions.push(Instruction::Move {
                destination,
                source: caught,
            });
            let caught_skip = self.instructions.len();
            self.instructions.push(Instruction::Jump { target: 0 });
            caught_skips.push(caught_skip);
            let exceptional_cleanup = self.instructions.len();
            self.patch(catch_enter, exceptional_cleanup)?;
            if let Some(finally) = finally {
                self.body(finally)?;
            }
            self.instructions
                .push(Instruction::Raise { value: exception });
            if let Some(mismatch) = mismatch {
                let next = self.instructions.len();
                self.patch(mismatch, next)?;
            } else {
                break;
            }
        }
        if catches.last().is_none_or(|catch| catch.filter.is_some()) {
            if let Some(finally) = finally {
                self.body(finally)?;
            }
            self.instructions
                .push(Instruction::Raise { value: exception });
        }

        let cleanup = self.instructions.len();
        for caught_skip in caught_skips {
            self.patch(caught_skip, cleanup)?;
        }
        self.patch(normal_skip, cleanup)?;
        if let Some(Instruction::EnterTry { cleanup: slot, .. }) = self.instructions.get_mut(enter)
        {
            *slot = cleanup;
        }
        if let Some(finally) = finally {
            self.body(finally)?;
        }
        Ok(destination)
    }

    /// Fills in a forward jump once its target is known.
    fn patch(&mut self, at: usize, target: usize) -> Result<(), CompileError> {
        match self.instructions.get_mut(at) {
            Some(
                Instruction::JumpUnless { target: slot, .. }
                | Instruction::Jump { target: slot }
                | Instruction::EnterTry { handler: slot, .. },
            ) => {
                *slot = target;
                Ok(())
            }
            _ => Err(CompileError::new("branch patch")),
        }
    }

    fn expression(&mut self, expression: &Expression) -> Result<Register, CompileError> {
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
            Expression::Name(name) => self
                .lookup(name)
                // An unbound name is not this backend's to resolve: it could
                // be a Class, a Module, or a method-scope local.
                .ok_or_else(|| CompileError::new("name unbound")),
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
            Expression::Grouped(inner) => self.expression(inner),
            Expression::Binary {
                left,
                operator,
                right,
            } => {
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
                let UnaryOperator::Negate = operator else {
                    return Err(CompileError::new(format!("unary {operator:?}")));
                };
                // The runtime spells this `negate`; `-@` is not a native
                // selector and emitting it produced a machine defect rather
                // than the RangeError the reference answers.
                let operand = self.expression(operand)?;
                let destination = self.allocate()?;
                self.instructions.push(Instruction::Unary {
                    destination,
                    selector: "negate",
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
                let receiver = self.expression(receiver)?;
                let destination = self.allocate()?;
                self.instructions.push(Instruction::BindMember {
                    destination,
                    receiver,
                    selector: selector.clone(),
                });
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
                let Expression::Name(name) = left.as_ref() else {
                    return Err(CompileError::new("assignment target"));
                };
                let Some(destination) = self.lookup(name) else {
                    return Err(CompileError::new("name assignment unbound"));
                };
                let source = self.expression(right)?;
                self.instructions.push(Instruction::Move {
                    destination,
                    source,
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
                parameters,
                body,
                has_header: true,
                ..
            } => self.closure(parameters, body),
            other => Err(CompileError::new(construct_name(other))),
        }
    }

    fn closure(
        &mut self,
        parameters: &[String],
        body: &[Statement],
    ) -> Result<Register, CompileError> {
        let captures = self.names.clone();
        let mut closure_functions = Vec::new();
        let mut lowering = Lowering::new(
            self.signatures,
            self.classes,
            self.declared_functions + self.closures.len(),
            &mut closure_functions,
        );
        for (name, _) in &captures {
            let register = lowering.allocate()?;
            lowering.names.push((name.clone(), register));
        }
        for parameter in parameters {
            let register = lowering.allocate()?;
            lowering.names.push((parameter.clone(), register));
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
            registers,
            instructions,
        });
        let count = u16::try_from(captures.len())
            .map_err(|_| CompileError::new("closure capture too wide"))?;
        let first = self.next_register;
        for (_, source) in captures {
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
        let destination = self.allocate()?;
        self.instructions.push(Instruction::MakeClosure {
            destination,
            function,
            first,
            count,
        });
        Ok(destination)
    }

    /// Lowers a literal by RE-LEXING its text.
    ///
    /// The parser keeps a literal as source text, and the lexer is what turns
    /// it into a value under the chapter 02 rules - suffixes, radix prefixes,
    /// separators and precision warnings. Parsing the text here would be a
    /// second literal implementation, and a differential row would then compare
    /// this backend's literal rules against the lexer's.
    fn literal(&mut self, text: &str) -> Result<Register, CompileError> {
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
            iris_lexer::Literal::String(text) => Instruction::LoadText {
                destination,
                text: text.clone(),
            },
        });
        Ok(destination)
    }

    /// `Float32.from_bits(bits)`, `value.to_bits()` and `value.hash()` are the
    /// call shapes this subset covers. A general call needs frames and user
    /// Methods it does not have.
    fn call(
        &mut self,
        callee: &Expression,
        arguments: &[Expression],
    ) -> Result<Register, CompileError> {
        let Expression::Member { receiver, selector } = callee else {
            return Err(CompileError::new(match callee {
                Expression::Name(_) => "call bare name",
                Expression::Closure { .. } => "call closure",
                _ => "call callee",
            }));
        };
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
            let expected = self.signatures[function].parameters.len();
            if arguments.len() != expected {
                return Err(CompileError::new("call arity"));
            }
            let count =
                u16::try_from(arguments.len()).map_err(|_| CompileError::new("call too wide"))?;
            let mut lowered = Vec::with_capacity(arguments.len());
            for argument in arguments {
                lowered.push(self.expression(argument)?);
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
        if !matches!(receiver.as_ref(), Expression::Call { .. })
            && !matches!(receiver.as_ref(), Expression::Name(name) if self.lookup(name).is_some())
        {
            return Err(CompileError::new(match receiver.as_ref() {
                Expression::Name(_) => "call unbound receiver",
                Expression::Member { .. } => "call member receiver",
                Expression::Literal(_) => "call literal receiver",
                Expression::Array(_) => "call array receiver",
                _ => "call receiver",
            }));
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

    fn class_index(&self, name: &str) -> Option<usize> {
        self.classes.iter().position(|class| class.name == name)
    }

    fn argument_window(
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
fn binary_selector(operator: &BinaryOperator) -> Result<&'static str, CompileError> {
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

fn construct_name(expression: &Expression) -> String {
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
