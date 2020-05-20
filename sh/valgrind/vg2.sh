#!/usr/bin/env bash
d=`gdate --utc --iso-8601=seconds`;
f="./out/${d}.txt";


# Run a Rust test function in Valgrind to check `unsafe` blocks interacting with FFI's does not leak memory.
# @see https://travisf.net/capstone-rs-unsafety-adventure#running-under-valgrind
# @see https://www.sqlite.org/c3ref/c_config_covering_index_scan.html#sqliteconfiglookaside (disable look aside to check memory leaks).

valgrind --leak-check=full \
        --errors-for-leak-kinds=definite \
         --show-leak-kinds=all \
         --track-origins=yes \
         --error-exitcode=1 \
         ./target/debug/deps/sqlite3ffi-f56cfaa70cb555f1 test_get_db_handle --nocapture --test-threads=1;