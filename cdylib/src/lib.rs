#![no_std]
#![feature(default_alloc_error_handler)]
#![feature(lang_items)]
#![warn(rust_2018_idioms)]
#![warn(unsafe_op_in_unsafe_fn)]

// Keep this explicit
#[allow(unused_extern_crates)]
extern crate unwind;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    // `unwind` crate should never panic.
    unsafe { core::hint::unreachable_unchecked() }
}
