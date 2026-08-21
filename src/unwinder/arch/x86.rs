use core::fmt;
use gimli::{Register, X86};

use super::maybe_cfi;

// Match DWARF_FRAME_REGISTERS in libgcc
pub const MAX_REG_RULES: usize = 17;

#[repr(C)]
#[derive(Clone, Default)]
pub struct Context {
    pub registers: [usize; 8],
    pub ra: usize,
    pub mcxsr: usize,
    pub fcw: usize,
}

impl fmt::Debug for Context {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fmt = fmt.debug_struct("Context");
        for i in 0..=7 {
            fmt.field(
                X86::register_name(Register(i as _)).unwrap(),
                &self.registers[i],
            );
        }
        fmt.field("ra", &self.ra)
            .field("mcxsr", &self.mcxsr)
            .field("fcw", &self.fcw)
            .finish()
    }
}

impl Context {
    pub fn get(&self, reg: Register) -> Option<&usize> {
        match reg {
            Register(0..=7) => Some(&self.registers[reg.0 as usize]),
            X86::RA => Some(&self.ra),
            X86::MXCSR => Some(&self.mcxsr),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, reg: Register) -> Option<&mut usize> {
        match reg {
            Register(0..=7) => Some(&mut self.registers[reg.0 as usize]),
            X86::RA => Some(&mut self.ra),
            X86::MXCSR => Some(&mut self.mcxsr),
            _ => None,
        }
    }
}

#[unsafe(naked)]
pub extern "C-unwind" fn save_context(f: extern "C" fn(&mut Context, *mut ()), ptr: *mut ()) {
    // No need to save caller-saved registers here.
    core::arch::naked_asm!(
        maybe_cfi!(".cfi_startproc"),
        "sub esp, 52",
        maybe_cfi!(".cfi_def_cfa_offset 56"),
        "
        mov [esp + 4], ecx
        mov [esp + 8], edx
        mov [esp + 12], ebx

        /* Adjust the stack to account for the return address */
        lea eax, [esp + 56]
        mov [esp + 16], eax

        mov [esp + 20], ebp
        mov [esp + 24], esi
        mov [esp + 28], edi

        /* Return address */
        mov eax, [esp + 52]
        mov [esp + 32], eax

        stmxcsr [esp + 36]
        fnstcw [esp + 40]

        mov eax, [esp + 60]
        mov ecx, esp
        push eax
        ",
        maybe_cfi!(".cfi_adjust_cfa_offset 4"),
        "push ecx",
        maybe_cfi!(".cfi_adjust_cfa_offset 4"),
        "
        call [esp + 64]

        add esp, 60
        ",
        maybe_cfi!(".cfi_def_cfa_offset 4"),
        "ret",
        maybe_cfi!(".cfi_endproc"),
    );
}

pub unsafe fn restore_context(ctx: &Context) -> ! {
    unsafe {
        core::arch::asm!(
            "
            /* Restore stack */
            mov esp, [edx + 16]

            /* Restore callee-saved control registers */
            ldmxcsr [edx + 36]
            fldcw [edx + 40]

            /* Restore return address */
            mov eax, [edx + 32]
            push eax

            /*
            * Restore general-purpose registers. Non-callee-saved registers are
            * also restored because sometimes it's used to pass unwind arguments.
            */
            mov eax, [edx + 0]
            mov ecx, [edx + 4]
            mov ebx, [edx + 12]
            mov ebp, [edx + 20]
            mov esi, [edx + 24]
            mov edi, [edx + 28]

            /* EDX restored last */
            mov edx, [edx + 8]

            ret
            ",
            in("edx") ctx,
            options(noreturn)
        );
    }
}
