use crate::{
    CandidateRevision, ClassError, ClassId, ClassRegistry, ClassRevision, ContractId, LogicalClass,
    MetaCapabilities, RevisionId, StaticSpine,
};

pub(crate) const CORE_ID_DOMAIN: u64 = 1 << 63;

/// Runtime-owned Class identities, independent of source registration order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum CoreClass {
    Invocation,
    InvocationSignature,
    InvocationParameter,
    ArgumentChanges,
    DecoratorContext,
    Plan,
    Transformation,
    DecoratorProtocolError,
    Array,
    Hash,
    Symbol,
    Type,
    Tuple,
    TypeError,
    ArgumentError,
    /// Canonical nominal Task identity; engines own execution and closed result metadata.
    Task,
}

impl CoreClass {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Invocation" => Some(Self::Invocation),
            "InvocationSignature" => Some(Self::InvocationSignature),
            "InvocationParameter" => Some(Self::InvocationParameter),
            "ArgumentChanges" => Some(Self::ArgumentChanges),
            "DecoratorContext" => Some(Self::DecoratorContext),
            "Plan" => Some(Self::Plan),
            "Transformation" => Some(Self::Transformation),
            "DecoratorProtocolError" => Some(Self::DecoratorProtocolError),
            "Array" => Some(Self::Array),
            "Hash" => Some(Self::Hash),
            "Symbol" => Some(Self::Symbol),
            "Type" => Some(Self::Type),
            "Tuple" => Some(Self::Tuple),
            "TypeError" => Some(Self::TypeError),
            "ArgumentError" => Some(Self::ArgumentError),
            "Task" => Some(Self::Task),
            _ => None,
        }
    }

    pub const fn id(self) -> ClassId {
        ClassId::new(CORE_ID_DOMAIN | self as u64)
    }
}

/// The named protocol Contract identity in the runtime-owned allocation domain.
pub fn core_contract_id(name: &str) -> Option<ContractId> {
    let ordinal = match name {
        "ClassDecorator" => 0,
        "ModuleDecorator" => 1,
        "ContractDecorator" => 2,
        "MethodDecorator" => 3,
        "PropertyDecorator" => 4,
        _ => return None,
    };
    Some(ContractId::new(CORE_ID_DOMAIN | ordinal))
}

impl ClassRegistry {
    /// Installs a core origin without consuming source Class, revision or commit IDs.
    /// Core origins belong to bootstrap commit zero; later mutations publish normally.
    pub fn register_core_class(
        &mut self,
        core: CoreClass,
        superclass: ClassId,
    ) -> Result<ClassId, ClassError> {
        let class = core.id();
        if self.classes.contains_key(&class) {
            return Ok(class);
        }
        self.require_meta_capability(superclass, crate::Capability::Subclass)?;
        let spine = StaticSpine::new(1).with_meta_capabilities(MetaCapabilities::all());
        let capabilities = self.effective_meta_capabilities(spine, Some(superclass))?;
        let revision = RevisionId::new(class.raw());
        let candidate = CandidateRevision::origin(
            class,
            revision,
            spine,
            Some(superclass),
            self.origin_mro(class, Some(superclass))?,
            capabilities,
        );
        self.revisions.insert(
            revision,
            ClassRevision::from_candidate(candidate, revision, 0, capabilities),
        );
        self.classes
            .insert(class, LogicalClass::new(class, revision));
        Ok(class)
    }
}
