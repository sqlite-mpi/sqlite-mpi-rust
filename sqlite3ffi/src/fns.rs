use std::ffi::CString;


// Note: The lifetime of the pointer and its memory is managed by the var the `CString` is bound to.
// See `as_ptr` docs.
pub fn to_cstr(s: &str) -> CString {
    CString::new(s).expect("Could not convert Rust string to C string")
}

