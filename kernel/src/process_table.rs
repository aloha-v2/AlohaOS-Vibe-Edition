//! Bounded process registry with PID allocation, parent/child and reap semantics.

use crate::{process::ProcessState, sync::IrqSpinLock};

const MAX_PROCESSES: usize = 64;
const NO_PID: u64 = 0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessRecord {
    pub pid: u64,
    pub parent: u64,
    pub state: ProcessState,
    pub exit_code: i32,
}

impl ProcessRecord {
    const EMPTY: Self = Self {
        pid: NO_PID,
        parent: NO_PID,
        state: ProcessState::Exited,
        exit_code: 0,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableError {
    Full,
    NoSuchProcess,
    NotChild,
    StillRunning,
}

struct ProcessTable {
    records: [ProcessRecord; MAX_PROCESSES],
    next_pid: u64,
}

impl ProcessTable {
    const EMPTY: Self = Self {
        records: [ProcessRecord::EMPTY; MAX_PROCESSES],
        next_pid: 1,
    };

    fn allocate_pid(&mut self) -> u64 {
        loop {
            let candidate = self.next_pid.max(1);
            self.next_pid = candidate.wrapping_add(1).max(1);
            if !self.records.iter().any(|record| record.pid == candidate) {
                return candidate;
            }
        }
    }
}

static TABLE: IrqSpinLock<ProcessTable> = IrqSpinLock::new(ProcessTable::EMPTY);

pub fn spawn(parent: Option<u64>) -> Result<u64, TableError> {
    let mut table = TABLE.lock();
    if let Some(parent) = parent {
        if !table.records.iter().any(|record| record.pid == parent) {
            return Err(TableError::NoSuchProcess);
        }
    }
    let slot = table
        .records
        .iter()
        .position(|record| record.pid == NO_PID)
        .ok_or(TableError::Full)?;
    let pid = table.allocate_pid();
    table.records[slot] = ProcessRecord {
        pid,
        parent: parent.unwrap_or(NO_PID),
        state: ProcessState::Ready,
        exit_code: 0,
    };
    Ok(pid)
}

pub fn set_state(pid: u64, state: ProcessState) -> Result<(), TableError> {
    let mut table = TABLE.lock();
    let record = table
        .records
        .iter_mut()
        .find(|record| record.pid == pid)
        .ok_or(TableError::NoSuchProcess)?;
    record.state = state;
    Ok(())
}

pub fn exit(pid: u64, code: i32) -> Result<(), TableError> {
    let mut table = TABLE.lock();
    let record = table
        .records
        .iter_mut()
        .find(|record| record.pid == pid)
        .ok_or(TableError::NoSuchProcess)?;
    record.state = ProcessState::Exited;
    record.exit_code = code;
    Ok(())
}

pub fn lookup(pid: u64) -> Option<ProcessRecord> {
    TABLE
        .lock()
        .records
        .iter()
        .copied()
        .find(|record| record.pid == pid)
}

/// Reap an exited child and return its status. Running children are retained.
pub fn wait(parent: u64, child: u64) -> Result<i32, TableError> {
    let mut table = TABLE.lock();
    let record = table
        .records
        .iter_mut()
        .find(|record| record.pid == child)
        .ok_or(TableError::NoSuchProcess)?;
    if record.parent != parent {
        return Err(TableError::NotChild);
    }
    if record.state != ProcessState::Exited && record.state != ProcessState::Faulted {
        return Err(TableError::StillRunning);
    }
    let status = record.exit_code;
    *record = ProcessRecord::EMPTY;
    Ok(status)
}

/// Reparent live children to the kernel (PID 0) when a parent exits.
pub fn orphan_children(parent: u64) -> usize {
    let mut table = TABLE.lock();
    let mut count = 0;
    for record in &mut table.records {
        if record.pid != NO_PID && record.parent == parent {
            record.parent = NO_PID;
            count += 1;
        }
    }
    count
}

#[cfg(feature = "m0-smoke")]
pub fn reset_for_smoke() {
    *TABLE.lock() = ProcessTable::EMPTY;
}
