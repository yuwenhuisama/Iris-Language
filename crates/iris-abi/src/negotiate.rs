//! `IRIS-V1-FFI-C038` versioned negotiation.

use crate::IrisStatus;

/// The ABI major this runtime implements.
///
/// `IRIS-V1-FFI-C039` makes a major mismatch reject load or attachment.
pub const ABI_MAJOR: u32 = 1;

/// The ABI minor this runtime implements.
pub const ABI_MINOR: u32 = 0;

/// The negotiated function table handed to an extension.
///
/// `IRIS-V1-FFI-C038` requires size-tagged records so a minor version may
/// APPEND fields, and `IRIS-V1-FFI-C039` forbids a participant from reading a
/// field the supplied size does not cover. `size` is therefore the first field
/// and every reader checks it before touching anything added later.
///
/// `IRIS-V1-FFI-C042` prohibits promising a Rust or C++ object layout as the
/// binary contract, which is why this is `#[repr(C)]` and holds only integers
/// and function pointers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct IrisAbiTable {
    /// Byte size of the table the producer filled in.
    pub size: u32,
    /// Negotiated major version.
    pub abi_major: u32,
    /// Negotiated minor version.
    pub abi_minor: u32,
    /// Feature bits both participants agreed on.
    pub features: u32,
}

impl IrisAbiTable {
    /// Whether the supplied size covers a field ending at `offset`.
    ///
    /// `IRIS-V1-FFI-C039` forbids reinterpreting an older record as a newer
    /// larger layout unless the size covers the field being read.
    #[must_use]
    pub const fn covers(&self, offset: u32) -> bool {
        self.size >= offset
    }
}

/// Negotiates an extension attachment.
///
/// `IRIS-V1-FFI-C038` makes every participant declare its required major and
/// minimum minor BEFORE receiving authority to create or observe Iris values,
/// so this runs before any handle exists. `IRIS-V1-FFI-C039` rejects a major
/// mismatch outright while a newer runtime minor stays compatible.
pub fn attach(requested_major: u32, minimum_minor: u32) -> Result<IrisAbiTable, IrisStatus> {
    if requested_major != ABI_MAJOR || minimum_minor > ABI_MINOR {
        return Err(IrisStatus::IncompatibleAbi);
    }
    Ok(IrisAbiTable {
        size: u32::try_from(core::mem::size_of::<IrisAbiTable>()).unwrap_or(u32::MAX),
        abi_major: ABI_MAJOR,
        abi_minor: ABI_MINOR,
        features: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::{ABI_MAJOR, ABI_MINOR, IrisAbiTable, attach};
    use crate::IrisStatus;

    #[test]
    fn c038_attachment_reports_the_negotiated_versions() {
        // When
        let Ok(table) = attach(ABI_MAJOR, 0) else {
            unreachable!("a compatible request attaches")
        };

        // Then
        assert_eq!(table.abi_major, ABI_MAJOR);
        assert_eq!(table.abi_minor, ABI_MINOR);
        assert!(table.covers(u32::try_from(size_of::<IrisAbiTable>()).unwrap_or(u32::MAX)));
    }

    #[test]
    fn c039_a_major_mismatch_rejects_attachment() {
        // When / Then
        assert_eq!(attach(2, 0), Err(IrisStatus::IncompatibleAbi));
        assert_eq!(attach(0, 0), Err(IrisStatus::IncompatibleAbi));
    }

    #[test]
    fn c039_a_minor_newer_than_the_runtime_is_rejected() {
        // A participant needing a minor this runtime does not implement cannot
        // be satisfied, while an older minimum stays compatible.
        assert_eq!(
            attach(ABI_MAJOR, ABI_MINOR + 1),
            Err(IrisStatus::IncompatibleAbi)
        );
        assert!(attach(ABI_MAJOR, 0).is_ok());
    }

    #[test]
    fn c039_a_short_record_does_not_cover_a_later_field() {
        // Given a table an older producer filled in
        let older = IrisAbiTable {
            size: 8,
            abi_major: ABI_MAJOR,
            abi_minor: 0,
            features: 0,
        };

        // Then reading past the supplied size is refused rather than guessed.
        assert!(older.covers(8));
        assert!(!older.covers(16));
    }
}
