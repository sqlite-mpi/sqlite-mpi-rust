use std::convert::TryFrom;

use std::collections::HashMap;
use std::collections::HashSet;
use std::os::raw::{
    c_int,
    c_char,
    c_void,
};
use std::ptr;
use std::mem::MaybeUninit;
use std::ffi::CStr;

extern crate chrono;


use serde::{
    Deserialize,
    Serialize,
    Deserializer,
};

use crate::cffi::{
    // Objects
    sqlite3,
    sqlite3_stmt,

    // Types
    sqlite3_int64,

    // Functions
    sqlite3_prepare_v2,
    sqlite3_step,
    sqlite3_reset,
    sqlite3_finalize,
    sqlite3_data_count,
    sqlite3_column_type,
    sqlite3_column_int64,
    sqlite3_column_double,
    sqlite3_column_text,
    sqlite3_column_blob,
    sqlite3_column_bytes,
    sqlite3_column_origin_name,
    sqlite3_column_name,
    sqlite3_stmt_readonly,
    sqlite3_sql,
    sqlite3_changes,
    sqlite3_bind_null,
    sqlite3_bind_int64,
    sqlite3_bind_double,
    sqlite3_bind_text,
    sqlite3_bind_blob,

    // Constants
    SQLITE_INTEGER,
    SQLITE_FLOAT,
    SQLITE_TEXT,
    SQLITE_BLOB,
    SQLITE_NULL,

    // Functions that return constants.
    SQLITE_TRANSIENT,
};


use crate::errmap;
use crate::err;
use crate::fns;
use crate::placeholder;
use crate::db::DbHandle;

use fns::to_cstr;
use errmap::PrimaryRC;

use err::{
    ReturnStatus,
    to_return_status_cint,
    to_return_status_cint_db_err,
};


use placeholder::PlaceholderMeta;


use crate::placeholder::PlaceholderTypes;


// @see https://www.sqlite.org/c3ref/stmt.html
// @see https://www.sqlite.org/cintro.html ("> 50% of the query is used in creating the statement.")
//      - @todo/medium cache statements
#[derive(Debug)]
pub struct StmtHandle<'a> {

    // Tree path: [db, stmt]
    // A parent db has a child stmt.
    // Make the child(stmt) ref the parent(db)
    // If `sqlite3_step(stmt)` returns `SQLITE_ERROR`, `sqlite3_errmsg(db)` needs to be called to get the specific runtime error (E.g. db constraint fail).
    // A stmt references the db it belongs to; this forces using the correct db connection to get the error.
    // Lifetimes: The db lives as long or longer than the `stmt`. `drop` is called outside->inner for a single struct,
    // but *scopes call `drop` on owned variables in the reverse order they were created.*
    // A stmt always has a db, but a db does not always have a stmt. This avoids Option and unnecessary if branches.

    db: &'a DbHandle,
    stmt: *mut sqlite3_stmt,
    pub placeholder_meta: PlaceholderMeta,
    pub is_read_only: bool,
}


