#!/usr/bin/env bash
# Issue: `cargo test` when in a workspace package will re-compile all C FFI code even if it has not changed.
# Solution: Run `cargo test` in the workspace root dir, where the cache seems to prevent re-compiling all C code.

#cargo test create_db_file_test --package sqlite3ffi
#cargo watch -x "test test_get_handle --package sqlite3ffi"
#cargo watch -s "clear && printf '\e[3J'; cargo test tests:: --package sqlite3ffi -- --nocapture"

#cargo watch -s "clear && printf '\e[3J'; cargo test test_get_db_handle --package sqlite3ffi -- --nocapture"


#cargo watch -s "clear && printf '\e[3J'; cargo test test_get_stmt_handle_err --package sqlite3ffi -- --nocapture"
#cargo watch -s "clear && printf '\e[3J'; cargo test test_get_stmt_handle_ok --package sqlite3ffi -- --nocapture"
cargo watch -s "clear && printf '\e[3J'; cargo test test_ --package sma -- --nocapture"
#cargo watch -s "clear && printf '\e[3J'; cargo test test_read_tx --package sma -- --nocapture"