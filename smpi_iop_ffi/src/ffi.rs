use libc::c_char;
use std::ffi::{CStr, CString};

use std::sync::Once;


use serde::{Deserialize, Serialize};
use serde_json::{
    json,
    Error,
};

use runtime::{
    runtime::{
        Runtime,
        InMsgId,
        OutMsgWithId,
    },
    json_in::in_json_to_rs,
    json_out::out_rs_to_json,
};

use runtime::{
    json_out::{
        ErrRes,
        E,
    },
};


pub type ReqJSON = String;
pub type ResJSON = String;

pub type CReqJSON = *const c_char;

pub type CInMsgId = *mut c_char;
pub type CResJSON = *mut c_char;

pub type CRetIJSONPtr = *mut c_char;
pub type CRetOJSONPtr = *mut c_char;

pub type COutFn = extern "C" fn(CRetOJSONPtr);

struct Rt {
    rt: Runtime,
}


// Note: `RetI` returns a message back to the host process that uses the guest FFI.
// - This is only for the set of sync responses where its not possible to call the callback function.
// - E.g.
//      - When an input msg id is not valid (for the client to map an input msg to an output msg):
//            - When a parse error occurs.
// In the case of an error, the host should log the response and not look for an output message.
// Clients will not be passed the sync responses as they are expecting a (in_msg_id, out_msg) response.
// Assumption: These types of response will only happen at integration/develop time.
// - If the JSON is well formed sync responses will never occur.
// @see smpi_iop_ffi/diagrams/chg/CHG.svg

// @see https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise
// - settled = (fulfilled | rejected)
// - pending = !settled
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[serde(tag = "ret_i_type", content = "val")]
#[serde(rename_all(serialize = "lowercase", deserialize = "lowercase"))]
enum RetI {
    Pending,
    Settled(SettledI),
    // @todo/maybe Event receive confirm (passing many events from host to ffi without needing a response)
}


#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[serde(tag = "ret_o_type", content = "val")]
#[serde(rename_all(serialize = "lowercase", deserialize = "lowercase"))]
enum RetO {
    Settled(SettledO)
    // @todo/maybe Event (passing data from ffi to host without host requests). E.g. logs?
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
struct Pending {
    in_msg_id: InMsgId,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
struct SettledI {
    // This is a string as the host caller may just be passing the response to another process.
    // - Prevent strongly typed languages having to define a type when they may just be passing it to another process.
    msg: ResJSON,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
struct SettledO {
    in_msg_id: InMsgId,
    msg: ResJSON,
}


impl RetI {
    //@see https://doc.rust-lang.org/std/convert/trait.Into.html#examples
    // - Generic function for types that implement `Into`.
    pub fn new_settled_from_error<T: Into<E>>(e: T) -> RetI {
        let out_msg = ErrRes::new(e.into());
        let msg = serde_json::to_string(&out_msg).unwrap();
        RetI::Settled(SettledI { msg })
    }
}


// @todo/low Allow FFI host to start/stop many runtimes (instead of just one per process).
// @todo/medium Allow clean shut down.

static mut RT: Option<Rt> = None;
static mut INIT: Once = Once::new();
static mut STOP: Once = Once::new();

fn reset() {
    unsafe {
        // This will cause `drop` to run which will `join` the BG thread, close all SQLite connections (releasing open write txs).
        RT = None;

        INIT = Once::new();
        STOP = Once::new();
    }
}


fn assert_started() -> &'static Rt {
    let error_not_init = "Runtime expected to be started, but was stopped.";

    // Assert: Runtime is init.
    // assert!(INIT.is_completed(), error_not_init); // (Nightly feature).

