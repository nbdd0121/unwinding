use core::fmt::{self, Write};
use core::ops;
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

macro_rules! ctx_helper {
    (savegp) => {
        "
        st.d $r0, $r3, 0x0
        st.d $r1, $r3, 0x8
        st.d $r2, $r3, 0x10
        st.d $r3, $r3, 0x18
        st.d $r22, $r3, 0xB0
        st.d $r23, $r3, 0xB8
        st.d $r24, $r3, 0xC0
        st.d $r25, $r3, 0xC8
        st.d $r26, $r3, 0xD0
        st.d $r27, $r3, 0xD8
        st.d $r28, $r3, 0xE0
        st.d $r29, $r3, 0xE8
        st.d $r30, $r3, 0xF0
        st.d $r31, $r3, 0xF8
        "
    };
    (savefp) => {
        "
        fst.d $f24, $r3, 0x1C0
        fst.d $f25, $r3, 0x1C8
        fst.d $f26, $r3, 0x1D0
        fst.d $f27, $r3, 0x1D8
        fst.d $f28, $r3, 0x1E0
        fst.d $f29, $r3, 0x1E8
        fst.d $f30, $r3, 0x1F0
        fst.d $f31, $r3, 0x1F8
        "
    };
    (restoregp) => {
        "
        ld.d $r1, $r4, 0x8
        ld.d $r2, $r4, 0x10
        ld.d $r3, $r4, 0x18
        ld.d $r5, $r4, 0x28
        ld.d $r6, $r4, 0x30
        ld.d $r7, $r4, 0x38
        ld.d $r8, $r4, 0x40
        ld.d $r9, $r4, 0x48
        ld.d $r10, $r4, 0x50
        ld.d $r11, $r4, 0x58
        ld.d $r12, $r4, 0x60
        ld.d $r13, $r4, 0x68
        ld.d $r14, $r4, 0x70
        ld.d $r15, $r4, 0x78
        ld.d $r16, $r4, 0x80
        ld.d $r17, $r4, 0x88
        ld.d $r18, $r4, 0x90
        ld.d $r19, $r4, 0x98
        ld.d $r20, $r4, 0xA0
        ld.d $r21, $r4, 0xA8
        ld.d $r22, $r4, 0xB0
        ld.d $r23, $r4, 0xB8
        ld.d $r24, $r4, 0xC0
        ld.d $r25, $r4, 0xC8
        ld.d $r26, $r4, 0xD0
        ld.d $r27, $r4, 0xD8
        ld.d $r28, $r4, 0xE0
        ld.d $r29, $r4, 0xE8
        ld.d $r30, $r4, 0xF0
        ld.d $r31, $r4, 0xF8
        "
    };
    (restorefp) => {
        "
        fld.d $f0, $r4, 0x100
        fld.d $f1, $r4, 0x108
        fld.d $f2, $r4, 0x110
        fld.d $f3, $r4, 0x118
        fld.d $f4, $r4, 0x120
        fld.d $f5, $r4, 0x128
        fld.d $f6, $r4, 0x130
        fld.d $f7, $r4, 0x138
        fld.d $f8, $r4, 0x140
        fld.d $f9, $r4, 0x148
        fld.d $f10, $r4, 0x150
        fld.d $f11, $r4, 0x158
        fld.d $f12, $r4, 0x160
        fld.d $f13, $r4, 0x168
        fld.d $f14, $r4, 0x170
        fld.d $f15, $r4, 0x178
        fld.d $f16, $r4, 0x180
        fld.d $f17, $r4, 0x188
        fld.d $f18, $r4, 0x190
        fld.d $f19, $r4, 0x198
        fld.d $f20, $r4, 0x1A0
        fld.d $f21, $r4, 0x1A8
        fld.d $f22, $r4, 0x1B0
        fld.d $f23, $r4, 0x1B8
        fld.d $f24, $r4, 0x1C0
        fld.d $f25, $r4, 0x1C8
        fld.d $f26, $r4, 0x1D0
        fld.d $f27, $r4, 0x1D8
        fld.d $f28, $r4, 0x1E0
        fld.d $f29, $r4, 0x1E8
        fld.d $f30, $r4, 0x1F0
        fld.d $f31, $r4, 0x1F8
        "
    };
}

#[unsafe(naked)]
pub extern "C-unwind" fn save_context(f: extern "C" fn(&mut Context, *mut ()), ptr: *mut ()) {
    #[allow(unused_unsafe)]
    unsafe {
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
            maybe_cfi!(".cfi_offset $r1, -16"),
            ctx_helper!(savegp),
            ctx_helper!(savefp),
            "
            move $r12, $r4
            move $r4, $r3
            jirl $r1, $r12, 0
            ld.d $r1, $r3, 0x200
            addi.d $r3, $r3, 0x210
            ",
            maybe_cfi!(".cfi_def_cfa_offset 0"),
            maybe_cfi!(".cfi_restore $r1"),
            "
            ret
            ",
            maybe_cfi!(".cfi_endproc"),
        )
    }
}

pub unsafe fn restore_context(ctx: &Context) -> ! {
    unsafe {
        core::arch::asm!(
            ctx_helper!(restoregp),
            ctx_helper!(restorefp),
            "
            ld.d $r4, $r4, 0x20
            ret
            ",
            in("$r4") ctx,
            options(noreturn)
        );
    }
}
