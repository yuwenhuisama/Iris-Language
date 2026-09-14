use super::{EvaluationError, SourceEvaluator, wrapper_generics};
use iris_runtime::{ClassId, ContractId, NominalType, Value};
use iris_syntax::{ClassDeclaration, MethodDeclaration, TypeExpression};
use std::collections::HashMap;

#[derive(Clone, Default)]
pub(super) struct QualifiedContracts {
    pub parameters: HashMap<ContractId, Vec<String>>,
    pub owners: HashMap<(ClassId, ContractId), Vec<TypeExpression>>,
    pub parents: HashMap<ContractId, Vec<(ContractId, Vec<TypeExpression>)>>,
    pub requirements: HashMap<ContractId, Vec<MethodDeclaration>>,
    views: Vec<(ContractId, ContractId, Vec<NominalType>)>,
}

impl QualifiedContracts {
    pub fn definition(&self, identity: ContractId) -> ContractId {
        self.views
            .iter()
            .find_map(|(view, definition, _)| (*view == identity).then_some(*definition))
            .unwrap_or(identity)
    }
}

impl SourceEvaluator {
    pub(super) fn check_closed_contract_bound(
        &mut self,
        argument: &NominalType,
        bound: &TypeExpression,
    ) -> Result<(), EvaluationError> {
        let TypeExpression::Generic { name, arguments } = bound else {
            return Ok(());
        };
        let Some(contract) = self.contract_names.get(name).copied() else {
            return Ok(());
        };
        let required = Value::Contract(contract, self.nominal_arguments(arguments)?);
        let declared =
            self.closed_qualified_contract((argument.class(), contract), argument.arguments())?;
        if declared != required {
            return Err(EvaluationError::TypeContractError);
        }
        Ok(())
    }

    pub(super) fn record_qualified_contracts(
        &mut self,
        class: ClassId,
        declaration: &ClassDeclaration,
    ) -> Result<(), EvaluationError> {
        if let Some(TypeExpression::Name(name)) = &declaration.extends {
            let superclass = self.class_name(name)?.ok_or(EvaluationError::NameError)?;
            let inherited = self
                .qualified_contracts
                .owners
                .iter()
                .filter(|((owner, _), _)| *owner == superclass)
                .map(|((_, contract), arguments)| (*contract, arguments.clone()))
                .collect::<Vec<_>>();
            for (contract, arguments) in inherited {
                self.qualified_contracts
                    .owners
                    .insert((class, contract), arguments);
            }
        }
        for target in &declaration.implements {
            let (name, arguments) = match target {
                TypeExpression::Name(name) => (name, Vec::new()),
                TypeExpression::Generic { name, arguments } => (name, arguments.clone()),
                _ => return Err(EvaluationError::UnsupportedConstruct),
            };
            let contract = self.contract_names[name];
            if self
                .qualified_contracts
                .parameters
                .get(&contract)
                .is_some_and(|parameters| parameters.len() != arguments.len())
            {
                return Err(EvaluationError::TypeContractError);
            }
            self.record_contract_arguments(class, (contract, arguments))?;
        }
        Ok(())
    }

    fn record_contract_arguments(
        &mut self,
        class: ClassId,
        (contract, arguments): (ContractId, Vec<TypeExpression>),
    ) -> Result<(), EvaluationError> {
        if let Some(known) = self.qualified_contracts.owners.get(&(class, contract)) {
            return if *known == arguments {
                Ok(())
            } else {
                Err(EvaluationError::TypeContractError)
            };
        }
        let bindings = self
            .qualified_contracts
            .parameters
            .get(&contract)
            .into_iter()
            .flatten()
            .cloned()
            .zip(arguments.iter().cloned())
            .collect();
        self.qualified_contracts
            .owners
            .insert((class, contract), arguments);
        let parents = self
            .qualified_contracts
            .parents
            .get(&contract)
            .cloned()
            .unwrap_or_default();
        for (parent, arguments) in parents {
            let arguments = arguments
                .iter()
                .map(|argument| wrapper_generics::substitute(argument, &bindings))
                .collect();
            self.record_contract_arguments(class, (parent, arguments))?;
        }
        Ok(())
    }

