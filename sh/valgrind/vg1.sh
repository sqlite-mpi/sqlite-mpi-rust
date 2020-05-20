#!/usr/bin/env bash
d=`gdate --utc --iso-8601=seconds`;
f="./out/${d}.txt";


# Run a Rust test function in Valgrind to check `unsafe` blocks interacting with FFI's does not leak memory.
# @see https://travisf.net/capstone-rs-unsafety-adventure#running-under-valgrind
# @see https://www.sqlite.org/c3ref/c_config_covering_index_scan.html#sqliteconfiglookaside (disable look aside to check memory leaks).

valgrind --leak-check=full \
         --show-leak-kinds=all \
         --track-origins=yes \
         --verbose \
         --log-file=${f} \
         --num-callers=500 \
         ./../../target/debug/deps/runtime-eac33d2eec1a26dc test_multiple_concurrent_runtimes --nocapture --test-threads=1
         # cargo test test_multiple_concurrent_runtimes  --package runtime -- --nocapture --test-threads=1