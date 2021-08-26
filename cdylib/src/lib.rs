#![no_std]
#![feature(default_alloc_error_handler)]
#![feature(lang_items)]
#![warn(rust_2018_idioms)]
#![warn(unsafe_op_in_unsafe_fn)]

// Keep this explicit
#[allow(unused_extern_crates)]
extern crate unwind;

use core::panic::PanicInfo;
use libc::c_int;
pub use unwind::*;

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    // `unwind` crate should never panic.
    unsafe { core::hint::unreachable_unchecked() }
}

#[lang = "eh_personality"]
extern "C" fn personality(
    version: c_int,
    _actions: UnwindAction,
    _exception_class: u64,
    _exception: &mut UnwindException,
    _ctx: &mut UnwindContext<'_>,
) -> UnwindReasonCode {
    if version != 1 {
        return UnwindReasonCode::FATAL_PHASE1_ERROR;
    }
    UnwindReasonCode::CONTINUE_UNWIND
}
