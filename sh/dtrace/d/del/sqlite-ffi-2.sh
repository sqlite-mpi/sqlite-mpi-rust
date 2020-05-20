#!/usr/bin/env bash
d=`gdate --utc --iso-8601=seconds`;
f_sql="./../out/shell-${d}.sql";
f_db="./../out/shell-${d}.sqlite";

dtrace \
    -s sqlite-ffi-sql.c \
    -c "/Users/Enzo/Dev/my-projects/smpi/src/v1/smpiprototype/sh/dtrace/d/del/apsw/open.py" \
    -o ${f_sql};


# No `BUSY_RECOVERY` issue, 3 threads.
# target/debug/deps/runtime-069474f52c940449



sqlite3 ${f_db} < ${f_sql};
chown enzo ${f_db};
chown enzo ${f_sql};
subl ${f_sql};

#rm ${f_sql};