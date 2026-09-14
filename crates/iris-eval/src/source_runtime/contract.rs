use super::decorator_phases::PhaseTarget;
use super::{Binding, EvaluationError, SourceEvaluator, meta_capabilities};
use iris_runtime::decorator_protocol::{DecoratorKind, DecoratorReason};
use iris_runtime::{ContractId, Value};
use iris_syntax::{ContractDeclaration, Statement, TypeExpression};

impl SourceEvaluator {
    pub(super) fn contract(
        &mut self,
        declaration: &ContractDeclaration,
    ) -> Result<(), EvaluationError> {
        let effective = self.effective_contract(declaration)?;
        let contract = ContractId::new(self.next_contract);
        self.next_contract += 1;
        let checkpoint = self.decorator_metadata.len();
        let outcome = (|| {
            let metadata = self.contract_decorator_metadata(contract, declaration)?;
            if !self.decorator_planning {
                self.contract_decorator_phases(declaration, &metadata)?;
            }
            Ok(metadata)
        })();
        let metadata = match outcome {
            Ok(metadata) => metadata,
            Err(error) => {
                self.decorator_metadata.truncate(checkpoint);
                return Err(error);
            }
        };
        let mut requirements = Vec::new();
        for method in &effective.requirements {
            requirements.push(method.selector.clone());
            self.contract_requirement_arities
                .insert((contract, method.selector.clone()), method.parameters.len());
            self.contract_requirement_parameters.insert(
                (contract, method.selector.clone()),
                method
                    .parameters
                    .iter()
                    .map(|parameter| parameter.annotation.clone())
                    .collect(),
            );
            if let Some(returns) = &method.return_type {
                self.contract_requirement_returns
                    .insert((contract, method.selector.clone()), returns.clone());
            }
        }
        self.contract_capabilities
            .insert(contract, effective.capabilities);
        self.contract_parents.insert(
            contract,
            effective
                .parents
                .iter()
                .map(|(parent, _)| *parent)
                .collect(),
        );
        self.qualified_contracts
            .parents
            .insert(contract, effective.parents);
        self.qualified_contracts
            .requirements
            .insert(contract, effective.requirements);
        self.contract_requirements.insert(contract, requirements);
        self.contract_metadata.insert(contract, metadata);
        self.package_contexts
            .contracts
            .insert(contract, (self.package.clone(), self.api_major));
        self.contract_names
            .insert(declaration.name.clone(), contract);
        self.qualified_contracts
            .parameters
            .insert(contract, declaration.parameters.clone());
        self.names.insert(
            declaration.name.clone(),
            Binding::immutable(Value::Contract(contract, Vec::new())),
        );
        Ok(())
    }

    pub(super) fn contract_decorator_phases(
        &mut self,
        declaration: &ContractDeclaration,
        metadata: &Value,
    ) -> Result<(), EvaluationError> {
        for decorator in &declaration.decorators {
            self.execute_decorator_phase(
                decorator,
                PhaseTarget {
                    kind: DecoratorKind::Contract,
                    reason: DecoratorReason::Origin,
                    metadata: metadata.clone(),
                    candidate: None,
                },
            )?;
        }
        Ok(())
    }

    fn contract_decorator_metadata(
        &mut self,
        contract: ContractId,
        declaration: &ContractDeclaration,
    ) -> Result<Value, EvaluationError> {
        let parents = declaration
            .parents
            .iter()
            .map(|parent| {
                let (TypeExpression::Name(name) | TypeExpression::Generic { name, .. }) = parent
                else {
                    return Err(EvaluationError::UnsupportedConstruct);
                };
                self.contract_names
                    .get(name)
                    .map(|parent| Value::Contract(*parent, Vec::new()))
                    .ok_or(EvaluationError::NameError)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let requirements = declaration
            .body
            .iter()
            .filter_map(|statement| match statement {
                Statement::Method(method) if method.body.is_none() => {
                    Some(Value::Symbol(method.selector.clone()))
                }
                _ => None,
            })
            .collect();
        let decorators = declaration
            .decorators
            .iter()
            .map(|decorator| Value::Symbol(decorator.name.clone()))
            .collect();
        let capabilities = meta_capabilities(&declaration.meta_deny)?
            .denied()
            .into_iter()
            .map(|capability| Value::Symbol(super::capability_name(capability).into()))
            .collect();
        self.metadata_record(vec![
            ("name", Value::Symbol(declaration.name.clone())),
            ("kind", Value::Symbol("contract".into())),
            ("package", Value::Text(self.package.clone())),
            ("type", Value::Contract(contract, Vec::new())),
            ("parents", Value::ReadonlyArray(parents)),
            ("requirements", Value::ReadonlyArray(requirements)),
            ("decorators", Value::ReadonlyArray(decorators)),
            ("meta_capabilities", Value::ReadonlyArray(capabilities)),
        ])
    }
}
