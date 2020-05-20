#!/usr/bin/env node

/**
 * This script:
 * - A. Breaks DTraces `ustack()` string into an array of [{fn, addr}], and inserts them into a `stack` SQLite table.
 * - B. Detects the boundary between the host program and the FFI, updates the SQLite cell `fns.is_ffi = bool`.
 *      - See DTrace script comments for more details.
 *
 * Usage:
 * - `./set-ffi-boundary.js ./../out/2019-09-17T18:38:34+00:00.sqlite`
 */


// `npm install sqlite3 -g`
const sqlite3 = require('/usr/local/lib/node_modules/sqlite3').verbose();
const [dbFile,] = process.argv.slice(2);
const db = new sqlite3.Database(dbFile);

/**
 * Gets the atomic units from the `ustack()` DTrace output.
 *
 * Example line: `runtime-069474f52c940449`sqlite3_prepare_v2+0x4f`
 */
const getParts = (stack) => {
    const no_white_space = stack.replace(/ /g, "").trim();
    const m = [...no_white_space.matchAll(/^.+?`(.+?)\+(.+?)$/gm)];
    return m.map(([_, fn, addr]) => ({fn, addr}));
};

// @todo/low A more reliable way to detect FFI boundary.
// Note: Rust uses `::` as path separators. Assumption: this is always true.
const isHostFn = (fn) => /::/.test(fn);


/**
 * E.g. FFI function boundary:
 *      - `sqlite3_open`
 *      - `sqlite3ffi::db::DbHandle::new::h7c56db7ad9853cba`
 *
 * E.g. Not a FFI boundary:
 *      - `sqlite3_step`
 *      - `sqlite3_prepare_v2`
 */
const isFFI = (stack) => {
    if (stack.length < 2) {
        return false;
    }

    return (
        !isHostFn(stack[0].fn) &&
        isHostFn(stack[1].fn)
    )
};

db.serialize(() => {
    db.run("CREATE TABLE stack (sid INTEGER PRIMARY KEY, id, i, fn, addr)");

    db.all("SELECT id, stack FROM fns", (err, rows) => {
        if (err) {
            console.error(err);
            return;
        }

        for (const {id, stack} of rows) {

            const stack_array = getParts(stack);
            for (const [i, {fn, addr}] of stack_array.entries()) {
                db.run("INSERT INTO stack (id, i, fn, addr) VALUES (?, ?, ?, ?)", [id, i, fn, addr]);
            }

            db.run("UPDATE fns SET is_ffi = ? WHERE id = ?", [isFFI(stack_array), id]);
        }
    });
});