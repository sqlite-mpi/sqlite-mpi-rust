#!/usr/bin/env bash
export RUST_BACKTRACE=1;
cargo watch -s "clear && printf '\e[3J'; cargo test --package smpi_iop_ffi -- --nocapture --test-threads=1"
