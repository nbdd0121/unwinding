Unwinding library in Rust and for Rust
======================================

This library serves two purposes:
1. Provide a pure Rust alternative to libgcc_eh or libunwind.
2. Provide easier unwinding support for `#![no_std]` targets.

## Unwinder

The unwinder can be enabled with `unwinder` feature. Here are the feature gates related to
the unwinder:
| Feature      | Default | Description |
|--------------|---------|-|
| unwinder     | Yes     | The primary feature gate to enable the unwinder |
| fde-phdr     | Yes     | Use `dl_iterator_phdr` to retrieve frame unwind table. Depends on libc. |
| fde-registry | Yes     | Provide `__register__frame` and others for dynamic registration |
| dwarf-expr   | Yes     | Enable the dwarf expression evaluator. Usually not necessary for Rust |
| hide-trace   | Yes     | Hide unwinder frames in back trace |

If you want to use the unwinder for other Rust (C++, or any programs that utilize the unwinder), you can build the [`unwind_dyn`](cdylib) crate provided, and use `LD_PRELOAD` to replace the system unwinder with it.
```sh
cd cdylib
cargo build --release
# Test the unwinder using rustc. Why not :)
LD_PRELOAD=`../target/release/libunwind_dyn.so` rustc +nightly -Ztreat-err-as-bug
```

If you want to link to the unwinder in a Rust binary, simply add
```rust
extern crate unwind;
```

## Personality and other utilities

The library also provides Rust personality function. This can be handy if you are working on a `#![no_std]` binary/staticlib/cdylib and you still want unwinding support.

Here are the feature gates related:
| Feature       | Default | Description |
|---------------|---------|-|
| personality   | No      | Provides `#[lang = eh_personality]` |
| print         | No      | Provides `(e)?print(ln)?`. This is really only here because panic handler needs to provide things. Depends on libc. |
| panic         | No      | Provides `begin_panic` and `catch_unwind`. Only stack unwinding functionality is provided and no printing is done, because this feature does not depend on libc. |
| panic-handler | No      | Provides `#[panic_handler]`. Provides similar behaviour on panic to std, with `RUST_BACKTRACE` support as well. Stack trace won't have symbols though. Depends on libc. |
| system-alloc  | No      | Provides a global allocator which calls `malloc` and friends. Provided for convience. |

If you are writing a `#![no_std]` program, simply enable `personality`, `panic-handler` and `system-alloc` in addition to the defaults, you instantly obtains the ability to do unwinding! An example is given in [`example/`](example).
