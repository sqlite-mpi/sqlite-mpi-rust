use std::collections::HashMap;

use serde::{
    Deserialize,
    Serialize,
    ser::SerializeStructVariant,
};
use serde_json::{
    Error as SerdeError,
    Map,
    Value,
    from_value,
};

use sqlite3ffi::{
    db::BindRunError,
    stmt::{
        RSet,
        ErrorBind,
        ErrorBindType,
    },
};

use sma::{
    ReadError,
    ReadBindRunError,
    WriteError,
    WriteBindRunError,
};


use crate::messages::*;

use crate::messages::{
    OutMsg::*,
    FileOp::*,
    TxOp::*,
};
use sqlite3ffi::err::ReturnStatus;

use crate::json_in::{
    ParseError,
    DataError
};


// @todo/low Write an interface for encoding to/from the JSON protocol.
// - Instead of standalone functions, assign to structs as traits?

// @todo/low Write a macro that creates every variant of enum recursively, and test its conversion to JSON.
// - E.g. Error cases can have a lot of different enum variants depending on the failure.
// - String paths can be generated for each enum variant.
//      - Enables usage in a general JSON error object, where the enum path is a single key.

// @todo/low Move JSON related functions into its own Rust module (encoding/decoding should be separate from the runtime, and allow using different formats without re-compiling the runtime).

#[derive(Debug)]
#[derive(Serialize)]
#[serde(tag = "res_type", content = "res")]
enum Res<'a> {
    TxIdOnly(&'a TxIdOnly),
    RSet(&'a RSet),
}

#[derive(Debug)]
#[derive(Serialize)]
struct OkRes<'a> {
    ok: bool,

    #[serde(flatten)]
    res: Res<'a>,
}

impl OkRes<'_> {
    pub fn new(res: Res) -> OkRes {
        OkRes {
            ok: true,
            res,
        }
    }
}

#[derive(Serialize)]
pub struct ErrRes {
    ok: bool,
    error: E,
}

impl ErrRes {
    pub fn new(error: E) -> ErrRes {
        ErrRes {
            ok: false,
            error,
        }
    }
}

#[derive(Serialize)]
pub struct E {
    // @todo/maybe use "err_type" instead, treat like a normal return type?
    error_type: String,
    message: Option<String>,
    data: ErrData,
}

// A general container for errors.
// This is a key value map of data associated with particular types.
// - In the future more keys may be added, for example:
//      - Rust stack/error line
//      - OS specific errors.
// Adding new keys in the future should not break existing clients.
// This is why its a key value map, and not just a single enum variant.
// `ErrData` contains the set of all possible keys and their types.
#[derive(Serialize)]
#[derive(Default)]
struct ErrData {
    #[serde(skip_serializing_if = "Option::is_none")]
    return_status: Option<ReturnStatus>
}


impl E {
    pub fn new_key(key: String) -> E {
        E {
            error_type: key,
            ..Default::default()
        }
    }

    pub fn new_key_msg(key: String, message: String) -> E {
        E {
            error_type: key,
            message: Some(message),
            ..Default::default()
        }
    }

    pub fn new_key_msg_status(key: String, message: String, return_status: ReturnStatus) -> E {
        let mut e = E {
            error_type: key,
            message: Some(message),
            ..Default::default()
        };

        e.data.return_status = Some(return_status);
        e
    }

    pub fn new_key_status(key: String, return_status: ReturnStatus) -> E {
        let mut e = E {
            error_type: key,
            ..Default::default()
        };

        e.data.return_status = Some(return_status);
        e
    }
}


impl Default for E {
    fn default() -> E {
        E {
            // This should always be set to a none zero length by the creator before encoding to JSON.
            // - At a minimum errors have a `error_type` string.
            error_type: "".to_string(),
            message: None,
            data: ErrData {
                ..Default::default()
            },
        }
    }
}

use crate::messages::{FileOpErr::*};

impl From<FileOpErr> for E {
    fn from(e: FileOpErr) -> Self {
        match e {
            FileDirectoryDoesNotExist => E::new_key(to_path(vec!["FileOp", "FileDirectoryDoesNotExist"])),
            FileOpErr::ReturnStatus(rs) => E::new_key_status(to_path(vec!["FileOp", "ReturnStatus"]), rs)
        }
    }
}

impl From<ParseError> for E {
    fn from(e: ParseError) -> Self {
        match e {
            ParseError::DataError(de) => match de {
                DataError::InvalidArgs => E::new_key(to_path(vec!["ParseError", "DataError", "InvalidArgs"])),
                DataError::InvalidFunction => E::new_key(to_path(vec!["ParseError", "DataError", "InvalidFunction"]))
            },
            ParseError::ParseError(se) => {
                E::new_key_msg(to_path(vec!["ParseError", "ParseError"]), "JSON: ".to_string() + &se.to_string())
            }
        }
    }
}



