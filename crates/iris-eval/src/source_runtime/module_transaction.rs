use super::{EvaluationError, SourceEvaluator};
use iris_runtime::ModuleId;
use iris_syntax::ModuleDeclaration;

impl SourceEvaluator {
    pub(super) fn module_transaction(
        &mut self,
        module: ModuleId,
        declaration: &ModuleDeclaration,
    ) -> Result<(), EvaluationError> {
        let names = self.module_names.clone();
        let packages = self.module_packages.clone();
        let classes = self.module_classes.clone();
        let mains = self.module_mains.clone();
        let methods = self.module_methods.clone();
        let overrides = self.module_method_overrides.clone();
        let constants = self.module_constants.clone();
        let properties = self.class_level_properties.clone();
        let accessors = self.class_property_accessors.clone();
        let shared = self.shared_class_properties.clone();
        let chains = self.wrapper_chains.clone();
        let captured = self.captured_methods.clone();
        let singletons = self.singleton_identities.clone();
        let declarations = self.singleton_declarations.clone();
        let property_methods = self.property_methods.clone();
        let metadata = self.decorator_metadata.len();
        let first_body = self.next_body;
        let outcome = self
            .stage_module_declaration(module, declaration)
            .and_then(|()| self.publish_module_transaction());
        if let Err(error) = outcome {
            self.runtime.roll_back_group();
            self.module_names = names;
            self.module_packages = packages;
            self.module_classes = classes;
            self.module_mains = mains;
            self.module_methods = methods;
            self.module_method_overrides = overrides;
            self.module_constants = constants;
            self.class_level_properties = properties;
            self.class_property_accessors = accessors;
            self.shared_class_properties = shared;
            self.wrapper_chains = chains;
            self.captured_methods = captured;
            self.singleton_identities = singletons;
            self.singleton_declarations = declarations;
            self.property_methods = property_methods;
            self.decorator_metadata.truncate(metadata);
            self.bodies.retain(|body, _| *body < first_body);
            self.rollback_state
                .bodies
                .retain(|body, _| *body < first_body);
            self.next_body = first_body;
            return Err(error);
        }
        Ok(())
    }
}
