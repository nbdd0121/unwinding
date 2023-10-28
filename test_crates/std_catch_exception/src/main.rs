extern crate unwinding;

fn main() {
    let _ = std::panic::catch_unwind(|| {
        unwinding::panic::begin_panic(Box::new("test"));
    });
}
