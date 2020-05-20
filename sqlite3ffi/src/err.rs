// @todo/low
// - Map SQLite to Rust `Result` type?
// - Research: binding.rs defines return codes as `u32`, but the SQLite docs says they are `i32`, despite being all positive.

use std::ffi::CStr;

use serde::{Deserialize, Serialize};

use crate::errmap::{
    PrimaryRC,
    PrimaryRow,
    ExtendedRow,
    get_rows,
    get_primary_row_by_enum,
};

use crate::cffi::{
    // Objects
    sqlite3,

    // Functions
    sqlite3_errmsg,
};


#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct ReturnStatus {
    // = SQLITE_OK
    pub is_ok: bool,

    // NOT (SQLITE_OK, SQLITE_ROW, SQLITE_DONE)
    pub is_err: bool,

    pub primary: PrimaryRow,
    pub extended: Option<ExtendedRow>,

    pub err_msg: Option<String>,
}

// Does this include any context other than a string describing the error code?
// May as well use Google/SQLite docs?
// @see https://doc.rust-lang.org/std/ffi/struct.CStr.html
//fn err_code_to_str(errorCode: ::std::os::raw::c_int) -> String {
//    unsafe {
//        CStr::from_ptr(sqlite3_errstr(errorCode)).to_string_lossy().into_owned()
//    }
//}


impl From<PrimaryRC> for ReturnStatus {
    fn from(prc: PrimaryRC) -> Self {
        // Example use: generate `ReturnStatus` from `PrimaryRC` during testing.
        to_return_status(&get_primary_row_by_enum(&prc).code)
    }
}


fn get_is_error(id: &PrimaryRC) -> bool {
    match id {
        PrimaryRC::SQLITE_OK => false,
        PrimaryRC::SQLITE_ROW => false,
        PrimaryRC::SQLITE_DONE => false,
        _ => true
    }
}


pub fn to_return_status(code: &u32) -> ReturnStatus {
    let (primary, extended) = get_rows(&code);

    let mut is_ok = false;
    if let PrimaryRC::SQLITE_OK = primary.id {
        is_ok = true;
    }

    let is_err = get_is_error(&primary.id);


    ReturnStatus {
        is_ok,
        is_err,
        primary,
        extended,
        err_msg: None,
    }
}

// @todo/important Is it ok to cast `c_int` to `u32` like this?
// Used directly for return values from `sqlite3_*` FFI.
pub fn to_return_status_cint(code: &::std::os::raw::c_int) -> ReturnStatus {
    to_return_status(&(*code as u32))
}

// Force client code to deal with error.
// Better error handling: Connect SQLite errors to Rusts native error handling.
pub fn to_return_status_cint_err(code: &::std::os::raw::c_int) -> Result<ReturnStatus, ReturnStatus> {
    let r = to_return_status(&(*code as u32));

    match r.is_ok {
        true => Ok(r),
        false => Err(r)
    }
}

pub fn to_return_status_cint_db_err(code: &::std::os::raw::c_int, db: *mut sqlite3) -> Result<ReturnStatus, ReturnStatus> {
    let mut r = to_return_status(&(*code as u32));

    if r.is_err {
        r.err_msg = Some(get_error_from_db(db));
        return Err(r);
    }

    Ok(r)
}


// @todo/medium `sqlite3_errmsg` is not thread safe, and only returns the most recent error.
pub fn get_error_from_db(db: *mut sqlite3) -> String {
    unsafe {
        // Note: `sqlite3_errmsg` may re-use the memory, no need for client to free.
        CStr::from_ptr(sqlite3_errmsg(db)).to_string_lossy().into_owned()
    }
}