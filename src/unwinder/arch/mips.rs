use core::arch::asm;
use core::fmt;
use core::ops;
use gimli::{Register, MIPS};

// Match DWARF_FRAME_REGISTERS in libgcc
pub const MAX_REG_RULES: usize = 188;

#[repr(C)]
#[derive(Clone, Default)]
pub struct Context {
    pub gp: [usize; 32],
    #[cfg(target_feature = "single-float")]
    pub fp: [usize; 32],
}

impl fmt::Debug for Context {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fmt = fmt.debug_struct("Context");
        for i in 0..=31 {
            fmt.field(MIPS::register_name(Register(i as _)).unwrap(), &self.gp[i]);
        }
        #[cfg(target_feature = "single-float")]
        for i in 0..=31 {
            fmt.field(
                MIPS::register_name(Register((i + 32) as _)).unwrap(),
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
            #[cfg(target_feature = "single-float")]
            Register(32..=63) => &self.fp[(reg.0 - 32) as usize],
            _ => unimplemented!(),
        }
    }
}

impl ops::IndexMut<gimli::Register> for Context {
    fn index_mut(&mut self, reg: Register) -> &mut usize {
        match reg {
            Register(0..=31) => &mut self.gp[reg.0 as usize],
            #[cfg(target_feature = "single-float")]
            Register(32..=63) => &mut self.fp[(reg.0 - 32) as usize],
            _ => unimplemented!(),
        }
    }
}

macro_rules! code {
    (save_gp) => {
        "
        sw $zero,   0x00($sp)
        sw $s0,     0x40($sp)
        sw $s1,     0x44($sp)
        sw $s2,     0x48($sp)
        sw $s3,     0x4C($sp)
        sw $s4,     0x50($sp)
        sw $s5,     0x54($sp)
        sw $s6,     0x58($sp)
        sw $s7,     0x5C($sp)
        sw $k0,     0x68($sp)
        sw $k1,     0x6C($sp)
        sw $gp,     0x70($sp)
        sw $t0,     0x74($sp)
        sw $fp,     0x78($sp)
        sw $ra,     0x7C($sp)
        "
    };
    (save_fp) => {
        "
        swc1 $f0,   0x80($sp)
        swc1 $f1,   0x84($sp)
        swc1 $f2,   0x88($sp)
        swc1 $f3,   0x8C($sp)
        swc1 $f4,   0x90($sp)
        swc1 $f5,   0x94($sp)
        swc1 $f6,   0x98($sp)
        swc1 $f7,   0x9C($sp)
        swc1 $f8,   0xA0($sp)
        swc1 $f9,   0xA4($sp)
        swc1 $f10,  0xA8($sp)
        swc1 $f11,  0xAC($sp)
        swc1 $f12,  0xB0($sp)
        swc1 $f13,  0xB4($sp)
        swc1 $f14,  0xB8($sp)
        swc1 $f15,  0xBC($sp)
        swc1 $f16,  0xC0($sp)
        swc1 $f17,  0xC4($sp)
        swc1 $f18,  0xC8($sp)
        swc1 $f19,  0xCC($sp)
        swc1 $f20,  0xD0($sp)
        swc1 $f21,  0xD4($sp)
        swc1 $f22,  0xD8($sp)
        swc1 $f23,  0xDC($sp)
        swc1 $f24,  0xE0($sp)
        swc1 $f25,  0xE4($sp)
        swc1 $f26,  0xE8($sp)
        swc1 $f27,  0xEC($sp)
        swc1 $f28,  0xF0($sp)
        swc1 $f29,  0xF4($sp)
        swc1 $f30,  0xF8($sp)
        swc1 $f31,  0xFC($sp)
        "
    };
    (restore_gp) => {
        "
        lw $at,     0x04($a0)
        lw $v0,     0x08($a0)
        lw $v1,     0x0C($a0)
        lw $a1,     0x14($a0)
        lw $a2,     0x18($a0)
        lw $a3,     0x1C($a0)
        lw $t0,     0x20($a0)
        lw $t1,     0x24($a0)
        lw $t2,     0x28($a0)
        lw $t3,     0x2C($a0)
        lw $t4,     0x30($a0)
        lw $t5,     0x34($a0)
        lw $t6,     0x38($a0)
        lw $t7,     0x3C($a0)
        lw $s0,     0x40($a0)
        lw $s1,     0x44($a0)
        lw $s2,     0x48($a0)
        lw $s3,     0x4C($a0)
        lw $s4,     0x50($a0)
        lw $s5,     0x54($a0)
        lw $s6,     0x58($a0)
        lw $s7,     0x5C($a0)
        lw $t8,     0x60($a0)
        lw $t9,     0x64($a0)
        lw $k0,     0x68($a0)
        lw $k1,     0x6C($a0)
        lw $gp,     0x70($a0)
        lw $sp,     0x74($a0)
        lw $fp,     0x78($a0)
        lw $ra,     0x7C($a0)
        "
    };
    (restore_fp) => {
        "
        lwc1 $f0,   0x80($a0)
        lwc1 $f1,   0x84($a0)
        lwc1 $f2,   0x88($a0)
        lwc1 $f3,   0x8C($a0)
        lwc1 $f4,   0x90($a0)
        lwc1 $f5,   0x94($a0)
        lwc1 $f6,   0x98($a0)
        lwc1 $f7,   0x9C($a0)
        lwc1 $f8,   0xA0($a0)
        lwc1 $f9,   0xA4($a0)
        lwc1 $f10,  0xA8($a0)
        lwc1 $f11,  0xAC($a0)
        lwc1 $f12,  0xB0($a0)
        lwc1 $f13,  0xB4($a0)
        lwc1 $f14,  0xB8($a0)
        lwc1 $f15,  0xBC($a0)
        lwc1 $f16,  0xC0($a0)
        lwc1 $f17,  0xC4($a0)
        lwc1 $f18,  0xC8($a0)
        lwc1 $f19,  0xCC($a0)
        lwc1 $f20,  0xD0($a0)
        lwc1 $f21,  0xD4($a0)
        lwc1 $f22,  0xD8($a0)
        lwc1 $f23,  0xDC($a0)
        lwc1 $f24,  0xE0($a0)
        lwc1 $f25,  0xE4($a0)
        lwc1 $f26,  0xE8($a0)
        lwc1 $f27,  0xEC($a0)
        lwc1 $f28,  0xF0($a0)
        lwc1 $f29,  0xF4($a0)
        lwc1 $f30,  0xF8($a0)
        lwc1 $f31,  0xFC($a0)
        "
    };
}

