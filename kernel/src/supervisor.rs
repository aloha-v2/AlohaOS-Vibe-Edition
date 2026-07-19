//! Supervisor-owned child processes and atomic path-based spawn.

use alloc::vec::Vec;

use crate::{
    fat32,
    process::{Process, ProcessState},
    process_table,
    spawn::{self, SpawnError},
    sync::IrqSpinLock,
    user_mode,
};

const MAX_CHILDREN: usize = 32;
const MAX_IMAGE_SIZE: usize = 256 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SupervisorError {
    NotMounted,
    NoSuchFile,
    ImageTooLarge,
    ReadFailed,
    OutOfMemory,
    Capacity,
    NoSuchProcess,
    Spawn(SpawnError),
}

impl From<SpawnError> for SupervisorError {
    fn from(value: SpawnError) -> Self {
        Self::Spawn(value)
    }
}

static CHILDREN: IrqSpinLock<Vec<Process>> = IrqSpinLock::new(Vec::new());

/// Validate a FAT32 path, load its ELF and transfer the resulting process to
/// the supervisor. Any failure after PID reservation removes the registry
/// record and drops the address space, so callers never observe a partial child.
pub fn spawn_path(parent: u64, path: &[u8]) -> Result<u64, SupervisorError> {
    if !fat32::is_mounted() {
        return Err(SupervisorError::NotMounted);
    }
    let location = fat32::lookup(path).ok_or(SupervisorError::NoSuchFile)?;
    let image_size = location.size as usize;
    if image_size == 0 || image_size > MAX_IMAGE_SIZE {
        return Err(SupervisorError::ImageTooLarge);
    }

    {
        let mut children = CHILDREN.lock();
        if children.len() >= MAX_CHILDREN {
            return Err(SupervisorError::Capacity);
        }
        children
            .try_reserve(1)
            .map_err(|_| SupervisorError::OutOfMemory)?;
    }

    let mut image = Vec::new();
    image
        .try_reserve_exact(image_size)
        .map_err(|_| SupervisorError::OutOfMemory)?;
    image.resize(image_size, 0);
    if fat32::read_at(&location, 0, &mut image) != image_size {
        return Err(SupervisorError::ReadFailed);
    }

    let child = spawn::spawn_elf(Some(parent), &image)?;
    let pid = child.pid;
    let mut children = CHILDREN.lock();
    if children.len() >= MAX_CHILDREN {
        drop(children);
        let _ = process_table::remove(pid);
        drop(child);
        return Err(SupervisorError::Capacity);
    }
    children.push(child);
    Ok(pid)
}

/// Temporarily transfer a child out of supervisor storage for execution.
pub fn take(pid: u64) -> Option<Process> {
    let mut children = CHILDREN.lock();
    let index = children.iter().position(|process| process.pid == pid)?;
    Some(children.swap_remove(index))
}

/// Return a live child to supervisor storage.
pub fn put(process: Process) -> Result<(), SupervisorError> {
    let mut children = CHILDREN.lock();
    if children.len() >= MAX_CHILDREN {
        return Err(SupervisorError::Capacity);
    }
    children
        .try_reserve(1)
        .map_err(|_| SupervisorError::OutOfMemory)?;
    children.push(process);
    Ok(())
}

/// Run one owned child. Sleeping children remain supervised; terminal children
/// release their address spaces while their process-table record stays available
/// for the parent's `wait` call.
pub fn run(pid: u64) -> Result<ProcessState, SupervisorError> {
    let mut process = take(pid).ok_or(SupervisorError::NoSuchProcess)?;
    let _ = user_mode::run(&mut process);
    let state = process.state;
    if matches!(state, ProcessState::Ready | ProcessState::Running | ProcessState::Sleeping) {
        put(process)?;
    }
    Ok(state)
}
