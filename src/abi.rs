use core::ffi::c_void;
use core::ops;
use core::ptr;
use gimli::Register;

use crate::arch::*;
use crate::find_fde::{self, FDEFinder};
use crate::frame::Frame;
use crate::util::*;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
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
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct UnwindAction(c_int);

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

pub type UnwindStopFn = unsafe extern "C" fn(
    c_int,
    UnwindAction,
    u64,
    &mut UnwindException,
    &mut UnwindContext<'_>,
    *mut c_void,
) -> UnwindReasonCode;

#[repr(C)]
pub struct UnwindException {
    pub exception_class: u64,
    pub exception_cleanup: Option<UnwindExceptionCleanupFn>,
    private_1: Option<UnwindStopFn>,
    private_2: usize,
    private_unused: [usize; Arch::UNWIND_PRIVATE_DATA_SIZE - 2],
}

pub type UnwindTraceFn =
    unsafe extern "C" fn(ctx: &mut UnwindContext<'_>, arg: *mut c_void) -> UnwindReasonCode;

pub struct UnwindContext<'a> {
    frame: Option<&'a Frame>,
    ctx: &'a mut Context,
}

pub type PersonalityRoutine = extern "C" fn(
    c_int,
    UnwindAction,
    u64,
    &mut UnwindException,
    &mut UnwindContext<'_>,
) -> UnwindReasonCode;

#[no_mangle]
pub extern "C" fn _Unwind_GetGR(unwind_ctx: &UnwindContext<'_>, index: c_int) -> usize {
    unwind_ctx.ctx[Register(index as u16)]
}

#[no_mangle]
pub extern "C" fn _Unwind_GetCFA(unwind_ctx: &UnwindContext<'_>) -> usize {
    unwind_ctx.ctx[Arch::SP]
}

#[no_mangle]
pub extern "C" fn _Unwind_SetGR(unwind_ctx: &mut UnwindContext<'_>, index: c_int, value: usize) {
    unwind_ctx.ctx[Register(index as u16)] = value;
}

#[no_mangle]
pub extern "C" fn _Unwind_GetIP(unwind_ctx: &UnwindContext<'_>) -> usize {
    unwind_ctx.ctx[Arch::RA]
}

