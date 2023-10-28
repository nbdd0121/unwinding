extern crate unwinding;

fn main() {
    let _ = unwinding::panic::catch_unwind(|| {
        panic!();
    });
}
