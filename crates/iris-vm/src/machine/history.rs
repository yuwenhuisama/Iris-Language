use super::{Machine, MachineError, selector_id};
use crate::compile::{ClassReopen, Program};
use iris_runtime::{ClassId, Value};

#[path = "module_history.rs"]
mod module;

impl Machine {
    pub(crate) fn replace_package_history(&mut self, history: crate::PackageHistory) {
        self.history = history;
    }

    pub(crate) fn replace_package_upgrades(&mut self, upgrades: crate::PackageUpgrade) {
        self.upgrades = upgrades;
    }

    pub(super) fn upgrade_package(
        &mut self,
        current: &Program,
        target_version: &str,
    ) -> Result<Value, MachineError> {
        if self.open_depth > 0 || self.decorator_planning || !self.decorator_phases.is_empty() {
            return Err(MachineError::UnsupportedConstruct);
        }
        let mut matches = self.upgrades.versions.iter().filter(|record| {
            record.package_id == current.package_id()
                && record.api_major == current.api_major()
                && record.version == target_version
        });
        let record = matches
            .next()
            .ok_or(MachineError::RevisionArtifactUnavailable)?
            .clone();
        if matches.next().is_some()
            || iris_runtime::artifact_digest(record.artifact.2.as_bytes())
                != record
                    .artifact
                    .1
                    .strip_prefix("b3:")
                    .unwrap_or(&record.artifact.1)
        {
            return Err(MachineError::RevisionArtifactUnavailable);
        }
        let candidate = crate::compile::history::compile_upgrade(&record.artifact.2, current)?;
        for (index, declaration) in current.classes.iter().enumerate() {
            let target = candidate
                .classes
                .iter()
                .position(|class| class.name == declaration.name)
                .ok_or(MachineError::RevisionArtifactUnavailable)?;
            let class = current
                .link
                .as_ref()
                .and_then(|link| link.classes.borrow().get(index).copied())
                .ok_or(MachineError::RevisionArtifactUnavailable)?;
            self.validate_history_shape((class, index, target), (current, &candidate))?;
        }
        for (index, declaration) in current.modules.iter().enumerate() {
            let target = candidate
                .modules
                .iter()
                .position(|module| module.name == declaration.name)
                .ok_or(MachineError::RevisionArtifactUnavailable)?;
            let module = current
                .link
                .as_ref()
                .and_then(|link| link.modules.borrow().get(index).map(|(_, module)| *module))
                .ok_or(MachineError::RevisionArtifactUnavailable)?;
            self.validate_module_history((module, target), (current, &candidate))?;
        }

        let checkpoint = self.code.checkpoint();
        let chains = self.wrapper_chains.clone();
        let signatures = self.method_signatures.clone();
        let metadata = self.decorator_metadata.len();
        let selectors = self.dynamic_selectors.clone();
        let next_selector = self.next_dynamic_selector;
        let pending = self.pending_replacements.clone();
        let closures = self.closures.clone();
        let active_values = self.active_values.clone();
        let qualified = self.qualified_methods.clone();
        let static_impls = self.static_impl_slots.clone();
        let dynamic_methods = self.dynamic_methods.clone();
        let modules = self.modules.clone();
        let outcome = (|| {
            let candidate = self.link_history(candidate, current)?;
            let classes = candidate
                .link
                .as_ref()
                .ok_or(MachineError::UnsupportedConstruct)?
                .classes
                .borrow()
                .clone();
            for declaration in &current.classes {
                let target = candidate
                    .classes
                    .iter()
                    .position(|class| class.name == declaration.name)
                    .ok_or(MachineError::RevisionArtifactUnavailable)?;
                let class = classes[target];
                let applications = candidate
                    .decorator_applications
                    .iter()
                    .filter(|application| {
                        application.target == crate::compile::decorators::Target::Class(target)
                    })
                    .collect::<Vec<_>>();
                self.runtime
                    .registry_mut()
                    .begin_transaction(class)
                    .map_err(MachineError::Class)?;
                let members = ClassReopen {
                    mixins: Vec::new(),
                    methods: candidate.classes[target].methods.clone(),
                    class_methods: candidate.classes[target].class_methods.clone(),
                    qualified_impls: Vec::new(),
                };
                self.open_depth += 1;
                let installed = self
                    .stage_members((class, &members), &candidate)
                    .and_then(|()| {
                        self.install_decorator_applications_with_reason(
                            class,
                            &applications,
                            &candidate,
                            &classes,
                            Some(iris_runtime::decorator_protocol::DecoratorReason::Upgrade),
                        )
                    });
                self.open_depth -= 1;
                installed?;
            }
            for declaration in &current.modules {
                let target = candidate
                    .modules
                    .iter()
                    .position(|module| module.name == declaration.name)
                    .ok_or(MachineError::RevisionArtifactUnavailable)?;
                let module = self
                    .modules
                    .iter()
                    .find_map(|(name, module)| (name == &declaration.name).then_some(*module))
                    .ok_or(MachineError::RevisionArtifactUnavailable)?;
                self.stage_module_replay((module, target), &candidate)?;
                self.install_module_upgrade_decorators(module, target, &candidate, &classes)?;
            }
            self.validate_replacements(&candidate)?;
            self.pending_replacements.clear();
            self.runtime
                .commit_structural_group()
                .map_err(|error| self.structural_error(error))?;
            Ok(())
        })();
        match outcome {
            Ok(()) => {
                self.package_versions.insert(
                    (current.package_id().to_owned(), current.api_major()),
                    target_version.to_owned(),
                );
                Ok(Value::Array(iris_runtime::ArrayRef::new(Vec::new())))
            }
            Err(error) => {
                self.runtime.roll_back_group();
                self.code.rollback(checkpoint);
                self.wrapper_chains = chains;
                self.method_signatures = signatures;
                self.decorator_metadata.truncate(metadata);
                self.dynamic_selectors = selectors;
                self.next_dynamic_selector = next_selector;
                self.pending_replacements = pending;
                self.closures = closures;
                self.active_values = active_values;
                self.qualified_methods = qualified;
                self.static_impl_slots = static_impls;
                self.dynamic_methods = dynamic_methods;
                self.modules = modules;
                Err(error)
            }
        }
    }

