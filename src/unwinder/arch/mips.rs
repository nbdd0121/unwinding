use core::arch::asm;
use core::fmt;
use core::ops;
use gimli::Register;

#[derive(Debug, Clone, Copy)]
pub struct MIPS;

impl MIPS {
    const ZERO: Register = Register(0);
    const AT: Register = Register(1);
    const V0: Register = Register(2);
    const V1: Register = Register(3);
    const A0: Register = Register(4);
    const A1: Register = Register(5);
    const A2: Register = Register(6);
    const A3: Register = Register(7);
    const T0: Register = Register(8);
    const T1: Register = Register(9);
    const T2: Register = Register(10);
    const T3: Register = Register(11);
    const T4: Register = Register(12);
    const T5: Register = Register(13);
    const T6: Register = Register(14);
    const T7: Register = Register(15);
    const S0: Register = Register(16);
    const S1: Register = Register(17);
    const S2: Register = Register(18);
    const S3: Register = Register(19);
    const S4: Register = Register(20);
    const S5: Register = Register(21);
    const S6: Register = Register(22);
    const S7: Register = Register(23);
    const T8: Register = Register(24);
    const T9: Register = Register(25);
    const K0: Register = Register(26);
    const K1: Register = Register(27);
    const GP: Register = Register(28);
    const SP: Register = Register(29);
    const FP: Register = Register(30);
    const RA: Register = Register(31);
    const F0: Register = Register(32);
    const F1: Register = Register(33);
    const F2: Register = Register(34);
    const F3: Register = Register(35);
    const F4: Register = Register(36);
    const F5: Register = Register(37);
    const F6: Register = Register(38);
    const F7: Register = Register(39);
    const F8: Register = Register(40);
    const F9: Register = Register(41);
    const F10: Register = Register(42);
    const F11: Register = Register(43);
    const F12: Register = Register(44);
    const F13: Register = Register(45);
    const F14: Register = Register(46);
    const F15: Register = Register(47);
    const F16: Register = Register(48);
    const F17: Register = Register(49);
    const F18: Register = Register(50);
    const F19: Register = Register(51);
    const F20: Register = Register(52);
    const F21: Register = Register(53);
    const F22: Register = Register(54);
    const F23: Register = Register(55);
    const F24: Register = Register(56);
    const F25: Register = Register(57);
    const F26: Register = Register(58);
    const F27: Register = Register(59);
    const F28: Register = Register(60);
    const F29: Register = Register(61);
    const F30: Register = Register(62);
    const F31: Register = Register(63);

