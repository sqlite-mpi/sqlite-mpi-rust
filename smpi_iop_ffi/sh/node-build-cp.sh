#!/usr/bin/env bash

cargo build --release;

cp \
  /Users/Enzo/Dev/my-projects/smpi/src/v1/git/sqlite-mpi-rust/target/release/libsmpi_iop_ffi.dylib \
  /Users/Enzo/Dev/my-projects/smpi/src/v1/git/smpi-iop-node-ffi/src/cdylib;
