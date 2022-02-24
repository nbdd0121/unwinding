#![doc = include_str!("../README.md")]
#![feature(c_unwind)]
#![feature(naked_functions)]
#![cfg_attr(
    any(feature = "personality", feature = "personality-dummy"),
    feature(lang_items)
)]
#![cfg_attr(
    any(feature = "panicking", feature = "panic-handler-dummy"),
    feature(core_intrinsics)
)]
#![cfg_attr(feature = "panic-handler", feature(thread_local))]
#![warn(rust_2018_idioms)]
#![warn(unsafe_op_in_unsafe_fn)]
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "unwinder")]
mod unwinder;

pub mod abi;

mod arch;
mod util;

#[cfg(feature = "print")]
pub mod print;

#[cfg(feature = "personality")]
mod personality;
#[cfg(feature = "personality-dummy")]
mod personality_dummy;

#[cfg(feature = "panic")]
pub mod panic;
#[cfg(feature = "panicking")]
pub mod panicking;

#[cfg(feature = "panic-handler")]
mod panic_handler;
#[cfg(feature = "panic-handler-dummy")]
mod panic_handler_dummy;

#[cfg(feature = "system-alloc")]
mod system_alloc;
