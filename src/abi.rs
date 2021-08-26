use core::ffi::c_void;
use core::ops;

use crate::arch::*;
use crate::util::*;

#[cfg(feature = "unwinder")]
use crate::unwinder::*;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct UnwindReasonCode(pub c_int);

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
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct UnwindAction(pub c_int);

impl UnwindAction {
    pub const SEARCH_PHASE: Self = Self(1);
    pub const CLEANUP_PHASE: Self = Self(2);
    pub const HANDLER_FRAME: Self = Self(4);
    pub const FORCE_UNWIND: Self = Self(8);
    pub const END_OF_STACK: Self = Self(16);
}

impl ops::BitOr for UnwindAction {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl UnwindAction {
    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn contains(&self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}

pub type UnwindExceptionCleanupFn = unsafe extern "C" fn(UnwindReasonCode, *mut UnwindException);

pub type UnwindStopFn = extern "C" fn(
    c_int,
    UnwindAction,
    u64,
    &mut UnwindException,
    &mut UnwindContext<'_>,
    *mut c_void,
) -> UnwindReasonCode;

#[cfg(not(feature = "unwinder"))]
#[repr(C)]
pub struct UnwindException {
    pub exception_class: u64,
    pub exception_cleanup: Option<UnwindExceptionCleanupFn>,
    private: [usize; Arch::UNWIND_PRIVATE_DATA_SIZE],
}

impl UnwindException {
    #[inline]
    pub fn new() -> UnwindException {
        unsafe { core::mem::zeroed() }
    }
}

pub type UnwindTraceFn =
    extern "C" fn(ctx: &mut UnwindContext<'_>, arg: *mut c_void) -> UnwindReasonCode;

#[cfg(not(feature = "unwinder"))]
pub struct UnwindContext<'a> {
    opaque: usize,
    phantom: core::marker::PhantomData<&'a ()>,
}

pub type PersonalityRoutine = extern "C" fn(
    c_int,
    UnwindAction,
    u64,
    &mut UnwindException,
    &mut UnwindContext<'_>,
) -> UnwindReasonCode;
