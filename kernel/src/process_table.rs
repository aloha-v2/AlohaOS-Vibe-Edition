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
    AlreadyExists,
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

    fn empty_slot(&self) -> Result<usize, TableError> {
        self.records
            .iter()
            .position(|record| record.pid == NO_PID)
            .ok_or(TableError::Full)
    }
}

static TABLE: IrqSpinLock<ProcessTable> = IrqSpinLock::new(ProcessTable::EMPTY);

pub fn spawn(parent: Option<u64>) -> Result<u64, TableError> {
    let mut table = TABLE.lock();
    validate_parent(&table, parent)?;
    let slot = table.empty_slot()?;
    let pid = table.allocate_pid();
    table.records[slot] = ProcessRecord {
        pid,
        parent: parent.unwrap_or(NO_PID),
        state: ProcessState::Ready,
        exit_code: 0,
    };
    Ok(pid)
}

/// Register an already-created Process object in the metadata table.
pub fn register(pid: u64, parent: Option<u64>) -> Result<(), TableError> {
    if pid == NO_PID {
        return Err(TableError::NoSuchProcess);
    }
    let mut table = TABLE.lock();
    if table.records.iter().any(|record| record.pid == pid) {
        return Err(TableError::AlreadyExists);
    }
    validate_parent(&table, parent)?;
    let slot = table.empty_slot()?;
    table.records[slot] = ProcessRecord {
        pid,
        parent: parent.unwrap_or(NO_PID),
        state: ProcessState::Ready,
        exit_code: 0,
    };
    if table.next_pid <= pid {
        table.next_pid = pid.saturating_add(1).max(1);
    }
    Ok(())
}

fn validate_parent(table: &ProcessTable, parent: Option<u64>) -> Result<(), TableError> {
    if let Some(parent) = parent {
        if !table.records.iter().any(|record| record.pid == parent) {
            return Err(TableError::NoSuchProcess);
        }
    }
    Ok(())
}

pub fn set_state(pid: u64, state: ProcessState) -> Result<(), TableError> {
    let mut table = TABLE.lock();
    let record = find_mut(&mut table, pid)?;
    record.state = state;
    Ok(())
}

pub fn exit(pid: u64, code: i32) -> Result<(), TableError> {
    let mut table = TABLE.lock();
    let record = find_mut(&mut table, pid)?;
    record.state = ProcessState::Exited;
    record.exit_code = code;
    Ok(())
}

pub fn fault(pid: u64, code: i32) -> Result<(), TableError> {
    let mut table = TABLE.lock();
    let record = find_mut(&mut table, pid)?;
    record.state = ProcessState::Faulted;
    record.exit_code = code;
    Ok(())
}

fn find_mut(table: &mut ProcessTable, pid: u64) -> Result<&mut ProcessRecord, TableError> {
    table
        .records
        .iter_mut()
        .find(|record| record.pid == pid)
        .ok_or(TableError::NoSuchProcess)
}

pub fn lookup(pid: u64) -> Option<ProcessRecord> {
    TABLE
        .lock()
        .records
        .iter()
        .copied()
        .find(|record| record.pid == pid)
}

pub fn wait(parent: u64, child: u64) -> Result<i32, TableError> {
    let mut table = TABLE.lock();
    let record = find_mut(&mut table, child)?;
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
