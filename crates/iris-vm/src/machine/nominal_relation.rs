use crate::compile::Program;

pub(super) fn nominal_subtype(program: &Program, source: &str, target: &str) -> bool {
    if program.contracts.iter().any(|contract| {
        contract.name == source
            && (contract.name == target || contract.parents.iter().any(|parent| parent == target))
    }) {
        return true;
    }
    let mut current = program
        .classes
        .iter()
        .position(|class| class.name == source);
    while let Some(index) = current {
        let class = &program.classes[index];
        if class.name == target
            || class.contracts.iter().any(|index| {
                let contract = &program.contracts[*index];
                contract.name == target || contract.parents.iter().any(|parent| parent == target)
            })
        {
            return true;
        }
        current = class.superclass;
    }
    false
}
