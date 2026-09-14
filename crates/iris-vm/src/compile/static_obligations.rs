use super::{
    Class, CompileError, Contract, declarations::Signature, effective_contracts::closed_requirement,
};

pub(super) fn validate_origins(
    classes: &[Class],
    contracts: &[Contract],
    signatures: &[Signature<'_>],
) -> Result<(), CompileError> {
    for index in 0..classes.len() {
        let mut owner = Some(index);
        while let Some(ancestor) = owner {
            for (contract, arguments) in &classes[ancestor].contract_arguments {
                for promise in &contracts[*contract].signatures {
                    let mut current = Some(index);
                    let mut selected = None;
                    while let Some(index) = current {
                        selected = classes[index]
                            .methods
                            .iter()
                            .rev()
                            .find(|(name, _)| *name == promise.selector)
                            .map(|(_, function)| *function)
                            .or_else(|| {
                                classes[index].mixins.iter().rev().find_map(|(name, _)| {
                                    signatures.iter().position(|signature| {
                                        signature.module == name
                                            && signature.selector == promise.selector
                                    })
                                })
                            });
                        if selected.is_some() {
                            break;
                        }
                        current = classes[index].superclass;
                    }
                    let implementation = selected
                        .and_then(|function| signatures[function].declaration)
                        .ok_or_else(|| CompileError::new("static impl"))?;
                    let promise = closed_requirement(&contracts[*contract], arguments, promise);
                    if !iris_syntax::method_signature_compatible(
                        implementation,
                        &promise,
                        |source, target| {
                            let mut current = classes.iter().position(|class| class.name == source);
                            while let Some(index) = current {
                                if classes[index].name == target {
                                    return true;
                                }
                                current = classes[index].superclass;
                            }
                            false
                        },
                    ) {
                        return Err(CompileError::new("static impl"));
                    }
                }
            }
            owner = classes[ancestor].superclass;
        }
    }
    Ok(())
}
