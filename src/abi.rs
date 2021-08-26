use core::ptr;
use gimli::Register;
use libc::{c_int, c_void};

use crate::arch::*;
use crate::find_fde::{self, FDEFinder};
use crate::frame::Frame;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnwindReasonCode(c_int);

#[allow(unused)]
impl UnwindReasonCode {
    pub const NO_REASON: Self = Self(0);
    pub const FOREIGN_EXCEPTION_CAUGHT: Self = Self(1);
    pub const FATAL_PHASE2_ERROR: Self = Self(2);
    pub const FATAL_PHASE1_ERROR: Self = Self(3);
    pub const NORMAL_STOP: Self = Self(4);
    pub const END_OF_STACK: Self = Self(5);
    pub const HANDLER_FOUND: Self = Self(6);
    pub const INSTALL_CONTEXT: Self = Self(7);
    pub const CONTINUE_UNWIND: Self = Self(8);
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnwindAction(c_int);

#[allow(unused)]
impl UnwindAction {
    pub const SEARCH_PHASE: Self = Self(1);
    pub const CLEANUP_PHASE: Self = Self(2);
    pub const HANDLER_FRAME: Self = Self(4);
    pub const FORCE_UNWIND: Self = Self(8);
    pub const END_OF_STACK: Self = Self(16);
}

pub type UnwindExceptionCleanupFn = extern "C" fn(UnwindReasonCode, *mut UnwindException);

pub type UnwindStopFn = extern "C" fn(
    c_int,
    UnwindAction,
    u64,
    *mut UnwindException,
    *mut UnwindContext<'_>,
    *mut c_void,
);

#[repr(C)]
pub struct UnwindException {
    pub exception_class: u64,
    pub exception_cleanup: Option<UnwindExceptionCleanupFn>,
}

pub type UnwindTraceFn =
    extern "C" fn(ctx: &mut UnwindContext<'_>, arg: *mut c_void) -> UnwindReasonCode;

pub struct UnwindContext<'a> {
    frame: &'a Frame,
    ctx: &'a mut Context,
}

pub type PersonalityRoutine = extern "C" fn(
    c_int,
    UnwindAction,
    u64,
    *mut UnwindException,
    *mut UnwindContext<'_>,
) -> UnwindReasonCode;

#[no_mangle]
pub extern "C" fn _Unwind_GetGR(unwind_ctx: &mut UnwindContext<'_>, index: c_int) -> usize {
    unwind_ctx.ctx[Register(index as u16)]
}

#[no_mangle]
pub extern "C" fn _Unwind_GetCFA(unwind_ctx: &mut UnwindContext<'_>) -> usize {
    unwind_ctx.ctx[Arch::SP]
}

#[no_mangle]
pub extern "C" fn _Unwind_SetGR(unwind_ctx: &mut UnwindContext<'_>, index: c_int, value: usize) {
    unwind_ctx.ctx[Register(index as u16)] = value;
}

#[no_mangle]
pub extern "C" fn _Unwind_GetIP(unwind_ctx: &mut UnwindContext<'_>) -> usize {
    unwind_ctx.ctx[Arch::RA]
}

#[no_mangle]
pub extern "C" fn _Unwind_GetIPInfo(
    unwind_ctx: &mut UnwindContext<'_>,
    ip_before_insn: &mut c_int,
) -> usize {
    *ip_before_insn = 0;
    unwind_ctx.ctx[Arch::RA]
}

#[no_mangle]
pub extern "C" fn _Unwind_SetIP(unwind_ctx: &mut UnwindContext<'_>, value: usize) {
    unwind_ctx.ctx[Arch::RA] = value;
}

#[no_mangle]
pub extern "C" fn _Unwind_GetLanguageSpecificData(
    unwind_ctx: &mut UnwindContext<'_>,
) -> *mut c_void {
    unwind_ctx.frame.lsda() as *mut c_void
}

#[no_mangle]
pub extern "C" fn _Unwind_GetRegionStart(unwind_ctx: &mut UnwindContext<'_>) -> usize {
    unwind_ctx.frame.initial_address()
}

#[no_mangle]
pub extern "C" fn _Unwind_GetTextRelBase(unwind_ctx: &mut UnwindContext<'_>) -> usize {
    unwind_ctx.frame.bases().eh_frame.text.unwrap() as _
}

#[no_mangle]
pub extern "C" fn _Unwind_GetDataRelBase(unwind_ctx: &mut UnwindContext<'_>) -> usize {
    unwind_ctx.frame.bases().eh_frame.data.unwrap() as _
}

#[no_mangle]
pub extern "C" fn _Unwind_FindEnclosingFunction(pc: *mut c_void) -> *mut c_void {
    match find_fde::get_finder().find_fde(pc as usize - 1) {
        Some(v) => v.fde.initial_address() as usize as _,
        None => ptr::null_mut(),
    }
}
