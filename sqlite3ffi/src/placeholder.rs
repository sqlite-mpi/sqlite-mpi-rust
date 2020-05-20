use std::collections::HashMap;
use std::collections::HashSet;

use std::os::raw::{
    c_int,
    c_char,
};

use std::ffi::CStr;

use crate::cffi::{
    // Objects
    sqlite3_stmt,

    // Functions
    sqlite3_bind_parameter_count,
    sqlite3_bind_parameter_name,
    sqlite3_bind_parameter_index
};


use crate::fns::{
    to_cstr
};

#[derive(Debug)]
#[derive(PartialEq)]
pub enum PlaceholderTypes {
    Index,
    Key,
    IndexAndKey,
    None,
}

#[derive(Debug)]
pub struct PlaceholderMeta {
    pub max_index: u32,
    pub names: HashMap<String, u32>,
    pub keys: HashMap<String, u32>,
    pub keys_normal: HashMap<String, HashSet<u32>>,
    pub contains_integer_key: bool,

    pub types_used: PlaceholderTypes,
}


impl PlaceholderMeta {
    pub fn new(stmt: *mut sqlite3_stmt) -> PlaceholderMeta {
        get_placeholder_meta(stmt)
    }

}


// @see https://www.sqlite.org/c3ref/bind_blob.html
// @see https://www.sqlite.org/lang_expr.html#varparam

// Placeholder types:
// Index: `?, ?123`.
// Key: `:keyA, @keyA, $keyA`
//
// The SQLite FFI does not allow you to get all the valid indexes, only the max index.
// E.g. Using the API you can only determine `?, ?10` = [1,2,3,4,5,6,7,8,9,10].
// - The valid indexes are [1, 10]
// - `?x` can create gaps in the index range that have no corresponding placeholder in the query string.
//
// Although `?10` will show up in the `names`, *it is not a key*.
// - You cannot determine from an index-based query that all valid indexes have a value.
//      - You can only *bind every index upto max index*, and trust the user has aligned the indexes and values correctly.
pub fn get_placeholder_meta(stmt: *mut sqlite3_stmt) -> PlaceholderMeta {
    let max_index = bp_count(stmt);

    let names = get_ph_names(stmt);
    let keys = get_key_indexes(&names);
    let keys_normal = get_key_indexes_normal(&keys);

    let mut contains_integer_key = false;
    for (k, _) in &keys {
        if k.starts_with("?") {
            contains_integer_key = true;
        }
    }

    let num_keys = *&keys.len() as u32;
    let has_keys = (num_keys > 0);
    let has_indexes = (contains_integer_key || (max_index > 0 && num_keys != max_index));


    let types_used = if has_keys && has_indexes {
        PlaceholderTypes::IndexAndKey
    } else if has_keys {
        PlaceholderTypes::Key
    } else if has_indexes {
        PlaceholderTypes::Index
    } else {
        PlaceholderTypes::None
    };


    PlaceholderMeta {
        max_index,
        names,
        keys,
        keys_normal,
        contains_integer_key,

        types_used,
    }
}

pub fn get_ph_names(stmt: *mut sqlite3_stmt) -> HashMap<String, u32> {
    let mut r: HashMap<String, u32> = HashMap::new();

    for i in 1..=bp_count(stmt) {
        if let Some(k) = bp_name(stmt, i) {
            r.insert(k, i);
        }
    }

    r
}


pub fn get_key_indexes(names: &HashMap<String, u32>) -> HashMap<String, u32> {
    let mut r: HashMap<String, u32> = HashMap::new();

    for (k, i) in names {
        if !k.starts_with("?") {
            r.insert(k.clone(), i.clone());
        }
    }

    r
}

pub fn get_key_indexes_normal(keys: &HashMap<String, u32>) -> HashMap<String, HashSet<u32>> {
    let mut r: HashMap<String, HashSet<u32>> = HashMap::new();

    for (k, i) in keys {
        if k.starts_with("?") ||
            k.starts_with(":") ||
            k.starts_with("@") ||
            k.starts_with("$") {
            let normal_k = k.split_at(1).1.to_owned();


            let indexes = r.entry(normal_k).or_insert_with(|| HashSet::new());
            indexes.insert(i.clone());

        } else {
            panic!("Query placeholders should not have prefixes other than ?, :, @, $");
        }
    }

    r
}


// @see https://www.sqlite.org/c3ref/bind_parameter_count.html
// - "returns the index of the largest (rightmost) parameter"
// - Indexes start at 1, not 0.
pub fn bp_count(stmt: *mut sqlite3_stmt) -> u32 {
    unsafe {
        sqlite3_bind_parameter_count(stmt) as u32
    }
}


// `sqlite3_bind_parameter_name`
//      - Returns null for `?`.
//      - Returns string for parameters: "?NNN" or ":AAA" or "@AAA" or "$AAA"
//      - Includes the prefix.
// @see https://www.sqlite.org/c3ref/bind_parameter_name.html
fn bp_name(stmt: *mut sqlite3_stmt, i: u32) -> Option<String> {

    let opt = unsafe {
        let s = sqlite3_bind_parameter_name(stmt, i as c_int);
        let ptr = s as *const c_char;

        if ptr.is_null() {
            return None;
        }


        Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
    };


    opt
}

// @see https://www.sqlite.org/c3ref/bind_parameter_index.html
fn bp_index(stmt: *mut sqlite3_stmt, key: &str) -> Option<u32> {
    let k_c = to_cstr(key);

    let i = unsafe {
        sqlite3_bind_parameter_index(stmt, k_c.as_ptr()) as u32
    };

    // No marching param found.
    if i == 0 {
        return None;
    }

    Some(i)
}