    pub fn register_name(register: Register) -> Option<&'static str> {
        match register {
            Self::ZERO => Some("$zero"),
            Self::AT => Some("$at"),
            Self::V0 => Some("$v0"),
            Self::V1 => Some("$v1"),
            Self::A0 => Some("$a0"),
            Self::A1 => Some("$a1"),
            Self::A2 => Some("$a2"),
            Self::A3 => Some("$a3"),
            Self::T0 => Some("$t0"),
            Self::T1 => Some("$t1"),
            Self::T2 => Some("$t2"),
            Self::T3 => Some("$t3"),
            Self::T4 => Some("$t4"),
            Self::T5 => Some("$t5"),
            Self::T6 => Some("$t6"),
            Self::T7 => Some("$t7"),
            Self::S0 => Some("$s0"),
            Self::S1 => Some("$s1"),
            Self::S2 => Some("$s2"),
            Self::S3 => Some("$s3"),
            Self::S4 => Some("$s4"),
            Self::S5 => Some("$s5"),
            Self::S6 => Some("$s6"),
            Self::S7 => Some("$s7"),
            Self::T8 => Some("$t8"),
            Self::T9 => Some("$t9"),
            Self::K0 => Some("$k0"),
            Self::K1 => Some("$k1"),
            Self::GP => Some("$gp"),
            Self::SP => Some("$sp"),
            Self::FP => Some("$fp"),
            Self::RA => Some("$ra"),
            Self::F0 => Some("$f0"),
            Self::F1 => Some("$f1"),
            Self::F2 => Some("$f2"),
            Self::F3 => Some("$f3"),
            Self::F4 => Some("$f4"),
            Self::F5 => Some("$f5"),
            Self::F6 => Some("$f6"),
            Self::F7 => Some("$f7"),
            Self::F8 => Some("$f8"),
            Self::F9 => Some("$f9"),
            Self::F10 => Some("$f10"),
            Self::F11 => Some("$f11"),
            Self::F12 => Some("$f12"),
            Self::F13 => Some("$f13"),
            Self::F14 => Some("$f14"),
            Self::F15 => Some("$f15"),
            Self::F16 => Some("$f16"),
            Self::F17 => Some("$f17"),
            Self::F18 => Some("$f18"),
            Self::F19 => Some("$f19"),
            Self::F20 => Some("$f20"),
            Self::F21 => Some("$f21"),
            Self::F22 => Some("$f22"),
            Self::F23 => Some("$f23"),
            Self::F24 => Some("$f24"),
            Self::F25 => Some("$f25"),
            Self::F26 => Some("$f26"),
            Self::F27 => Some("$f27"),
            Self::F28 => Some("$f28"),
            Self::F29 => Some("$f29"),
            Self::F30 => Some("$f30"),
            Self::F31 => Some("$f31"),
            _ => return None,
        }
    }
}

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
        for i in 32..=63 {
            fmt.field(
                MIPS::register_name(Register(i as _)).unwrap(),
                &self.fp[i - 32],
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
        sw $ra,     0x7C($sp)
        sw $gp,     0x70($sp)
        sw $v0,     0x08($sp)
        sw $v1,     0x0C($sp)
        sw $a0,     0x10($sp)
        sw $a1,     0x14($sp)
        sw $a2,     0x18($sp)
        sw $a3,     0x1C($sp)
        sw $t0,     0x20($sp)
        sw $t1,     0x24($sp)
        sw $t2,     0x28($sp)
        sw $t3,     0x2C($sp)
        sw $t4,     0x30($sp)
        sw $t5,     0x34($sp)
        sw $t6,     0x38($sp)
        sw $t7,     0x3C($sp)
        sw $s0,     0x40($sp)
        sw $s1,     0x44($sp)
        sw $s2,     0x48($sp)
        sw $s3,     0x4C($sp)
        sw $s4,     0x50($sp)
        sw $s5,     0x54($sp)
        sw $s6,     0x58($sp)
        sw $s7,     0x5C($sp)
        sw $t8,     0x60($sp)
        sw $t9,     0x64($sp)
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
        lw $ra,     0x7C($a0)
        lw $gp,     0x70($a0)
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
            move $t0, $sp
            add $sp, $sp, -0x100
            ",
            code!(save_gp),
            code!(save_fp),
            "
            move $t0, $a0
            move $a0, $sp
            jalr $t0
            lw $ra, 0x7C($sp)
            add $sp, $sp, 0x100
            jr $ra
            ",
            options(noreturn)
        );
        #[cfg(not(target_feature = "single-float"))]
        asm!(
            "
            move $t0, $sp
            add $sp, $sp, -0x80
            ",
            code!(save_gp),
            "
            move $t0, $a0
            move $a0, $sp
            jalr $t0
            lw $ra, 0x7C($sp)
            add $sp, $sp, 0x80
            jr $ra
            ",
            options(noreturn)
        );
    }
}

#[naked]
pub unsafe extern "C" fn restore_context(ctx: &Context) -> ! {
    unsafe {
        #[cfg(target_feature = "single-float")]
        asm!(
            code!(restore_fp),
            code!(restore_gp),
            "
            lw $a0, 0x10($sp)
            jr $ra
            ",
            options(noreturn)
        );
        #[cfg(not(target_feature = "single-float"))]
        asm!(
            code!(restore_gp),
            "
            lw $a0, 0x10($sp)
            jr $ra
            ",
            options(noreturn)
        );
    }
}
