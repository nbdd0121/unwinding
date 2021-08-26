use crate::print::*;
use alloc::boxed::Box;
use core::any::Any;
use core::cell::Cell;
use core::panic::{Location, PanicInfo};

#[thread_local]
static PANIC_COUNT: Cell<usize> = Cell::new(0);

#[link(name = "c")]
extern "C" {}

pub(crate) fn drop_panic() {
    eprintln!("Rust panics must be rethrown");
}

pub(crate) fn foreign_exception() {
    eprintln!("Rust cannot catch foreign exceptions");
}

pub(crate) fn panic_caught() {
    PANIC_COUNT.set(0);
}

fn do_panic(msg: Box<dyn Any + Send>) -> ! {
    if PANIC_COUNT.get() >= 1 {
        eprintln!("thread panicked while processing panic. aborting.");
        core::intrinsics::abort();
    }
    PANIC_COUNT.set(1);
    let code = crate::panic::begin_panic(Box::new(msg));
    eprintln!("failed to initiate panic, error {}", code.0);
    core::intrinsics::abort();
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    eprintln!("{}", info);

    struct NoPayload;
    do_panic(Box::new(NoPayload))
}

#[track_caller]
pub fn panic_any<M: 'static + Any + Send>(msg: M) -> ! {
    eprintln!("panicked at {}", Location::caller());
    do_panic(Box::new(msg))
}
