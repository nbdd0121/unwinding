#![feature(c_unwind)]
#![feature(naked_functions)]
#![feature(asm)]
#![cfg_attr(
    any(feature = "personality", feature = "personality-dummy"),
    feature(lang_items)
)]
#![cfg_attr(feature = "panic", feature(core_intrinsics))]
#![warn(rust_2018_idioms)]
#![warn(unsafe_op_in_unsafe_fn)]
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod abi;

mod arch;
mod find_fde;
mod frame;
mod util;

#[cfg(feature = "print")]
pub mod print;

#[cfg(feature = "personality")]
mod personality;
#[cfg(feature = "personality-dummy")]
mod personality_dummy;

#[cfg(feature = "panic")]
pub mod panic;

#[cfg(feature = "system-alloc")]
mod system_alloc;
