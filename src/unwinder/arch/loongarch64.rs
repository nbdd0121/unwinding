use core::{fmt, ops};
use gimli::{LoongArch, Register};

use super::maybe_cfi;

// https://github.com/loongson/la-abi-specs/blob/release/ladwarf.adoc
pub const MAX_REG_RULES: usize = 65;

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
                &format_args!("{:#x}", self.gp[i]),
            );
        }
        fmt.field("sp", &self.gp[3]);
        for i in 0..=31 {
            fmt.field(
                LoongArch::register_name(Register((i + 32) as _)).unwrap(),
                &format_args!("{:#x}", self.fp[i]),
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

macro_rules! save_regs {
    (gp$(, $fp:ident)?) => {
        core::arch::naked_asm!(
            maybe_cfi!(".cfi_startproc"),
            "
            move $r12, $r3
            addi.d $r3, $r3, -0x210
            ",
            maybe_cfi!(".cfi_def_cfa_offset 0x210"),
            "
            st.d $r1, $r3, 0x200
            ",
            maybe_cfi!(".cfi_offset 1, -16"), // ra
            helper!(save_gp),
            save_regs!(maybe_save_fp($($fp)?)),
            "
            move $r12, $r4
            move $r4, $r3 // argument a0 pointing the ctx base

            addi.d $r13, $r3, 0x210
            st.d $r13, $r3, 0x18 // save sp

            jirl $r1, $r12, 0 // call f

            ld.d $r1, $r3, 0x200
            addi.d $r3, $r3, 0x210 // restore ra and sp
            ",
            maybe_cfi!(".cfi_def_cfa_offset 0"),
            maybe_cfi!(".cfi_restore 1"), // ra
            "
            ret
            ",
            maybe_cfi!(".cfi_endproc"),
        )
    };
    (maybe_save_fp(fp)) => {
        helper!(save_fp)
    };
    (maybe_save_fp()) => {
        ""
    };
}

macro_rules! restore_regs {
    ($ctx:expr, gp$(, $fp:ident)?) => {
        core::arch::asm!(
            restore_regs!(maybe_restore_fp($($fp)?)),
            helper!(restore_gp),
            "
            ld.d $r4, $r4, 0x20
            ret
            ",
            in("$r4") $ctx,
            options(noreturn)
        )
    };
    (maybe_restore_fp(fp)) => {
        helper!(restore_fp)
    };
    (maybe_restore_fp()) => {
        ""
    };
}

macro_rules! helper {
    // see https://loongson.github.io/LoongArch-Documentation/LoongArch-ELF-ABI-EN.html for ABI conventions
    (save_gp) => {
        "
        st.d $r0, $r3, 0x0 // zero
        st.d $r1, $r3, 0x8 // ra
        st.d $r2, $r3, 0x10 // tp
        // sp is saved later
        st.d $r21, $r3, 0xa8 // reserved
        st.d $r22, $r3, 0xb0 // fp
        st.d $r23, $r3, 0xb8 // s0
        st.d $r24, $r3, 0xc0 // s1
        st.d $r25, $r3, 0xc8 // s2
        st.d $r26, $r3, 0xd0 // s3
        st.d $r27, $r3, 0xd8 // s4
        st.d $r28, $r3, 0xe0 // s5
        st.d $r29, $r3, 0xe8 // s6
        st.d $r30, $r3, 0xf0 // s7
        st.d $r31, $r3, 0xf8 // s8
        "
    };
    (save_fp) => {
        "
        fst.d $f24, $r3, 0x1c0 // fs0
        fst.d $f25, $r3, 0x1c8 // fs1
        fst.d $f26, $r3, 0x1d0 // fs2
        fst.d $f27, $r3, 0x1d8 // fs3
        fst.d $f28, $r3, 0x1e0 // fs4
        fst.d $f29, $r3, 0x1e8 // fs5
        fst.d $f30, $r3, 0x1f0 // fs6
        fst.d $f31, $r3, 0x1f8 // fs7
        "
    };
    (restore_gp) => {
        "
        ld.d $r1, $r4, 0x8 // ra
        ld.d $r2, $r4, 0x10 // tp
        ld.d $r3, $r4, 0x18 // sp
        // r4(a0) is restored later
        ld.d $r5, $r4, 0x28 // a1
        ld.d $r6, $r4, 0x30 // a2
        ld.d $r7, $r4, 0x38 // a3
        ld.d $r8, $r4, 0x40 // a4
        ld.d $r9, $r4, 0x48 // a5
        ld.d $r10, $r4, 0x50 // a6
        ld.d $r11, $r4, 0x58 // a7
        ld.d $r12, $r4, 0x60 // t0
        ld.d $r13, $r4, 0x68 // t1
        ld.d $r14, $r4, 0x70 // t2
        ld.d $r15, $r4, 0x78 // t3
        ld.d $r16, $r4, 0x80 // t4
        ld.d $r17, $r4, 0x88 // t5
        ld.d $r18, $r4, 0x90 // t6
        ld.d $r19, $r4, 0x98 // t7
        ld.d $r20, $r4, 0xa0 // t8
        ld.d $r21, $r4, 0xa8 // reserved
        ld.d $r22, $r4, 0xb0 // fp
        ld.d $r23, $r4, 0xb8 // s0
        ld.d $r24, $r4, 0xc0 // s1
        ld.d $r25, $r4, 0xc8 // s2
        ld.d $r26, $r4, 0xd0 // s3
        ld.d $r27, $r4, 0xd8 // s4
        ld.d $r28, $r4, 0xe0 // s5
        ld.d $r29, $r4, 0xe8 // s6
        ld.d $r30, $r4, 0xf0 // s7
        ld.d $r31, $r4, 0xf8 // s8
        "
    };
    (restore_fp) => {
        "
        fld.d $f0, $r4, 0x100 // fa0
        fld.d $f1, $r4, 0x108 // fa1
        fld.d $f2, $r4, 0x110 // fa2
        fld.d $f3, $r4, 0x118 // fa3
        fld.d $f4, $r4, 0x120 // fa4
        fld.d $f5, $r4, 0x128 // fa5
        fld.d $f6, $r4, 0x130 // fa6
        fld.d $f7, $r4, 0x138 // fa7
        fld.d $f8, $r4, 0x140 // ft0
        fld.d $f9, $r4, 0x148 // ft1
        fld.d $f10, $r4, 0x150 // ft2
        fld.d $f11, $r4, 0x158 // ft3
        fld.d $f12, $r4, 0x160 // ft4
        fld.d $f13, $r4, 0x168 // ft5
        fld.d $f14, $r4, 0x170 // ft6
        fld.d $f15, $r4, 0x178 // ft7
        fld.d $f16, $r4, 0x180 // ft8
        fld.d $f17, $r4, 0x188 // ft9
        fld.d $f18, $r4, 0x190 // ft10
        fld.d $f19, $r4, 0x198 // ft11
        fld.d $f20, $r4, 0x1a0 // ft12
        fld.d $f21, $r4, 0x1a8 // ft13
        fld.d $f22, $r4, 0x1b0 // ft14
        fld.d $f23, $r4, 0x1b8 // ft15
        fld.d $f24, $r4, 0x1c0 // fs0
        fld.d $f25, $r4, 0x1c8 // fs1
        fld.d $f26, $r4, 0x1d0 // fs2
        fld.d $f27, $r4, 0x1d8 // fs3
        fld.d $f28, $r4, 0x1e0 // fs4
        fld.d $f29, $r4, 0x1e8 // fs5
        fld.d $f30, $r4, 0x1f0 // fs6
        fld.d $f31, $r4, 0x1f8 // fs7
        "
    };
}

#[unsafe(naked)]
pub extern "C-unwind" fn save_context(f: extern "C" fn(&mut Context, *mut ()), ptr: *mut ()) {
    #[allow(unused_unsafe)]
    unsafe {
        #[cfg(target_feature = "d")]
        save_regs!(gp, fp);
        #[cfg(not(target_feature = "d"))]
        save_regs!(gp);
    }
}

pub unsafe fn restore_context(ctx: &Context) -> ! {
    unsafe {
        #[cfg(target_feature = "d")]
        restore_regs!(ctx, gp, fp);
        #[cfg(not(target_feature = "d"))]
        restore_regs!(ctx, gp);
    }
}
