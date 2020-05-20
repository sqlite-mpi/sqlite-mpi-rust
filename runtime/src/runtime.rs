#![allow(warnings)]

use crate::*;

use std::fs;
use std::path::Path;


use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{
    Sender,
    Receiver,
};


use crate::messages::*;
use crate::messages::InMsg::*;
use crate::messages::FileOp::*;
use crate::messages::TxOp::*;

use crate::active_txs::*;

use crate::utils::is_valid_uuid_v4_hypenated;

use sma::{
    fns::get_unique_id,
    DbFile,
    ReadError,
};

use sqlite3ffi::errmap::PrimaryRC;

pub type InMsgId = String;


#[derive(Debug)]
#[derive(PartialEq)]
pub struct InMsgWithId {
    pub id: InMsgId,
    pub msg: InMsg,
}

impl InMsgWithId {
    pub fn new_gen_id(msg: InMsg) -> InMsgWithId {
        InMsgWithId {
            id: get_unique_id(),
            msg
        }
    }
}


// @todo/medium Only store `FileAbs` instead of `FilePath`, so they do not need to be passed around together in function signatures.
#[derive(Debug)]
pub struct WtxReq {
    pub id: InMsgId,
    pub args: ArgsFile,
}


// Allow going from specific to general as it cannot fail.
// - Do not auto convert from general to specific as it can fail.
//      - Instead: Use `match` arms to manually construct types.
// - Specific types are useful as enum variants cannot be referenced in type signatures.
// `wtx.into()` can be used as input to any type signature with `InMsgWithId`.
impl From<WtxReq> for InMsgWithId {
    fn from(f: WtxReq) -> Self {
        let WtxReq { id, args } = f;
        InMsgWithId {
            id,
            msg: File(GetWriteTx(args)),
        }
    }
}


#[derive(Debug)]
pub struct OutMsgWithId {
    pub in_msg_id: InMsgId,
    pub msg: OutMsg,
}


#[derive(Debug)]
pub struct Runtime {
    tx: Sender<InputItem>,
    thread_handle: Option<thread::JoinHandle<()>>,
}


#[derive(Debug)]
pub enum InputError {
    ChannelError,
    InvalidId
}

//fn ms(i: u64) -> Duration {
//    Duration::from_millis(i)
//}


// A `Runtime` instance is intended to be owned by FFI host process.
// - Assumption: When an FFI function is called from the host, it runs sync with the host.
//      - `Runtime` becomes the interface to the threads that are doing the actual SQLite FFI.
//      - SQLite FFI functions run sync, and should not block the host.
impl Runtime {
    pub fn new<F>(output_fn: F) -> Runtime
        where F: Fn(OutMsgWithId) + Send + 'static {
//        println!("FG: {:?}", thread::current().id());

        let (tx, rx): (Sender<InputItem>, Receiver<InputItem>) = mpsc::channel();

        // @todo/next keep index of outstanding requests; do not allow two ids.
        Runtime {
            tx: tx.clone(),
            thread_handle: Some(start_thread(tx.clone(), rx, output_fn)),
        }
    }


    pub fn input(&self, msg: InMsgWithId) -> Result<(), InputError> {
        match is_valid_uuid_v4_hypenated(&msg.id) {
            Ok(_) => {},
            Err(e) => {
                dbg!(e);
                println!("Invalid input message ID, must be hyphenated v4 uuid.");
                return Err(InputError::InvalidId);
            }
        }

        let to_bg = &self.tx.send(InputItem::InMsgWithId(msg));

        if let Err(send_err) = to_bg {
            // @todo/low log this error in production release (and others in code base).
            dbg!(send_err);
            return Err(InputError::ChannelError);
        }

        Ok(())
    }
}


pub enum InputItem {
    InMsgWithId(InMsgWithId),
    BreakLoop,
}


impl Drop for Runtime {
    fn drop(&mut self) {

        // Issue: When the scope that owns the runtime drops it, the event loop thread is implicitly detached and does not `drop` all its resources.
        // - When read/write tx's are open, the SQLite db handles are not closed, leaving a `-wal` and `-shm` file next to the database file.
        // Fix: When `Runtime` is dropped, break out of loop in thread and wait for it to end.

        // Alternatives:
        // - Instead of letting the child thread own a `tx` clone that prevents breaking out of the `rx` blocking loop by dropping all `tx`s for the channel.
        //      - Use an external and internal queue; the child thread can send input it its self by adding to its own internal queue (E.g. a VecDeque or Binary heap with a mutex).
        //      - This would prevent having to have `InputItem::BreakLoop`.

        // @todo/medium
        // Questions:
        // - What happens to all messages currently in the input or output queues of the thread event loop?

        // Note: Should not join in `drop` (but there seems no alternative).
        // - @see https://stackoverflow.com/questions/41331577/joining-a-thread-in-a-method-that-takes-mut-self-like-drop-results-in-cann
        &self.tx.send(InputItem::BreakLoop).expect("Ok");
        &self.thread_handle.take().expect("Ok").join();
    }
}


