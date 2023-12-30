#!/usr/bin/env bash
set -o pipefail
trap "rm -f run.log" EXIT
${CARGO:-cargo} run --release $BUILD_STD 2>&1 | tee run.log
if [ $? -ne 134 ]; then
    echo process is not aborted
    exit 1
fi
grep -Pz 'fatal runtime error: Rust cannot catch foreign exceptions' run.log
