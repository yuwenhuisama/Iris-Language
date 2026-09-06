use super::{CompileError, Function, Instruction, Register};
use std::collections::BTreeSet;

pub(super) fn prepare(
    program: &mut iris_syntax::Program,
    registry: &iris_native_host::NativeRegistry,
) -> Result<BTreeSet<String>, CompileError> {
    let mut names = BTreeSet::new();
    for (_, module) in registry
        .declarations()
        .map_err(|_| CompileError::new("native metadata"))?
    {
        if !program.declarations.iter().any(|declaration| {
            matches!(declaration,
            iris_syntax::Declaration::Import(import) if import.target == module.name)
        }) {
            continue;
        }
        if program.declarations.iter().any(|declaration| {
            matches!(declaration,
            iris_syntax::Declaration::Module(existing) if existing.name == module.name)
        }) {
            return Err(CompileError::new("native module declaration conflict"));
        }
        for statement in &module.body {
            if let iris_syntax::Statement::Method(method) = statement {
                names.insert(format!("{}.{}", module.name, method.selector));
            }
        }
        program
            .declarations
            .push(iris_syntax::Declaration::Module(module));
    }
    Ok(names)
}
pub(super) fn bind(function: &mut Function, names: &BTreeSet<String>) -> Result<(), CompileError> {
    if names.contains(&function.name) {
        let destination = Register::try_from(function.parameters)
            .map_err(|_| CompileError::new("native parameter limit"))?;
        let count = u16::try_from(function.parameters)
            .map_err(|_| CompileError::new("native parameter limit"))?;
        function.registers = function.parameters + 1;
        function.instructions = vec![
            Instruction::NativeCall {
                destination,
                name: function.name.clone(),
                first: 0,
                count,
            },
            Instruction::Return { value: destination },
        ];
    }
    Ok(())
}
