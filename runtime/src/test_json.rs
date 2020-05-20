use std::collections::HashMap;

use crate::json_in::*;
use crate::messages::*;
use crate::runtime::{
    InMsgWithId
};

use serde_json::{
    json,
    Error,
};
use sma::Params;

use sqlite3ffi::stmt::KeyVal;


#[cfg(test)]
mod tests {
    use super::*;
    use sqlite3ffi::stmt::Val;
    use serde_json::Value;


    #[test]
    fn test_read_tx() {
        let id = "example-uuid".to_string();
        let from = json!({
            "id": id,
            "fn": "file/get_read_tx",
            "args": {
                "file": "a/b/c.sqlite"
            }
        }).to_string();


        let to = InMsgWithId {
            id,
            msg: InMsg::File(FileOp::GetReadTx(ArgsFile { file: "a/b/c.sqlite".to_string() })),
        };

        assert_eq!(in_json_to_rs(&from).expect("Ok"), to);
    }


    // @todo/medium Output messages.
    // @todo/low What about JSON strings as a value?
    // @todo/low Blob data.


    #[test]
    fn test_args_index() {

        // Assert: `parse` can read `stringify` output using the same `serde` macro rules.
        let a = ArgsTxParams {
            tx_id: "1".to_string(),
            q: "123".to_string(),
            params: Params::Index(vec![Val::String("abc".to_string())]),
        };
        let j = serde_json::to_string(&a).expect("Ok");


        // Assert: JSON structure matches the external JSON interface.
        let a1: Value = serde_json::from_str(&j).expect("Ok");
        match &a1["index_based"][0] {
            Value::String(s) => assert_eq!("abc", s),
            _ => assert!(false)
        }

        // Assert: Strongly typed parsing parses back to original type.
        let args: ArgsTxParams = serde_json::from_str(&j).expect("Ok");
        match args.params {
            Params::Index(v) => {
                assert_eq!(vec![Val::String("abc".to_string())], v);
            }
            _ => assert!(false)
        }


        // Assert: All SQLite param types work.
        let from = json!({
            "tx_id": "123",
            "q": "SELECT 1",
            // Note: no `bool`
            "index_based": [1, "string", 1.12345]
        }).to_string();

        let args: ArgsTxParams = serde_json::from_str(&from).expect("Ok");
    }


    #[test]
    fn test_args_key() {
        let data: HashMap<String, Val> = [
            ("i64".to_string(), Val::I64(123)),
            ("f64".to_string(), Val::F64(1.12345)),
            ("string_key".to_string(), Val::String("string_val".to_string())),
        ].iter().cloned().collect();

        // Assert: `parse` can read `stringify` output using the same `serde` macro rules.
        let a = ArgsTxParams {
            tx_id: "1".to_string(),
            q: "123".to_string(),
            params: Params::Key(KeyVal { data: data.clone() }),
        };
        let j = serde_json::to_string(&a).expect("Ok");


        // Assert: JSON structure matches the external JSON interface.
        let a1: Value = serde_json::from_str(&j).expect("Ok");
        match &a1["key_based"]["string_key"] {
            Value::String(s) => assert_eq!("string_val", s),
            _ => assert!(false)
        }


        // Assert: Strongly typed parsing parses back to original type.
        let args: ArgsTxParams = serde_json::from_str(&j).expect("Ok");
        match args.params {
            Params::Key(kv) => {
                assert_eq!(kv.data, data);
            }
            _ => assert!(false)
        }

        // Assert: All SQLite param types work.
        let from = json!({
            "tx_id": "123",
            "q": "SELECT 1",
            "key_based": {
                "i64": 1,
                "f64": 1.12345,
                "string": "string",
                // Note: No `bool`
            }
        }).to_string();

        let args: ArgsTxParams = serde_json::from_str(&from).expect("Ok");
    }


    #[test]
    fn test_args_params_errors() {
        let errors = vec![
            // SQLite does not have a `bool` value, it uses 1 or 0.
            // Error: `data did not match any variant of untagged enum Val`
            json!({
                "tx_id": "123",
                "query": "SELECT 1",
                "index_based": [true, false]
            }).to_string(),
            json!({
                "tx_id": "123",
                "query": "SELECT 1",
                "key_based": {
                    "i64": 1,
                    "f64": 1.12345,
                    "string": "string",
                    "bool": true
                }
            }).to_string(),
            json!({
                "tx_id": "123",
                "query": "SELECT 1"
            }).to_string()
        ];


        for j in errors {
            let args: Result<ArgsTxParams, Error> = serde_json::from_str(&j);
            assert!(&args.is_err());
        }
    }


    #[test]
    fn test_errors() {
        let id = "example-uuid".to_string();

        let errors = vec![
            json!({
                "id": id,
                "fn": "file/get_read_tx___",
                "args": {
                    "file": "a/b/c.sqlite"
                }
            }).to_string(),
            json!({
                "id": id,
                "fn": "file/get_read_tx",
                "args": {}
            }).to_string(),
            json!({
                "id": id,
                "fn": "file/get_read_tx",
                "args": {
                    "file": null
                }
            }).to_string(),
            json!({
                "id": id,
                "fn": null,
                "arg___": {
                    "file": "a/b/c.sqlite"
                }
            }).to_string(),
            json!({
                "id": id,
                "fn___": "file/get_read_tx",
                "args": {
                    "file": "a/b/c.sqlite"
                }
            }).to_string(),
            json!({
                "id": id,
                "fn": "file/get_read_tx",
                "arg___": {
                    "file": "a/b/c.sqlite"
                }
            }).to_string()
        ];


        for j in errors {
            let r = in_json_to_rs(&j);
            assert!(&r.is_err());
        }
    }
}
