use std::collections::HashMap;

use iris_runtime::{
    Capability, ClassError, ClassId, ComparisonSlot, CompositionEdge, DispatchContext,
    DispatchError, DispatchOutcome, Kernel, MetaCapabilities, Method, MethodBody, MethodOwner,
    ModuleId, Runtime, Selector, StaticSpine, Truthiness, TruthinessError, TruthinessMethod, Value,
};
use iris_syntax::{
    BinaryOperator, ClassDeclaration, Expression, MethodDeclaration, MethodKind, ModuleDeclaration,
    Program, ProgramEntry, Statement,
};

use crate::EvaluationError;
use crate::source_method::{builtin, literal, visibility};

pub(super) fn evaluate(program: &Program) -> Result<Value, EvaluationError> {
    let mut evaluator = SourceEvaluator::new()?;
    evaluator.program(program)
}

/// The code, captured environment and receiver of one Closure allocation.
///
/// `IRIS-V1-RUNTIME-C072` requires a Closure created in an instance Method to
/// capture its CURRENT receiver and keep reading and writing that receiver's raw
/// ivars after escape, so the receiver is stored alongside the captured locals.
struct ClosureRecord {
    parameters: Vec<String>,
    body: Vec<Statement>,
    captured: HashMap<String, Value>,
    receiver: Option<Value>,
}

pub(super) struct SourceEvaluator {
    runtime: Runtime,
    kernel: Kernel,
    names: HashMap<String, Binding>,
    selectors: HashMap<String, Selector>,
    bodies: HashMap<u64, MethodDeclaration>,
    closures: HashMap<iris_runtime::ObjectId, ClosureRecord>,
    next_closure: u64,
    contract_names: HashMap<String, iris_runtime::ContractId>,
    class_contracts: HashMap<ClassId, Vec<iris_runtime::ContractId>>,
    qualified_methods: HashMap<(ClassId, iris_runtime::ContractId, Selector), Method>,
    contract_parents: HashMap<iris_runtime::ContractId, Vec<iris_runtime::ContractId>>,
    next_contract: u64,
    module_names: HashMap<String, ModuleId>,
    module_classes: HashMap<ModuleId, ClassId>,
    module_methods: HashMap<(ModuleId, Selector), Method>,
    module_method_overrides: HashMap<iris_runtime::MethodId, bool>,
    property_methods: HashMap<iris_runtime::MethodId, bool>,
    class_mixins: HashMap<ClassId, Vec<ClassId>>,
    static_superclasses: HashMap<ClassId, Option<ClassId>>,
    lexical_class: Option<ClassId>,
    current_method: Option<Method>,
    active_exception: Option<Value>,
    active_context: Option<Value>,
    next_selector: u64,
    next_body: u64,
}

#[derive(Clone)]
struct Binding {
    value: Value,
    mutable: bool,
}

impl Binding {
    const fn new(value: Value, mutable: bool) -> Self {
        Self { value, mutable }
    }

    const fn immutable(value: Value) -> Self {
        Self::new(value, false)
    }

    fn value(&self) -> Value {
        self.value.clone()
    }

    fn assign(&mut self, value: Value) -> Result<Value, ()> {
        if self.mutable {
            self.value = value.clone();
            Ok(value)
        } else {
            Err(())
        }
    }
}

fn construction_error(error: iris_runtime::ConstructionError) -> EvaluationError {
    match error {
        iris_runtime::ConstructionError::Runtime(iris_runtime::ExecutionError::Raised(value)) => {
            EvaluationError::Raised(value)
        }
        error => EvaluationError::Construction(error),
    }
}

impl SourceEvaluator {
    pub(super) fn new() -> Result<Self, EvaluationError> {
        let mut runtime = Runtime::new();
        let kernel = Kernel::new(runtime.registry_mut()).map_err(EvaluationError::Runtime)?;
        Ok(Self {
            runtime,
            kernel,
            names: HashMap::new(),
            selectors: HashMap::new(),
            bodies: HashMap::new(),
            closures: HashMap::new(),
            next_closure: 900_000,
            contract_names: HashMap::new(),
            class_contracts: HashMap::new(),
            qualified_methods: HashMap::new(),
            contract_parents: HashMap::new(),
            next_contract: 0,
            module_names: HashMap::new(),
            module_classes: HashMap::new(),
            module_methods: HashMap::new(),
            module_method_overrides: HashMap::new(),
            property_methods: HashMap::new(),
            class_mixins: HashMap::new(),
            static_superclasses: HashMap::new(),
            lexical_class: None,
            current_method: None,
            active_exception: None,
            active_context: None,
            next_selector: 1_000,
            next_body: 10_000,
        })
    }

    pub(super) fn program(&mut self, program: &Program) -> Result<Value, EvaluationError> {
        let mut values = Vec::new();
        for entry in &program.entries {
            match entry {
                ProgramEntry::Declaration(iris_syntax::Declaration::Class(class)) => {
                    self.class(class)?;
                }
                ProgramEntry::Declaration(iris_syntax::Declaration::Module(module)) => {
                    self.module(module)?;
                }
                ProgramEntry::Declaration(iris_syntax::Declaration::Contract(contract)) => {
                    self.contract(contract)?;
                }
                ProgramEntry::Statement(statement) => {
                    let value = self.statement(statement, &HashMap::new(), None)?;
                    if !matches!(statement, Statement::Binding { .. } | Statement::Method(_)) {
                        values.push(value);
                    }
                }
            }
        }
        match values.as_slice() {
            [] => Err(EvaluationError::UnsupportedConstruct),
            [value] => Ok(value.clone()),
            _ => Ok(Value::Array(values)),
        }
    }

    fn class(&mut self, declaration: &ClassDeclaration) -> Result<(), EvaluationError> {
        let superclass = match &declaration.extends {
            Some(iris_syntax::TypeExpression::Name(name)) => self.class_name(name)?,
            Some(_) => return Err(EvaluationError::UnsupportedConstruct),
            // A reopen keeps the superclass its origin declaration established;
            // only an origin declaration defaults to the C005 root.
            None if declaration.reopen => None,
            None => self.class_name("Object")?,
        };
        let mut mixins = Vec::new();
        let mut class_mixins = Vec::new();
        for mixin in &declaration.mixins {
            match &mixin.target {
                iris_syntax::TypeExpression::Name(name) => {
                    if let Some(module) = self.module_names.get(name) {
                        mixins.push(CompositionEdge::new(*module, mixin.private_access));
                    } else if let Some(class) = self.class_name(name)? {
                        class_mixins.push(class);
                    }
                }
                _ => return Err(EvaluationError::UnsupportedConstruct),
            }
        }
        let class = if declaration.reopen {
            let class = self
                .class_name(&declaration.name)?
                .ok_or(EvaluationError::UnsupportedConstruct)?;
            if self.is_builtin_class(class) && declaration.extends.is_some() {
                let replacement = self
                    .kernel
                    .class(iris_runtime::BuiltinClass::Nil)
                    .map_err(EvaluationError::Runtime)?;
                let mut candidate = self
                    .runtime
                    .registry_mut()
                    .open(class)
                    .map_err(EvaluationError::Class)?;
                candidate.replace_runtime_superclass(Some(replacement));
                self.runtime
                    .registry_mut()
                    .publish(candidate)
                    .map_err(EvaluationError::Class)?;
                return Ok(());
            }
            if superclass.is_some() {
                return Err(EvaluationError::UnsupportedConstruct);
            }
            if !mixins.is_empty() {
                // IRIS-V1-RUNTIME-C148 lets a stable built-in Class compose
                // Modules on open; only its superclass is protected, by C150.
                let mut candidate = self
                    .runtime
                    .registry_mut()
                    .open(class)
                    .map_err(EvaluationError::Class)?;
                for edge in &mixins {
                    candidate.add_module(edge.module());
                }
                self.runtime
                    .registry_mut()
                    .publish(candidate)
                    .map_err(EvaluationError::Class)?;
            }
            if !declaration.meta_deny.is_empty() {
                return Err(EvaluationError::Class(ClassError::MetaCapabilityDenied {
                    target: class,
                    operation: Capability::MethodSet,
                    policy_origin: iris_runtime::PolicyOrigin::Class(class),
                    reason: "open declarations cannot change MetaCapabilities policy",
                }));
            }
            class
        } else {
            let capabilities = meta_capabilities(&declaration.meta_deny)?;
            self.validate_module_overrides(superclass, &mixins)?;
            let class = self
                .runtime
                .registry_mut()
                .define_class_with_capabilities_and_composition_edges(
                    StaticSpine::new(1),
                    superclass,
                    capabilities,
                    &mixins,
                )
                .map_err(EvaluationError::Class)?;
            let body = self.register_body(MethodDeclaration {
                decorators: Vec::new(),
                is_override: false,
                impl_contract: None,
                kind: MethodKind::Instance,
                selector: "to_bool".into(),
                parameters: Vec::new(),
                visibility: iris_syntax::Visibility::Public,
                body: vec![Statement::Expression(Expression::Literal("true".into()))],
            });
            let selector = self.selector("to_bool");
            self.runtime
                .registry_mut()
                .publish_origin_method(class, selector, body, iris_runtime::Visibility::Public)
                .map_err(EvaluationError::Class)?;
            self.names.insert(
                declaration.name.clone(),
                Binding::immutable(Value::Class(class)),
            );
            self.static_superclasses.insert(class, superclass);
            self.class_mixins.insert(class, class_mixins);
            // IRIS-V1-TYPES-C044: only a Class declares instance Contract
            // conformance, and it must list each claimed Contract explicitly in
            // its header, so this records the `for` list rather than deriving it.
            let mut conformances = Vec::new();
            for target in &declaration.implements {
                let iris_syntax::TypeExpression::Name(name) = target else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                conformances.push(
                    *self
                        .contract_names
                        .get(name)
                        .ok_or(EvaluationError::UnsupportedConstruct)?,
                );
            }
            self.class_contracts.insert(class, conformances);
            class
        };
        let builtin = declaration.reopen && builtin(&declaration.name, &self.kernel).is_some();
        let outcome = self.class_body(class, builtin, declaration);
        if outcome.is_err() && !declaration.reopen {
            // IRIS-V1-RUNTIME-C024 requires a rejected declaration to publish no
            // Class. The name is bound before the body is validated, so an origin
            // declaration that fails validation must unbind it again.
            self.names.remove(&declaration.name);
        }
        outcome
    }

    fn class_body(
        &mut self,
        class: ClassId,
        builtin: bool,
        declaration: &ClassDeclaration,
    ) -> Result<(), EvaluationError> {
        self.publish_decorators(class, &declaration.decorators)?;
        let mut declared_in_body: Vec<(MethodKind, Selector)> = Vec::new();
        for statement in &declaration.body {
            match statement {
                Statement::StoredProperty {
                    decorators,
                    name,
                    initializer,
                } => {
                    self.stored_property(class, builtin, decorators, name, initializer.clone())?;
                }
                Statement::SharedBinding {
                    mutable,
                    name,
                    value,
                } => self.shared_binding(class, *mutable, name, value)?,
                Statement::Method(method) => {
                    let selector = self.selector(&method.selector);
                    // IRIS-V1-RUNTIME-C024 forbids an overload set: one complete
                    // selector maps to at most one Method per revision, and a
                    // redefinition needs `override`. An origin declaration passes
                    // `requires_override` as false, so a duplicate inside ONE body
                    // would otherwise publish silently with the second body winning.
                    let duplicate = declared_in_body
                        .iter()
                        .any(|(kind, seen)| *kind == method.kind && *seen == selector);
                    self.class_method(class, builtin, declaration.reopen || duplicate, method)?;
                    declared_in_body.push((method.kind, selector));
                }
                _ => return Err(EvaluationError::UnsupportedConstruct),
            }
        }
        Ok(())
    }

    fn shared_binding(
        &mut self,
        class: ClassId,
        mutable: bool,
        name: &str,
        value: &Expression,
    ) -> Result<(), EvaluationError> {
        let selector = self.selector(name);
        if self.class_var_owner_from(class, selector).is_ok() {
            return Err(EvaluationError::Class(
                iris_runtime::ClassError::DuplicateClassVariable {
                    class,
                    name: selector,
                },
            ));
        }
        let value = self.expression(value, &HashMap::new(), None)?;
        self.runtime
            .declare_class_var(class, selector, value, mutable)
            .map_err(EvaluationError::Construction)?;
        Ok(())
    }

    fn class_method(
        &mut self,
        class: ClassId,
        builtin: bool,
        requires_override: bool,
        method: &MethodDeclaration,
    ) -> Result<(), EvaluationError> {
        let selector = self.selector(&method.selector);
        // IRIS-V1-TYPES-C048: `impl C::member` qualifies a slot in the
        // Contract-qualified namespace. C049 keeps that namespace separate from
        // ordinary dispatch, so this must NOT publish into the ordinary table.
        if let Some(Some(contract)) = &method.impl_contract {
            let contract = *self
                .contract_names
                .get(contract)
                .ok_or(EvaluationError::UnsupportedConstruct)?;
            if !self
                .class_contracts
                .get(&class)
                .is_some_and(|declared| declared.contains(&contract))
            {
                return Err(EvaluationError::UnsupportedConstruct);
            }
            let body = self.register_body(method.clone());
            let qualified = Method::new(
                iris_runtime::MethodId::new(self.next_body),
                iris_runtime::MethodOwner::Class(class),
                selector,
                body,
                iris_runtime::Visibility::Public,
            );
            self.next_body += 1;
            self.qualified_methods
                .insert((class, contract, selector), qualified);
            return Ok(());
        }
        let replaces = self.replaces_method(class, builtin, method.kind, selector)?;
        if method.is_override && !replaces {
            return Err(EvaluationError::Class(ClassError::OverrideWithoutTarget {
                class,
                selector,
            }));
        }
        if requires_override && replaces && !method.is_override {
            return Err(EvaluationError::Class(ClassError::OverrideRequired {
                class,
                selector,
            }));
        }
        let body = self.register_body(method.clone());
        let decorators = self.decorator_transforms(&method.decorators);
        match method.kind {
            MethodKind::Instance => {
                let defined = self
                    .runtime
                    .registry_mut()
                    .publish_decorated_method(class, selector, body, visibility(method), decorators)
                    .map_err(EvaluationError::Class)?;
                self.property_methods.insert(defined.id(), false);
            }
            MethodKind::Class => {
                self.runtime
                    .registry_mut()
                    .publish_singleton_method(class, selector, body, visibility(method))
                    .map_err(EvaluationError::Class)?;
            }
            MethodKind::Property => {
                let method_visibility = visibility(method);
                let defined = if builtin && self.builtin_class_property(&method.selector) {
                    self.runtime
                        .registry_mut()
                        .publish_singleton_method(class, selector, body, method_visibility)
                        .map_err(EvaluationError::Class)?
                } else {
                    self.runtime
                        .registry_mut()
                        .publish_decorated_method(
                            class,
                            selector,
                            body,
                            method_visibility,
                            decorators,
                        )
                        .map_err(EvaluationError::Class)?
                };
                self.property_methods.insert(defined.id(), true);
            }
            MethodKind::Module => return Err(EvaluationError::UnsupportedConstruct),
        }
        Ok(())
    }

