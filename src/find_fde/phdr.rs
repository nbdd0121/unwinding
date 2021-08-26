use crate::find_fde::FDESearchResult;
use crate::util::*;

use core::ffi::c_void;
use core::mem;
use core::slice;
use gimli::{BaseAddresses, EhFrame, EhFrameHdr, NativeEndian, UnwindSection};
use libc::{dl_iterate_phdr, dl_phdr_info, PT_DYNAMIC, PT_GNU_EH_FRAME, PT_LOAD};

struct CallbackData {
    pc: usize,
    result: Option<FDESearchResult>,
}

pub struct PhdrFinder(());

pub fn get_finder() -> &'static PhdrFinder {
    &PhdrFinder(())
}

impl super::FDEFinder for PhdrFinder {
    fn find_fde(&self, pc: usize) -> Option<FDESearchResult> {
        let mut data = CallbackData { pc, result: None };
        unsafe { dl_iterate_phdr(Some(phdr_callback), &mut data as *mut CallbackData as _) };
        data.result
    }
}

unsafe extern "C" fn phdr_callback(
    info: *mut dl_phdr_info,
    _size: usize,
    data: *mut c_void,
) -> c_int {
    unsafe {
        let data = &mut *(data as *mut CallbackData);
        let phdrs = slice::from_raw_parts((*info).dlpi_phdr, (*info).dlpi_phnum as usize);

        let mut text = None;
        let mut eh_frame_hdr = None;
        let mut dynamic = None;

        for phdr in phdrs {
            let start = (*info).dlpi_addr + phdr.p_vaddr;
            match phdr.p_type {
                PT_LOAD => {
                    let end = start + phdr.p_memsz;
                    let range = start..end;
                    if range.contains(&(data.pc as _)) {
                        text = Some(range);
                    }
                }
                PT_GNU_EH_FRAME => {
                    eh_frame_hdr = Some(start);
                }
                PT_DYNAMIC => {
                    dynamic = Some(start);
                }
                _ => (),
            }
        }

        let text = match text {
            Some(v) => v,
            None => return 0,
        };

        let eh_frame_hdr = match eh_frame_hdr {
            Some(v) => v,
            None => return 0,
        };

        let mut bases = BaseAddresses::default()
            .set_eh_frame_hdr(eh_frame_hdr as _)
            .set_text(text.start as _);

        // Find the GOT section.
        if let Some(start) = dynamic {
            const DT_NULL: usize = 0;
            const DT_PLTGOT: usize = 3;

            let mut tags = start as *const [usize; 2];
            let mut tag = *tags;
            while tag[0] != DT_NULL {
                if tag[0] == DT_PLTGOT {
                    bases = bases.set_got(tag[1] as _);
                    break;
                }
                tags = tags.add(1);
                tag = *tags;
            }
        }

        // Parse .eh_frame_hdr section.
        let eh_frame_hdr = EhFrameHdr::new(
            get_unlimited_slice(eh_frame_hdr as usize as _),
            NativeEndian,
        )
        .parse(&bases, mem::size_of::<usize>() as _);
        let eh_frame_hdr = match eh_frame_hdr {
            Ok(v) => v,
            Err(_) => return 0,
        };

        let eh_frame = deref_pointer(eh_frame_hdr.eh_frame_ptr());
        bases = bases.set_eh_frame(eh_frame as _);
        let eh_frame = EhFrame::new(get_unlimited_slice(eh_frame as usize as _), NativeEndian);

        // Use binary search table for address if available.
        if let Some(table) = eh_frame_hdr.table() {
            if let Ok(fde) =
                table.fde_for_address(&eh_frame, &bases, data.pc as _, EhFrame::cie_from_offset)
            {
                data.result = Some(FDESearchResult {
                    fde,
                    bases,
                    eh_frame,
                });
                return 1;
            }
        }

        // Otherwise do the linear search.
        if let Ok(fde) = eh_frame.fde_for_address(&bases, data.pc as _, EhFrame::cie_from_offset) {
            data.result = Some(FDESearchResult {
                fde,
                bases,
                eh_frame,
            });
            return 1;
        }

        0
    }
}
