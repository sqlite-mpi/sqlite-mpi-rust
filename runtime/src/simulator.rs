/**
Simulates a host runtime interacting with guest FFI code.

- Terms.
    - Host
        - The runtime that includes the FFI.
        - Uses async/await keywords at the language level.
            - Event loop at the runtime level.
        - E.g. V8, CLR, Python 3.
    - Guest
        - FFI code.

- Typical interactions between host (H) and guest (G).
    - H sends a message to G over the FFI `send(msg)`.
    - `send(msg)` returns instantly with a request id.
    - H creates a map of request id => promise (still outstanding).
    - G actions the request, and calls H's `recv(reqId, msg)` callback with the response.
    - H resolves the promise with the msg.
        - `msg` can also be an error.
        - H's client code decides how to handle these.
            - In JS it would be a rejected promise.

- The reason for this simulator.
    - Its difficult to create a real (H, G) environment inside `cargo test`.
    - Using Rust threads seems easier to implement as the state can be moved between them.
        - Tests can pass or fail based on this state.
    - Multiple OS processes can be simulated using threads.
        - E.g. to test SQLITE_BUSY responses when the database file is locked by another process holding a write lock.
        - Designing a test case with precise timings between many simulated OS processes is easier with threads.

- Why Rust `async/await` is not used.
    - Futures are very new; most event loops are in a transition phase to the standard API.
        - Poor documentation.
        - Non obvious usage.
        - Debugging is difficult.
    - `async/await` keywords are not in the language.
        - Its best to wait 2 years until they become stable and usable.
    - Threads are well supported.
        - In the documentation, at the OS level, in DTrace.
        - Ownership makes them viable.
**/

use super::*;
use std::collections::HashMap;

use sqlite3ffi::stmt::{
    RSet,
    Rows,
    Val,
};
use sqlite3ffi::err::ReturnStatus;

use sma::fns::get_unique_id;

use crate::messages::*;
use crate::messages::InMsg::*;
use crate::messages::FileOp::*;
use crate::messages::TxOp::*;


pub fn get_tx_q(tx_id: &str, q: &str) -> InMsg {
    Tx(Q(ArgsTx {
        tx_id: tx_id.to_string(),
        q: q.to_string(),
    }))
}

pub fn get_tx_commit(tx_id: &str) -> InMsg {
    Tx(Commit(TxIdOnly {
        tx_id: tx_id.to_string()
    }))
}

pub fn get_tx_rollback(tx_id: &str) -> InMsg {
    Tx(Rollback(TxIdOnly {
        tx_id: tx_id.to_string()
    }))
}

pub fn commit(i: &I, o: &O, tx_id: &TxId) -> Option<OutMsg> {
    let i_id = i(get_tx_commit(tx_id));
    let msg_ids = &vec![&i_id];
    match o(msg_ids, ms(20)) {
        Some(mut hm) => hm.remove(&i_id),
        None => None
    }
}

pub fn q(i: &I, o: &O, tx_id: &TxId, q: &String) -> RSet {
    let i_id = i(get_tx_q(tx_id, q));

    match get_single(o, &i_id) {
        OutMsg::Tx(Ok(rset)) => rset,
        _ => unreachable!()
    }
}

// @see https://www.sqlite.org/pragma.html#pragma_integrity_check
pub fn integrity_check(f: String) -> bool {
    let rset = one_read(f, "PRAGMA integrity_check");
    if let RSet { num_rows: 1, num_cols: 1, rows: Rows { data, .. }, .. } = &rset {
        if let Val::String(s) = &data[0][0] {
            if s == "ok" {
                return true;
            }
        }
    }

    dbg!(("`PRAGMA integrity_check` failed", &rset));
    false
}

// @todo/medium Make this efficient for the host by allowing read/writes outside of read/write txs.
// - One message interaction instead of three (tx open, read, tx close).
// - Why start with tx? Initial focus is on clear semantics of SQLite lock mapping to async/await state.
pub fn one_read(f: String, q: &str) -> RSet {
    let (i, o) = get_new_runtime_async();
    let a = i(File(GetReadTx(ArgsFile { file: f.clone() })));
    if let OutMsg::File(Ok(TxIdOnly { tx_id })) = get_single(&o, &a) {
        let b = i(get_tx_q(&tx_id, &q));

        if let OutMsg::Tx(Ok(rset)) = get_single(&o, &b) {
            return rset;
        }
    }

    unreachable!();
}


pub fn get_response(o: &O, ids: &Vec<&InMsgId>) -> HashMapOutput {
    o(ids, ms(20)).unwrap()
}