// @todo/medium Ensure all threads exits when `Runtime` is dropped.
// @see evernote:///view/14186947/s134/b66753bd-d37c-4b40-8b10-6a57de70e765/b66753bd-d37c-4b40-8b10-6a57de70e765/
// @todo/medium Auto rollback if no interaction.
fn start_thread<F>(tx: Sender<InputItem>, rx: Receiver<InputItem>, output_fn: F) -> thread::JoinHandle<()> where F: Fn(OutMsgWithId) + Send + 'static {
    let t = thread::Builder::new().name("sqlite-mpi.bg".to_string());

    return t.spawn(move || {
//        println!("BG: {:?}", thread::current().id());

        // @see https://stackoverflow.com/questions/57578601/how-to-wait-on-multiple-mpsc-channels-with-different-priorities-to-create-an-ord
        let mut at = ActiveTxs::new(tx);

        // @todo/low Use binary heap with custom sort to order incoming messages (W.O, W, R.O, R).
        //  - Writes take priority.
        // @see https://doc.rust-lang.org/rust-by-example/flow_control/for.html#for-and-iterators
        for in_item in rx {
            match in_item {
                InputItem::InMsgWithId(in_msg) => {
                    match in_msg.msg {
                        File(GetWriteTx(args)) => {
                            let wtx = WtxReq {
                                id: in_msg.id,
                                args,
                            };

                            maybe_queue_write_tx_req(&mut at, wtx, &output_fn);
                        }
                        _ => {
                            immediate_response(&mut at, in_msg, &output_fn)
                        }
                    }
                }
                InputItem::BreakLoop => break
            }
        }
    }).unwrap();
}

fn file_err(e: FileOpErr) -> OutMsg {
    OutMsg::File(Err(e))
}

fn tx_err(e: TxOpErr) -> OutMsg {
    OutMsg::Tx(Err(e))
}

fn maybe_queue_write_tx_req<F>(at: &mut ActiveTxs, wtx_req: WtxReq, output_fn: &F) where F: Fn(OutMsgWithId) + Send + 'static {
    let id = wtx_req.id.clone();
    let file = &wtx_req.args.file;

    let file_op_res = process_write_req(at, wtx_req);

    // If not queued, respond.
    if let Some(res) = file_op_res {
        let out_with_id = OutMsgWithId {
            in_msg_id: id,
            msg: OutMsg::File(res),
        };
        output_fn(out_with_id);
    } else {
//        println!("Write tx request {} was queued.", id);
    }
}

// @todo/maybe Use an enum to wrap `InMsg` when queued in the event loop input queue.
// - E.g. enum = (Retry|Queued|FreshWrite)
// - Allow determining what to do with a message without looking up state in other structs.
//      - State is stored in a single queue/struct, instead of many different structs per type of `InMsg` action.
// - Issue: state is owned by the channel so it would not be possible to determine the current system state without collecting all messages in the channel.
//fn process_write_request(at: &mut ActiveTxs, get_wtx: GetWriteTxWithId) -> Option<FileOpRes> {
//
//}


fn process_write_req(at: &mut ActiveTxs, wtx_req: WtxReq) -> Option<FileOpRes> {
    use QState::*;

    let WtxReq { id, args } = &wtx_req;
    let file_path = &args.file;


    let f_abs = match get_file_abs(file_path) {
        Ok(f_abs) => f_abs,
        Err(e) => return Some(Err(e))
    };

    match at.wr_qstate(&f_abs) {
        NextRetry(in_msg_id) if *id == *in_msg_id => process_wtx_req(at, wtx_req, &f_abs),
        Empty => process_wtx_req(at, wtx_req, &f_abs),
        Active(_) | NextRetry(_) => {
            at.wr_queue(&f_abs, wtx_req);
            None
        }
    }
}


