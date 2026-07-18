//! Owned user-process creation with all-or-nothing rollback.
//!
//! [`spawn_elf`] ties together the three resources a live process needs:
//! a registry record (PID + parent link), a private address space with a
//! kernel stack, and a validated ELF image loaded into that address space.
//! The stages acquire resources in order:
//!
//! 1. reserve a PID + record in the process registry,
//! 2. build the [`Process`] (address space, user pages, kernel stack),
//! 3. load and map the ELF image.
//!
//! If any stage fails, every resource acquired so far is released before
//! returning. The registry record is removed by the [`Reservation`] guard on
//! drop, and the partially built [`Process`] frees its frames through its own
//! `Drop`, so a failed spawn leaks neither PIDs nor physical frames.

use crate::{
    address_space::MapError,
    process::{LoadError, Process},
    process_table::{self, TableError},
};

/// Reason a spawn could not be completed. Each variant names the stage that
/// rejected the request; by the time the error is observed every resource the
/// spawn had acquired has already been rolled back.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpawnError {
    /// The process registry rejected the reservation (e.g. table full).
    Registry(TableError),
    /// The address space or kernel stack could not be allocated.
    AddressSpace(MapError),
    /// The ELF image failed validation or could not be mapped.
    Load(LoadError),
}

impl From<TableError> for SpawnError {
    fn from(value: TableError) -> Self {
        Self::Registry(value)
    }
}

/// RAII reservation of a registry slot.
///
/// Holding a reservation keeps a PID + record allocated in the process table.
/// Dropping it *without committing* removes that record, which turns every
/// early return in [`spawn_elf`] into a clean registry rollback.
struct Reservation {
    pid: u64,
    committed: bool,
}

impl Reservation {
    fn acquire(parent: Option<u64>) -> Result<Self, TableError> {
        Ok(Self {
            pid: process_table::spawn(parent)?,
            committed: false,
        })
    }

    /// Keep the registry record and yield the owned PID. After this the guard
    /// no longer rolls the record back on drop.
    fn commit(mut self) -> u64 {
        self.committed = true;
        self.pid
    }
}

impl Drop for Reservation {
    fn drop(&mut self) {
        if !self.committed {
            let _ = process_table::remove(self.pid);
        }
    }
}

/// Create a ready-to-run process from `image`, owned by optional `parent`.
///
/// On success the returned [`Process`] owns a registry record in the `Ready`
/// state plus a private address space with `image` loaded. On failure every
/// resource acquired so far is released and the matching [`SpawnError`] stage
/// is returned.
pub fn spawn_elf(parent: Option<u64>, image: &[u8]) -> Result<Process, SpawnError> {
    let reservation = Reservation::acquire(parent)?;
    let mut process = match Process::new(reservation.pid) {
        Ok(process) => process,
        // `reservation` drops here and removes the registry record.
        Err(error) => return Err(SpawnError::AddressSpace(error)),
    };
    if let Err(error) = process.load_elf(image) {
        // `process` drops here (freeing its frames) and so does `reservation`.
        return Err(SpawnError::Load(error));
    }
    // All stages succeeded: retain the registry record and hand over ownership.
    let _pid = reservation.commit();
    Ok(process)
}
