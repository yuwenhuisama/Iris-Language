use super::{MachineError, VerifyError};
use crate::compile::Program;
use iris_runtime::{ClassId, MethodBody, Selector};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CodeBundleId(usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct FunctionRef {
    bundle: CodeBundleId,
    index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CodeLink {
    id: CodeBundleId,
    pub(crate) classes: RefCell<Vec<ClassId>>,
    pub(crate) selectors: HashMap<String, Selector>,
    pub(crate) modules: RefCell<Vec<(String, iris_runtime::ModuleId)>>,
    pub(crate) contracts: Vec<iris_runtime::ContractId>,
    bodies: Vec<MethodBody>,
    pub(crate) history_methods: RefCell<HashMap<(ClassId, Selector), iris_runtime::Method>>,
}

struct CodeBundle {
    source: Rc<Program>,
    executable: Rc<Program>,
}

#[derive(Default)]
pub(super) struct CodeStore {
    bundles: Vec<CodeBundle>,
    functions: Vec<FunctionRef>,
    selectors: HashMap<String, Selector>,
    next_contract: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ResolvedBody {
    pub(super) program: Rc<Program>,
    pub(super) classes: Vec<ClassId>,
    pub(super) function: usize,
}

impl CodeStore {
    pub(super) fn allocate_contract(&mut self) -> Result<iris_runtime::ContractId, MachineError> {
        let identity = iris_runtime::ContractId::new(self.next_contract);
        self.next_contract = self
            .next_contract
            .checked_add(1)
            .ok_or(MachineError::UnsupportedConstruct)?;
        Ok(identity)
    }
    pub(super) fn register(&mut self, source: &Program) -> Result<Rc<Program>, MachineError> {
        if let Ok(owner) = self.owner(source) {
            return Ok(owner);
        }
        self.load(source)
    }

    pub(super) fn load(&mut self, source: &Program) -> Result<Rc<Program>, MachineError> {
        let id = CodeBundleId(self.bundles.len());
        let mut selectors = HashMap::new();
        for name in super::selector_names(source) {
            let proposed = super::selector_id(source, name)
                .ok_or_else(|| MachineError::UnknownSelector(name.to_owned()))?;
            let selector = match self.selectors.get(name) {
                Some(selector) => *selector,
                None => {
                    let mut raw = proposed.raw();
                    while self.selectors.values().any(|known| known.raw() == raw) {
                        raw = raw
                            .checked_add(1)
                            .ok_or(MachineError::UnsupportedConstruct)?;
                    }
                    let selector = Selector::new(raw);
                    self.selectors.insert(name.to_owned(), selector);
                    selector
                }
            };
            selectors.insert(name.to_owned(), selector);
        }
        let mut bodies = Vec::with_capacity(source.functions.len());
        for index in 0..source.functions.len() {
            let raw = u64::try_from(self.functions.len())
                .map_err(|_| MachineError::UnsupportedConstruct)?;
            bodies.push(MethodBody::new(raw));
            self.functions.push(FunctionRef { bundle: id, index });
        }
        let mut executable = source.clone();
        let mut contracts = Vec::with_capacity(source.contracts.len());
        for declaration in &source.contracts {
            let identity = match declaration.core_identity {
                Some(identity) => identity,
                None => {
                    let identity = iris_runtime::ContractId::new(self.next_contract);
                    self.next_contract = self
                        .next_contract
                        .checked_add(1)
                        .ok_or(MachineError::UnsupportedConstruct)?;
                    identity
                }
            };
            contracts.push(identity);
        }
        executable.link = Some(Box::new(CodeLink {
            id,
            classes: RefCell::new(Vec::new()),
            selectors,
            modules: RefCell::new(Vec::new()),
            contracts,
            bodies,
            history_methods: RefCell::new(HashMap::new()),
        }));
        let executable = Rc::new(executable);
        self.bundles.push(CodeBundle {
            source: Rc::new(source.clone()),
            executable: Rc::clone(&executable),
        });
        Ok(executable)
    }

    pub(super) fn owner(&self, program: &Program) -> Result<Rc<Program>, MachineError> {
        let bundle = match &program.link {
            Some(link) => self.bundles.get(link.id.0),
            None => self
                .bundles
                .iter()
                .rev()
                .find(|bundle| bundle.source.as_ref() == program),
        };
        bundle
            .map(|bundle| Rc::clone(&bundle.executable))
            .ok_or(MachineError::UnsupportedConstruct)
    }

    pub(super) fn body(
        &self,
        function: usize,
        program: &Program,
    ) -> Result<MethodBody, MachineError> {
        let owner = self.owner(program)?;
        owner
            .link
            .as_ref()
            .and_then(|link| link.bodies.get(function))
            .copied()
            .ok_or(MachineError::Invalid(VerifyError::UnknownFunction {
                function,
            }))
    }

    pub(super) fn resolve(&self, handle: usize) -> Option<ResolvedBody> {
        let reference = self.functions.get(handle)?;
        let program = Rc::clone(&self.bundles.get(reference.bundle.0)?.executable);
        let classes = program.link.as_ref()?.classes.borrow().clone();
        Some(ResolvedBody {
            program,
            classes,
            function: reference.index,
        })
    }

    pub(super) fn retain_classes(
        &self,
        program: &Program,
        classes: &[ClassId],
    ) -> Result<(), MachineError> {
        let owner = self.owner(program)?;
        let link = owner
            .link
            .as_ref()
            .ok_or(MachineError::UnsupportedConstruct)?;
        link.classes.replace(classes.to_vec());
        Ok(())
    }

    pub(super) fn selector_name(&self, selector: Selector) -> Option<String> {
        self.selectors
            .iter()
            .find_map(|(name, known)| (*known == selector).then(|| name.clone()))
    }

    pub(super) fn class_owner(&self, class: ClassId) -> Option<(Rc<Program>, usize)> {
        self.bundles.iter().find_map(|bundle| {
            let index = bundle
                .executable
                .link
                .as_ref()?
                .classes
                .borrow()
                .iter()
                .position(|known| *known == class)?;
            Some((Rc::clone(&bundle.executable), index))
        })
    }

    pub(super) fn module_owner(
        &self,
        module: iris_runtime::ModuleId,
    ) -> Option<(Rc<Program>, usize)> {
        self.bundles.iter().find_map(|bundle| {
            let program = &bundle.executable;
            let link = program.link.as_ref()?;
            let modules = link.modules.borrow();
            let (name, _) = modules.iter().find(|(_, identity)| *identity == module)?;
            let index = program
                .modules
                .iter()
                .position(|known| known.name == *name)?;
            Some((Rc::clone(program), index))
        })
    }

    pub(super) fn selectors(&self) -> impl Iterator<Item = Selector> + '_ {
        self.selectors.values().copied()
    }
}
