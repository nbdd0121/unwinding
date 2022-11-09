#[repr(C)]
pub struct IndexEntry {
    offset: u32,
    pub data: u32,
}

impl IndexEntry {
    pub fn addr(&self) -> usize {
        assert!(self.offset & 1 << 31 == 0);
        let offset = self.offset | (self.offset & (1 << 30)) << 1;

        (self as *const _ as usize).wrapping_add(offset as usize)
    }
}