impl StmtHandle<'_> {
    pub fn new<'a>(db: &'a DbHandle, q: &str) -> Result<StmtHandle<'a>, ReturnStatus> {
        let stmt = StmtHandle::new_stmt(db, q)?;

        // Compute place holder data from `stmt` before `StmtHandle` creation so its always `Some`.
        // Assumption: placeholder data never changes for the same `stmt`, will be needed in most cases.
        let placeholder_meta = PlaceholderMeta::new(stmt);
        let is_read_only = is_read_only(stmt);

        Ok(
            StmtHandle {
                db,
                stmt,
                placeholder_meta,
                is_read_only,
            }
        )
    }

    fn new_stmt<'a>(db: &'a DbHandle, q: &str) -> Result<*mut sqlite3_stmt, ReturnStatus> {
        let mut mu = MaybeUninit::uninit();
        let mut stmt: *mut sqlite3_stmt = mu.as_mut_ptr();
        let q_c = to_cstr(q);

        let r = unsafe {
            sqlite3_prepare_v2(
                db.db,
                q_c.as_ptr(),
                q_c.to_bytes_with_nul().len() as ::std::os::raw::c_int,
                &mut stmt,
                ptr::null_mut(),
            )
        };

        // Examples of errors:
        // - SQLITE_ERROR, 1, "no such table"
        //      - For SQL syntax or schema errors, no specific/extended result code is used.
        // On error, stmt will be null.
        // Note: This does not need to be freed.
        // - "sqlite3_finalize() on a NULL pointer is a harmless no-op."
        to_return_status_cint_db_err(&r, db.db)?;


        Ok(stmt)
    }


    pub fn run(&self) -> Result<RSet, ReturnStatus> {
        let StmtHandle { db, stmt, is_read_only, .. } = &self;


        let mut rset = RSet {
            is_read_only: *is_read_only,
            is_iud: is_iud(*stmt),

            ..Default::default()
        };


        let mut add_row = |row: Vec<Val>| {
            if rset.rows.data.len() == 0 {
                // First row.

                rset.col_names = get_headers(*stmt);
                rset.num_cols = row.len() as u32;
            }
            rset.rows.data.push(row);
        };


        loop {
            let r = unsafe { sqlite3_step(*stmt) };
            let step = to_return_status_cint_db_err(&r, db.db);

            match step {
                Err(status) => {
                    match status.primary.id {
                        PrimaryRC::SQLITE_BUSY => {
                            // @todo/important If "COMMIT" or outside of transaction, retry `step()`, else "ROLLBACK"
                            // @see https://www.sqlite.org/c3ref/step.html
//                            dbg!(("sqlite3_step: ", &status));

                            return Err(status);
                        }
                        _ => {
                            // SQLITE_ERROR etc

                            // Errors: (FULL, IOERR, BUSY, NOMEM)
                            // @todo/low "It is recommended that applications respond to the errors listed above by explicitly issuing a ROLLBACK command"
                            // @see https://www.sqlite.org/lang_transaction.html
                            dbg!(format!("sqlite3_step: {:?}", status));
                            return Err(status);
                        }
                    }
                }
                Ok(status) => {
                    match status.primary.id {
                        PrimaryRC::SQLITE_ROW => {
                            let row = get_row(*stmt);
                            add_row(row);
                            continue;
                        }

                        PrimaryRC::SQLITE_DONE => {
                            // `sqlite3_reset`
                            // - Resets the VM so the `sqlite3_step` can be called on the `stmt` again.
                            // - Will return last error code (but `sqlite3_step` error should be the same).
                            // - Keeps bindings (call `sqlite3_clear_bindings` to clear them).
                            // - `sqlite3_step` automatically calls this.
                            unsafe {
                                sqlite3_reset(*stmt);
                            }

                            // @see https://www.sqlite.org/pragma.html#pragma_data_version (determine if file has changed).
                            if rset.is_iud {
                                rset.rows_changed = Some(get_changes(db.db));
                            }

                            rset.num_rows = rset.rows.data.len() as u32;
                            return Ok(rset);
                        }
                        _ => {
                            // Only SQLITE_OK remaining. Create enum of non-error values so compiler can verify `match`?
                            panic!("sqlite3_step Ok values are not exhaustive. {:?}", status)
                        }
                    }
                }
            }
        }
    }


    fn bind_kv_check_errors(
        types_used: &PlaceholderTypes,
        keys_normal: &HashMap<String, HashSet<u32>>,
        key_val: &KeyVal,
    ) -> Result<(), ErrorBind> {
        if *types_used != PlaceholderTypes::Key {
            return Err(
                ErrorBind {
                    kind: ErrorBindType::PlaceholderDataTypeNotCompatible,
                    msg: "Input data is key-based, but the query string contains index-based (?, ?NNN) placeholders.".to_string(),
                }
            );
        }

        let data_keys: HashSet<String> = key_val.data.keys().cloned().collect();
        let placeholder_keys: HashSet<String> = keys_normal.keys().cloned().collect();

        // Alternative: `placeholder_keys.is_subset(&data_keys)`
        // `a.difference(b)` = `a - b`
        // let missing = placeholder_keys.difference(&data_keys);
        let missing = placeholder_keys.difference(&data_keys);

        let has_missing_keys = &missing.clone().count() > &0;

        if has_missing_keys {
            return Err(
                ErrorBind {
                    kind: ErrorBindType::MissingKeysInData,
                    msg: format!("All key-based placeholders must have data provided. Missing keys {:?}", &missing),
                }
            );
        };

        Ok(())
    }

    // @see https://www.sqlite.org/c3ref/bind_blob.html
    // - Issue: "Unbound parameters are interpreted as NULL".
    // @todo/low Should `key_val` be passed ownership and assigned to the `StmtHandle` struct for debugging? Does SQLite copy the values on bind?
    pub fn bind_kv(&self, key_val: &KeyVal) -> Result<(), ErrorBind> {
        // Assumption: `stmt` is at the start state (is "reset" with no previous data bindings).
        // If re-using the same stmt, reset on complete/re-start.

        let StmtHandle {
            stmt,
            placeholder_meta: PlaceholderMeta {
                types_used,
                keys_normal,
                ..
            },
            ..
        } = self;

        StmtHandle::bind_kv_check_errors(types_used, keys_normal, &key_val)?;

        for (k, indexes) in keys_normal {
            for i in indexes {
                match self.bind_val(i, &key_val.data[k]) {
                    Err(rs) => return Err(ErrorBind {
                        kind: ErrorBindType::ReturnStatus(rs),
                        msg: "Bind failed with SQLite error code".to_string(),
                    }),
                    Ok(_) => continue
                }
            }
        }


        Ok(())
    }


    fn bind_index_check_errors(
        types_used: &PlaceholderTypes,
        max_index: &u32,
        vals: &Vec<Val>,
    ) -> Result<(), ErrorBind> {
        if *types_used != PlaceholderTypes::Index {
            return Err(
                ErrorBind {
                    kind: ErrorBindType::PlaceholderDataTypeNotCompatible,
                    msg: "Input data is index-based, but the query string contains no placeholders, or includes key-based (:x, $y, @z) placeholders.".to_string(),
                }
            );
        }

        if (vals.len() as u32) < *max_index {
            return Err(
                ErrorBind {
                    kind: ErrorBindType::MissingIndexesInData,
                    msg: format!("Max index-based placeholder in query = {}. Provided {} input data elements. Every placeholder index must have data bound.", max_index, vals.len()),
                }
            );
        }

        Ok(())
    }

    pub fn bind_index(&self, vals: &IndexVal) -> Result<(), ErrorBind> {
        let StmtHandle {
            stmt,
            placeholder_meta: PlaceholderMeta {
                types_used,
                max_index,
                ..
            },
            ..
        } = self;

        StmtHandle::bind_index_check_errors(types_used, max_index, &vals)?;

        for (i, v) in vals.iter().enumerate() {
            let target_index = ((i + 1) as u32);
            match self.bind_val(&target_index, &v) {
                Err(rs) => return Err(ErrorBind {
                    kind: ErrorBindType::ReturnStatus(rs),
                    msg: "Bind failed with SQLite error code".to_string(),
                }),
                Ok(_) => continue
            }
        }


        Ok(())
    }


    fn bind_val(&self, index: &u32, val: &Val) -> Result<(), ReturnStatus> {
        let StmtHandle { db, stmt, .. } = self;
        let i = *index as c_int;

        let r = match val {
            Val::I64(v) => {
                unsafe {
                    // Note: `sqlite3_int64` = `::std::os::raw::c_longlong`
                    sqlite3_bind_int64(*stmt, i, *v as sqlite3_int64)
                }
            }
            Val::F64(v) => {
                unsafe {
                    sqlite3_bind_double(*stmt, i, *v)
                }
            }
            Val::String(v) => {
                let cstr = to_cstr(&v);
                unsafe {
                    sqlite3_bind_text(
                        *stmt,
                        i,
                        cstr.as_ptr(),
                        -1 as c_int,
                        SQLITE_TRANSIENT(),
                    )
                }
            }
            Val::Null => {
                unsafe {
                    sqlite3_bind_null(*stmt, i)
                }
            }
            Val::Blob(v) => {
                // @todo/low Test when length=0, length>c_int.max
                unsafe {
                    sqlite3_bind_blob(
                        *stmt,
                        i,
                        v.as_ptr() as *const c_void,
                        v.len() as c_int,
                        SQLITE_TRANSIENT(),
                    )
                }
            }
        };

        to_return_status_cint_db_err(&r, db.db)?;

        Ok(())
    }
}