    fn replaces_method(
        &mut self,
        class: ClassId,
        builtin: bool,
        kind: MethodKind,
        selector: Selector,
    ) -> Result<bool, EvaluationError> {
        let result = match (builtin, kind) {
            (true, MethodKind::Class) => self
                .runtime
                .registry_mut()
                .dispatch_class_object(class, selector),
            (false, MethodKind::Class) => self
                .runtime
                .registry()
                .dispatch_class_object(class, selector),
            (true, MethodKind::Instance | MethodKind::Property) => {
                self.runtime.registry_mut().dispatch(class, selector)
            }
            (false, MethodKind::Instance | MethodKind::Property) => {
                self.runtime.registry().dispatch(class, selector)
            }
            (_, MethodKind::Module) => return Err(EvaluationError::UnsupportedConstruct),
        };
        match result {
            Ok(DispatchOutcome::Invoke(_)) | Err(DispatchError::VisibilityDenied { .. }) => {
                Ok(true)
            }
            Ok(DispatchOutcome::WouldInvokeMethodMissing { .. }) => Ok(false),
            Err(DispatchError::Class(error)) => Err(EvaluationError::Class(error)),
            Err(_) => Err(EvaluationError::UnsupportedConstruct),
        }
    }

    fn stored_property(
        &mut self,
        class: ClassId,
        builtin: bool,
        decorators: &[iris_syntax::Decorator],
        name: &str,
        initializer: Expression,
    ) -> Result<(), EvaluationError> {
        let getter = MethodDeclaration {
            decorators: Vec::new(),
            is_override: false,
            impl_contract: None,
            kind: MethodKind::Property,
            selector: name.into(),
            parameters: Vec::new(),
            visibility: iris_syntax::Visibility::Public,
            body: vec![Statement::Expression(Expression::RawIvar(format!(
                "@{name}"
            )))],
        };
        let setter = MethodDeclaration {
            decorators: Vec::new(),
            is_override: false,
            impl_contract: None,
            kind: MethodKind::Property,
            selector: format!("{name}="),
            parameters: vec![iris_syntax::Parameter {
                name: "value".into(),
                category: iris_syntax::ParameterCategory::Positional,
                default: None,
            }],
            visibility: iris_syntax::Visibility::Public,
            body: vec![Statement::Expression(Expression::Assignment {
                left: Box::new(Expression::RawIvar(format!("@{name}"))),
                operator: iris_syntax::AssignmentOperator::Assign,
                right: Box::new(Expression::Name("value".into())),
            })],
        };
        let initializer = MethodDeclaration {
            decorators: Vec::new(),
            is_override: false,
            impl_contract: None,
            kind: MethodKind::Property,
            selector: name.into(),
            parameters: Vec::new(),
            visibility: iris_syntax::Visibility::Private,
            body: vec![Statement::Expression(Expression::Assignment {
                left: Box::new(Expression::RawIvar(format!("@{name}"))),
                operator: iris_syntax::AssignmentOperator::Assign,
                right: Box::new(initializer),
            })],
        };
        self.class_method(class, builtin, false, &getter)?;
        self.class_method(class, builtin, false, &setter)?;
        let body = self.register_body(initializer);
        let property = self.selector(&format!("@{name}"));
        let decorators = self.decorator_transforms(decorators);
        self.runtime
            .registry_mut()
            .publish_decorated_stored_property(class, property, body, decorators)
            .map_err(EvaluationError::Class)
    }

    fn publish_decorators(
        &mut self,
        class: ClassId,
        decorators: &[iris_syntax::Decorator],
    ) -> Result<(), EvaluationError> {
        if decorators.is_empty() {
            return Ok(());
        }
        let candidate = self
            .runtime
            .registry_mut()
            .open(class)
            .map_err(EvaluationError::Class)?;
        let transforms = self.decorator_transforms(decorators);
        self.runtime
            .registry_mut()
            .publish_decorated(candidate, transforms)
            .map_err(EvaluationError::Class)?;
        Ok(())
    }

    fn decorator_transforms(
        &self,
        decorators: &[iris_syntax::Decorator],
    ) -> Vec<iris_runtime::DecoratorTransform> {
        decorators
            .iter()
            .map(|decorator| {
                iris_runtime::DecoratorTransform::metadata(
                    decorator.name.clone(),
                    decorator
                        .arguments
                        .iter()
                        .map(|argument| format!("{argument:?}")),
                )
            })
            .collect()
    }

    /// Registers a declared Contract as an object with its own identity.
    ///
    /// `IRIS-V1-TYPES-C043` forms Contract inheritance as the union of static
    /// requirements with no implementation MRO, `super`, stored state, or Method
    /// bodies, so parents are recorded as a plain relation rather than composed
    /// the way a Module is.
    fn contract(
        &mut self,
        declaration: &iris_syntax::ContractDeclaration,
    ) -> Result<(), EvaluationError> {
        let mut parents = Vec::new();
        for parent in &declaration.parents {
            let iris_syntax::TypeExpression::Name(name) = parent else {
                return Err(EvaluationError::UnsupportedConstruct);
            };
            parents.push(
                *self
                    .contract_names
                    .get(name)
                    .ok_or(EvaluationError::UnsupportedConstruct)?,
            );
        }
        let contract = iris_runtime::ContractId::new(self.next_contract);
        self.next_contract += 1;
        self.contract_names
            .insert(declaration.name.clone(), contract);
        self.contract_parents.insert(contract, parents);
        self.names.insert(
            declaration.name.clone(),
            Binding::immutable(Value::Contract(contract)),
        );
        Ok(())
    }

    fn module(&mut self, declaration: &ModuleDeclaration) -> Result<(), EvaluationError> {
        let mut components = Vec::new();
        for mixin in &declaration.mixins {
            match &mixin.target {
                iris_syntax::TypeExpression::Name(name) => components.push(CompositionEdge::new(
                    *self
                        .module_names
                        .get(name)
                        .ok_or(EvaluationError::UnsupportedConstruct)?,
                    mixin.private_access,
                )),
                _ => return Err(EvaluationError::UnsupportedConstruct),
            }
        }
        let capabilities = meta_capabilities(&declaration.meta_deny)?;
        let module = self
            .runtime
            .registry_mut()
            .define_module_with_composition_edges(&components, capabilities)
            .map_err(EvaluationError::Class)?;
        self.module_names.insert(declaration.name.clone(), module);
        let module_class = self
            .runtime
            .registry_mut()
            .define_class(StaticSpine::new(1), None)
            .map_err(EvaluationError::Class)?;
        self.module_classes.insert(module, module_class);
        for statement in &declaration.body {
            match statement {
                Statement::SharedBinding {
                    mutable,
                    name,
                    value,
                } => self.shared_binding(module_class, *mutable, name, value)?,
                Statement::Method(method) => {
                    if method.kind == MethodKind::Class || method.kind == MethodKind::Property {
                        return Err(EvaluationError::UnsupportedConstruct);
                    }
                    let body = self.register_body(method.clone());
                    let selector = self.selector(&method.selector);
                    let defined = self
                        .runtime
                        .registry_mut()
                        .define_module_method(module, selector, body, visibility(method))
                        .map_err(EvaluationError::Class)?;
                    self.module_methods.insert((module, selector), defined);
                    self.module_method_overrides
                        .insert(defined.id(), method.is_override);
                }
                _ => return Err(EvaluationError::UnsupportedConstruct),
            }
        }
        Ok(())
    }

