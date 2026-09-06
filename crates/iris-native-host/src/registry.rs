use crate::{FunctionMetadata, Metadata, NativeError, resource::ResourceRecord};
use iris_native_sdk::{EntryV1, FunctionV1, ModuleV1, ResourceV1, SUCCESS};
use iris_runtime::Value;
use sha2::{Digest, Sha256};
use std::{cell::RefCell, collections::BTreeSet, path::Path, rc::Rc};

#[derive(Clone, Debug, Default)]
pub struct NativePolicy {
    pub allow_native: bool,
    pub permissions: BTreeSet<String>,
    pub max_resources: usize,
}
pub struct TrustedModule<'a> {
    pub path: &'a Path,
    pub metadata: &'a [u8],
    pub metadata_sha256: [u8; 32],
    pub artifact_sha256: [u8; 32],
}
pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

pub(crate) struct Loaded {
    pub metadata: Metadata,
    pub functions: Vec<FunctionV1>,
    pub resources: Vec<(String, ResourceV1)>,
    pub quota: usize,
    pub _library: Option<libloading::Library>,
}
#[derive(Default)]
pub struct NativeRegistry {
    pub(crate) resources: RefCell<Vec<ResourceRecord>>,
    pub(crate) modules: RefCell<Vec<Rc<Loaded>>>,
}
impl NativeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads only explicitly trusted native code after verifying both digests.
    /// The caller must keep the artifact and transitive libraries immutable during load.
    pub fn load(
        &self,
        trusted: TrustedModule<'_>,
        policy: &NativePolicy,
    ) -> Result<(), NativeError> {
        if !policy.allow_native {
            return Err(NativeError::Denied);
        }
        if sha256(trusted.metadata) != trusted.metadata_sha256 {
            return Err(NativeError::Digest);
        }
        let metadata: Metadata = serde_json::from_slice(trusted.metadata)?;
        metadata.validate()?;
        if !metadata
            .permissions
            .required
            .iter()
            .all(|permission| policy.permissions.contains(permission))
        {
            return Err(NativeError::Denied);
        }
        if self.modules.borrow().iter().any(|loaded| {
            loaded.metadata.package_id == metadata.package_id
                || loaded
                    .metadata
                    .modules
                    .iter()
                    .any(|held| metadata.modules.iter().any(|new| held.name == new.name))
        }) {
            return Err(NativeError::Conflict);
        }
        if sha256(&std::fs::read(trusted.path)?) != trusted.artifact_sha256 {
            return Err(NativeError::Digest);
        }
        // SAFETY: admission and actual digests were checked before constructors execute;
        // the explicit trust contract covers this code and its transitive dependencies.
        let library = unsafe { libloading::Library::new(trusted.path) }?;
        // SAFETY: the trusted module exports precisely the documented C entry signature.
        let entry = unsafe { library.get::<EntryV1>(b"iris_native_module_v1\0") }?;
        let mut descriptor = ModuleV1::default();
        // SAFETY: descriptor is initialized, aligned and exclusively writable for this call.
        let status = unsafe { entry(&mut descriptor) };
        if status != SUCCESS {
            return Err(NativeError::Protocol(status));
        }
        if descriptor.header.size < size_of::<ModuleV1>()
            || descriptor.metadata_sha256 != trusted.metadata_sha256
        {
            return Err(NativeError::Digest);
        }
        self.attach(metadata, descriptor, policy.max_resources, Some(library))
    }

    pub(crate) fn attach(
        &self,
        metadata: Metadata,
        descriptor: ModuleV1,
        quota: usize,
        library: Option<libloading::Library>,
    ) -> Result<(), NativeError> {
        if descriptor.header.major != 1
            || descriptor.header.size < size_of::<ModuleV1>()
            || descriptor.header.features != 0
            || descriptor.function_count
                != metadata
                    .modules
                    .iter()
                    .map(|module| module.functions.len())
                    .sum::<usize>()
            || descriptor.resource_count
                != metadata
                    .modules
                    .iter()
                    .map(|module| module.resources.len())
                    .sum::<usize>()
        {
            return Err(NativeError::Protocol(iris_native_sdk::INCOMPATIBLE_ABI));
        }
        let mut functions = Vec::new();
        for index in 0..descriptor.function_count {
            let callback = descriptor.function_at.ok_or(NativeError::Metadata)?;
            let mut function = None;
            // SAFETY: trusted callback, valid index and disjoint initialized output.
            let status = unsafe { callback(index, &mut function) };
            if status != SUCCESS {
                return Err(NativeError::Protocol(status));
            }
            functions.push(function.ok_or(NativeError::Metadata)?);
        }
        let mut resources = Vec::new();
        for module in &metadata.modules {
            for resource in &module.resources {
                let callback = descriptor.resource_at.ok_or(NativeError::Metadata)?;
                let mut entry = ResourceV1::default();
                // SAFETY: trusted callback, valid index and disjoint initialized output.
                let status = unsafe { callback(resources.len(), &mut entry) };
                if status != SUCCESS {
                    return Err(NativeError::Protocol(status));
                }
                if entry.size < size_of::<ResourceV1>()
                    || entry.close.is_none()
                    || entry.destroy.is_none()
                {
                    return Err(NativeError::Metadata);
                }
                resources.push((format!("{}::{}", module.name, resource.name), entry));
            }
        }
        self.modules.borrow_mut().push(Rc::new(Loaded {
            metadata,
            functions,
            resources,
            quota,
            _library: library,
        }));
        Ok(())
    }

    pub fn metadata(&self) -> Vec<Metadata> {
        self.modules
            .borrow()
            .iter()
            .map(|module| module.metadata.clone())
            .collect()
    }

    pub fn is_native_module(&self, name: &str) -> bool {
        self.modules.borrow().iter().any(|loaded| {
            loaded
                .metadata
                .modules
                .iter()
                .any(|module| module.name == name)
        })
    }

    pub fn call(&self, qualified: &str, arguments: &[Value]) -> Result<Value, NativeError> {
        let (loaded, index, module_name, metadata) =
            self.find(qualified).ok_or(NativeError::Metadata)?;
        if arguments.len() != metadata.parameters.len() {
            return Err(NativeError::Type);
        }
        for (argument, parameter) in arguments.iter().zip(&metadata.parameters) {
            self.check_type(argument, &parameter.value_type, &module_name, &loaded)?;
        }
        let function = loaded.functions[index];
        let frame = crate::frame::Frame::new(self, Rc::clone(&loaded));
        let handles = arguments
            .iter()
            .map(|value| frame.insert(value.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        let mut result = iris_native_sdk::CallResultV1::default();
        // SAFETY: frame stays live without an exclusive Rust borrow during callbacks;
        // arguments and result occupy disjoint storage, and the library is retained.
        let status = unsafe {
            function(
                &crate::callbacks::HOST,
                frame.context(),
                handles.as_ptr(),
                handles.len(),
                &mut result,
            )
        };
        let value = frame.finish(status, result)?;
        self.check_type(&value, &metadata.returns, &module_name, &loaded)?;
        Ok(value)
    }

    fn find(&self, qualified: &str) -> Option<(Rc<Loaded>, usize, String, FunctionMetadata)> {
        for loaded in self.modules.borrow().iter() {
            let mut index = 0;
            for module in &loaded.metadata.modules {
                for function in &module.functions {
                    if qualified == format!("{}.{}", module.name, function.name) {
                        return Some((
                            Rc::clone(loaded),
                            index,
                            module.name.clone(),
                            function.clone(),
                        ));
                    }
                    index += 1;
                }
            }
        }
        None
    }
}
