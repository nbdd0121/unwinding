mod phdr;
mod registry;

use crate::util::*;
use gimli::{BaseAddresses, EhFrame, FrameDescriptionEntry};

#[derive(Debug)]
pub struct FDESearchResult {
    pub fde: FrameDescriptionEntry<StaticSlice>,
    pub bases: BaseAddresses,
    pub eh_frame: EhFrame<StaticSlice>,
}

pub trait FDEFinder {
    fn find_fde(&self, pc: usize) -> Option<FDESearchResult>;
}

pub struct GlobalFinder(());

impl FDEFinder for GlobalFinder {
    fn find_fde(&self, pc: usize) -> Option<FDESearchResult> {
        if let Some(v) = registry::get_finder().find_fde(pc) {
            return Some(v);
        }
        if let Some(v) = phdr::get_finder().find_fde(pc) {
            return Some(v);
        }
        None
    }
}

pub fn get_finder() -> &'static GlobalFinder {
    &GlobalFinder(())
}
