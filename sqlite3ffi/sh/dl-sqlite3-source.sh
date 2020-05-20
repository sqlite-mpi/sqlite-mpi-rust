#!/usr/bin/env bash

v=3280000
cd $(dirname "$0");


# Make target dir.
target=./../c-code/sqlite3/$v;
rm -rf $target;
mkdir -p $target;
cd $target;


# Download, unzip.
curl -o amalgamation.zip https://www.sqlite.org/2019/sqlite-amalgamation-$v.zip;

# Should match sha1 downloaded via HTTPS URL.
# @see https://www.sqlite.org/download.html.
shasum -a 1 amalgamation.zip > amalgamation-sha1.txt;

unzip amalgamation.zip;
mv sqlite-amalgamation-$v all;