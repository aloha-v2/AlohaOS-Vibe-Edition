//! PS/2 set-1 keyboard driver with arrows and an IRQ-safe ring buffer.

use core::sync::atomic::{AtomicU8,AtomicUsize,Ordering};
use crate::{pic,pic::KEYBOARD_VECTOR};

const BUFFER_SIZE:usize=256;
static BUFFER:[AtomicU8;BUFFER_SIZE]=[const{AtomicU8::new(0)};BUFFER_SIZE];
static WRITE_INDEX:AtomicUsize=AtomicUsize::new(0);static READ_INDEX:AtomicUsize=AtomicUsize::new(0);static DROPPED:AtomicUsize=AtomicUsize::new(0);
static mut SHIFT:bool=false;static mut EXTENDED:bool=false;

#[derive(Clone,Copy)]pub enum Key{Character(u8),Enter,Backspace,Up,Down}

pub fn interrupt(){let scancode=unsafe{pic::inb(0x60)};let write=WRITE_INDEX.load(Ordering::Relaxed);let next=(write+1)%BUFFER_SIZE;if next!=READ_INDEX.load(Ordering::Acquire){BUFFER[write].store(scancode,Ordering::Relaxed);WRITE_INDEX.store(next,Ordering::Release)}else{DROPPED.fetch_add(1,Ordering::Relaxed)}unsafe{pic::end_of_interrupt(KEYBOARD_VECTOR)}}
pub fn pop_scancode()->Option<u8>{let read=READ_INDEX.load(Ordering::Relaxed);if read==WRITE_INDEX.load(Ordering::Acquire){return None}let value=BUFFER[read].load(Ordering::Relaxed);READ_INDEX.store((read+1)%BUFFER_SIZE,Ordering::Release);Some(value)}

pub fn decode(scancode:u8)->Option<Key>{unsafe{
    if scancode==0xe0{EXTENDED=true;return None}
    if EXTENDED{EXTENDED=false;return match scancode{0x48=>Some(Key::Up),0x50=>Some(Key::Down),_=>None}}
    match scancode{0x2a|0x36=>{SHIFT=true;return None},0xaa|0xb6=>{SHIFT=false;return None},code if code&0x80!=0=>return None,_=>{}}
    if scancode==0x1c{return Some(Key::Enter)}if scancode==0x0e{return Some(Key::Backspace)}
    let normal=match scancode{0x02..=0x0a=>b'1'+(scancode-2),0x0b=>b'0',0x0f=>b' ',0x10=>b'q',0x11=>b'w',0x12=>b'e',0x13=>b'r',0x14=>b't',0x15=>b'y',0x16=>b'u',0x17=>b'i',0x18=>b'o',0x19=>b'p',0x1e=>b'a',0x1f=>b's',0x20=>b'd',0x21=>b'f',0x22=>b'g',0x23=>b'h',0x24=>b'j',0x25=>b'k',0x26=>b'l',0x2c=>b'z',0x2d=>b'x',0x2e=>b'c',0x2f=>b'v',0x30=>b'b',0x31=>b'n',0x32=>b'm',_=>return None};
    Some(Key::Character(if SHIFT&&normal.is_ascii_lowercase(){normal.to_ascii_uppercase()}else{normal}))
}}
pub fn dropped_scancodes()->usize{DROPPED.load(Ordering::Relaxed)}
