use std::mem;
use std::collections::VecDeque;
use std::collections::HashMap;

use std::thread;

use std::sync::mpsc::Sender;

use sma::{
    ReadTx,
    WriteTx,
};

use crate::messages::*;
use crate::runtime::{
    InMsgWithId,
    InMsgId,
    WtxReq,
    InputItem
};
use std::time::Duration;


pub type FileAbs = String;
type ReadTxsForFile = HashMap<TxId, ReadTx>;

#[derive(Debug)]
struct FileTxs {
    read_txs: ReadTxsForFile,
    pub write_queue: WriteQueue,
}


// Note: One active write tx per SQLite file.
#[derive(Debug)]
pub struct WriteQueue {
    state: QState,

    // Queue of input messages requesting a write tx.
    queue: VecDeque<WtxReq>,
}


pub struct ActiveWithNext {
    active: WriteTx,
    next: Option<WtxReq>,
}


impl WriteQueue {
    pub fn new() -> WriteQueue {
        WriteQueue {
            state: QState::Empty,
            queue: VecDeque::new(),
        }
    }

    // Returns the active `WriteTx`, and the next `WtxReq`
    // - The calling function should drop `WriteTx` to release the write lock, and then schedule the `WtxReq`.
    pub fn take_active(&mut self) -> Option<ActiveWithNext> {
        let next = self.queue.pop_front();

        let next_qstate = match &next {
            Some(wtx_req) => QState::NextRetry(wtx_req.id.clone()),
            None => QState::Empty
        };

        // Takes ownership of a value removed from the parent &mut struct.
        // @see https://stackoverflow.com/questions/28258548/cannot-move-out-of-borrowed-content-when-trying-to-transfer-ownership

        match mem::replace(&mut self.state, next_qstate) {
            QState::Active(owned) => {
                Some(ActiveWithNext {
                    active: owned,
                    next,
                })
            }

            // @todo/low Enforce calling `take_active` only when qstate is active.
            _ => None
        }
    }
}


// @todo/low generalise into `QState` trait? E.g. Where you have a queue, but the item being processed is not owned as it is in a channel or owned by a closure timer before going into a channel.
// Issue: Channels take ownership of items, so you cannot have a queue that owns items where the first item is "en route to being processed".
// Fix: Instead of using an `enum` to wrap an item which indicates its queue state (E.g. `Enum::Retry(item)`),
// record the current state *before* transferring item ownership to the queue. (Instead of *after* an `Enum::Retry(item)` is taken from the `rx` side of a channel).
// - The state of the queue can be determined at any time without collecting the entire channels items.
// - When receiving new items to queue, appropriate action can be determined without depending on items currently in the channel.
#[derive(Debug)]
pub enum QState {
    Active(WriteTx),

    // `InMsg` currently owned by a timer closure or event loop input channel; its en route to be processed.
    // - If in this state, add any write tx requests to the queue.
    NextRetry(InMsgId),

    // No active wtx or retry in process; Can attempt to move to the `ActiveWtx` state.
    Empty,
}


#[derive(Debug)]
pub struct ActiveTxs {
    txs: HashMap<FileAbs, FileTxs>,

    // @todo/low Replace with closure to allow any method?
    event_loop_in: Sender<InputItem>,
}

impl Drop for ActiveTxs {
    fn drop(&mut self) {
//        dbg!("at.d()");
    }
}



