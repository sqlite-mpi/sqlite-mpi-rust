#!/usr/bin/env bash
#cargo watch -s "clear && printf '\e[3J'; cargo test test_runtime_read_and_write_tx  --package runtime -- --nocapture --test-threads=1"
cargo watch -s "clear && printf '\e[3J'; cargo test test_err  --package runtime -- --nocapture --test-threads=1"