    pub(super) fn closed_contract_admits(
        &mut self,
        value: &Value,
        (contract, arguments): (ContractId, &[NominalType]),
    ) -> Result<bool, EvaluationError> {
        let receiver = match value {
            Value::ContractView(receiver, _) => receiver.as_ref(),
            value => value,
        };
        if !self.value_conforms_to_contract(receiver, contract)? {
            return Ok(false);
        }
        if arguments.is_empty() {
            return Ok(true);
        }
        let class = self.class_of_value(receiver)?;
        let owner_arguments = match receiver {
            Value::Object(object) => self
                .runtime
                .type_arguments_of(*object)
                .map_err(EvaluationError::Construction)?
                .to_vec(),
            _ => Vec::new(),
        };
        Ok(
            self.closed_qualified_contract((class, contract), &owner_arguments)?
                == Value::Contract(contract, arguments.to_vec()),
        )
    }

    pub(super) fn qualified_requirement_bindings(
        &self,
        class: ClassId,
        contract: ContractId,
    ) -> HashMap<String, TypeExpression> {
        self.qualified_contracts
            .parameters
            .get(&contract)
            .into_iter()
            .flatten()
            .zip(
                self.qualified_contracts
                    .owners
                    .get(&(class, contract))
                    .into_iter()
                    .flatten(),
            )
            .map(|(name, argument)| (name.clone(), argument.clone()))
            .collect()
    }

    pub(super) fn closed_qualified_contract(
        &mut self,
        (class, contract): (ClassId, ContractId),
        owner_arguments: &[NominalType],
    ) -> Result<Value, EvaluationError> {
        let arguments = self
            .qualified_contracts
            .owners
            .get(&(class, contract))
            .cloned()
            .unwrap_or_default();
        let parameters = self
            .wrapper_owner_parameters
            .get(&class)
            .cloned()
            .unwrap_or_default();
        let bindings = parameters
            .iter()
            .zip(owner_arguments)
            .map(|(name, argument)| Ok((name.clone(), self.wrapper_nominal_annotation(argument)?)))
            .collect::<Result<HashMap<_, _>, EvaluationError>>()?;
        let arguments = arguments
            .iter()
            .map(|argument| wrapper_generics::substitute(argument, &bindings))
            .collect::<Vec<_>>();
        Ok(Value::Contract(
            contract,
            self.nominal_arguments(&arguments)?,
        ))
    }

    pub(super) fn contract_view(
        &mut self,
        value: Value,
        target: &Value,
    ) -> Result<Value, EvaluationError> {
        let Value::Contract(contract, arguments) = target else {
            return Err(EvaluationError::UnsupportedConstruct);
        };
        if !self.closed_contract_admits(&value, (*contract, arguments))? {
            return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
        }
        let value = match value {
            Value::ContractView(receiver, _) => *receiver,
            value => value,
        };
        let generic = self
            .qualified_contracts
            .parameters
            .get(contract)
            .is_some_and(|parameters| !parameters.is_empty());
        if arguments.is_empty() && !generic {
            return Ok(Value::ContractView(Box::new(value), *contract));
        }
        if arguments.is_empty() {
            return Err(EvaluationError::Runtime(iris_runtime::KernelError::Type));
        }
        let identity = match self
            .qualified_contracts
            .views
            .iter()
            .find(|(_, definition, known)| definition == contract && known == arguments)
        {
            Some((identity, _, _)) => *identity,
            None => {
                let identity = ContractId::new(self.next_contract);
                self.next_contract += 1;
                self.qualified_contracts
                    .views
                    .push((identity, *contract, arguments.clone()));
                identity
            }
        };
        Ok(Value::ContractView(Box::new(value), identity))
    }
}
