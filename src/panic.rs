use alloc::boxed::Box;
use core::any::Any;
use core::mem::ManuallyDrop;

use crate::abi::*;

#[repr(C)]
struct Exception {
    exception: UnwindException,
    payload: Box<dyn Any + Send>,
}

const RUST_EXCEPTION_CLASS: u64 = u64::from_be_bytes(*b"MOZ\0RUST");

pub fn begin_panic(payload: Box<dyn Any + Send>) -> UnwindReasonCode {
    unsafe extern "C" fn exception_cleanup(
        _unwind_code: UnwindReasonCode,
        exception: *mut UnwindException,
    ) {
        unsafe {
            let _ = Box::from_raw(exception as *mut Exception);
        }
        core::intrinsics::abort();
    }

    let mut unwind_ex = UnwindException::new();
    unwind_ex.exception_class = RUST_EXCEPTION_CLASS;
    unwind_ex.exception_cleanup = Some(exception_cleanup);
    let exception = Box::new(Exception {
        exception: unwind_ex,
        payload,
    });
    _Unwind_RaiseException(unsafe { &mut *(Box::into_raw(exception) as *mut UnwindException) })
}

#[cold]
unsafe fn cleanup(payload: *mut u8) -> Box<dyn Any + Send + 'static> {
    let exception = payload as *mut UnwindException;
    if unsafe { (*exception).exception_class } != RUST_EXCEPTION_CLASS {
        unsafe { _Unwind_DeleteException(exception) };
        core::intrinsics::abort();
    }
    let unwind_ex = unsafe { Box::from_raw(exception as *mut Exception) };
    unwind_ex.payload
}

pub fn catch_unwind<R, F: FnOnce() -> R>(f: F) -> Result<R, Box<dyn Any + Send>> {
    union Data<F, R> {
        f: ManuallyDrop<F>,
        r: ManuallyDrop<R>,
        p: ManuallyDrop<Box<dyn Any + Send>>,
    }

    let mut data = Data {
        f: ManuallyDrop::new(f),
    };

    let data_ptr = &mut data as *mut _ as *mut u8;
    unsafe {
        return if core::intrinsics::r#try(do_call::<F, R>, data_ptr, do_catch::<F, R>) == 0 {
            Ok(ManuallyDrop::into_inner(data.r))
        } else {
            Err(ManuallyDrop::into_inner(data.p))
        };
    }

    #[inline]
    fn do_call<F: FnOnce() -> R, R>(data: *mut u8) {
        unsafe {
            let data = data as *mut Data<F, R>;
            let data = &mut (*data);
            let f = ManuallyDrop::take(&mut data.f);
            data.r = ManuallyDrop::new(f());
        }
    }

    #[inline]
    fn do_catch<F: FnOnce() -> R, R>(data: *mut u8, payload: *mut u8) {
        unsafe {
            let data = data as *mut Data<F, R>;
            let data = &mut (*data);
            let obj = cleanup(payload);
            data.p = ManuallyDrop::new(obj);
        }
    }
}
