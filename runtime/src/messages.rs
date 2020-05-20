use serde::{Deserialize, Serialize};

use sqlite3ffi::stmt::RSet;

use sqlite3ffi::err::ReturnStatus;
use sqlite3ffi::db::BindRunError;

use sma::{
    Params,
    ReadError,
    ReadBindRunError,
    WriteError,
    WriteBindRunError,
};

// Question: How do you know which responses are compatible with which requests using just the pub type system?
// - Is it possible to encode `reqRes = (pub enum_variant_a, pub enum_variant_b)`?
// - Just use functions that return a particular pub type?



pub type TxId = String;
pub type FilePath = String;
pub type Query = String;

#[derive(Debug)]
pub enum Message {
    In(InMsg),
    Out(OutMsg),
}


#[derive(Debug)]
#[derive(PartialEq)]
pub enum InMsg {
    File(FileOp),
    Tx(TxOp),
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum FileOp {
    GetReadTx(ArgsFile),
    GetWriteTx(ArgsFile),
}

// @todo/low Use `enum` to categorise op into (read, write, r_or_w, end)?
// Issue: requires runtime state to determine if its valid (e.g. used within a read or write tx).
// - Quesiton: At the JSON level should the read/write tx ops be different types?
#[derive(Debug)]
#[derive(PartialEq)]
pub enum TxOp {
    Q(ArgsTx),
    Read(ArgsTx),
    Write(ArgsTx),
    QParams(ArgsTxParams),
    ReadParams(ArgsTxParams),
    WriteParams(ArgsTxParams),
    Commit(TxIdOnly),
    Rollback(TxIdOnly),
}

use TxOp::*;

impl TxOp {
    pub fn get_tx_id(&self) -> TxId {
        let tx_id = match &self {
            Q(a) | Read(a) | Write(a) => &a.tx_id,
            QParams(a) | ReadParams(a) | WriteParams(a) => &a.tx_id,
            Commit(a) | Rollback(a) => &a.tx_id
        };
        tx_id.clone()
    }

    pub fn get_type(&self) -> TxOpType {
        match &self {
            Read(_) | ReadParams(_) => TxOpType::Read,
            Write(_) | WriteParams(_) => TxOpType::Write,
            Q(_) | QParams(_) => TxOpType::Q,
            Commit(a) | Rollback(a) => TxOpType::End
        }
    }
}

pub enum TxOpType {
    Read,
    Write,
    // Could be either read write; client did not categorise and does not care about enforcing query string matches read/write.
    Q,
    End,
}


#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct ArgsFile {
    pub file: FilePath
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct ArgsTx {
    pub tx_id: TxId,
    pub q: Query,
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct ArgsTxParams {
    pub tx_id: TxId,
    pub q: Query,

    #[serde(flatten)]
    pub params: Params,
}


// @todo/maybe Use pub enum tree paths to categorise response pub types so that the tree paths match the request pub types?
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub enum OutMsg {
    File(FileOpRes),
    Tx(TxOpRes),
//    RSet(RSetRes),
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
#[derive(Serialize, Deserialize)]
pub struct TxIdOnly {
    pub tx_id: TxId,
}


pub type FileOpRes = Result<TxIdOnly, FileOpErr>;
pub type TxOpRes = Result<RSet, TxOpErr>;
//pub type RSetRes = std::result::Result<RSet, RSetErr>;


#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub enum FileOpErr {
    FileDirectoryDoesNotExist,
    ReturnStatus(ReturnStatus),
}

// @todo/medium General error type with fields (kind, message, meta) for JSON-like response.

#[derive(Debug)]
//#[derive(PartialEq)]
#[derive(Serialize, Deserialize)]
#[derive(Clone)]
pub enum TxOpErr {
    InvalidTxId,

    // WHEN: write tx, q().
    // @todo/important Note: This is not the return status for all of the other calls; *only* for write_tx.q(), commit and rollback.
    // - Make a general return error message.
    ReturnStatus(ReturnStatus),
    // When: write tx, q_params()
    BindRunError(BindRunError),

    ReadError(ReadError),
    ReadBindRunError(ReadBindRunError),

    WriteError(WriteError),
    WriteBindRunError(WriteBindRunError),
}


#[derive(Debug)]
#[derive(PartialEq)]
pub struct RSetErr;


fn example() {
    use self::InMsg::*;
    use self::FileOp::*;

    let a = Message::In(
        File(
            GetReadTx(
                ArgsFile {
                    file: "a/b/c.sqlite".to_string()
                }
            )
        )
    );

    match a {
        Message::In(File(r)) => {
            match r {
                GetReadTx(args) => {
                    dbg!(args.file);
                }
                _ => ()
            }
        }
        _ => ()
    }
}