use alloc::boxed::Box;
use core::any::Any;
use core::mem::MaybeUninit;

use crate::abi::*;
#[cfg(feature = "panic-handler")]
pub use crate::panic_handler::*;
use crate::panicking::Exception;

#[repr(transparent)]
struct RustPanic(Box<dyn Any + Send>, ForeignGuard);

struct ForeignGuard;

impl Drop for ForeignGuard {
    fn drop(&mut self) {
        #[cfg(feature = "panic-handler")]
        {
            foreign_exception();
        }
        core::intrinsics::abort();
    }
}

#[repr(C)]
struct ExceptionWithPayload {
    exception: MaybeUninit<UnwindException>,
    payload: RustPanic,
}

unsafe impl Exception for RustPanic {
    const CLASS: [u8; 8] = *b"MOZ\0RUST";

    fn wrap(this: Self) -> *mut UnwindException {
        Box::into_raw(Box::new(ExceptionWithPayload {
            exception: MaybeUninit::uninit(),
            payload: this,
        })) as *mut UnwindException
    }

    unsafe fn unwrap(ex: *mut UnwindException) -> Self {
        let ex = unsafe { Box::from_raw(ex as *mut ExceptionWithPayload) };
        ex.payload
    }
}

pub fn begin_panic(payload: Box<dyn Any + Send>) -> UnwindReasonCode {
    crate::panicking::begin_panic(RustPanic(payload, ForeignGuard))
}

pub fn catch_unwind<R, F: FnOnce() -> R>(f: F) -> Result<R, Box<dyn Any + Send>> {
    #[cold]
    fn process_panic(p: Option<RustPanic>) -> Box<dyn Any + Send> {
        match p {
            None => {
                #[cfg(feature = "panic-handler")]
                {
                    drop_panic();
                }
                core::intrinsics::abort();
            }
            Some(e) => {
                panic_caught();
                core::mem::forget(e.1);
                e.0
            }
        }
    }
    crate::panicking::catch_unwind(f).map_err(process_panic)
}
