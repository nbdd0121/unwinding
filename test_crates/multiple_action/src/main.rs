#![no_std]
#![no_main]

use unwinding::println;

extern crate unwinding;
extern crate alloc;

struct Guard;

impl Drop for Guard {
    fn drop(&mut self) {
        println!("drop executed");
    }
}

#[inline(never)]
fn work() {
    if core::hint::black_box(true) {
        unwinding::panic::begin_panic(alloc::boxed::Box::new(()));
    }

    println!("unreachable");
}

#[unsafe(no_mangle)]
extern "C-unwind" fn main() -> i32 {
    let _ = unwinding::panic::catch_unwind(|| {
        let _guard = Guard;
        work()
    });

    0
}