// `Some` = Respond
// `None` = Delay response.
//      - When: external process has a write lock causing SQLITE_BUSY.
//          - Try and get a write lock again after a time delay.
fn process_wtx_req(at: &mut ActiveTxs, wtx_req: WtxReq, f_abs: &FileAbs) -> Option<FileOpRes> {

    // @todo/low Make sure all `DbHandle::new` calls use `FileAbs` instead of `FilePath` (contains relative components; exact copy of API message input).
    let f_res = get_file(&f_abs);

    match f_res {
        Ok(f) => {
            match f.get_write_tx() {
                Ok(wtx) => {
                    let tx_id = wtx.id.clone();
                    at.wr_active(f_abs, wtx);
                    Some(Ok(TxIdOnly { tx_id }))
                }
                Err((_, rs)) => {
                    match rs.primary.id {
                        PrimaryRC::SQLITE_BUSY => {


                            // @todo/next Only retry a set amount of times: `if limit reached then wr_fail()`.
                            // @todo/important Cancel write tx's with no interactions after a certain time period.

                            let d = ms(2000);
                            at.wr_retry(f_abs, wtx_req, d);


                            None
                        }
                        _ => {
                            dbg!(("Unknown error when processing wtx request", &rs));
                            at.wr_fail(f_abs, wtx_req);
                            Some(Err(FileOpErr::ReturnStatus(rs)))
                        }
                    }
                }
            }
        }
        Err(e) => {
            // @todo/next Its possible to get a SQLITE_BUSY when opening the file because it will try to read pragmas which could return busy.
            // Question: Why does a read return SQLITE_BUSY? Should'nt reads all be allowed in WAL mode?

            dbg!((">>>>> Error initing a file handle.", &e));
            Some(Err(e))
        }
    }
}


fn immediate_response<F>(at: &mut ActiveTxs, in_msg: InMsgWithId, output_fn: &F) where F: Fn(OutMsgWithId) + Send + 'static {
    let InMsgWithId { id, msg } = in_msg;

    let out_msg = get_res(at, &msg);

    let out_with_id = OutMsgWithId {
        in_msg_id: id,
        msg: out_msg,
    };
    output_fn(out_with_id);
}


fn get_res(at: &mut ActiveTxs, i: &InMsg) -> OutMsg {

    // @todo/next Store list of transactions, write queue. See `del_tx_data.json`.

    match i {
        File(op) => {
            let o = process_file_op(at, &op);
            return OutMsg::File(o);
        }
        Tx(op) => {
            // @todo/low Use traits for the same functions on read/write txs?

            let o = match op.get_type() {
                TxOpType::End => process_tx_op_end(at, &op),
                _ => process_tx_op_body(at, &op)
            };
            return OutMsg::Tx(o);
        }
    }
}


fn process_file_op(at: &mut ActiveTxs, op: &FileOp) -> FileOpRes {
    match op {
        GetReadTx(args) => {
            let f = get_file(&args.file)?;
            let f_abs = f.get_file_abs();

            match f.get_read_tx() {
                Ok(tx) => {
                    let res = Ok(TxIdOnly { tx_id: tx.id.clone() });
                    at.add_read(&f_abs, tx.id.clone(), tx);
                    return res;
                }
                Err((f, rs)) => return Err(FileOpErr::ReturnStatus(rs))
            }
        }
        GetWriteTx(args) => {
            // @todo/next remove this?
            // If active write, add to end of queue
            // Else begin

            let f = get_file(&args.file)?;
            let f_abs = f.get_file_abs();

            match f.get_write_tx() {
                Ok(tx) => {
                    let res = Ok(TxIdOnly { tx_id: tx.id.clone() });
                    // @todo/next queue writes requests if one is currently active.
//                    at.add_read(f_abs, tx.id.clone(), tx);
                    return res;
                }
                Err((f, rs)) => return Err(FileOpErr::ReturnStatus(rs))
            }
        }
    }
}

fn get_file_abs(f_path: &FilePath) -> Result<FileAbs, FileOpErr> {
    match normalize_file_path(f_path) {
        Some(s) => Ok(s),
        None => Err(FileOpErr::FileDirectoryDoesNotExist)
    }
}


fn get_file(f_path: &FilePath) -> Result<DbFile, FileOpErr> {
    let file_abs = get_file_abs(f_path)?;

    match DbFile::new(file_abs.to_string()) {
        Ok(f) => Ok(f),
        Err(rs) => Err(FileOpErr::ReturnStatus(rs))
    }
}


