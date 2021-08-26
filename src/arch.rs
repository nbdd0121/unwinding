#[cfg(target_arch = "x86_64")]
mod x86_64 {
    use gimli::{Register, X86_64};

    pub struct Arch;

    #[allow(unused)]
    impl Arch {
        pub const SP: Register = X86_64::RSP;
        pub const RA: Register = X86_64::RA;

        pub const UNWIND_DATA_REG: (Register, Register) = (X86_64::RAX, X86_64::RDX);
        pub const UNWIND_PRIVATE_DATA_SIZE: usize = 6;
    }
}
#[cfg(target_arch = "x86_64")]
pub use x86_64::*;

#[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))]
mod riscv {
    use gimli::{Register, RiscV};

    pub struct Arch;

    #[allow(unused)]
    impl Arch {
        pub const SP: Register = RiscV::SP;
        pub const RA: Register = RiscV::RA;

        pub const UNWIND_DATA_REG: (Register, Register) = (RiscV::A0, RiscV::A1);
        pub const UNWIND_PRIVATE_DATA_SIZE: usize = 2;
    }
}
#[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))]
pub use riscv::*;
