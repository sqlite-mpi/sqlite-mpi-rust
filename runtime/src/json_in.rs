use serde::{Deserialize, Serialize};
use serde_json::{
    Error as SerdeError,
    Map,
    Value,
    from_value,
};
use crate::messages::*;

use crate::messages::{
    InMsg::*,
    FileOp::*,
    TxOp::*,
};

use crate::{
    InMsgWithId,
    InMsgId,
};


#[derive(Serialize, Deserialize)]
struct FnArgs {
    id: InMsgId,

    #[serde(rename = "fn")]
    fn_name: String,
    args: Map<String, Value>,
}


// @todo/low Convert this to a general trait that will parse different types of format from bytes to a Rust representation.
// - The Rust representation allows full use of the type system, but may not be represented as concise as possible in a string format like JSON.
// - E.g nested `enum` semantics are not well represented in JSON.
//      - @see https://serde.rs/enum-representations.html

#[derive(Debug)]
pub enum ParseError {
    DataError(DataError),
    ParseError(SerdeError),
}

#[derive(Debug)]
pub enum DataError {
    InvalidFunction,
    InvalidArgs,
}

impl From<SerdeError> for ParseError {
    fn from(se: SerdeError) -> Self {
        ParseError::ParseError(se)
    }
}

// @todo/low Allow encode/decode to JSON to work both ways (instead of a single custom JSON->Rust function).
pub fn in_json_to_rs(s: &String) -> Result<InMsgWithId, ParseError> {
    let fn_args: FnArgs = serde_json::from_str(s)?;

    let FnArgs { id, fn_name, args } = fn_args;

    // Wrap in general JSON value type so Rust can convert it to a concrete arg type for the given fn enum variant.
    let a = Value::Object(args);


    let msg = match fn_name.as_ref() {
        "file/get_read_tx" => {
            File(
                GetReadTx(from_value(a)?)
            )
        }
        "file/get_write_tx" => {
            File(
                GetWriteTx(from_value(a)?)
            )
        }
        "tx/q" => {
            Tx(
                Q(from_value(a)?)
            )
        }
        "tx/read" => {
            Tx(
                Read(from_value(a)?)
            )
        }
        "tx/write" => {
            Tx(
                Write(from_value(a)?)
            )
        }
        "tx/q_params" => {
            Tx(
                QParams(from_value(a)?)
            )
        }
        "tx/read_params" => {
            Tx(
                ReadParams(from_value(a)?)
            )
        }
        "tx/write_params" => {
            Tx(
                WriteParams(from_value(a)?)
            )
        }
        "tx/commit" => {
            Tx(
                Commit(from_value(a)?)
            )
        }
        "tx/rollback" => {
            Tx(
                Rollback(from_value(a)?)
            )
        }
        _ => {
            return Err(ParseError::DataError(DataError::InvalidFunction));
        }
    };


    Ok(InMsgWithId { id, msg })
}
