use core::arch::asm;
use core::fmt;
use core::ops;
use gimli::{Register, RiscV};

// Match DWARF_FRAME_REGISTERS in libgcc
pub const MAX_REG_RULES: usize = 65;

#[cfg(all(target_feature = "f", not(target_feature = "d")))]
compile_error!("RISC-V with only F extension is not supported");

#[repr(C)]
#[derive(Clone, Default)]
pub struct Context {
    pub gp: [usize; 32],
    #[cfg(target_feature = "d")]
    pub fp: [u64; 32],
}

impl fmt::Debug for Context {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fmt = fmt.debug_struct("Context");
        for i in 0..=31 {
            fmt.field(RiscV::register_name(Register(i as _)).unwrap(), &self.gp[i]);
        }
        #[cfg(target_feature = "d")]
        for i in 0..=31 {
            fmt.field(
                RiscV::register_name(Register((i + 32) as _)).unwrap(),
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
            // We cannot support indexing fp here. It is 64-bit if D extension is implemented,
            // and 32-bit if only F extension is implemented.
            _ => unimplemented!(),
        }
    }
}

impl ops::IndexMut<gimli::Register> for Context {
    fn index_mut(&mut self, reg: Register) -> &mut usize {
        match reg {
            Register(0..=31) => &mut self.gp[reg.0 as usize],
            // We cannot support indexing fp here. It is 64-bit if D extension is implemented,
            // and 32-bit if only F extension is implemented.
            _ => unimplemented!(),
        }
    }
}

macro_rules! code {
    (save_gp) => {
        "
        sw x0, 0x00(a0)
        sw ra, 0x04(a0)
        sw sp, 0x08(a0)
        sw gp, 0x0C(a0)
        sw tp, 0x10(a0)
        sw s0, 0x20(a0)
        sw s1, 0x24(a0)
        sw s2, 0x48(a0)
        sw s3, 0x4C(a0)
        sw s4, 0x50(a0)
        sw s5, 0x54(a0)
        sw s6, 0x58(a0)
        sw s7, 0x5C(a0)
        sw s8, 0x60(a0)
        sw s9, 0x64(a0)
        sw s10, 0x68(a0)
        sw s11, 0x6C(a0)
        "
    };
    (save_fp) => {
        "
        fsd fs0, 0xC0(a0)
        fsd fs1, 0xC8(a0)
        fsd fs2, 0x110(a0)
        fsd fs3, 0x118(a0)
        fsd fs4, 0x120(a0)
        fsd fs5, 0x128(a0)
        fsd fs6, 0x130(a0)
        fsd fs7, 0x138(a0)
        fsd fs8, 0x140(a0)
        fsd fs9, 0x148(a0)
        fsd fs10, 0x150(a0)
        fsd fs11, 0x158(a0)
        "
    };
    (restore_gp) => {
        "
        lw ra, 0x04(a0)
        lw sp, 0x08(a0)
        lw gp, 0x0C(a0)
        lw tp, 0x10(a0)
        lw t0, 0x14(a0)
        lw t1, 0x18(a0)
        lw t2, 0x1C(a0)
        lw s0, 0x20(a0)
        lw s1, 0x24(a0)
        lw a1, 0x2C(a0)
        lw a2, 0x30(a0)
        lw a3, 0x34(a0)
        lw a4, 0x38(a0)
        lw a5, 0x3C(a0)
        lw a6, 0x40(a0)
        lw a7, 0x44(a0)
        lw s2, 0x48(a0)
        lw s3, 0x4C(a0)
        lw s4, 0x50(a0)
        lw s5, 0x54(a0)
        lw s6, 0x58(a0)
        lw s7, 0x5C(a0)
        lw s8, 0x60(a0)
        lw s9, 0x64(a0)
        lw s10, 0x68(a0)
        lw s11, 0x6C(a0)
        lw t3, 0x70(a0)
        lw t4, 0x74(a0)
        lw t5, 0x78(a0)
        lw t6, 0x7C(a0)
        "
    };
    (restore_fp) => {
        "
        fld ft0, 0x80(a0)
        fld ft1, 0x88(a0)
        fld ft2, 0x90(a0)
        fld ft3, 0x98(a0)
        fld ft4, 0xA0(a0)
        fld ft5, 0xA8(a0)
        fld ft6, 0xB0(a0)
        fld ft7, 0xB8(a0)
        fld fs0, 0xC0(a0)
        fld fs1, 0xC8(a0)
        fld fa0, 0xD0(a0)
        fld fa1, 0xD8(a0)
        fld fa2, 0xE0(a0)
        fld fa3, 0xE8(a0)
        fld fa4, 0xF0(a0)
        fld fa5, 0xF8(a0)
        fld fa6, 0x100(a0)
        fld fa7, 0x108(a0)
        fld fs2, 0x110(a0)
        fld fs3, 0x118(a0)
        fld fs4, 0x120(a0)
        fld fs5, 0x128(a0)
        fld fs6, 0x130(a0)
        fld fs7, 0x138(a0)
        fld fs8, 0x140(a0)
        fld fs9, 0x148(a0)
        fld fs10, 0x150(a0)
        fld fs11, 0x158(a0)
        fld ft8, 0x160(a0)
        fld ft9, 0x168(a0)
        fld ft10, 0x170(a0)
        fld ft11, 0x178(a0)
        "
    };
}

#[naked]
pub extern "C-unwind" fn save_context() -> Context {
    // No need to save caller-saved registers here.
    #[cfg(target_feature = "d")]
    unsafe {
        asm!(
            concat!(code!(save_gp), code!(save_fp), "ret"),
            options(noreturn)
        );
    }
    #[cfg(not(target_feature = "d"))]
    unsafe {
        asm!(concat!(code!(save_gp), "ret"), options(noreturn));
    }
}

#[naked]
pub unsafe extern "C" fn restore_context(ctx: &Context) -> ! {
    #[cfg(target_feature = "d")]
    unsafe {
        asm!(
            concat!(code!(restore_fp), code!(restore_gp), "lw a0, 0x28(a0)\nret"),
            options(noreturn)
        );
    }
    #[cfg(not(target_feature = "d"))]
    unsafe {
        asm!(
            concat!(code!(restore_gp), "lw a0, 0x28(a0)\nret"),
            options(noreturn)
        );
    }
}
