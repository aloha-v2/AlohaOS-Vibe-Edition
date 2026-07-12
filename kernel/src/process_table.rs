//! Bounded process registry with sleep deadlines and blocking wait notifications.

use crate::{process::ProcessState, sync::IrqSpinLock};

const MAX_PROCESSES: usize = 64;
const NO_PID: u64 = 0;
const NO_DEADLINE: u64 = u64::MAX;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessRecord {
    pub pid: u64,
    pub parent: u64,
    pub state: ProcessState,
    pub exit_code: i32,
    pub wake_tick: u64,
    pub waiting_for: u64,
    pub wake_pending: bool,
}

impl ProcessRecord {
    const EMPTY: Self = Self {
        pid: NO_PID,
        parent: NO_PID,
        state: ProcessState::Exited,
        exit_code: 0,
        wake_tick: NO_DEADLINE,
        waiting_for: NO_PID,
        wake_pending: false,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableError {
    Full,
    AlreadyExists,
    NoSuchProcess,
    NotChild,
    StillRunning,
    InvalidDeadline,
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
    table.records[slot] = new_record(pid, parent);
    Ok(pid)
}

pub fn register(pid: u64, parent: Option<u64>) -> Result<(), TableError> {
    if pid == NO_PID { return Err(TableError::NoSuchProcess); }
    let mut table = TABLE.lock();
    if table.records.iter().any(|record| record.pid == pid) {
        return Err(TableError::AlreadyExists);
    }
    validate_parent(&table, parent)?;
    let slot = table.empty_slot()?;
    table.records[slot] = new_record(pid, parent);
    if table.next_pid <= pid { table.next_pid = pid.saturating_add(1).max(1); }
    Ok(())
}

const fn new_record(pid: u64, parent: Option<u64>) -> ProcessRecord {
    ProcessRecord {
        pid,
        parent: match parent { Some(value) => value, None => NO_PID },
        state: ProcessState::Ready,
        exit_code: 0,
        wake_tick: NO_DEADLINE,
        waiting_for: NO_PID,
        wake_pending: false,
    }
}

fn validate_parent(table: &ProcessTable, parent: Option<u64>) -> Result<(), TableError> {
    if let Some(parent) = parent {
        if !table.records.iter().any(|record| record.pid == parent) {
            return Err(TableError::NoSuchProcess);
        }
    }
    Ok(())
}

fn find_mut(table: &mut ProcessTable, pid: u64) -> Result<&mut ProcessRecord, TableError> {
    table.records.iter_mut().find(|record| record.pid == pid).ok_or(TableError::NoSuchProcess)
}

pub fn set_state(pid: u64, state: ProcessState) -> Result<(), TableError> {
    let mut table = TABLE.lock();
    find_mut(&mut table, pid)?.state = state;
    Ok(())
}

pub fn sleep_until(pid: u64, now: u64, deadline: u64) -> Result<(), TableError> {
    if deadline <= now { return Err(TableError::InvalidDeadline); }
    let mut table = TABLE.lock();
    let record = find_mut(&mut table, pid)?;
    record.state = ProcessState::Sleeping;
    record.wake_tick = deadline;
    record.wake_pending = false;
    Ok(())
}

/// Advance monotonic process time and mark expired sleepers ready.
pub fn advance_time(now: u64) -> usize {
    let mut table = TABLE.lock();
    let mut woken = 0;
    for record in &mut table.records {
        if record.pid != NO_PID
            && record.state == ProcessState::Sleeping
            && record.wake_tick <= now
        {
            record.state = ProcessState::Ready;
            record.wake_tick = NO_DEADLINE;
            record.wake_pending = true;
            woken += 1;
        }
    }
    woken
}

/// Block a parent on one live child. Exit will publish a wake event.
pub fn wait_blocking(parent: u64, child: u64) -> Result<Option<i32>, TableError> {
    let mut table = TABLE.lock();
    let child_record = table
        .records
        .iter()
        .copied()
        .find(|record| record.pid == child)
        .ok_or(TableError::NoSuchProcess)?;
    if child_record.parent != parent { return Err(TableError::NotChild); }
    if child_record.state == ProcessState::Exited || child_record.state == ProcessState::Faulted {
        let status = child_record.exit_code;
        let record = find_mut(&mut table, child)?;
        *record = ProcessRecord::EMPTY;
        return Ok(Some(status));
    }
    let parent_record = find_mut(&mut table, parent)?;
    parent_record.state = ProcessState::Sleeping;
    parent_record.waiting_for = child;
    parent_record.wake_pending = false;
    Ok(None)
}

pub fn exit(pid: u64, code: i32) -> Result<(), TableError> {
    let mut table = TABLE.lock();
    {
        let record = find_mut(&mut table, pid)?;
        record.state = ProcessState::Exited;
        record.exit_code = code;
        record.wake_tick = NO_DEADLINE;
    }
    for waiter in &mut table.records {
        if waiter.pid != NO_PID && waiter.waiting_for == pid {
            waiter.state = ProcessState::Ready;
            waiter.waiting_for = NO_PID;
            waiter.wake_pending = true;
        }
    }
    Ok(())
}

pub fn fault(pid: u64, code: i32) -> Result<(), TableError> {
    exit(pid, code)?;
    let mut table = TABLE.lock();
    find_mut(&mut table, pid)?.state = ProcessState::Faulted;
    Ok(())
}

pub fn take_wake(pid: u64) -> Result<bool, TableError> {
    let mut table = TABLE.lock();
    let record = find_mut(&mut table, pid)?;
    let pending = record.wake_pending;
    record.wake_pending = false;
    Ok(pending)
}

pub fn lookup(pid: u64) -> Option<ProcessRecord> {
    TABLE.lock().records.iter().copied().find(|record| record.pid == pid)
}

pub fn wait(parent: u64, child: u64) -> Result<i32, TableError> {
    match wait_blocking(parent, child)? {
        Some(status) => Ok(status),
        None => Err(TableError::StillRunning),
    }
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
pub fn reset_for_smoke() { *TABLE.lock() = ProcessTable::EMPTY; }
