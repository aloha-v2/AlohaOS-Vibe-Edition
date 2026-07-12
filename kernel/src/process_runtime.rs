//! Bridge between PIT time, process registry wake events and owned Process state.

use crate::{process::{Process, ProcessState}, process_table};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TickReport {
    pub tick: u64,
    pub sleepers_woken: usize,
}

/// Advance process deadlines directly from the monotonic PIT tick.
pub fn on_timer_tick(tick: u64) -> TickReport {
    TickReport {
        tick,
        sleepers_woken: process_table::advance_time(tick),
    }
}

/// Reconcile an owned Process object after its registry record was woken.
/// Returns true exactly once for each published wake event.
pub fn reconcile(process: &mut Process) -> bool {
    let Ok(pending) = process_table::take_wake(process.pid) else {
        return false;
    };
    if !pending {
        return false;
    }
    if let Some(record) = process_table::lookup(process.pid) {
        process.state = record.state;
    }
    true
}

/// Complete a previously blocking wait after reconcile observed a child wake.
pub fn complete_wait(process: &mut Process, child: u64) -> Result<i32, process_table::TableError> {
    let status = process_table::wait(process.pid, child)?;
    process.state = ProcessState::Ready;
    process_table::set_state(process.pid, ProcessState::Ready)?;
    Ok(status)
}