#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub struct ErrorBind {
    pub kind: ErrorBindType,
    pub msg: String,
}

// @todo/low Just use `enum(String)` for err message instead of struct with (kind, msg)?
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub enum ErrorBindType {
    PlaceholderDataTypeNotCompatible,
    MissingKeysInData,
    MissingIndexesInData,
    ReturnStatus(ReturnStatus),
}


pub type IndexVal = Vec<Val>;


// Placeholder data,
#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct KeyVal {
    #[serde(flatten)]
    pub data: HashMap<String, Val>
}

// @todo/low Also type for index-based/return row of data?

// @see https://www.sqlite.org/c3ref/c_blob.html
// @see https://www.sqlite.org/datatype3.html
// @see https://www.sqlite.org/c3ref/column_blob.html
#[derive(Debug)]
#[allow(non_camel_case_types)]
enum Type {
    SQLITE_INTEGER,
    SQLITE_FLOAT,
    SQLITE_TEXT,
    SQLITE_NULL,
    SQLITE_BLOB,
}


// Type for mapping to/from SQLite types to Rust types.
// Input: for assigning data values to placeholders.
// Output: for representing return result set row values.
// @todo/low Should these be separate types?
// Alternative: Just wrap `serde_json::Value` in struct, and `impl` methods to get SQLite specific values?
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[derive(PartialEq)]
#[derive(Clone)]
#[serde(untagged)] // Deserialize to `[1]` instead of `[{"I64": 1}]` for `vec![Val::I64(1)]`. Useful for testing `assert_eq!(a, b)`.
//#[serde(deserialize_with = "json_scalar_to_rust")]
//#[serde(try_from = "serde_json::Value")]
//#[serde(from = "serde_json::Value")]
pub enum Val {
    I64(i64),
    F64(f64),
    String(String),
    Null,
    Blob(Vec<u8>),
    // Note: No `bool` as SQLite uses `i64` 1 or 0.
}

