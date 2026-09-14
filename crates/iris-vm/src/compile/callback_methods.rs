use super::declarations::Signature;
use super::decorators::Application;
use super::lowering::{Lowering, lower_function};
use super::{CompileError, Function, Instruction, Register};
use iris_runtime::decorator_protocol::DecoratorKind;
use iris_syntax::{Expression, MethodDeclaration, MethodKind};

impl Lowering<'_, '_> {
    pub(super) fn callback_method(
        &mut self,
        method: &MethodDeclaration,
    ) -> Result<Register, CompileError> {
        let class = self
            .open_class
            .ok_or_else(|| CompileError::new("callback declaration owner"))?;
        if method.is_async || !matches!(method.kind, MethodKind::Instance | MethodKind::Property) {
            return Err(CompileError::new("callback method mode"));
        }
        let signature = Signature {
            declaration: Some(method),
            module: &self.classes[class].name,
            selector: &method.selector,
            parameters: method.parameters.iter().collect(),
            return_type: method.return_type.as_ref(),
            body: method
                .body
                .as_deref()
                .ok_or_else(|| CompileError::new("callback abstract method"))?,
            receiver: true,
            class_method: false,
            private: method.visibility == iris_syntax::Visibility::Private,
            is_async: false,
            constants: Vec::new(),
            expression_body: None,
        };
        let mut nested = Vec::new();
        let body = lower_function(
            &signature,
            self.declarations(),
            self.declared_functions + self.closures.len(),
            &mut nested,
            self.program_bindings,
        )?;
        self.closures.extend(nested);
        let function = self.declared_functions + self.closures.len();
        self.closures.push(body);
        let kind = if method.kind == MethodKind::Property {
            DecoratorKind::Property
        } else {
            DecoratorKind::Method
        };
        let mut applications = Vec::new();
        for decorator in &method.decorators {
            let decorator_class = self
                .classes
                .iter()
                .position(|class| class.name == decorator.name)
                .ok_or_else(|| CompileError::diagnostic("IRIS-DECORATOR-KIND"))?;
            let contract_name = if kind == DecoratorKind::Property {
                "PropertyDecorator"
            } else {
                "MethodDecorator"
            };
            if !self.classes[decorator_class]
                .contracts
                .iter()
                .any(|index| self.contracts[*index].name == contract_name)
            {
                return Err(CompileError::diagnostic("IRIS-DECORATOR-KIND"));
            }
            let mut nested = Vec::new();
            let mut lowering = Lowering::new(
                self.declarations(),
                self.declared_functions + self.closures.len(),
                &mut nested,
                self.program_bindings,
                false,
            );
            let value = lowering.expression(&Expression::Array(decorator.arguments.clone()))?;
            lowering.instructions.push(Instruction::Return { value });
            let argument_function = Function {
                body_entry: 0,
                signature: None,
                name: "<decorator-arguments>".into(),
                parameters: 0,
                captures: 0,
                fixed_arity: Some(0),
                parameter_types: Vec::new(),
                return_type: "Array".into(),
                is_async: false,
                registers: usize::from(lowering.next_register),
                instructions: std::mem::take(&mut lowering.instructions),
            };
            drop(lowering);
            self.closures.extend(nested);
            let arguments = self.declared_functions + self.closures.len();
            self.closures.push(argument_function);
            applications.push(Application {
                property: None,
                target: super::decorators::Target::Class(class),
                method: Some(function),
                decorator: decorator_class,
                arguments,
                kind,
            });
        }
        self.instructions.push(Instruction::DeclareMethod {
            class,
            function,
            applications,
        });
        let destination = self.allocate()?;
        self.instructions.push(Instruction::LoadNil { destination });
        Ok(destination)
    }
}