    fn statement(
        &mut self,
        statement: &Statement,
        locals: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<Value, EvaluationError> {
        match statement {
            Statement::Binding {
                mutable,
                name,
                value,
            } => {
                let value = self.expression(value, locals, receiver)?;
                self.names
                    .insert(name.clone(), Binding::new(value.clone(), *mutable));
                Ok(value)
            }
            Statement::Expression(expression) => self.expression(expression, locals, receiver),
            Statement::If {
                condition,
                then_body,
                else_body,
            } => {
                if self.condition(condition, locals, receiver.clone())? {
                    self.block(then_body, locals, receiver)
                } else if let Some(else_body) = else_body {
                    self.block(else_body, locals, receiver)
                } else {
                    Ok(Value::Nil)
                }
            }
            Statement::Raise(raise) => {
                let value = match raise {
                    Some(raise) => self.expression(&raise.value, locals, receiver.clone())?,
                    None => {
                        // IRIS-V1-CONTROL-C058: a bare `raise` continues the
                        // CURRENT propagation, so it keeps the active context
                        // rather than creating a fresh one. Outside a catch
                        // extent there is nothing to continue.
                        return Err(self.active_exception.clone().map_or(
                            EvaluationError::NoActiveExceptionError,
                            EvaluationError::Raised,
                        ));
                    }
                };
                // IRIS-V1-CONTROL-C057: an explicit cause MUST be an
                // ExceptionContext, and `from nil` suppresses chaining.
                let cause = match raise.as_ref().and_then(|raise| raise.cause.as_ref()) {
                    Some(cause) => {
                        let cause = self.expression(cause, locals, receiver)?;
                        match cause {
                            Value::Nil => cause,
                            Value::ExceptionContext(..) => {
                                // D-161: cause edges may not form a cycle. The
                                // check runs BEFORE linkage, so a rejected
                                // attempt leaves the existing graph unchanged.
                                if Self::context_reaches(&cause, &value) {
                                    return Err(EvaluationError::ExceptionChainError);
                                }
                                cause
                            }
                            _ => {
                                return Err(EvaluationError::Runtime(
                                    iris_runtime::KernelError::Type,
                                ));
                            }
                        }
                    }
                    None => Value::Nil,
                };
                self.active_context = Some(Value::ExceptionContext(
                    Box::new(value.clone()),
                    Box::new(cause),
                    Vec::new(),
                ));
                Err(EvaluationError::Raised(value))
            }
            Statement::Try {
                body,
                catches,
                finally,
            } => self.try_statement(body, catches, finally, locals, receiver),
            Statement::Break { label, value } => {
                let value = match value {
                    Some(value) => self.expression(value, locals, receiver)?,
                    None => Value::Nil,
                };
                Err(EvaluationError::LoopBreak(label.clone(), value))
            }
            Statement::While {
                label,
                condition,
                body,
            } => self.while_statement(label.as_deref(), condition, body, locals, receiver),
            Statement::For {
                label,
                binding,
                iterable,
                body,
            } => self.for_statement(label.as_deref(), binding, iterable, body, locals, receiver),
            Statement::Continue(label) => Err(EvaluationError::LoopContinue(label.clone())),
            Statement::Match {
                subject,
                arms,
                fallback,
            } => self.match_statement(subject, arms, fallback.as_ref(), locals, receiver),
            Statement::SharedBinding { .. }
            | Statement::StoredProperty { .. }
            | Statement::Method(_)
            | Statement::Return(_) => Err(EvaluationError::UnsupportedConstruct),
        }
    }

    /// Runs `match value { arms }` per `IRIS-V1-CONTROL-C050`.
    ///
    /// The scrutinee is evaluated ONCE and arms are tested in source order, with
    /// no fallthrough: the first match produces the result. A guard runs only
    /// after structural success and after provisional bindings exist, per
    /// `IRIS-V1-CONTROL-C052`, and a false guard discards those bindings and
    /// continues with the next arm.
    fn match_statement(
        &mut self,
        subject: &Expression,
        arms: &[iris_syntax::MatchArm],
        fallback: Option<&iris_syntax::MatchBody>,
        locals: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<Value, EvaluationError> {
        let value = self.expression(subject, locals, receiver.clone())?;
        for arm in arms {
            let mut bound = locals.clone();
            if !self.pattern_matches(&arm.pattern, &value, &mut bound)? {
                continue;
            }
            if let Some(guard) = &arm.guard {
                let test = self.expression(guard, &bound, receiver.clone())?;
                if !self.truthy(test)? {
                    continue;
                }
            }
            return self.match_body(&arm.body, &bound, receiver);
        }
        match fallback {
            Some(body) => self.match_body(body, locals, receiver),
            None => Err(EvaluationError::UnsupportedConstruct),
        }
    }

    fn match_body(
        &mut self,
        body: &iris_syntax::MatchBody,
        locals: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<Value, EvaluationError> {
        match body {
            iris_syntax::MatchBody::Expression(expression) => {
                self.expression(expression, locals, receiver)
            }
            iris_syntax::MatchBody::Block(statements) => self.block(statements, locals, receiver),
        }
    }

    /// Tests one pattern from the `IRIS-V1-CONTROL-C051` vocabulary.
    ///
    /// `_` discards and creates no binding per `IRIS-V1-CONTROL-C053`, a binding
    /// pattern always matches and binds, and alternatives succeed on the first
    /// alternative that matches.
    fn pattern_matches(
        &mut self,
        pattern: &iris_syntax::Pattern,
        value: &Value,
        bound: &mut HashMap<String, Value>,
    ) -> Result<bool, EvaluationError> {
        match pattern {
            iris_syntax::Pattern::Name(name) if name == "_" => Ok(true),
            iris_syntax::Pattern::Name(name) => {
                bound.insert(name.clone(), value.clone());
                Ok(true)
            }
            iris_syntax::Pattern::Literal(literal) => {
                let expected =
                    self.expression(&Expression::Literal(literal.clone()), &HashMap::new(), None)?;
                Ok(expected == *value)
            }
            iris_syntax::Pattern::Array(elements) => {
                let Value::Array(values) = value else {
                    return Ok(false);
                };
                if values.len() != elements.len() {
                    return Ok(false);
                }
                for (element, value) in elements.iter().zip(values) {
                    if !self.pattern_matches(element, value, bound)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            iris_syntax::Pattern::Alternatives(alternatives) => {
                for alternative in alternatives {
                    if self.pattern_matches(alternative, value, bound)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
    }

    /// Runs `for pattern in iterable { body }` per `IRIS-V1-CONTROL-C044`.
    ///
    /// The iterable is evaluated ONCE, `next()` is called repeatedly, and the
    /// loop exits normally on `Iteration.done`. Each yielded value binds in a
    /// FRESH per-iteration scope, which is what lets a Closure created in the
    /// body capture that iteration's value rather than a shared cell.
    /// `IRIS-V1-CONTROL-C046` requires the Iterator to close on every exit path,
    /// so `close` is sent on natural exhaustion, on `break`, and on an error.
    fn for_statement(
        &mut self,
        label: Option<&str>,
        binding: &iris_syntax::Pattern,
        iterable: &Expression,
        body: &[Statement],
        locals: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<Value, EvaluationError> {
        let source = self.expression(iterable, locals, receiver.clone())?;
        let iterator = self.send(source, "iterator", &[])?;
        let outcome =
            self.for_iterations(label, binding, &iterator, body, locals, receiver.clone());
        let closed = self.send(iterator, "close", &[]);
        match (outcome, closed) {
            (Ok(value), Ok(_)) => Ok(value),
            (Ok(_), Err(error)) => Err(error),
            // IRIS-V1-CONTROL-C047: when `close` fails during cleanup while a
            // primary context exists, the close failure is APPENDED to that
            // context's suppressed list in occurrence order rather than
            // replacing the primary propagation.
            (Err(primary), Err(EvaluationError::Raised(cleanup))) => {
                if let Some(Value::ExceptionContext(value, cause, suppressed)) =
                    self.active_context.take()
                {
                    let mut suppressed = suppressed;
                    suppressed.push(Value::ExceptionContext(
                        Box::new(cleanup),
                        Box::new(Value::Nil),
                        Vec::new(),
                    ));
                    self.active_context = Some(Value::ExceptionContext(value, cause, suppressed));
                }
                Err(primary)
            }
            (Err(primary), _) => Err(primary),
        }
    }

    fn for_iterations(
        &mut self,
        label: Option<&str>,
        binding: &iris_syntax::Pattern,
        iterator: &Value,
        body: &[Statement],
        locals: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<Value, EvaluationError> {
        loop {
            let step = self.send(iterator.clone(), "next", &[])?;
            let value = match step {
                Value::IterationDone => return Ok(Value::Nil),
                Value::IterationYield(value) => *value,
                _ => return Err(EvaluationError::UnsupportedConstruct),
            };
            let mut iteration = locals.clone();
            // IRIS-V1-CONTROL-C045: a destructuring mismatch raises
            // PatternMatchError. The caller closes the Iterator before this
            // propagates, which is what C045 requires of every exit path.
            if !self.pattern_matches(binding, &value, &mut iteration)? {
                return Err(EvaluationError::PatternMatchError);
            }
            match self.block(body, &iteration, receiver.clone()) {
                Ok(_) => {}
                Err(EvaluationError::LoopBreak(target, value))
                    if target.is_none() || target.as_deref() == label =>
                {
                    return Ok(value);
                }
                Err(EvaluationError::LoopContinue(target))
                    if target.is_none() || target.as_deref() == label => {}
                Err(error) => return Err(error),
            }
        }
    }

    /// Runs `while condition { body }` per `IRIS-V1-CONTROL-C043`.
    ///
    /// The condition is truth-tested BEFORE each iteration, so zero iterations
    /// are possible, and natural completion yields `nil`. A `break` unwinds as a
    /// control signal: this loop consumes one that targets it, either unlabelled
    /// or naming its own label, and lets any other keep unwinding to an outer
    /// loop.
    fn while_statement(
        &mut self,
        label: Option<&str>,
        condition: &Expression,
        body: &[Statement],
        locals: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<Value, EvaluationError> {
        loop {
            let test = self.expression(condition, locals, receiver.clone())?;
            if !self.truthy(test)? {
                return Ok(Value::Nil);
            }
            match self.block(body, locals, receiver.clone()) {
                Ok(_) => {}
                Err(EvaluationError::LoopBreak(target, value))
                    if target.is_none() || target.as_deref() == label =>
                {
                    return Ok(value);
                }
                Err(EvaluationError::LoopContinue(target))
                    if target.is_none() || target.as_deref() == label => {}
                Err(error) => return Err(error),
            }
        }
    }

    fn block(
        &mut self,
        statements: &[Statement],
        parent: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<Value, EvaluationError> {
        let mut locals = parent.clone();
        let mut result = Value::Nil;
        for statement in statements {
            match statement {
                Statement::Binding { name, value, .. } => {
                    let value = self.expression(value, &locals, receiver.clone())?;
                    locals.insert(name.clone(), value);
                }
                Statement::Expression(_) | Statement::If { .. } | Statement::Try { .. } => {
                    result = self.statement(statement, &locals, receiver.clone())?;
                }
                Statement::Raise(_) => return self.statement(statement, &locals, receiver.clone()),
                // A `break` leaves the block immediately, carrying its loop
                // result outward as the control signal the target loop consumes.
                Statement::Break { .. } | Statement::Continue(_) => {
                    return self.statement(statement, &locals, receiver.clone());
                }
                Statement::While { .. } | Statement::For { .. } => {
                    result = self.statement(statement, &locals, receiver.clone())?;
                }
                Statement::SharedBinding { .. }
                | Statement::StoredProperty { .. }
                | Statement::Method(_)
                | Statement::Return(_) => return Err(EvaluationError::UnsupportedConstruct),
                Statement::Match { .. } => {
                    result = self.statement(statement, &locals, receiver.clone())?;
                }
            }
        }
        Ok(result)
    }

    fn try_statement(
        &mut self,
        body: &[Statement],
        catches: &[iris_syntax::CatchClause],
        finally: &Option<Vec<Statement>>,
        locals: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<Value, EvaluationError> {
        let result = match self.block(body, locals, receiver.clone()) {
            Ok(value) => Ok(value),
            Err(EvaluationError::Raised(value)) => {
                self.catch_exception(value, catches, locals, receiver.clone())
            }
            Err(error) => Err(error),
        };
        if let Some(finally) = finally {
            let pending = match &result {
                Err(EvaluationError::Raised(value)) => Some(value.clone()),
                Ok(_) | Err(_) => None,
            };
            let previous = self.active_exception.clone();
            let pending_context = pending
                .is_some()
                .then(|| self.active_context.clone())
                .flatten();
            self.active_exception = pending;
            let final_result = self.block(finally, locals, receiver);
            self.active_exception = previous;
            if let Err(EvaluationError::Raised(raised)) = &final_result {
                // IRIS-V1-CONTROL-C064: a `finally` that raises while a context
                // is pending becomes PRIMARY, and the pending context becomes
                // its cause. Without this the new context would simply overwrite
                // the old one and the chain would be lost.
                if let Some(cause) = pending_context {
                    self.active_context = Some(Value::ExceptionContext(
                        Box::new(raised.clone()),
                        Box::new(cause),
                        Vec::new(),
                    ));
                }
            }
            final_result?;
        }
        result
    }

    /// Reports whether a context chain already reaches a raised value.
    ///
    /// `D-161` performs an identity-based cycle check across cause edges, so a
    /// context whose chain already carries this value cannot become its cause.
    fn context_reaches(context: &Value, value: &Value) -> bool {
        let mut current = context;
        loop {
            let Value::ExceptionContext(carried, cause, _) = current else {
                return false;
            };
            if **carried == *value {
                return true;
            }
            current = cause;
        }
    }

    fn catch_exception(
        &mut self,
        value: Value,
        catches: &[iris_syntax::CatchClause],
        locals: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<Value, EvaluationError> {
        for catch in catches {
            if !catch
                .filter
                .as_ref()
                .is_none_or(|filter| self.catch_matches(&value, filter))
            {
                continue;
            }
            let mut catch_locals = locals.clone();
            if let Some(iris_syntax::CatchBinding::Name(name)) = &catch.binding {
                catch_locals.insert(name.clone(), value.clone());
            }
            // IRIS-V1-CONTROL-C060 binds the ExceptionContext alongside the
            // raised object. C056 gives every propagation a fresh context, and
            // an unchained one carries a nil cause per C057.
            if let Some(context) = &catch.context {
                let context_value = self.active_context.clone().unwrap_or_else(|| {
                    Value::ExceptionContext(
                        Box::new(value.clone()),
                        Box::new(Value::Nil),
                        Vec::new(),
                    )
                });
                catch_locals.insert(context.clone(), context_value);
            }
            let previous = self.active_exception.replace(value.clone());
            let result = self.block(&catch.body, &catch_locals, receiver);
            self.active_exception = previous;
            return result;
        }
        Err(EvaluationError::Raised(value))
    }

    fn catch_matches(&self, value: &Value, filter: &iris_syntax::TypeExpression) -> bool {
        match filter {
            iris_syntax::TypeExpression::Name(name) => match (name.as_str(), value) {
                ("Symbol", Value::Symbol(_))
                | ("Integer", Value::Integer(_))
                | ("Nil", Value::Nil)
                | ("Bool", Value::Bool(_)) => true,
                (_, Value::Object(object)) => self
                    .class_name(name)
                    .ok()
                    .flatten()
                    .zip(self.runtime.class_of(*object).ok())
                    .is_some_and(|(filter, mut class)| {
                        loop {
                            if class == filter {
                                break true;
                            }
                            let Some(superclass) =
                                self.static_superclasses.get(&class).copied().flatten()
                            else {
                                break false;
                            };
                            class = superclass;
                        }
                    }),
                _ => false,
            },
            iris_syntax::TypeExpression::Typeof(_)
            | iris_syntax::TypeExpression::Function { .. }
            | iris_syntax::TypeExpression::Intersection(_)
            | iris_syntax::TypeExpression::Union(_)
            | iris_syntax::TypeExpression::Generic { .. } => false,
        }
    }

    fn condition(
        &mut self,
        expression: &Expression,
        locals: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<bool, EvaluationError> {
        let value = self.expression(expression, locals, receiver)?;
        self.truthy(value)
    }

    fn truthy(&mut self, value: Value) -> Result<bool, EvaluationError> {
        let result = self.send(value.clone(), "to_bool", &[])?;
        let method = TruthinessMethod::Returns(result);
        Truthiness::test(&value, method).map_err(|error| match error {
            TruthinessError::TypeContract => EvaluationError::TypeContractError,
            TruthinessError::Raised(value) => {
                EvaluationError::Execution(iris_runtime::ExecutionError::Raised(value))
            }
        })
    }

    fn expression(
        &mut self,
        expression: &Expression,
        locals: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<Value, EvaluationError> {
        match expression {
            // IRIS-V1-CONTROL-C026 evaluates a keyword argument in place with
            // the positionals, so the value is produced here and the name is
            // carried to the binding step.
            Expression::KeywordArgument { name, value } => {
                let value = self.expression(value, locals, receiver)?;
                Ok(Value::KeywordArgument(name.clone(), Box::new(value)))
            }
            Expression::Hash(entries) => {
                // IRIS-V1-RUNTIME-C134: Hash CONSTRUCTION with a NaN key of
                // either width must raise InvalidKeyError, so every key is
                // hashed here rather than only on later insertion.
                let mut built: Vec<(Value, Value)> = Vec::new();
                for (key, value) in entries {
                    let key = self.expression(key, locals, receiver.clone())?;
                    self.send(key.clone(), "hash", &[])?;
                    let value = self.expression(value, locals, receiver.clone())?;
                    // IRIS-V1-COLLECTIONS-C028 dispatches the key's current
                    // `==`, so a repeated key UPDATES its entry rather than
                    // adding a second one.
                    match built.iter_mut().find(|(seen, _)| *seen == key) {
                        Some(entry) => entry.1 = value,
                        None => built.push((key, value)),
                    }
                }
                Ok(Value::Hash(built))
            }
            Expression::Closure { parameters, body } => {
                // IRIS-V1-RUNTIME-C042: every evaluation allocates a NEW Closure
                // with its own captured environment, so this never caches.
                let object = iris_runtime::ObjectId::new(self.next_closure);
                self.next_closure += 1;
                self.closures.insert(
                    object,
                    ClosureRecord {
                        parameters: parameters.clone(),
                        body: body.clone(),
                        captured: locals.clone(),
                        receiver: receiver.clone(),
                    },
                );
                Ok(Value::Closure(object))
            }
            Expression::Name(name) if name == "super" => Err(EvaluationError::Runtime(
                iris_runtime::KernelError::Dispatch(iris_runtime::DispatchError::InvalidSuper {
                    selector: self
                        .current_method
                        .ok_or(EvaluationError::UnsupportedConstruct)?
                        .selector(),
                }),
            )),
            Expression::Name(name) => locals
                .get(name)
                .cloned()
                .or_else(|| self.names.get(name).map(Binding::value))
                .or_else(|| (name == "self").then_some(receiver).flatten())
                .or_else(|| builtin(name, &self.kernel))
                .or_else(|| {
                    self.module_names
                        .contains_key(name)
                        .then(|| Value::Symbol(name.clone()))
                })
                .ok_or(EvaluationError::UnsupportedConstruct),
            Expression::RawIvar(name) => {
                let selector = self.selector(name);
                match receiver.ok_or(EvaluationError::UnsupportedConstruct)? {
                    Value::Object(object) => self
                        .runtime
                        .raw_ivar(object, selector)
                        .map_err(EvaluationError::Construction),
                    Value::Class(class) => self
                        .runtime
                        .class_raw_ivar(class, selector)
                        .map_err(EvaluationError::Construction),
                    Value::Nil
                    | Value::Bool(_)
                    | Value::Integer(_)
                    | Value::Float32(_)
                    | Value::Float64(_) => Ok(Value::Nil),
                    _ => Err(EvaluationError::UnsupportedConstruct),
                }
            }
            Expression::ClassVar(name) => self.read_class_var(name),
            Expression::Literal(source) => literal(source),
            Expression::Symbol(symbol) => Ok(Value::Symbol(symbol.clone())),
            Expression::Grouped(expression) => self.expression(expression, locals, receiver),
            Expression::Array(values) => values
                .iter()
                .map(|value| self.expression(value, locals, receiver.clone()))
                .collect::<Result<Vec<_>, _>>()
                .map(Value::Array),
            Expression::If {
                condition,
                then_body,
                else_body,
            } => {
                if self.condition(condition, locals, receiver.clone())? {
                    self.block(then_body, locals, receiver)
                } else if let Some(else_body) = else_body {
                    self.block(else_body, locals, receiver)
                } else {
                    Ok(Value::Nil)
                }
            }
            Expression::Member {
                receiver: target,
                selector,
            } if matches!(target.as_ref(), Expression::Name(name) if name == "Iteration")
                && selector == "done" =>
            {
                Ok(Value::IterationDone)
            }
            Expression::Member {
                receiver: target,
                selector,
            } => {
                let target = self.expression(target, locals, receiver)?;
                self.member_read(target, selector)
            }
            Expression::Call { callee, arguments } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| self.expression(argument, locals, receiver.clone()))
                    .collect::<Result<Vec<_>, _>>()?;
                match callee.as_ref() {
                    Expression::ContractView {
                        receiver: target,
                        selector,
                    } => {
                        let view = self.expression(target, locals, receiver.clone())?;
                        self.qualified_send(view, selector, &arguments)
                    }
                    // `Iteration.yield(value)` and `Iteration.done` are the
                    // IRIS-V1-COLLECTIONS-C013 iteration results, not Class
                    // sends, so they are built here rather than dispatched.
                    Expression::Member {
                        receiver: target,
                        selector,
                    } if matches!(target.as_ref(), Expression::Name(name) if name == "Iteration")
                        && selector == "yield" =>
                    {
                        let [value] = arguments.as_slice() else {
                            return Err(EvaluationError::ArgumentError);
                        };
                        Ok(Value::IterationYield(Box::new(value.clone())))
                    }
                    Expression::Member {
                        receiver: target,
                        selector,
                    } if matches!(target.as_ref(), Expression::Name(name) if name.starts_with("Reflection::")) =>
                    {
                        let Expression::Name(namespace) = target.as_ref() else {
                            return Err(EvaluationError::UnsupportedConstruct);
                        };
                        self.reflection(namespace, selector, &arguments)
                    }
                    Expression::Member {
                        receiver: target,
                        selector,
                    } if matches!(selector.as_str(), "method" | "remove_module")
                        && !matches!(target.as_ref(), Expression::Name(name) if self.module_names.contains_key(name)) =>
                    {
                        let target = self.expression(target, locals, receiver)?;
                        self.send(target, selector, &arguments)
                    }
                    Expression::Member {
                        receiver: target,
                        selector,
                    } if selector == "append" => {
                        self.append_array_binding(target, &arguments, locals)
                    }
                    Expression::Name(name) if name == "super" => {
                        self.super_send(receiver, &arguments, None)
                    }
                    Expression::Member {
                        receiver: target,
                        selector,
                    } if matches!(target.as_ref(), Expression::Name(name) if name == "super") => {
                        self.super_send(receiver, &arguments, Some(selector))
                    }
                    Expression::Member {
                        receiver: target,
                        selector,
                    } if matches!(target.as_ref(), Expression::Name(name) if self.module_names.contains_key(name)) =>
                    {
                        let Expression::Name(name) = target.as_ref() else {
                            return Err(EvaluationError::UnsupportedConstruct);
                        };
                        let module = *self
                            .module_names
                            .get(name)
                            .ok_or(EvaluationError::UnsupportedConstruct)?;
                        if selector == "method" {
                            let [Value::Symbol(name)] = arguments.as_slice() else {
                                return Err(EvaluationError::UnsupportedConstruct);
                            };
                            let selector = self.selector(name);
                            return Ok(self
                                .module_methods
                                .get(&(module, selector))
                                .copied()
                                .map(Value::Method)
                                .unwrap_or(Value::Nil));
                        }
                        if selector == "invoke" {
                            let [Value::Method(method), receiver, Value::Array(args)] =
                                arguments.as_slice()
                            else {
                                return Err(EvaluationError::UnsupportedConstruct);
                            };
                            return self.reflective_invoke(*method, receiver.clone(), args);
                        }
                        let selector_name = selector.clone();
                        let selector = self.selector(&selector_name);
                        let method = self
                            .module_methods
                            .get(&(module, selector))
                            .copied()
                            .ok_or(EvaluationError::MessageNotFound {
                                receiver_class: "Module".into(),
                                selector: selector_name,
                            })?;
                        self.invoke_method(method, Value::Symbol(name.clone()), &arguments)
                    }
                    Expression::Member {
                        receiver: target,
                        selector,
                    } => {
                        let target = self.expression(target, locals, receiver)?;
                        self.send(target, selector, &arguments)
                    }
                    // A local binding shadows a self-send: `b()` where `b` is a
                    // parameter invokes that value rather than sending `b` to
                    // self. This must cover EVERY local, not just callable ones,
                    // because falling through for a nil block would send `b` to
                    // self, reach method_missing, and re-evaluate `b()` forever.
                    Expression::Name(selector) if locals.contains_key(selector) => {
                        let callee = locals
                            .get(selector)
                            .cloned()
                            .ok_or(EvaluationError::UnsupportedConstruct)?;
                        self.call(callee, &arguments)
                    }
                    Expression::Name(selector) => match receiver {
                        Some(receiver) => self.send(receiver, selector, &arguments),
                        None => self
                            .expression(callee, locals, None)
                            .and_then(|value| self.call(value, &arguments)),
                    },
                    _ => Err(EvaluationError::UnsupportedConstruct),
                }
            }
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.expression(left, locals, receiver.clone())?;
                if matches!(operator, BinaryOperator::LogicalAnd) {
                    return if self.truthy(left.clone())? {
                        self.expression(right, locals, receiver)
                    } else {
                        Ok(left)
                    };
                }
                if matches!(operator, BinaryOperator::LogicalOr) {
                    return if self.truthy(left.clone())? {
                        Ok(left)
                    } else {
                        self.expression(right, locals, receiver)
                    };
                }
                // IRIS-V1-TYPES-C014: entering `Dynamic<T>` checks the value
                // satisfies reified `T` and then disables static member checking
                // INSIDE the bound. It yields the same value, so it is resolved
                // before the right side is evaluated as an ordinary name.
                if matches!(operator, BinaryOperator::As)
                    && matches!(right.as_ref(), Expression::Name(name) if name == "Dynamic")
                {
                    return Ok(left);
                }
                let right = self.expression(right, locals, receiver)?;
                let selector = match operator {
                    BinaryOperator::Power => "**",
                    BinaryOperator::Multiply => "*",
                    BinaryOperator::Divide => "/",
                    BinaryOperator::Add => "+",
                    BinaryOperator::Subtract => "-",
                    BinaryOperator::ShiftLeft => "<<",
                    BinaryOperator::ShiftRight => ">>",
                    BinaryOperator::BitwiseAnd => "&",
                    BinaryOperator::BitwiseXor => "^",
                    BinaryOperator::BitwiseOr => "|",
                    BinaryOperator::Less => "<",
                    BinaryOperator::LessEqual => "<=",
                    BinaryOperator::Greater => ">",
                    BinaryOperator::GreaterEqual => ">=",
                    BinaryOperator::Compare => "<=>",
                    BinaryOperator::Equal => "==",
                    BinaryOperator::NotEqual => "!=",
                    BinaryOperator::NamedInfix { selector } => selector,
                    BinaryOperator::Identity => {
                        return Ok(Value::Bool(left == right));
                    }
                    BinaryOperator::Is => {
                        return self.type_test(&left, &right);
                    }
                    BinaryOperator::As => {
                        return self.contract_view(left, &right);
                    }
                    _ => return Err(EvaluationError::UnsupportedConstruct),
                };
                self.send(left, selector, &[right])
            }
            Expression::Unary { operator, operand } => match operator {
                iris_syntax::UnaryOperator::Not => self
                    .expression(operand, locals, receiver)
                    .and_then(|value| self.truthy(value))
                    .map(|value| Value::Bool(!value)),
                iris_syntax::UnaryOperator::Plus
                | iris_syntax::UnaryOperator::Negate
                | iris_syntax::UnaryOperator::BitwiseNot => {
                    Err(EvaluationError::UnsupportedConstruct)
                }
            },
            Expression::ContractView { .. } => Err(EvaluationError::UnsupportedConstruct),
            Expression::Assignment {
                left,
                operator,
                right,
            } => {
                if let Expression::Name(name) = left.as_ref() {
                    if locals.contains_key(name) {
                        return Err(EvaluationError::ImmutableBinding);
                    }
                    // IRIS-V1-CONTROL-C037: logical assignment reads the target
                    // ONCE, truth-tests it, and evaluates the right side only on
                    // the writing path. A `to_bool` failure must therefore
                    // prevent both the RHS and the write.
                    if let iris_syntax::AssignmentOperator::LogicalAnd
                    | iris_syntax::AssignmentOperator::LogicalOr = operator
                    {
                        let current = self
                            .names
                            .get(name)
                            .map(Binding::value)
                            .ok_or(EvaluationError::ImmutableBinding)?;
                        let truthy = self.truthy(current.clone())?;
                        let writes = match operator {
                            iris_syntax::AssignmentOperator::LogicalAnd => truthy,
                            _ => !truthy,
                        };
                        if !writes {
                            return Ok(current);
                        }
                        let value = self.expression(right, locals, receiver)?;
                        let binding = self
                            .names
                            .get_mut(name)
                            .ok_or(EvaluationError::ImmutableBinding)?;
                        return binding
                            .assign(value)
                            .map_err(|()| EvaluationError::ImmutableBinding);
                    }
                    let value = self.expression(right, locals, receiver)?;
                    let binding = self
                        .names
                        .get_mut(name)
                        .ok_or(EvaluationError::ImmutableBinding)?;
                    return binding
                        .assign(value)
                        .map_err(|()| EvaluationError::ImmutableBinding);
                }
                if let Expression::RawIvar(name) = left.as_ref() {
                    let selector = self.selector(name);
                    return match receiver.ok_or(EvaluationError::UnsupportedConstruct)? {
                        Value::Object(object) => {
                            let value =
                                self.expression(right, locals, Some(Value::Object(object)))?;
                            self.runtime
                                .assign_raw_ivar(object, selector, value)
                                .map_err(EvaluationError::Construction)
                        }
                        Value::Class(class) => {
                            let value =
                                self.expression(right, locals, Some(Value::Class(class)))?;
                            self.runtime
                                .assign_class_raw_ivar(class, selector, value)
                                .map_err(EvaluationError::Construction)
                        }
                        value @ (Value::Nil
                        | Value::Bool(_)
                        | Value::Integer(_)
                        | Value::Float32(_)
                        | Value::Float64(_)) => self.assign_value_raw_ivar(value),
                        _ => Err(EvaluationError::UnsupportedConstruct),
                    };
                }
                if let Expression::ClassVar(name) = left.as_ref() {
                    let value = self.expression(right, locals, receiver)?;
                    return self.assign_class_var(name, value);
                }
                let Expression::Member {
                    receiver: target,
                    selector,
                } = left.as_ref()
                else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let target = self.expression(target, locals, receiver.clone())?;
                let value = self.expression(right, locals, receiver)?;
                self.send(target, &format!("{selector}="), &[value])
            }
        }
    }

    fn call(&mut self, value: Value, arguments: &[Value]) -> Result<Value, EvaluationError> {
        match value {
            // IRIS-V1-CONTROL-C076 forbids invoking a callable by applying an
            // argument list to it, so direct application of a Closure is not a
            // call at all. Only the `call` selector reaches invoke_closure.
            Value::Closure(_) => Err(EvaluationError::UnsupportedConstruct),
            Value::Class(class) => match self.kernel.construct(class, arguments) {
                Ok(value) => Ok(value),
                Err(iris_runtime::KernelError::Type) => {
                    self.construct(class, arguments).map(Value::Object)
                }
                Err(error) => Err(EvaluationError::Runtime(error)),
            },
            Value::BoundMethod(bound) => self.invoke_method(
                self.validate_bound_method(bound)?,
                match bound.receiver() {
                    iris_runtime::BoundReceiver::Class(class) => Value::Class(class),
                    iris_runtime::BoundReceiver::Object(object) => Value::Object(object),
                },
                arguments,
            ),
            Value::Method(_) => Err(EvaluationError::UnsupportedConstruct),
            _ => Err(EvaluationError::UnsupportedConstruct),
        }
    }

    fn validate_bound_method(
        &self,
        bound: iris_runtime::BoundMethod,
    ) -> Result<Method, EvaluationError> {
        let receiver = match bound.receiver() {
            iris_runtime::BoundReceiver::Class(class) => class,
            iris_runtime::BoundReceiver::Object(object) => self
                .runtime
                .class_of(object)
                .map_err(EvaluationError::Construction)?,
        };
        self.runtime
            .registry()
            .validate_method_binding(receiver, bound.method())
            .map(|()| bound.method())
            .map_err(iris_runtime::ConstructionError::from)
            .map_err(EvaluationError::Construction)
    }

    fn construct(
        &mut self,
        class: ClassId,
        arguments: &[Value],
    ) -> Result<iris_runtime::ObjectId, EvaluationError> {
        let mut runtime = std::mem::take(&mut self.runtime);
        let mut invocation_error = None;
        let result = runtime.construct(class, arguments, |runtime, method, receiver, arguments| {
            let previous = std::mem::replace(&mut self.runtime, std::mem::take(runtime));
            let result = match self.invoke_method(method, Value::Object(receiver), arguments) {
                Ok(value) => Ok(value),
                Err(EvaluationError::Raised(value)) => {
                    Err(iris_runtime::ExecutionError::Raised(value))
                }
                Err(error) => {
                    invocation_error = Some(error);
                    Err(iris_runtime::ExecutionError::Raised(Value::Nil))
                }
            };
            *runtime = std::mem::replace(&mut self.runtime, previous);
            result
        });
        self.runtime = runtime;
        match invocation_error {
            Some(error) => Err(error),
            None => result.map_err(construction_error),
        }
    }

    fn send(
        &mut self,
        receiver: Value,
        selector: &str,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        match receiver {
            Value::Class(class) if selector == "new" => {
                self.construct(class, arguments).map(Value::Object)
            }
            Value::Class(class) if selector == "alias_method" => {
                let [Value::Symbol(alias), Value::Symbol(original)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let alias = self.selector(alias);
                let original = self.selector(original);
                self.runtime
                    .registry_mut()
                    .alias_method(class, alias, original)
                    .map(|()| Value::Nil)
                    .map_err(EvaluationError::Class)
            }
            Value::Class(class) if selector == "remove_method" => {
                let [Value::Symbol(selector)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let selector = self.selector(selector);
                self.runtime
                    .registry_mut()
                    .remove_method(class, selector)
                    .map(|()| Value::Nil)
                    .map_err(EvaluationError::Class)
            }
            Value::Class(class) if selector == "undef_method" => {
                let [Value::Symbol(selector)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let selector = self.selector(selector);
                self.runtime
                    .registry_mut()
                    .undef_method(class, selector)
                    .map(|()| Value::Nil)
                    .map_err(EvaluationError::Class)
            }
            Value::Class(class) if selector == "method" => {
                let [Value::Symbol(selector)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let selector = self.selector(selector);
                match self.runtime.registry().dispatch(class, selector) {
                    Ok(iris_runtime::DispatchOutcome::Invoke(method)) => Ok(Value::Method(method)),
                    Ok(iris_runtime::DispatchOutcome::WouldInvokeMethodMissing { .. }) => {
                        Ok(Value::Nil)
                    }
                    Err(error) => Err(EvaluationError::Construction(error.into())),
                }
            }
            Value::Class(_) if selector == "invoke" => {
                let [Value::Method(method), receiver, Value::Array(args)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                self.reflective_invoke(*method, receiver.clone(), args)
            }
            Value::Class(class) if selector == "remove_module" => {
                let [Value::Symbol(module)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let module = *self
                    .module_names
                    .get(module)
                    .ok_or(EvaluationError::UnsupportedConstruct)?;
                self.remove_module(class, module).map(|()| Value::Nil)
            }
            Value::Class(class) if selector == "set_superclass" => {
                let [Value::Class(superclass)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                self.set_superclass(class, *superclass)
            }
            Value::Class(class) if selector == "ancestors" => self.ancestors(class),
            // IRIS-V1-TYPES-C076: a Class exposes `.type` metadata, and the Type
            // object it yields is deliberately NOT the Class object itself.
            Value::Class(class) if selector == "type" => Ok(Value::Type(class)),
            Value::Type(left) if selector == "subtype?" => {
                let [Value::Type(right)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                self.subtype(left, *right)
            }
            Value::Class(class) => {
                let selector_id = self.selector(selector);
                if self.is_builtin_class(class) {
                    match self
                        .kernel
                        .dispatch_class_object(self.runtime.registry(), class, selector_id)
                        .map_err(EvaluationError::Runtime)?
                    {
                        iris_runtime::DispatchOutcome::Invoke(method) => {
                            return self.invoke_selected(method, Value::Class(class), arguments);
                        }
                        iris_runtime::DispatchOutcome::WouldInvokeMethodMissing { .. } => {
                            if let Some(native) =
                                iris_runtime::NativeSelector::from_source(selector)
                            {
                                return self
                                    .kernel
                                    .send(
                                        self.runtime.registry(),
                                        Value::Class(class),
                                        native,
                                        arguments,
                                    )
                                    .map_err(EvaluationError::Runtime);
                            }
                        }
                    }
                }
                let method = match self.class_dispatch(class, selector_id)? {
                    iris_runtime::DispatchOutcome::Invoke(method) => method,
                    iris_runtime::DispatchOutcome::WouldInvokeMethodMissing { .. } => self
                        .runtime
                        .registry()
                        .dispatch(class, selector_id)
                        .map_err(iris_runtime::ConstructionError::from)
                        .map_err(EvaluationError::Construction)
                        .and_then(|outcome| match outcome {
                            iris_runtime::DispatchOutcome::Invoke(method) => Ok(method),
                            iris_runtime::DispatchOutcome::WouldInvokeMethodMissing { .. } => {
                                Err(EvaluationError::MessageNotFound {
                                    receiver_class: "Class".into(),
                                    selector: selector.into(),
                                })
                            }
                        })?,
                };
                self.invoke_selected(method, Value::Class(class), arguments)
            }
            Value::Object(object) => {
                let selector = self.selector(selector);
                match self.resolve_instance_method(object, selector) {
                    Ok(method) => self.invoke_method(method, Value::Object(object), arguments),
                    Err(EvaluationError::Construction(
                        iris_runtime::ConstructionError::Dispatch(
                            iris_runtime::DispatchError::MissingMethod { .. },
                        ),
                    )) if selector == self.selector("to_bool") && arguments.is_empty() => {
                        Ok(Value::Bool(true))
                    }
                    // IRIS-V1-RUNTIME-C088: an ordinary object answers `hash`
                    // with a runtime-stable identity hash assigned at allocation,
                    // which survives GC movement and exposes no address.
                    Err(EvaluationError::Construction(
                        iris_runtime::ConstructionError::Dispatch(
                            iris_runtime::DispatchError::MissingMethod { .. },
                        ),
                    )) if selector == self.selector("hash") && arguments.is_empty() => {
                        let hash = self
                            .runtime
                            .identity_hash(object)
                            .map_err(|_| EvaluationError::UnsupportedConstruct)?;
                        Ok(Value::Integer(hash.into()))
                    }
                    Err(EvaluationError::Construction(
                        iris_runtime::ConstructionError::Dispatch(
                            iris_runtime::DispatchError::MissingMethod { .. },
                        ),
                    )) if self.comparison_slot(selector).is_some() => {
                        let slot = self
                            .comparison_slot(selector)
                            .ok_or(EvaluationError::UnsupportedConstruct)?;
                        self.default_comparison(object, slot, arguments)
                    }
                    Err(EvaluationError::Construction(
                        iris_runtime::ConstructionError::Dispatch(
                            iris_runtime::DispatchError::MissingMethod { .. },
                        ),
                    )) if selector == self.selector("<=>") && arguments.len() == 1 => {
                        Ok(Value::Nil)
                    }
                    Err(EvaluationError::Construction(
                        iris_runtime::ConstructionError::Dispatch(
                            iris_runtime::DispatchError::MissingMethod { .. },
                        ),
                    )) => self.invoke_method_missing(object, selector, arguments),
                    Err(error) => Err(error),
                }
            }
            value => self.value_send(value, selector, arguments),
        }
    }

    fn member_read(&mut self, receiver: Value, selector: &str) -> Result<Value, EvaluationError> {
        let Value::Object(object) = receiver else {
            return self.send(receiver, selector, &[]);
        };
        let selector = self.selector(selector);
        let method = self.resolve_instance_method(object, selector)?;
        if self.property_methods.get(&method.id()) == Some(&true) {
            self.invoke_method(method, Value::Object(object), &[])
        } else {
            let class = self
                .runtime
                .class_of(object)
                .map_err(EvaluationError::Construction)?;
            self.runtime
                .registry_mut()
                .bind_instance(object, class, selector)
                .map(Value::BoundMethod)
                .map_err(iris_runtime::ConstructionError::from)
                .map_err(EvaluationError::Construction)
        }
    }

    fn reflection(
        &mut self,
        namespace: &str,
        selector: &str,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        match (namespace, selector) {
            ("Reflection::Object", "list_ivars") => {
                let [target] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                self.list_ivars(target)
            }
            ("Reflection::Object", "get_ivar") => {
                let [target, Value::Symbol(name)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                self.get_ivar(target, name)
            }
            ("Reflection::Object", "set_ivar") => {
                let [target, Value::Symbol(name), value] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                self.set_ivar(target, name, value.clone())
            }
            ("Reflection::Object", "remove_ivar") => {
                let [target, Value::Symbol(name)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                self.remove_ivar(target, name)
            }
            ("Reflection::Class", "method") | ("Reflection::Module", "method") => {
                let [target, Value::Symbol(name)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                self.reflection_method(target, name)
            }
            ("Reflection::Class", "invoke") | ("Reflection::Module", "invoke") => {
                let [Value::Method(method), receiver, Value::Array(args)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                self.reflective_invoke(*method, receiver.clone(), args)
            }
            ("Reflection::Class", "remove_module") => {
                let [Value::Class(class), Value::Symbol(module)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let module = *self
                    .module_names
                    .get(module)
                    .ok_or(EvaluationError::UnsupportedConstruct)?;
                self.remove_module(*class, module).map(|()| Value::Nil)
            }
            ("Reflection::Class", "set_superclass") => {
                let [Value::Class(target), Value::Class(superclass)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                self.set_superclass(*target, *superclass)
            }
            ("Reflection::Class", "ancestors") => {
                let [Value::Class(target)] = arguments else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                self.ancestors(*target)
            }
            _ => Err(EvaluationError::UnsupportedConstruct),
        }
    }

    fn list_ivars(&self, target: &Value) -> Result<Value, EvaluationError> {
        let names = match target {
            Value::Object(object) => self.runtime.raw_ivar_names(*object),
            Value::Class(class) => self.runtime.class_raw_ivar_names(*class),
            _ => return Err(EvaluationError::UnsupportedConstruct),
        }
        .map_err(EvaluationError::Construction)?;
        Ok(Value::Array(
            names
                .into_iter()
                .map(|name| Value::Symbol(self.selector_name(name)))
                .collect(),
        ))
    }

    fn get_ivar(&mut self, target: &Value, name: &str) -> Result<Value, EvaluationError> {
        let selector = self.selector_id(name)?;
        match target {
            Value::Object(object) => self.runtime.raw_ivar(*object, selector),
            Value::Class(class) => self.runtime.class_raw_ivar(*class, selector),
            _ => return Err(EvaluationError::UnsupportedConstruct),
        }
        .map_err(EvaluationError::Construction)
    }

    fn set_ivar(
        &mut self,
        target: &Value,
        name: &str,
        value: Value,
    ) -> Result<Value, EvaluationError> {
        let selector = self.selector_id(name)?;
        match target {
            Value::Object(object) => self.runtime.assign_raw_ivar(*object, selector, value),
            Value::Class(class) => self.runtime.assign_class_raw_ivar(*class, selector, value),
            _ => return Err(EvaluationError::UnsupportedConstruct),
        }
        .map_err(EvaluationError::Construction)
    }

    fn remove_ivar(&mut self, target: &Value, name: &str) -> Result<Value, EvaluationError> {
        let selector = self.selector_id(name)?;
        let value = match target {
            Value::Object(object) => self.runtime.remove_raw_ivar(*object, selector),
            Value::Class(class) => self.runtime.remove_class_raw_ivar(*class, selector),
            _ => return Err(EvaluationError::UnsupportedConstruct),
        }
        .map_err(EvaluationError::Construction)?;
        value.ok_or(EvaluationError::UnsupportedConstruct)
    }

    fn selector_id(&mut self, name: &str) -> Result<Selector, EvaluationError> {
        if name.starts_with('@') {
            Ok(self.selector(name))
        } else {
            Err(EvaluationError::UnsupportedConstruct)
        }
    }

    fn reflection_method(&mut self, target: &Value, name: &str) -> Result<Value, EvaluationError> {
        let selector = self.selector(name);
        match target {
            Value::Class(class) => match self.runtime.registry().dispatch(*class, selector) {
                Ok(iris_runtime::DispatchOutcome::Invoke(method)) => Ok(Value::Method(method)),
                Ok(iris_runtime::DispatchOutcome::WouldInvokeMethodMissing { .. }) => {
                    Ok(Value::Nil)
                }
                Err(error) => Err(EvaluationError::Construction(error.into())),
            },
            Value::Symbol(module) => self
                .module_names
                .get(module)
                .and_then(|module| self.module_methods.get(&(*module, selector)))
                .copied()
                .map(Value::Method)
                .ok_or(EvaluationError::UnsupportedConstruct),
            _ => Err(EvaluationError::UnsupportedConstruct),
        }
    }

    fn reflective_invoke(
        &mut self,
        method: Method,
        receiver: Value,
        args: &[Value],
    ) -> Result<Value, EvaluationError> {
        let class = match receiver {
            Value::Object(object) => self.runtime.class_of(object),
            Value::Class(class) => Ok(class),
            _ => return Err(EvaluationError::UnsupportedConstruct),
        }
        .map_err(EvaluationError::Construction)?;
        self.runtime
            .registry()
            .validate_method_binding(class, method)
            .map_err(iris_runtime::ConstructionError::from)
            .map_err(EvaluationError::Construction)?;
        self.invoke_method(method, receiver, args)
    }

    fn remove_module(&mut self, class: ClassId, module: ModuleId) -> Result<(), EvaluationError> {
        self.runtime
            .registry()
            .require_meta_capability(class, Capability::Modules)
            .map_err(EvaluationError::Class)?;
        let mut candidate = self
            .runtime
            .registry_mut()
            .open(class)
            .map_err(EvaluationError::Class)?;
        candidate.remove_module(module);
        self.runtime
            .registry_mut()
            .publish(candidate)
            .map(|_| ())
            .map_err(EvaluationError::Class)
    }

    fn set_superclass(
        &mut self,
        target: ClassId,
        superclass: ClassId,
    ) -> Result<Value, EvaluationError> {
        if self.is_builtin_class(target) {
            return Err(EvaluationError::Class(
                iris_runtime::ClassError::ProtectedSuperclass { class: target },
            ));
        }
        self.runtime
            .registry()
            .require_meta_capability(target, Capability::Superclass)
            .map_err(EvaluationError::Class)?;
        let mut candidate = self
            .runtime
            .registry_mut()
            .open(target)
            .map_err(EvaluationError::Class)?;
        candidate.replace_runtime_superclass(Some(superclass));
        self.runtime
            .registry_mut()
            .publish(candidate)
            .map(|_| Value::Nil)
            .map_err(EvaluationError::Class)
    }

    fn ancestors(&self, target: ClassId) -> Result<Value, EvaluationError> {
        let mro = self
            .runtime
            .registry()
            .active(target)
            .map_err(EvaluationError::Class)?
            .mro();
        Ok(Value::Array(
            mro.iter()
                .filter_map(|entry| match entry {
                    iris_runtime::MroEntry::Class(class) => Some(Value::Class(*class)),
                    iris_runtime::MroEntry::Module(_) => None,
                })
                .collect(),
        ))
    }

    pub(super) fn class_name(&self, name: &str) -> Result<Option<ClassId>, EvaluationError> {
        match self.names.get(name) {
            Some(Binding {
                value: Value::Class(class),
                ..
            }) => Ok(Some(*class)),
            Some(_) => Err(EvaluationError::UnsupportedConstruct),
            None => Ok(builtin(name, &self.kernel).and_then(|value| match value {
                Value::Class(class) => Some(class),
                _ => None,
            })),
        }
    }

    /// Reports whether a Class is one of the five protected built-in value Classes.
    ///
    /// This must NOT test whether the Class is absent from the runtime registry.
    /// Built-in and declared Classes now share one registry so that `Object` can
    /// appear in an ordinary MRO, which makes every built-in `active` and would
    /// make such a test answer false for all of them. `Object` is deliberately
    /// excluded: `IRIS-V1-RUNTIME-C150` protects exactly the five value Classes,
    /// and `IRIS-V1-RUNTIME-C005` makes `Object` the ordinary-object root, so it
    /// constructs and dispatches like a declared Class.
    fn is_builtin_class(&self, class: ClassId) -> bool {
        [
            iris_runtime::BuiltinClass::Nil,
            iris_runtime::BuiltinClass::Bool,
            iris_runtime::BuiltinClass::Integer,
            iris_runtime::BuiltinClass::Float32,
            iris_runtime::BuiltinClass::Float64,
        ]
        .into_iter()
        .any(|kind| {
            self.kernel
                .class(kind)
                .is_ok_and(|candidate| candidate == class)
        })
    }

    fn builtin_class_property(&self, selector: &str) -> bool {
        iris_runtime::NativeSelector::from_source(selector.trim_end_matches('=')).is_some_and(
            |selector| {
                matches!(
                    selector,
                    iris_runtime::NativeSelector::Nan | iris_runtime::NativeSelector::Infinity
                )
            },
        )
    }

    fn class_dispatch(
        &self,
        class: ClassId,
        selector: Selector,
    ) -> Result<iris_runtime::DispatchOutcome, EvaluationError> {
        if self.is_builtin_class(class) {
            return self
                .kernel
                .dispatch_class_object(self.runtime.registry(), class, selector)
                .map_err(EvaluationError::Runtime);
        }
        self.runtime
            .registry()
            .dispatch_class_object(class, selector)
            .map_err(iris_runtime::ConstructionError::from)
            .map_err(EvaluationError::Construction)
    }

    fn invoke_selected(
        &mut self,
        method: Method,
        receiver: Value,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        if self.bodies.contains_key(&method.body().raw()) {
            self.invoke_method(method, receiver, arguments)
        } else {
            self.kernel
                .invoke_selected(method, receiver, arguments)
                .map_err(EvaluationError::Runtime)
        }
    }

    fn value_send(
        &mut self,
        receiver: Value,
        selector: &str,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        // IRIS-V1-CONTROL-C076 makes `call` the sole invocation spelling for the
        // ordinary callable kinds, so a callable answers it as an ordinary
        // selector rather than being applied directly to an argument list.
        // The chapter 04 exception example reads `context.value`, and
        // IRIS-V1-CONTROL-C057 owns the cause, so both are ordinary reads on
        // the context rather than dispatched sends.
        if let Value::ExceptionContext(value, cause, suppressed) = &receiver {
            match selector {
                "value" => return Ok((**value).clone()),
                "cause" => return Ok((**cause).clone()),
                "suppressed" => return Ok(Value::Array(suppressed.clone())),
                _ => {}
            }
        }
        if selector == "call" {
            match receiver {
                Value::Closure(object) => return self.invoke_closure(object, arguments),
                Value::BoundMethod(bound) => {
                    return self.invoke_method(
                        self.validate_bound_method(bound)?,
                        match bound.receiver() {
                            iris_runtime::BoundReceiver::Class(class) => Value::Class(class),
                            iris_runtime::BoundReceiver::Object(object) => Value::Object(object),
                        },
                        arguments,
                    );
                }
                _ => {}
            }
        }
        // IRIS-V1-RUNTIME-C042 makes Closure default equality identity-only and
        // forbids structural comparison, and IRIS-V1-RUNTIME-C040 gives each
        // BoundMethod read a distinct identity. Neither has a built-in Class to
        // dispatch through, so both answer by identity here rather than failing.
        if matches!(selector, "==" | "!=")
            && matches!(receiver, Value::Closure(_) | Value::BoundMethod(_))
        {
            let [other] = arguments else {
                return Err(EvaluationError::UnsupportedConstruct);
            };
            let equal = receiver == *other;
            return Ok(Value::Bool(if selector == "==" { equal } else { !equal }));
        }
        let selector_id = iris_runtime::NativeSelector::from_source(selector)
            .map_or_else(|| self.selector(selector), iris_runtime::NativeSelector::id);
        match self
            .kernel
            .dispatch_value(self.runtime.registry(), &receiver, selector_id)
            .map_err(EvaluationError::Runtime)?
        {
            iris_runtime::DispatchOutcome::Invoke(method) => {
                if matches!(selector, "==" | "!=" | "<" | "<=" | ">" | ">=")
                    && !self.bodies.contains_key(&method.body().raw())
                {
                    let comparison = self.value_send(receiver, "<=>", arguments)?;
                    let result = match comparison {
                        Value::Nil => selector == "!=",
                        Value::Integer(value) if value == (-1_i8).into() => {
                            matches!(selector, "!=" | "<" | "<=")
                        }
                        Value::Integer(value) if value == 0_u8.into() => {
                            matches!(selector, "==" | "<=" | ">=")
                        }
                        Value::Integer(value) if value == 1_u8.into() => {
                            matches!(selector, "!=" | ">" | ">=")
                        }
                        Value::Integer(_)
                        | Value::Bool(_)
                        | Value::Float32(_)
                        | Value::Float64(_)
                        | Value::Array(_)
                        | Value::Hash(_)
                        | Value::Symbol(_)
                        | Value::Class(_)
                        | Value::Type(_)
                        | Value::Contract(_)
                        | Value::Closure(_)
                        | Value::KeywordArgument(_, _)
                        | Value::IterationYield(_)
                        | Value::IterationDone
                        | Value::ExceptionContext(..)
                        | Value::ContractView(_, _)
                        | Value::Object(_)
                        | Value::BoundMethod(_)
                        | Value::Method(_) => {
                            return Err(EvaluationError::UnsupportedConstruct);
                        }
                    };
                    return Ok(Value::Bool(result));
                }
                self.invoke_selected(method, receiver, arguments)
            }
            iris_runtime::DispatchOutcome::WouldInvokeMethodMissing { .. } => {
                Err(EvaluationError::MessageNotFound {
                    receiver_class: receiver_class_name(&receiver).into(),
                    selector: selector.into(),
                })
            }
        }
    }

    fn assign_value_raw_ivar(&self, receiver: Value) -> Result<Value, EvaluationError> {
        let class = match receiver {
            Value::Nil => self.kernel.class(iris_runtime::BuiltinClass::Nil),
            Value::Bool(_) => self.kernel.class(iris_runtime::BuiltinClass::Bool),
            Value::Integer(_) => self.kernel.class(iris_runtime::BuiltinClass::Integer),
            Value::Float32(_) => self.kernel.class(iris_runtime::BuiltinClass::Float32),
            Value::Float64(_) => self.kernel.class(iris_runtime::BuiltinClass::Float64),
            Value::Array(_)
            | Value::Hash(_)
            | Value::Symbol(_)
            | Value::Class(_)
            | Value::Type(_)
            | Value::Contract(_)
            | Value::Closure(_)
            | Value::KeywordArgument(_, _)
            | Value::IterationYield(_)
            | Value::IterationDone
            | Value::ExceptionContext(..)
            | Value::ContractView(_, _)
            | Value::Object(_)
            | Value::BoundMethod(_)
            | Value::Method(_) => return Err(EvaluationError::UnsupportedConstruct),
        }
        .map_err(EvaluationError::Runtime)?;
        self.kernel
            .require_meta_capability(self.runtime.registry(), class, Capability::InstanceState)
            .map_err(|_| iris_runtime::ConstructionError::InstanceState { class })
            .map_err(EvaluationError::Construction)
            .map(|()| receiver)
    }

    fn append_array_binding(
        &mut self,
        target: &Expression,
        arguments: &[Value],
        locals: &HashMap<String, Value>,
    ) -> Result<Value, EvaluationError> {
        let [value] = arguments else {
            return Err(EvaluationError::Runtime(iris_runtime::KernelError::Arity));
        };
        let Expression::Name(name) = target else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        if locals.contains_key(name) {
            return Err(EvaluationError::UnsupportedConstruct);
        }
        let binding = self
            .names
            .get_mut(name)
            .ok_or(EvaluationError::UnsupportedConstruct)?;
        let Value::Array(values) = &mut binding.value else {
            return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
        };
        values.push(value.clone());
        Ok(Value::Nil)
    }

    fn selector(&mut self, name: &str) -> Selector {
        if name == "initialize" {
            return Selector::INITIALIZE;
        }
        if let Some(selector) = self.selectors.get(name) {
            return *selector;
        }
        let selector = iris_runtime::NativeSelector::from_source(name).map_or_else(
            || {
                let selector = Selector::new(self.next_selector);
                self.next_selector += 1;
                selector
            },
            iris_runtime::NativeSelector::id,
        );
        self.selectors.insert(name.into(), selector);
        selector
    }

    fn register_body(&mut self, method: MethodDeclaration) -> MethodBody {
        let body = MethodBody::new(self.next_body);
        self.next_body += 1;
        self.bodies.insert(body.raw(), method);
        body
    }

    fn comparison_slot(&mut self, selector: Selector) -> Option<ComparisonSlot> {
        [
            ("==", ComparisonSlot::Equal),
            ("!=", ComparisonSlot::NotEqual),
            ("<", ComparisonSlot::Less),
            ("<=", ComparisonSlot::LessEqual),
            (">", ComparisonSlot::Greater),
            (">=", ComparisonSlot::GreaterEqual),
        ]
        .into_iter()
        .find_map(|(name, slot)| (self.selector(name) == selector).then_some(slot))
    }

    /// Derives a comparison result for a receiver whose slot is still the default.
    ///
    /// `IRIS-V1-RUNTIME-C086` requires equality to test reference identity FIRST
    /// and return `true` without consulting `<=>`, which is why an object whose
    /// `<=>` always answers `nil` still equals itself. Otherwise the current
    /// visible `<=>` is sent, exactly once, and `IRIS-V1-RUNTIME-C084` maps its
    /// response, with `nil` meaning unordered. `IRIS-V1-RUNTIME-C085` rejects any
    /// response outside `Integer(-1)`, `Integer(0)`, `Integer(1)` and `nil`.
    fn default_comparison(
        &mut self,
        object: iris_runtime::ObjectId,
        slot: ComparisonSlot,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        let [other] = arguments else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        if matches!(slot, ComparisonSlot::Equal | ComparisonSlot::NotEqual)
            && *other == Value::Object(object)
        {
            return Ok(Value::Bool(matches!(slot, ComparisonSlot::Equal)));
        }
        let ordering = match self.send(Value::Object(object), "<=>", std::slice::from_ref(other))? {
            Value::Nil => None,
            Value::Integer(value) if value == (-1_i8).into() => Some(-1_i8),
            Value::Integer(value) if value == 0_u8.into() => Some(0_i8),
            Value::Integer(value) if value == 1_u8.into() => Some(1_i8),
            // D-094 separates two failures. Another Integer satisfies the broad
            // `Integer?` return type but violates the protocol, so it is a
            // ComparisonContractError. A non-Integer, non-nil result violates the
            // return type contract itself and is a TypeError.
            Value::Integer(_) => return Err(EvaluationError::ComparisonContractError),
            _ => return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type)),
        };
        Ok(Value::Bool(match (slot, ordering) {
            (ComparisonSlot::Equal, Some(0))
            | (ComparisonSlot::Less, Some(-1))
            | (ComparisonSlot::LessEqual, Some(-1 | 0))
            | (ComparisonSlot::Greater, Some(1))
            | (ComparisonSlot::GreaterEqual, Some(0 | 1))
            | (ComparisonSlot::NotEqual, None) => true,
            (ComparisonSlot::NotEqual, Some(order)) => order != 0,
            _ => false,
        }))
    }

    /// Evaluates `value is T` against the receiver's CURRENT runtime ancestry.
    ///
    /// `IRIS-V1-TYPES-C028` requires the test to consult current runtime
    /// ancestry, so a committed superclass change is observable here, and
    /// forbids it from sending `to_bool` or converting the value. `C009` makes
    /// `Object` the top Type, so every value answers true for it.
    fn type_test(&mut self, value: &Value, target: &Value) -> Result<Value, EvaluationError> {
        let Value::Class(target) = target else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        if self
            .kernel
            .class(iris_runtime::BuiltinClass::Object)
            .is_ok_and(|root| root == *target)
        {
            return Ok(Value::Bool(true));
        }
        let class = match value {
            Value::Object(object) => self
                .runtime
                .class_of(*object)
                .map_err(EvaluationError::Construction)?,
            Value::Nil => self
                .kernel
                .class(iris_runtime::BuiltinClass::Nil)
                .map_err(EvaluationError::Runtime)?,
            Value::Bool(_) => self
                .kernel
                .class(iris_runtime::BuiltinClass::Bool)
                .map_err(EvaluationError::Runtime)?,
            Value::Integer(_) => self
                .kernel
                .class(iris_runtime::BuiltinClass::Integer)
                .map_err(EvaluationError::Runtime)?,
            Value::Float32(_) => self
                .kernel
                .class(iris_runtime::BuiltinClass::Float32)
                .map_err(EvaluationError::Runtime)?,
            Value::Float64(_) => self
                .kernel
                .class(iris_runtime::BuiltinClass::Float64)
                .map_err(EvaluationError::Runtime)?,
            Value::Class(class) => *class,
            Value::Array(_)
            | Value::Hash(_)
            | Value::Symbol(_)
            | Value::Type(_)
            | Value::Contract(_)
            | Value::Closure(_)
            | Value::KeywordArgument(_, _)
            | Value::IterationYield(_)
            | Value::IterationDone
            | Value::ExceptionContext(..)
            | Value::ContractView(_, _)
            | Value::BoundMethod(_)
            | Value::Method(_) => {
                return Ok(Value::Bool(false));
            }
        };
        let ancestry = self
            .runtime
            .registry()
            .active(class)
            .map_err(EvaluationError::Class)?
            .mro()
            .iter()
            .any(|entry| matches!(entry, iris_runtime::MroEntry::Class(entry) if entry == target));
        Ok(Value::Bool(ancestry))
    }

    /// Answers `Type#subtype?` over current nominal ancestry.
    ///
    /// `IRIS-V1-TYPES-C079` requires this query to use the SAME nominal rules as
    /// the runtime guards, so it walks the same active MRO that `is` consults
    /// rather than a separately cached relation. `C009` makes `Object` the top
    /// Type, so every nominal Type is its subtype.
    fn subtype(&mut self, left: ClassId, right: ClassId) -> Result<Value, EvaluationError> {
        if self
            .kernel
            .class(iris_runtime::BuiltinClass::Object)
            .is_ok_and(|root| root == right)
        {
            return Ok(Value::Bool(true));
        }
        let ancestry = self
            .runtime
            .registry()
            .active(left)
            .map_err(EvaluationError::Class)?
            .mro()
            .iter()
            .any(|entry| matches!(entry, iris_runtime::MroEntry::Class(entry) if *entry == right));
        Ok(Value::Bool(ancestry))
    }

    /// Builds the Contract view that `value as ContractType` proves.
    ///
    /// `IRIS-V1-TYPES-C049` requires the `as ContractType` part to prove or check
    /// the nominal Contract view, so a receiver whose Class never declared that
    /// Contract with `for` is rejected rather than silently viewed.
    fn contract_view(&mut self, value: Value, target: &Value) -> Result<Value, EvaluationError> {
        let Value::Contract(contract) = target else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        let Value::Object(object) = value else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        let class = self
            .runtime
            .class_of(object)
            .map_err(EvaluationError::Construction)?;
        if !self
            .class_contracts
            .get(&class)
            .is_some_and(|declared| declared.contains(contract))
        {
            return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
        }
        Ok(Value::ContractView(
            Box::new(Value::Object(object)),
            *contract,
        ))
    }

    /// Sends `view..member(args)` to the Contract-qualified slot.
    ///
    /// `IRIS-V1-TYPES-C049` states that `..member` chooses the Contract-qualified
    /// slot identity while ordinary `value.member` always sends the unqualified
    /// selector, so this resolves ONLY the qualified table and never falls back
    /// to ordinary dispatch, which would merge the two namespaces.
    fn qualified_send(
        &mut self,
        view: Value,
        selector: &str,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        let Value::ContractView(receiver, contract) = view else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        let Value::Object(object) = *receiver else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        let class = self
            .runtime
            .class_of(object)
            .map_err(EvaluationError::Construction)?;
        let selector_id = self.selector(selector);
        let method = *self
            .qualified_methods
            .get(&(class, contract, selector_id))
            // A missing qualified slot is a Contract dispatch failure, NOT a
            // missing message: IRIS-V1-TYPES-C049 keeps the qualified namespace
            // separate, so `method_missing` must not be reached from here.
            .ok_or(EvaluationError::Construction(
                iris_runtime::ConstructionError::Dispatch(
                    iris_runtime::DispatchError::ContractDispatch {
                        contract: iris_runtime::ModuleId::new(contract.raw()),
                        selector: selector_id,
                    },
                ),
            ))?;
        self.invoke_method(method, Value::Object(object), arguments)
    }

    /// Invokes a Closure with its captured environment restored.
    ///
    /// `IRIS-V1-RUNTIME-C072` keeps the captured receiver fixed, so the body runs
    /// against the receiver captured at creation rather than any current one,
    /// which is what lets an escaped Closure keep writing that receiver's ivars.
    fn invoke_closure(
        &mut self,
        object: iris_runtime::ObjectId,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        let record = self
            .closures
            .get(&object)
            .ok_or(EvaluationError::UnsupportedConstruct)?;
        let parameters = record.parameters.clone();
        let body = record.body.clone();
        let receiver = record.receiver.clone();
        let mut locals = record.captured.clone();
        for (name, argument) in parameters.iter().zip(arguments) {
            locals.insert(name.clone(), argument.clone());
        }
        let mut result = Value::Nil;
        for statement in &body {
            result = self.statement(statement, &locals, receiver.clone())?;
        }
        Ok(result)
    }

    fn resolve_instance_method(
        &self,
        object: iris_runtime::ObjectId,
        selector: Selector,
    ) -> Result<Method, EvaluationError> {
        let class = self
            .runtime
            .class_of(object)
            .map_err(EvaluationError::Construction)?;
        let context = match self.current_method.map(|method| method.owner()) {
            Some(MethodOwner::Module(module)) => {
                DispatchContext::module_implementation(module, true)
            }
            Some(MethodOwner::Class(owner)) => DispatchContext::implementation(owner, true),
            None => DispatchContext::external(),
        };
        match self
            .runtime
            .registry()
            .dispatch_with_context(class, selector, context)
            .map_err(iris_runtime::ConstructionError::from)
        {
            Ok(DispatchOutcome::Invoke(method)) => Ok(method),
            Ok(DispatchOutcome::WouldInvokeMethodMissing { .. }) => self
                .class_mixins
                .get(&class)
                .into_iter()
                .flatten()
                .rev()
                .find_map(|class| self.runtime.registry().dispatch(*class, selector).ok())
                .and_then(|outcome| match outcome {
                    iris_runtime::DispatchOutcome::Invoke(method) => Some(method),
                    iris_runtime::DispatchOutcome::WouldInvokeMethodMissing { .. } => None,
                })
                .ok_or(EvaluationError::Construction(
                    iris_runtime::ConstructionError::Dispatch(
                        iris_runtime::DispatchError::MissingMethod { selector },
                    ),
                )),
            Err(iris_runtime::ConstructionError::Dispatch(
                iris_runtime::DispatchError::MissingMethod { .. },
            )) => self
                .class_mixins
                .get(&class)
                .into_iter()
                .flatten()
                .rev()
                .find_map(|class| self.runtime.registry().dispatch(*class, selector).ok())
                .and_then(|outcome| match outcome {
                    iris_runtime::DispatchOutcome::Invoke(method) => Some(method),
                    iris_runtime::DispatchOutcome::WouldInvokeMethodMissing { .. } => None,
                })
                .ok_or(EvaluationError::Construction(
                    iris_runtime::ConstructionError::Dispatch(
                        iris_runtime::DispatchError::MissingMethod { selector },
                    ),
                )),
            Err(error) => Err(EvaluationError::Construction(error)),
        }
    }

    fn invoke_method_missing(
        &mut self,
        object: iris_runtime::ObjectId,
        missing: Selector,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        let fallback = self.selector("method_missing");
        if missing == fallback {
            return Err(EvaluationError::MessageNotFound {
                receiver_class: self.object_class_name(object)?,
                selector: "method_missing".into(),
            });
        }
        let method = match self.resolve_instance_method(object, fallback) {
            Ok(method) => method,
            Err(EvaluationError::Construction(iris_runtime::ConstructionError::Dispatch(
                iris_runtime::DispatchError::MissingMethod { .. },
            ))) => {
                return Err(EvaluationError::MessageNotFound {
                    receiver_class: self.object_class_name(object)?,
                    selector: self.selector_name(missing),
                });
            }
            Err(error) => return Err(error),
        };
        // IRIS-V1-RUNTIME-C099 passes the trailing block as the separate
        // `block: Closure?` parameter, NOT inside the positional argument
        // snapshot, so a Closure in final position is split out here.
        let (positional, block) = match arguments {
            [head @ .., Value::Closure(closure)] => (head.to_vec(), Value::Closure(*closure)),
            _ => (arguments.to_vec(), Value::Nil),
        };
        self.invoke_method(
            method,
            Value::Object(object),
            &[
                Value::Symbol(self.selector_name(missing)),
                Value::Array(positional),
                block,
            ],
        )
    }

    fn selector_name(&self, selector: Selector) -> String {
        self.selectors
            .iter()
            .find_map(|(name, id)| (*id == selector).then(|| name.clone()))
            .unwrap_or_else(|| "<unknown>".into())
    }

    fn object_class_name(&self, object: iris_runtime::ObjectId) -> Result<String, EvaluationError> {
        let class = self
            .runtime
            .class_of(object)
            .map_err(EvaluationError::Construction)?;
        Ok(self
            .names
            .iter()
            .find_map(|(name, binding)| {
                matches!(binding.value, Value::Class(candidate) if candidate == class)
                    .then(|| name.clone())
            })
            .unwrap_or_else(|| "Object".into()))
    }

    fn validate_module_overrides(
        &self,
        superclass: Option<ClassId>,
        mixins: &[CompositionEdge],
    ) -> Result<(), EvaluationError> {
        let Some(superclass) = superclass else {
            return Ok(());
        };
        for edge in mixins {
            let module = edge.module();
            for ((owner, selector), method) in &self.module_methods {
                if *owner != module {
                    continue;
                }
                let replaces = matches!(
                    self.runtime.registry().dispatch(superclass, *selector),
                    Ok(iris_runtime::DispatchOutcome::Invoke(_))
                );
                let is_override = self
                    .module_method_overrides
                    .get(&method.id())
                    .copied()
                    .unwrap_or(false);
                if replaces && !is_override {
                    return Err(EvaluationError::Class(ClassError::OverrideRequired {
                        class: superclass,
                        selector: *selector,
                    }));
                }
                if !replaces && is_override {
                    return Err(EvaluationError::Class(ClassError::OverrideWithoutTarget {
                        class: superclass,
                        selector: *selector,
                    }));
                }
            }
        }
        Ok(())
    }

    fn read_class_var(&mut self, name: &str) -> Result<Value, EvaluationError> {
        let selector = self.selector(name);
        let class = self.class_var_owner(selector)?;
        self.runtime
            .class_var(class, selector)
            .map_err(EvaluationError::Construction)?
            .ok_or(EvaluationError::Construction(
                iris_runtime::ConstructionError::MissingDeclaredClassVariable {
                    class,
                    name: selector,
                },
            ))
    }

    fn assign_class_var(&mut self, name: &str, value: Value) -> Result<Value, EvaluationError> {
        let selector = self.selector(name);
        let class = self.class_var_owner(selector)?;
        self.runtime
            .assign_class_var(class, selector, value)
            .map_err(EvaluationError::Construction)
    }

    fn class_var_owner(&self, name: Selector) -> Result<ClassId, EvaluationError> {
        let class = self
            .lexical_class
            .ok_or(EvaluationError::UnsupportedConstruct)?;
        self.class_var_owner_from(class, name)
    }

    fn class_var_owner_from(
        &self,
        mut class: ClassId,
        name: Selector,
    ) -> Result<ClassId, EvaluationError> {
        loop {
            if self
                .runtime
                .registry()
                .active(class)
                .map_err(EvaluationError::Class)?
                .class_vars()
                .contains(&name)
            {
                return Ok(class);
            }
            class = self
                .static_superclasses
                .get(&class)
                .copied()
                .flatten()
                .ok_or(EvaluationError::Construction(
                    iris_runtime::ConstructionError::MissingDeclaredClassVariable { class, name },
                ))?;
        }
    }

    fn invoke_method(
        &mut self,
        method: Method,
        receiver: Value,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        let previous_lexical_class = self.lexical_class;
        let previous_method = self.current_method;
        self.lexical_class = match method.owner() {
            MethodOwner::Class(class) => Some(class),
            MethodOwner::Module(module) => self.module_classes.get(&module).copied(),
        };
        self.current_method = Some(method);
        let result = self.invoke_method_with_context(method, receiver, arguments);
        self.lexical_class = previous_lexical_class;
        self.current_method = previous_method;
        result
    }

    /// Binds arguments to the parameter categories `IRIS-V1-CONTROL-C023` gives.
    ///
    /// Positionals fill in order, `*rest` takes the remaining positionals as a
    /// fresh Array, keyword parameters bind by NAME rather than position, and
    /// `**kwargs` collects unmatched keywords. An unfilled parameter takes its
    /// default, and `IRIS-V1-CONTROL-C025` raises ArgumentError when a required
    /// one is left unbound or a supplied argument matches nothing.
    fn bind_parameters(
        &mut self,
        parameters: &[iris_syntax::Parameter],
        arguments: &[Value],
    ) -> Result<HashMap<String, Value>, EvaluationError> {
        use iris_syntax::ParameterCategory;
        let mut positional = Vec::new();
        let mut keyword: Vec<(String, Value)> = Vec::new();
        for argument in arguments {
            match argument {
                Value::KeywordArgument(name, value) => {
                    // IRIS-V1-CONTROL-D-357 makes a duplicate keyword an
                    // ArgumentError rather than a silent last-one-wins.
                    if keyword.iter().any(|(seen, _)| seen == name) {
                        return Err(EvaluationError::ArgumentError);
                    }
                    keyword.push((name.clone(), value.as_ref().clone()));
                }
                value => positional.push(value.clone()),
            }
        }
        let mut locals = HashMap::new();
        let mut next = 0usize;
        for parameter in parameters {
            let name = parameter.name.clone();
            let bound = match parameter.category {
                ParameterCategory::Positional => {
                    let value = positional.get(next).cloned();
                    next += usize::from(value.is_some());
                    value
                }
                ParameterCategory::Rest => {
                    let rest = positional.split_off(next.min(positional.len()));
                    Some(Value::Array(rest))
                }
                ParameterCategory::Keyword => keyword
                    .iter()
                    .position(|(seen, _)| *seen == name)
                    .map(|index| keyword.remove(index).1),
                // `**kwargs` binds a fresh `Hash<Symbol,V>` of the keywords no
                // declared parameter matched.
                ParameterCategory::KeywordRest => {
                    let rest = std::mem::take(&mut keyword)
                        .into_iter()
                        .map(|(name, value)| (Value::Symbol(name), value))
                        .collect();
                    Some(Value::Hash(rest))
                }
                // C025 binds an omitted optional block to `nil`, so the block
                // channel is never a missing-argument error.
                ParameterCategory::Block => {
                    Some(positional.get(next).cloned().unwrap_or(Value::Nil))
                }
            };
            if parameter.category == ParameterCategory::Block {
                next += usize::from(next < positional.len());
            }
            let value = match bound {
                Some(value) => value,
                None => match &parameter.default {
                    Some(default) => self.expression(default, &locals, None)?,
                    None => return Err(EvaluationError::ArgumentError),
                },
            };
            locals.insert(name, value);
        }
        // A leftover argument in either channel matches no parameter, which
        // C025 makes an arity error rather than a silent discard.
        if next < positional.len() || !keyword.is_empty() {
            return Err(EvaluationError::ArgumentError);
        }
        Ok(locals)
    }

    fn invoke_method_with_context(
        &mut self,
        method: Method,
        receiver: Value,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        let Some(declaration) = self.bodies.get(&method.body().raw()) else {
            return Err(EvaluationError::Execution(
                iris_runtime::ExecutionError::Raised(Value::Nil),
            ));
        };
        let parameters = declaration.parameters.clone();
        let body = declaration.body.clone();
        let locals = self.bind_parameters(&parameters, arguments)?;
        self.block(&body, &locals, Some(receiver))
    }

    fn super_send(
        &mut self,
        receiver: Option<Value>,
        arguments: &[Value],
        selector: Option<&str>,
    ) -> Result<Value, EvaluationError> {
        let receiver = receiver.ok_or(EvaluationError::UnsupportedConstruct)?;
        let method = self
            .current_method
            .ok_or(EvaluationError::UnsupportedConstruct)?;
        let selector = selector.map_or(method.selector(), |name| self.selector(name));
        let successor = match receiver.clone() {
            Value::Object(object) => {
                let class = self
                    .runtime
                    .class_of(object)
                    .map_err(EvaluationError::Construction)?;
                self.runtime
                    .registry()
                    .dispatch_super_selector(class, method, selector)
                    .map_err(iris_runtime::KernelError::from)
                    .map_err(EvaluationError::Runtime)?
            }
            Value::Class(class) => self
                .runtime
                .registry()
                .dispatch_class_object_super_selector(class, method, selector)
                .map_err(iris_runtime::KernelError::from)
                .map_err(EvaluationError::Runtime)?,
            _ => return Err(EvaluationError::UnsupportedConstruct),
        };
        self.invoke_method(successor, receiver, arguments)
    }
}

fn meta_capabilities(names: &[String]) -> Result<MetaCapabilities, EvaluationError> {
    let mut denied = Vec::with_capacity(names.len());
    for name in names {
        let capability = match name.as_str() {
            "method_set" => Capability::MethodSet,
            "method_body" => Capability::MethodBody,
            "property_set" => Capability::PropertySet,
            "property_body" => Capability::PropertyBody,
            "modules" => Capability::Modules,
            "superclass" => Capability::Superclass,
            "subclass" => Capability::Subclass,
            "shape" => Capability::Shape,
            "class_state_set" => Capability::ClassStateSet,
            "class_state_write" => Capability::ClassStateWrite,
            "instance_state" => Capability::InstanceState,
            "native" => Capability::Native,
            _ => return Err(EvaluationError::ParseDiagnostic),
        };
        denied.push(capability);
    }
    Ok(MetaCapabilities::denying(&denied))
}

fn receiver_class_name(value: &Value) -> &'static str {
    match value {
        Value::Nil => "Nil",
        Value::Bool(_) => "Bool",
        Value::Integer(_) => "Integer",
        Value::Float32(_) => "Float32",
        Value::Float64(_) => "Float64",
        Value::Array(_) | Value::Hash(_) => "Array",
        Value::Symbol(_) => "Symbol",
        Value::Class(_) => "Class",
        Value::Type(_) => "Type",
        Value::Contract(_) => "Contract",
        Value::Closure(_) => "Closure",
        Value::KeywordArgument(_, _) | Value::IterationYield(_) => "Iteration",
        Value::IterationDone => "Iteration",
        Value::ExceptionContext(..) => "ExceptionContext",
        Value::ContractView(_, _) => "ContractView",
        Value::Object(_) => "Object",
        Value::BoundMethod(_) => "BoundMethod",
        Value::Method(_) => "Method",
    }
}

#[cfg(test)]
mod tests {
    use iris_parser::parse;
    use iris_runtime::Value;

    use super::SourceEvaluator;

    fn source_evaluator(
        source: &str,
    ) -> Result<(SourceEvaluator, iris_syntax::Program), crate::EvaluationError> {
        let parsed = parse(source);
        assert!(parsed.program_accepted, "{parsed:#?}");
        Ok((SourceEvaluator::new()?, parsed.program))
    }

    #[test]
    fn class_variable_read_uses_the_declaring_class_cell() -> Result<(), crate::EvaluationError> {
        // Given
        let source = "class A { public fun read() -> Integer { @@c } }; A.new().read()";
        let (mut evaluator, program) = source_evaluator(source)?;
        let Some(iris_syntax::Declaration::Class(class)) = program.declarations.first() else {
            return Err(crate::EvaluationError::UnsupportedConstruct);
        };
        evaluator.class(class)?;
        let class = evaluator
            .class_name("A")?
            .ok_or(crate::EvaluationError::UnsupportedConstruct)?;
        let selector = evaluator.selector("c");
        evaluator
            .runtime
            .declare_class_var(class, selector, Value::Integer(1_u8.into()), true)
            .map_err(crate::EvaluationError::Construction)?;

        // When
        let result = evaluator.statement(&program.statements[0], &Default::default(), None);

        // Then
        assert_eq!(result, Ok(Value::Integer(1_u8.into())));
        Ok(())
    }

    #[test]
    fn shared_mut_declaration_initializes_and_updates_its_cell()
    -> Result<(), crate::EvaluationError> {
        // Given
        let source = "class A { shared mut @@count: Integer = 0 public fun bump() -> Integer { @@count = 1 } public fun read() -> Integer { @@count } }; let a = A.new(); a.bump(); a.read()";
        let (mut evaluator, program) = source_evaluator(source)?;

        // When
        let result = evaluator.program(&program);

        // Then
        assert_eq!(
            result,
            Ok(Value::Array(vec![
                Value::Integer(1_u8.into()),
                Value::Integer(1_u8.into()),
            ]))
        );
        Ok(())
    }

    #[test]
    fn shared_let_assignment_returns_a_typed_immutable_storage_error()
    -> Result<(), crate::EvaluationError> {
        // Given
        let source = "class A { shared let @@count: Integer = 0 public fun bump() -> Integer { @@count = 1 } }; A.new().bump()";
        let (mut evaluator, program) = source_evaluator(source)?;

        // When
        let result = evaluator.program(&program);

        // Then
        assert!(matches!(
            result,
            Err(crate::EvaluationError::Construction(
                iris_runtime::ConstructionError::ImmutableClassVariable { .. }
            ))
        ));
        Ok(())
    }

    #[test]
    fn shared_cell_is_read_through_the_declaring_and_subclass_static_lexical_contexts()
    -> Result<(), crate::EvaluationError> {
        // Given
        let source = "class A { shared mut @@count: Integer = 1 public fun read() -> Integer { @@count } }; class B extends A { public fun read_child() -> Integer { @@count } }; [A.new().read(), B.new().read_child()]";
        let (mut evaluator, program) = source_evaluator(source)?;

        // When
        let result = evaluator.program(&program);

        // Then
        assert_eq!(
            result,
            Ok(Value::Array(vec![
                Value::Integer(1_u8.into()),
                Value::Integer(1_u8.into()),
            ]))
        );
        Ok(())
    }

    #[test]
    fn module_shared_declaration_publishes_a_module_anchored_cell()
    -> Result<(), crate::EvaluationError> {
        // Given
        let source = "module M { shared let @@version: Integer = 1 module fun read() -> Integer { @@version } }; M.read()";
        let (mut evaluator, program) = source_evaluator(source)?;

        // When
        let result = evaluator.program(&program);

        // Then
        assert_eq!(result, Ok(Value::Integer(1_u8.into())));
        Ok(())
    }

    #[test]
    fn duplicate_shared_declaration_on_static_ancestry_keeps_active_revision_unchanged()
    -> Result<(), crate::EvaluationError> {
        // Given
        let source = "class A { shared mut @@count: Integer = 1 }; class B extends A { shared mut @@count: Integer = 2 }";
        let (mut evaluator, program) = source_evaluator(source)?;
        let [
            iris_syntax::Declaration::Class(parent),
            iris_syntax::Declaration::Class(child),
        ] = program.declarations.as_slice()
        else {
            return Err(crate::EvaluationError::UnsupportedConstruct);
        };
        evaluator.class(parent)?;
        let parent_id = evaluator.class_name("A")?;
        let child_id = evaluator
            .runtime
            .registry_mut()
            .define_class(iris_runtime::StaticSpine::new(1), parent_id)
            .map_err(crate::EvaluationError::Class)?;
        evaluator.names.insert(
            child.name.clone(),
            super::Binding::immutable(Value::Class(child_id)),
        );
        evaluator.static_superclasses.insert(child_id, parent_id);
        let before = evaluator
            .runtime
            .registry()
            .active_revision(child_id)
            .map_err(crate::EvaluationError::Class)?;

        // When
        let result = evaluator.class(child);

        // Then
        assert!(matches!(
            result,
            Err(crate::EvaluationError::Class(
                iris_runtime::ClassError::DuplicateClassVariable { .. }
            ))
        ));
        assert_eq!(
            evaluator
                .runtime
                .registry()
                .active_revision(child_id)
                .map_err(crate::EvaluationError::Class)?,
            before
        );
        Ok(())
    }

    #[test]
    fn subclass_method_reads_the_declaring_class_cell() -> Result<(), crate::EvaluationError> {
        // Given
        let source = "class A { public fun read() -> Integer { @@c } }; class B extends A { public fun child_read() -> Integer { @@c } }; [A.new().read(), B.new().child_read()]";
        let (mut evaluator, program) = source_evaluator(source)?;
        for declaration in &program.declarations {
            let iris_syntax::Declaration::Class(class) = declaration else {
                return Err(crate::EvaluationError::UnsupportedConstruct);
            };
            evaluator.class(class)?;
        }
        let class = evaluator
            .class_name("A")?
            .ok_or(crate::EvaluationError::UnsupportedConstruct)?;
        let selector = evaluator.selector("c");
        evaluator
            .runtime
            .declare_class_var(class, selector, Value::Integer(1_u8.into()), true)
            .map_err(crate::EvaluationError::Construction)?;

        // When
        let result = evaluator.statement(&program.statements[0], &Default::default(), None);

        // Then
        assert_eq!(
            result,
            Ok(Value::Array(vec![
                Value::Integer(1_u8.into()),
                Value::Integer(1_u8.into()),
            ]))
        );
        Ok(())
    }

    #[test]
    fn class_variable_assignment_to_absent_storage_fails_without_creating_a_cell()
    -> Result<(), crate::EvaluationError> {
        // Given
        let source = "class A { public fun write() -> Integer { @@c = 1 } }; A.new().write()";
        let (mut evaluator, program) = source_evaluator(source)?;
        let Some(iris_syntax::Declaration::Class(class)) = program.declarations.first() else {
            return Err(crate::EvaluationError::UnsupportedConstruct);
        };
        evaluator.class(class)?;
        let class = evaluator
            .class_name("A")?
            .ok_or(crate::EvaluationError::UnsupportedConstruct)?;
        let selector = evaluator.selector("c");

        // When
        let result = evaluator.statement(&program.statements[0], &Default::default(), None);

        // Then
        eprintln!("{result:?}");
        assert!(matches!(
            result,
            Err(crate::EvaluationError::Construction(
                iris_runtime::ConstructionError::MissingDeclaredClassVariable { .. }
            ))
        ));
        assert!(matches!(
            evaluator.runtime.class_var(class, selector),
            Err(iris_runtime::ConstructionError::MissingDeclaredClassVariable { .. })
        ));
        Ok(())
    }

    #[test]
    fn class_object_raw_ivars_and_class_variables_use_distinct_storage()
    -> Result<(), crate::EvaluationError> {
        // Given
        let source = "class A { shared mut @@x: Integer = 2 class fun set_raw() -> Integer { @x = 1 } class fun raw() -> Integer { @x } class fun shared() -> Integer { @@x } }; let ignored = A.set_raw(); [A.raw(), A.shared()]";
        let (mut evaluator, program) = source_evaluator(source)?;

        // When
        let result = evaluator.program(&program);

        // Then
        assert_eq!(
            result,
            Ok(Value::Array(vec![
                Value::Integer(1_u8.into()),
                Value::Integer(2_u8.into()),
            ]))
        );
        Ok(())
    }

    #[test]
    fn class_fun_and_instance_method_share_the_declaring_class_variable_cell()
    -> Result<(), crate::EvaluationError> {
        // Given
        let source = "class P { shared mut @@n: Integer = 0; public class fun bump() -> Integer { @@n = @@n + 1 } public fun read() -> Integer { @@n } }; [P.bump(), P.new().read(), P.bump()]";
        let (mut evaluator, program) = source_evaluator(source)?;

        // When
        let result = evaluator.program(&program);

        // Then
        assert_eq!(
            result,
            Ok(Value::Array(vec![
                Value::Integer(1_u8.into()),
                Value::Integer(1_u8.into()),
                Value::Integer(2_u8.into()),
            ]))
        );
        Ok(())
    }

    #[test]
    fn class_fun_rejects_immutable_and_absent_class_variable_assignment()
    -> Result<(), crate::EvaluationError> {
        // Given
        let immutable = "class P { shared let @@n: Integer = 0; public class fun write() -> Integer { @@n = 1 } }; P.write()";
        let absent = "class P { public class fun write() -> Integer { @@n = 1 } }; P.write()";
        let (mut immutable_evaluator, immutable_program) = source_evaluator(immutable)?;
        let (mut absent_evaluator, absent_program) = source_evaluator(absent)?;
        let Some(iris_syntax::Declaration::Class(absent_declaration)) =
            absent_program.declarations.first()
        else {
            return Err(crate::EvaluationError::UnsupportedConstruct);
        };
        absent_evaluator.class(absent_declaration)?;
        let absent_class = absent_evaluator
            .class_name("P")?
            .ok_or(crate::EvaluationError::UnsupportedConstruct)?;
        let absent_selector = absent_evaluator.selector("n");

        // When
        let immutable_result = immutable_evaluator.program(&immutable_program);
        let absent_result =
            absent_evaluator.statement(&absent_program.statements[0], &Default::default(), None);

        // Then
        assert!(matches!(
            immutable_result,
            Err(crate::EvaluationError::Construction(
                iris_runtime::ConstructionError::ImmutableClassVariable { .. }
            ))
        ));
        assert!(matches!(
            absent_result,
            Err(crate::EvaluationError::Construction(
                iris_runtime::ConstructionError::MissingDeclaredClassVariable { .. }
            ))
        ));
        assert!(matches!(
            absent_evaluator
                .runtime
                .class_var(absent_class, absent_selector),
            Err(iris_runtime::ConstructionError::MissingDeclaredClassVariable { .. })
        ));
        Ok(())
    }

    #[test]
    fn open_builtin_integer_adds_a_source_method_without_replacing_native_hash()
    -> Result<(), crate::EvaluationError> {
        // Given
        let source = "open class Integer { public fun doubled() -> Integer { 2 } }; [Integer(1).doubled(), Integer(1).hash]";
        let (mut evaluator, program) = source_evaluator(source)?;

        // When
        let result = evaluator.program(&program);

        // Then
        assert_eq!(
            result,
            Ok(Value::Array(vec![
                Value::Integer(2_u8.into()),
                Value::Integer(17_824_117_788_395_916_856_u64.into()),
            ]))
        );
        Ok(())
    }

    #[test]
    fn open_builtin_with_extends_is_rejected_without_publishing_a_revision()
    -> Result<(), crate::EvaluationError> {
        // Given
        let source = "open class Integer extends Object { }";
        let (mut evaluator, program) = source_evaluator(source)?;
        let Some(iris_syntax::Declaration::Class(class)) = program.declarations.first() else {
            return Err(crate::EvaluationError::UnsupportedConstruct);
        };
        let integer = evaluator
            .kernel
            .class(iris_runtime::BuiltinClass::Integer)
            .map_err(crate::EvaluationError::Runtime)?;
        let before = evaluator
            .runtime
            .registry_mut()
            .active_revision(integer)
            .map_err(crate::EvaluationError::Class)?;

        // When
        let result = evaluator.class(class);

        // Then
        assert_eq!(
            result,
            Err(crate::EvaluationError::Class(
                iris_runtime::ClassError::ProtectedSuperclass { class: integer }
            ))
        );
        assert_eq!(
            evaluator
                .runtime
                .registry_mut()
                .active_revision(integer)
                .map_err(crate::EvaluationError::Class)?,
            before
        );
        Ok(())
    }

    #[test]
    fn retained_method_binding_failure_does_not_execute_its_observable_body()
    -> Result<(), crate::EvaluationError> {
        // Given
        let source = "let mut log = []; class A { public fun m() -> Nil { log.append(:entered); raise :body } }; class B extends A { }; class Other { }; let method = Reflection::Class.method(A, :m); Reflection::Class.set_superclass(B, Other); Reflection::Class.invoke(method, B.new(), [])";
        let (mut evaluator, program) = source_evaluator(source)?;

        // When
        let result = evaluator.program(&program);

        // Then
        assert!(matches!(
            result,
            Err(crate::EvaluationError::Construction(
                iris_runtime::ConstructionError::Dispatch(
                    iris_runtime::DispatchError::MethodBinding { .. }
                )
            ))
        ));
        assert_eq!(
            evaluator.names.get("log").map(|binding| &binding.value),
            Some(&Value::Array(Vec::new()))
        );
        Ok(())
    }

    #[test]
    fn open_builtin_replaces_compatible_method_and_preserves_singletons_and_float_bits()
    -> Result<(), crate::EvaluationError> {
        // Given
        let source = "open class Integer { override public fun hash() -> Integer { 2 } }; [Integer(1).hash, nil.hash, false.hash, true.hash]";
        let (mut evaluator, program) = source_evaluator(source)?;

        // When
        let result = evaluator.program(&program);

        // Then
        assert!(matches!(
            result,
            Ok(Value::Array(values))
                if matches!(
                    values.as_slice(),
                    [
                        Value::Integer(replaced),
                        Value::Integer(nil_hash),
                        Value::Integer(false_hash),
                        Value::Integer(true_hash),
                    ] if replaced == &2_u8.into()
                        && nil_hash == &11_850_167_709_044_604_115_u64.into()
                        && false_hash == &17_921_396_551_637_717_540_u64.into()
                        && true_hash == &14_186_115_676_603_356_736_u64.into()
                )
        ));
        assert!(matches!(
            evaluator.kernel.send(
                evaluator.runtime.registry(),
                Value::Class(
                    evaluator
                        .kernel
                        .class(iris_runtime::BuiltinClass::Float64)
                        .map_err(crate::EvaluationError::Runtime)?,
                ),
                iris_runtime::NativeSelector::FromBits,
                &[Value::Integer(0x8000_0000_0000_0000_u64.into())],
            ),
            Ok(Value::Float64(value)) if value.to_bits() == 0x8000_0000_0000_0000
        ));
        Ok(())
    }
}
