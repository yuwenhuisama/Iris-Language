use crate::{NativeError, NativeRegistry};

impl NativeRegistry {
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
                    let iris_syntax::Declaration::Module(module) = declaration else {
                        return Err(NativeError::Metadata);
                    };
                    declarations.push((metadata.package_id.clone(), module));
                }
            }
        }
        Ok(declarations)
    }
}