    pub(super) fn rollback_class(
        &mut self,
        class: ClassId,
        arguments: &[Value],
    ) -> Result<Value, MachineError> {
        let (current, index) = self
            .code
            .class_owner(class)
            .ok_or(MachineError::RevisionArtifactUnavailable)?;
        let name = &current.classes[index].name;
        let (actual, source) = self.history_source((&current, name), arguments)?;
        if self.open_depth > 0 || self.runtime.registry().is_staging(class) {
            return Err(MachineError::UnsupportedConstruct);
        }
        let historical = crate::compile::history::compile(source, name)?;
        let target = historical
            .classes
            .iter()
            .position(|declaration| declaration.name == *name)
            .ok_or(MachineError::RevisionArtifactUnavailable)?;
        self.validate_history_shape((class, index, target), (&current, &historical))?;
        let historical = self.link_history(historical, &current)?;
        let classes = historical
            .link
            .as_ref()
            .ok_or(MachineError::UnsupportedConstruct)?
            .classes
            .borrow()
            .clone();
        let applications = historical
            .decorator_applications
            .iter()
            .filter(|application| {
                application.target == crate::compile::decorators::Target::Class(target)
            })
            .collect::<Vec<_>>();
        let metadata = self.decorator_metadata.len();
        let planning = self.decorator_planning;
        self.decorator_planning = true;
        let planned = applications.iter().try_for_each(|application| {
            self.execute_decorator_phase(application, &historical, &classes)
                .map(|_| ())
        });
        self.decorator_planning = planning;
        self.decorator_metadata.truncate(metadata);
        planned?;
        let chains = self.wrapper_chains.clone();
        let signatures = self.method_signatures.clone();
        let members = ClassReopen {
            mixins: Vec::new(),
            methods: historical.classes[target]
                .methods
                .iter()
                .filter(|(_, function)| historical.functions[*function].signature.is_some())
                .cloned()
                .collect(),
            class_methods: historical.classes[target].class_methods.clone(),
            qualified_impls: historical.classes[target].qualified_impls.clone(),
        };
        self.runtime
            .registry_mut()
            .begin_transaction(class)
            .map_err(MachineError::Class)?;
        self.open_depth += 1;
        let outcome = (|| {
            self.stage_members((class, &members), &historical)?;
            self.install_decorator_applications(class, &applications, &historical, &classes)?;
            self.validate_replacements(&historical)?;
            self.runtime
                .registry_mut()
                .commit_transaction(class)
                .map_err(MachineError::Class)
        })();
        self.open_depth -= 1;
        self.pending_replacements.clear();
        if let Err(error) = outcome {
            self.runtime.registry_mut().roll_back_transaction(class);
            self.wrapper_chains = chains;
            self.method_signatures = signatures;
            self.decorator_metadata.truncate(metadata);
            return Err(error);
        }
        Ok(Value::Symbol(actual))
    }

