#!/usr/bin/env bash
#d=`gdate --utc --iso-8601=seconds`;
d=`date -u +"%Y-%m-%dT%H:%M:%SZ"`;
f_sql="./../out/${d}.sql";

f_name="${d}.sqlite";
f_db="./../out/${f_name}";


# Issue: "invalid address" somtimes.
# https://stackoverflow.com/questions/1198114/why-does-dtrace-give-me-invalid-address-errors-sometimes-but-not-always

dtrace \
    -s sqlite-ffi-sql.c \
    -c "./../../../target/debug/deps/runtime-eac33d2eec1a26dc test_multiple_concurrent_runtimes --nocapture --test-threads=1" \
    -o ${f_sql};


sqlite3 ${f_db} < ${f_sql};
chown enzo ${f_db};
chown enzo ${f_sql};
node ./set-ffi-boundary.js ${f_db};
#subl ${f_sql};
echo ${f_db};



# Open in Interplay web UI.
cp ${f_db} /Users/Enzo/Dev/my-projects/interplay/interplay/public/dtrace-out-auto-mv/
chown enzo "/Users/Enzo/Dev/my-projects/interplay/interplay/public/dtrace-out-auto-mv/${f_name}";
open -a "Google Chrome" http://localhost:3000/?db_file_url=/dtrace-out-auto-mv/${f_name};

#rm ${f_sql};