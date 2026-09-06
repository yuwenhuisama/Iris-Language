use super::{EvaluationError, SourceEvaluator};
use iris_native_host::NativeRegistry;
use iris_runtime::Value;
use std::rc::Rc;

impl SourceEvaluator {
    pub(crate) fn install_natives(
        &mut self,
        registry: Rc<NativeRegistry>,
    ) -> Result<(), EvaluationError> {
        let declarations = registry.declarations().map_err(native_error)?;
        let previous = self.package.clone();
        let previous_major = self.api_major;
        let previous_version = self.package_version.clone();
        for metadata in registry.metadata() {
            self.package_contexts.packages.insert(
                metadata.package_id,
                (u64::from(metadata.api_major), Some(metadata.version)),
            );
        }
        for (package, module) in declarations {
            if self.module_names.contains_key(&module.name) {
                return Err(EvaluationError::UnsupportedConstruct);
            }
            self.package = package;
            self.select_package_metadata();
            self.module(&module)?;
            let module_id = self.module_names[&module.name];
            for statement in module.body {
                if let iris_syntax::Statement::Method(method) = statement {
                    let selector = self.selector(&method.selector);
                    let body = self.module_methods[&(module_id, selector)].body().raw();
                    self.native_bodies
                        .insert(body, format!("{}.{}", module.name, method.selector));
                }
            }
        }
        self.package = previous;
        self.api_major = previous_major;
        self.package_version = previous_version;
        self.natives = Some(registry);
        Ok(())
    }
    pub(super) fn native_resource_send(
        &mut self,
        value: &iris_runtime::ExternalResource,
        selector: &str,
        arguments: &[Value],
    ) -> Result<Value, EvaluationError> {
        match (selector, arguments) {
            ("close", []) => self
                .natives
                .as_ref()
                .ok_or(EvaluationError::UnsupportedConstruct)?
                .close(value)
                .map_err(|error| self.native_error(error)),
            ("closed?", []) => Ok(Value::Bool(!value.is_open())),
            ("same?" | "==", [Value::ExternalResource(other)]) => Ok(Value::Bool(value == other)),
            _ => Err(EvaluationError::MessageNotFound {
                receiver_class: value.type_name().to_owned(),
                selector: selector.to_owned(),
            }),
        }
    }
    pub(super) fn native_error(&mut self, error: iris_native_host::NativeError) -> EvaluationError {
        let (value, context) = error.into_propagation();
        self.active_context = Some(context);
        EvaluationError::Raised(value)
    }
}
pub(super) fn native_error(error: iris_native_host::NativeError) -> EvaluationError {
    EvaluationError::Raised(error.raised_value())
}

impl crate::Session {
    pub fn with_natives(registry: Rc<NativeRegistry>) -> Result<Self, EvaluationError> {
        let mut session = Self::new()?;
        session.evaluator.install_natives(registry)?;
        Ok(session)
    }
    pub fn evaluate_in_package(
        &mut self,
        package: &str,
        source: &str,
    ) -> Result<Value, EvaluationError> {
        self.evaluator.enter_package(package, source);
        self.evaluate(source)
    }
}
pub fn evaluate_with_natives(
    source: &str,
    registry: Rc<NativeRegistry>,
) -> Result<Value, EvaluationError> {
    crate::Session::with_natives(registry)?.evaluate(source)
}
pub fn evaluate_packages_with_natives(
    programs: &[(String, String)],
    registry: Rc<NativeRegistry>,
) -> Result<Value, EvaluationError> {
    let mut session = crate::Session::with_natives(registry)?;
    let mut value = Value::Nil;
    for (package, source) in programs {
        let parsed = iris_parser::parse(source);
        if !parsed.program_accepted {
            return Err(EvaluationError::ParseDiagnostic);
        }
        session.evaluator.enter_package(package, source);
        match session.evaluator.program(&parsed.program) {
            Ok(result) => value = result,
            Err(EvaluationError::UnsupportedConstruct) if parsed.program.statements.is_empty() => {}
            Err(error) => return Err(error),
        }
    }
    Ok(value)
}

pub fn evaluate_package_tree_with_natives(
    sources: &[iris_native_host::PackageSource],
    registry: Rc<NativeRegistry>,
) -> Result<Value, EvaluationError> {
    iris_native_host::PackageSource::validate_metadata(sources).map_err(native_error)?;
    for source in sources {
        source.validate_imports().map_err(native_error)?;
    }
    let mut session = crate::Session::with_natives(registry)?;
    let mut value = Value::Nil;
    for source in sources {
        let metadata = (u64::from(source.api_major), Some(source.version.clone()));
        if let Some(previous) = session
            .evaluator
            .package_contexts
            .packages
            .insert(source.package_id.clone(), metadata.clone())
            && previous != metadata
        {
            return Err(native_error(iris_native_host::NativeError::Conflict));
        }
    }
    for source in sources {
        let parsed = iris_parser::parse(&source.source);
        session
            .evaluator
            .enter_package(&source.package_id, &source.source);
        session
            .evaluator
            .enter_api_major(u64::from(source.api_major));
        session
            .evaluator
            .enter_package_resolution(Some(source.version.clone()), Vec::new());
        session.evaluator.source_path = source.path.clone();
        match session.evaluator.program(&parsed.program) {
            Ok(result) => value = result,
            Err(EvaluationError::UnsupportedConstruct) if parsed.program.statements.is_empty() => {}
            Err(error) => return Err(error),
        }
    }
    Ok(value)
}
