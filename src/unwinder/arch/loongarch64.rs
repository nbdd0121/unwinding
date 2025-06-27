use core::fmt;
use core::ops;
use gimli::{LoongArch, Register};

use super::maybe_cfi;

pub const MAX_REG_RULES: usize = 0; // TODO: find the GCC's MAX_REG_RULES for loongarch64

#[repr(C)]
#[derive(Clone, Default)]
pub struct Context {
    pub gp: [usize; 32],
    // sp is gp[3]
    pub fp: [usize; 32],
}

impl fmt::Debug for Context {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fmt = fmt.debug_struct("Context");
        for i in 0..=31 {
            fmt.field(
                LoongArch::register_name(Register(i as _)).unwrap(),
                &self.gp[i],
            );
        }
        fmt.field("sp", &self.gp[3]);
        for i in 0..=31 {
            fmt.field(
                LoongArch::register_name(Register((i + 32) as _)).unwrap(),
                &self.fp[i],
            );
        }
        fmt.finish()
    }
}

impl ops::Index<Register> for Context {
    type Output = usize;

    fn index(&self, reg: Register) -> &usize {
        match reg {
            Register(0..=31) => &self.gp[reg.0 as usize],
            Register(32..=63) => &self.fp[(reg.0 - 32) as usize],
            _ => unimplemented!(),
        }
    }
}

impl ops::IndexMut<gimli::Register> for Context {
    fn index_mut(&mut self, reg: Register) -> &mut usize {
        match reg {
            Register(0..=31) => &mut self.gp[reg.0 as usize],
            Register(32..=63) => &mut self.fp[(reg.0 - 32) as usize],
            _ => unimplemented!(),
        }
    }
}

macro_rules! save {
    (gp$(, $fp:ident)?) => {
        core::arch::naked_asm!("addi.d $r3, $r3, -16", "ret",);
    };
    (maybesavefp(fp)) => {
        "
        fst.d $f24, ($r3, 0x140)
        "
    };
    (maybesavefp()) => {
        ""
    };
}

#[unsafe(naked)]
pub extern "C-unwind" fn save_context(f: extern "C" fn(&mut Context, *mut ()), ptr: *mut ()) {
    unsafe {
        save!(gp, fp);
    }
}

macro_rules! restore {
    ($ctx:expr, gp$(, $fp:ident)?) => {
        core::arch::asm!(
        "addi.d $r3, $r3, -16",
        in("$r1") $ctx,
        options(noreturn));
    };
    (mayberestore(fp)) => {
        "
        "
    };
    (mayberestore()) => {
        ""
    };
}

pub unsafe fn restore_context(ctx: &Context) -> ! {
    unsafe {
        restore!(ctx, gp, fp);
    }
}