// Any operation that is done inside of a transaction where the tx is still active afterwards.
fn process_tx_op_body(at: &ActiveTxs, op: &TxOp) -> TxOpRes {
    match get_tx(at, op) {
        Err(e) => return Err(e),
        Ok(t) => match t {
            RW::Read(rtx) => {
                match op {
                    Read(a) | Q(a) => {
                        match rtx.q(&a.q) {
                            Err(e) => return Err(TxOpErr::ReadError(e)),
                            Ok(rs) => return Ok(rs)
                        }
                    }
                    ReadParams(a) | QParams(a) => {
                        match rtx.q_params(&a.q, &a.params) {
                            Err(e) => return Err(TxOpErr::ReadBindRunError(e)),
                            Ok(rs) => return Ok(rs)
                        }
                    }
                    Write(_) | WriteParams(_) => {
                        return Err(TxOpErr::ReadError(ReadError::QueryIsWrite));
                    }
                    _ => {
                        // @todo/low Use traits instead of categories/enums.
                        assert!(false, "Commit or Rollback not possible");
                        return Err(TxOpErr::InvalidTxId);
                    }
                }
            }
            RW::Write(wtx) => {
                match op {
                    Q(a) => {
                        match wtx.q(&a.q) {
                            Err(e) => return Err(TxOpErr::ReturnStatus(e)),
                            Ok(rs) => return Ok(rs)
                        }
                    }
                    QParams(a) => {
                        match wtx.q_params(&a.q, &a.params) {
                            Err(e) => return Err(TxOpErr::BindRunError(e)),
                            Ok(rs) => return Ok(rs)
                        }
                    }
                    Read(a) => {
                        match wtx.read(&a.q) {
                            Err(e) => return Err(TxOpErr::ReadError(e)),
                            Ok(rs) => return Ok(rs)
                        }
                    }
                    ReadParams(a) => {
                        match wtx.read_params(&a.q, &a.params) {
                            Err(e) => return Err(TxOpErr::ReadBindRunError(e)),
                            Ok(rs) => return Ok(rs)
                        }
                    }
                    Write(a) => {
                        match wtx.write(&a.q) {
                            Err(e) => return Err(TxOpErr::WriteError(e)),
                            Ok(rs) => return Ok(rs)
                        }
                    }
                    WriteParams(a) => {
                        match wtx.write_params(&a.q, &a.params) {
                            Err(e) => return Err(TxOpErr::WriteBindRunError(e)),
                            Ok(rs) => return Ok(rs)
                        }
                    }
                    _ => {
                        assert!(false, "Commit or Rollback not possible");
                        return Err(TxOpErr::InvalidTxId);
                    }
                }
            }
        }
    }
}

// `commit` or `rollback`
// Note: This removes the tx from the `ActiveTxs` so requires a mutable reference.
fn process_tx_op_end(at: &mut ActiveTxs, op: &TxOp) -> TxOpRes {
    match remove_tx(at, &op) {
        Err(e) => return Err(e),
        Ok(t) => match t {
            RWO::Read(rtx) => {
                match op {
                    Commit(_) => {
                        match rtx.commit() {
                            (_, Err(e)) => return Err(TxOpErr::ReturnStatus(e)),
                            (_, Ok(rs)) => return Ok(rs)
                        }
                    }
                    Rollback(_) => {
                        match rtx.rollback() {
                            (_, Err(e)) => return Err(TxOpErr::ReturnStatus(e)),
                            (_, Ok(rs)) => return Ok(rs)
                        }
                    }
                    _ => {
                        assert!(false, "Read, Write or Q not possible");
                        return Err(TxOpErr::InvalidTxId);
                    }
                }
            }
            RWO::Write(wtx) => {
                match op {
                    Commit(_) => {
                        match wtx.commit() {
                            (_, Err(e)) => return Err(TxOpErr::ReturnStatus(e)),
                            (_, Ok(rs)) => return Ok(rs)
                        }
                    }
                    Rollback(_) => {
                        match wtx.rollback() {
                            (_, Err(e)) => return Err(TxOpErr::ReturnStatus(e)),
                            (_, Ok(rs)) => return Ok(rs)
                        }
                    }
                    _ => {
                        assert!(false, "Read, Write or Q not possible");
                        return Err(TxOpErr::InvalidTxId);
                    }
                }
            }
        }
    }
}


fn get_tx<'a>(at: &'a ActiveTxs, tx_op: &TxOp) -> Result<RW<'a>, TxOpErr> {
    let tx_id = tx_op.get_tx_id();

    match at.get_tx(tx_id) {
        Some(rw) => Ok(rw),
        None => Err(TxOpErr::InvalidTxId)
    }
}

fn remove_tx(at: &mut ActiveTxs, tx_op: &TxOp) -> Result<RWO, TxOpErr> {
    let tx_id = tx_op.get_tx_id();

    match at.remove_tx(tx_id) {
        Some(rw) => Ok(rw),
        None => Err(TxOpErr::InvalidTxId)
    }
}


// Expects a path to an existing directory, but the file does not have to exist.
fn normalize_file_path(f: &String) -> Option<String> {
    let full = Path::new(&f);

    let dir_file = (
        full.parent(),
        full.file_name()
    );

    if let (Some(dir), Some(file)) = dir_file {

        // SQLite FFI will create a file, but will not create directories.
        if !full.is_dir() && dir.is_dir() {
            let normal_dir = fs::canonicalize(dir).expect("Ok");
            return Some(normal_dir.join(file).to_str().unwrap().to_string());
        }
    }

    None
}
