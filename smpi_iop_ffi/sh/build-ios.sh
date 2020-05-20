#!/usr/bin/env bash

# @see https://medium.com/visly/rust-on-ios-39f799b3c1dd
# Note: `rust-bindgen` needs a patch for this to work
# - See https://github.com/rust-lang/rust-bindgen/issues/1211


# .sh works the same regardless of cwd.
d=`dirname $0`
shDir=`realpath ${d}`
iopDir="${shDir}/../"
projRoot="${shDir}/../../"
#iosProj="${projRoot}/../rn/SMPIDemo/ios"
#iosProj=/Users/Enzo/Dev/my-projects/smpi/src/v1/rn/test/20191118_165723/smpi-iop-react-native/ios
iosProj=/Users/Enzo/Dev/my-projects/smpi/src/v1/git/smpi-iop-react-native/ios

# `cargo lipo` creates a "universal binary"
# - This is an iOS concept of bundling all architectures into a single binary.
cd $projRoot;
cargo lipo --release;

cd $iopDir;
cbindgen src/lib.rs -l c > generated/c-ffi-interface.h


cp  generated/c-ffi-interface.h "${iosProj}/include"
cp  "${projRoot}/target/universal/release/libsmpi_iop_ffi.a" "${iosProj}/libs"
