//! Process ownership model for AlohaOS user programs.

use core::ptr;

use crate::{address_space::{AddressSpace, MapError, USER_REGION_START}, memory};

pub const USER_CODE_BASE: u64 = USER_REGION_START;
pub const USER_STACK_TOP: u64 = USER_REGION_START + 0x20_0000;
const USER_STACK_PAGES: u64 = 4;
const KERNEL_STACK_PAGES: u64 = 8;
const MAX_BOOTSTRAP_IMAGE: usize = memory::FRAME_SIZE as usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessState { Ready, Running, Sleeping, Exited, Faulted }

pub struct Process {
    pub pid: u64,
    pub state: ProcessState,
    pub entry: u64,
    pub user_stack_top: u64,
    pub exit_code: i32,
    pub address_space: AddressSpace,
    code_frame: u64,
    kernel_stack_start: u64,
}

impl Process {
    pub fn new(pid: u64) -> Result<Self, MapError> {
        let mut address_space = AddressSpace::new()?;
        let code_frame = address_space.map_zeroed_user_page(USER_CODE_BASE, false, true)?;
        for page in 1..=USER_STACK_PAGES {
            address_space.map_zeroed_user_page(USER_STACK_TOP - page * memory::FRAME_SIZE, true, false)?;
        }
        let kernel_stack_start = memory::allocate_contiguous(KERNEL_STACK_PAGES)
            .ok_or(MapError::OutOfFrames)?;
        unsafe {
            ptr::write_bytes(
                kernel_stack_start as *mut u8,
                0,
                (KERNEL_STACK_PAGES * memory::FRAME_SIZE) as usize,
            );
        }
        Ok(Self {
            pid,
            state: ProcessState::Ready,
            entry: USER_CODE_BASE,
            user_stack_top: USER_STACK_TOP,
            exit_code: 0,
            address_space,
            code_frame,
            kernel_stack_start,
        })
    }

    pub fn kernel_stack_top(&self) -> u64 {
        self.kernel_stack_start + KERNEL_STACK_PAGES * memory::FRAME_SIZE
    }

    pub fn load_bootstrap_image(&mut self, image: &[u8]) -> bool {
        if image.is_empty() || image.len() > MAX_BOOTSTRAP_IMAGE { return false; }
        unsafe { ptr::copy_nonoverlapping(image.as_ptr(), self.code_frame as *mut u8, image.len()); }
        true
    }

    pub fn mark_running(&mut self) { self.state = ProcessState::Running; }
    pub fn exit(&mut self, code: i32) { self.exit_code = code; self.state = ProcessState::Exited; }
    pub fn fault(&mut self) { self.state = ProcessState::Faulted; }
}

impl Drop for Process {
    fn drop(&mut self) {
        unsafe {
            let _ = memory::deallocate_contiguous(self.kernel_stack_start, KERNEL_STACK_PAGES);
        }
    }
}