static SEP: &str = "/";

type Path = Vec<&'static str>;

fn to_path(p: Path) -> String {
    p.join(SEP).to_string()
}


fn bre_match(mut p: Path, bre: BindRunError) -> E {
    p.push("BindRunError");
    match bre {
        BindRunError::ReturnStatus(rs) => ks(p, "ReturnStatus", rs),
        BindRunError::ErrorBind(eb) => {
            p.push("ErrorBind");

            use ErrorBindType::*;
            match eb.kind {
                PlaceholderDataTypeNotCompatible => km(p, "PlaceholderDataTypeNotCompatible", eb.msg),
                MissingKeysInData => km(p, "MissingKeysInData", eb.msg),
                MissingIndexesInData => km(p, "MissingIndexesInData", eb.msg),
                ErrorBindType::ReturnStatus(rs) => ks(p, "ReturnStatus", rs),
            }
        }
    }
}


fn k(mut p: Path, a: &'static str) -> E {
    p.push(a);
    let k = to_path(p);
    E::new_key(k)
}

fn ks(mut p: Path, a: &'static str, b: ReturnStatus) -> E {
    p.push(a);
    let k = to_path(p);
    E::new_key_status(k, b)
}

fn km(mut p: Path, a: &'static str, b: String) -> E {
    p.push(a);
    let k = to_path(p);
    E::new_key_msg(k, b)
}


use crate::messages::{TxOpErr::*};
use std::error::Error;


impl From<TxOpErr> for E {
    // @todo/low Use a macro to auto generate these.
    // @todo/low Allow encode and decode of JSON without custom functions.
    // - At the moment data can only flow from Rust -> JSON with custom functions.
    fn from(e: TxOpErr) -> Self {
        // `p` == Parent node path.
        let mut p: Path = vec![];

        p.push("TxOp");
        match e {
            InvalidTxId => k(p, "InvalidTxId"),
            TxOpErr::ReturnStatus(rs) => ks(p, "ReturnStatus", rs),
            TxOpErr::BindRunError(bre) => bre_match(p, bre),
            ReadError(re) => {
                p.push("ReadError");
                match re {
                    ReadError::QueryIsWrite => k(p, "QueryIsWrite"),
                    ReadError::ReturnStatus(rs) => ks(p, "ReturnStatus", rs),
                }
            }
            ReadBindRunError(rbre) => {
                p.push("ReadBindRunError");
                match rbre {
                    ReadBindRunError::QueryIsWrite => k(p, "QueryIsWrite"),
                    ReadBindRunError::BindRunError(bre) => bre_match(p, bre)
                }
            }
            TxOpErr::WriteError(we) => {
                p.push("WriteError");
                match we {
                    WriteError::QueryIsRead => k(p, "QueryIsRead"),
                    WriteError::ReturnStatus(rs) => ks(p, "ReturnStatus", rs),
                }
            }
            TxOpErr::WriteBindRunError(wbre) => {
                p.push("WriteBindRunError");
                match wbre {
                    WriteBindRunError::QueryIsRead => k(p, "QueryIsRead"),
                    WriteBindRunError::BindRunError(bre) => bre_match(p, bre)
                }
            }
        }
    }
}


// @todo/low Split JSON out into its work workspace package (outside of runtime).
// - JSON package takes the runtime types and encode/decodes them.
//      - It is optional for the runtime package.


// @todo/low Question: Does the response/output need to be categorised as well?
// E.g. The requester has the context of what they asked, so no need to have a tree of enums until the <Result> is reached?
// Just different result types?
pub fn out_rs_to_json(o: &OutMsg) -> Result<String, SerdeError> {
    let t = |res| {
        let r = OkRes::new(res);
        serde_json::to_string(&r)
    };

    let f = |e| {
        let r = ErrRes::new(e);
        serde_json::to_string(&r)
    };


    match o {
        File(f_op_res) => match f_op_res {
            Ok(tx_id) => {
                t(Res::TxIdOnly(tx_id))
            }
            Err(e_orig) => {
                // Copy error to reduce lifetime syntax in code.
                let owned: FileOpErr = (*e_orig).clone();
                let e: E = owned.into();
                f(e)
            }
        },
        Tx(tx_op_res) => match tx_op_res {
            Ok(rset) => {
                t(Res::RSet(rset))
            }
            Err(e_orig) => {
                // Copy error to reduce lifetime syntax in code.
                let owned: TxOpErr = (*e_orig).clone();
                let e: E = owned.into();
                f(e)
            }
        }
    }
}