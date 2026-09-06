use super::SourceEvaluator;
use iris_runtime::{ClassId, ContractId};
use std::collections::HashMap;

#[derive(Default)]
pub(super) struct PackageContexts {
    pub(super) packages: HashMap<String, (u64, Option<String>)>,
    pub(super) classes: HashMap<ClassId, (String, u64)>,
    pub(super) contracts: HashMap<ContractId, (String, u64)>,
}

impl SourceEvaluator {
    pub(super) fn class_identity(&self, class: ClassId) -> (&str, u64) {
        self.package_contexts.classes.get(&class).map_or(
            (self.package.as_str(), self.api_major),
            |(package, major)| (package.as_str(), *major),
        )
    }

    pub(super) fn select_package_metadata(&mut self) {
        if let Some((major, version)) = self.package_contexts.packages.get(&self.package) {
            self.api_major = *major;
            self.package_version = version.clone();
        }
    }
}
