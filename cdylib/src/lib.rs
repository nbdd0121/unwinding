#![no_std]
#![feature(default_alloc_error_handler)]
#![feature(lang_items)]
#![warn(rust_2018_idioms)]
#![warn(unsafe_op_in_unsafe_fn)]

// Keep this explicit
#[allow(unused_extern_crates)]
extern crate unwind;

use core::alloc::{GlobalAlloc, Layout};
use core::panic::PanicInfo;
use core::{cmp, mem, ptr};
use libc::c_int;
use unwind::*;

pub struct System;

const MIN_ALIGN: usize = mem::size_of::<usize>() * 2;

// Taken std
unsafe impl GlobalAlloc for System {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() <= MIN_ALIGN && layout.align() <= layout.size() {
            unsafe { libc::malloc(layout.size()) as *mut u8 }
        } else {
            let mut out = ptr::null_mut();
            let align = layout.align().max(mem::size_of::<usize>());
            let ret = unsafe { libc::posix_memalign(&mut out, align, layout.size()) };
            if ret != 0 {
                ptr::null_mut()
            } else {
                out as *mut u8
            }
        }
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if layout.align() <= MIN_ALIGN && layout.align() <= layout.size() {
            unsafe { libc::calloc(layout.size(), 1) as *mut u8 }
        } else {
            let ptr = unsafe { self.alloc(layout) };
            if !ptr.is_null() {
                unsafe { ptr::write_bytes(ptr, 0, layout.size()) };
            }
            ptr
        }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        unsafe { libc::free(ptr as *mut libc::c_void) }
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if layout.align() <= MIN_ALIGN && layout.align() <= new_size {
            unsafe { libc::realloc(ptr as *mut libc::c_void, new_size) as *mut u8 }
        } else {
            let new_layout = unsafe { Layout::from_size_align_unchecked(new_size, layout.align()) };

            let new_ptr = unsafe { self.alloc(new_layout) };
            if !new_ptr.is_null() {
                let size = cmp::min(layout.size(), new_size);
                unsafe { ptr::copy_nonoverlapping(ptr, new_ptr, size) };
                unsafe { self.dealloc(ptr, layout) };
            }
            new_ptr
        }
    }
}

#[global_allocator]
static GLOBAL: System = System;

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
