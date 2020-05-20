#!/usr/bin/env bash

# Assumption: `cargo build` has been run, output is in `target` dir.
#JNI_LIBS=/Users/Enzo/Dev/my-projects/smpi/src/v1/rn/SMPIDemo/android/app/src/main/jniLibs
#JNI_LIBS=/Users/Enzo/Dev/my-projects/smpi/src/v1/rn/test/20191118_165723/smpi-iop-react-native/android/src/main/jniLibs
JNI_LIBS=/Users/Enzo/Dev/my-projects/smpi/src/v1/git/smpi-iop-react-native/android/src/main/jniLibs

COMPILED=libsmpi_iop_ffi.so

rm -rf $JNI_LIBS
mkdir $JNI_LIBS
mkdir $JNI_LIBS/arm64-v8a
mkdir $JNI_LIBS/armeabi-v7a
mkdir $JNI_LIBS/x86

R=`dirname $0`
cd $R/../../;
pwd;
cp target/aarch64-linux-android/release/$COMPILED $JNI_LIBS/arm64-v8a/$COMPILED
cp target/armv7-linux-androideabi/release/$COMPILED $JNI_LIBS/armeabi-v7a/$COMPILED
cp target/i686-linux-android/release/$COMPILED $JNI_LIBS/x86/$COMPILED
