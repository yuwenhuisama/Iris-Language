//! The runtime-owned handle table.

use crate::{IrisFrame, IrisHandle, IrisHandleKind, IrisStatus};

/// One occupied or free slot.
#[derive(Clone, Debug)]
struct Slot<T> {
    /// The rooted target, or `None` when the slot is free.
    ///
    /// `IRIS-V1-FFI-C008` makes a live handle STRONGLY ROOT its target, so the
    /// table owning the value is what keeps it reachable.
    target: Option<T>,
    /// Bumped on every release so a stale handle fails the generation check.
    generation: u16,
    /// The frame owning this slot, for a scoped handle.
    frame: Option<u64>,
}

/// A per-runtime table of rooted handles.
///
/// `IRIS-V1-FFI-C009` binds handles to one runtime, so the table carries the
/// runtime tag and rejects a handle stamped with a different one.
#[derive(Debug)]
pub struct HandleTable<T> {
    runtime: u16,
    slots: Vec<Slot<T>>,
    free: Vec<u32>,
    /// Open frames, innermost last.
    frames: Vec<u64>,
    next_frame: u64,
    /// Set when the runtime is destroyed.
    ///
    /// `IRIS-V1-FFI-C009` invalidates every handle on destruction and makes
    /// later use fail with a closed or invalid-runtime status.
    closed: bool,
}

impl<T> HandleTable<T> {
    /// Creates the table for one runtime.
    #[must_use]
    pub const fn new(runtime: u16) -> Self {
        Self {
            runtime,
            slots: Vec::new(),
            free: Vec::new(),
            frames: Vec::new(),
            next_frame: 1,
            closed: false,
        }
    }

    /// Roots `target` and answers its handle.
    pub fn retain(&mut self, target: T, kind: IrisHandleKind) -> IrisHandle {
        let frame = match kind {
            IrisHandleKind::Scoped => self.frames.last().copied(),
            IrisHandleKind::ExplicitRelease => None,
        };
        if let Some(slot) = self.free.pop() {
            let index = slot as usize;
            self.slots[index].target = Some(target);
            self.slots[index].frame = frame;
            return IrisHandle::pack(self.runtime, slot, self.slots[index].generation);
        }
        let slot = u32::try_from(self.slots.len()).unwrap_or(u32::MAX);
        self.slots.push(Slot {
            target: Some(target),
            generation: 0,
            frame,
        });
        IrisHandle::pack(self.runtime, slot, 0)
    }

    /// Reads the target a handle denotes.
    ///
    /// Answers `InvalidRuntime` for a foreign or destroyed runtime and
    /// `InvalidHandle` for a released, reused or never-issued slot, which is
    /// how `IRIS-V1-FFI-C009` keeps a stale handle from denoting a new target.
    pub fn get(&self, handle: IrisHandle) -> Result<&T, IrisStatus> {
        self.slot_of(handle).and_then(|slot| {
            self.slots[slot]
                .target
                .as_ref()
                .ok_or(IrisStatus::InvalidHandle)
        })
    }

    /// Releases a handle's root.
    pub fn release(&mut self, handle: IrisHandle) -> IrisStatus {
        match self.slot_of(handle) {
            Err(status) => status,
            Ok(slot) => {
                if self.slots[slot].target.take().is_none() {
                    return IrisStatus::InvalidHandle;
                }
                // C009 permits slot reuse, so the generation moves and every
                // handle already issued for this slot becomes detectably stale.
                self.slots[slot].generation = self.slots[slot].generation.wrapping_add(1);
                self.slots[slot].frame = None;
                self.free.push(handle.slot());
                IrisStatus::Success
            }
        }
    }

    /// Opens a handle frame.
    pub fn open_frame(&mut self) -> IrisFrame {
        let frame = self.next_frame;
        self.next_frame += 1;
        self.frames.push(frame);
        IrisFrame(frame)
    }

    /// Closes a frame, bulk-releasing every live handle it owns.
    pub fn close_frame(&mut self, frame: IrisFrame) -> IrisStatus {
        if self.closed {
            return IrisStatus::InvalidRuntime;
        }
        let Some(position) = self.frames.iter().rposition(|open| *open == frame.0) else {
            return IrisStatus::InvalidArgument;
        };
        self.frames.remove(position);
        for slot in 0..self.slots.len() {
            if self.slots[slot].frame == Some(frame.0) && self.slots[slot].target.take().is_some() {
                self.slots[slot].generation = self.slots[slot].generation.wrapping_add(1);
                self.slots[slot].frame = None;
                if let Ok(slot) = u32::try_from(slot) {
                    self.free.push(slot);
                }
            }
        }
        IrisStatus::Success
    }

