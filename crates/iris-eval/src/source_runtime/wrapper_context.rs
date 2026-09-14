use crate::source_runtime::SourceEvaluator;
use iris_runtime::{ClassId, ContractId, Method};
use iris_syntax::TypeExpression;
use std::collections::HashMap;

#[derive(Clone)]
pub(in crate::source_runtime) struct LexicalContext {
    type_bindings: HashMap<String, TypeExpression>,
    class: Option<ClassId>,
    method: Option<Method>,
    contract: Option<ContractId>,
    main: Option<ClassId>,
    package: String,
    major: u64,
    version: Option<String>,
    observing: bool,
    source: String,
    source_path: String,
    artifact: Option<std::rc::Rc<crate::source_runtime::rollback_artifact::BodyContext>>,
}

impl crate::source_runtime::gc_roots::TraceRoots for LexicalContext {
    fn trace_roots(&self, roots: &mut Vec<iris_runtime::Value>) {
        roots.extend(self.method.map(iris_runtime::Value::Method));
        if let Some(artifact) = &self.artifact {
            roots.extend(
                artifact
                    .methods
                    .values()
                    .copied()
                    .map(iris_runtime::Value::Method),
            );
        }
    }
}

impl SourceEvaluator {
    pub(in crate::source_runtime) fn wrapper_context(&self) -> LexicalContext {
        LexicalContext {
            type_bindings: self.method_type_bindings.clone(),
            class: self.lexical_class,
            method: self.current_method,
            contract: self.current_contract,
            main: self.module_body_main,
            package: self.package.clone(),
            major: self.api_major,
            version: self.package_version.clone(),
            observing: self.observing,
            source: self.source.clone(),
            source_path: self.source_path.clone(),
            artifact: self.rollback_state.context.clone(),
        }
    }

    pub(in crate::source_runtime) fn restore_wrapper_context(&mut self, context: LexicalContext) {
        self.method_type_bindings = context.type_bindings;
        self.lexical_class = context.class;
        self.current_method = context.method;
        self.current_contract = context.contract;
        self.module_body_main = context.main;
        self.package = context.package;
        self.api_major = context.major;
        self.package_version = context.version;
        self.observing = context.observing;
        self.source = context.source;
        self.source_path = context.source_path;
        self.rollback_state.context = context.artifact;
    }
}
