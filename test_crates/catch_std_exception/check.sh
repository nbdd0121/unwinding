#!/usr/bin/env bash
set -o pipefail
trap "rm -f run.log" EXIT
cargo run --release $BUILD_STD 2>&1 | tee run.log
if [ $? -ne 134 ]; then
    echo process is not aborted
    exit 1
fi
grep -Pz 'panicked at test_crates/catch_std_exception/src/main.rs:5:9:\nexplicit panic\nnote: run with `RUST_BACKTRACE=1` environment variable to display a backtrace' run.log
