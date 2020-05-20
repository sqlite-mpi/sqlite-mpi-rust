#!/usr/bin/env bash
cargo watch -s "clear && printf '\e[3J'; date; ./sh/valgrind/vg2.sh  >& /dev/null && echo 'Valgrind PASSING' || echo 'Valgrind FAILED. Returned memory errors (non 0 return code).'"
