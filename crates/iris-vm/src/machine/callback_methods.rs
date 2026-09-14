use super::{Machine, MachineError, OpenGroupState, selector_id};
use crate::compile::{Program, decorators::Application};
use iris_runtime::ClassId;

impl Machine {
    pub(super) fn declare_callback_method(
        &mut self,
        target: (usize, usize),
        applications: &[Application],
        program: &Program,
        classes: &[ClassId],
    ) -> Result<(), MachineError> {
        let (class_index, function) = target;
        let class = classes[class_index];
        let result = (|| {
            if self.open_depth == 0 {
                return Err(MachineError::MetaTransactionError);
            }
            let signature = program.functions[function]
                .signature
                .as_ref()
                .ok_or(MachineError::UnsupportedConstruct)?;
            let selector =
                selector_id(program, &signature.selector).ok_or(MachineError::NameError)?;
            let visibility = match signature.visibility {
                iris_syntax::Visibility::Public => iris_runtime::Visibility::Public,
                iris_syntax::Visibility::Private => iris_runtime::Visibility::Private,
                iris_syntax::Visibility::Protected => iris_runtime::Visibility::Protected,
            };
            if signature.kind == iris_syntax::MethodKind::Property {
                self.runtime
                    .registry_mut()
                    .begin_transaction(class)
                    .map_err(MachineError::Class)?;
                let existing = self
                    .runtime
                    .registry()
                    .staged_method(class, selector)
                    .and_then(|identity| self.method_signatures.get(&identity));
                if existing
                    .is_some_and(|metadata| metadata.kind != iris_syntax::MethodKind::Property)
                {
                    return Err(MachineError::ArgumentError);
                }
                let capability = if existing.is_some() {
                    iris_runtime::Capability::PropertyBody
                } else {
                    iris_runtime::Capability::PropertySet
                };
                self.runtime
                    .registry()
                    .require_candidate_meta_capability(class, capability)
                    .map_err(MachineError::Class)?;
                let method = self
                    .runtime
                    .registry_mut()
                    .publish_origin_method(
                        class,
                        selector,
                        self.code.body(function, program)?,
                        visibility,
                    )
                    .map_err(MachineError::Class)?;
                self.remember_signature(method, program);
                self.pending_replacements.insert(
                    (class, selector, false),
                    super::method_removal::PendingMethod::Function(method.body()),
                );
            } else {
                self.define_checked((class, selector, function), program)?;
            }
            if signature.kind != iris_syntax::MethodKind::Property
                && visibility != iris_runtime::Visibility::Public
            {
                let method = self
                    .runtime
                    .registry_mut()
                    .publish_method(
                        class,
                        selector,
                        self.code.body(function, program)?,
                        visibility,
                    )
                    .map_err(MachineError::Class)?;
                self.remember_signature(method, program);
            }
            self.install_decorator_applications(
                class,
                &applications.iter().collect::<Vec<_>>(),
                program,
                classes,
            )?;
            Ok(())
        })();
        if result.is_err() {
            self.open_group_state = Some(OpenGroupState::Aborted);
        }
        result
    }
}
