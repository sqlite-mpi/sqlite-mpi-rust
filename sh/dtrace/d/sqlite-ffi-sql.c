#!/usr/sbin/dtrace -s
// #pragma D option flowindent

// Do not add columns (cpu, id, function, name).
#pragma D option quiet

// Issue: "dtrace: 2683 dynamic variable drops with non-empty dirty list"
// Fix: Increase `dynvarsize`
// @see dtrace-ebook.pdf#page=162
// @see DTrace_Chapter_5_File_Systems.pdf#page=52
#pragma D option dynvarsize=1024m


// @todo/low Allow tracing all `sqlite3*` entry/return functions when a targeted function has been entered.
// E.g: "While any target function is active, record all functions"
// - Would allow flame graphs *per function entry* to see exactly what path SQLite is taking internally to decide on a specific return value (BUSY_RECOVERY).
//      - E.g. Click on the entry, get a flame graph pop up.

// @todo/important `dtrace: error on enabled probe ID 15 (ID 375944: pid61564:runtime-069474f52c940449:sqlite3_open:return): invalid address (0x12068) in action #16 at DIF offset 56`

BEGIN
{
    // @todo/low Meta data table for `in_1_p1` (for C APIs that take a pointer to a pointer as input).
    trace("CREATE TABLE fns (id INTEGER PRIMARY KEY, ppid, pid, tid, prov, mod, func, start_ts, end_ts, duration, level, stack, is_ffi, in_0, in_1, in_2, in_3, in_4, out_1, in_1_p1, in_3_p1);\n");
    trace("CREATE VIEW fns_ffi AS SELECT * FROM fns WHERE is_ffi = 1;\n");
}

// Store the entry meta data until return is called, then output a single row.
//struct entrymeta {
//    int64_t arg0;
//    int64_t arg1;
//    int64_t arg2;
//    int64_t arg3;
//    int64_t arg4;
//
//    uint64_t walltimestamp;
//};
//
//self struct entrymeta entry[string];

pid$target::sqlite3_open:entry,
pid$target::sqlite3_close:entry,
pid$target::sqlite3_prepare_v2:entry,
pid$target::sqlite3_step:entry,
pid$target::sqlite3_reset:entry,
pid$target::sqlite3_finalize:entry
{

    // Note: Using `self struct entrymeta entry[string];` does not work as `self->entry[probefunc] = 0` does not work to delete?
    // - Question: How do you clear a struct from an associative array?

    // Issue: A. The target functions can call themselves recursively, or call each other.
    // E.g: (sqlite3_prepare_v2, sqlite3_prepare_v2), (sqlite3_prepare_v2, sqlite3_finalize), (sqlite3_prepare_v2, sqlite3_step)
    // B. Its also not possible to detect the boundary between FFI consumer and internal FFI calls inside a DTrace script.
    // - `ustack()` only provides a single string containing newline separated list of function calls.
    // Fix:
    // A. Use current stack depth/tree level to match `entry` with its `return`.
    // B. Store `ustack()` string with row in order to detect FFI boundary with string processing.

    // Also see: `stackdepth`
    // Note: In the cases when target functions call themselves or other target functions, timelines can overlap for a single thread.
    // - A flame graph-like UI is needed.
    // - All start/end timestamps can be merged into a table: (fn, ts, type=start/end) order by ts asc, and the depth can be calculated with a single loop. See `getFreeTime` in Interplay UI.
    self->entry_levels[probefunc, "level"]++;
    level = self->entry_levels[probefunc, "level"];

    self->entry[probefunc, level, "walltimestamp"] = walltimestamp;
    self->entry[probefunc, level, "arg0"] = arg0;
    self->entry[probefunc, level, "arg1"] = arg1;
    self->entry[probefunc, level, "arg2"] = arg2;
    self->entry[probefunc, level, "arg3"] = arg3;
    self->entry[probefunc, level, "arg4"] = arg4;

     // Issue: SQLite calls these functions internally too; only trace when going from program into SQLite code.
//     ustack();
    // trace(copyinstr(arg0));
}



//pid$target::sqlite3_open:return
///self->entry[probefunc, "walltimestamp"] != 0 /
//{
//    print("sqlite3_open:return");

