//! Per-process file handle table backed by the read-only FAT32 volume.
//!
//! Each process owns a small, fixed-size table of open files. A handle stores
//! the resolved [`fat32::FileLocation`] plus a byte cursor that [`read`] both
//! consumes and advances, so sequential reads walk the file to EOF. The table
//! is allocation-free and never activates a user address space, so it is safe
//! to drive directly from the syscall dispatcher.
//!
//! [`read`]: HandleTable::read

use crate::fat32::{self, FileLocation};

/// Maximum number of files a single process may hold open at once. File
/// descriptors are indices into the table, so the valid range is `0..MAX_HANDLES`.
pub const MAX_HANDLES: usize = 16;

#[derive(Clone, Copy)]
struct Handle {
    location: FileLocation,
    offset: u32,
    used: bool,
}

impl Handle {
    const EMPTY: Self = Self {
        location: FileLocation::EMPTY,
        offset: 0,
        used: false,
    };
}

/// Why a handle operation could not be completed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandleError {
    /// No FAT32 volume is mounted, so files cannot be opened.
    NotMounted,
    /// The requested name did not resolve to a file in the volume.
    NotFound,
    /// The table has no free descriptor slot.
    TooManyOpen,
    /// The descriptor is out of range or refers to a closed handle.
    BadHandle,
}

/// A process's open-file table. Descriptors are slot indices; the lowest free
/// slot is always reused first so freed descriptors are handed back promptly.
pub struct HandleTable {
    handles: [Handle; MAX_HANDLES],
}

impl HandleTable {
    /// Create an empty table with every descriptor closed.
    pub const fn new() -> Self {
        Self {
            handles: [Handle::EMPTY; MAX_HANDLES],
        }
    }

    /// Resolve an 8.3 `name` to a file and allocate a descriptor for it.
    ///
    /// Returns the new descriptor, or an error if the volume is unmounted, the
    /// name does not resolve, or the table is full.
    pub fn open(&mut self, name: &[u8]) -> Result<usize, HandleError> {
        if !fat32::is_mounted() {
            return Err(HandleError::NotMounted);
        }
        let location = fat32::lookup(name).ok_or(HandleError::NotFound)?;
        let slot = self
            .handles
            .iter()
            .position(|handle| !handle.used)
            .ok_or(HandleError::TooManyOpen)?;
        self.handles[slot] = Handle {
            location,
            offset: 0,
            used: true,
        };
        Ok(slot)
    }

    /// Read from descriptor `fd` into `buffer`, advancing its cursor.
    ///
    /// Returns the number of bytes copied, which is `0` at EOF. The cursor only
    /// advances by the number of bytes actually read, so a short read at a
    /// cluster or disk boundary can be retried.
    pub fn read(&mut self, fd: usize, buffer: &mut [u8]) -> Result<usize, HandleError> {
        let handle = self
            .handles
            .get_mut(fd)
            .filter(|handle| handle.used)
            .ok_or(HandleError::BadHandle)?;
        let read = fat32::read_at(&handle.location, handle.offset, buffer);
        handle.offset = handle.offset.saturating_add(read as u32);
        Ok(read)
    }

    /// Release descriptor `fd`, returning its slot to the table.
    pub fn close(&mut self, fd: usize) -> Result<(), HandleError> {
        let handle = self
            .handles
            .get_mut(fd)
            .filter(|handle| handle.used)
            .ok_or(HandleError::BadHandle)?;
        *handle = Handle::EMPTY;
        Ok(())
    }
}
