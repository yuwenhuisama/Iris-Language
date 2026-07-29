use std::collections::HashMap;

use iris_runtime::{
    ClassId, Kernel, Method, MethodBody, ModuleId, Runtime, Selector, StaticSpine, Truthiness,
    TruthinessError, TruthinessMethod, Value,
};
use iris_syntax::{
    BinaryOperator, ClassDeclaration, Expression, MethodDeclaration, MethodKind, ModuleDeclaration,
    Program, Statement,
};

use crate::EvaluationError;
use crate::source_method::{builtin, invoke, literal, visibility};

pub(super) fn evaluate(program: &Program) -> Result<Value, EvaluationError> {
    let mut evaluator = SourceEvaluator::new()?;
    evaluator.program(program)
}

struct SourceEvaluator {
    runtime: Runtime,
    kernel: Kernel,
    names: HashMap<String, Value>,
    selectors: HashMap<String, Selector>,
    bodies: HashMap<u64, MethodDeclaration>,
    module_names: HashMap<String, ModuleId>,
    module_methods: HashMap<(ModuleId, Selector), Method>,
    property_methods: HashMap<iris_runtime::MethodId, bool>,
    mixins: HashMap<ClassId, Vec<ClassId>>,
    next_selector: u64,
    next_body: u64,
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
            module_methods: HashMap::new(),
            property_methods: HashMap::new(),
            mixins: HashMap::new(),
            next_selector: 1_000,
            next_body: 1,
        })
    }

    fn program(&mut self, program: &Program) -> Result<Value, EvaluationError> {
        for declaration in &program.declarations {
            match declaration {
                iris_syntax::Declaration::Class(class) => self.class(class)?,
                iris_syntax::Declaration::Module(module) => self.module(module)?,
                iris_syntax::Declaration::Contract(_) => {
                    return Err(EvaluationError::UnsupportedConstruct);
                }
            }
        }
        let mut values = Vec::new();
        for statement in &program.statements {
            let value = self.statement(statement, &HashMap::new(), None)?;
            if !matches!(statement, Statement::Binding { .. } | Statement::Method(_)) {
                values.push(value);
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
            if superclass.is_some() || !mixins.is_empty() {
                return Err(EvaluationError::UnsupportedConstruct);
            }
            self.class_name(&declaration.name)?
                .ok_or(EvaluationError::UnsupportedConstruct)?
        } else {
            let class = self
                .runtime
                .registry_mut()
                .define_class(StaticSpine::new(1), superclass)
                .map_err(EvaluationError::Class)?;
            self.names
                .insert(declaration.name.clone(), Value::Class(class));
            self.mixins.insert(class, mixins);
            class
        };
        self.publish_decorators(class, &declaration.decorators)?;
        for statement in &declaration.body {
            match statement {
                Statement::StoredProperty {
                    decorators,
                    name,
                    initializer,
                } => {
                    self.stored_property(class, decorators, name, initializer.clone())?;
                }
                Statement::Method(method) => self.class_method(class, method)?,
                _ => return Err(EvaluationError::UnsupportedConstruct),
            }
        }
        Ok(())
    }

    fn class_method(
        &mut self,
        class: ClassId,
        method: &MethodDeclaration,
    ) -> Result<(), EvaluationError> {
        let body = self.register_body(method.clone());
        let selector = self.selector(&method.selector);
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
                let defined = self
                    .runtime
                    .registry_mut()
                    .publish_decorated_method(class, selector, body, method_visibility, decorators)
                    .map_err(EvaluationError::Class)?;
                self.property_methods.insert(defined.id(), true);
            }
            MethodKind::Module => return Err(EvaluationError::UnsupportedConstruct),
        }
        Ok(())
    }

    fn stored_property(
        &mut self,
        class: ClassId,
        decorators: &[iris_syntax::Decorator],
        name: &str,
        initializer: Expression,
    ) -> Result<(), EvaluationError> {
        let getter = MethodDeclaration {
            decorators: Vec::new(),
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
        self.class_method(class, &getter)?;
        self.class_method(class, &setter)?;
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
        for statement in &declaration.body {
            let Statement::Method(method) = statement else {
                return Err(EvaluationError::UnsupportedConstruct);
            };
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
        Ok(())
    }

    fn statement(
        &mut self,
        statement: &Statement,
        locals: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<Value, EvaluationError> {
        match statement {
            Statement::Binding { name, value } => {
                let value = self.expression(value, locals, receiver)?;
                self.names.insert(name.clone(), value.clone());
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
            Statement::StoredProperty { .. }
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
                Statement::Binding { name, value } => {
                    let value = self.expression(value, &locals, receiver.clone())?;
                    locals.insert(name.clone(), value);
                }
                Statement::Expression(_) | Statement::If { .. } => {
                    result = self.statement(statement, &locals, receiver.clone())?;
                }
                Statement::StoredProperty { .. }
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

    fn condition(
        &mut self,
        expression: &Expression,
        locals: &HashMap<String, Value>,
        receiver: Option<Value>,
    ) -> Result<bool, EvaluationError> {
        let value = self.expression(expression, locals, receiver)?;
        let to_bool = self.selector("to_bool");
        let method = match value {
            Value::Object(object) => match self.resolve_instance_method(object, to_bool) {
                Ok(method) => TruthinessMethod::Returns(self.invoke_method(
                    method,
                    Value::Object(object),
                    &[],
                )?),
                Err(EvaluationError::Construction(iris_runtime::ConstructionError::Dispatch(
                    iris_runtime::DispatchError::MissingMethod { .. },
                ))) => TruthinessMethod::Default,
                Err(error) => return Err(error),
            },
            _ => TruthinessMethod::Default,
        };
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
            Expression::Name(name) => locals
                .get(name)
                .or_else(|| self.names.get(name))
                .cloned()
                .or_else(|| (name == "self").then_some(receiver).flatten())
                .or_else(|| builtin(name, &self.kernel))
                .ok_or(EvaluationError::UnsupportedConstruct),
            Expression::RawIvar(name) => {
                let Value::Object(object) =
                    receiver.ok_or(EvaluationError::UnsupportedConstruct)?
                else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                let selector = self.selector(name);
                self.runtime
                    .raw_ivar(object, selector)
                    .map_err(EvaluationError::Construction)
            }
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
                    Expression::Name(name) => self
                        .names
                        .get(name)
                        .cloned()
                        .map_or(Err(EvaluationError::UnsupportedConstruct), |value| {
                            self.call(value, &arguments)
                        }),
                    _ => Err(EvaluationError::UnsupportedConstruct),
                }
            }
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.expression(left, locals, receiver.clone())?;
                let right = self.expression(right, locals, receiver)?;
                let selector = match operator {
                    BinaryOperator::Multiply => "*",
                    BinaryOperator::NamedInfix { selector } => selector,
                    BinaryOperator::Identity => {
                        return Ok(Value::Bool(left == right));
                    }
                    _ => return Err(EvaluationError::UnsupportedConstruct),
                };
                self.send(left, selector, &[right])
            }
            Expression::Unary { .. } | Expression::ContractView { .. } => {
                Err(EvaluationError::UnsupportedConstruct)
            }
            Expression::Assignment { left, right, .. } => {
                if let Expression::RawIvar(name) = left.as_ref() {
                    let Value::Object(object) =
                        receiver.ok_or(EvaluationError::UnsupportedConstruct)?
                    else {
                        return Err(EvaluationError::UnsupportedConstruct);
                    };
                    let value = self.expression(right, locals, Some(Value::Object(object)))?;
                    let selector = self.selector(name);
                    return self
                        .runtime
                        .assign_raw_ivar(object, selector, value)
                        .map_err(EvaluationError::Construction);
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
            Value::Class(class) => self.construct(class, arguments).map(Value::Object),
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
        let bodies = &self.bodies;
        self.runtime
            .construct(class, arguments, |runtime, method, receiver, arguments| {
                let Some(declaration) = bodies.get(&method.body().raw()) else {
                    return Err(iris_runtime::ExecutionError::Raised(Value::Nil));
                };
                if let Some(Statement::Expression(Expression::Assignment { left, right, .. })) =
                    declaration.body.last()
                    && matches!(left.as_ref(), Expression::RawIvar(_))
                    && let Expression::Literal(value) = right.as_ref()
                {
                    let value = literal(value)
                        .map_err(|_| iris_runtime::ExecutionError::Raised(Value::Nil))?;
                    return runtime
                        .assign_raw_ivar(receiver, method.selector(), value)
                        .map_err(|_| iris_runtime::ExecutionError::Raised(Value::Nil));
                }
                invoke(bodies, method, Value::Object(receiver), arguments)
            })
            .map_err(EvaluationError::Construction)
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
                let method = match self
                    .runtime
                    .registry()
                    .dispatch_class_object(class, selector_id)
                    .map_err(iris_runtime::ConstructionError::from)
                    .map_err(EvaluationError::Construction)?
                {
                    iris_runtime::DispatchOutcome::Invoke(method) => method,
                    iris_runtime::DispatchOutcome::WouldInvokeMethodMissing { .. } => {
                        return Err(EvaluationError::MessageNotFound {
                            receiver_class: "Class".into(),
                            selector: selector.into(),
                        });
                    }
                };
                self.invoke_method(method, Value::Class(class), arguments)
            }
            Value::Object(object) => {
                let selector = self.selector(selector);
                let method = self.resolve_instance_method(object, selector)?;
                self.invoke_method(method, Value::Object(object), arguments)
            }
            value => iris_runtime::NativeSelector::from_source(selector).map_or(
                Err(EvaluationError::MessageNotFound {
                    receiver_class: receiver_class_name(&value).into(),
                    selector: selector.into(),
                }),
                |native| {
                    self.kernel
                        .send(value, native, arguments)
                        .map_err(EvaluationError::Runtime)
                },
            ),
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
            Some(Value::Class(class)) => Ok(Some(*class)),
            Some(_) => Err(EvaluationError::UnsupportedConstruct),
            None => Ok(None),
        }
    }

    fn selector(&mut self, name: &str) -> Selector {
        if let Some(selector) = self.selectors.get(name) {
            return *selector;
        }
        let selector = Selector::new(self.next_selector);
        self.next_selector += 1;
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

    fn invoke_method(
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
        let raw_ivar = match declaration.body.last() {
            Some(Statement::Expression(Expression::RawIvar(name))) => Some(name.clone()),
            _ => None,
        };
        if let Some(name) = raw_ivar {
            let Value::Object(object) = receiver else {
                return Err(EvaluationError::UnsupportedConstruct);
            };
            let selector = self.selector(&name);
            return self
                .runtime
                .raw_ivar(object, selector)
                .map_err(EvaluationError::Construction);
        }
        if matches!(declaration.body.last(), Some(Statement::Expression(Expression::Name(name))) if name == "super")
        {
            return Err(EvaluationError::Runtime(
                iris_runtime::KernelError::Dispatch(iris_runtime::DispatchError::InvalidSuper {
                    selector: method.selector(),
                }),
            ));
        }
        if matches!(declaration.body.last(), Some(Statement::Expression(Expression::Call { callee, .. })) if matches!(callee.as_ref(), Expression::Name(name) if name == "super"))
        {
            let successor = match receiver {
                Value::Object(object) => {
                    let class = self
                        .runtime
                        .class_of(object)
                        .map_err(EvaluationError::Construction)?;
                    self.runtime
                        .registry()
                        .dispatch_super(class, method)
                        .map_err(iris_runtime::KernelError::from)
                        .map_err(EvaluationError::Runtime)?
                }
                Value::Class(class) => self
                    .runtime
                    .registry()
                    .dispatch_class_object_super(class, method)
                    .map_err(iris_runtime::KernelError::from)
                    .map_err(EvaluationError::Runtime)?,
                _ => return Err(EvaluationError::UnsupportedConstruct),
            };
            return self.invoke_method(successor, receiver, arguments);
        }
        let parameters = declaration.parameters.clone();
        let body = declaration.body.clone();
        let mut locals = HashMap::new();
        for (parameter, argument) in parameters.iter().zip(arguments) {
            locals.insert(parameter.clone(), argument.clone());
        }
        self.block(&body, &locals, Some(receiver))
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