#[no_mangle]
pub extern "C" fn _Unwind_GetIPInfo(
    unwind_ctx: &UnwindContext<'_>,
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
pub extern "C" fn _Unwind_GetLanguageSpecificData(unwind_ctx: &UnwindContext<'_>) -> *mut c_void {
    unwind_ctx
        .frame
        .map(|f| f.lsda() as *mut c_void)
        .unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn _Unwind_GetRegionStart(unwind_ctx: &UnwindContext<'_>) -> usize {
    unwind_ctx.frame.map(|f| f.initial_address()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn _Unwind_GetTextRelBase(unwind_ctx: &UnwindContext<'_>) -> usize {
    unwind_ctx
        .frame
        .map(|f| f.bases().eh_frame.text.unwrap() as _)
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn _Unwind_GetDataRelBase(unwind_ctx: &UnwindContext<'_>) -> usize {
    unwind_ctx
        .frame
        .map(|f| f.bases().eh_frame.data.unwrap() as _)
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn _Unwind_FindEnclosingFunction(pc: *mut c_void) -> *mut c_void {
    find_fde::get_finder()
        .find_fde(pc as usize - 1)
        .map(|r| r.fde.initial_address() as usize as _)
        .unwrap_or(ptr::null_mut())
}

macro_rules! try1 {
    ($e: expr) => {{
        match $e {
            Ok(v) => v,
            Err(_) => return UnwindReasonCode::FATAL_PHASE1_ERROR,
        }
    }};
}

macro_rules! try2 {
    ($e: expr) => {{
        match $e {
            Ok(v) => v,
            Err(_) => return UnwindReasonCode::FATAL_PHASE2_ERROR,
        }
    }};
}

#[no_mangle]
pub extern "C-unwind" fn _Unwind_RaiseException(
    exception: &mut UnwindException,
) -> UnwindReasonCode {
    let saved_ctx = save_context();

    // Phase 1: Search for handler
    let mut ctx = saved_ctx.clone();
    let handler_cfa = loop {
        if let Some(frame) = try1!(Frame::from_context(&ctx)) {
            if let Some(personality) = frame.personality() {
                let result = personality(
                    1,
                    UnwindAction::SEARCH_PHASE,
                    exception.exception_class,
                    exception,
                    &mut UnwindContext {
                        frame: Some(&frame),
                        ctx: &mut ctx,
                    },
                );

                match result {
                    UnwindReasonCode::CONTINUE_UNWIND => (),
                    UnwindReasonCode::HANDLER_FOUND => {
                        exception.private_1 = None;
                        exception.private_2 = ctx[Arch::SP];
                        break ctx[Arch::SP];
                    }
                    _ => return UnwindReasonCode::FATAL_PHASE1_ERROR,
                }
            }

            ctx = try1!(frame.unwind(&ctx));
        } else {
            return UnwindReasonCode::END_OF_STACK;
        }
    };

    let mut ctx = saved_ctx;
    let code = raise_exception_phase2(exception, &mut ctx, handler_cfa);
    match code {
        UnwindReasonCode::INSTALL_CONTEXT => unsafe { restore_context(&ctx) },
        _ => code,
    }
}

fn raise_exception_phase2(
    exception: &mut UnwindException,
    ctx: &mut Context,
    handler_cfa: usize,
) -> UnwindReasonCode {
    loop {
        if let Some(frame) = try2!(Frame::from_context(ctx)) {
            let is_handler = ctx[Arch::SP] == handler_cfa;
            if let Some(personality) = frame.personality() {
                let code = personality(
                    1,
                    UnwindAction::CLEANUP_PHASE
                        | if is_handler {
                            UnwindAction::HANDLER_FRAME
                        } else {
                            UnwindAction::empty()
                        },
                    exception.exception_class,
                    exception,
                    &mut UnwindContext {
                        frame: Some(&frame),
                        ctx,
                    },
                );

                match code {
                    UnwindReasonCode::CONTINUE_UNWIND => (),
                    UnwindReasonCode::INSTALL_CONTEXT => break,
                    _ => return UnwindReasonCode::FATAL_PHASE2_ERROR,
                }
            }

            *ctx = try2!(frame.unwind(ctx));
        } else {
            return UnwindReasonCode::FATAL_PHASE2_ERROR;
        }
    }

    UnwindReasonCode::INSTALL_CONTEXT
}

#[no_mangle]
pub unsafe extern "C-unwind" fn _Unwind_ForceUnwind(
    exception: &mut UnwindException,
    stop: UnwindStopFn,
    stop_arg: *mut c_void,
) -> UnwindReasonCode {
    let mut ctx = save_context();

    exception.private_1 = Some(stop);
    exception.private_2 = stop_arg as _;

    let code = unsafe { force_unwind_phase2(exception, &mut ctx, stop, stop_arg) };
    match code {
        UnwindReasonCode::INSTALL_CONTEXT => unsafe { restore_context(&ctx) },
        _ => code,
    }
}

unsafe fn force_unwind_phase2(
    exception: &mut UnwindException,
    ctx: &mut Context,
    stop: UnwindStopFn,
    stop_arg: *mut c_void,
) -> UnwindReasonCode {
    loop {
        let frame = try2!(Frame::from_context(ctx));

        let code = unsafe {
            stop(
                1,
                UnwindAction::FORCE_UNWIND
                    | UnwindAction::END_OF_STACK
                    | if frame.is_none() {
                        UnwindAction::END_OF_STACK
                    } else {
                        UnwindAction::empty()
                    },
                exception.exception_class,
                exception,
                &mut UnwindContext {
                    frame: frame.as_ref(),
                    ctx,
                },
                stop_arg,
            )
        };
        match code {
            UnwindReasonCode::NO_REASON => (),
            _ => return UnwindReasonCode::FATAL_PHASE2_ERROR,
        }

        if let Some(frame) = frame {
            if let Some(personality) = frame.personality() {
                let code = personality(
                    1,
                    UnwindAction::FORCE_UNWIND | UnwindAction::CLEANUP_PHASE,
                    exception.exception_class,
                    exception,
                    &mut UnwindContext {
                        frame: Some(&frame),
                        ctx,
                    },
                );

                match code {
                    UnwindReasonCode::CONTINUE_UNWIND => (),
                    UnwindReasonCode::INSTALL_CONTEXT => break,
                    _ => return UnwindReasonCode::FATAL_PHASE2_ERROR,
                }
            }

            *ctx = try2!(frame.unwind(ctx));
        } else {
            return UnwindReasonCode::END_OF_STACK;
        }
    }

    UnwindReasonCode::INSTALL_CONTEXT
}

#[no_mangle]
pub extern "C-unwind" fn _Unwind_Resume(exception: &mut UnwindException) -> ! {
    let mut ctx = save_context();

    let code = match exception.private_1 {
        None => {
            let handler_cfa = exception.private_2;
            raise_exception_phase2(exception, &mut ctx, handler_cfa)
        }
        Some(stop) => {
            let stop_arg = exception.private_2 as _;
            unsafe { force_unwind_phase2(exception, &mut ctx, stop, stop_arg) }
        }
    };
    assert!(code == UnwindReasonCode::INSTALL_CONTEXT);

    unsafe { restore_context(&ctx) }
}

#[no_mangle]
pub extern "C-unwind" fn _Unwind_Resume_or_Rethrow(
    exception: &mut UnwindException,
) -> UnwindReasonCode {
    let stop = match exception.private_1 {
        None => return _Unwind_RaiseException(exception),
        Some(v) => v,
    };

    let mut ctx = save_context();

    let stop_arg = exception.private_2 as _;
    let code = unsafe { force_unwind_phase2(exception, &mut ctx, stop, stop_arg) };
    assert!(code == UnwindReasonCode::INSTALL_CONTEXT);

    unsafe { restore_context(&ctx) }
}

#[no_mangle]
pub unsafe extern "C" fn _Unwind_DeleteException(exception: *mut UnwindException) {
    if let Some(cleanup) = unsafe { (*exception).exception_cleanup } {
        unsafe { cleanup(UnwindReasonCode::FOREIGN_EXCEPTION_CAUGHT, exception) };
    }
}

#[no_mangle]
pub unsafe extern "C-unwind" fn _Unwind_Backtrace(
    trace: UnwindTraceFn,
    trace_argument: *mut c_void,
) -> UnwindReasonCode {
    let mut ctx = save_context();
    let mut skipping = cfg!(feature = "hide-trace");

    loop {
        let frame = try1!(Frame::from_context(&ctx));
        if !skipping {
            let code = unsafe {
                trace(
                    &mut UnwindContext {
                        frame: frame.as_ref(),
                        ctx: &mut ctx,
                    },
                    trace_argument,
                )
            };
            match code {
                UnwindReasonCode::NO_REASON => (),
                _ => return UnwindReasonCode::FATAL_PHASE1_ERROR,
            }
        }
        if let Some(frame) = frame {
            if skipping {
                if frame.initial_address() == _Unwind_Backtrace as usize {
                    skipping = false;
                }
            }
            ctx = try1!(frame.unwind(&ctx));
        } else {
            return UnwindReasonCode::END_OF_STACK;
        }
    }
}
