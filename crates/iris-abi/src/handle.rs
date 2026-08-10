//! `IRIS-V1-FFI-C007` opaque handles.

/// An opaque reference to one Iris value or metadata object.
///
/// `IRIS-V1-FFI-C007` forbids a raw managed pointer, Class pointer, Method
/// pointer, GC address, object layout address, vtable address or interior
/// pointer from crossing the ABI, so this is an OPAQUE INTEGER naming a slot in
/// a runtime-owned table rather than an address. That indirection is also what
/// lets `IRIS-V1-FFI-C008` permit the runtime to move, compact, pin or intern
/// managed values while live handles keep denoting the same Iris value.
///
/// The bits carry a runtime tag, a table slot and a generation counter.
/// `IRIS-V1-FFI-C009` forbids treating the numeric value as stable identity,
/// forbids persisting it, and forbids assuming a slot is never reused, so the
/// generation is what makes a stale handle DETECTABLE after its slot is reused
/// rather than silently denoting the new occupant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct IrisHandle(pub u64);

impl IrisHandle {
    /// The handle that names nothing.
    pub const NULL: Self = Self(0);

    /// Packs a runtime tag, slot and generation into one opaque handle.
    #[must_use]
    pub const fn pack(runtime: u16, slot: u32, generation: u16) -> Self {
        Self(((runtime as u64) << 48) | ((generation as u64) << 32) | slot as u64)
    }

    /// The runtime this handle belongs to.
    ///
    /// `IRIS-V1-FFI-C009` binds a handle to exactly one runtime, so the owner
    /// travels IN the handle and a foreign handle is rejected on arrival.
    #[must_use]
    pub const fn runtime(self) -> u16 {
        (self.0 >> 48) as u16
    }

    /// The generation stamped when this slot was issued.
    #[must_use]
    pub const fn generation(self) -> u16 {
        (self.0 >> 32) as u16
    }

    /// The table slot this handle names.
    #[must_use]
    pub const fn slot(self) -> u32 {
        self.0 as u32
    }

    /// Whether this handle names nothing.
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }
}

/// How a handle's root is released.
///
/// `IRIS-V1-FFI-C010` makes every ABI value handle a ROOTED handle whatever its
/// release mode, so this selects when the root ends rather than whether one
/// exists.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum IrisHandleKind {
    /// Rooted until an explicit release call.
    ExplicitRelease = 0,
    /// Rooted until its handle frame is closed.
    ///
    /// `IRIS-V1-FFI-C008` bulk-releases every live handle in the frame.
    Scoped = 1,
}

/// An opaque handle frame.
///
/// `IRIS-V1-FFI-C008` bounds a scoped handle's lifetime by its frame, and
/// closing the frame bulk-releases every live handle it holds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct IrisFrame(pub u64);
