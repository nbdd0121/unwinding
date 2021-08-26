use libc::{c_int, c_void};

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
}

pub type PersonalityRoutine = extern "C" fn(
    c_int,
    UnwindAction,
    u64,
    *mut UnwindException,
    *mut UnwindContext<'_>,
) -> UnwindReasonCode;
