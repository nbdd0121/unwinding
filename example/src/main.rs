#![no_std]
#![feature(start)]
#![feature(default_alloc_error_handler)]

extern crate alloc;
extern crate unwind;

use alloc::{borrow::ToOwned, string::String};
use unwind::print::*;

#[link(name = "c")]
extern "C" {}

struct PrintOnDrop(String);

impl Drop for PrintOnDrop {
    fn drop(&mut self) {
        println!("dropped: {:?}", self.0);
    }
}

struct PanicOnDrop;

impl Drop for PanicOnDrop {
    fn drop(&mut self) {
        panic!("panic on drop");
    }
}

fn foo() {
    panic!("panic");
}

fn bar() {
    let _p = PrintOnDrop("string".to_owned());
    foo()
}

fn main() {
    let _ = unwind::panic::catch_unwind(|| {
        bar();
        println!("done");
    });
    println!("caught");
    let _p = PanicOnDrop;
    foo();
}

#[start]
fn start(_argc: isize, _argv: *const *const u8) -> isize {
    unwind::panic::catch_unwind(|| {
        main();
        0
    })
    .unwrap_or(101)
}
