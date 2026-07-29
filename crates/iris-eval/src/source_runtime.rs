use std::collections::HashMap;

use iris_runtime::{
    ClassError, ClassId, DispatchError, DispatchOutcome, Kernel, Method, MethodBody, MethodOwner,
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

struct SourceEvaluator {
    runtime: Runtime,
    kernel: Kernel,
    names: HashMap<String, Binding>,
    selectors: HashMap<String, Selector>,
    bodies: HashMap<u64, MethodDeclaration>,
    module_names: HashMap<String, ModuleId>,
    module_classes: HashMap<ModuleId, ClassId>,
    module_methods: HashMap<(ModuleId, Selector), Method>,
    property_methods: HashMap<iris_runtime::MethodId, bool>,
    mixins: HashMap<ClassId, Vec<ClassId>>,
    static_superclasses: HashMap<ClassId, Option<ClassId>>,
    lexical_class: Option<ClassId>,
    current_method: Option<Method>,
    active_exception: Option<Value>,
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

fn execution_error(error: EvaluationError) -> iris_runtime::ExecutionError {
    match error {
        EvaluationError::Raised(value) => iris_runtime::ExecutionError::Raised(value),
        _ => iris_runtime::ExecutionError::Raised(Value::Nil),
    }
}

impl SourceEvaluator {
    fn new() -> Result<Self, EvaluationError> {
        Ok(Self {
            runtime: Runtime::new(),
            kernel: Kernel::new().map_err(EvaluationError::Runtime)?,
            names: HashMap::new(),
            selectors: HashMap::new(),
            bodies: HashMap::new(),
            module_names: HashMap::new(),
            module_classes: HashMap::new(),
            module_methods: HashMap::new(),
            property_methods: HashMap::new(),
            mixins: HashMap::new(),
            static_superclasses: HashMap::new(),
            lexical_class: None,
            current_method: None,
            active_exception: None,
            next_selector: 1_000,
            next_body: 10_000,
        })
    }

    fn program(&mut self, program: &Program) -> Result<Value, EvaluationError> {
        let mut values = Vec::new();
        for entry in &program.entries {
            match entry {
                ProgramEntry::Declaration(iris_syntax::Declaration::Class(class)) => {
                    self.class(class)?;
                }
                ProgramEntry::Declaration(iris_syntax::Declaration::Module(module)) => {
                    self.module(module)?;
                }
                ProgramEntry::Declaration(iris_syntax::Declaration::Contract(_)) => {
                    return Err(EvaluationError::UnsupportedConstruct);
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
            None => None,
        };
        let mut mixins = Vec::new();
        for mixin in &declaration.mixins {
            match mixin {
                iris_syntax::TypeExpression::Name(name) => {
                    if let Some(class) = self.class_name(name)? {
                        mixins.push(class);
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
                    .kernel
                    .registry_mut()
                    .open(class)
                    .map_err(EvaluationError::Class)?;
                candidate.replace_runtime_superclass(Some(replacement));
                self.kernel
                    .registry_mut()
                    .publish(candidate)
                    .map_err(EvaluationError::Class)?;
                return Ok(());
            }
            if superclass.is_some() || !mixins.is_empty() {
                return Err(EvaluationError::UnsupportedConstruct);
            }
            class
        } else {
            let class = self
                .runtime
                .registry_mut()
                .define_class(StaticSpine::new(1), superclass)
                .map_err(EvaluationError::Class)?;
            self.class_method(
                class,
                false,
                false,
                &MethodDeclaration {
                    decorators: Vec::new(),
                    is_override: false,
                    kind: MethodKind::Instance,
                    selector: "to_bool".into(),
                    parameters: Vec::new(),
                    visibility: iris_syntax::Visibility::Public,
                    body: vec![Statement::Expression(Expression::Literal("true".into()))],
                },
            )?;
            self.names.insert(
                declaration.name.clone(),
                Binding::immutable(Value::Class(class)),
            );
            self.mixins.insert(class, mixins);
            self.static_superclasses.insert(class, superclass);
            class
        };
        let builtin = declaration.reopen && builtin(&declaration.name, &self.kernel).is_some();
        self.publish_decorators(class, &declaration.decorators)?;
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
                    self.class_method(class, builtin, declaration.reopen, method)?;
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
                let defined = if builtin {
                    self.kernel
                        .registry_mut()
                        .publish_decorated_method(
                            class,
                            selector,
                            body,
                            visibility(method),
                            decorators,
                        )
                        .map_err(EvaluationError::Class)?
                } else {
                    self.runtime
                        .registry_mut()
                        .publish_decorated_method(
                            class,
                            selector,
                            body,
                            visibility(method),
                            decorators,
                        )
                        .map_err(EvaluationError::Class)?
                };
                self.property_methods.insert(defined.id(), false);
            }
            MethodKind::Class => {
                if builtin {
                    self.kernel
                        .registry_mut()
                        .publish_singleton_method(class, selector, body, visibility(method))
                        .map_err(EvaluationError::Class)?;
                } else {
                    self.runtime
                        .registry_mut()
                        .publish_singleton_method(class, selector, body, visibility(method))
                        .map_err(EvaluationError::Class)?;
                }
            }
            MethodKind::Property => {
                let method_visibility = visibility(method);
                let defined = if builtin {
                    self.kernel
                        .registry_mut()
                        .publish_decorated_method(
                            class,
                            selector,
                            body,
                            method_visibility,
                            decorators,
                        )
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
                .kernel
                .registry_mut()
                .dispatch_class_object(class, selector),
            (false, MethodKind::Class) => self
                .runtime
                .registry()
                .dispatch_class_object(class, selector),
            (true, MethodKind::Instance | MethodKind::Property) => {
                self.kernel.registry_mut().dispatch(class, selector)
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
            kind: MethodKind::Property,
            selector: format!("{name}="),
            parameters: vec!["value".into()],
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

    fn module(&mut self, declaration: &ModuleDeclaration) -> Result<(), EvaluationError> {
        let module = self
            .runtime
            .registry_mut()
            .define_module(&[])
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
                    if method.kind != MethodKind::Module {
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
                    Some(raise) => self.expression(&raise.value, locals, receiver)?,
                    None => self
                        .active_exception
                        .clone()
                        .ok_or(EvaluationError::UnsupportedConstruct)?,
                };
                Err(EvaluationError::Raised(value))
            }
            Statement::Try {
                body,
                catches,
                finally,
            } => self.try_statement(body, catches, finally, locals, receiver),
            Statement::SharedBinding { .. }
            | Statement::StoredProperty { .. }
            | Statement::Method(_)
            | Statement::Return(_)
            | Statement::Break { .. }
            | Statement::Continue(_)
            | Statement::While { .. }
            | Statement::For { .. }
            | Statement::Match { .. } => Err(EvaluationError::UnsupportedConstruct),
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
                Statement::SharedBinding { .. }
                | Statement::StoredProperty { .. }
                | Statement::Method(_)
                | Statement::Return(_)
                | Statement::Break { .. }
                | Statement::Continue(_)
                | Statement::While { .. }
                | Statement::For { .. }
                | Statement::Match { .. } => return Err(EvaluationError::UnsupportedConstruct),
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
            self.active_exception = pending;
            let final_result = self.block(finally, locals, receiver);
            self.active_exception = previous;
            final_result?;
        }
        result
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
                    Expression::Name(_) => self
                        .expression(callee, locals, receiver)
                        .and_then(|value| self.call(value, &arguments)),
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
            Expression::Assignment { left, right, .. } => {
                if let Expression::Name(name) = left.as_ref() {
                    if locals.contains_key(name) {
                        return Err(EvaluationError::ImmutableBinding);
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
                let Value::Object(object) = self.expression(target, locals, receiver.clone())?
                else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let value = self.expression(right, locals, receiver)?;
                let setter = self.selector(&format!("{selector}="));
                let method = match self.runtime.dispatch_instance(object, setter) {
                    Ok(method) => method,
                    Err(iris_runtime::ConstructionError::Dispatch(
                        iris_runtime::DispatchError::MissingMethod { .. },
                    )) => {
                        return Err(EvaluationError::Construction(
                            iris_runtime::ConstructionError::Dispatch(
                                iris_runtime::DispatchError::MissingMethod { selector: setter },
                            ),
                        ));
                    }
                    Err(error) => return Err(EvaluationError::Construction(error)),
                };
                self.invoke_method(method, Value::Object(object), &[value])
            }
        }
    }

    fn call(&mut self, value: Value, arguments: &[Value]) -> Result<Value, EvaluationError> {
        match value {
            Value::Class(class) => match self.kernel.construct(class, arguments) {
                Ok(value) => Ok(value),
                Err(iris_runtime::KernelError::Type) => {
                    self.construct(class, arguments).map(Value::Object)
                }
                Err(error) => Err(EvaluationError::Runtime(error)),
            },
            Value::BoundMethod(bound) => self.invoke_method(
                bound.method(),
                match bound.receiver() {
                    iris_runtime::BoundReceiver::Class(class) => Value::Class(class),
                    iris_runtime::BoundReceiver::Object(object) => Value::Object(object),
                },
                arguments,
            ),
            _ => Err(EvaluationError::UnsupportedConstruct),
        }
    }

    fn construct(
        &mut self,
        class: ClassId,
        arguments: &[Value],
    ) -> Result<iris_runtime::ObjectId, EvaluationError> {
        let mut runtime = std::mem::take(&mut self.runtime);
        let result =
            runtime.construct(class, arguments, |_runtime, method, receiver, arguments| {
                let declaration = self
                    .bodies
                    .get(&method.body().raw())
                    .cloned()
                    .ok_or(iris_runtime::ExecutionError::Raised(Value::Nil))?;
                if let Some(Statement::Expression(Expression::Assignment { left, right, .. })) =
                    declaration.body.last()
                    && matches!(left.as_ref(), Expression::RawIvar(_))
                    && let Expression::Literal(value) = right.as_ref()
                {
                    let value = literal(value)
                        .map_err(|_| iris_runtime::ExecutionError::Raised(Value::Nil))?;
                    return _runtime
                        .assign_raw_ivar(receiver, method.selector(), value)
                        .map_err(|_| iris_runtime::ExecutionError::Raised(Value::Nil));
                }
                self.invoke_method(method, Value::Object(receiver), arguments)
                    .map_err(execution_error)
            });
        self.runtime = runtime;
        result.map_err(construction_error)
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
            Value::Class(class) => {
                let selector_id = self.selector(selector);
                if self.is_builtin_class(class) {
                    match self
                        .kernel
                        .dispatch_class_object(class, selector_id)
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
                                    .send(Value::Class(class), native, arguments)
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

    fn class_name(&self, name: &str) -> Result<Option<ClassId>, EvaluationError> {
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

    fn is_builtin_class(&self, class: ClassId) -> bool {
        if self.runtime.registry().active(class).is_ok() {
            return false;
        }
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

    fn class_dispatch(
        &self,
        class: ClassId,
        selector: Selector,
    ) -> Result<iris_runtime::DispatchOutcome, EvaluationError> {
        if self.is_builtin_class(class) {
            return self
                .kernel
                .dispatch_class_object(class, selector)
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
        let selector_id = iris_runtime::NativeSelector::from_source(selector)
            .map_or_else(|| self.selector(selector), iris_runtime::NativeSelector::id);
        match self
            .kernel
            .dispatch_value(&receiver, selector_id)
            .map_err(EvaluationError::Runtime)?
        {
            iris_runtime::DispatchOutcome::Invoke(method) => {
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
            return Selector::new(1);
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

    fn resolve_instance_method(
        &self,
        object: iris_runtime::ObjectId,
        selector: Selector,
    ) -> Result<Method, EvaluationError> {
        match self.runtime.dispatch_instance(object, selector) {
            Ok(method) => Ok(method),
            Err(iris_runtime::ConstructionError::Dispatch(
                iris_runtime::DispatchError::MissingMethod { .. },
            )) => self
                .mixins
                .get(
                    &self
                        .runtime
                        .class_of(object)
                        .map_err(EvaluationError::Construction)?,
                )
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
        let mut locals = HashMap::new();
        for (parameter, argument) in parameters.iter().zip(arguments) {
            locals.insert(parameter.clone(), argument.clone());
        }
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

fn receiver_class_name(value: &Value) -> &'static str {
    match value {
        Value::Nil => "Nil",
        Value::Bool(_) => "Bool",
        Value::Integer(_) => "Integer",
        Value::Float32(_) => "Float32",
        Value::Float64(_) => "Float64",
        Value::Array(_) => "Array",
        Value::Symbol(_) => "Symbol",
        Value::Class(_) => "Class",
        Value::Object(_) => "Object",
        Value::BoundMethod(_) => "BoundMethod",
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
            .kernel
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
                .kernel
                .registry_mut()
                .active_revision(integer)
                .map_err(crate::EvaluationError::Class)?,
            before
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
