//! End-to-end test for a real user shell ELF loaded from FAT32.
use crate::{process::ProcessState,serial,supervisor,syscall_entry};

pub fn run()->! {
    let shell=supervisor::spawn_path(0,b"SHELL.ELF").expect("user-shell-smoke: shell spawn failed");
    let mut parent=supervisor::take(shell).expect("user-shell-smoke: shell missing");
    serial::info(format_args!("user-shell-smoke: entering shell ELF pid={}",shell));
    crate::user_mode::run(&mut parent);
    assert_eq!(parent.state,ProcessState::Sleeping);
    supervisor::put(parent).expect("user-shell-smoke: parent requeue failed");
    supervisor::run_one_child(shell).expect("user-shell-smoke: child did not run");
    let mut parent=supervisor::take(shell).expect("user-shell-smoke: parent resume missing");
    assert_eq!(crate::user_mode::resume(&mut parent).unwrap(),Some(syscall_entry::ReturnPath::Sysret));
    assert_eq!(parent.state,ProcessState::Exited);
    assert_eq!(parent.exit_code,37);
    serial::info(format_args!("user-shell-smoke: real shell spawn/wait exit=37"));
    crate::halt()
}