//    a1_orig = self->entry[probefunc, "arg1"];

    // @todo/next Get p2 from p1(p2(obj)) when opening so the pointer can be matched with close.

    // A.
//    print(a1_orig);
//    print(copyin(a1_orig, sizeof(uintptr_t)));
//    print((int64_t)copyin(a1_orig, sizeof(uintptr_t)));

//    printf("a1: %d\n", (uint64_t)copyin(a1_orig, sizeof(uintptr_t)));
//    printf("a1: %d\n", copyin(a1_orig, sizeof(uintptr_t)));
//    print(*(int64_t *)copyin(a1_orig, sizeof(uintptr_t)));
//    x = *(int64_t *)copyin(a1_orig, sizeof(uintptr_t));
//    printf("a1: %d\n", x);

// THIS WORKS
//    x = *(int64_t *)copyin(a1_orig, sizeof(uintptr_t));

    // B.
//    print((void *)copyin(a1_orig, sizeof(uintptr_t)));

    // C.
//    print(copyin(a1_orig, 4));

    // D.
//    print(copyin(a1_orig, 8));

    // E.
//    print(*(uintptr_t *)a1_orig);

//}



pid$target::sqlite3_open:return,
pid$target::sqlite3_close:return,
pid$target::sqlite3_prepare_v2:return,
pid$target::sqlite3_step:return,
pid$target::sqlite3_reset:return,
pid$target::sqlite3_finalize:return
/self->entry_levels[probefunc, "level"] != 0 /
{
        level = self->entry_levels[probefunc, "level"];
        self->entry_levels[probefunc, "level"]--;


        // Note: Recursive functions cannot catch the return probe.
        // Note: Return probe may be missing for some compiler optimisations; Use in debug mode to be sure.

        start_ts = self->entry[probefunc, level, "walltimestamp"];
        in_0 = self->entry[probefunc, level, "arg0"];
        in_1 = self->entry[probefunc, level, "arg1"];
        in_2 = self->entry[probefunc, level, "arg2"];
        in_3 = self->entry[probefunc, level, "arg3"];
        in_4 = self->entry[probefunc, level, "arg4"];
        out_1 = arg1;

        self->entry[probefunc, level, "walltimestamp"] = 0;
        self->entry[probefunc, level, "arg0"] = 0;
        self->entry[probefunc, level, "arg1"] = 0;
        self->entry[probefunc, level, "arg2"] = 0;
        self->entry[probefunc, level, "arg3"] = 0;
        self->entry[probefunc, level, "arg4"] = 0;

        // For C APIs that take a pointer to a pointer as input.
        // - Get the address of the second pointer so that functions that use it as input can be related.
        // - Note: May only be set on the function return.
        // - `int64_t` is the type of `arg0` dtrace variables.
        //      - Cast to this type so that other functions that take `p1` as an input argument can be related.
        //      - `copyin` is required to copy from the user process to the kernel (where dtrace runs).
        //          - Requires setting `csrutil enable --without dtrace` on macOS.
        in_1_p1 = probefunc == "sqlite3_open" ? *(int64_t *)copyin(in_1, sizeof(uintptr_t)) : 0;
        in_3_p1 = probefunc == "sqlite3_prepare_v2" ? *(int64_t *)copyin(in_3, sizeof(uintptr_t)) : 0;


        // probename
        duration = walltimestamp - start_ts;

        printf("INSERT INTO fns (ppid, pid, tid, prov, mod, func, start_ts, end_ts, duration, level, in_0, in_1, in_2, in_3, in_4, out_1, in_1_p1, in_3_p1) VALUES (%d, %d, %d, '%s', '%s', '%s', %d, %d, %d, %d, %d, %d, %d, %d, %d, %d, %d, %d);\n", ppid, pid, tid, probeprov, probemod, probefunc, start_ts, walltimestamp, duration, level, in_0, in_1, in_2, in_3, in_4, out_1, in_1_p1, in_3_p1);


        // Note: `ustack()` cannot be stored in a variable (because its called in the kernel, but the symbols are resolved when the buffer is processed by the DTrace client?).
        printf("UPDATE fns SET stack = '");
        ustack();
        printf("' WHERE id = (SELECT MAX(id) FROM fns);\n");
}