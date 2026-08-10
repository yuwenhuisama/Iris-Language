//! `IRIS-V1-FFI-C027` payload descriptors and `C030` Closeable resources.

use crate::{IrisHandle, IrisStatus};

/// Why a payload descriptor was refused at registration.
///
/// `IRIS-V1-FFI-C027` makes the runtime validate a descriptor BEFORE it owns
/// any payload storage, so each variant names the property that failed rather
/// than reporting a generic rejection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DescriptorRejection {
    /// Alignment was zero or not a power of two.
    AlignmentNotPowerOfTwo,
    /// Size was not a multiple of alignment, so an array of these would skew.
    SizeNotMultipleOfAlignment,
    /// `C029` forbids final cleanup that may raise into Iris.
    CleanupMayRaise,
}

impl DescriptorRejection {
    /// The stable diagnostic code for this rejection.
    #[must_use]
    pub const fn diagnostic(self) -> &'static str {
        match self {
            Self::AlignmentNotPowerOfTwo => "ffi.payload-alignment-invalid",
            Self::SizeNotMultipleOfAlignment => "ffi.payload-size-misaligned",
            Self::CleanupMayRaise => "ffi.payload-cleanup-may-raise",
        }
    }
}

/// Whether a payload's final cleanup may raise into Iris.
///
/// `IRIS-V1-FFI-C029` states that failures in final memory cleanup are runtime
/// diagnostics and NOT catchable language results, so a descriptor claiming it
/// may raise is refused at registration instead of being trusted and then
/// contained at drop time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CleanupPolicy {
    /// Cleanup cannot raise, which is the only conforming choice.
    NoRaise,
    /// Cleanup claims it may raise, which `C029` prohibits.
    MayRaise,
}

/// What a payload promises to report when the collector traces it.
///
/// `IRIS-V1-FFI-C028` limits trace logic to reporting managed handles or roots
/// and forbids it from creating Iris values, calling Iris Methods, raising, or
/// dereferencing moved objects. Modelling trace as DATA the payload declares,
/// rather than as an arbitrary callback the runtime invokes, is what makes
/// those prohibitions unreachable by construction.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TraceReport {
    /// The managed handles this payload keeps reachable.
    pub roots: Vec<IrisHandle>,
}

/// A runtime-owned native payload descriptor.
///
/// `IRIS-V1-FFI-C027` requires a native-backed Class storing a native payload
/// to register one of these, and gives the RUNTIME control of the allocation
/// and lifetime of the payload storage attached to Iris objects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayloadDescriptor {
    /// Payload storage size in bytes.
    pub size: u32,
    /// Required alignment in bytes.
    pub alignment: u32,
    /// What the payload reports to the collector.
    pub trace: TraceReport,
    /// Whether final cleanup may raise into Iris.
    pub cleanup: CleanupPolicy,
    /// Whether this payload represents an external resource.
    ///
    /// `IRIS-V1-FFI-C030` requires such a resource to offer explicit idempotent
    /// `Closeable` behaviour, because GC timing is not a resource-management
    /// promise.
    pub external_resource: bool,
}

impl PayloadDescriptor {
    /// Validates a descriptor for registration.
    ///
    /// `IRIS-V1-FFI-C027` makes the runtime own the payload storage, so a
    /// descriptor whose shape it could not lay out safely is refused here
    /// rather than producing an unsound allocation later.
    pub fn validate(&self) -> Result<(), DescriptorRejection> {
        if self.alignment == 0 || !self.alignment.is_power_of_two() {
            return Err(DescriptorRejection::AlignmentNotPowerOfTwo);
        }
        if !self.size.is_multiple_of(self.alignment) {
            return Err(DescriptorRejection::SizeNotMultipleOfAlignment);
        }
        // C029 makes cleanup failures diagnostics rather than catchable
        // results, so a descriptor announcing it may raise never registers.
        if self.cleanup == CleanupPolicy::MayRaise {
            return Err(DescriptorRejection::CleanupMayRaise);
        }
        Ok(())
    }
}

/// A registered payload and its deterministic-release state.
///
/// `IRIS-V1-FFI-C030` permits an object to have BOTH a payload descriptor for
/// memory safety and a `Closeable` Method for deterministic external release,
/// which is why close state lives beside the descriptor rather than replacing
/// it.
#[derive(Clone, Debug)]
pub struct NativePayload {
    descriptor: PayloadDescriptor,
    closed: bool,
    /// How many times the native close actually ran.
    ///
    /// `IRIS-V1-FFI-C030` requires close to be IDEMPOTENT, so a second call
    /// must succeed without releasing the resource twice. Counting the real
    /// releases is what distinguishes idempotent from merely tolerated.
    releases: u32,
}

impl NativePayload {
    /// Registers a validated descriptor.
    pub fn register(descriptor: PayloadDescriptor) -> Result<Self, DescriptorRejection> {
        descriptor.validate()?;
        Ok(Self {
            descriptor,
            closed: false,
            releases: 0,
        })
    }

    /// The registered descriptor.
    #[must_use]
    pub const fn descriptor(&self) -> &PayloadDescriptor {
        &self.descriptor
    }

    /// How many times the external resource was actually released.
    #[must_use]
    pub const fn releases(&self) -> u32 {
        self.releases
    }

    /// Closes the external resource, idempotently.
    ///
    /// `IRIS-V1-FFI-C030` makes deterministic release explicit and idempotent,
    /// so the second call answers success WITHOUT releasing again. It reports
    /// `Success` rather than a duplicate status because a double close is
    /// well-defined here, unlike a double completion under `C037`.
    pub fn close(&mut self) -> IrisStatus {
        if self.closed {
            return IrisStatus::Success;
        }
        self.closed = true;
        self.releases += 1;
        IrisStatus::Success
    }

