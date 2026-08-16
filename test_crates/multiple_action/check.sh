#!/usr/bin/env bash
set -o pipefail
trap "rm -f run.log" EXIT
${CARGO:-cargo} run --release $BUILD_STD 2>&1 | tee run.log
if [ $? -ne 0 ]; then
    echo process did not exit successfully
    exit 1
fi
grep -Pz 'drop executed' run.log
