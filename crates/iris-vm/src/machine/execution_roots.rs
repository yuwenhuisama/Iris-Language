use super::{Machine, MachineError};
use crate::compile::Register;
use iris_runtime::Value;
use std::cell::{Ref, RefCell};
use std::rc::Rc;

pub(super) struct ActiveFrame {
    pub registers: Rc<RefCell<Vec<Value>>>,
    pub selected: Option<Vec<Register>>,
    pub converted: Vec<(iris_runtime::ObjectId, MachineError)>,
}

pub(super) struct RegisterWindow<'frame> {
    file: &'frame RefCell<Vec<Value>>,
    read: Option<Ref<'frame, Vec<Value>>>,
}

impl<'frame> RegisterWindow<'frame> {
    pub(super) fn new(file: &'frame RefCell<Vec<Value>>) -> Self {
        Self {
            file,
            read: Some(file.borrow()),
        }
    }

    pub(super) fn write(&mut self, index: usize, value: Value) {
        self.release();
        self.file.borrow_mut()[index] = value;
        self.read = Some(self.file.borrow());
    }

    pub(super) fn release(&mut self) {
        self.read = None;
    }
}

impl std::ops::Deref for RegisterWindow<'_> {
    type Target = Vec<Value>;
    fn deref(&self) -> &Self::Target {
        match &self.read {
            Some(read) => read,
            None => unreachable!("released register window only exits its instruction"),
        }
    }
}

pub(super) trait LocalRoots {
    fn append_roots(&self, roots: &mut Vec<Value>);
}

impl LocalRoots for Option<Value> {
    fn append_roots(&self, roots: &mut Vec<Value>) {
        roots.extend(self.iter().cloned());
    }
}

impl LocalRoots for Result<Value, MachineError> {
    fn append_roots(&self, roots: &mut Vec<Value>) {
        match self {
            Ok(value) => roots.push(value.clone()),
            Err(MachineError::Raised(raised)) => roots.extend([raised.0.clone(), raised.1.clone()]),
            Err(_) => {}
        }
    }
}

impl Machine {
    pub(super) fn with_local_roots<Root: LocalRoots + 'static, Output>(
        &mut self,
        root: Root,
        run: impl FnOnce(&mut Self, &Root) -> Output,
    ) -> (Root, Output) {
        let root = Rc::new(root);
        self.local_roots.push(root.clone());
        let output = run(self, &root);
        self.local_roots.pop();
        match Rc::into_inner(root) {
            Some(root) => (root, output),
            None => unreachable!("local root ownership is scoped to this call"),
        }
    }
}