#[naked]
pub extern "C-unwind" fn save_context(f: extern "C" fn(&mut Context, *mut ()), ptr: *mut ()) {
    unsafe {
        #[cfg(target_feature = "single-float")]
        asm!(
            "
            .set noreorder
            .set nomacro
            .set noat
            move $t0, $sp
            add $sp, $sp, -0x110
            sw $ra, 0x100($sp)
            ",
            code!(save_gp),
            code!(save_fp),
            "
            move $t9, $a0
            move $a0, $sp
            /* jalr must use $t9 in PIE code */
            jalr $t9
            nop
            lw $ra, 0x100($sp)
            add $sp, $sp, 0x110
            jr $ra
            nop
            .set at
            .set macro
            .set reorder
            ",
            options(noreturn)
        );
        #[cfg(not(target_feature = "single-float"))]
        asm!(
            "
            .set noreorder
            .set nomacro
            .set noat
            move $t0, $sp
            add $sp, $sp, -0x90
            sw $ra, 0x80($sp)
            ",
            code!(save_gp),
            "
            move $t9, $a0
            move $a0, $sp
            /* jalr must use $t9 in PIE code */
            jalr $t9
            nop
            lw $ra, 0x80($sp)
            add $sp, $sp, 0x90
            jr $ra
            nop
            .set at
            .set macro
            .set reorder
            ",
            options(noreturn)
        );
    }
}

pub unsafe extern "C" fn restore_context(ctx: &Context) -> ! {
    unsafe {
        #[cfg(target_feature = "single-float")]
        asm!(
            "
            .set noreorder
            .set nomacro
            .set noat
            ",
            code!(restore_fp),
            code!(restore_gp),
            "
            lw $a0, 0x10($a0)
            jr $ra
            nop
            .set at
            .set macro
            .set reorder
            ",
            in("$4") ctx,
            options(noreturn)
        );
        #[cfg(not(target_feature = "single-float"))]
        asm!(
            "
            .set noreorder
            .set nomacro
            .set noat
            ",
            code!(restore_gp),
            "
            lw $a0, 0x10($a0)
            jr $ra
            nop
            .set at
            .set macro
            .set reorder
            ",
            in("$4") ctx,
            options(noreturn)
        );
    }
}