// @todo/medium Propagate failures with `try_from`.
//impl TryFrom<serde_json::Value> for Val {
//    type Error = &'static str;
//
//    fn try_from(v: serde_json::Value) -> Result<Self, Self::Error> {
//        Ok(Val::String("wow".to_string()))
//    }
//}

//
//impl From<serde_json::Value> for Val {
//    fn from(v: serde_json::Value) -> Self {
////        ParseError::ParseError(se)
//        dbg!(v);
//        Val::String("wow".to_string())
//    }
//}


//fn json_to_rust<S>(x: &f32, s: S) -> Result<S::Ok, S::Error>
//    where
//        S: Deserializer,
//{
//    s.serialize_f32(x.round())
//}

//fn json_scalar_to_rust<D>(de: &mut D) -> Result<Val, D::Error>
//    where D: Deserializer
//{
//    Ok(Val::String("wow".to_string()));
//
////    let s: String = Deserialize::deserialize(de)?;
////    if s.is_empty() {
////        Err(serde::Error::custom("empty string"))
////    } else {
////        Ok(s)
////    }
//}


fn get_cell_type(code: u32) -> Type {
    match code {
        SQLITE_INTEGER => Type::SQLITE_INTEGER,
        SQLITE_FLOAT => Type::SQLITE_FLOAT,
        SQLITE_TEXT => Type::SQLITE_TEXT,
        SQLITE_BLOB => Type::SQLITE_BLOB,
        SQLITE_NULL => Type::SQLITE_NULL,
        _ => panic!("Unknown cell type, code={}", code)
    }
}


