use super::{Machine, MachineError};
use crate::compile::Program;
use iris_runtime::decorator_protocol::{
    Invocation, InvocationParameter, InvocationPayload, InvocationSignature, InvocationSlot,
    ParameterCategory, SelectedCall, SlotKind, SourceCall,
};
use iris_runtime::{ClassId, Method, Value};
use iris_syntax::TypeExpression;

impl Machine {
    pub(super) fn invoke_wrapped(
        &mut self,
        method: Method,
        arguments: Vec<Value>,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        let bindings = match self.closed_method_bindings(method) {
            Some(bindings) => bindings,
            None => {
                let owner = self
                    .resolve_method_body(method.body(), program)
                    .map_err(|_| MachineError::UnsupportedConstruct)?;
                if !self
                    .direct_owner_bindings(
                        method,
                        arguments.first(),
                        (&owner.program, &owner.classes),
                    )?
                    .is_empty()
                {
                    return self.invoke_selected_method(method, arguments, program, classes);
                }
                Vec::new()
            }
        };
        let previous = std::mem::replace(&mut self.method_types, bindings);
        self.active_values.push(Value::Method(method));
        let result = self.invoke_wrapped_bound(method, arguments, program, classes);
        self.active_values.pop();
        self.method_types = previous;
        result
    }

    fn invoke_wrapped_bound(
        &mut self,
        method: Method,
        arguments: Vec<Value>,
        _program: &Program,
        _classes: &[ClassId],
    ) -> Result<Value, MachineError> {
        if let Some(Value::Class(class)) = arguments.first() {
            self.runtime
                .registry()
                .validate_method_binding(*class, method)
                .map_err(iris_runtime::ConstructionError::from)
                .map_err(MachineError::Construction)?;
        }
        let chain = self
            .wrapper_chains
            .get(&method.id())
            .cloned()
            .ok_or(MachineError::UnsupportedConstruct)?;
        let owner = std::rc::Rc::clone(&chain.program);
        let program = owner.as_ref();
        let owned_classes = program
            .link
            .as_ref()
            .ok_or(MachineError::UnsupportedConstruct)?
            .classes
            .borrow()
            .clone();
        let classes = owned_classes.as_slice();
        if let Some(callback) = chain.body_closure {
            return self.invoke_closure_value(callback, &arguments[1..], program, classes);
        }
        if program.functions[chain.function].is_async {
            let signature = program.functions[chain.function]
                .signature
                .as_ref()
                .ok_or(MachineError::UnsupportedConstruct)?;
            let result = self.reify_type(
                &signature
                    .return_type
                    .clone()
                    .unwrap_or(TypeExpression::Name("Object".into())),
                program,
                classes,
            )?;
            let outer = self.allocate_task(result);
            let prepared = self.prepare_wrapped(method, arguments, program, classes);
            match prepared {
                Ok(invocation) => {
                    self.start_async_layer(outer, chain, 0, invocation, program, classes)?
                }
                Err(error) => self.complete_task(outer, Err(error)),
            }
            return Ok(Value::Task(outer));
        }
        let invocation = self.prepare_wrapped(method, arguments, program, classes)?;
        self.invoke_layer(chain, 0, invocation, program, classes)
    }