// Gets a single response, fails otherwise.
pub fn get_single(o: &O, id: &InMsgId) -> OutMsg {
    dbg!(&id);

    match o(&vec![id], ms(20000000)) {
        Some(mut hm) => match hm.remove(id) {
            Some(out_msg) => out_msg,
            None => unreachable!(),
        },
        None => unreachable!(),
    }
}

pub fn get_tx_id(out_msg: OutMsg) -> TxId {
    match out_msg {
        OutMsg::File(Ok(TxIdOnly { tx_id })) => tx_id,
        x => {
            dbg!(x);
            unreachable!();
        }
    }
}


pub fn no_response(o: &O, ids: &Vec<&InMsgId>) {
    match o(ids, ms(20)) {
        None => {}
        Some(_) => assert!(false, "Input message ids should not have a response yet {:?}", ids)
    }
}

pub fn writes(i: &I, f: &FileAbs, n: u32) -> Vec<InMsgId> {
    let mut v = vec![];
    for _ in 0..n {
        v.push(i(File(GetWriteTx(ArgsFile { file: f.clone() }))));
    }
    v
}

pub fn reads(i: &I, f: &FileAbs, n: u32) -> Vec<InMsgId> {
    let mut v = vec![];
    for _ in 0..n {
        v.push(i(File(GetReadTx(ArgsFile { file: f.clone() }))));
    }
    v
}


// @todo/low Move to `impl Runtime`?
pub fn get_new_runtime() -> Box<dyn Fn(InMsg) -> OutMsg> {
    let (tx, rx) = channel();

    let output_fn = move |r| {
        tx.send(r).expect("Ok");
    };

    let rt = Runtime::new(output_fn);

    let get = move |i: InMsg| -> OutMsg {
        let with_id = InMsgWithId::new_gen_id(i);
        let id = with_id.id.clone();
        rt.input(with_id).expect("Ok");
        let out_with_id = rx.recv().expect("Ok"); // Note: `recv` blocks.
        let OutMsgWithId { in_msg_id, msg } = out_with_id;
        assert_eq!(id, in_msg_id);
        msg
    };

    Box::new(get)
}


pub type I = Box<dyn Fn(InMsg) -> InMsgId>;
pub type O = Box<dyn Fn(&Vec<&InMsgId>, Duration) -> Option<HashMap<InMsgId, OutMsg>>>;
pub type HashMapOutput = HashMap<InMsgId, OutMsg>;

// Allows passing many input messages without waiting for the responses.
// - Allow runtime to determine order, just like when used from a FFI/host event loop.
//      - Assert: Runtime manages queue/processing order correctly when it is handling many messages.
// - Responses are returned in a HashMap to allow ordering for matching: (get(k1), get(k2)) = (x, y)
pub fn get_new_runtime_async() -> (I, O) {
    let (tx, rx) = channel();
    let tx1 = tx.clone();

    let output_fn = move |r| {
        tx.send(r).expect("Ok");
    };

    let rt = Runtime::new(output_fn);

    let i = move |i: InMsg| -> InMsgId {
        let with_id = InMsgWithId::new_gen_id(i);
        let id = with_id.id.clone();
        rt.input(with_id).expect("Ok");
        id
    };

    // Waits for the given set of message ids.
    // - Each `recv` will wait for a max of `d` before failing the whole set.
    // - Returns `None` if all messages are not collected.
    // - Returns `Some` when all messages are collected.
    //      - Messages not in the target set are returned to the channel in both cases.
    let o = move |ids_ref: &Vec<&InMsgId>, d: Duration| -> Option<HashMapOutput> {
        let mut hm = HashMap::new();
        let mut to_return: Vec<OutMsgWithId> = vec![];
        let mut ids = ids_ref.clone();

        let r = loop {
            match rx.recv_timeout(d) {
                Ok(out_msg) => {
                    let OutMsgWithId { in_msg_id, msg } = &out_msg;

                    if ids.contains(&in_msg_id) {
                        let i = ids.iter().position(|x| **x == *in_msg_id).unwrap();
                        ids.remove(i);

                        hm.insert(out_msg.in_msg_id, out_msg.msg);

                        if ids.len() == 0 {
                            break Some(hm);
                        }
                    } else {
                        to_return.push(out_msg);
                    }
                }
                Err(e) => {
                    // `rx.recv_timeout` timeout reached.
                    // - Could not collect 100% of input ids.
                    break None;
                }
            }
        };

        // Return unused messages to the channel.
        // E.g. When only listening for a subset of output messages.
        for m in to_return.into_iter() {
            tx1.send(m).expect("Ok");
        }


        r
    };


    (
        Box::new(i),
        Box::new(o)
    )
}
