use super::types::IndexEntry;

extern "C" {
    fn __exidx_start();
    fn __exidx_end();
    static __executable_start: u8;
    static __etext: u8;
}

pub fn find_exidx() -> Option<&'static [IndexEntry]> {
    let base = __exidx_start as usize;
    let len = __exidx_end as usize - base;
    assert!(len > 0);
    assert!(len % 8 == 0);

    let exidx = unsafe { core::slice::from_raw_parts(base as *const IndexEntry, len / 8) };
    Some(exidx)
}

pub fn find_entry_for_pc(table: &[IndexEntry], pc: usize) -> Option<&IndexEntry> {
    unsafe {
        let text_start = &__executable_start as *const u8 as usize;
        let text_end = &__etext as *const u8 as usize;
        if !(text_start..text_end).contains(&pc) {
            return None;
        }

        table.iter().rev().find(|x| x.addr() <= pc)
    }
}