    /// Destroys the runtime, invalidating every handle it issued.
    pub fn close(&mut self) {
        self.closed = true;
        self.slots.clear();
        self.free.clear();
        self.frames.clear();
    }

    /// Every rooted target, for the collector to trace.
    ///
    /// `IRIS-V1-FFI-C008` makes a live handle a strong root, so these are GC
    /// roots and `IRIS-V1-FFI-C036` limits native tracing to exactly them.
    pub fn roots(&self) -> impl Iterator<Item = &T> {
        self.slots.iter().filter_map(|slot| slot.target.as_ref())
    }

    fn slot_of(&self, handle: IrisHandle) -> Result<usize, IrisStatus> {
        if self.closed || handle.runtime() != self.runtime {
            return Err(IrisStatus::InvalidRuntime);
        }
        if handle.is_null() {
            return Err(IrisStatus::InvalidHandle);
        }
        let slot = handle.slot() as usize;
        let held = self.slots.get(slot).ok_or(IrisStatus::InvalidHandle)?;
        if held.generation != handle.generation() {
            return Err(IrisStatus::InvalidHandle);
        }
        Ok(slot)
    }
}

#[cfg(test)]
mod tests {
    use super::HandleTable;
    use crate::{IrisFrame, IrisHandle, IrisHandleKind, IrisStatus};

    #[test]
    fn c008_a_live_handle_roots_its_target_and_release_ends_the_root() {
        // Given
        let mut table = HandleTable::new(1);

        // When
        let handle = table.retain(41_i64, IrisHandleKind::ExplicitRelease);

        // Then
        assert_eq!(table.get(handle), Ok(&41));
        assert_eq!(table.roots().count(), 1);
        assert_eq!(table.release(handle), IrisStatus::Success);
        assert_eq!(table.roots().count(), 0);
    }

    #[test]
    fn c009_a_released_handle_does_not_denote_the_slot_after_reuse() {
        // Given
        let mut table = HandleTable::new(1);
        let stale = table.retain(41_i64, IrisHandleKind::ExplicitRelease);
        assert_eq!(table.release(stale), IrisStatus::Success);

        // When the slot is reused, C009 forbids the old handle from denoting
        // the new occupant, which is what the generation counter enforces.
        let fresh = table.retain(7_i64, IrisHandleKind::ExplicitRelease);

        // Then
        assert_eq!(fresh.slot(), stale.slot());
        assert_eq!(table.get(fresh), Ok(&7));
        assert_eq!(table.get(stale), Err(IrisStatus::InvalidHandle));
    }

    #[test]
    fn c009_a_handle_from_another_runtime_is_rejected() {
        // Given
        let mut first = HandleTable::new(1);
        let mut second = HandleTable::<i64>::new(2);
        let handle = first.retain(41, IrisHandleKind::ExplicitRelease);

        // When / Then
        assert_eq!(second.get(handle), Err(IrisStatus::InvalidRuntime));
        assert_eq!(second.release(handle), IrisStatus::InvalidRuntime);
    }

    #[test]
    fn c009_runtime_destruction_invalidates_every_handle() {
        // Given
        let mut table = HandleTable::new(1);
        let handle = table.retain(41_i64, IrisHandleKind::ExplicitRelease);

        // When
        table.close();

        // Then
        assert_eq!(table.get(handle), Err(IrisStatus::InvalidRuntime));
    }

    #[test]
    fn c008_closing_a_frame_bulk_releases_its_scoped_handles() {
        // Given
        let mut table = HandleTable::new(1);
        let outer = table.retain(1_i64, IrisHandleKind::ExplicitRelease);
        let frame = table.open_frame();
        let scoped = table.retain(2_i64, IrisHandleKind::Scoped);

        // When
        assert_eq!(table.close_frame(frame), IrisStatus::Success);

        // Then an explicit-release handle outlives the frame, and the scoped
        // one does not.
        assert_eq!(table.get(scoped), Err(IrisStatus::InvalidHandle));
        assert_eq!(table.get(outer), Ok(&1));
    }

    #[test]
    fn c007_a_null_handle_names_nothing() {
        // Given
        let table = HandleTable::<i64>::new(1);

        // When / Then
        assert_eq!(table.get(IrisHandle::NULL), Err(IrisStatus::InvalidRuntime));
        assert!(IrisHandle::NULL.is_null());
    }

    #[test]
    fn c008_closing_an_unopened_frame_is_an_argument_error() {
        // Given
        let mut table = HandleTable::<i64>::new(1);

        // When / Then
        assert_eq!(table.close_frame(IrisFrame(9)), IrisStatus::InvalidArgument);
    }
}