    if let Some(rt) = unsafe { &RT } {
        return rt;
    }
    unreachable!(error_not_init);
}


pub fn to_ptr(s: String) -> *mut c_char {
    CString::new(s).unwrap().into_raw()
}

// Takes ownership of `ptr`, converts it to a Rust `String`.
// Note: this frees the memory when the owning scope ends; no need to call `free_string(ptr)`.
pub fn from_ptr(ptr: *mut c_char) -> String {
    unsafe {
        CString::from_raw(ptr).into_string().unwrap()
    }
}


fn to_json_ptr<T: Serialize>(t: &T) -> *mut c_char {
    let s = serde_json::to_string(t).unwrap();
    to_ptr(s)
}

fn copy_str(c_ptr: *const c_char) -> String {
    // Copy from string; client still owns string and must free it.
    let c_buf = unsafe {
        CStr::from_ptr(c_ptr)
    };

    c_buf.to_str().unwrap().to_owned()
}

// @todo/maybe Should this take an enum as input to allow any request or event to be passed as input?
#[no_mangle]
pub extern "C" fn smpi_input(c_req: CReqJSON) -> CRetIJSONPtr {
    // @todo/important Should this be protected by a lock in case of a bad FFI host implementation sharing the `input` function with many threads?
    let Rt { rt, .. } = assert_started();

    let req = copy_str(c_req);

    let with_id = match in_json_to_rs(&req) {
        Ok(x) => x,
        Err(parse_err) => {
            dbg!(&parse_err);

            // @todo/low Separate errors outside of the runtime event loop into a different category.
            // - E.g. this returns `E.error_type = ParseError/ParseError`, but the runtime can also return errors for that category.
            //      - Use `LayerX/ParseError/ParseError`, `LayerY/ParseError/ParseError`?
            return to_json_ptr(&RetI::new_settled_from_error(parse_err));
        }
    };

    // @todo/important How to assert that the host calling this function has returned and stored the `req_id` in its key value map?
    // - Is it possible that the outputFn callback will be called before this function returns to the host?
    // - Assumption: The callback is only called on the host after all sync code has completed.
    //      - In JS a sync function will run to completion (block) before processing other incoming async events (outputFn call).
    // Fix, possible: use `free_string` as a notification that the host has stored the `req_id` into its key value map.
    // @todo/important Fix: Let the client create a input msg id, do not return a value from this function.
    // - RR = Request/Response, ST = Stream.
    // Instead of `input` being a RR interface, convert it to a ST interface, like output.
    // - All output flows through `output`.
    // - `input` would never return a value.
    let ret_i = match rt.input(with_id) {
        Ok(id) => {
            RetI::Pending
        }
        Err(e) => {
            // @todo/medium handle error case.
            dbg!(e);
            unreachable!();
        }
    };

    to_json_ptr(&ret_i)
}


// Note: When returning C strings from Rust to the host runtime, the host must return the ptr for it to be freed.
// - Rust uses a different allocator to the host language.
// @see https://doc.rust-lang.org/std/ffi/struct.CString.html#method.into_raw
// - "one should not use the standard C free()"
// @see https://stackoverflow.com/a/42498913/4949386
// - Pass string Rust -> Node FFI with example code.
#[no_mangle]
pub extern "C" fn smpi_free_string(ptr: *mut c_char) {
    assert!(!ptr.is_null(), "null ptr passed to Rust free_string fn");

    unsafe {
        // Free memory.
        // - Convert to Rust CString, bind to var in scope (implicit), allow it to be dropped at the end of the scope.
        CString::from_raw(ptr);
    };
}


// Allows the host process to set the callback function *once*.
// - Instead of creating a new callback for every request, use a single ID to represent a (request, response).
// - Why?
//      - Portability: Not every language supports closures.
//      - Less complex objects being shared over FFI with C pointers.
//      - One function call per request instead of two.
//      - Single point of concurrency for both sides.
// @todo/low Host must free fn ptr.
#[no_mangle]
pub extern "C" fn smpi_start(c_out_fn: COutFn) {
    let mut is_first = false;

    let init = unsafe { &INIT };

    init.call_once(|| {
        is_first = true;

        // @todo/low Allow shutting down runtime, restarting with new fn.
        assert!(unsafe { RT.is_none() });


        let o = move |out: OutMsgWithId| {
            let OutMsgWithId {
                in_msg_id,
                msg
            } = out;

            let out_string = out_rs_to_json(&msg).unwrap();

            let o = RetO::Settled(SettledO { in_msg_id, msg: out_string });

            // @todo/important Should this be protected by a lock in case of a bad FFI host implementation sharing the `output` function with many threads?
            c_out_fn(to_json_ptr(&o));
        };

        let rt = Runtime::new(o);

        unsafe {
            RT = Some(Rt { rt });
        }
    });

    assert!(is_first, "`start` was called more than once. It should only be called once for each runtime instance.");
}


// Resets FFI state to the same as a freshly started process.
// Why this is needed:
// - RN, JS only refresh during dev should release any currently open wtxs.
//      - By default on JS reload the native shell is kept alive (which keeps currently open write txs alive).
//          - This causes the next loads `getWriteTx` requests to block forever.
// - Node, clean shutdown/release of resources.
// - Testing, to test from the FFI start state without restarting the test process.
#[no_mangle]
pub extern "C" fn smpi_stop() {
    let mut is_first = false;
    let once = unsafe { &STOP };

    once.call_once(|| {
        is_first = true;
        assert!(unsafe { RT.is_some() });
    });

    // Issue: Cannot mutate a `Once` var whilst its `call_once` function is active. This results in a panic.
    if is_first {
        reset();
    }

    assert!(is_first, "`stop` was called more than once. It should only be called once for each runtime instance.");
}