    /// Declares one managed handle as a trace root.
    ///
    /// `IRIS-V1-FFI-C028` permits reporting managed handles, so a root is
    /// recorded as DATA rather than as a callback the collector would invoke.
    pub fn add_root(&mut self, handle: IrisHandle) {
        self.descriptor.trace.roots.push(handle);
    }

    /// The managed roots this payload reports to the collector.
    ///
    /// `IRIS-V1-FFI-C028` limits tracing to reporting managed handles, so this
    /// hands back declared roots and cannot reach Iris in any other way.
    #[must_use]
    pub fn trace(&self) -> &[IrisHandle] {
        &self.descriptor.trace.roots
    }

    /// Runs final cleanup in a GC-safe context.
    ///
    /// `IRIS-V1-FFI-C029` forbids final cleanup from raising into Iris, and
    /// makes a failure a runtime DIAGNOSTIC rather than a catchable result. A
    /// descriptor that may raise never registers, so reaching cleanup at all
    /// means it cannot raise, and this answers a status only.
    pub fn final_cleanup(&mut self) -> IrisStatus {
        // C030 lets final cleanup release as a LAST RESORT when the script
        // never closed the resource deterministically.
        if !self.closed && self.descriptor.external_resource {
            self.closed = true;
            self.releases += 1;
        }
        IrisStatus::Success
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CleanupPolicy, DescriptorRejection, NativePayload, PayloadDescriptor, TraceReport,
    };
    use crate::{IrisHandle, IrisStatus};

    fn descriptor() -> PayloadDescriptor {
        PayloadDescriptor {
            size: 8,
            alignment: 8,
            trace: TraceReport::default(),
            cleanup: CleanupPolicy::NoRaise,
            external_resource: true,
        }
    }

    #[test]
    fn c027_a_well_formed_descriptor_registers() {
        assert!(NativePayload::register(descriptor()).is_ok());
    }

    #[test]
    fn c027_alignment_must_be_a_power_of_two() {
        // Given a descriptor the runtime could not lay out
        let refused = PayloadDescriptor {
            alignment: 3,
            ..descriptor()
        };

        // When / Then
        assert_eq!(
            refused.validate(),
            Err(DescriptorRejection::AlignmentNotPowerOfTwo)
        );
        assert_eq!(
            DescriptorRejection::AlignmentNotPowerOfTwo.diagnostic(),
            "ffi.payload-alignment-invalid"
        );
    }

    #[test]
    fn c027_size_must_be_a_multiple_of_alignment() {
        // An array of these would place later elements off their alignment.
        let refused = PayloadDescriptor {
            size: 12,
            alignment: 8,
            ..descriptor()
        };
        assert_eq!(
            refused.validate(),
            Err(DescriptorRejection::SizeNotMultipleOfAlignment)
        );
    }

    #[test]
    fn c029_a_descriptor_whose_cleanup_may_raise_is_refused() {
        // C029 makes cleanup failures diagnostics rather than catchable
        // results, so this is refused at REGISTRATION instead of being
        // accepted and then contained when it drops.
        let refused = PayloadDescriptor {
            cleanup: CleanupPolicy::MayRaise,
            ..descriptor()
        };
        assert_eq!(
            refused.validate(),
            Err(DescriptorRejection::CleanupMayRaise)
        );
        assert_eq!(
            DescriptorRejection::CleanupMayRaise.diagnostic(),
            "ffi.payload-cleanup-may-raise"
        );
        assert!(NativePayload::register(refused).is_err());
    }

    #[test]
    fn c030_closing_twice_releases_exactly_once() {
        // Given a registered external resource
        let registered = NativePayload::register(descriptor());
        assert_eq!(registered.as_ref().err(), None);
        let Ok(mut payload) = registered else { return };

        // When close is called twice
        let first = payload.close();
        let second = payload.close();

        // Then both calls succeed and the resource released exactly once,
        // which is what makes close idempotent rather than merely tolerated.
        assert_eq!(first, IrisStatus::Success);
        assert_eq!(second, IrisStatus::Success);
        assert_eq!(payload.releases(), 1);
    }

    #[test]
    fn c030_final_cleanup_does_not_release_again_after_close() {
        // Given a resource already closed deterministically
        let registered = NativePayload::register(descriptor());
        assert_eq!(registered.as_ref().err(), None);
        let Ok(mut payload) = registered else { return };
        payload.close();

        // When the runtime later runs final cleanup
        let status = payload.final_cleanup();

        // Then it answers a status only and does not double-release.
        assert_eq!(status, IrisStatus::Success);
        assert_eq!(payload.releases(), 1);
    }

    #[test]
    fn c030_final_cleanup_releases_as_a_last_resort() {
        // Given a resource the script never closed
        let registered = NativePayload::register(descriptor());
        assert_eq!(registered.as_ref().err(), None);
        let Ok(mut payload) = registered else { return };

        // When final cleanup runs
        payload.final_cleanup();

        // Then it releases, because GC timing is not a resource promise but
        // leaking the OS handle entirely is not acceptable either.
        assert_eq!(payload.releases(), 1);
    }

    #[test]
    fn c028_trace_reports_only_declared_managed_roots() {
        // Given a payload holding one managed root
        let root = IrisHandle::pack(1, 4, 0);
        let registered = NativePayload::register(PayloadDescriptor {
            trace: TraceReport { roots: vec![root] },
            ..descriptor()
        });
        assert_eq!(registered.as_ref().err(), None);
        let Ok(payload) = registered else { return };

        // When the collector traces it. Trace is DATA rather than a callback,
        // so creating an Iris value or raising from here is unreachable by
        // construction rather than merely forbidden.
        assert_eq!(payload.trace(), [root]);
    }
}
