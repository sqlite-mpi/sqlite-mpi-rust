use libc::c_char;
use std::ffi::CString;
use std::ptr;

use uuid::Uuid;

use serde_json::value::Value;
use serde_json::json;

extern crate chrono;

use chrono::{DateTime, Utc};


use crate::ffi::*;


use std::time::Duration;

pub fn get_unique_id() -> String {
    Uuid::new_v4().to_hyphenated().to_string()
}

pub fn ms(ms: u64) -> Duration {
    Duration::from_millis(ms)
}

use std::sync::mpsc;
use std::sync::mpsc::{
    Sender,
    Receiver,
};


static mut TX: Option<Sender<String>> = None;
static mut RX: Option<Receiver<String>> = None;


fn to_ptr(v: Value) -> *mut c_char {
    let j = v.to_string();
    CString::new(j).unwrap().into_raw()
}

fn to_ptr_string(s: String) -> *mut c_char {
    CString::new(s).unwrap().into_raw()
}

fn copy(ptr: *mut c_char) -> String {
    let s = unsafe {
        // Take ownership of the pointer and its content.
        // - If this is dropped, the content is freed.
        CString::from_raw(ptr)
    };

    // Copy the content.
    let cp: String = s.clone().into_string().unwrap();

    // Move ownership back to a pointer.
    // - Rust will not free content at the end of this content scope after `into_raw`.
    //      - `s` is consumed by `into_raw`.
    let same_ptr = s.into_raw();

    assert!(ptr::eq(ptr, same_ptr));
    assert!(!ptr.is_null());

    // Return pointer ownership.
    cp
}


fn drop(c_ptr: *mut c_char) {
    unsafe {
        CString::from_raw(c_ptr);
    }
}

fn new_test_file() -> String {
    let now: DateTime<Utc> = Utc::now();
    format!("/tmp/{}.sqlite3", now)
}

fn i(i: *mut c_char) -> String {
    let ret_ptr = smpi_input(i);
    let ret = copy(ret_ptr);
    smpi_free_string(ret_ptr);
    drop(i);
    return ret;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Assert: Basic FFI usage from the FFI-host process perspective works.
    // - Note: only the FFI message passing logic is tested.
    //      - See `runtime` for testing of threads, concurrency, locking, message formats, errors etc.
    #[test]
    fn test_ffi() {
        let (tx, rx) = mpsc::channel();
        unsafe {
            TX = Some(tx);
            RX = Some(rx);
        }

        // @see https://stackoverflow.com/questions/19387003/how-to-make-a-closure-typed-extern-c-fn
        extern "C" fn out_fn(ret_o: CRetOJSONPtr) {
            let ro = copy(ret_o);
            smpi_free_string(ret_o);

            unsafe {
                TX.as_ref().unwrap().send(ro).unwrap();
            }
        };

        smpi_start(out_fn);

        let f1 = new_test_file();

        // Assert: There are no outstanding write locks on `f1` so the wtx returns immediately.
        let assert_wtx_immediate = || {


            let wtx = to_ptr(json!({
                "id": get_unique_id(),
                "fn": "file/get_write_tx",
                "args": {
                    "file": &f1
                }
            }));

            // Place message onto input queue, get a C string request Id, copy it to Rust string, call FFI to free C string.
            let ret = i(wtx);



            // Assert: pending returned from `input`.
            let ret: Value = serde_json::from_str(&ret).expect("Ok");
            // @todo/low return value for FFI host (not client - it only receives from the output event stream).

            // Wait for response.
            let ret_o = unsafe {
                RX.as_ref().unwrap().recv().unwrap()
            };

            // Assert: get_wtx request returns pending from the `input` fn, and then settled<fulfilled> from `output` fn.
            let ret_o: Value = serde_json::from_str(&ret_o).expect("Ok");
            match (&ret_o["ret_o_type"], &ret_o["val"]["msg"]) {
                (Value::String(t), Value::String(msg)) => {
                    assert_eq!(t, "settled");
                    let out_msg: Value = serde_json::from_str(&msg).expect("Ok");
                    assert!(&out_msg["ok"].as_bool().unwrap());
                }
                _ => assert!(false, "Invalid JSON schema")
            }
        };


        assert_wtx_immediate();

        let assert_invalid_json = || {
            // Assert: Parse error returns a settled<rejected> response.
            let sync_err_ptr = to_ptr_string("this is not valid JSON".to_string());
            let e = i(sync_err_ptr);

            let ret: Value = serde_json::from_str(&e).expect("Ok");
            assert_eq!(&ret["ret_i_type"], "settled");
            let msg: Value = serde_json::from_str(&ret["val"]["msg"].as_str().unwrap()).expect("Ok");
            let ok = &msg["ok"].as_bool().unwrap();
            assert_eq!(*ok, false);
        };

        assert_invalid_json();


        // Assert: stop followed by start works.
        smpi_stop();
        smpi_start(out_fn);

        // Assert: Previous write lock is cleared by start/stopping the runtime.
        assert_wtx_immediate();
        assert_invalid_json();
    }
}
