//! Runtime class registration and exception matching.

use iris_runtime::{
    BuiltinClass, ClassError, ClassId, DecoratorTransform, KernelError, MethodBody, StaticSpine,
    Value, Visibility,
};

use crate::compile::Program;

use super::{Machine, MachineError, literal_runtime_value, selector_id};
use crate::VerifyError;
use crate::compile::Instruction;

impl Machine {
    pub(super) fn invoke_function(
        &mut self,
        function: usize,
        arguments: Vec<Value>,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        let callee = program
            .functions
            .get(function)
            .cloned()
            .ok_or(MachineError::Invalid(super::VerifyError::UnknownFunction {
                function,
            }))?;
        if arguments.len() != callee.parameters {
            return Err(MachineError::Kernel(KernelError::Arity));
        }
        if !callee.is_async {
            let returned = self.run_body(
                &callee.instructions,
                callee.registers,
                arguments,
                program,
                classes,
            )?;
            return Ok(returned.into_iter().next().unwrap_or(Value::Nil));
        }
        self.spawn_task(
            function,
            callee.registers,
            &callee.instructions,
            arguments,
            program,
            classes,
        )
    }

    /// Runs an ASYNC body eagerly and answers its Task.
    ///
    /// The body runs at CALL time - `C012` makes creating the Task and
    /// starting its first run one operation - so a body with no `await` is
    /// already finished when the Task is answered. A body that parks on a
    /// pending Gate is neither finished nor failed, and its outcome is
    /// recorded only when the Gate completes.
    pub(super) fn spawn_task(
        &mut self,
        function: usize,
        registers: usize,
        instructions: &[Instruction],
        arguments: Vec<Value>,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        self.async_depth += 1;
        let outcome = self
            .run_body(instructions, registers, arguments, program, classes)
            .map(|returned| returned.into_iter().next().unwrap_or(Value::Nil));
        self.async_depth -= 1;
        let identity = iris_runtime::ObjectId::new(self.next_context);
        self.next_context = self.next_context.saturating_add(1);
        // A body that SUSPENDED is not finished and not failed: it is parked
        // on a Gate. The Task is answered now, and the outcome is recorded
        // only when the Gate completes and the frame runs to its end.
        if let Err(MachineError::Suspended(_)) = outcome {
            if let Some(mut frame) = self.pending_frame.take() {
                if frame.function.is_none() {
                    frame.function = Some(function);
                }
                self.suspended.push(super::SuspendedTask {
                    identity,
                    frame,
                    function,
                });
            }
            return Ok(Value::Task(identity));
        }
        if outcome.is_err() {
            self.unobserved_failures.push(identity);
        }
        self.tasks.insert(identity, outcome.map_err(Box::new));
        Ok(Value::Task(identity))
    }

