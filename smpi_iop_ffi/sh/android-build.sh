#!/usr/bin/env bash

# Note: `--release` missing.


# Issue: `error occurred: Failed to find tool. Is `arm-linux-androideabi-clang` installed?` when building `sqlite3.c`.
# Fix: `export AR|TARGET`.
# @see https://github.com/briansmith/ring/issues/897#issuecomment-532707344
# @see https://stackoverflow.com/questions/54189024/failed-to-cross-compile-library-from-windows-to-android
# Note: the paths are also in `~/.cargo/config`.


#export CC=x
export AR=~/.NDK/arm64/bin/aarch64-linux-android-ar;
export TARGET_CC=~/.NDK/arm64/bin/aarch64-linux-android-clang;
cargo build --release --target aarch64-linux-android;


export AR=~/.NDK/arm/bin/arm-linux-androideabi-ar;
export TARGET_CC=~/.NDK/arm/bin/arm-linux-androideabi-clang;
cargo build --release --target armv7-linux-androideabi;

export AR=~/.NDK/x86/bin/i686-linux-android-ar;
export TARGET_CC=~/.NDK/x86/bin/i686-linux-android-clang;
cargo build --release --target i686-linux-android;


unset AR;
unset TARGET_CC;
