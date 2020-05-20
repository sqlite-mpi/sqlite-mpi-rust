// @todo/important enable for production build.
#![allow(warnings)]
#![allow(unused_parens)]

use std::collections::HashMap;

extern crate chrono;

use chrono::{DateTime, Utc};


#[cfg(test)]
mod test_placeholders;
mod test_transactions;

// @todo/low Possible features:
// - @see https://www.sqlite.org/c3ref/progress_handler.html (Used for percent complete indicator of long queries).
// - `ATTACH` for multiple database files.
// - @see https://www.sqlite.org/c3ref/wal_checkpoint_v2.html, allow client to run WAL checkpoint to merge writes into the db file. E.g. on app close.

// Note: `mod x` must be in root crate file (main.rs or lib.rs).
// `use crate::x::{...}` can be used from sibling files.
mod cffi;
pub mod errmap;
pub mod err;
mod fns;
mod placeholder;
mod config;
pub mod db;
pub mod stmt;


use errmap::{
    PrimaryRC
};


use db::DbHandle;

use stmt::{
    StmtHandle,
    RSet,
    Rows,
    KeyVal,
    Val,
    ErrorBindType,
};