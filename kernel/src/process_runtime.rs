//! Bridge between PIT wake events, saved syscall frames and owned Process state.

use crate::{
    process::{Process, ProcessState, SuspendedCall},
    process_table,
    syscall_entry::SyscallFrame,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TickReport { pub tick: u64, pub sleepers_woken: usize }

pub fn on_timer_tick(tick: u64) -> TickReport { TickReport { tick, sleepers_woken: process_table::advance_time(tick) } }

pub fn reconcile(process: &mut Process) -> bool {
    let Ok(pending) = process_table::take_wake(process.pid) else { return false; };
    if !pending { return false; }
    if let Some(record) = process_table::lookup(process.pid) { process.state = record.state; }
    true
}

pub fn complete_wait(process: &mut Process, child: u64) -> Result<i32, process_table::TableError> {
    let status = process_table::wait(process.pid, child)?;
    process.state = ProcessState::Ready;
    process_table::set_state(process.pid, ProcessState::Ready)?;
    Ok(status)
}

pub fn resume_frame(process: &mut Process) -> Result<Option<SyscallFrame>, process_table::TableError> {
    if !reconcile(process) { return Ok(None); }
    let Some((mut frame, call)) = process.take_suspended_syscall() else { return Ok(None); };
    frame.result = match call {
        SuspendedCall::Sleep => 0,
        SuspendedCall::Wait { child } => complete_wait(process, child)? as u32 as u64,
    };
    process.state = ProcessState::Ready;
    process_table::set_state(process.pid, ProcessState::Ready)?;
    Ok(Some(frame))
}
