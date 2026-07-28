use std::collections::HashMap;

use iris_runtime::{ClassId, Kernel, Method, MethodBody, Runtime, Selector, StaticSpine, Value};
use iris_syntax::{
    BinaryOperator, ClassDeclaration, Expression, MethodDeclaration, MethodKind, Program, Statement,
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
    class_methods: HashMap<(ClassId, Selector), Method>,
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
            class_methods: HashMap::new(),
            mixins: HashMap::new(),
            next_selector: 1_000,
            next_body: 1,
        })
    }

    fn program(&mut self, program: &Program) -> Result<Value, EvaluationError> {
        for declaration in &program.declarations {
            let iris_syntax::Declaration::Class(class) = declaration else {
                return Err(EvaluationError::UnsupportedConstruct);
            };
            self.class(class)?;
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
        for statement in &declaration.body {
            let Statement::Method(method) = statement else {
                return Err(EvaluationError::UnsupportedConstruct);
            };
            let body = self.register_body(method.clone());
            let selector = self.selector(&method.selector);
            match method.kind {
                MethodKind::Instance => {
                    self.runtime
                        .registry_mut()
                        .publish_method(class, selector, body, visibility(method))
                        .map_err(EvaluationError::Class)?;
                }
                MethodKind::Class => {
                    self.class_methods.insert(
                        (class, selector),
                        Method::new(
                            iris_runtime::MethodId::new(body.raw()),
                            iris_runtime::MethodOwner::Class(class),
                            selector,
                            body,
                            visibility(method),
                        ),
                    );
                }
                MethodKind::Property => {
                    let method_visibility = visibility(method);
                    self.runtime
                        .registry_mut()
                        .publish_method(class, selector, body, method_visibility)
                        .map_err(EvaluationError::Class)?;
                }
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
            Statement::Binding { name, value } => {
                let value = self.expression(value, locals, receiver)?;
                self.names.insert(name.clone(), value.clone());
                Ok(value)
            }
            Statement::Expression(expression) => self.expression(expression, locals, receiver),
            Statement::Method(_)
            | Statement::Return(_)
            | Statement::Break { .. }
            | Statement::Continue(_)
            | Statement::While { .. }
            | Statement::For { .. }
            | Statement::Match { .. } => Err(EvaluationError::UnsupportedConstruct),
        }
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
                self.send(target, selector, &[])
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
                let class = self
                    .runtime
                    .class_of(object)
                    .map_err(EvaluationError::Construction)?;
                let method = match self.runtime.dispatch_instance(object, setter) {
                    Ok(method) => method,
                    Err(iris_runtime::ConstructionError::Dispatch(
                        iris_runtime::DispatchError::MissingMethod { .. },
                    )) => self.class_methods.get(&(class, setter)).copied().ok_or(
                        EvaluationError::Construction(iris_runtime::ConstructionError::Dispatch(
                            iris_runtime::DispatchError::MissingMethod { selector: setter },
                        )),
                    )?,
                    Err(error) => return Err(EvaluationError::Construction(error)),
                };
                self.invoke_method(method, Value::Object(object), &[value])
            }
        }
    }

    fn call(&mut self, value: Value, arguments: &[Value]) -> Result<Value, EvaluationError> {
        match value {
            Value::Class(class) => self.construct(class, arguments).map(Value::Object),
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
            .construct(class, arguments, |_, method, receiver, arguments| {
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
                let method = self
                    .class_methods
                    .get(&(class, selector_id))
                    .copied()
                    .ok_or(EvaluationError::MessageNotFound {
                        receiver_class: "Class".into(),
                        selector: selector.into(),
                    })?;
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
        if matches!(declaration.body.last(), Some(Statement::Expression(Expression::Call { callee, .. })) if matches!(callee.as_ref(), Expression::Name(name) if name == "super"))
        {
            let Value::Object(object) = receiver else {
                return Err(EvaluationError::UnsupportedConstruct);
            };
            let class = self
                .runtime
                .class_of(object)
                .map_err(EvaluationError::Construction)?;
            let successor = self
                .runtime
                .registry()
                .dispatch_super(class, method)
                .map_err(|error| {
                    let raised = match error {
                        iris_runtime::DispatchError::NoSuperMethod { .. } => {
                            Value::Symbol("NoSuperMethodError".into())
                        }
                        _ => Value::Nil,
                    };
                    EvaluationError::Execution(iris_runtime::ExecutionError::Raised(raised))
                })?;
            return self.invoke_method(successor, Value::Object(object), arguments);
        }
        invoke(&self.bodies, method, receiver, arguments).map_err(EvaluationError::Execution)
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
    }
}
