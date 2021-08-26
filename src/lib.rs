#![feature(c_unwind)]
#![feature(naked_functions)]
#![feature(asm)]
#![cfg_attr(any(feature = "personality", feature = "personality-dummy"), feature(lang_items))]
#![warn(rust_2018_idioms)]
#![warn(unsafe_op_in_unsafe_fn)]
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

mod abi;
mod arch;
mod find_fde;
mod frame;
mod util;

#[cfg(feature = "personality")]
mod personality;
#[cfg(feature = "personality-dummy")]
mod personality_dummy;

#[cfg(feature = "system-alloc")]
mod system_alloc;

pub use abi::*;
