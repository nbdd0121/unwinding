use super::arch::*;
use crate::abi::PersonalityRoutine;

#[derive(Debug)]
pub struct Frame {}

impl Frame {
    pub fn from_context(ctx: &Context, signal: bool) -> Result<Option<Self>, gimli::Error> {
        todo!()
    }

    pub fn unwind(&self, ctx: &Context) -> Result<Context, gimli::Error> {
        todo!()
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
        todo!()
    }

    pub fn is_signal_trampoline(&self) -> bool {
        todo!()
    }
}
