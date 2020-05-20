#!/usr/bin/env bash
cargo watch -s "clear && printf '\e[3J'; cargo test test_get_db_handle --package sqlite3ffi -- --nocapture"
