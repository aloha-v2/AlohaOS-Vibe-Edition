//! Small user-space command engine backed only by `aloha-user` syscalls.
#![no_std]

use aloha_user::{self as user, Errno};

const COMMAND_MAX: usize = 64;
const IO_MAX: usize = 256;

pub fn execute(command: &[u8]) -> i32 {
    let command = trim(command);
    if command == b"ls" || command == b"ls /" {
        return ls();
    }
    if command == b"help" {
        print(b"ls cat stat spawn wait help\n");
        return 0;
    }
    if command.starts_with(b"cat ") {
        return cat(trim(&command[4..]));
    }
    if command.starts_with(b"stat ") {
        return stat(trim(&command[5..]));
    }
    if command.starts_with(b"spawn ") {
        return spawn_and_wait(trim(&command[6..]));
    }
    print(b"unknown command\n");
    1
}

pub fn command_buffer() -> [u8; COMMAND_MAX] {
    [0; COMMAND_MAX]
}

fn ls() -> i32 {
    let mut output = [0u8; IO_MAX];
    match user::list(&mut output) {
        Ok(length) => { print(&output[..length]); 0 }
        Err(error) => { print_error(error); 1 }
    }
}

fn cat(path: &[u8]) -> i32 {
    let handle = match user::open(path) {
        Ok(handle) => handle,
        Err(error) => { print_error(error); return 1; }
    };
    let mut buffer = [0u8; IO_MAX];
    let mut status = 0;
    loop {
        match user::read(handle, &mut buffer) {
            Ok(0) => break,
            Ok(length) => print(&buffer[..length]),
            Err(error) => { print_error(error); status = 1; break; }
        }
    }
    if user::close(handle).is_err() { status = 1; }
    status
}

fn stat(path: &[u8]) -> i32 {
    match user::stat(path) {
        Ok(size) => { print_decimal(size); print(b"\n"); 0 }
        Err(error) => { print_error(error); 1 }
    }
}

fn spawn_and_wait(path: &[u8]) -> i32 {
    let pid = match user::spawn(path) {
        Ok(pid) => pid,
        Err(error) => { print_error(error); return 1; }
    };
    print(b"spawned ");
    print_decimal(pid);
    print(b"\n");
    match user::wait(pid) {
        Ok(status) => { print(b"exit "); print_decimal(status as u32 as u64); print(b"\n"); status }
        Err(error) => { print_error(error); 1 }
    }
}

fn print(bytes: &[u8]) {
    let _ = user::write(bytes);
}

fn print_decimal(mut value: u64) {
    let mut digits = [0u8; 20];
    let mut index = digits.len();
    if value == 0 { print(b"0"); return; }
    while value != 0 {
        index -= 1;
        digits[index] = b'0' + (value % 10) as u8;
        value /= 10;
    }
    print(&digits[index..]);
}

fn print_error(error: Errno) {
    print(b"error ");
    print(match error {
        Errno::NoSuchFile => b"no such file\n",
        Errno::BadAddress => b"bad address\n",
        Errno::Busy => b"busy\n",
        Errno::NotSupported => b"not supported\n",
        Errno::OutOfMemory => b"out of memory\n",
        _ => b"invalid\n",
    });
}

fn trim(mut value: &[u8]) -> &[u8] {
    while value.first() == Some(&b' ') { value = &value[1..]; }
    while value.last() == Some(&b' ') { value = &value[..value.len() - 1]; }
    value
}