// Note: Requires `SQLITE_ENABLE_COLUMN_METADATA` compile flag.
fn get_headers(stmt: *mut sqlite3_stmt) -> Vec<ColName> {
    let mut headers = vec![];

    let num_cols = get_num_cols(stmt);

    for n in 0..num_cols {
        let n_c = n as c_int;


        let (ptr_a, ptr_b) = unsafe {

            // @see https://www.sqlite.org/c3ref/column_name.html
            // @see https://www.sqlite.org/c3ref/column_database_name.html

            let a: *const c_char = sqlite3_column_name(stmt, n_c);
            let b: *const c_char = sqlite3_column_origin_name(stmt, n_c);

            assert!(!a.is_null());

            (
                CStr::from_ptr(a as *const c_char),
                if b.is_null() {
                    None
                } else {
                    Some(CStr::from_ptr(b as *const c_char))
                }
            )
        };


        let name = ptr_a.to_string_lossy().into_owned();
        let name_origin = match ptr_b {
            None => None,
            Some(ptr) => Some(ptr.to_string_lossy().into_owned())
        };

        let c_name = ColName { name, name_origin };

        headers.push(c_name);
    }

    return headers;
}


fn get_num_cols(stmt: *mut sqlite3_stmt) -> u32 {
    unsafe {
        sqlite3_data_count(stmt) as u32
    }
}

// @see https://www.sqlite.org/c3ref/column_blob.html
// - "pointers returned are valid until a type conversion occurs as described above, or until sqlite3_step() or sqlite3_reset() or sqlite3_finalize() is called. The memory space used to hold strings and BLOBs is freed automatically."
fn get_row(stmt: *mut sqlite3_stmt) -> Vec<Val> {
    let mut rw = vec![];

    let num_cols = get_num_cols(stmt);

    for n in 0..num_cols {
        let n_c = n as c_int;

        let t = get_cell_type(unsafe {
            sqlite3_column_type(stmt, n_c) as u32
        });

        let v = match t {
            Type::SQLITE_INTEGER => {
                // @todo/low Use smaller int type if possible.
                // @todo/low Write tests around for the range of JSON's number type (float).
                Val::I64(
                    unsafe { sqlite3_column_int64(stmt, n_c) } as i64
                )
            }
            Type::SQLITE_FLOAT => {
                Val::F64(
                    unsafe { sqlite3_column_double(stmt, n_c) } as f64
                )
            }
            Type::SQLITE_TEXT => {
                // Note: `sqlite3_column_text` is UTF-8, and so is Rusts `String`
                let ptr = unsafe {
                    let s = sqlite3_column_text(stmt, n_c);

                    // @todo/low Should the length of string in bytes be read from SQLite here? E.g: `sqlite3_column_bytes`
                    CStr::from_ptr(s as *const c_char)
                };

                Val::String(
                    ptr.to_string_lossy().into_owned()
                )
            }
            Type::SQLITE_NULL => {
                Val::Null
            }
            Type::SQLITE_BLOB => {
                let (ptr, num_bytes) = unsafe {
                    (
                        sqlite3_column_blob(stmt, n_c) as *const u8,
                        sqlite3_column_bytes(stmt, n_c) as u32
                    )
                };

                Val::Blob(to_owned_vec(ptr, &num_bytes))
            }
        };

        rw.push(v);
    }

    return rw;
}


