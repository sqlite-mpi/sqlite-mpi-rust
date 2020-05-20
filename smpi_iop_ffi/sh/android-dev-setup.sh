#!/usr/bin/env bash

# Sets up a dev machine to `cargo build` for Android.

# @see https://medium.com/visly/rust-on-android-19f34a2fb43
# @see https://mozilla.github.io/firefox-browser-architecture/experiments/2017-09-21-rust-on-android.html

rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
export ANDROID_HOME=/Users/$USER/Library/Android/sdk

# Create standalone toolchains.
# - Allows building Rust for various Android architectures.
# Once per dev host (not project).
mkdir ~/.NDK
${ANDROID_HOME}/ndk/21.1.6352462/build/tools/make_standalone_toolchain.py --api 26 --arch arm64 --install-dir ~/.NDK/arm64
${ANDROID_HOME}/ndk/21.1.6352462/build/tools/make_standalone_toolchain.py --api 26 --arch arm --install-dir ~/.NDK/arm
${ANDROID_HOME}/ndk/21.1.6352462/build/tools/make_standalone_toolchain.py --api 26 --arch x86 --install-dir ~/.NDK/x86

# Manual steps:
# - Create `~/.cargo/config` toml file:
#```
#[target.aarch64-linux-android]
#ar = "./.NDK/arm64/bin/aarch64-linux-android-ar"
#linker = "./.NDK/arm64/bin/aarch64-linux-android-clang"
#
#[target.armv7-linux-androideabi]
#ar = "./.NDK/arm/bin/arm-linux-androideabi-ar"
#linker = "./.NDK/arm/bin/arm-linux-androideabi-clang"
#
#[target.i686-linux-android]
#ar = "./.NDK/x86/bin/i686-linux-android-ar"
#linker = "./.NDK/x86/bin/i686-linux-android-clang"
# ```