#[derive(Debug)]
pub enum RW<'a> {
    Read(&'a ReadTx),
    Write(&'a WriteTx),
}


#[derive(Debug)]
pub enum RWO {
    Read(ReadTx),
    Write(WriteTx),
}


impl ActiveTxs {
    pub fn new(event_loop_in: Sender<InputItem>) -> ActiveTxs {
        ActiveTxs {
            txs: HashMap::new(),
            event_loop_in,
        }
    }

    fn get_file_txs_mut(&mut self, f: &FileAbs) -> &mut FileTxs {
        self.txs.entry(f.clone()).or_insert_with(|| {
            FileTxs {
                read_txs: HashMap::new(),
                write_queue: WriteQueue::new(),
            }
        })
    }

    pub fn send_at(&self, in_msg: InMsgWithId, t: Duration) {
        let tx = self.event_loop_in.clone();

        thread::spawn(move || {
            println!("TID: {:?}, sleeping for: {:?}", thread::current().id(), t);
            thread::sleep(t);
            println!("TID: {:?}, sleep complete", thread::current().id());
            tx.send(InputItem::InMsgWithId(in_msg)).expect("Ok");
        });
    }


    // Get a `take-r` reference to a transaction.
    // - Transaction lives after it is used.
    // @todo/low Use Rusts ownership system to enforce one reference to be held at a time to prevent multiple queries being run out of order on the same transaction?
    pub fn get_tx(&self, tx_id: TxId) -> Option<RW> {

        // @todo/maybe Optimize lookup for server side use of 1000's of db files.
        for (_, txs) in self.txs.iter() {
            if let Some(rt) = txs.read_txs.get(&tx_id) {
                return Some(RW::Read(rt));
            }

            if let QState::Active(wt) = &txs.write_queue.state {
                if wt.id == tx_id {
                    return Some(RW::Write(wt));
                }
            }
        }

        None
    }

    pub fn wr_qstate(&mut self, f: &FileAbs) -> &QState {
        &self.get_file_txs_mut(f).write_queue.state
    }

    pub fn wr_queue(&mut self, f: &FileAbs, wtx_req: WtxReq) {
        &self.get_file_txs_mut(f).write_queue.queue.push_back(wtx_req);
    }


    pub fn wr_active(&mut self, f: &FileAbs, wtx: WriteTx) {
        let wq = &mut self.get_file_txs_mut(f).write_queue;

        let next = QState::Active(wtx);
        let prev = mem::replace(&mut wq.state, next);

        // @todo/low Also assert prev != NextRetry(id) if id !== next_in_queue_id
        // - Encode these state transitions with the Rust ownership system? See per file db/tx. E.g. `Enum(s) => s.fn()`
        match prev {
            QState::Active(_) => assert!(false, "Write Tx should be closed before starting another."),
            _ => {}
        }
    }

    pub fn wr_retry(&mut self, f: &FileAbs, wtx_req: WtxReq, t: Duration) {
        let wq = &mut self.get_file_txs_mut(f).write_queue;

        let next = QState::NextRetry(wtx_req.id.clone());
        let prev = mem::replace(&mut wq.state, next);

        match prev {
            QState::NextRetry(ref id) if *id == wtx_req.id => {}
            QState::Empty => {}
            _ => assert!(false, "To transition to `NextRetry(x)`, current state must be `(NextRetry(x) | Empty)`")
        }

        self.send_at(wtx_req.into(), t);
    }

    pub fn wr_fail(&mut self, f: &FileAbs, wtx_req: WtxReq) {
        let wq = &mut self.get_file_txs_mut(f).write_queue;


        match wq.queue.pop_front() {
            Some(wtx_req) => {
                let next = QState::NextRetry(wtx_req.id.clone());
                mem::replace(&mut wq.state, next);
                self.event_loop_in.send(InputItem::InMsgWithId(wtx_req.into())).expect("Ok");
            }
            None => {
                mem::replace(&mut wq.state, QState::Empty);
            }
        };
    }


    // Removes a transaction from `ActiveTxs` so that `commit` or `rollback` can be completed.
    // @todo/low What happens if an error occurs whilst committing, should the tx still be kept alive for a retry?
    // @todo/important `RAISE(ROLLBACK)` in a q/read/write op needs to drop all resources and return correct error code.
    // @see https://sqlite.org/lang_createtrigger.html
    pub fn remove_tx(&mut self, tx_id: TxId) -> Option<RWO> {

        // @todo/maybe Optimize lookup for server side use of 1000's of db files.
        for (_, txs) in self.txs.iter_mut() {
            if let Some(rt) = txs.read_txs.remove(&tx_id) {
                return Some(RWO::Read(rt));
            }

            match &txs.write_queue.state {
                QState::Active(wt) if wt.id == tx_id => {
                    match txs.write_queue.take_active() {
                        Some(ActiveWithNext { active, next }) => {

                            // @todo/important enforce this by placing the `event_loop.send` into a closure which is called *after* the active wtx is closed.
                            // Assumption: before the next `wtx_req` gets to the start of the event loop,
                            // the active `WriteTx` returned here would of been dropped (freeing the SQLite write lock).
                            // Issue: this trusts the client of the API to not move the active wtx in another queue and allow the event loop to continue.
                            if let Some(wtx_req) = next {
                                self.event_loop_in.send(InputItem::InMsgWithId(wtx_req.into())).expect("Ok");
                            }

                            return Some(RWO::Write(active));
                        }
                        None => {
                            assert!(false, "take_active should only be called when qstate is active.");
                        }
                    }
                }
                _ => {}
            }
        }

        None
    }


    pub fn add_read(&mut self, f: &FileAbs, tx_id: TxId, read_tx: ReadTx) {
        let file_txs = self.get_file_txs_mut(f);
        let exists = file_txs.read_txs.contains_key(&tx_id);

        // Client code should ensure `add` is called once per Id.
        assert!(!exists);
        file_txs.read_txs.insert(tx_id.clone(), read_tx);
    }
}
