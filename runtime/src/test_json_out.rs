use std::collections::HashMap;


use crate::json_out::*;
use crate::messages::*;

use serde_json::{
    json,
    Error,
};
use sma::Params;

use sqlite3ffi::stmt::RSet;
use sqlite3ffi::stmt::KeyVal;
use sqlite3ffi::err::{to_return_status, ReturnStatus};
use sqlite3ffi::errmap::PrimaryRC;
use sqlite3ffi::db::BindRunError;


use crate::messages::{
    OutMsg::*
};


use serde_json::Value;


struct ErrTx {}

impl ErrTx {
    fn enum_to_json(t: TxOpErr) -> String {
        let o = Tx(Err(t));
        out_rs_to_json(&o).unwrap()
    }

    // A set of all `TxOpErr` variants.
    // Note: Cannot iterate over all variants
    // - @see https://stackoverflow.com/questions/21371534/in-rust-is-there-a-way-to-iterate-through-the-values-of-an-enum
    fn get_variants_set_a() -> Vec<TxOpErr> {
        let mut o = vec![];
        let busy: ReturnStatus = PrimaryRC::SQLITE_BUSY.into();

        o.push(TxOpErr::InvalidTxId);
        o.push(TxOpErr::ReturnStatus(busy.clone()));
        o.push(TxOpErr::BindRunError(BindRunError::ReturnStatus(busy.clone())));

        // @todo/low Test all variants.

        o
    }
}


/**
Tests for converting between JSON and Rust types.
- Loosely describes the JSON schema and request/response protocol.
- Ensures custom JSON to Rust types map correctly.
    - Enums in Rust are not serialized cleanly as there is no semantic equal to enums in JSON.
- Defines general JSON error object and its map to/from a Rust error.
**/
#[cfg(test)]
mod tests {
    use super::*;
    use sqlite3ffi::stmt::Val;
    use serde_json::Value;
    use serde_json::Value as V;


    fn res_ok_file() -> String {
        let o = File(Ok(
            TxIdOnly {
                tx_id: "abc".to_string()
            }
        ));

        out_rs_to_json(&o).unwrap()
    }

    fn res_ok_tx() -> String {
        let o = Tx(Ok(RSet::default()));
        out_rs_to_json(&o).unwrap()
    }

    fn res_err_file_no_dir() -> String {
        let o = File(Err(FileOpErr::FileDirectoryDoesNotExist));
        out_rs_to_json(&o).unwrap()
    }

    fn res_err_file_rs() -> String {
        let e = PrimaryRC::SQLITE_BUSY.into();

        let o = File(Err(FileOpErr::ReturnStatus(e)));
        out_rs_to_json(&o).unwrap()
    }


    // Assert: Result matches JSON schema `{ok: true, res_type: "x", res: {}}`.
    // Note: This is the JSON schema clients are expecting.
    fn is_ok_res(json: &Value) -> bool {
        match json {
            Value::Object(o) => {
                match (
                    o.get("ok"),
                    o.get("res_type"),
                    o.get("res")
                ) {
                    (
                        Some(V::Bool(true)),
                        Some(V::String(_)),
                        Some(V::Object(_))
                    ) => true,
                    _ => false
                }
            }
            _ => false
        }
    }

    // Assert: Result matches JSON schema `{ok: false, error: {error_type: "", message: "", data: {return_status: {}}}}`.
    fn is_err_res(json: &Value) -> bool {
        match json {
            Value::Object(o) => {
                match (
                    o.get("ok"),
                    o.get("error"),
                ) {
                    (
                        Some(V::Bool(false)),
                        Some(V::Object(e)),
                    ) => match (e.get("error_type"), e.get("message"), e.get("data")) {
                        (
                            Some(V::String(_)),
                            Some(_),
                            Some(V::Object(_)),
                        ) => true,
                        _ => false
                    },
                    _ => false
                }
            }
            _ => false
        }
    }

    fn err_has_rs(json: &Value) -> bool {
        if let V::Object(_) = json["error"]["data"]["return_status"] {
            return true;
        }
        false
    }


    fn is_ok_res_str(json: &String) -> bool {
        let v = serde_json::from_str(json).unwrap();
        is_ok_res(&v)
    }

    fn is_err_res_str(json: &String) -> bool {
        let v = serde_json::from_str(json).unwrap();
        is_err_res(&v)
    }


    #[test]
    fn test_ok() {
        let a = res_ok_file();
        let b = res_ok_tx();

        assert!(is_ok_res_str(&a));
        assert!(is_ok_res_str(&b));
    }

    #[test]
    fn test_err_file() {
        let a = res_err_file_no_dir();
        assert!(is_err_res_str(&a));


        let b = res_err_file_rs();
        assert!(is_err_res_str(&b));

        let v = serde_json::from_str(&b).unwrap();
        assert!(err_has_rs(&v));
    }


    #[test]
    fn test_err_tx() {
        let a = ErrTx::get_variants_set_a();

        for tx_op_err in a {
            let string = ErrTx::enum_to_json(tx_op_err);
            assert!(is_err_res_str(&string));
        }
    }
}
