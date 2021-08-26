mod arch;
mod find_fde;
mod frame;

use core::ffi::c_void;
use core::ptr;
use gimli::Register;

use crate::abi::*;
use crate::arch::*;
use crate::util::*;
use arch::*;
use find_fde::FDEFinder;
use frame::Frame;

#[repr(C)]
pub struct UnwindException {
    pub exception_class: u64,
    pub exception_cleanup: Option<UnwindExceptionCleanupFn>,
    private_1: Option<UnwindStopFn>,
    private_2: usize,
    private_unused: [usize; Arch::UNWIND_PRIVATE_DATA_SIZE - 2],
}

pub struct UnwindContext<'a> {
    frame: Option<&'a Frame>,
    ctx: &'a mut Context,
}

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
pub extern "C-unwind" fn _Unwind_ForceUnwind(
    exception: &mut UnwindException,
    stop: UnwindStopFn,
    stop_arg: *mut c_void,
) -> UnwindReasonCode {
    let mut ctx = save_context();

    exception.private_1 = Some(stop);
    exception.private_2 = stop_arg as _;

    let code = force_unwind_phase2(exception, &mut ctx, stop, stop_arg);
    match code {
        UnwindReasonCode::INSTALL_CONTEXT => unsafe { restore_context(&ctx) },
        _ => code,
    }
}

fn force_unwind_phase2(
    exception: &mut UnwindException,
    ctx: &mut Context,
    stop: UnwindStopFn,
    stop_arg: *mut c_void,
) -> UnwindReasonCode {
    loop {
        let frame = try2!(Frame::from_context(ctx));

        let code = stop(
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
        );
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
            force_unwind_phase2(exception, &mut ctx, stop, stop_arg)
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
    let code = force_unwind_phase2(exception, &mut ctx, stop, stop_arg);
    assert!(code == UnwindReasonCode::INSTALL_CONTEXT);

    unsafe { restore_context(&ctx) }
}

#[no_mangle]
pub unsafe extern "C" fn _Unwind_DeleteException(exception: *mut UnwindException) {
    if let Some(cleanup) = unsafe { (*exception).exception_cleanup } {
        unsafe { cleanup(UnwindReasonCode::FOREIGN_EXCEPTION_CAUGHT, exception) };
    }
}

#[inline(never)]
#[no_mangle]
pub extern "C-unwind" fn _Unwind_Backtrace(
    trace: UnwindTraceFn,
    trace_argument: *mut c_void,
) -> UnwindReasonCode {
    let mut ctx = save_context();
    let mut skipping = cfg!(feature = "hide-trace");

    loop {
        let frame = try1!(Frame::from_context(&ctx));
        if !skipping {
            let code = trace(
                &mut UnwindContext {
                    frame: frame.as_ref(),
                    ctx: &mut ctx,
                },
                trace_argument,
            );
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
