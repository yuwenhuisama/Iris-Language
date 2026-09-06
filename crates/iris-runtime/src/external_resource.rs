use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone)]
pub struct ExternalResource(Arc<ResourceIdentity>);
struct ResourceIdentity {
    name: String,
    open: AtomicBool,
}
impl ExternalResource {
    pub fn new(name: String) -> Self {
        Self(Arc::new(ResourceIdentity {
            name,
            open: AtomicBool::new(true),
        }))
    }
    pub fn type_name(&self) -> &str {
        &self.0.name
    }
    pub fn is_open(&self) -> bool {
        self.0.open.load(Ordering::Acquire)
    }
    pub fn take_open(&self) -> bool {
        self.0.open.swap(false, Ordering::AcqRel)
    }
}
impl PartialEq for ExternalResource {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl std::fmt::Debug for ExternalResource {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        output
            .debug_struct("ExternalResource")
            .field("type", &self.0.name)
            .field("open", &self.is_open())
            .finish()
    }
}