    fn prepare_wrapped(
        &mut self,
        method: Method,
        arguments: Vec<Value>,
        _program: &Program,
        _classes: &[ClassId],
    ) -> Result<Invocation, MachineError> {
        let chain = self
            .wrapper_chains
            .get(&method.id())
            .ok_or(MachineError::UnsupportedConstruct)?;
        let owner = std::rc::Rc::clone(&chain.program);
        let program = owner.as_ref();
        let owned_classes = program
            .link
            .as_ref()
            .ok_or(MachineError::UnsupportedConstruct)?
            .classes
            .borrow()
            .clone();
        let classes = owned_classes.as_slice();
        let function = &program.functions[chain.function];
        let signature = function
            .signature
            .as_ref()
            .ok_or(MachineError::UnsupportedConstruct)?;
        if function
            .fixed_arity
            .is_some_and(|arity| arguments.len() != arity + 1)
        {
            return Err(MachineError::ArgumentError);
        }
        let mut parameters = Vec::new();
        for parameter in &signature.parameters {
            let category = match parameter.category {
                iris_syntax::ParameterCategory::Positional => ParameterCategory::Positional,
                iris_syntax::ParameterCategory::Keyword => ParameterCategory::Keyword,
                iris_syntax::ParameterCategory::Rest => ParameterCategory::Rest,
                iris_syntax::ParameterCategory::KeywordRest => ParameterCategory::KeywordRest,
                iris_syntax::ParameterCategory::Block => ParameterCategory::Block,
            };
            let annotation = parameter
                .annotation
                .clone()
                .unwrap_or(TypeExpression::Name("Object".into()));
            parameters.push(
                InvocationParameter::new(
                    &parameter.name,
                    category,
                    self.reify_type(&annotation, program, classes)?,
                )
                .with_optional(parameter.default.is_some()),
            );
        }
        let result_type = self.reify_type(
            &signature
                .return_type
                .clone()
                .unwrap_or(TypeExpression::Name("Object".into())),
            program,
            classes,
        )?;
        let closed = InvocationSignature::new(parameters, result_type, function.is_async)
            .map_err(|_| MachineError::ArgumentError)?;
        let super::prepared_call::CallArguments {
            positional,
            keywords,
            block,
        } = super::prepared_call::CallArguments::scan(&arguments[1..])?;
        let mut position = 0;
        let omitted: Vec<_> = signature
            .parameters
            .iter()
            .filter_map(|parameter| {
                let present = match parameter.category {
                    iris_syntax::ParameterCategory::Positional => {
                        position += 1;
                        position <= positional.len()
                    }
                    iris_syntax::ParameterCategory::Keyword => {
                        keywords.iter().any(|(name, _)| name == &parameter.name)
                    }
                    iris_syntax::ParameterCategory::Block => block.is_some(),
                    _ => true,
                };
                (!present).then(|| parameter.name.clone())
            })
            .collect();
        let defaulted = omitted
            .iter()
            .filter(|name| {
                signature
                    .parameters
                    .iter()
                    .any(|parameter| &parameter.name == *name && parameter.default.is_some())
            })
            .cloned()
            .collect();
        let source =
            SourceCall::new(positional, keywords, block).with_provenance(omitted, defaulted);
        let bound = self
            .run_body(
                &function.instructions[..function.body_entry],
                function.registers,
                arguments.clone(),
                program,
                classes,
            )
            .map_err(|error| match error {
                MachineError::TypeContractError => self.decorator_type_error(),
                error => error,
            })?;
        let payload = InvocationPayload::from_bindings(
            &closed,
            &bound[1..function.parameters],
            |target, value| self.decorator_accepts(target, value),
        )
        .map_err(|error| self.payload_error(error))?;
        let owner_bindings =
            self.direct_owner_bindings(method, arguments.first(), (program, classes))?;
        let owner = match method.owner() {
            iris_runtime::MethodOwner::Class(class) if !owner_bindings.is_empty() => {
                let Some(Value::Object(object)) = arguments.first() else {
                    return Err(MachineError::UnsupportedConstruct);
                };
                Value::Type(
                    class,
                    self.runtime
                        .type_arguments_of(*object)
                        .map_err(MachineError::Construction)?
                        .to_vec(),
                )
            }
            iris_runtime::MethodOwner::Class(class)
                if signature.kind == iris_syntax::MethodKind::Class =>
            {
                Value::Class(class)
            }
            iris_runtime::MethodOwner::Class(class) => Value::Type(class, Vec::new()),
            iris_runtime::MethodOwner::Module(module) => Value::Symbol(
                self.modules
                    .iter()
                    .find_map(|(name, identity)| (*identity == module).then(|| name.clone()))
                    .ok_or(MachineError::NameError)?,
            ),
        };
        let (selector, kind) = if signature.kind == iris_syntax::MethodKind::Property {
            match signature.selector.strip_suffix('=') {
                Some(logical) => (logical, SlotKind::Setter),
                None => (signature.selector.as_str(), SlotKind::Getter),
            }
        } else {
            (signature.selector.as_str(), SlotKind::Method)
        };
        let slot = InvocationSlot::new(owner, selector, kind);
        let slot = match self.selected_qualifier(method, (program, classes))? {
            Some(qualifier) => slot.qualified(qualifier),
            None => slot,
        };
        let method_types = signature
            .type_parameters
            .iter()
            .map(|name| {
                self.method_types
                    .iter()
                    .find(|(parameter, _)| parameter == name)
                    .map(|(_, value)| value.clone())
                    .ok_or(MachineError::NameError)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let selected = SelectedCall::new(arguments[0].clone(), slot, closed).with_type_arguments(
            owner_bindings.into_iter().map(|(_, value)| value).collect(),
            method_types,
        );
        Ok(Invocation::new(selected, source, payload))
    }
}
