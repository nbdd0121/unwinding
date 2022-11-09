use core::arch::asm;
use core::fmt;
use core::ops;
use gimli::{Arm, Register};

// Match FIRST_PSEUDO_REGISTER from GCC
pub const MAX_REG_RULES: usize = 107;

#[repr(C)]
#[derive(Clone, Default)]
pub struct Context {
    pub gp: [usize; 13],
    pub sp: usize,
    pub lr: usize,
    pub pc: usize,
}

impl ops::Index<Register> for Context {
    type Output = usize;

    fn index(&self, reg: Register) -> &usize {
        match reg {
            Register(0..=12) => &self.gp[reg.0 as usize],
            Arm::SP => &self.sp,
            Arm::LR => &self.lr,
            Arm::PC => &self.pc,
            _ => unimplemented!(),
        }
    }
}

impl ops::IndexMut<Register> for Context {
    fn index_mut(&mut self, reg: Register) -> &mut usize {
        match reg {
            Register(0..=12) => &mut self.gp[reg.0 as usize],
            Arm::SP => &mut self.sp,
            Arm::LR => &mut self.lr,
            Arm::PC => &mut self.pc,
            _ => unimplemented!(),
        }
    }
}

#[naked]
pub extern "C-unwind" fn save_context() -> Context {
    // No need to save caller-saved registers here. r9 register is somewhat
    // special as depending on the platform it may be calle-saved or not,
    // probably there is no way to detect whether it's calle-saved or not except
    // by having to support each possible platform which wouldn't work reliably
    // in no_std environments.
    //
    // TODO: support hard fp
    unsafe {
        asm!(
            "
            str r4, [r0, #0x10]
            str r5, [r0, #0x14]
            str r6, [r0, #0x18]
            str r7, [r0, #0x1C]
            str r8, [r0, #0x20]
            str r9, [r0, #0x24]
            str r10, [r0, #0x28]
            str r11, [r0, #0x2C]
            str r12, [r0, #0x30]
            str sp, [r0, #0x34]
            str lr, [r0, #0x38]
            str pc, [r0, #0x3C]
            bx lr
            ",
            options(noreturn)
        );
    }
}

//#[naked]
pub extern "C-unwind" fn restore_context(ctx: &Context) -> ! {
    todo!("restore_context")
}

#[no_mangle]
pub extern "C-unwind" fn __aeabi_unwind_cpp_pr0() {
    todo!()
}

#[no_mangle]
pub extern "C-unwind" fn __aeabi_unwind_cpp_pr1() {
    todo!()
}
