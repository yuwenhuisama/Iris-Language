use crate::{NativeError, NativeRegistry, NativeType};
use iris_syntax::{Statement, TypeExpression};

impl NativeRegistry {
    /// Projects validated ABI signatures into source Types without changing native dispatch.
    ///
    /// ABI v1 is synchronous. Its `Integer` is signed i64, not all Iris Integers;
    /// the linked `FunctionMetadata` and `NativeRegistry::call` retain that range
    /// contract. Resources have no registered source nominal Type, so their
    /// projection is `Object`; the registry retains and checks the exact
    /// module-local resource name and ownership instead of inventing a Type.
    pub fn declarations(
        &self,
    ) -> Result<Vec<(String, iris_syntax::ModuleDeclaration)>, NativeError> {
        let mut declarations = Vec::new();
        for metadata in self.metadata() {
            for module in metadata.modules {
                let mut source = format!(
                    "module {} meta deny method_set, method_body, property_set, property_body, modules, superclass, subclass, shape, class_state_set, class_state_write, instance_state, native {{",
                    module.name
                );
                for function in &module.functions {
                    source.push_str(&format!(" public module fun {}(", function.name));
                    source.push_str(
                        &function
                            .parameters
                            .iter()
                            .map(|parameter| parameter.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", "),
                    );
                    source.push_str(") { nil }");
                }
                source.push('}');
                let parsed = iris_parser::parse(&source);
                if !parsed.program_accepted {
                    return Err(NativeError::Metadata);
                }
                for declaration in parsed.program.declarations {
                    let iris_syntax::Declaration::Module(mut declaration) = declaration else {
                        return Err(NativeError::Metadata);
                    };
                    for (statement, function) in declaration.body.iter_mut().zip(&module.functions)
                    {
                        let Statement::Method(method) = statement else {
                            return Err(NativeError::Metadata);
                        };
                        for (parameter, metadata) in
                            method.parameters.iter_mut().zip(&function.parameters)
                        {
                            parameter.annotation = Some(source_type(&metadata.value_type)?);
                        }
                        method.return_type = Some(source_type(&function.returns)?);
                    }
                    declarations.push((metadata.package_id.clone(), declaration));
                }
            }
        }
        Ok(declarations)
    }
}

fn source_type(name: &str) -> Result<TypeExpression, NativeError> {
    let name = match NativeType::parse(name)? {
        NativeType::Nil => "Nil",
        NativeType::Bool => "Bool",
        NativeType::Integer => "Integer",
        NativeType::String => "String",
        NativeType::Bytes => "Bytes",
        NativeType::Resource(_) => "Object",
    };
    Ok(TypeExpression::Name(name.to_owned()))
}
