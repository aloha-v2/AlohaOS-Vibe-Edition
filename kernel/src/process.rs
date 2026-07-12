//! Minimal process ownership model for the first AlohaOS user program.

use crate::{address_space::{AddressSpace, MapError, USER_REGION_START}, memory};

pub const USER_CODE_BASE: u64 = USER_REGION_START;
pub const USER_STACK_TOP: u64 = USER_REGION_START + 0x20_0000;
const USER_STACK_PAGES: u64 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessState {
    Ready,
    Running,
    Sleeping,
    Exited,
    Faulted,
}

pub struct Process {
    pub pid: u64,
    pub state: ProcessState,
    pub entry: u64,
    pub user_stack_top: u64,
    pub exit_code: i32,
    pub address_space: AddressSpace,
}

impl Process {
    pub fn new(pid: u64) -> Result<Self, MapError> {
        let mut address_space = AddressSpace::new()?;
        address_space.map_zeroed_user_page(USER_CODE_BASE, false, true)?;
        for page in 1..=USER_STACK_PAGES {
            address_space.map_zeroed_user_page(
                USER_STACK_TOP - page * memory::FRAME_SIZE,
                true,
                false,
            )?;
        }
        Ok(Self {
            pid,
            state: ProcessState::Ready,
            entry: USER_CODE_BASE,
            user_stack_top: USER_STACK_TOP,
            exit_code: 0,
            address_space,
        })
    }

    pub fn mark_running(&mut self) {
        self.state = ProcessState::Running;
    }

    pub fn exit(&mut self, code: i32) {
        self.exit_code = code;
        self.state = ProcessState::Exited;
    }

    pub fn fault(&mut self) {
        self.state = ProcessState::Faulted;
    }
}