fn to_owned_vec(ptr: *const u8, num_bytes: &u32) -> Vec<u8> {
    let n = *num_bytes as usize;
    let mut dst = Vec::with_capacity(n);

    unsafe {
        dst.set_len(n);
        ptr::copy(ptr, dst.as_mut_ptr(), n);
    }

    dst
}

// @see https://www.sqlite.org/c3ref/stmt_readonly.html
// Will the query directly write to the database file?
// - Indirect changes (e.g. from user functions) are not counted.
fn is_read_only(stmt: *mut sqlite3_stmt) -> bool {
    let i = unsafe {
        sqlite3_stmt_readonly(stmt) as u32
    };

    i != 0
}

// @see https://www.sqlite.org/c3ref/changes.html
// If query starts with (INSERT|UPDATE|DELETE).
// `sqlite_changes` only counts rows modified by these keywords.
fn is_iud(stmt: *mut sqlite3_stmt) -> bool {
    let ptr = unsafe {
        let s = sqlite3_sql(stmt);
        CStr::from_ptr(s as *const c_char)
    };

    let q = ptr.to_string_lossy().into_owned().trim().to_lowercase();

    (
        q.starts_with("insert") ||
            q.starts_with("update") ||
            q.starts_with("delete")
    )
}

// Num rows modified.
// - INSERT|UPDATE|DELETE only.
// @see https://www.sqlite.org/c3ref/changes.html
fn get_changes(db: *mut sqlite3) -> u64 {
    unsafe {
        sqlite3_changes(db) as u64
    }
}


// Note: `SELECT`s are not the only queries that return results (`PRAGMA`).
// `sqlite3_step` is more of a general "return tabular data".

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct RSet {
    // @todo/low Should this be part of a `meta` type?
    // Copy `is_read_only` into result set so that clients can perform `if is_read_only then (commit | rollback)`.
    pub is_read_only: bool,

    // iud = INSERT|UPDATE|DELETE
    pub is_iud: bool,
    // @todo/low total_changes?
    pub rows_changed: Option<u64>,

    // @todo/low col AS x names, db, and table meta data. @see https://www.sqlite.org/c3ref/column_database_name.html
    pub col_names: Vec<ColName>,
    pub num_cols: u32,
    pub num_rows: u32,

    pub rows: Rows,
}


#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct Rows {
    pub data: Vec<Vec<Val>>
}


#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct ColName {
    pub name: String,
    pub name_origin: Option<String>,
}


impl Default for RSet {
    fn default() -> RSet {
        RSet {
            is_read_only: false,
            is_iud: false,
            rows_changed: None,

            col_names: vec![],
            num_cols: 0,
            num_rows: 0,
            rows: Rows { data: vec![] },
        }
    }
}


impl Drop for StmtHandle<'_> {
    fn drop(&mut self) {
//        let s = "stmt.d()";
//        dbg!(s);

        let stmt = self.stmt;
        assert!(!(stmt.is_null()), "Drop called but StmtHandle.stmt was null.");

        // @see https://www.sqlite.org/c3ref/finalize.html
        // - Every `sqlite3_stmt` must be run through `finalize` to avoid memory leaks.
        // - `finalize` can be called at any time.
        // - `finalize` returns the last error for the last evaluation of the statement, or SQLITE_OK otherwise.

        let r = unsafe { sqlite3_finalize(stmt) };
        let finalize = to_return_status_cint(&r);


        // This will return the last error of the statement.
        // Even if `sqlite3_finalize` returns an error, it still needs to be called to free memory of the statement.
        if !finalize.is_ok {
            dbg!(finalize);
        }

        // @todo/medium Make sure pointer memory is freed/zeroed/null;
    }
}
