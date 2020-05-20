extern crate chrono;

use std::mem::MaybeUninit;

use std::sync::Once;

use serde::{Deserialize, Serialize};

use crate::cffi::{
    // Objects
    sqlite3,

    // Functions
    sqlite3_open,
    sqlite3_close,
    sqlite3_extended_result_codes,
    sqlite3_threadsafe,
};


use crate::errmap;
use crate::err;
use crate::fns;
use crate::config;


use config::set_error_callback;
use fns::to_cstr;
use errmap::PrimaryRow;

use err::{
    ReturnStatus,
    to_return_status_cint,
};
use crate::stmt::{
    StmtHandle,
    RSet,
    KeyVal,
    IndexVal,
    ErrorBind
};


#[derive(Debug)]
pub struct DbHandle {
    pub db: *mut sqlite3,
    pub file: String,
}

impl Drop for DbHandle {
    // @see https://www.sqlite.org/c3ref/close.html
    // - "If an sqlite3 object is destroyed while a transaction is open, the transaction is automatically rolled back."
    fn drop(&mut self) {
//        dbg!("db.d()");

        let db = self.db;
        assert!(!(db.is_null()), "Drop called but db handle was null.");

        let r = unsafe { sqlite3_close(db) };
        let close = to_return_status_cint(&r);

        // Any outstanding resources on the connection will cause this to fail.
        // Assume: Rust code will always ensure connection is closable when `DbHandle` drops out of scope.
        assert!(close.is_ok, "sqlite3_close failed: {:?}", close);


        // @todo/medium Make sure pointer memory is freed/zeroed/null;
    }
}


#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub enum BindRunError {
    ReturnStatus(ReturnStatus),
    ErrorBind(ErrorBind)
}

impl From<ReturnStatus> for BindRunError {
    fn from(rs: ReturnStatus) -> Self {
        BindRunError::ReturnStatus(rs)
    }
}
impl From<ErrorBind> for BindRunError {
    fn from(eb: ErrorBind) -> Self {
        BindRunError::ErrorBind(eb)
    }
}



impl DbHandle {
    /*
    @todo/important Check usage of pointers is correct/safe.
    @todo/low Map SQL read/writes to Rusts ownership semantics. (You need a mut to write, and a & to read). Mutate "changes" on write?
    */
    pub fn new(file: String) -> Result<DbHandle, ReturnStatus> {
        // @todo/low Call on init, not on first db handle request.
        set_error_cb_once();
        assert_is_threadsafe();

        let c_file = to_cstr(file.as_str());

        // Question: *mut *mut = coerce a pointer to a pointer? How does a pointer to a pointer FFI work?
        // Question: Should the `MaybeUninit` var be held onto after the memory is initialised?
        let mut mu = MaybeUninit::uninit();
        let mut db: *mut sqlite3 = mu.as_mut_ptr();


        let r = unsafe {
            sqlite3_open(c_file.as_ptr(), &mut db)
        };


        let open = to_return_status_cint(&r);
        if !open.is_ok {
            let r = unsafe { sqlite3_close(db) };
            return Err(open);
        }

        extended_result_codes_on(db)?;

        Ok(
            DbHandle {
                file,
                db,
            }
        )
    }


    // @todo/low prevent exporting static StmtHandle?
    pub fn new_stmt(&self, q: &str) -> Result<StmtHandle, ReturnStatus> {
        StmtHandle::new(&self, q)
    }

    // @todo/low only allow access to run, ignore statements?
    pub fn run(&self, q: &str) -> Result<RSet, ReturnStatus> {
        let s = &self.new_stmt(q)?;
        s.run()
    }

    pub fn run_kv(&self, q: &str, kv: &KeyVal) -> Result<RSet, BindRunError> {
        let s = &self.new_stmt(&q)?;
        &s.bind_kv(&kv)?;
        Ok(s.run()?)
    }

    pub fn run_index(&self, q: &str, vals: &IndexVal) -> Result<RSet, BindRunError> {
        let s = &self.new_stmt(q)?;
        &s.bind_index(&vals)?;
        Ok(s.run()?)
    }

}

static START: Once = Once::new();

// @see https://www.sqlite.org/c3ref/threadsafe.html
// Note: this only checks compile time flag, not start time or run time over rides.
fn assert_is_threadsafe() {
    START.call_once(|| {
        assert_ne!(unsafe { sqlite3_threadsafe() } as u32, 0, "SQLite was not compiled with SQLITE_THREADSAFE flag.");
    });
}


fn set_error_cb_once() {
    match set_error_callback() {
        Some(res) => {
            assert!(res.is_ok());
//            println!("set_error_callback ran once.")
        }
        None => {
            // dbg!("set_error_callback did not run again");
        }
    }
}


fn extended_result_codes_on(db: *mut sqlite3) -> Result<ReturnStatus, ReturnStatus> {
    let code = unsafe { sqlite3_extended_result_codes(db, 1) };
    let row = to_return_status_cint(&code);

    if !row.is_ok {
        return Err(row);
    }

    Ok(row)
}
