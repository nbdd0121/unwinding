mod finder_static;
mod types;

use super::arch::*;
use crate::abi::PersonalityRoutine;
use crate::arch::*;

#[derive(Debug)]
struct UnwindData {
    // Personality routine used for unwinding
    personality_fn_index: u32,
    // Data to pass to personality routine
    arg: u32,
    lsda: usize,
    cmdbuf: [u8; 4],
    cmdbuf_len: usize,
}

#[derive(Debug)]
pub struct Frame {
    // Function entrypoint address
    entrypoint: usize,
    data: UnwindData,
}

impl Frame {
    pub fn from_context(ctx: &Context, signal: bool) -> Result<Option<Self>, gimli::Error> {
        assert!(!signal);

        let ra = ctx[Arch::RA];

        // Reached end of stack
        if ra == 0 {
            return Ok(None);
        }

        let exidx = finder_static::find_exidx().unwrap();
        let result = finder_static::find_entry_for_pc(exidx, ra).unwrap();

        let data = result.data;
        let unwind_data = if data == 1 {
            return Ok(None);
        } else if data & (1 << 31) != 0 {
            let arg = data & 0xffffff;
            let fn_index = (data >> 24) & 0xf;

            if fn_index != 0 {
                // TODO: gracefully handle error
                panic!("Inline entry may use pr0 only")
            }

            let cmdbuf = [(arg >> 16) as u8, (arg >> 8) as u8, arg as u8, 0];

            UnwindData {
                personality_fn_index: fn_index,
                arg,
                lsda: 0,
                cmdbuf,
                cmdbuf_len: 3,
            }
        } else {
            todo!("generic model not supported")
        };

        Ok(Some(Self {
            entrypoint: result.addr(),
            data: unwind_data,
        }))
    }

    pub fn unwind(&self, ctx: &Context) -> Result<Context, gimli::Error> {
        Ok(vrs_interpret(ctx, &self.data.cmdbuf[..self.data.cmdbuf_len]).unwrap())
    }

    pub fn bases(&self) -> &gimli::read::BaseAddresses {
        todo!()
    }

    pub fn personality(&self) -> Option<PersonalityRoutine> {
        todo!()
    }

    pub fn lsda(&self) -> usize {
        todo!()
    }

    pub fn initial_address(&self) -> usize {
        self.entrypoint
    }

    pub fn is_signal_trampoline(&self) -> bool {
        false
    }
}

#[no_mangle]
pub extern "C-unwind" fn __aeabi_unwind_cpp_pr0() {
    todo!()
}

#[no_mangle]
pub extern "C-unwind" fn __aeabi_unwind_cpp_pr1() {
    todo!()
}

fn vrs_interpret(ctx: &Context, code: &[u8]) -> Result<Context, ()> {
    let mut new_ctx = ctx.clone();
    let mut it = code.iter().copied();
    let mut wrote_pc = false;

    'main_loop: while let Some(b) = it.next() {
        if b & 0x80 == 0 {
            let adj = ((b & 0x3f) << 2) as usize + 4;

            if b & 0x40 == 0 {
                new_ctx.sp += adj;
            } else {
                new_ctx.sp -= adj;
            }
        } else {
            match b & 0xf0 {
                0x80 => {
                    let v = (b & 0xf) as u32;
                    let mask = (v << 12) | it.next().map(|x| x as u32).ok_or(())?;
                    if mask & (1 << 15) != 0 {
                        wrote_pc = true;
                    }
                    vrs_pop(&mut new_ctx, mask);
                }
                0xa0 => {
                    let n = ((b & 0x7) + 1) as u32;
                    let mut mask = ((1 << n) - 1) << 4;
                    if b & 8 != 0 {
                        mask |= 1 << 14;
                    }
                    vrs_pop(&mut new_ctx, mask);
                }
                0xb0 => match b & 0xf {
                    0 => break 'main_loop,
                    _ => todo!(),
                },
                cmd => todo!("unknown command {:#x}", cmd),
            }
        }
    }

    if !wrote_pc {
        new_ctx.pc = new_ctx.lr;
    }

    Ok(new_ctx)
}

fn vrs_pop(ctx: &mut Context, mask: u32) {
    let mut sp = ctx.sp as *const usize;
    let mut sp_popped = false;

    for i in 0..16 {
        if mask & 1 << i != 0 {
            let v = unsafe { sp.read() };
            sp = sp.wrapping_add(1);

            match i {
                0..=12 => ctx.gp[i] = v,
                13 => {
                    sp_popped = true;
                    ctx.sp = v;
                }
                14 => {
                    ctx.lr = v;
                }
                15 => {
                    ctx.pc = v;
                }
                _ => todo!(),
            }
        }
    }

    if !sp_popped {
        ctx.sp = sp as usize;
    }
}
