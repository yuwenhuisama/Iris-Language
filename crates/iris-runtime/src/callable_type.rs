use std::{collections::HashMap, error::Error, fmt, sync::Arc};

use crate::{ClassId, ClassRegistry, ComposedType, NominalType, TypeAtom, Value};

pub(crate) const CALLABLE_ID_DOMAIN: u64 = crate::core_registration::CORE_ID_DOMAIN | (1 << 62);

/// Logical callable Type kind, not an ordinary user Class.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CallableKind {
    Closure,
    BoundMethod,
}

/// An already-resolved, normalized closed Type within a callable signature.
///
/// Names and aliases must be resolved before construction. Composed Types must
/// use the same normal form as ordinary Type reification. Nested closed nominal
/// arguments (including Task results) retain their full structure. A nested
/// callable can use its interned identity as a nominal leaf in this encoding.
/// This boundary admits Types only, never arbitrary runtime Values.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SignatureType {
    Nominal(NominalType),
    Composed(ComposedType),
}

impl From<NominalType> for SignatureType {
    fn from(nominal: NominalType) -> Self {
        Self::Nominal(nominal)
    }
}

impl From<ComposedType> for SignatureType {
    fn from(composed: ComposedType) -> Self {
        Self::Composed(composed)
    }
}

/// The invariant S in Closure<S> or BoundMethod<S>: ordered Types and result.
///
/// Parameter names, categories, optionality, defaults and async execution flags
/// belong to invocation metadata, not S. An async callable's reified result must
/// already be its closed Task<T> Type. No subtyping or covariance is inferred.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CallableSignature {
    parameters: Vec<SignatureType>,
    result: SignatureType,
}

impl CallableSignature {
    pub const fn new(parameters: Vec<SignatureType>, result: SignatureType) -> Self {
        Self { parameters, result }
    }

    pub fn parameters(&self) -> &[SignatureType] {
        &self.parameters
    }

    pub const fn result(&self) -> &SignatureType {
        &self.result
    }
}

/// Canonical callable metadata used for Type reflection and exact admission.
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct CallableType {
    kind: CallableKind,
    signature: CallableSignature,
}

impl CallableType {
    pub const fn kind(&self) -> CallableKind {
        self.kind
    }

    pub const fn signature(&self) -> &CallableSignature {
        &self.signature
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallableTypeError {
    IdentityExhausted,
}

impl fmt::Display for CallableTypeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IdentityExhausted => {
                formatter.write_str("Iris callable Type identity space exhausted")
            }
        }
    }
}

impl Error for CallableTypeError {}

#[derive(Debug, Default)]
pub(crate) struct CallableTypes {
    identities: HashMap<Arc<CallableType>, ClassId>,
    entries: Vec<Arc<CallableType>>,
}

impl ClassRegistry {
    /// Interns an exact closed callable Type without publishing an ordinary Class.
    ///
    /// Returns Value::Type(identity, []) using the upper quarter of the u64
    /// identity space, disjoint from user identities and fixed core identities.
    /// The identity is stable for this registry's lifetime, not across runtimes
    /// or registration orders. No user Class, revision, commit or Method sequence
    /// is consumed. Repeated equal normalized signatures return equal Type values.
    ///
    /// Consumers must use `callable_type` before treating this Type's identity as
    /// a nominal Class. That lookup reports the callable kind and S; `class`, `active`,
    /// construction and dispatch intentionally reject callable Type identities.
    /// No Value/TypeAtom variants or backend behavior are changed by this API.
    pub fn intern_callable_type(
        &mut self,
        kind: CallableKind,
        signature: CallableSignature,
    ) -> Result<Value, CallableTypeError> {
        self.intern_callable_identity(kind, signature)
            .map(|identity| Value::Type(identity, Vec::new()))
    }

    /// Reifies Block<S> as the transparent BoundMethod<S> | Closure<S> union.
    ///
    /// Returns `Value::ComposedType(ComposedType::Union(..))` with canonical
    /// sorted atoms. Only the two callable members are interned; the alias has
    /// no identity, Class, wrapper, or publication counters of its own.
    /// S must already be resolved and normalized, exactly as for callable Types.
    /// Engine consumers must admit each member through its callable metadata,
    /// not through nominal Class lookup or generic variance.
    pub fn intern_block_alias(
        &mut self,
        signature: CallableSignature,
    ) -> Result<Value, CallableTypeError> {
        let bound = self.intern_callable_identity(CallableKind::BoundMethod, signature.clone())?;
        let closure = self.intern_callable_identity(CallableKind::Closure, signature)?;
        let mut members = vec![
            TypeAtom::Nominal(bound, Vec::new()),
            TypeAtom::Nominal(closure, Vec::new()),
        ];
        members.sort();
        Ok(Value::ComposedType(ComposedType::Union(members)))
    }

    fn intern_callable_identity(
        &mut self,
        kind: CallableKind,
        signature: CallableSignature,
    ) -> Result<ClassId, CallableTypeError> {
        let descriptor = CallableType { kind, signature };
        if let Some(identity) = self.callable_types.identities.get(&descriptor) {
            return Ok(*identity);
        }
        let ordinal = u64::try_from(self.callable_types.entries.len())
            .map_err(|_| CallableTypeError::IdentityExhausted)?;
        let raw = CALLABLE_ID_DOMAIN
            .checked_add(ordinal)
            .ok_or(CallableTypeError::IdentityExhausted)?;
        let identity = ClassId::new(raw);
        let descriptor = Arc::new(descriptor);
        self.callable_types
            .identities
            .insert(Arc::clone(&descriptor), identity);
        self.callable_types.entries.push(descriptor);
        Ok(identity)
    }

    /// Resolves only callable Type identities issued by this registry.
    pub fn callable_type(&self, identity: ClassId) -> Option<&CallableType> {
        let ordinal = usize::try_from(identity.raw().checked_sub(CALLABLE_ID_DOMAIN)?).ok()?;
        self.callable_types.entries.get(ordinal).map(Arc::as_ref)
    }
}
