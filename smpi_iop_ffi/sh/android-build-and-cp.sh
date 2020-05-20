#!/usr/bin/env bash
clear;
R=`dirname $0`
cd $R;
./android-build.sh;
./android-cp-jni-libs-dev.sh;
