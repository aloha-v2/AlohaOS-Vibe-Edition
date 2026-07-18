//! Process ownership, suspended syscall state and ELF loading.

use core::ptr;
use crate::{address_space::{AddressSpace, MapError, USER_REGION_START}, elf::{self, ElfError}, memory, syscall_entry::SyscallFrame};

pub const USER_CODE_BASE: u64 = USER_REGION_START;
pub const USER_STACK_TOP: u64 = USER_REGION_START + 0x20_0000;
const USER_STACK_PAGES: u64 = 4;
const KERNEL_STACK_PAGES: u64 = 8;
const MAX_BOOTSTRAP_IMAGE: usize = memory::FRAME_SIZE as usize;
const WRITABLE_FLAG: u64 = 1 << 1;
const NO_EXECUTE_FLAG: u64 = 1 << 63;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessState { Ready, Running, Sleeping, Exited, Faulted }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuspendedCall { Sleep, Wait { child: u64 } }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadError { Elf(ElfError), Map(MapError), IncompatiblePage, Arithmetic }
impl From<ElfError> for LoadError { fn from(value: ElfError) -> Self { Self::Elf(value) } }
impl From<MapError> for LoadError { fn from(value: MapError) -> Self { Self::Map(value) } }

pub struct Process { pub pid:u64,pub state:ProcessState,pub entry:u64,pub user_stack_top:u64,pub exit_code:i32,pub address_space:AddressSpace,code_frame:u64,kernel_stack_start:u64,suspended:Option<(SyscallFrame,SuspendedCall)> }
impl Process {
 pub fn new(pid:u64)->Result<Self,MapError>{let mut address_space=AddressSpace::new()?;let code_frame=address_space.map_zeroed_user_page(USER_CODE_BASE,false,true)?;for page in 1..=USER_STACK_PAGES{address_space.map_zeroed_user_page(USER_STACK_TOP-page*memory::FRAME_SIZE,true,false)?;}let kernel_stack_start=memory::allocate_contiguous(KERNEL_STACK_PAGES).ok_or(MapError::OutOfFrames)?;unsafe{ptr::write_bytes(kernel_stack_start as*mut u8,0,(KERNEL_STACK_PAGES*memory::FRAME_SIZE)as usize);}Ok(Self{pid,state:ProcessState::Ready,entry:USER_CODE_BASE,user_stack_top:USER_STACK_TOP,exit_code:0,address_space,code_frame,kernel_stack_start,suspended:None})}
 pub fn kernel_stack_top(&self)->u64{self.kernel_stack_start+KERNEL_STACK_PAGES*memory::FRAME_SIZE}
 pub fn load_bootstrap_image(&mut self,image:&[u8])->bool{if image.is_empty()||image.len()>MAX_BOOTSTRAP_IMAGE{return false}unsafe{ptr::copy_nonoverlapping(image.as_ptr(),self.code_frame as*mut u8,image.len());}true}
 pub fn suspend_syscall(&mut self,frame:SyscallFrame,call:SuspendedCall){self.state=ProcessState::Sleeping;self.suspended=Some((frame,call));}
 pub fn take_suspended_syscall(&mut self)->Option<(SyscallFrame,SuspendedCall)>{self.suspended.take()}
 #[cfg(feature="user-resume-smoke")]pub fn suspended_frame_mut(&mut self)->Option<&mut SyscallFrame>{self.suspended.as_mut().map(|saved|&mut saved.0)}
 pub fn load_elf(&mut self,image:&[u8])->Result<(),LoadError>{let plan=elf::validate(image)?;for segment in plan.segments(){let segment_end=segment.virtual_address.checked_add(segment.memory_size).ok_or(LoadError::Arithmetic)?;let mut page=segment.virtual_address&!(memory::FRAME_SIZE-1);let last=(segment_end-1)&!(memory::FRAME_SIZE-1);loop{if let Some((_,flags))=self.address_space.translate(page){let writable=flags&WRITABLE_FLAG!=0;let executable=flags&NO_EXECUTE_FLAG==0;if writable!=segment.writable||executable!=segment.executable{return Err(LoadError::IncompatiblePage)}}else{self.address_space.map_zeroed_user_page(page,segment.writable,segment.executable)?}if page==last{break}page=page.checked_add(memory::FRAME_SIZE).ok_or(LoadError::Arithmetic)?;}let file_start=usize::try_from(segment.file_offset).map_err(|_|LoadError::Arithmetic)?;let file_size=usize::try_from(segment.file_size).map_err(|_|LoadError::Arithmetic)?;let file_end=file_start.checked_add(file_size).ok_or(LoadError::Arithmetic)?;self.initialize_user_memory(segment.virtual_address,&image[file_start..file_end])?;self.zero_user_memory(segment.virtual_address.checked_add(segment.file_size).ok_or(LoadError::Arithmetic)?,segment.memory_size-segment.file_size)?;}self.entry=plan.entry;Ok(())}
 fn initialize_user_memory(&self,address:u64,source:&[u8])->Result<(),LoadError>{let mut copied=0usize;while copied<source.len(){let virtual_address=address.checked_add(copied as u64).ok_or(LoadError::Arithmetic)?;let(physical,_)=self.address_space.translate(virtual_address).ok_or(LoadError::IncompatiblePage)?;let count=((memory::FRAME_SIZE-physical%memory::FRAME_SIZE)as usize).min(source.len()-copied);unsafe{ptr::copy_nonoverlapping(source[copied..].as_ptr(),physical as*mut u8,count);}copied+=count;}Ok(())}
 fn zero_user_memory(&self,address:u64,length:u64)->Result<(),LoadError>{let mut zeroed=0u64;while zeroed<length{let virtual_address=address.checked_add(zeroed).ok_or(LoadError::Arithmetic)?;let(physical,_)=self.address_space.translate(virtual_address).ok_or(LoadError::IncompatiblePage)?;let count=(memory::FRAME_SIZE-physical%memory::FRAME_SIZE).min(length-zeroed);unsafe{ptr::write_bytes(physical as*mut u8,0,count as usize);}zeroed+=count;}Ok(())}
 pub fn mark_running(&mut self){self.state=ProcessState::Running}
 pub fn exit(&mut self,code:i32){self.exit_code=code;self.state=ProcessState::Exited}
 pub fn fault(&mut self){self.state=ProcessState::Faulted}
}
impl Drop for Process{fn drop(&mut self){unsafe{let _=memory::deallocate_contiguous(self.kernel_stack_start,KERNEL_STACK_PAGES);}}}
