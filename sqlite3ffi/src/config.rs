use std::sync::{Once, ONCE_INIT};

use std::ffi::{CStr};

use std::os::raw::{
    c_void,
    c_int,
    c_char,
};

use crate::cffi::{

    // Functions
    sqlite3_config,

    // Constants
    SQLITE_CONFIG_LOG,
};


use crate::err::{
    ReturnStatus,
    to_return_status_cint,
    to_return_status_cint_err,
};


// @see https://www.sqlite.org/c3ref/config.html
// - `sqlite3_config`
//      - Makes global changes to the sqlite lib.
//      - No other functions must be running whilst it is running; its not thread safe.


// @todo/important If the error callback is per process (not per thread), what happens when it is called from many threads concurrently?
// @see evernote:///view/14186947/s134/2e19502e-1c60-4e55-a9ce-c4e3415dc387/2e19502e-1c60-4e55-a9ce-c4e3415dc387/
// @see https://doc.rust-lang.org/nomicon/ffi.html#callbacks-from-c-code-to-rust-functions
extern "C" fn log_cb(data_ptr: *mut c_void, code: c_int, msg_c: *const c_char) {
    let r = to_return_status_cint(&code);

    let msg = unsafe {
        CStr::from_ptr(msg_c).to_string_lossy().into_owned()
    };

    // @todo/low Expose external FFI so users can redirect logs to their specific systems log collection method.

    let log = match r.extended {
        Some(ex) => format!("{:?}, {}", &ex, &msg),
        None => format!("{:?}, {}", &r.primary.id, &msg)
    };

    dbg!(log);
}


static ONCE_ONLY: Once = ONCE_INIT;

pub fn set_error_callback() -> Option<Result<ReturnStatus, ReturnStatus>> {
    let mut opt_r = None;

    ONCE_ONLY.call_once(|| {
        let r = unsafe {
            sqlite3_config(
                SQLITE_CONFIG_LOG as i32,
                log_cb as extern "C" fn(*mut std::ffi::c_void, i32, *const c_char)
            )
        };

        opt_r = Some(to_return_status_cint_err(&r));
    });

    opt_r
}
