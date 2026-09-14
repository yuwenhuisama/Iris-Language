use super::{EvaluationError, SourceEvaluator};
use iris_runtime::ClassId;
use iris_syntax::MethodDeclaration;

impl SourceEvaluator {
    pub(in super::super) fn validate_qualified_history(
        &self,
        class: ClassId,
        methods: &[MethodDeclaration],
    ) -> Result<(), EvaluationError> {
        for method in methods {
            let Some(Some(name)) = &method.impl_contract else {
                continue;
            };
            let contract = self
                .contract_names
                .get(name)
                .ok_or(EvaluationError::TypeContractError)?;
            if !self
                .class_contracts
                .get(&class)
                .is_some_and(|contracts| contracts.contains(contract))
                || !self
                    .contract_requirements
                    .get(contract)
                    .is_some_and(|requirements| requirements.contains(&method.selector))
            {
                return Err(EvaluationError::TypeContractError);
            }
            if self
                .qualified_contracts
                .parameters
                .get(contract)
                .is_some_and(|parameters| !parameters.is_empty())
            {
                return Err(EvaluationError::UnsupportedConstruct);
            }
        }
        for ((owner, contract, _), active) in &self.qualified_methods {
            if *owner != class {
                continue;
            }
            let promise = self
                .bodies
                .get(&active.body().raw())
                .ok_or(EvaluationError::UnsupportedConstruct)?;
            let replacement = methods.iter().rev().find(|method| {
                method.selector == promise.selector && method.kind == promise.kind
                    && matches!(&method.impl_contract, Some(Some(name)) if self.contract_names.get(name) == Some(contract))
            }).ok_or(EvaluationError::TypeContractError)?;
            if !iris_syntax::method_signature_compatible(replacement, promise, |source, target| {
                self.nominal_subtype(source, target)
            }) || replacement.visibility != promise.visibility
                || replacement.body.is_none()
            {
                return Err(EvaluationError::TypeContractError);
            }
        }
        Ok(())
    }
}