    /// Runs every parked frame whose Gate has COMPLETED, in suspension order.
    pub(super) fn drive_ready(
        &mut self,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<(), MachineError> {
        let ready: Vec<iris_runtime::ObjectId> = self
            .suspended
            .iter()
            .filter(|task| matches!(self.gates.get(&task.frame.gate), Some(Some(_))))
            .map(|task| task.frame.gate)
            .collect();
        for gate in ready {
            self.resume_gate(gate, program, classes)?;
        }
        Ok(())
    }

    /// Resumes every frame parked on `gate`, in SUSPENSION order.
    ///
    /// `IRIS-V1-ASYNC-C014` fixes that order, so two tasks awaiting one Gate
    /// observe their effects in the order they paused rather than in whatever
    /// order a map happened to hold them.
    pub(super) fn resume_gate(
        &mut self,
        gate: iris_runtime::ObjectId,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<(), MachineError> {
        let posted = self
            .gates
            .get(&gate)
            .cloned()
            .flatten()
            .unwrap_or(Value::Nil);
        let mut ready = Vec::new();
        let mut held = Vec::new();
        for task in std::mem::take(&mut self.suspended) {
            if task.frame.gate == gate {
                ready.push(task);
            } else {
                held.push(task);
            }
        }
        self.suspended = held;
        for task in ready {
            let super::SuspendedTask {
                identity,
                mut frame,
                function,
            } = task;
            let cleanup = frame.cleanup.clone();
            let mut frames = std::mem::take(&mut frame.continuations);
            frames.reverse();
            frames.push(frame);
            self.pending_frame = None;
            self.async_depth += 1;
            let outcome = frames.into_iter().try_fold(posted.clone(), |value, frame| {
                let function = frame.function.unwrap_or(function);
                let callee =
                    program
                        .functions
                        .get(function)
                        .cloned()
                        .ok_or(MachineError::Invalid(super::VerifyError::UnknownFunction {
                            function,
                        }))?;
                self.run_frame_from(
                    &callee.instructions,
                    callee.registers,
                    Vec::new(),
                    program,
                    classes,
                    Some((frame, value)),
                )
                .map(|returned| returned.into_iter().next().unwrap_or(Value::Nil))
            });
            self.async_depth -= 1;
            let outcome = match cleanup {
                Some(resource) if !matches!(outcome, Err(MachineError::Suspended(_))) => {
                    self.close_after(resource, outcome, program, classes)
                }
                _ => outcome,
            };
            // A resumed frame may await AGAIN, on another Gate, so it parks
            // once more under the same Task identity rather than completing.
            if let Err(MachineError::Suspended(_)) = outcome {
                if let Some(frame) = self.pending_frame.take() {
                    self.suspended.push(super::SuspendedTask {
                        identity,
                        frame,
                        function,
                    });
                }
                continue;
            }
            if outcome.is_err() {
                self.unobserved_failures.push(identity);
            }
            self.tasks.insert(identity, outcome.map_err(Box::new));
        }
        Ok(())
    }

    pub(super) fn observe_task(
        &mut self,
        task: Value,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        let Value::Task(identity) = task else {
            return Err(MachineError::Kernel(KernelError::Type));
        };
        // Observing is what DRIVES a parked frame: `C014` readies a frame when
        // its Gate completes, but the continuation runs here, so the effect
        // becomes visible only once something observes the Task.
        self.drive_ready(program, classes)?;
        let outcome = self
            .tasks
            .get(&identity)
            .cloned()
            .ok_or(MachineError::UnsupportedConstruct)?;
        self.unobserved_failures.retain(|held| *held != identity);
        outcome.map_err(|error| *error)
    }

    pub(super) fn invoke_using(
        &mut self,
        resource: Value,
        block: Value,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        let Value::Closure(callback) = block else {
            return Err(MachineError::Kernel(KernelError::Type));
        };
        let outcome = self.invoke_closure_value(callback, &[], program, classes);
        // `C013` makes a suspension a REGISTERED CONTINUATION, not an exit, so
        // the protected region has not been left and the resource must stay
        // open. Closing here would run cleanup once on the way out and again
        // on the replayed re-entry, which is the double close `V084` forbids.
        if matches!(outcome, Err(MachineError::Suspended(_))) {
            if let Some(frame) = self.pending_frame.as_mut() {
                frame.cleanup = Some(resource);
            }
            return outcome;
        }
        self.close_after(resource, outcome, program, classes)
    }

    pub(super) fn close_after(
        &mut self,
        resource: Value,
        outcome: Result<Value, MachineError>,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        let closed = self.close_resource(resource, program, classes);
        match (outcome, closed) {
            (Ok(value), Ok(())) => Ok(value),
            (Ok(_), Err(error)) => Err(error),
            (Err(MachineError::Raised(primary)), Err(MachineError::Raised(cleanup))) => {
                let (value, context) = *primary;
                let (_, cleanup_context) = *cleanup;
                let context = match context {
                    Value::ExceptionContext(
                        identity,
                        held,
                        cause,
                        mut suppressed,
                        sites,
                        location,
                    ) => {
                        suppressed.push(cleanup_context);
                        Value::ExceptionContext(identity, held, cause, suppressed, sites, location)
                    }
                    context => context,
                };
                Err(MachineError::Raised(Box::new((value, context))))
            }
            (Err(primary), _) => Err(primary),
        }
    }

    fn close_resource(
        &mut self,
        resource: Value,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<(), MachineError> {
        if let Value::ExternalResource(_) = &resource {
            return self.send("close", resource, &[]).map(|_| ());
        }
        let Value::Object(object) = resource else {
            return Err(MachineError::MessageNotFound {
                receiver_class: super::value_class_name(&resource).to_owned(),
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
            MachineError::Invalid(super::VerifyError::UnknownFunction {
                function: usize::MAX,
            })
        })?;
        let callee = program
            .functions
            .get(function)
            .cloned()
            .ok_or(MachineError::Invalid(super::VerifyError::UnknownFunction {
                function,
            }))?;
        self.run_body(
            &callee.instructions,
            callee.registers,
            vec![Value::Object(object)],
            program,
            classes,
        )?;
        Ok(())
    }

    pub(super) fn invoke_closure_value(
        &mut self,
        callback: iris_runtime::ObjectId,
        arguments: &[Value],
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        let closure = self
            .closures
            .get(&callback)
            .cloned()
            .ok_or(MachineError::Kernel(KernelError::Type))?;
        let callee =
            program
                .functions
                .get(closure.function)
                .cloned()
                .ok_or(MachineError::Invalid(super::VerifyError::UnknownFunction {
                    function: closure.function,
                }))?;
        let mut passed = closure.captures;
        passed.extend_from_slice(arguments);
        self.closure_depth += 1;
        let returned = self.run_body(
            &callee.instructions,
            callee.registers,
            passed,
            program,
            classes,
        );
        self.closure_depth -= 1;
        if returned.is_err()
            && let Some(frame) = self.pending_frame.as_mut()
        {
            frame.function = Some(closure.function);
        }
        let returned = returned?;
        Ok(returned.into_iter().next().unwrap_or(Value::Nil))
    }

    pub(super) fn dynamic_selector(&mut self, name: &str) -> iris_runtime::Selector {
        if let Some(selector) = self.dynamic_selectors.get(name) {
            return *selector;
        }
        let selector = iris_runtime::Selector::new(self.next_dynamic_selector);
        self.next_dynamic_selector = self.next_dynamic_selector.saturating_add(1);
        self.dynamic_selectors.insert(name.to_owned(), selector);
        selector
    }

    pub(super) fn selector_name(
        &self,
        program: &Program,
        selector: iris_runtime::Selector,
    ) -> Option<String> {
        self.dynamic_selectors
            .iter()
            .find_map(|(name, known)| (*known == selector).then(|| name.clone()))
            .or_else(|| {
                program
                    .classes
                    .iter()
                    .flat_map(|class| {
                        class.methods.iter().chain(&class.class_methods).chain(
                            class.reopens.iter().flat_map(|reopen| {
                                reopen.methods.iter().chain(&reopen.class_methods)
                            }),
                        )
                    })
                    .map(|(name, _)| name.as_str())
                    .chain(
                        program
                            .classes
                            .iter()
                            .flat_map(|class| class.stored_properties.iter())
                            .map(|property| property.name.as_str()),
                    )
                    .chain(program.functions.iter().filter_map(|function| {
                        function.name.split_once('.').map(|(_, selector)| selector)
                    }))
                    .find(|name| selector_id(program, name).is_some_and(|known| known == selector))
                    .map(str::to_owned)
            })
    }

    pub(super) fn require_reflection(
        &self,
        operation: &str,
        target: ClassId,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<(), MachineError> {
        if self.reflection_grants.is_empty() {
            return Ok(());
        }
        let requested = format!("reflection.{operation}");
        let target_name = self.dispatch_class_name(program, classes, target);
        let granted = self.reflection_grants.iter().any(|(name, scope)| {
            name == &requested
                && (scope.is_empty()
                    || scope
                        .rsplit("::")
                        .next()
                        .is_some_and(|name| name == target_name))
        });
        if granted {
            Ok(())
        } else {
            Err(MachineError::ReflectionAccess)
        }
    }

    pub(super) fn register_classes(
        &mut self,
        program: &Program,
    ) -> Result<Vec<ClassId>, MachineError> {
        self.modules.clear();
        // A module is DECLARED, and also discovered from the names of its
        // functions - a module with methods but no declaration entry is still
        // registered, and one that only composes others still exists.
        // A name is held ONCE: `module N { } module N { }` declares the same
        // module twice, and a duplicate made the ordering loop below
        // unsatisfiable - `ordered` holds each name once, so it could never
        // reach a length that counted one name twice, and the run hung.
        let mut module_names: Vec<String> = Vec::new();
        for declaration in &program.modules {
            if !module_names.contains(&declaration.name) {
                module_names.push(declaration.name.clone());
            }
        }
        for declaration in &program.functions {
            let Some((module, _)) = declaration.name.split_once('.') else {
                continue;
            };
            if program.classes.iter().any(|class| class.name == module)
                || module_names.iter().any(|known| known == module)
            {
                continue;
            }
            module_names.push(module.to_owned());
        }
        // A module composing another must be defined AFTER it, so the edge
        // names an identity that already exists. Ordering by dependency is
        // what makes `module B mixin A { }` reachable through `C mixin B`.
        let mut ordered: Vec<String> = Vec::with_capacity(module_names.len());
        while ordered.len() < module_names.len() {
            let mut progressed = false;
            for name in &module_names {
                if ordered.iter().any(|known| known == name) {
                    continue;
                }
                let ready = program
                    .modules
                    .iter()
                    .find(|declaration| declaration.name == *name)
                    .is_none_or(|declaration| {
                        declaration.mixins.iter().all(|needed| {
                            ordered.iter().any(|known| known == needed)
                                || !module_names.iter().any(|known| known == needed)
                        })
                    });
                if ready {
                    ordered.push(name.clone());
                    progressed = true;
                }
            }
            // A CYCLE cannot be ordered, and the registry refuses one anyway,
            // so the remainder is defined without its edges rather than looping.
            if !progressed {
                for name in &module_names {
                    if !ordered.iter().any(|known| known == name) {
                        ordered.push(name.clone());
                    }
                }
            }
        }
        for module in &ordered {
            let module = module.as_str();
            let components: Vec<iris_runtime::ModuleId> = program
                .modules
                .iter()
                .find(|declaration| declaration.name == module)
                .map(|declaration| {
                    declaration
                        .mixins
                        .iter()
                        .filter_map(|needed| {
                            self.modules
                                .iter()
                                .find_map(|(name, id)| (name == needed).then_some(*id))
                        })
                        .collect()
                })
                .unwrap_or_default();
            let module_id = self
                .runtime
                .registry_mut()
                .define_module(&components)
                .map_err(MachineError::Class)?;
            self.modules.push((module.to_owned(), module_id));
            for (function, method) in program.functions.iter().enumerate() {
                let Some((owner, selector)) = method.name.split_once('.') else {
                    continue;
                };
                if owner != module {
                    continue;
                }
                let selector = selector_id(program, selector)
                    .ok_or_else(|| MachineError::UnknownSelector(selector.to_owned()))?;
                let method = self
                    .runtime
                    .registry_mut()
                    .define_module_method(
                        module_id,
                        selector,
                        MethodBody::new(function as u64),
                        Visibility::Public,
                    )
                    .map_err(MachineError::Class)?;
                self.remember_signature(method, program);
            }
        }
        let mut classes = Vec::with_capacity(program.classes.len());
        for (index, declaration) in program.classes.iter().enumerate() {
            let superclass = declaration
                .superclass
                .and_then(|parent| classes.get(parent).copied());
            // A mixin is resolved to a module IDENTITY here rather than at
            // compile time, because modules are registered in the same load
            // and the runtime composes them into the class's MRO.
            // A `private` edge grants the module reach into the class's
            // PRIVATE methods, so the marker travels into the composition
            // edge rather than being dropped at the name.
            let mut modules = Vec::with_capacity(declaration.mixins.len());
            for (name, private_access) in &declaration.mixins {
                // A mixin may name a CLASS rather than a module, and the class
                // then contributes its methods the way a module does - the
                // reference answers `:A` for a method only the mixed-in class
                // declares. A module is registered for it on first use, so the
                // MRO carries one identity per named class.
                let module = match self.modules.iter().find(|(known, _)| known == name) {
                    Some((_, module)) => *module,
                    None => {
                        let Some(source) = program.classes.iter().find(|class| class.name == *name)
                        else {
                            return Err(MachineError::NameError);
                        };
                        let module = self
                            .runtime
                            .registry_mut()
                            .define_module(&[])
                            .map_err(MachineError::Class)?;
                        for (selector, function) in &source.methods {
                            let selector = selector_id(program, selector)
                                .ok_or_else(|| MachineError::UnknownSelector(selector.clone()))?;
                            let method = self
                                .runtime
                                .registry_mut()
                                .define_module_method(
                                    module,
                                    selector,
                                    MethodBody::new(*function as u64),
                                    Visibility::Public,
                                )
                                .map_err(MachineError::Class)?;
                            self.remember_signature(method, program);
                        }
                        self.modules.push((name.clone(), module));
                        module
                    }
                };
                modules.push(iris_runtime::CompositionEdge::new(module, *private_access));
            }
            let class = self
                .runtime
                .registry_mut()
                .define_class_with_capabilities_and_composition_edges(
                    StaticSpine::new(index as u64 + 1),
                    superclass,
                    meta_capabilities(&declaration.meta_deny)?,
                    &modules,
                )
                .map_err(MachineError::Class)?;
            self.runtime
                .registry_mut()
                .begin_origin_transaction(class)
                .map_err(MachineError::Class)?;
            self.runtime
                .registry_mut()
                .stage_decorators(
                    class,
                    declaration.decorators.iter().map(|(identity, arguments)| {
                        DecoratorTransform::metadata(identity, arguments)
                    }),
                )
                .map_err(MachineError::Class)?;
            // `C024` forbids an OVERLOAD SET, so a second declaration of one
            // selector replaces the first and must write `override`. The
            // refusal is raised here rather than at compile time, because the
            // class identity it names exists only once the class is defined.
            // A class variable is ANCHORED once per ancestry, so a subclass
            // redeclaring one its superclass already anchors is a duplicate
            // rather than a fresh slot.
            if let Some(name) = declaration.duplicate_class_variable.as_ref() {
                let name = selector_id(program, name)
                    .ok_or_else(|| MachineError::UnknownSelector(name.clone()))?;
                return Err(MachineError::Class(ClassError::DuplicateClassVariable {
                    class,
                    name,
                }));
            }
            // `D-173` makes a replacement whose return Type contradicts a
            // contract requirement an INCOMPATIBLE member, not a new one.
            if declaration.contract_signature_clash {
                return Err(MachineError::TypeContractError);
            }
            if let Some(selector) = declaration.override_required.first() {
                let selector = selector_id(program, selector)
                    .ok_or_else(|| MachineError::UnknownSelector(selector.clone()))?;
                return Err(MachineError::Class(ClassError::OverrideRequired {
                    class,
                    selector,
                }));
            }
            for (selector, function) in &declaration.methods {
                // `C077` refuses a private method from every path but the
                // declaring class, so the declared visibility is published -
                // a send carries the class whose body wrote it, which is the
                // authority that decides the call.
                //
                // `initialize` is the exception: construction calls it on the
                // object's behalf rather than from any caller's frame, so a
                // class declaring it without `public` must still be
                // constructible.
                let visibility =
                    if declaration.private_methods.contains(selector) && selector != "initialize" {
                        Visibility::Private
                    } else {
                        Visibility::Public
                    };
                let selector = selector_id(program, selector)
                    .ok_or_else(|| MachineError::UnknownSelector(selector.clone()))?;
                let method = self
                    .runtime
                    .registry_mut()
                    .publish_origin_method(
                        class,
                        selector,
                        MethodBody::new(*function as u64),
                        visibility,
                    )
                    .map_err(MachineError::Class)?;
                self.remember_signature(method, program);
            }
            for property in &declaration.stored_properties {
                let selector = selector_id(program, &property.name)
                    .ok_or_else(|| MachineError::UnknownSelector(property.name.clone()))?;
                self.runtime
                    .registry_mut()
                    .publish_stored_property(class, selector, MethodBody::new(0))
                    .map_err(MachineError::Class)?;
            }
            for variable in &declaration.class_variables {
                let selector = selector_id(program, &variable.name)
                    .ok_or_else(|| MachineError::UnknownSelector(variable.name.clone()))?;
                let value = literal_runtime_value(&variable.initializer)?;
                self.runtime
                    .declare_class_var(class, selector, value, variable.mutable)
                    .map_err(MachineError::Construction)?;
                // A non-literal initializer RUNS on first read, so the slot is
                // declared now and the body is remembered rather than being
                // evaluated where the class is defined.
                if let Some(function) = variable.initializer_function {
                    self.pending_class_initializers
                        .insert((class, selector), function);
                }
            }
            for (selector, function) in &declaration.class_methods {
                let selector = selector_id(program, selector)
                    .ok_or_else(|| MachineError::UnknownSelector(selector.clone()))?;
                let method = self
                    .runtime
                    .registry_mut()
                    .publish_singleton_method(
                        class,
                        selector,
                        MethodBody::new(*function as u64),
                        Visibility::Public,
                    )
                    .map_err(MachineError::Class)?;
                self.remember_signature(method, program);
            }
            self.runtime
                .registry_mut()
                .commit_origin_transaction(class)
                .map_err(MachineError::Class)?;
            // A reopen is NOT applied here: it takes effect where it was
            // written, so `ApplyReopen` drives it from that position.
            classes.push(class);
        }
        // A reopen of a BUILT-IN class publishes onto the kernel's own class,
        // which has no entry in `classes`. That is what makes the added method
        // reachable on every value of that class rather than on a new one.
        for reopen in &program.builtin_reopens {
            let kind = match reopen.target.as_str() {
                "Object" => iris_runtime::BuiltinClass::Object,
                "Nil" => iris_runtime::BuiltinClass::Nil,
                "Bool" => iris_runtime::BuiltinClass::Bool,
                "Integer" => iris_runtime::BuiltinClass::Integer,
                "Float32" => iris_runtime::BuiltinClass::Float32,
                "Float64" => iris_runtime::BuiltinClass::Float64,
                _ => iris_runtime::BuiltinClass::String,
            };
            let class = self.kernel.class(kind).map_err(MachineError::Kernel)?;
            self.apply_members(
                (
                    class,
                    &crate::compile::ClassReopen {
                        methods: reopen.methods.clone(),
                        class_methods: Vec::new(),
                    },
                ),
                program,
            )?;
        }
        Ok(classes)
    }

    pub(super) fn catch_matches(
        &self,
        value: &Value,
        name: &str,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<bool, MachineError> {
        let filter = match name {
            "Symbol" => return Ok(matches!(value, Value::Symbol(_))),
            "Integer" => return Ok(matches!(value, Value::Integer(_))),
            "Nil" => return Ok(matches!(value, Value::Nil)),
            "Bool" => return Ok(matches!(value, Value::Bool(_))),
            "Object" => self
                .kernel
                .class(BuiltinClass::Object)
                .map_err(MachineError::Kernel),
            _ => program
                .classes
                .iter()
                .position(|class| class.name == name)
                .and_then(|index| classes.get(index).copied())
                .map_or_else(|| Err(MachineError::Kernel(KernelError::Type)), Ok),
        }?;
        let Value::Object(object) = value else {
            return Ok(false);
        };
        let mut class = self
            .runtime
            .class_of(*object)
            .map_err(MachineError::Construction)?;
        loop {
            if class == filter {
                return Ok(true);
            }
            let Some(superclass) = self
                .runtime
                .registry()
                .active(class)
                .map_err(MachineError::Class)?
                .runtime_superclass()
            else {
                return Ok(false);
            };
            class = superclass;
        }
    }

    pub(super) fn receiver_class(&self, receiver: &Value) -> Result<ClassId, MachineError> {
        match receiver {
            Value::Object(object) => self
                .runtime
                .class_of(*object)
                .map_err(MachineError::Construction),
            Value::Class(class) => Ok(*class),
            _ => Err(MachineError::Kernel(KernelError::Type)),
        }
    }

    /// Names the Class a dispatch failure happened on.
    ///
    /// The reference reports a missing selector as `MessageNotFound` naming
    /// the receiver's CLASS, while a raw dispatch error carries only an
    /// interned selector number. Two backends that both refuse a program but
    /// describe the refusal differently still DISAGREE, so the name is
    /// recovered from the program's class table.
    pub(super) fn dispatch_class_name(
        &self,
        program: &Program,
        classes: &[ClassId],
        class: ClassId,
    ) -> String {
        classes
            .iter()
            .position(|known| *known == class)
            .and_then(|index| program.classes.get(index))
            .map_or_else(|| "Object".to_owned(), |entry| entry.name.clone())
    }

    pub(super) fn initialize_properties(
        &mut self,
        program: &Program,
        classes: &[ClassId],
        class: ClassId,
        object: iris_runtime::ObjectId,
    ) -> Result<(), MachineError> {
        // A BUILT-IN class has no declaration entry, so it declares no stored
        // property and there is nothing to initialize - `Object.new()` is an
        // ordinary construction rather than a class the program never named.
        let Some(index) = classes.iter().position(|known| *known == class) else {
            return Ok(());
        };
        // A SUPERCLASS initializes its own properties first, which is the
        // order the reference produces: `[:base, :child]` rather than the
        // reverse. Walking up and then initializing downwards is what gives a
        // subclass initializer a fully built base to read.
        let mut lineage = vec![index];
        let mut current = index;
        while let Some(parent) = program.classes[current].superclass {
            lineage.push(parent);
            current = parent;
        }
        for index in lineage.into_iter().rev() {
            for property in &program.classes[index].stored_properties {
                let selector = selector_id(program, &property.name)
                    .ok_or_else(|| MachineError::UnknownSelector(property.name.clone()))?;
                // An initializer that is not a literal RUNS, with the object
                // bound as its receiver, so it can call the object's own
                // methods and observe side effects in declaration order.
                let value = match property.initializer_function {
                    Some(function) => {
                        let callee = program.functions.get(function).cloned().ok_or(
                            MachineError::Invalid(VerifyError::UnknownFunction { function }),
                        )?;
                        let returned = self.run_body(
                            &callee.instructions,
                            callee.registers,
                            vec![Value::Object(object)],
                            program,
                            classes,
                        )?;
                        returned.into_iter().next().unwrap_or(Value::Nil)
                    }
                    None => literal_runtime_value(&property.initializer)?,
                };
                self.runtime
                    .assign_raw_ivar(object, selector, value)
                    .map_err(MachineError::Construction)?;
            }
            let declared = program.classes[index]
                .stored_properties
                .iter()
                .map(|property| selector_id(program, &property.name))
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| MachineError::UnknownSelector("stored property".to_owned()))?;
            let runtime_properties = self
                .runtime
                .registry()
                .active(classes[index])
                .map_err(MachineError::Class)?
                .properties()
                .to_vec();
            for property in runtime_properties {
                if declared.contains(&property.selector()) {
                    continue;
                }
                let function = usize::try_from(property.initializer().raw()).map_err(|_| {
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
                let returned = self.run_body(
                    &callee.instructions,
                    callee.registers,
                    vec![Value::Object(object)],
                    program,
                    classes,
                )?;
                let value = returned.into_iter().next().unwrap_or(Value::Nil);
                self.runtime
                    .assign_raw_ivar(object, property.selector(), value)
                    .map_err(MachineError::Construction)?;
            }
        }
        Ok(())
    }
}

impl Machine {
    /// Dispatches with the CALLER's lexical authority, per `C077`.
    ///
    /// A private method answers only its declaring class - or a module
    /// composed with `private` access - so the class whose body wrote the send
    /// decides whether the call is authorized. Without a caller the send is
    /// external, which is what refuses a private method from outside.
    pub(super) fn dispatch_from(
        &self,
        object: iris_runtime::ObjectId,
        selector: iris_runtime::Selector,
        caller: Option<usize>,
        caller_module: Option<&str>,
        classes: &[ClassId],
    ) -> Result<iris_runtime::Method, iris_runtime::ConstructionError> {
        let class = self.runtime.class_of(object)?;
        let module = caller_module.and_then(|name| {
            self.modules
                .iter()
                .find_map(|(known, id)| (known == name).then_some(*id))
        });
        let context = match (caller.and_then(|index| classes.get(index).copied()), module) {
            (Some(owner), _) => {
                iris_runtime::DispatchContext::implementation(owner, owner == class)
            }
            // A MODULE composed with `private` access is the authority for the
            // class's private methods, which a class owner cannot express.
            (None, Some(module)) => {
                iris_runtime::DispatchContext::module_implementation(module, true)
            }
            (None, None) => iris_runtime::DispatchContext::external(),
        };
        match self
            .runtime
            .registry()
            .dispatch_with_context(class, selector, context)?
        {
            iris_runtime::DispatchOutcome::Invoke(method) => Ok(method),
            iris_runtime::DispatchOutcome::WouldInvokeMethodMissing { selector } => {
                Err(iris_runtime::DispatchError::MissingMethod { selector }.into())
            }
        }
    }
}

impl Machine {
    /// Runs the ANCESTOR's class method for a `super()` on the singleton side.
    ///
    /// A class method's `super()` has the Class itself as its receiver, so the
    /// ancestor is found by walking the declared superclass chain rather than
    /// through instance dispatch - the two sides hold separate tables.
    pub(super) fn class_super_value(
        &mut self,
        class: ClassId,
        selector: &str,
        arguments: &[Value],
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        let Some(index) = classes.iter().position(|known| *known == class) else {
            return Err(MachineError::Class(ClassError::ClassIdentityExhausted));
        };
        let mut ancestor = program.classes[index].superclass;
        while let Some(current) = ancestor {
            let Some(declaration) = program.classes.get(current) else {
                break;
            };
            if let Some((_, function)) = declaration
                .class_methods
                .iter()
                .find(|(name, _)| name == selector)
            {
                let Some(owner) = classes.get(current).copied() else {
                    break;
                };
                let mut passed = Vec::with_capacity(arguments.len() + 1);
                passed.push(Value::Class(owner));
                passed.extend_from_slice(arguments);
                return self.invoke_function(*function, passed, program, classes);
            }
            ancestor = declaration.superclass;
        }
        Err(MachineError::Kernel(KernelError::Dispatch(
            iris_runtime::DispatchError::NoSuperMethod {
                selector: selector_id(program, selector)
                    .unwrap_or(iris_runtime::Selector::INITIALIZE),
            },
        )))
    }
}

impl Machine {
    /// Closes an iterator, answering what its `close` did.
    ///
    /// The outcome is ANSWERED rather than propagated, because a close that
    /// raises while a body exception is already travelling must not displace
    /// it - `IRIS-V1-CONTROL-C047` makes the body's exception primary and
    /// appends the cleanup failure to its suppressed list.
    pub(super) fn close_iterator(
        &mut self,
        iterator: &Value,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<(), MachineError> {
        if self.iteration_send(iterator, "close", &[])?.is_some() {
            return Ok(());
        }
        let Value::Object(object) = iterator else {
            return Err(MachineError::MessageNotFound {
                receiver_class: super::value_class_name(iterator).to_owned(),
                selector: "close".to_owned(),
            });
        };
        let selector = selector_id(program, "close")
            .ok_or_else(|| MachineError::UnknownSelector("close".to_owned()))?;
        let method = self
            .runtime
            .dispatch_instance(*object, selector)
            .map_err(MachineError::Construction)?;
        let function = usize::try_from(method.body().raw()).map_err(|_| {
            MachineError::Invalid(super::VerifyError::UnknownFunction {
                function: usize::MAX,
            })
        })?;
        let callee = program
            .functions
            .get(function)
            .cloned()
            .ok_or(MachineError::Invalid(super::VerifyError::UnknownFunction {
                function,
            }))?;
        self.run_body(
            &callee.instructions,
            callee.registers,
            vec![Value::Object(*object)],
            program,
            classes,
        )
        .map(|_| ())
    }
}

/// The policy a class's `meta deny` list describes.
///
/// `IRIS-V1-META-C081` fixes the capability vocabulary, so a name outside it
/// is a diagnostic rather than a silently ignored denial - registering every
/// class with the full policy let a denied operation succeed anyway.
fn meta_capabilities(names: &[String]) -> Result<iris_runtime::MetaCapabilities, MachineError> {
    let mut denied = Vec::with_capacity(names.len());
    for name in names {
        denied.push(match name.as_str() {
            "method_set" => iris_runtime::Capability::MethodSet,
            "method_body" => iris_runtime::Capability::MethodBody,
            "property_set" => iris_runtime::Capability::PropertySet,
            "property_body" => iris_runtime::Capability::PropertyBody,
            "modules" => iris_runtime::Capability::Modules,
            "superclass" => iris_runtime::Capability::Superclass,
            "subclass" => iris_runtime::Capability::Subclass,
            "shape" => iris_runtime::Capability::Shape,
            "class_state_set" => iris_runtime::Capability::ClassStateSet,
            "class_state_write" => iris_runtime::Capability::ClassStateWrite,
            "instance_state" => iris_runtime::Capability::InstanceState,
            "native" => iris_runtime::Capability::Native,
            _ => return Err(MachineError::ParseDiagnostic),
        });
    }
    Ok(iris_runtime::MetaCapabilities::denying(&denied))
}

impl Machine {
    /// Runs a class-level initializer the first time its property is READ.
    ///
    /// A body that RAISES is retried on the next read, and one that succeeds
    /// runs exactly once - so the entry is removed only after the value is
    /// stored.
    pub(super) fn force_class_initializer(
        &mut self,
        class: iris_runtime::ClassId,
        selector: iris_runtime::Selector,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<(), MachineError> {
        let Some(function) = self
            .pending_class_initializers
            .get(&(class, selector))
            .copied()
        else {
            return Ok(());
        };
        let callee = program
            .functions
            .get(function)
            .cloned()
            .ok_or(MachineError::Invalid(super::VerifyError::UnknownFunction {
                function,
            }))?;
        // An initializer that does not COMPLETE leaves the property without a
        // value its declared Type admits, so the annotation is what fails -
        // reporting the raised value let the failure escape as though the read
        // itself had raised it.
        let returned = self
            .run_body(
                &callee.instructions,
                callee.registers,
                vec![Value::Class(class)],
                program,
                classes,
            )
            .map_err(|_| MachineError::TypeContractError)?;
        let value = returned.into_iter().next().unwrap_or(Value::Nil);
        self.runtime
            .assign_class_var(class, selector, value)
            .map_err(MachineError::Construction)?;
        self.pending_class_initializers.remove(&(class, selector));
        Ok(())
    }

    pub(super) fn materialize_closed_class(
        &mut self,
        class: iris_runtime::ClassId,
        arguments: &[iris_syntax::TypeExpression],
        program: &Program,
        classes: &[iris_runtime::ClassId],
    ) -> Result<(), MachineError> {
        let Some(rendered) = crate::compile::lowering::Lowering::type_selector(arguments) else {
            return Ok(());
        };
        let suffix = format!("<{rendered}>");
        let Some(declaration) = classes
            .iter()
            .position(|known| *known == class)
            .and_then(|index| program.classes.get(index))
        else {
            return Ok(());
        };
        for variable in &declaration.class_variables {
            if !variable.name.ends_with(&suffix) {
                continue;
            }
            let selector = selector_id(program, &variable.name)
                .ok_or_else(|| MachineError::UnknownSelector(variable.name.clone()))?;
            self.force_class_initializer(class, selector, program, classes)?;
        }
        Ok(())
    }
}
