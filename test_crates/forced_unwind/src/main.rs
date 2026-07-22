#![no_std]
#![no_main]

extern crate alloc;
extern crate unwinding;

use alloc::boxed::Box;
use core::ffi::{c_int, c_void};
use core::mem::MaybeUninit;
use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use unwinding::abi::*;
use unwinding::print::*;

#[link(name = "c")]
unsafe extern "C" {}

static STOP_ARG: u8 = 0;

static EXCEPTION: AtomicPtr<UnwindException> = AtomicPtr::new(ptr::null_mut());

static DROPPED: AtomicBool = AtomicBool::new(false);

static EXCEPTION_FREED: AtomicBool = AtomicBool::new(false);

struct RecordOnDrop;

impl Drop for RecordOnDrop {
    fn drop(&mut self) {
        DROPPED.store(true, Ordering::Relaxed);
    }
}

unsafe extern "C" fn exception_cleanup(_code: UnwindReasonCode, _exception: *mut UnwindException) {
    EXCEPTION_FREED.store(true, Ordering::Relaxed);
}

unsafe extern "C" fn stop_fn(
    version: c_int,
    actions: UnwindAction,
    _exception_class: u64,
    exception: *mut UnwindException,
    _unwind_ctx: &mut UnwindContext<'_>,
    arg: *mut c_void,
) -> UnwindReasonCode {
    assert_eq!(version, 1);
    assert_eq!(
        arg, &raw const STOP_ARG as *mut c_void,
        "stop argument not passed through"
    );
    assert_eq!(
        exception,
        EXCEPTION.load(Ordering::Relaxed),
        "exception object not passed through"
    );
    assert!(
        actions.contains(UnwindAction::FORCE_UNWIND),
        "FORCE_UNWIND not set"
    );
    assert!(
        actions.contains(UnwindAction::CLEANUP_PHASE),
        "CLEANUP_PHASE not set"
    );

    if actions.contains(UnwindAction::END_OF_STACK) {
        // Unwinder is at end of stack, so `_d` should have been cleaned up.
        assert!(
            DROPPED.load(Ordering::Relaxed),
            "reached end of stack but the cleanup never ran"
        );
        // Unwinder must not have freed our exception.
        assert!(
            !EXCEPTION_FREED.load(Ordering::Relaxed),
            "unwinder called the exception cleanup routine during forced unwind"
        );
        eprintln!("forced unwind reached end of stack with cleanups run");
        unsafe { libc::exit(0) };
    }
    // Not at end of stack yet, keep unwinding.
    UnwindReasonCode::NO_REASON
}

fn foo() {
    // Leak the box so it outlives `foo()`.
    let exception: *mut UnwindException =
        Box::into_raw(Box::new(MaybeUninit::<UnwindException>::zeroed())).cast();
    unsafe {
        // Made-up class so the exception stays foreign and nothing reads into it.
        (*exception).exception_class = u64::from_ne_bytes(*b"TSTFRCEU");
        (*exception).exception_cleanup = Some(exception_cleanup);
    }
    EXCEPTION.store(exception, Ordering::Relaxed);

    let arg = &raw const STOP_ARG as *mut c_void;
    let code = unsafe { _Unwind_ForcedUnwind(exception, stop_fn, arg) };

    // The stop function exits at end of stack, so reaching here means the test failed.
    panic!("forced unwind returned unexpectedly: {}", code.0);
}

fn main() {
    let _d = RecordOnDrop;
    foo();
}

// `C-unwind` lets the unwind propagate past `main`, to the end of the stack.
#[unsafe(export_name = "main")]
extern "C-unwind" fn start(_argc: isize, _argv: *const *const u8) -> isize {
    main();
    0
}