    fn history_source(
        &self,
        (current, name): (&Program, &str),
        arguments: &[Value],
    ) -> Result<(String, &str), MachineError> {
        let revision = match arguments {
            [Value::Integer(number)] => number
                .to_u64()
                .ok_or(MachineError::RevisionArtifactUnavailable)?,
            [] | [Value::Symbol(_)] => return Err(MachineError::RevisionArtifactUnavailable),
            _ => return Err(MachineError::ArgumentError),
        };
        let mut records = self.history.artifacts.iter().filter(|record| {
            record.package_id == current.package_id()
                && record.api_major == current.api_major()
                && record.logical_owner == name
                && record.revision == revision
        });
        let record = records
            .next()
            .ok_or(MachineError::RevisionArtifactUnavailable)?;
        if records.next().is_some() {
            return Err(MachineError::RevisionArtifactUnavailable);
        }
        let (_, digest, source) = &record.artifact;
        let actual = iris_runtime::artifact_digest(source.as_bytes());
        if actual != digest.strip_prefix("b3:").unwrap_or(digest) {
            return Err(MachineError::RevisionArtifactUnavailable);
        }
        Ok((actual, source))
    }

    fn validate_history_shape(
        &self,
        (class, current_index, target): (ClassId, usize, usize),
        (current, historical): (&Program, &Program),
    ) -> Result<(), MachineError> {
        let active = self
            .runtime
            .registry()
            .active(class)
            .map_err(MachineError::Class)?;
        let original = &current.classes[current_index];
        let replacement = &historical.classes[target];
        if original.generic
            || !active.properties().is_empty()
            || !active.modules().is_empty()
            || !active.class_vars().is_empty()
            || original.superclass.is_some()
        {
            return Err(MachineError::UnsupportedConstruct);
        }
        let contract_names = |program: &Program, index: usize| {
            program.classes[index]
                .contracts
                .iter()
                .map(|contract| program.contracts[*contract].name.clone())
                .collect::<std::collections::BTreeSet<_>>()
        };
        if contract_names(current, current_index) != contract_names(historical, target)
            || active.methods().len() != replacement.methods.len()
            || active.singleton_methods().len() != replacement.class_methods.len()
        {
            return Err(MachineError::TypeContractError);
        }
        for (table, replacements) in [
            (active.methods(), &replacement.methods),
            (active.singleton_methods(), &replacement.class_methods),
        ] {
            for (name, function) in replacements {
                let selector = selector_id(current, name).ok_or(MachineError::TypeContractError)?;
                let identity = table
                    .get(&selector)
                    .ok_or(MachineError::TypeContractError)?;
                let method = self
                    .runtime
                    .registry()
                    .method_by_id(*identity)
                    .ok_or(MachineError::UnsupportedConstruct)?;
                let owner = self
                    .resolve_method_body(method.body(), current)
                    .map_err(|_| MachineError::UnsupportedConstruct)?;
                if owner.program.functions[owner.function]
                    .instructions
                    .iter()
                    .any(|instruction| {
                        matches!(
                            instruction,
                            crate::Instruction::NativeCall { .. }
                                | crate::Instruction::NativeFixture { .. }
                                | crate::Instruction::FfiOpen { .. }
                        )
                    })
                {
                    return Err(MachineError::UnsupportedConstruct);
                }
                match (
                    &owner.program.functions[owner.function].signature,
                    &historical.functions[*function].signature,
                ) {
                    (None, None) if name == "to_bool" => {}
                    (Some(promise), Some(signature)) => {
                        if !promise.type_parameters.is_empty() {
                            return Err(MachineError::UnsupportedConstruct);
                        }
                        if promise.visibility != signature.visibility
                            || !iris_syntax::method_signature_compatible(
                                signature,
                                promise,
                                |source, target| {
                                    super::nominal_relation::nominal_subtype(
                                        current, source, target,
                                    )
                                },
                            )
                        {
                            return Err(MachineError::TypeContractError);
                        }
                        let capability = match promise.kind {
                            iris_syntax::MethodKind::Property => {
                                iris_runtime::Capability::PropertyBody
                            }
                            iris_syntax::MethodKind::Instance | iris_syntax::MethodKind::Class => {
                                iris_runtime::Capability::MethodBody
                            }
                            iris_syntax::MethodKind::Module => {
                                return Err(MachineError::UnsupportedConstruct);
                            }
                        };
                        self.runtime
                            .registry()
                            .require_candidate_meta_capability(class, capability)
                            .map_err(MachineError::Class)?;
                    }
                    _ => return Err(MachineError::TypeContractError),
                }
            }
        }
        Ok(())
    }
}
