#!/usr/bin/env bash
cargo watch -s "clear && printf '\e[3J'; cargo build --package smpi_iop_ffi"
