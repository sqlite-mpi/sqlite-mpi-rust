#![cfg(target_os = "android")]
#![allow(non_snake_case)]

use libc::c_char;
use std::ffi::{CString, CStr};
use jni::JNIEnv;
use jni::objects::{JObject, JString, JValue};
use jni::sys::jstring;


use crate::ffi::{
    smpi_start,
    smpi_stop,
    smpi_input,
    to_ptr,
    from_ptr,
};

// Notes
// - This exports a JNI which wraps the FFI.
//      - @todo/low Instead of (JNI <-> FFI <-> Rust, FFI <-> Rust), move runtime to pure Rust and use it directly from the JNI functions:
//          - (JNI <-> Rust, FFI <-> Rust)

// Examples of calling Java from Rust:
// @see https://stackoverflow.com/a/48489590
// @see https://stackoverflow.com/questions/6619980/call-function-pointer-from-jni

// @see https://github.com/jni-rs/jni-rs/blob/master/example/mylib/src/lib.rs
// - Example code for Rust JNI API.

// @see https://docs.oracle.com/javase/7/docs/technotes/guides/jni/spec/types.html
// - Signature string format.


static mut CB: Option<Box<Fn(String)>> = None;

fn set_cb<F: Fn(String) + 'static>(f: F) {
    unsafe {
        CB = Some(Box::new(f));
    }
}

fn clear_cb() {
    unsafe {
        CB = None;
    }
}


#[no_mangle]
pub extern "C" fn call_cb(o_msg_json_ptr: *mut c_char) {
    let o_msg = from_ptr(o_msg_json_ptr);

    // Get a mutable borrow - do not take ownership of closure.
    if let Some(f) = unsafe { &mut CB } {
        f(o_msg);
    }
}


// @see https://github.com/jni-rs/jni-rs/blob/master/example/mylib/src/lib.rs#L132
// - Async callback example (must convert local references to global, and then back to local to pass to other threads).

#[no_mangle]
pub extern "system" fn Java_com_sqlitempi_iop_java_IOP_start(env: JNIEnv<'static>, _: JObject, cb_obj: JObject<'static>, fn_name: JString) {
    // @todo/low Ensure lifetime of JNIEnv is valid.

    let jvm = env.get_java_vm().unwrap();
    let cb_ref = env.new_global_ref(cb_obj).unwrap();
    let f = to_string(&env, fn_name);

    // @todo/low Print to Android log stream for app.
    dbg!(">>>>> FROM RUST");
    dbg!(env.get_version());

    let o_fn = move |o_msg: String| {
        // Use the `JavaVM` interface to attach a `JNIEnv` to the current thread.
        let env_1 = jvm.attach_current_thread().unwrap();
        let cb_1 = cb_ref.as_obj();

        let j_msg = from_string(&env_1, o_msg);
        let v = JValue::Object(j_msg.into());
        env_1.call_method(cb_1, f.clone(), "(Ljava/lang/String;)V", &[v]).unwrap();
    };

    set_cb(o_fn);
    smpi_start(call_cb);
}


#[no_mangle]
pub extern "system" fn Java_com_sqlitempi_iop_java_IOP_stop(env: JNIEnv, _: JObject, c_ref: JObject) {
    clear_cb();
    smpi_stop();
}

#[no_mangle]
pub extern "system" fn Java_com_sqlitempi_iop_java_IOP_input(env: JNIEnv, _: JObject, j_i_msg: JString) -> jstring {
    let ptr = to_ptr(to_string(&env, j_i_msg));
    let ret_i = smpi_input(ptr);

    from_string(&env, from_ptr(ret_i))
}


fn to_string(env: &JNIEnv, s: JString) -> String {
    env.get_string(s)
        .expect("Could not convert JString to Rust String")
        .into()
}


fn from_string(env: &JNIEnv, s: String) -> jstring {
    let x: JString = env.new_string(s)
        .expect("Could not convert Rust String to jstring")
        .into();

    // Return ptr for `jstring`.
    x.into_inner()
}

