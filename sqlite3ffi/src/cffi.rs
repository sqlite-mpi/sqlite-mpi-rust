#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// Prevent `foreign function is never used` warning at compile time for `sqlite3_*` functions.
#![allow(dead_code)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));


use std::mem;


// @see https://www.sqlite.org/c3ref/c_static.html
// - These are C function pointers with special values?
// - @todo/low Integers are not passed because a function can also be passed to the same arg in the FFI.

pub fn SQLITE_STATIC() -> sqlite3_destructor_type {
    Some(unsafe { mem::transmute(0isize) })
}

// "SQLite makes its own private copy of the data immediately, before the sqlite3_bind_*() routine returns".
pub fn SQLITE_TRANSIENT() -> sqlite3_destructor_type {
    Some(unsafe { mem::transmute(-1isize) })
}
