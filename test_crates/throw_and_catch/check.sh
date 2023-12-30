#!/usr/bin/env bash
set -o pipefail
trap "rm -f run.log" EXIT
${CARGO:-cargo} run --release $BUILD_STD 2>&1 | tee run.log
if [ $? -ne 134 ]; then
    echo process is not aborted
    exit 1
fi
grep -Pz 'panicked at test_crates/throw_and_catch/src/main.rs:36:5:\npanic\nnote: run with `RUST_BACKTRACE=1` environment variable to display a backtrace\ndropped: "string"\ncaught\npanicked at test_crates/throw_and_catch/src/main.rs:46:5:\npanic\npanicked at test_crates/throw_and_catch/src/main.rs:25:9:\npanic on drop\n( *\d+:.*\n)+thread panicked while processing panic\. aborting\.' run.log

